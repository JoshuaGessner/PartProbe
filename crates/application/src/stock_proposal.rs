use partprobe_domain::{
    ActorId, DensityKilogramsPerCubicMillimeter, Money, RecordedAt, ShopResourceCatalog,
    ShopSettingsDraft, ValueState,
};
use partprobe_geometry_import::ControlledGeometryResult;
use partprobe_setup_planner::{
    DeveloperEstimateInputProposal, ExactStepEnvelopeEvidence, RectangularStockProposal,
    propose_developer_estimate_inputs, propose_rectangular_stock,
};

use crate::{
    AnalyzedGeometryEvidence, DraftBaseCostInputs, DraftEstimateInputs, DraftMaterialCostInputs,
    DraftOperationCostInputs, DraftQuantityInputs, DraftStockInputs, DraftTimeInputs,
};
use partprobe_estimation_engine::make_quantity;
use rust_decimal::Decimal;

/// Stable identity of explicit session-only developer proposal adoption.
pub const DEVELOPER_ESTIMATE_PROPOSAL_ADOPTION_RULE_ID: &str =
    "partprobe-developer-estimate-proposal-adoption";
/// Exact behavior version of the coarse exclusion/adoption mapping.
pub const DEVELOPER_ESTIMATE_PROPOSAL_ADOPTION_RULE_VERSION: (u16, u16, u16) = (1, 0, 0);

/// Application-owned, read-only resolution of a reviewable exact-STEP stock proposal.
#[derive(Clone, Copy, Debug, Default)]
pub struct ExactStepStockProposalApplication;

impl ExactStepStockProposalApplication {
    /// Resolves the catalog's exact active selection against controlled geometry evidence.
    ///
    /// This boundary has no mutation, adoption, pricing, purchasing, or estimate authority.
    #[must_use]
    pub fn propose(
        &self,
        geometry: &AnalyzedGeometryEvidence,
        catalog: &ShopResourceCatalog,
    ) -> ValueState<RectangularStockProposal> {
        let analysis = match &geometry.result {
            ControlledGeometryResult::ExactBrepWithEnvelope(analysis) => analysis,
            ControlledGeometryResult::ExactBrep(_) => {
                return ValueState::Unavailable {
                    reason: "exact STEP envelope evidence is unavailable".to_owned(),
                };
            }
            ControlledGeometryResult::Mesh(_) => {
                return ValueState::Unavailable {
                    reason: "mesh geometry is not authorized for stock proposals".to_owned(),
                };
            }
        };
        let Some(selection) = catalog.active_selection() else {
            return ValueState::Unavailable {
                reason: "no resource selection is active for proposals".to_owned(),
            };
        };
        let Some(allowance) = catalog.stock_allowances().iter().find(|record| {
            record.id() == selection.stock_allowance_id()
                && record.version() == selection.stock_allowance_version()
        }) else {
            return ValueState::Blocked {
                reason: "active resource selection does not resolve its stock allowance".to_owned(),
            };
        };
        let evidence = match ExactStepEnvelopeEvidence::from_controlled_analysis(
            analysis,
            geometry.output_hash.clone(),
        ) {
            Ok(evidence) => evidence,
            Err(error) => {
                return ValueState::Blocked {
                    reason: format!("exact STEP envelope evidence is invalid: {error}"),
                };
            }
        };
        match propose_rectangular_stock(&evidence, allowance, selection) {
            Ok(proposal) => ValueState::available(proposal),
            Err(error) => ValueState::Blocked {
                reason: format!("active resource selection cannot produce stock: {error}"),
            },
        }
    }
}

/// Application-owned resolution of session-only developer estimate-input proposals.
#[derive(Clone, Copy, Debug, Default)]
pub struct DeveloperEstimateProposalApplication;

impl DeveloperEstimateProposalApplication {
    /// Resolves exact STEP evidence against the persisted starter resource-library draft.
    ///
    /// The returned proposal is non-authoritative `NeedsReview` evidence. This service does not
    /// mutate [`crate::DraftEstimateSession`], adopt values, or fall back to numeric defaults.
    #[must_use]
    pub fn propose(
        &self,
        geometry: &AnalyzedGeometryEvidence,
        settings: &ShopSettingsDraft,
    ) -> ValueState<DeveloperEstimateInputProposal> {
        let analysis = match &geometry.result {
            ControlledGeometryResult::ExactBrepWithEnvelope(analysis) => analysis,
            ControlledGeometryResult::ExactBrep(_) => {
                return ValueState::Unavailable {
                    reason: "exact STEP envelope evidence is unavailable".to_owned(),
                };
            }
            ControlledGeometryResult::Mesh(_) => {
                return ValueState::Unavailable {
                    reason: "mesh geometry is not authorized for estimate-input proposals"
                        .to_owned(),
                };
            }
        };
        let Some(library) = settings.resource_library() else {
            return ValueState::Unavailable {
                reason: "persisted starter resource-library draft is unavailable".to_owned(),
            };
        };
        let evidence = match ExactStepEnvelopeEvidence::from_controlled_analysis(
            analysis,
            geometry.output_hash.clone(),
        ) {
            Ok(evidence) => evidence,
            Err(error) => {
                return ValueState::Blocked {
                    reason: format!("exact STEP envelope evidence is invalid: {error}"),
                };
            }
        };
        match propose_developer_estimate_inputs(&evidence, library) {
            Ok(proposal) => ValueState::available(proposal),
            Err(error) => ValueState::Blocked {
                reason: format!("developer estimate-input proposal is blocked: {error}"),
            },
        }
    }

