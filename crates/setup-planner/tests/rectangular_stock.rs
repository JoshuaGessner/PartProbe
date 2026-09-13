use partprobe_domain::{
    ActorId, LibraryRecordState, MachineProfileId, MaterialDefinitionId, MaterialOfferId,
    RecordedAt, ResourceSelection, ResourceSelectionId, ResourceSelectionState, RuntimeProfileId,
    ShopResourceVersion, SourceKind, SourceRef, StockAllowanceMillimeters, StockAllowanceProfile,
    StockAllowanceProfileId, StockForm,
};
use partprobe_geometry_core::{
    ExactStepEnvelopeDerivative, ProvisionalGeometryDecimal, ProvisionalGeometrySnapshot,
    Sha256Digest,
};
use partprobe_geometry_import::ProvisionalExactStepAnalysis;
use partprobe_setup_planner::{
    EXACT_STEP_ENVELOPE_EVIDENCE_SCHEMA_VERSION, ExactStepEnvelopeEvidence,
    RECTANGULAR_STOCK_ENVELOPE_RULE_ID, RECTANGULAR_STOCK_ENVELOPE_RULE_VERSION,
    StockProposalError, StockProposalReadiness, StockProposalReasonCode, propose_rectangular_stock,
};
use rust_decimal::Decimal;

fn decimal(value: &str) -> Decimal {
    Decimal::from_str_exact(value).expect("test decimal must be exact")
}

fn envelope(x: &str, y: &str, z: &str, part_volume: &str) -> ExactStepEnvelopeEvidence {
    let source_hash = Sha256Digest::new("a".repeat(64)).expect("source hash must be valid");
    let geometry_decimal =
        |value| ProvisionalGeometryDecimal::new(value).expect("geometry decimal must be canonical");
    let snapshot = ProvisionalGeometrySnapshot::new(
        source_hash.clone(),
        "8.0.0",
        3,
        1,
        1,
        geometry_decimal("1"),
        geometry_decimal(part_volume),
        [
            geometry_decimal("0"),
            geometry_decimal("0"),
            geometry_decimal("0"),
        ],
    )
    .expect("test snapshot must be valid");
    let derivative = ExactStepEnvelopeDerivative::new(
        source_hash,
        [
            geometry_decimal(x),
            geometry_decimal(y),
            geometry_decimal(z),
        ],
    )
    .expect("test envelope derivative must be valid");
    let analysis = ProvisionalExactStepAnalysis::new(snapshot, derivative)
        .expect("test controlled analysis must be valid");
    ExactStepEnvelopeEvidence::from_controlled_analysis(
        &analysis,
        Sha256Digest::new("b".repeat(64)).expect("output hash must be valid"),
    )
    .expect("test envelope must enclose its part")
}

fn source() -> SourceRef {
    SourceRef::new(
        SourceKind::Manual,
        "shop-stock-review",
        Some("1".to_owned()),
        Some(RecordedAt::new("2026-09-10T12:00:00Z").expect("time must be valid")),
    )
    .expect("source must be valid")
}

fn allowance_at_version(
    id: &str,
    version: u32,
    form: StockForm,
    state: LibraryRecordState,
) -> StockAllowanceProfile {
    StockAllowanceProfile::new(
        StockAllowanceProfileId::new(id).expect("allowance ID must be valid"),
        ShopResourceVersion::new(version).expect("version must be valid"),
        form,
        StockAllowanceMillimeters::new(decimal("3")).expect("allowance must be valid"),
        StockAllowanceMillimeters::new(decimal("3")).expect("allowance must be valid"),
        StockAllowanceMillimeters::new(decimal("2")).expect("allowance must be valid"),
        source(),
        state,
    )
}

fn allowance(id: &str, form: StockForm, state: LibraryRecordState) -> StockAllowanceProfile {
    allowance_at_version(id, 1, form, state)
}

fn selection(stock_id: &str, state: ResourceSelectionState) -> ResourceSelection {
    let version = ShopResourceVersion::new(1).expect("version must be valid");
    ResourceSelection::new(
        ResourceSelectionId::new("proposal-selection").expect("selection ID must be valid"),
        version,
        MaterialDefinitionId::new("al-6061-t6").expect("material ID must be valid"),
        version,
        MaterialOfferId::new("al-offer").expect("offer ID must be valid"),
        version,
        StockAllowanceProfileId::new(stock_id).expect("stock ID must be valid"),
        version,
        MachineProfileId::new("vmc-1").expect("machine ID must be valid"),
        version,
        RuntimeProfileId::new("vmc-al").expect("runtime ID must be valid"),
        version,
        state,
        ActorId::new("reviewer").expect("actor must be valid"),
        RecordedAt::new("2026-09-10T12:30:00Z").expect("time must be valid"),
        "reviewed exact resource chain for proposal eligibility",
    )
    .expect("selection must be valid")
}

