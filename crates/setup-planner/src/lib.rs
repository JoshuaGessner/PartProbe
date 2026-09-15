//! Pure, UI-independent stock and setup proposal rules.

use partprobe_domain::{
    DensityKilogramsPerCubicMeter, LibraryRecordState, MachineProfileId, MassKilograms,
    MaterialDefinitionId, MaterialOfferId, Money, ProcessClass, ResourceSelection,
    ResourceSelectionId, ResourceSelectionState, RuntimeMinutes, RuntimeProfileId,
    ShopResourceLibrary, ShopResourceLibraryId, ShopResourceVersion, StockAllowanceMillimeters,
    StockAllowanceProfile, StockAllowanceProfileId, StockForm, VolumeCubicMillimeters,
    money::{checked_decimal_mul_exact, checked_decimal_sub_exact},
};
use partprobe_geometry_core::Sha256Digest;
use partprobe_geometry_import::ProvisionalExactStepAnalysis;
use rust_decimal::Decimal;

pub mod stock_candidate;

/// First additive exact-STEP envelope evidence contract used only by the proposal spike.
pub const EXACT_STEP_ENVELOPE_EVIDENCE_SCHEMA_VERSION: u16 =
    partprobe_geometry_core::EXACT_STEP_ENVELOPE_DERIVATIVE_SCHEMA_VERSION;
/// Stable identity of the first rectangular stock-envelope proposal rule.
pub const RECTANGULAR_STOCK_ENVELOPE_RULE_ID: &str = "partprobe-rectangular-stock-envelope-policy";
/// Exact behavior version of [`propose_rectangular_stock`].
pub const RECTANGULAR_STOCK_ENVELOPE_RULE_VERSION: (u16, u16, u16) = (1, 0, 0);
/// Stable identity of the session-only developer estimate-input proposal rule.
pub const DEVELOPER_ESTIMATE_INPUT_PROPOSAL_RULE_ID: &str =
    "partprobe-developer-estimate-input-proposal";
/// Exact behavior version of [`propose_developer_estimate_inputs`].
pub const DEVELOPER_ESTIMATE_INPUT_PROPOSAL_RULE_VERSION: (u16, u16, u16) = (1, 0, 0);

/// Canonical positive millimetre length used by the proposal-only evidence contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalMillimeterLength(Decimal);

impl CanonicalMillimeterLength {
    /// Creates a finite, positive canonical millimetre length.
    pub fn new(value: Decimal) -> Result<Self, StockProposalError> {
        if value <= Decimal::ZERO {
            return Err(StockProposalError::InvalidEnvelopeDimension);
        }
        Ok(Self(value))
    }

    /// Returns the exact decimal value in millimetres.
    #[must_use]
    pub const fn value(self) -> Decimal {
        self.0
    }
}

/// Axis-aligned extents in the exact STEP source coordinate frame, canonicalized to millimetres.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AxisAlignedEnvelopeMillimeters {
    x: CanonicalMillimeterLength,
    y: CanonicalMillimeterLength,
    z: CanonicalMillimeterLength,
}

impl AxisAlignedEnvelopeMillimeters {
    /// Creates exact positive X/Y/Z extents.
    #[must_use]
    pub const fn new(
        x: CanonicalMillimeterLength,
        y: CanonicalMillimeterLength,
        z: CanonicalMillimeterLength,
    ) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub const fn x(self) -> CanonicalMillimeterLength {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> CanonicalMillimeterLength {
        self.y
    }

    #[must_use]
    pub const fn z(self) -> CanonicalMillimeterLength {
        self.z
    }

    fn checked_volume(self) -> Result<Decimal, StockProposalError> {
        self.x
            .value()
            .checked_mul(self.y.value())
            .and_then(|value| value.checked_mul(self.z.value()))
            .ok_or(StockProposalError::ArithmeticOverflow)
    }
}

/// Source-bound exact-STEP envelope facts expected from a later controlled geometry derivative.
///
/// This additive input does not alter or reinterpret `geometry-snapshot-v1`. Native emission is
/// provisional developer evidence and does not establish supported desktop import or adoption.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactStepEnvelopeEvidence {
    source_hash: Sha256Digest,
    analysis_output_hash: Sha256Digest,
    extents_mm: AxisAlignedEnvelopeMillimeters,
    part_volume_mm3: VolumeCubicMillimeters,
}

