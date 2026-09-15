//! Immutable, review-only developer stock revisions. No persistence or estimate adoption.

use partprobe_domain::{
    ActorId, MassKilograms, Money, RecordId, RecordedAt, ShopSettingsDraft,
    StockAllowanceMillimeters, VolumeCubicMillimeters,
};

use crate::{
    AxisAlignedEnvelopeMillimeters, BlankDerivedInputs, CoarseTimeProposal,
    DeveloperEstimateInputProposal, DeveloperEstimateProposalReadiness, ExactStepEnvelopeEvidence,
    StockProposalError, derive_blank_inputs, map_decimal_error, propose_developer_estimate_inputs,
};

/// Additive, session-only evidence version. It does not change proposal or adoption v1.
pub const STOCK_CANDIDATE_SCHEMA_VERSION: u16 = 1;
/// Identity of the final-dimension stock override rule.
pub const STOCK_CANDIDATE_RULE_ID: &str = "partprobe-developer-stock-candidate";
/// Exact rule behavior version.
pub const STOCK_CANDIDATE_RULE_VERSION: (u16, u16, u16) = (1, 0, 0);

/// Required, explicitly supplied edit evidence; native identity must come from the application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StockCandidateEdit {
    dimensions_mm: AxisAlignedEnvelopeMillimeters,
    actor: ActorId,
    recorded_at: RecordedAt,
    reason: String,
}

impl StockCandidateEdit {
    /// Validates a nonempty reason without creating authorization or review authority.
    pub fn new(
        dimensions_mm: AxisAlignedEnvelopeMillimeters,
        actor: ActorId,
        recorded_at: RecordedAt,
        reason: impl Into<String>,
    ) -> Result<Self, StockCandidateError> {
        let reason = reason.into();
        if reason.trim().is_empty() || reason.len() > 2_048 || reason.contains('\0') {
            return Err(StockCandidateError::InvalidReason);
        }
        Ok(Self {
            dimensions_mm,
            actor,
            recorded_at,
            reason,
        })
    }

    #[must_use]
    pub const fn dimensions_mm(&self) -> AxisAlignedEnvelopeMillimeters {
        self.dimensions_mm
    }
    #[must_use]
    pub const fn actor(&self) -> &ActorId {
        &self.actor
    }
    #[must_use]
    pub const fn recorded_at(&self) -> &RecordedAt {
        &self.recorded_at
    }
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Additional categorical limitations, alongside every original proposal reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StockCandidateReasonCode {
    UserEditedBlankDimensions,
    StockAllowanceProfileOverridden,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CandidateBasis {
    envelope: ExactStepEnvelopeEvidence,
    settings: ShopSettingsDraft,
    proposal: DeveloperEstimateInputProposal,
}

/// One immutable revision. The caller must retain predecessors and enforce session sequencing.
///
/// Private fields and no Serde boundary prevent this from becoming an unchecked bridge payload.
/// This value cannot be passed to the existing proposal-adoption API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeveloperStockCandidate {
    id: RecordId,
    revision: u32,
    previous_revision: Option<u32>,
    previous_dimensions_mm: AxisAlignedEnvelopeMillimeters,
    basis: Box<CandidateBasis>,
    edit: StockCandidateEdit,
    total_allowance_mm: [StockAllowanceMillimeters; 3],
    derived: BlankDerivedInputs,
}

impl DeveloperStockCandidate {
    /// Creates revision one only from a fully replayed original developer proposal.
    pub fn new(
        id: RecordId,
        envelope: &ExactStepEnvelopeEvidence,
        settings: &ShopSettingsDraft,
        proposal: &DeveloperEstimateInputProposal,
        edit: StockCandidateEdit,
    ) -> Result<Self, StockCandidateError> {
        let library = settings
            .resource_library()
            .ok_or(StockCandidateError::ResourcesUnavailable)?;
        if &propose_developer_estimate_inputs(envelope, library)? != proposal {
            return Err(StockCandidateError::ProposalMismatch);
        }
        let basis = Box::new(CandidateBasis {
            envelope: envelope.clone(),
            settings: settings.clone(),
            proposal: proposal.clone(),
        });
        Self::build(id, 1, None, proposal.blank_dimensions_mm(), basis, edit)
    }

