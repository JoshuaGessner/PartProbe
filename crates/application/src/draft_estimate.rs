use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;

use partprobe_domain::{
    CostCategory, DensityKilogramsPerCubicMillimeter, DomainError, EffectiveDate, ItemQuantity,
    MassKilograms, Money, PricingPolicy, RateBasis, RateCard, RateScope, ValueState,
    VolumeCubicMillimeters,
};
use partprobe_estimation_engine::{
    BaseInternalCostComponents, GeometryBasis, MaterialCostComponents, OperationCostComponents,
    PricingOutcome, ResolvedRate, RuleOutcome, apply_pricing_policy, base_internal_cost,
    cycle_time, extend_rate, make_quantity, material_cost, operation_cost, part_mass,
    removed_volume, resolve_rate, risk_reserve, run_cost, setup_lot_cost, total_internal_cost,
};
use partprobe_geometry_core::{GeometryStageReport, StageStatus};
use partprobe_geometry_import::{
    AssetCapability, AssetReadGrant, CorrelationId, GeometryJobId, GeometryWorkerRequest,
    GeometryWorkerSupervisor, LocalAssetRoot, ProvisionalGeometrySnapshot, ResourceQuotas,
    Sha256Digest, SnapshotReference, WorkerAssetFallbackReason, WorkerAssetTransport,
    decode_provisional_geometry_snapshot,
};
use partprobe_security::{AuthorizationAuditSink, AuthorizationPolicy};
use rust_decimal::Decimal;

use crate::{AssetReadServiceError, AssetReadSubject, LocalAssetReadService};

/// Sanitized failure from the geometry-analysis boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GeometryAnalysisFailure {
    /// The worker completed without approval-usable output.
    WorkerUnavailable {
        /// Overall worker status.
        status: StageStatus,
        /// Stable, content-free diagnostic codes.
        diagnostic_codes: Vec<String>,
    },
    /// A successful response omitted its controlled output.
    ControlledOutputMissing,
    /// Controlled output did not satisfy the pinned provisional snapshot contract.
    ControlledOutputInvalid,
}

/// Port that turns one authorized, pathless source grant into validated geometry evidence.
pub trait GeometryAnalysisPort {
    /// Analyzes one authorized model without exposing its filesystem path to the worker protocol.
    fn analyze(
        &self,
        request: &GeometryWorkerRequest,
        grant: AssetReadGrant,
        cancellation: &AtomicBool,
    ) -> Result<AnalyzedGeometryEvidence, GeometryAnalysisFailure>;
}

/// Validated path-free worker request fields whose source hash is derived only after authorization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftGeometryRequestTemplate {
    template: GeometryWorkerRequest,
}

impl DraftGeometryRequestTemplate {
    /// Validates every worker-request field while retaining a private placeholder digest.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        schema_version: partprobe_domain::SchemaVersion,
        job_id: GeometryJobId,
        correlation_id: CorrelationId,
        asset_capability: AssetCapability,
        stages: Vec<partprobe_geometry_core::GeometryStage>,
        analysis_profile: partprobe_geometry_core::AnalysisProfile,
        quotas: ResourceQuotas,
    ) -> Result<Self, DomainError> {
        let placeholder_hash =
            Sha256Digest::new("0000000000000000000000000000000000000000000000000000000000000000")?;
        Ok(Self {
            template: GeometryWorkerRequest::new(
                schema_version,
                job_id,
                correlation_id,
                asset_capability,
                placeholder_hash,
                stages,
                analysis_profile,
                quotas,
            )?,
        })
    }

    fn with_source_hash(
        &self,
        source_hash: Sha256Digest,
    ) -> Result<GeometryWorkerRequest, DomainError> {
        GeometryWorkerRequest::new(
            self.template.schema_version(),
            self.template.job_id().clone(),
            self.template.correlation_id().clone(),
            self.template.asset_capability().clone(),
            source_hash,
            self.template.stages().to_vec(),
            self.template.analysis_profile().clone(),
            self.template.quotas(),
        )
    }

    /// Returns the opaque source capability bound to the eventual worker request.
    #[must_use]
    pub const fn asset_capability(&self) -> &AssetCapability {
        self.template.asset_capability()
    }

    /// Returns the maximum authorized source size used by fingerprinting and worker validation.
    #[must_use]
    pub const fn max_input_bytes(&self) -> u64 {
        self.template.quotas().max_input_bytes()
    }
}