    /// Explicitly adopts one already-reviewed proposal into a session-only coarse estimate input
    /// set. All cost categories outside stock material and coarse machine/runtime evidence are
    /// excluded only after the caller records that those limitations were reviewed.
    pub fn adopt(
        &self,
        proposal: DeveloperEstimateInputProposal,
        quantities: DraftQuantityInputs,
        review: DeveloperEstimateProposalReview,
    ) -> Result<AdoptedDeveloperEstimateInputs, DeveloperEstimateProposalAdoptionError> {
        if !review.proposal_values_reviewed {
            return Err(DeveloperEstimateProposalAdoptionError::ProposalNotReviewed);
        }
        if !review.coarse_limitations_accepted {
            return Err(DeveloperEstimateProposalAdoptionError::LimitationsNotAccepted);
        }
        if review.reason.trim().is_empty() || review.reason.len() > 1_024 {
            return Err(DeveloperEstimateProposalAdoptionError::InvalidReviewReason);
        }
        let make = make_quantity(
            quantities.deliver,
            quantities.planned_spares,
            quantities.destructive_samples,
        )
        .map_err(|_| DeveloperEstimateProposalAdoptionError::ArithmeticFailure)?;
        let purchased = proposal
            .unit_stock_material_cost()
            .checked_mul(Decimal::from(make.value()))
            .map_err(|_| DeveloperEstimateProposalAdoptionError::ArithmeticFailure)?;
        let density = DensityKilogramsPerCubicMillimeter::new(
            proposal.material_density_kg_per_m3().value() / Decimal::from(1_000_000_000_u64),
        )
        .map_err(|_| DeveloperEstimateProposalAdoptionError::ArithmeticFailure)?;
        let currency = proposal.unit_stock_material_cost().currency().clone();
        let zero = || Money::new(Decimal::ZERO, currency.clone());
        let times = proposal.coarse_times();
        let inputs = DraftEstimateInputs {
            stock: DraftStockInputs {
                stock_volume: proposal.blank_volume_mm3(),
                density,
            },
            quantities,
            times: DraftTimeInputs {
                setup_hours: minutes_to_hours(times.setup().value()),
                programming_hours: minutes_to_hours(times.programming().value()),
                cutting_hours_per_item: minutes_to_hours(times.cutting().value()),
                non_cutting_hours_per_item: Decimal::ZERO,
                load_unload_hours_per_item: minutes_to_hours(times.load_unload().value()),
                in_cycle_inspection_hours_per_item: Decimal::ZERO,
                quality_inspection_hours: minutes_to_hours(times.quality().value()),
            },
            material: DraftMaterialCostInputs {
                purchased,
                cut: zero(),
                certificate: zero(),
                inbound_freight: zero(),
                approved_remnant_credit: zero(),
            },
            operation: DraftOperationCostInputs {
                prove_out: zero(),
                tooling: zero(),
                consumables: zero(),
                fixture: zero(),
                outside: zero(),
                freight: zero(),
            },
            base: DraftBaseCostInputs {
                nonrecurring_engineering: zero(),
                administration: zero(),
                overhead: zero(),
                accepted_risk_impacts: vec![zero()],
                expected_rework: zero(),
            },
        };
        Ok(AdoptedDeveloperEstimateInputs {
            proposal,
            inputs,
            review,
        })
    }
}

fn minutes_to_hours(minutes: Decimal) -> Decimal {
    minutes / Decimal::from(60_u32)
}

/// Native review evidence bound to one session-only developer proposal adoption.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeveloperEstimateProposalReview {
    pub proposal_values_reviewed: bool,
    pub coarse_limitations_accepted: bool,
    pub actor: ActorId,
    pub recorded_at: RecordedAt,
    pub reason: String,
}

/// One explicit adoption result. It remains session-only and is not a quote or production record.
#[derive(Clone, Debug, PartialEq)]
pub struct AdoptedDeveloperEstimateInputs {
    pub proposal: DeveloperEstimateInputProposal,
    pub inputs: DraftEstimateInputs,
    pub review: DeveloperEstimateProposalReview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeveloperEstimateProposalAdoptionError {
    ProposalNotReviewed,
    LimitationsNotAccepted,
    InvalidReviewReason,
    ArithmeticFailure,
}