impl ExactStepEnvelopeEvidence {
    /// Builds proposal input from one already source-validated controlled exact STEP analysis.
    pub fn from_controlled_analysis(
        analysis: &ProvisionalExactStepAnalysis,
        analysis_output_hash: Sha256Digest,
    ) -> Result<Self, StockProposalError> {
        if analysis.snapshot().solid_body_count() != 1 {
            return Err(StockProposalError::MultipleSolidBodiesUnsupported);
        }
        let envelope = analysis.envelope();
        if envelope.schema_version() != EXACT_STEP_ENVELOPE_EVIDENCE_SCHEMA_VERSION {
            return Err(StockProposalError::UnsupportedEnvelopeEvidenceVersion);
        }
        let [x, y, z] = envelope.aabb_extents_mm().each_ref().map(|value| {
            Decimal::from_str_exact(value.as_str())
                .map_err(|_| StockProposalError::InvalidExactStepDecimal)
                .and_then(CanonicalMillimeterLength::new)
        });
        let extents_mm = AxisAlignedEnvelopeMillimeters::new(x?, y?, z?);
        let part_volume_mm3 = VolumeCubicMillimeters::new(
            Decimal::from_str_exact(analysis.snapshot().enclosed_volume_mm3())
                .map_err(|_| StockProposalError::InvalidExactStepDecimal)?,
        )
        .map_err(|_| StockProposalError::InvalidExactStepDecimal)?;
        if part_volume_mm3.value() > extents_mm.checked_volume()? {
            return Err(StockProposalError::PartVolumeExceedsModelEnvelope);
        }
        Ok(Self {
            source_hash: analysis.snapshot().source_hash().clone(),
            analysis_output_hash,
            extents_mm,
            part_volume_mm3,
        })
    }

    #[must_use]
    pub const fn source_hash(&self) -> &Sha256Digest {
        &self.source_hash
    }

    #[must_use]
    pub const fn analysis_output_hash(&self) -> &Sha256Digest {
        &self.analysis_output_hash
    }

    #[must_use]
    pub const fn extents_mm(&self) -> AxisAlignedEnvelopeMillimeters {
        self.extents_mm
    }

    #[must_use]
    pub const fn part_volume_mm3(&self) -> VolumeCubicMillimeters {
        self.part_volume_mm3
    }
}

/// Current non-authoritative readiness state of the stock proposal spike.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StockProposalReadiness {
    /// The candidate needs estimator review before it can become an estimate input.
    NeedsReview,
}

/// Stable, non-percentage reasons why the first proposal remains review-only.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StockProposalReasonCode {
    /// Only the source-coordinate axis-aligned orientation was evaluated.
    AxisAlignedOrientationOnly,
    /// No governed standard stock-size catalog was resolved.
    StandardSizeNotResolved,
    /// Current supplier availability and reservation state were not evaluated.
    AvailabilityNotResolved,
}

/// One explainable rectangular blank proposal. It is not adopted estimate or purchasing input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RectangularStockProposal {
    evidence_schema_version: u16,
    rule_id: &'static str,
    rule_version: (u16, u16, u16),
    source_hash: Sha256Digest,
    analysis_output_hash: Sha256Digest,
    selection_id: ResourceSelectionId,
    selection_version: ShopResourceVersion,
    stock_allowance_id: StockAllowanceProfileId,
    stock_allowance_version: ShopResourceVersion,
    model_extents_mm: AxisAlignedEnvelopeMillimeters,
    total_allowance_mm: [StockAllowanceMillimeters; 3],
    blank_dimensions_mm: AxisAlignedEnvelopeMillimeters,
    blank_volume_mm3: VolumeCubicMillimeters,
    removed_volume_mm3: VolumeCubicMillimeters,
    readiness: StockProposalReadiness,
    reason_codes: [StockProposalReasonCode; 3],
}

/// Current non-authoritative readiness of the session-only developer proposal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeveloperEstimateProposalReadiness {
    /// Every proposed value requires explicit human review and adoption.
    NeedsReview,
}