impl GeometryAnalysisPort for GeometryWorkerSupervisor {
    fn analyze(
        &self,
        request: &GeometryWorkerRequest,
        grant: AssetReadGrant,
        cancellation: &AtomicBool,
    ) -> Result<AnalyzedGeometryEvidence, GeometryAnalysisFailure> {
        let execution = self.execute_with_grant(request, grant, cancellation);
        let (response, output, asset_transport, fallback_reason) = execution.into_parts();
        if !response.status().permits_authoritative_output() {
            return Err(GeometryAnalysisFailure::WorkerUnavailable {
                status: response.status(),
                diagnostic_codes: response
                    .diagnostic_codes()
                    .iter()
                    .map(|code| code.as_str().to_owned())
                    .collect(),
            });
        }
        let output = output.ok_or(GeometryAnalysisFailure::ControlledOutputMissing)?;
        let snapshot =
            decode_provisional_geometry_snapshot(&output, request.expected_source_hash())
                .map_err(|_| GeometryAnalysisFailure::ControlledOutputInvalid)?;
        let snapshot_reference = response
            .snapshot_reference()
            .cloned()
            .ok_or(GeometryAnalysisFailure::ControlledOutputMissing)?;
        AnalyzedGeometryEvidence::new(
            response.status(),
            response.stage_reports().to_vec(),
            snapshot_reference,
            output.content_hash().clone(),
            output.byte_length(),
            snapshot,
            asset_transport,
            fallback_reason,
        )
        .map_err(|_| GeometryAnalysisFailure::ControlledOutputInvalid)
    }
}

/// Validated, provisional geometry and worker lineage retained by a draft-estimate session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalyzedGeometryEvidence {
    /// Overall worker status.
    pub status: StageStatus,
    /// Ordered stage outcomes and sanitized warnings.
    pub stage_reports: Vec<GeometryStageReport>,
    /// Opaque schema reference for the controlled output.
    pub snapshot_reference: SnapshotReference,
    /// Digest of the controlled output bytes.
    pub output_hash: Sha256Digest,
    /// Exact controlled-output byte count.
    pub output_byte_length: u64,
    /// Validated provisional exact-B-rep measurements.
    pub snapshot: ProvisionalGeometrySnapshot,
    /// Transport selected by the supervisor.
    pub asset_transport: Option<WorkerAssetTransport>,
    /// Explicit direct-transport fallback reason, when applicable.
    pub fallback_reason: Option<WorkerAssetFallbackReason>,
}

impl AnalyzedGeometryEvidence {
    /// Creates evidence only when successful output and source lineage are internally consistent.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        status: StageStatus,
        stage_reports: Vec<GeometryStageReport>,
        snapshot_reference: SnapshotReference,
        output_hash: Sha256Digest,
        output_byte_length: u64,
        snapshot: ProvisionalGeometrySnapshot,
        asset_transport: Option<WorkerAssetTransport>,
        fallback_reason: Option<WorkerAssetFallbackReason>,
    ) -> Result<Self, DomainError> {
        if !status.permits_authoritative_output() || output_byte_length == 0 {
            return Err(DomainError::InvalidValue {
                field: "analyzed geometry evidence",
                reason: "successful status and nonempty controlled output are required",
            });
        }
        Ok(Self {
            status,
            stage_reports,
            snapshot_reference,
            output_hash,
            output_byte_length,
            snapshot,
            asset_transport,
            fallback_reason,
        })
    }
}

/// Explicit acknowledgement required before provisional measurements feed a draft estimate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DraftGeometryReview {
    /// Canonical millimeter interpretation was reviewed.
    pub canonical_units_reviewed: bool,
    /// Worker warnings (including an empty warning set) were reviewed.
    pub warnings_reviewed: bool,
}

impl DraftGeometryReview {
    /// Records the two explicit review decisions used by the GUI-2 session-only slice.
    #[must_use]
    pub const fn new(canonical_units_reviewed: bool, warnings_reviewed: bool) -> Self {
        Self {
            canonical_units_reviewed,
            warnings_reviewed,
        }
    }

    fn is_complete(self) -> bool {
        self.canonical_units_reviewed && self.warnings_reviewed
    }
}

/// Explicit stock facts; no production stock or density default is supplied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftStockInputs {
    /// Selected stock volume in cubic millimeters.
    pub stock_volume: VolumeCubicMillimeters,
    /// Selected material density in kilograms per cubic millimeter.
    pub density: DensityKilogramsPerCubicMillimeter,
}