#[test]
fn exact_cube_uses_total_axis_allowance_and_exposes_review_reasons() {
    let policy = allowance(
        "rectangular-default",
        StockForm::Rectangular,
        LibraryRecordState::Reviewed,
    );
    let active = selection(
        "rectangular-default",
        ResourceSelectionState::ActiveForProposals,
    );

    let proposal = propose_rectangular_stock(&envelope("10", "10", "10", "1000"), &policy, &active)
        .expect("reviewed rectangular policy must produce a proposal");

    assert_eq!(
        proposal.evidence_schema_version(),
        EXACT_STEP_ENVELOPE_EVIDENCE_SCHEMA_VERSION
    );
    assert_eq!(proposal.rule_id(), RECTANGULAR_STOCK_ENVELOPE_RULE_ID);
    assert_eq!(
        proposal.rule_version(),
        RECTANGULAR_STOCK_ENVELOPE_RULE_VERSION
    );
    assert_eq!(proposal.source_hash().as_str(), "a".repeat(64));
    assert_eq!(proposal.analysis_output_hash().as_str(), "b".repeat(64));
    assert_eq!(proposal.selection_id().as_str(), "proposal-selection");
    assert_eq!(proposal.selection_version().value(), 1);
    assert_eq!(
        proposal.stock_allowance_id().as_str(),
        "rectangular-default"
    );
    assert_eq!(proposal.stock_allowance_version().value(), 1);
    assert_eq!(proposal.blank_dimensions_mm().x().value(), decimal("13"));
    assert_eq!(proposal.blank_dimensions_mm().y().value(), decimal("13"));
    assert_eq!(proposal.blank_dimensions_mm().z().value(), decimal("12"));
    assert_eq!(proposal.blank_volume_mm3().value(), decimal("2028"));
    assert_eq!(proposal.removed_volume_mm3().value(), decimal("1028"));
    assert_eq!(proposal.readiness(), StockProposalReadiness::NeedsReview);
    assert_eq!(
        proposal.reason_codes(),
        &[
            StockProposalReasonCode::AxisAlignedOrientationOnly,
            StockProposalReasonCode::StandardSizeNotResolved,
            StockProposalReasonCode::AvailabilityNotResolved,
        ]
    );
}

#[test]
fn analytic_prism_changes_blank_and_removed_volume_under_the_same_policy() {
    let policy = allowance(
        "rectangular-default",
        StockForm::Rectangular,
        LibraryRecordState::Approved,
    );
    let active = selection(
        "rectangular-default",
        ResourceSelectionState::ActiveForProposals,
    );

    let cube = propose_rectangular_stock(&envelope("10", "10", "10", "1000"), &policy, &active)
        .expect("cube proposal must succeed");
    let prism = propose_rectangular_stock(&envelope("12", "8", "5", "480"), &policy, &active)
        .expect("prism proposal must succeed");

    assert_eq!(prism.blank_dimensions_mm().x().value(), decimal("15"));
    assert_eq!(prism.blank_dimensions_mm().y().value(), decimal("11"));
    assert_eq!(prism.blank_dimensions_mm().z().value(), decimal("7"));
    assert_eq!(prism.blank_volume_mm3().value(), decimal("1155"));
    assert_eq!(prism.removed_volume_mm3().value(), decimal("675"));
    assert_ne!(cube.blank_volume_mm3(), prism.blank_volume_mm3());
    assert_ne!(cube.removed_volume_mm3(), prism.removed_volume_mm3());
}

#[test]
fn reviewed_but_inactive_selection_cannot_generate_a_proposal() {
    let policy = allowance(
        "rectangular-default",
        StockForm::Rectangular,
        LibraryRecordState::Reviewed,
    );
    let reviewed = selection("rectangular-default", ResourceSelectionState::Reviewed);

    assert_eq!(
        propose_rectangular_stock(&envelope("10", "10", "10", "1000"), &policy, &reviewed),
        Err(StockProposalError::SelectionNotActiveForProposals)
    );
}