/// Stable reasons why the developer proposal is not estimate or production authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeveloperEstimateProposalReasonCode {
    /// The proposal exists only in the current application session.
    SessionOnlyDeveloperEvidence,
    /// The bundled shop resources remain draft evidence.
    DraftShopResourceLibrary,
    /// Only the source-coordinate axis-aligned orientation was evaluated.
    AxisAlignedOrientationOnly,
    /// No governed standard stock size was resolved.
    StandardSizeNotResolved,
    /// Supplier availability, lead time, and reservation state were not evaluated.
    AvailabilityNotResolved,
    /// Volumetric cutting time is coarse planning evidence, not CAM simulation.
    CoarseRuntimeNotCam,
    /// An estimator must review and explicitly adopt values before calculation.
    HumanReviewAndAdoptionRequired,
}

/// Version-pinned coarse time evidence produced by the developer proposal rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoarseTimeProposal {
    setup: RuntimeMinutes,
    programming: RuntimeMinutes,
    cutting: RuntimeMinutes,
    load_unload: RuntimeMinutes,
    quality: RuntimeMinutes,
}

impl CoarseTimeProposal {
    #[must_use]
    pub const fn setup(self) -> RuntimeMinutes {
        self.setup
    }

    #[must_use]
    pub const fn programming(self) -> RuntimeMinutes {
        self.programming
    }

    #[must_use]
    pub const fn cutting(self) -> RuntimeMinutes {
        self.cutting
    }

    #[must_use]
    pub const fn load_unload(self) -> RuntimeMinutes {
        self.load_unload
    }

    #[must_use]
    pub const fn quality(self) -> RuntimeMinutes {
        self.quality
    }
}

/// Explainable, session-only inputs proposed from exact STEP and one persisted draft library.
///
/// This value is not installed into an estimate session and has no routing, purchasing, quote,
/// or production authority. Every value remains `NeedsReview` until a later application boundary
/// records explicit human adoption.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeveloperEstimateInputProposal {
    evidence_schema_version: u16,
    rule_id: &'static str,
    rule_version: (u16, u16, u16),
    source_hash: Sha256Digest,
    analysis_output_hash: Sha256Digest,
    library_id: ShopResourceLibraryId,
    library_version: ShopResourceVersion,
    material_id: MaterialDefinitionId,
    material_version: ShopResourceVersion,
    material_offer_id: MaterialOfferId,
    material_offer_version: ShopResourceVersion,
    stock_allowance_id: StockAllowanceProfileId,
    stock_allowance_version: ShopResourceVersion,
    machine_id: MachineProfileId,
    machine_version: ShopResourceVersion,
    runtime_id: RuntimeProfileId,
    runtime_version: ShopResourceVersion,
    model_extents_mm: AxisAlignedEnvelopeMillimeters,
    part_volume_mm3: VolumeCubicMillimeters,
    total_allowance_mm: [StockAllowanceMillimeters; 3],
    blank_dimensions_mm: AxisAlignedEnvelopeMillimeters,
    blank_volume_mm3: VolumeCubicMillimeters,
    removed_volume_mm3: VolumeCubicMillimeters,
    material_density_kg_per_m3: DensityKilogramsPerCubicMeter,
    blank_mass_kg: MassKilograms,
    material_price_per_kg: Money,
    unit_stock_material_cost: Money,
    removal_rate_mm3_per_minute: partprobe_domain::RemovalRateCubicMillimetersPerMinute,
    coarse_times: CoarseTimeProposal,
    readiness: DeveloperEstimateProposalReadiness,
    reason_codes: [DeveloperEstimateProposalReasonCode; 7],
}

impl DeveloperEstimateInputProposal {
    #[must_use]
    pub const fn evidence_schema_version(&self) -> u16 {
        self.evidence_schema_version
    }