/// Explicit quantities used by CALC-005.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftQuantityInputs {
    /// Required delivered parts.
    pub deliver: ItemQuantity,
    /// Planned setup or process spares.
    pub planned_spares: ItemQuantity,
    /// Planned destructive samples.
    pub destructive_samples: ItemQuantity,
}

/// Explicit setup, programming, cycle, and inspection time inputs in hours.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftTimeInputs {
    pub setup_hours: Decimal,
    pub programming_hours: Decimal,
    pub cutting_hours_per_item: Decimal,
    pub non_cutting_hours_per_item: Decimal,
    pub load_unload_hours_per_item: Decimal,
    pub in_cycle_inspection_hours_per_item: Decimal,
    pub quality_inspection_hours: Decimal,
}

/// Explicit monetary inputs to CALC-007.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftMaterialCostInputs {
    pub purchased: Money,
    pub cut: Money,
    pub certificate: Money,
    pub inbound_freight: Money,
    pub approved_remnant_credit: Money,
}

/// Explicit non-rate monetary inputs to CALC-012.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftOperationCostInputs {
    pub prove_out: Money,
    pub tooling: Money,
    pub consumables: Money,
    pub fixture: Money,
    pub outside: Money,
    pub freight: Money,
}

/// Explicit lot-level, risk, and rework inputs to CALC-013 through CALC-015.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftBaseCostInputs {
    pub nonrecurring_engineering: Money,
    pub administration: Money,
    pub overhead: Money,
    pub accepted_risk_impacts: Vec<Money>,
    pub expected_rework: Money,
}

/// Complete manually supplied input set for the GUI-2 deterministic draft estimate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftEstimateInputs {
    pub stock: DraftStockInputs,
    pub quantities: DraftQuantityInputs,
    pub times: DraftTimeInputs,
    pub material: DraftMaterialCostInputs,
    pub operation: DraftOperationCostInputs,
    pub base: DraftBaseCostInputs,
}

/// Pinned rate-card selection context; ordered scopes are never inferred.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftRateContext {
    pub rate_card: RateCard,
    pub effective_on: EffectiveDate,
    pub ordered_scopes: Vec<RateScope>,
}

impl DraftRateContext {
    /// Requires at least one explicit scope for deterministic resolution.
    pub fn new(
        rate_card: RateCard,
        effective_on: EffectiveDate,
        ordered_scopes: Vec<RateScope>,
    ) -> Result<Self, DomainError> {
        if ordered_scopes.is_empty() {
            return Err(DomainError::InvalidValue {
                field: "draft estimate rate scopes",
                reason: "at least one explicit ordered scope is required",
            });
        }
        Ok(Self {
            rate_card,
            effective_on,
            ordered_scopes,
        })
    }
}

/// Every immutable rate selected for one draft estimate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftResolvedRates {
    pub setup_labor: ResolvedRate,
    pub programming: ResolvedRate,
    pub run_labor: ResolvedRate,
    pub machine: ResolvedRate,
    pub quality_inspection: ResolvedRate,
}

/// Replay-relevant source, input, rate, policy, and rule evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftEstimateTrace {
    pub geometry: AnalyzedGeometryEvidence,
    pub inputs: DraftEstimateInputs,
    pub resolved_rates: DraftResolvedRates,
    pub pricing_policy: PricingPolicy,
    pub calculation_rule_ids: Vec<&'static str>,
}

/// Deterministic GUI-2 draft output. This is session-only and not an approved quote.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftEstimateResult {
    pub net_part_volume: VolumeCubicMillimeters,
    pub part_mass: MassKilograms,
    pub stock_mass: MassKilograms,
    pub removed_volume: RuleOutcome<VolumeCubicMillimeters>,
    pub make_quantity: ItemQuantity,
    pub material_cost: Money,
    pub setup_cost: Money,
    pub programming_cost: Money,
    pub cycle_hours_per_item: Decimal,
    pub run_cost: Money,
    pub quality_inspection_cost: Money,
    pub operation_cost: Money,
    pub base_internal_cost: Money,
    pub risk_reserve: Money,
    pub total_internal_cost: Money,
    pub pricing: PricingOutcome,
    pub trace: DraftEstimateTrace,
}