    /// Creates one successor without mutating the predecessor or original proposal.
    ///
    /// This pure check does not replace an application's optimistic latest-revision check.
    pub fn revise(
        &self,
        current_envelope: &ExactStepEnvelopeEvidence,
        current_settings: &ShopSettingsDraft,
        edit: StockCandidateEdit,
    ) -> Result<Self, StockCandidateError> {
        if current_envelope != &self.basis.envelope || current_settings != &self.basis.settings {
            return Err(StockCandidateError::StaleBasis);
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(StockCandidateError::RevisionOverflow)?;
        Self::build(
            self.id.clone(),
            revision,
            Some(self.revision),
            self.edit.dimensions_mm,
            self.basis.clone(),
            edit,
        )
    }

    fn build(
        id: RecordId,
        revision: u32,
        previous_revision: Option<u32>,
        previous_dimensions_mm: AxisAlignedEnvelopeMillimeters,
        basis: Box<CandidateBasis>,
        edit: StockCandidateEdit,
    ) -> Result<Self, StockCandidateError> {
        if edit.dimensions_mm == previous_dimensions_mm {
            return Err(StockCandidateError::NoStockChange);
        }
        let model = basis.envelope.extents_mm();
        let blank = edit.dimensions_mm;
        let total_allowance_mm = [
            (blank.x(), model.x()),
            (blank.y(), model.y()),
            (blank.z(), model.z()),
        ]
        .map(|(dimension, extent)| {
            if dimension.value() < extent.value() {
                return Err(StockCandidateError::BlankDoesNotEncloseModel);
            }
            let allowance = partprobe_domain::money::checked_decimal_sub_exact(
                dimension.value(),
                extent.value(),
                "stock candidate total allowance",
            )
            .map_err(map_decimal_error)?;
            StockAllowanceMillimeters::new(allowance).map_err(|_| {
                StockCandidateError::Calculation(StockProposalError::ArithmeticOverflow)
            })
        });
        let [x, y, z] = total_allowance_mm;
        let total_allowance_mm = [x?, y?, z?];
        let library = basis
            .settings
            .resource_library()
            .ok_or(StockCandidateError::ResourcesUnavailable)?;
        let derived = derive_blank_inputs(&basis.envelope, library, blank)?;
        Ok(Self {
            id,
            revision,
            previous_revision,
            previous_dimensions_mm,
            basis,
            edit,
            total_allowance_mm,
            derived,
        })
    }

    #[must_use]
    pub const fn evidence_schema_version(&self) -> u16 {
        STOCK_CANDIDATE_SCHEMA_VERSION
    }
    #[must_use]
    pub const fn rule_id(&self) -> &'static str {
        STOCK_CANDIDATE_RULE_ID
    }
    #[must_use]
    pub const fn rule_version(&self) -> (u16, u16, u16) {
        STOCK_CANDIDATE_RULE_VERSION
    }
    #[must_use]
    pub const fn id(&self) -> &RecordId {
        &self.id
    }
    #[must_use]
    pub const fn revision(&self) -> u32 {
        self.revision
    }
    #[must_use]
    pub const fn previous_revision(&self) -> Option<u32> {
        self.previous_revision
    }
    #[must_use]
    pub const fn previous_dimensions_mm(&self) -> AxisAlignedEnvelopeMillimeters {
        self.previous_dimensions_mm
    }
    #[must_use]
    pub fn original_proposal(&self) -> &DeveloperEstimateInputProposal {
        &self.basis.proposal
    }
    #[must_use]
    pub fn settings_basis(&self) -> &ShopSettingsDraft {
        &self.basis.settings
    }
    #[must_use]
    pub const fn edit(&self) -> &StockCandidateEdit {
        &self.edit
    }
    #[must_use]
    pub const fn total_allowance_mm(&self) -> [StockAllowanceMillimeters; 3] {
        self.total_allowance_mm
    }
    #[must_use]
    pub const fn blank_volume_mm3(&self) -> VolumeCubicMillimeters {
        self.derived.blank_volume
    }
    #[must_use]
    pub const fn removed_volume_mm3(&self) -> VolumeCubicMillimeters {
        self.derived.removed_volume
    }
    #[must_use]
    pub const fn blank_mass_kg(&self) -> MassKilograms {
        self.derived.blank_mass
    }
    #[must_use]
    pub const fn unit_stock_material_cost(&self) -> &Money {
        &self.derived.unit_material_cost
    }
    #[must_use]
    pub fn coarse_times(&self) -> CoarseTimeProposal {
        CoarseTimeProposal {
            cutting: self.derived.cutting,
            ..self.basis.proposal.coarse_times()
        }
    }
    #[must_use]
    pub const fn readiness(&self) -> DeveloperEstimateProposalReadiness {
        DeveloperEstimateProposalReadiness::NeedsReview
    }
    #[must_use]
    pub const fn reason_codes(&self) -> [StockCandidateReasonCode; 2] {
        [
            StockCandidateReasonCode::UserEditedBlankDimensions,
            StockCandidateReasonCode::StockAllowanceProfileOverridden,
        ]
    }
}

/// Content-minimized failures; no missing state is represented as numeric zero.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StockCandidateError {
    InvalidReason,
    ResourcesUnavailable,
    ProposalMismatch,
    StaleBasis,
    NoStockChange,
    BlankDoesNotEncloseModel,
    RevisionOverflow,
    Calculation(StockProposalError),
}

impl From<StockProposalError> for StockCandidateError {
    fn from(error: StockProposalError) -> Self {
        Self::Calculation(error)
    }
}

impl std::fmt::Display for StockCandidateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidReason => {
                "stock edit reason must be nonempty, bounded, and contain no null byte"
            }
            Self::ResourcesUnavailable => "stock candidate requires a starter resource library",
            Self::ProposalMismatch => {
                "stock candidate does not match the replayed original proposal"
            }
            Self::StaleBasis => "stock candidate analysis or settings basis changed",
            Self::NoStockChange => "stock candidate dimensions are unchanged",
            Self::BlankDoesNotEncloseModel => {
                "stock candidate does not enclose the source-axis model bounds"
            }
            Self::RevisionOverflow => "stock candidate revision exceeded supported bounds",
            Self::Calculation(error) => return error.fmt(formatter),
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for StockCandidateError {}