    #[must_use]
    pub const fn rule_id(&self) -> &'static str {
        self.rule_id
    }

    #[must_use]
    pub const fn rule_version(&self) -> (u16, u16, u16) {
        self.rule_version
    }

    #[must_use]
    pub const fn source_hash(&self) -> &Sha256Digest {
        &self.source_hash
    }

    #[must_use]
    pub const fn analysis_output_hash(&self) -> &Sha256Digest {
        &self.analysis_output_hash
    }

    #[must_use]
    pub const fn library_id(&self) -> &ShopResourceLibraryId {
        &self.library_id
    }

    #[must_use]
    pub const fn library_version(&self) -> ShopResourceVersion {
        self.library_version
    }

    #[must_use]
    pub const fn material_id(&self) -> &MaterialDefinitionId {
        &self.material_id
    }

    #[must_use]
    pub const fn material_version(&self) -> ShopResourceVersion {
        self.material_version
    }

    #[must_use]
    pub const fn material_offer_id(&self) -> &MaterialOfferId {
        &self.material_offer_id
    }

    #[must_use]
    pub const fn material_offer_version(&self) -> ShopResourceVersion {
        self.material_offer_version
    }

    #[must_use]
    pub const fn stock_allowance_id(&self) -> &StockAllowanceProfileId {
        &self.stock_allowance_id
    }

    #[must_use]
    pub const fn stock_allowance_version(&self) -> ShopResourceVersion {
        self.stock_allowance_version
    }

    #[must_use]
    pub const fn machine_id(&self) -> &MachineProfileId {
        &self.machine_id
    }

    #[must_use]
    pub const fn machine_version(&self) -> ShopResourceVersion {
        self.machine_version
    }

    #[must_use]
    pub const fn runtime_id(&self) -> &RuntimeProfileId {
        &self.runtime_id
    }

    #[must_use]
    pub const fn runtime_version(&self) -> ShopResourceVersion {
        self.runtime_version
    }

    #[must_use]
    pub const fn model_extents_mm(&self) -> AxisAlignedEnvelopeMillimeters {
        self.model_extents_mm
    }

    #[must_use]
    pub const fn part_volume_mm3(&self) -> VolumeCubicMillimeters {
        self.part_volume_mm3
    }

    /// Returns total dimensional addition for X/Y/Z, not a per-side allowance.
    #[must_use]
    pub const fn total_allowance_mm(&self) -> [StockAllowanceMillimeters; 3] {
        self.total_allowance_mm
    }

    #[must_use]
    pub const fn blank_dimensions_mm(&self) -> AxisAlignedEnvelopeMillimeters {
        self.blank_dimensions_mm
    }

    #[must_use]
    pub const fn blank_volume_mm3(&self) -> VolumeCubicMillimeters {
        self.blank_volume_mm3
    }

    #[must_use]
    pub const fn removed_volume_mm3(&self) -> VolumeCubicMillimeters {
        self.removed_volume_mm3
    }

    #[must_use]
    pub const fn material_density_kg_per_m3(&self) -> DensityKilogramsPerCubicMeter {
        self.material_density_kg_per_m3
    }

    #[must_use]
    pub const fn blank_mass_kg(&self) -> MassKilograms {
        self.blank_mass_kg
    }

    #[must_use]
    pub const fn material_price_per_kg(&self) -> &Money {
        &self.material_price_per_kg
    }

    #[must_use]
    pub const fn unit_stock_material_cost(&self) -> &Money {
        &self.unit_stock_material_cost
    }

    #[must_use]
    pub const fn removal_rate_mm3_per_minute(
        &self,
    ) -> partprobe_domain::RemovalRateCubicMillimetersPerMinute {
        self.removal_rate_mm3_per_minute
    }

    #[must_use]
    pub const fn coarse_times(&self) -> CoarseTimeProposal {
        self.coarse_times
    }

    #[must_use]
    pub const fn readiness(&self) -> DeveloperEstimateProposalReadiness {
        self.readiness
    }

    #[must_use]
    pub const fn reason_codes(&self) -> &[DeveloperEstimateProposalReasonCode; 7] {
        &self.reason_codes
    }
}

impl RectangularStockProposal {
    #[must_use]
    pub const fn evidence_schema_version(&self) -> u16 {
        self.evidence_schema_version
    }