/// In-memory draft session that never persists or silently approves an estimate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftEstimateSession {
    geometry: AnalyzedGeometryEvidence,
    geometry_review: Option<DraftGeometryReview>,
    inputs: Option<DraftEstimateInputs>,
    rate_context: Option<DraftRateContext>,
    pricing_policy: Option<PricingPolicy>,
}

impl DraftEstimateSession {
    fn new(geometry: AnalyzedGeometryEvidence) -> Self {
        Self {
            geometry,
            geometry_review: None,
            inputs: None,
            rate_context: None,
            pricing_policy: None,
        }
    }

    /// Returns the provisional evidence for display and explicit review.
    #[must_use]
    pub const fn geometry(&self) -> &AnalyzedGeometryEvidence {
        &self.geometry
    }

    /// Replaces the session-only geometry review state.
    pub fn set_geometry_review(&mut self, review: DraftGeometryReview) {
        self.geometry_review = Some(review);
    }

    /// Replaces the complete manual input set and invalidates no historical persisted record.
    pub fn set_inputs(&mut self, inputs: DraftEstimateInputs) {
        self.inputs = Some(inputs);
    }

    /// Replaces the pinned rate selection context for subsequent evaluation.
    pub fn set_rate_context(&mut self, context: DraftRateContext) {
        self.rate_context = Some(context);
    }

    /// Replaces the pinned pricing policy for subsequent evaluation.
    pub fn set_pricing_policy(&mut self, policy: PricingPolicy) {
        self.pricing_policy = Some(policy);
    }

    /// Recalculates solely through deterministic domain and estimation-engine rules.
    #[must_use]
    pub fn evaluate(&self) -> ValueState<DraftEstimateResult> {
        let Some(review) = self.geometry_review else {
            return unavailable("geometry units and warnings have not been reviewed");
        };
        if !review.is_complete() {
            return unavailable("geometry units and warnings require explicit review");
        }
        let Some(inputs) = &self.inputs else {
            return unavailable(
                "manual stock, material, time, cost, and quantity inputs are missing",
            );
        };
        let Some(rate_context) = &self.rate_context else {
            return unavailable(
                "a pinned rate-card, effective date, and ordered scope are missing",
            );
        };
        let Some(pricing_policy) = &self.pricing_policy else {
            return unavailable("a pinned pricing policy is missing");
        };

        let rates = match resolve_required_rates(rate_context) {
            Ok(rates) => rates,
            Err(failure) => return failure.into_value_state(),
        };
        match calculate(&self.geometry, inputs, &rates, pricing_policy) {
            Ok(result) => ValueState::available(result),
            Err(error) => ValueState::Blocked {
                reason: format!("deterministic estimate inputs are invalid: {error}"),
            },
        }
    }
}

/// Application boundary that authorizes source access before invoking geometry analysis.
#[derive(Debug)]
pub struct DraftEstimateApplication<P, A, G> {
    asset_reads: LocalAssetReadService<P, A>,
    geometry_analysis: G,
}