#[test]
fn selection_must_pin_the_exact_allowance_identity_and_version() {
    let policy = allowance(
        "rectangular-default",
        StockForm::Rectangular,
        LibraryRecordState::Reviewed,
    );
    let active = selection("another-policy", ResourceSelectionState::ActiveForProposals);

    assert_eq!(
        propose_rectangular_stock(&envelope("10", "10", "10", "1000"), &policy, &active),
        Err(StockProposalError::SelectionStockAllowanceMismatch)
    );

    let version_two_policy = allowance_at_version(
        "rectangular-default",
        2,
        StockForm::Rectangular,
        LibraryRecordState::Reviewed,
    );
    let version_one_selection = selection(
        "rectangular-default",
        ResourceSelectionState::ActiveForProposals,
    );
    assert_eq!(
        propose_rectangular_stock(
            &envelope("10", "10", "10", "1000"),
            &version_two_policy,
            &version_one_selection,
        ),
        Err(StockProposalError::SelectionStockAllowanceMismatch)
    );
}

#[test]
fn draft_or_non_rectangular_allowance_remains_visibly_inapplicable() {
    let active = selection(
        "rectangular-default",
        ResourceSelectionState::ActiveForProposals,
    );
    let draft = allowance(
        "rectangular-default",
        StockForm::Rectangular,
        LibraryRecordState::Draft,
    );
    let round = allowance(
        "rectangular-default",
        StockForm::Round,
        LibraryRecordState::Reviewed,
    );

    assert_eq!(
        propose_rectangular_stock(&envelope("10", "10", "10", "1000"), &draft, &active),
        Err(StockProposalError::StockAllowanceNotReviewed)
    );
    assert_eq!(
        propose_rectangular_stock(&envelope("10", "10", "10", "1000"), &round, &active),
        Err(StockProposalError::UnsupportedStockForm)
    );
}

#[test]
fn impossible_exact_envelope_evidence_is_rejected_before_proposal() {
    let source_hash = Sha256Digest::new("a".repeat(64)).expect("source hash must be valid");
    let geometry_decimal =
        |value| ProvisionalGeometryDecimal::new(value).expect("geometry decimal must be canonical");
    let snapshot = ProvisionalGeometrySnapshot::new(
        source_hash.clone(),
        "8.0.0",
        3,
        1,
        1,
        geometry_decimal("600"),
        geometry_decimal("1000.1"),
        [
            geometry_decimal("5"),
            geometry_decimal("5"),
            geometry_decimal("5"),
        ],
    )
    .expect("snapshot alone does not assert envelope closure");
    let derivative = ExactStepEnvelopeDerivative::new(
        source_hash,
        [
            geometry_decimal("10"),
            geometry_decimal("10"),
            geometry_decimal("10"),
        ],
    )
    .expect("positive derivative must be valid");
    let analysis = ProvisionalExactStepAnalysis::new(snapshot, derivative)
        .expect("source-consistent analysis must be valid");
    let result = ExactStepEnvelopeEvidence::from_controlled_analysis(
        &analysis,
        Sha256Digest::new("b".repeat(64)).expect("output hash must be valid"),
    );

    assert_eq!(
        result,
        Err(StockProposalError::PartVolumeExceedsModelEnvelope)
    );
}

#[test]
fn multi_body_exact_analysis_is_rejected_before_proposal() {
    let source_hash = Sha256Digest::new("a".repeat(64)).expect("source hash must be valid");
    let geometry_decimal =
        |value| ProvisionalGeometryDecimal::new(value).expect("geometry decimal must be canonical");
    let snapshot = ProvisionalGeometrySnapshot::new(
        source_hash.clone(),
        "8.0.0",
        4,
        1,
        2,
        geometry_decimal("600"),
        geometry_decimal("1000"),
        [
            geometry_decimal("5"),
            geometry_decimal("5"),
            geometry_decimal("5"),
        ],
    )
    .expect("multi-body snapshot remains valid geometry evidence");
    let derivative = ExactStepEnvelopeDerivative::new(
        source_hash,
        [
            geometry_decimal("10"),
            geometry_decimal("10"),
            geometry_decimal("10"),
        ],
    )
    .expect("positive derivative must be valid");
    let analysis = ProvisionalExactStepAnalysis::new(snapshot, derivative)
        .expect("source-consistent analysis must be valid");

    assert_eq!(
        ExactStepEnvelopeEvidence::from_controlled_analysis(
            &analysis,
            Sha256Digest::new("b".repeat(64)).expect("output hash must be valid"),
        ),
        Err(StockProposalError::MultipleSolidBodiesUnsupported)
    );
}