    #[must_use]
    pub const fn rule_id(&self) -> &'static str {
        self.rule_id
    }

    #[must_use]
    pub const fn rule_version(&self) -> (u16, u16, u16) {
        self.rule_version
    }

    #[must_use]
    pub const fn source_hash(&self) -> &Sha256Digest {
        &self.source_hash
    }

    #[must_use]
    pub const fn analysis_output_hash(&self) -> &Sha256Digest {
        &self.analysis_output_hash
    }

    #[must_use]
    pub const fn selection_id(&self) -> &ResourceSelectionId {
        &self.selection_id
    }

    #[must_use]
    pub const fn selection_version(&self) -> ShopResourceVersion {
        self.selection_version
    }

    #[must_use]
    pub const fn stock_allowance_id(&self) -> &StockAllowanceProfileId {
        &self.stock_allowance_id
    }

    #[must_use]
    pub const fn stock_allowance_version(&self) -> ShopResourceVersion {
        self.stock_allowance_version
    }

    #[must_use]
    pub const fn model_extents_mm(&self) -> AxisAlignedEnvelopeMillimeters {
        self.model_extents_mm
    }

    /// Returns total dimensional addition for X/Y/Z, not a per-side allowance.
    #[must_use]
    pub const fn total_allowance_mm(&self) -> [StockAllowanceMillimeters; 3] {
        self.total_allowance_mm
    }

    #[must_use]
    pub const fn blank_dimensions_mm(&self) -> AxisAlignedEnvelopeMillimeters {
        self.blank_dimensions_mm
    }

    #[must_use]
    pub const fn blank_volume_mm3(&self) -> VolumeCubicMillimeters {
        self.blank_volume_mm3
    }

    #[must_use]
    pub const fn removed_volume_mm3(&self) -> VolumeCubicMillimeters {
        self.removed_volume_mm3
    }

    #[must_use]
    pub const fn readiness(&self) -> StockProposalReadiness {
        self.readiness
    }

    #[must_use]
    pub const fn reason_codes(&self) -> &[StockProposalReasonCode; 3] {
        &self.reason_codes
    }
}

/// Visible failure states for the first exact-STEP rectangular proposal rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StockProposalError {
    MultipleSolidBodiesUnsupported,
    InvalidEnvelopeDimension,
    InvalidExactStepDecimal,
    UnsupportedEnvelopeEvidenceVersion,
    PartVolumeExceedsModelEnvelope,
    SelectionNotActiveForProposals,
    SelectionStockAllowanceMismatch,
    StockAllowanceNotReviewed,
    UnsupportedStockForm,
    ArithmeticOverflow,
    ArithmeticRequiresRounding,
    PartVolumeExceedsBlank,
    ResourceRecordNotDraft(&'static str),
    UnsupportedProcessClass,
    MachineEnvelopeMismatch,
}

impl std::fmt::Display for StockProposalError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::MultipleSolidBodiesUnsupported => {
                "rectangular stock proposals require exactly one solid body"
            }
            Self::InvalidEnvelopeDimension => "exact STEP envelope dimensions must be positive",
            Self::InvalidExactStepDecimal => {
                "exact STEP evidence contains an unsupported decimal value"
            }
            Self::UnsupportedEnvelopeEvidenceVersion => {
                "exact STEP envelope evidence version is unsupported"
            }
            Self::PartVolumeExceedsModelEnvelope => {
                "exact STEP part volume exceeds its source-axis envelope"
            }
            Self::SelectionNotActiveForProposals => {
                "resource selection is not active for proposals"
            }
            Self::SelectionStockAllowanceMismatch => {
                "resource selection does not pin the exact stock-allowance version"
            }
            Self::StockAllowanceNotReviewed => {
                "stock-allowance profile is not reviewed or approved"
            }
            Self::UnsupportedStockForm => "stock-envelope rule supports only rectangular stock",
            Self::ArithmeticOverflow => "stock-envelope arithmetic exceeded supported bounds",
            Self::ArithmeticRequiresRounding => {
                "proposal arithmetic requires an unconfigured rounding policy"
            }
            Self::PartVolumeExceedsBlank => "exact STEP part volume exceeds the proposed blank",
            Self::ResourceRecordNotDraft(record) => {
                return write!(
                    formatter,
                    "developer proposal requires draft {record} evidence"
                );
            }
            Self::UnsupportedProcessClass => {
                "developer proposal currently supports only coarse milling profiles"
            }
            Self::MachineEnvelopeMismatch => {
                "proposed rectangular blank exceeds the pinned machine envelope"
            }
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for StockProposalError {}