impl<P, A, G> DraftEstimateApplication<P, A, G>
where
    P: AuthorizationPolicy,
    A: AuthorizationAuditSink,
    G: GeometryAnalysisPort,
{
    /// Composes the existing governed read boundary with a geometry-analysis adapter.
    #[must_use]
    pub const fn new(asset_reads: LocalAssetReadService<P, A>, geometry_analysis: G) -> Self {
        Self {
            asset_reads,
            geometry_analysis,
        }
    }

    /// Authorizes, audits, opens, analyzes, and starts one ephemeral draft session.
    pub fn start_session(
        &self,
        subject: AssetReadSubject,
        root: &LocalAssetRoot,
        request: &GeometryWorkerRequest,
        relative_path: &Path,
        cancellation: &AtomicBool,
    ) -> Result<DraftEstimateSession, DraftEstimateApplicationError> {
        let grant = self
            .asset_reads
            .authorize_and_open(
                subject,
                root,
                request.asset_capability().clone(),
                relative_path,
            )
            .map_err(DraftEstimateApplicationError::AssetRead)?;
        let geometry = self
            .geometry_analysis
            .analyze(request, grant, cancellation)
            .map_err(DraftEstimateApplicationError::GeometryAnalysis)?;
        Ok(DraftEstimateSession::new(geometry))
    }

    /// Authorizes and audits a selected source before deriving the request's source fingerprint.
    ///
    /// The same already-open grant is rewound and consumed by the geometry port. Neither a raw
    /// path nor unaudited file read is required in the desktop adapter.
    pub fn start_session_from_unfingerprinted_source(
        &self,
        subject: AssetReadSubject,
        root: &LocalAssetRoot,
        request_template: &DraftGeometryRequestTemplate,
        relative_path: &Path,
        cancellation: &AtomicBool,
    ) -> Result<DraftEstimateSession, DraftEstimateApplicationError> {
        let mut grant = self
            .asset_reads
            .authorize_and_open(
                subject,
                root,
                request_template.asset_capability().clone(),
                relative_path,
            )
            .map_err(DraftEstimateApplicationError::AssetRead)?;
        let source_hash = grant
            .fingerprint_sha256(request_template.max_input_bytes())
            .map_err(|_| DraftEstimateApplicationError::SourceFingerprint)?;
        let request = request_template
            .with_source_hash(source_hash)
            .map_err(|_| DraftEstimateApplicationError::SourceFingerprint)?;
        let geometry = self
            .geometry_analysis
            .analyze(&request, grant, cancellation)
            .map_err(DraftEstimateApplicationError::GeometryAnalysis)?;
        Ok(DraftEstimateSession::new(geometry))
    }

    /// Returns the governed asset-read service for audit verification and adapter composition.
    #[must_use]
    pub const fn asset_reads(&self) -> &LocalAssetReadService<P, A> {
        &self.asset_reads
    }
}

/// Content-free failures before a session can be created.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DraftEstimateApplicationError {
    /// Authorization, audit, or containment rejected source access.
    AssetRead(AssetReadServiceError),
    /// Authorized source bytes could not produce the bounded request fingerprint.
    SourceFingerprint,
    /// Geometry analysis did not yield validated provisional evidence.
    GeometryAnalysis(GeometryAnalysisFailure),
}

fn resolve_required_rates(
    context: &DraftRateContext,
) -> Result<DraftResolvedRates, RateResolutionFailure> {
    Ok(DraftResolvedRates {
        setup_labor: required_rate(context, CostCategory::SetupLabor)?,
        programming: required_rate(context, CostCategory::Programming)?,
        run_labor: required_rate(context, CostCategory::RunLabor)?,
        machine: required_rate(context, CostCategory::Machine)?,
        quality_inspection: required_rate(context, CostCategory::QualityInspection)?,
    })
}

fn required_rate(
    context: &DraftRateContext,
    category: CostCategory,
) -> Result<ResolvedRate, RateResolutionFailure> {
    match resolve_rate(
        &context.rate_card,
        category,
        RateBasis::PerHour,
        &context.effective_on,
        &context.ordered_scopes,
    ) {
        ValueState::Available { value } => Ok(value),
        ValueState::Unavailable { reason } => Err(RateResolutionFailure::Unavailable(reason)),
        ValueState::Blocked { reason } => Err(RateResolutionFailure::Blocked(reason)),
        ValueState::Unknown { reason } => Err(RateResolutionFailure::Unknown(reason)),
        ValueState::Stale { reason, as_of, .. } => Err(RateResolutionFailure::Blocked(format!(
            "stale rate evidence as of {as_of}: {reason}"
        ))),
    }
}

enum RateResolutionFailure {
    Unavailable(String),
    Blocked(String),
    Unknown(String),
}

impl RateResolutionFailure {
    fn into_value_state(self) -> ValueState<DraftEstimateResult> {
        match self {
            Self::Unavailable(reason) => ValueState::Unavailable { reason },
            Self::Blocked(reason) => ValueState::Blocked { reason },
            Self::Unknown(reason) => ValueState::Unknown { reason },
        }
    }
}

fn calculate(
    geometry: &AnalyzedGeometryEvidence,
    inputs: &DraftEstimateInputs,
    rates: &DraftResolvedRates,
    pricing_policy: &PricingPolicy,
) -> Result<DraftEstimateResult, partprobe_estimation_engine::CalculationError> {
    if inputs.quantities.deliver.value() == 0 {
        return Err(DomainError::InvalidValue {
            field: "deliver quantity",
            reason: "must be greater than zero for a draft estimate",
        }
        .into());
    }
    let net_volume_decimal =
        Decimal::from_str(geometry.snapshot.enclosed_volume_mm3()).map_err(|_| {
            DomainError::InvalidValue {
                field: "provisional enclosed volume",
                reason: "must be an exact decimal",
            }
        })?;
    let net_part_volume = VolumeCubicMillimeters::new(net_volume_decimal)?;
    let calculated_part_mass = part_mass(net_part_volume, inputs.stock.density)?;
    let calculated_stock_mass = part_mass(inputs.stock.stock_volume, inputs.stock.density)?;
    let removed_volume = removed_volume(
        inputs.stock.stock_volume,
        net_part_volume,
        GeometryBasis {
            enclosed: true,
            multi_body_resolved: geometry.snapshot.solid_body_count() == 1,
        },
    )?;
    let make_quantity = make_quantity(
        inputs.quantities.deliver,
        inputs.quantities.planned_spares,
        inputs.quantities.destructive_samples,
    )?;
    let material_cost = material_cost(&MaterialCostComponents {
        purchased: inputs.material.purchased.clone(),
        cut: inputs.material.cut.clone(),
        certificate: inputs.material.certificate.clone(),
        inbound_freight: inputs.material.inbound_freight.clone(),
        approved_remnant_credit: inputs.material.approved_remnant_credit.clone(),
    })?;
    let setup_cost = setup_lot_cost(inputs.times.setup_hours, &rates.setup_labor)?;
    let programming_cost = extend_rate(
        &rates.programming,
        RateBasis::PerHour,
        inputs.times.programming_hours,
    )?;
    let cycle_hours_per_item = cycle_time(
        inputs.times.cutting_hours_per_item,
        inputs.times.non_cutting_hours_per_item,
        inputs.times.load_unload_hours_per_item,
        inputs.times.in_cycle_inspection_hours_per_item,
    )?;
    let run_cost = run_cost(
        cycle_hours_per_item,
        make_quantity,
        &[rates.run_labor.clone(), rates.machine.clone()],
    )?;
    let quality_inspection_cost = extend_rate(
        &rates.quality_inspection,
        RateBasis::PerHour,
        inputs.times.quality_inspection_hours,
    )?;
    let operation_cost = operation_cost(&OperationCostComponents {
        setup: setup_cost.clone(),
        programming: programming_cost.clone(),
        prove_out: inputs.operation.prove_out.clone(),
        run: run_cost.clone(),
        tooling: inputs.operation.tooling.clone(),
        consumables: inputs.operation.consumables.clone(),
        fixture: inputs.operation.fixture.clone(),
        quality_inspection: quality_inspection_cost.clone(),
        outside: inputs.operation.outside.clone(),
        freight: inputs.operation.freight.clone(),
    })?;
    let base_internal_cost = base_internal_cost(&BaseInternalCostComponents {
        material: material_cost.clone(),
        operations: vec![operation_cost.clone()],
        nonrecurring_engineering: inputs.base.nonrecurring_engineering.clone(),
        administration: inputs.base.administration.clone(),
        overhead: inputs.base.overhead.clone(),
    })?;
    let risk_reserve = risk_reserve(
        pricing_policy.currency(),
        &inputs.base.accepted_risk_impacts,
    )?;
    let total_internal_cost = total_internal_cost(
        &base_internal_cost,
        &risk_reserve,
        &inputs.base.expected_rework,
    )?;
    let pricing = apply_pricing_policy(&total_internal_cost, pricing_policy)?;

    Ok(DraftEstimateResult {
        net_part_volume,
        part_mass: calculated_part_mass,
        stock_mass: calculated_stock_mass,
        removed_volume,
        make_quantity,
        material_cost,
        setup_cost,
        programming_cost,
        cycle_hours_per_item,
        run_cost,
        quality_inspection_cost,
        operation_cost,
        base_internal_cost,
        risk_reserve,
        total_internal_cost,
        pricing,
        trace: DraftEstimateTrace {
            geometry: geometry.clone(),
            inputs: inputs.clone(),
            resolved_rates: rates.clone(),
            pricing_policy: pricing_policy.clone(),
            calculation_rule_ids: vec![
                "CALC-001", "CALC-003", "CALC-005", "CALC-007", "CALC-008", "CALC-010", "CALC-011",
                "CALC-012", "CALC-013", "CALC-014", "CALC-015", "CALC-018",
            ],
        },
    })
}

fn unavailable(reason: &str) -> ValueState<DraftEstimateResult> {
    ValueState::Unavailable {
        reason: reason.to_owned(),
    }
}