/// Proposes one source-axis-aligned rectangular blank from exact STEP envelope evidence.
///
/// Each X/Y/Z allowance is interpreted as the **total dimensional addition on that axis**. The
/// rule does not select a standard size, rotate the part, evaluate workholding, inspect current
/// availability, or adopt the proposal into an estimate.
pub fn propose_rectangular_stock(
    envelope: &ExactStepEnvelopeEvidence,
    allowance: &StockAllowanceProfile,
    selection: &ResourceSelection,
) -> Result<RectangularStockProposal, StockProposalError> {
    if selection.state() != ResourceSelectionState::ActiveForProposals {
        return Err(StockProposalError::SelectionNotActiveForProposals);
    }
    if selection.stock_allowance_id() != allowance.id()
        || selection.stock_allowance_version() != allowance.version()
    {
        return Err(StockProposalError::SelectionStockAllowanceMismatch);
    }
    if !matches!(
        allowance.state(),
        LibraryRecordState::Reviewed | LibraryRecordState::Approved
    ) {
        return Err(StockProposalError::StockAllowanceNotReviewed);
    }
    if allowance.stock_form() != StockForm::Rectangular {
        return Err(StockProposalError::UnsupportedStockForm);
    }

    let model = envelope.extents_mm();
    let total_allowance = [
        allowance.x_allowance_mm(),
        allowance.y_allowance_mm(),
        allowance.z_allowance_mm(),
    ];
    let blank = AxisAlignedEnvelopeMillimeters::new(
        add_allowance(model.x(), total_allowance[0])?,
        add_allowance(model.y(), total_allowance[1])?,
        add_allowance(model.z(), total_allowance[2])?,
    );
    let blank_volume = blank.checked_volume()?;
    let removed_volume = blank_volume
        .checked_sub(envelope.part_volume_mm3().value())
        .ok_or(StockProposalError::ArithmeticOverflow)?;
    if removed_volume < Decimal::ZERO {
        return Err(StockProposalError::PartVolumeExceedsBlank);
    }

    Ok(RectangularStockProposal {
        evidence_schema_version: EXACT_STEP_ENVELOPE_EVIDENCE_SCHEMA_VERSION,
        rule_id: RECTANGULAR_STOCK_ENVELOPE_RULE_ID,
        rule_version: RECTANGULAR_STOCK_ENVELOPE_RULE_VERSION,
        source_hash: envelope.source_hash().clone(),
        analysis_output_hash: envelope.analysis_output_hash().clone(),
        selection_id: selection.id().clone(),
        selection_version: selection.version(),
        stock_allowance_id: allowance.id().clone(),
        stock_allowance_version: allowance.version(),
        model_extents_mm: model,
        total_allowance_mm: total_allowance,
        blank_dimensions_mm: blank,
        blank_volume_mm3: VolumeCubicMillimeters::new(blank_volume)
            .map_err(|_| StockProposalError::ArithmeticOverflow)?,
        removed_volume_mm3: VolumeCubicMillimeters::new(removed_volume)
            .map_err(|_| StockProposalError::ArithmeticOverflow)?,
        readiness: StockProposalReadiness::NeedsReview,
        reason_codes: [
            StockProposalReasonCode::AxisAlignedOrientationOnly,
            StockProposalReasonCode::StandardSizeNotResolved,
            StockProposalReasonCode::AvailabilityNotResolved,
        ],
    })
}

fn add_allowance(
    model: CanonicalMillimeterLength,
    allowance: StockAllowanceMillimeters,
) -> Result<CanonicalMillimeterLength, StockProposalError> {
    let value = model
        .value()
        .checked_add(allowance.value())
        .ok_or(StockProposalError::ArithmeticOverflow)?;
    CanonicalMillimeterLength::new(value)
}

/// Produces review-only estimate inputs from one exact STEP envelope and one persisted starter
/// resource-library draft.
///
/// This developer boundary intentionally does not consume the active catalog-selection contract
/// used by [`propose_rectangular_stock`]. It accepts only the separately persisted starter-library
/// draft, retains every source/library version pin, and does not mutate an estimate session.
pub fn propose_developer_estimate_inputs(
    envelope: &ExactStepEnvelopeEvidence,
    library: &ShopResourceLibrary,
) -> Result<DeveloperEstimateInputProposal, StockProposalError> {
    let material = library.material();
    let offer = library.material_offer();
    let allowance = library.stock_allowance();
    let machine = library.machine();
    let runtime = library.runtime();

    require_draft(material.state(), "material")?;
    require_draft(offer.state(), "material-offer")?;
    require_draft(allowance.state(), "stock-allowance")?;
    require_draft(machine.state(), "machine")?;
    require_draft(runtime.state(), "runtime")?;
    if allowance.stock_form() != StockForm::Rectangular {
        return Err(StockProposalError::UnsupportedStockForm);
    }
    if machine.process_class() != ProcessClass::Milling {
        return Err(StockProposalError::UnsupportedProcessClass);
    }

    let model = envelope.extents_mm();
    let blank = AxisAlignedEnvelopeMillimeters::new(
        add_allowance(model.x(), allowance.x_allowance_mm())?,
        add_allowance(model.y(), allowance.y_allowance_mm())?,
        add_allowance(model.z(), allowance.z_allowance_mm())?,
    );
    let derived = derive_blank_inputs(envelope, library, blank)?;

    Ok(DeveloperEstimateInputProposal {
        evidence_schema_version: EXACT_STEP_ENVELOPE_EVIDENCE_SCHEMA_VERSION,
        rule_id: DEVELOPER_ESTIMATE_INPUT_PROPOSAL_RULE_ID,
        rule_version: DEVELOPER_ESTIMATE_INPUT_PROPOSAL_RULE_VERSION,
        source_hash: envelope.source_hash().clone(),
        analysis_output_hash: envelope.analysis_output_hash().clone(),
        library_id: library.id().clone(),
        library_version: library.version(),
        material_id: material.id().clone(),
        material_version: material.version(),
        material_offer_id: offer.id().clone(),
        material_offer_version: offer.version(),
        stock_allowance_id: allowance.id().clone(),
        stock_allowance_version: allowance.version(),
        machine_id: machine.id().clone(),
        machine_version: machine.version(),
        runtime_id: runtime.id().clone(),
        runtime_version: runtime.version(),
        model_extents_mm: model,
        part_volume_mm3: envelope.part_volume_mm3(),
        total_allowance_mm: [
            allowance.x_allowance_mm(),
            allowance.y_allowance_mm(),
            allowance.z_allowance_mm(),
        ],
        blank_dimensions_mm: blank,
        blank_volume_mm3: derived.blank_volume,
        removed_volume_mm3: derived.removed_volume,
        material_density_kg_per_m3: material.density_kg_per_m3(),
        blank_mass_kg: derived.blank_mass,
        material_price_per_kg: offer.price_per_kg().clone(),
        unit_stock_material_cost: derived.unit_material_cost,
        removal_rate_mm3_per_minute: runtime.removal_rate_mm3_per_minute(),
        coarse_times: CoarseTimeProposal {
            setup: runtime.setup_minutes(),
            programming: runtime.programming_minutes(),
            cutting: derived.cutting,
            load_unload: runtime.load_unload_minutes(),
            quality: runtime.inspection_minutes(),
        },
        readiness: DeveloperEstimateProposalReadiness::NeedsReview,
        reason_codes: [
            DeveloperEstimateProposalReasonCode::SessionOnlyDeveloperEvidence,
            DeveloperEstimateProposalReasonCode::DraftShopResourceLibrary,
            DeveloperEstimateProposalReasonCode::AxisAlignedOrientationOnly,
            DeveloperEstimateProposalReasonCode::StandardSizeNotResolved,
            DeveloperEstimateProposalReasonCode::AvailabilityNotResolved,
            DeveloperEstimateProposalReasonCode::CoarseRuntimeNotCam,
            DeveloperEstimateProposalReasonCode::HumanReviewAndAdoptionRequired,
        ],
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BlankDerivedInputs {
    blank_volume: VolumeCubicMillimeters,
    removed_volume: VolumeCubicMillimeters,
    blank_mass: MassKilograms,
    unit_material_cost: Money,
    cutting: RuntimeMinutes,
}

fn derive_blank_inputs(
    envelope: &ExactStepEnvelopeEvidence,
    library: &ShopResourceLibrary,
    blank: AxisAlignedEnvelopeMillimeters,
) -> Result<BlankDerivedInputs, StockProposalError> {
    let machine = library.machine();
    let material = library.material();
    let offer = library.material_offer();
    let runtime = library.runtime();
    if blank.x().value() > machine.envelope_x_mm().value()
        || blank.y().value() > machine.envelope_y_mm().value()
        || blank.z().value() > machine.envelope_z_mm().value()
    {
        return Err(StockProposalError::MachineEnvelopeMismatch);
    }

    let blank_volume = checked_product3(
        blank.x().value(),
        blank.y().value(),
        blank.z().value(),
        "developer proposal blank volume",
    )?;
    let removed_volume = checked_decimal_sub_exact(
        blank_volume,
        envelope.part_volume_mm3().value(),
        "developer proposal removed volume",
    )
    .map_err(map_decimal_error)?;
    if removed_volume < Decimal::ZERO {
        return Err(StockProposalError::PartVolumeExceedsBlank);
    }

    let density_kg_per_mm3 = checked_div_exact(
        material.density_kg_per_m3().value(),
        Decimal::from(1_000_000_000_u64),
        "developer proposal density conversion",
    )?;
    let blank_mass = checked_decimal_mul_exact(
        blank_volume,
        density_kg_per_mm3,
        "developer proposal blank mass",
    )
    .map_err(map_decimal_error)?;
    let unit_stock_material_cost = offer
        .price_per_kg()
        .checked_mul(blank_mass)
        .map_err(map_decimal_error)?;
    let cutting_minutes = checked_div_exact(
        removed_volume,
        runtime.removal_rate_mm3_per_minute().value(),
        "developer proposal cutting minutes",
    )?;

    Ok(BlankDerivedInputs {
        blank_volume: VolumeCubicMillimeters::new(blank_volume)
            .map_err(|_| StockProposalError::ArithmeticOverflow)?,
        removed_volume: VolumeCubicMillimeters::new(removed_volume)
            .map_err(|_| StockProposalError::ArithmeticOverflow)?,
        blank_mass: MassKilograms::new(blank_mass)
            .map_err(|_| StockProposalError::ArithmeticOverflow)?,
        unit_material_cost: unit_stock_material_cost,
        cutting: RuntimeMinutes::new(cutting_minutes)
            .map_err(|_| StockProposalError::ArithmeticOverflow)?,
    })
}

fn require_draft(
    state: LibraryRecordState,
    record: &'static str,
) -> Result<(), StockProposalError> {
    if state == LibraryRecordState::Draft {
        Ok(())
    } else {
        Err(StockProposalError::ResourceRecordNotDraft(record))
    }
}

fn checked_product3(
    x: Decimal,
    y: Decimal,
    z: Decimal,
    operation: &'static str,
) -> Result<Decimal, StockProposalError> {
    let xy = checked_decimal_mul_exact(x, y, operation).map_err(map_decimal_error)?;
    checked_decimal_mul_exact(xy, z, operation).map_err(map_decimal_error)
}

fn checked_div_exact(
    numerator: Decimal,
    denominator: Decimal,
    operation: &'static str,
) -> Result<Decimal, StockProposalError> {
    if denominator.is_zero() {
        return Err(StockProposalError::ArithmeticOverflow);
    }
    let quotient = numerator
        .checked_div(denominator)
        .ok_or(StockProposalError::ArithmeticOverflow)?;
    let reconstructed =
        checked_decimal_mul_exact(quotient, denominator, operation).map_err(map_decimal_error)?;
    if reconstructed == numerator {
        Ok(quotient)
    } else {
        Err(StockProposalError::ArithmeticRequiresRounding)
    }
}

fn map_decimal_error(error: partprobe_domain::DomainError) -> StockProposalError {
    match error {
        partprobe_domain::DomainError::RoundingRequired { .. } => {
            StockProposalError::ArithmeticRequiresRounding
        }
        _ => StockProposalError::ArithmeticOverflow,
    }
}
