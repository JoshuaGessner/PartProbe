use partprobe_application::{AnalyzedGeometryEvidence, ExactStepStockProposalApplication};
use partprobe_domain::{
    ActorId, CoarseRuntimeProfile, CurrencyCode, DensityKilogramsPerCubicMeter, EffectiveDate,
    LibraryRecordState, MachineEnvelopeMillimeters, MachineProfile, MachineProfileId,
    MaterialDefinition, MaterialDefinitionId, MaterialOffer, MaterialOfferId, Money, ProcessClass,
    RecordedAt, RemovalRateCubicMillimetersPerMinute, ResourceSelection, ResourceSelectionId,
    ResourceSelectionState, RuntimeMinutes, RuntimeProfileId, ShopResourceCatalog,
    ShopResourceCatalogId, ShopResourceVersion, SourceKind, SourceRef, StockAllowanceMillimeters,
    StockAllowanceProfile, StockAllowanceProfileId, StockForm, ValueState,
};
use partprobe_geometry_core::{
    ExactStepEnvelopeDerivative, ProvisionalGeometryDecimal, ProvisionalGeometrySnapshot,
    Sha256Digest, StageStatus,
};
use partprobe_geometry_import::{
    ControlledGeometryResult, PROVISIONAL_EXACT_STEP_ANALYSIS_REFERENCE,
    PROVISIONAL_GEOMETRY_SNAPSHOT_REFERENCE, ProvisionalExactStepAnalysis, SnapshotReference,
};
use rust_decimal::Decimal;

const SOURCE_HASH: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const OUTPUT_HASH: &str = "2222222222222222222222222222222222222222222222222222222222222222";

fn decimal(value: &str) -> Decimal {
    Decimal::from_str_exact(value).expect("test decimal must be exact")
}

fn geometry_decimal(value: &str) -> ProvisionalGeometryDecimal {
    ProvisionalGeometryDecimal::new(value).expect("geometry decimal must be canonical")
}

fn digest(value: &str) -> Sha256Digest {
    Sha256Digest::new(value).expect("test hash must be valid")
}

fn source(label: &str) -> SourceRef {
    SourceRef::new(
        SourceKind::Manual,
        label,
        Some("1".to_owned()),
        Some(RecordedAt::new("2026-09-10T12:00:00Z").expect("time must be valid")),
    )
    .expect("source must be valid")
}

fn reviewed_catalog(selection_state: ResourceSelectionState) -> ShopResourceCatalog {
    let version = ShopResourceVersion::new(1).expect("version must be valid");
    let material_id = MaterialDefinitionId::new("al-6061-t6").expect("material ID must be valid");
    let offer_id = MaterialOfferId::new("al-offer").expect("offer ID must be valid");
    let allowance_id =
        StockAllowanceProfileId::new("rectangular-default").expect("stock ID must be valid");
    let machine_id = MachineProfileId::new("vmc-1").expect("machine ID must be valid");
    let runtime_id = RuntimeProfileId::new("vmc-al").expect("runtime ID must be valid");
    let material = MaterialDefinition::new(
        material_id.clone(),
        version,
        "Aluminum",
        "6061",
        Some("ASTM B221".to_owned()),
        Some("T6".to_owned()),
        DensityKilogramsPerCubicMeter::new(decimal("2700")).expect("density must be valid"),
        source("material-handbook"),
        LibraryRecordState::Reviewed,
    )
    .expect("material must be valid");
    let offer = MaterialOffer::new(
        offer_id.clone(),
        version,
        material_id.clone(),
        version,
        "test supplier",
        Money::new(
            decimal("8.50"),
            CurrencyCode::new("USD").expect("currency must be valid"),
        ),
        EffectiveDate::new("2026-09-10").expect("date must be valid"),
        source("supplier-test-quote"),
        LibraryRecordState::Reviewed,
    )
    .expect("offer must be valid");
    let allowance = StockAllowanceProfile::new(
        allowance_id.clone(),
        version,
        StockForm::Rectangular,
        StockAllowanceMillimeters::new(decimal("3")).expect("allowance must be valid"),
        StockAllowanceMillimeters::new(decimal("3")).expect("allowance must be valid"),
        StockAllowanceMillimeters::new(decimal("2")).expect("allowance must be valid"),
        source("shop-stock-review"),
        LibraryRecordState::Reviewed,
    );
    let machine = MachineProfile::new(
        machine_id.clone(),
        version,
        "Test VMC",
        ProcessClass::Milling,
        MachineEnvelopeMillimeters::new(decimal("762")).expect("envelope must be valid"),
        MachineEnvelopeMillimeters::new(decimal("508")).expect("envelope must be valid"),
        MachineEnvelopeMillimeters::new(decimal("508")).expect("envelope must be valid"),
        source("machine-manual"),
        LibraryRecordState::Reviewed,
    )
    .expect("machine must be valid");
    let runtime = CoarseRuntimeProfile::new(
        runtime_id.clone(),
        version,
        machine_id.clone(),
        version,
        material_id.clone(),
        version,
        RemovalRateCubicMillimetersPerMinute::new(decimal("16000"))
            .expect("removal rate must be valid"),
        RuntimeMinutes::new(decimal("60")).expect("setup must be valid"),
        RuntimeMinutes::new(decimal("45")).expect("programming must be valid"),
        RuntimeMinutes::new(decimal("3")).expect("load time must be valid"),
        RuntimeMinutes::new(decimal("15")).expect("inspection must be valid"),
        source("runtime-review"),
        LibraryRecordState::Reviewed,
    );
    let selection = ResourceSelection::new(
        ResourceSelectionId::new("proposal-selection").expect("selection ID must be valid"),
        version,
        material_id,
        version,
        offer_id,
        version,
        allowance_id,
        version,
        machine_id,
        version,
        runtime_id,
        version,
        selection_state,
        ActorId::new("reviewer").expect("actor must be valid"),
        RecordedAt::new("2026-09-10T12:30:00Z").expect("time must be valid"),
        "reviewed exact chain for proposal eligibility",
    )
    .expect("selection must be valid");
    ShopResourceCatalog::new(
        ShopResourceCatalogId::new("shop-catalog").expect("catalog ID must be valid"),
        version,
        CurrencyCode::new("USD").expect("currency must be valid"),
        vec![material],
        vec![offer],
        vec![allowance],
        vec![machine],
        vec![runtime],
        vec![selection],
    )
    .expect("catalog must be internally consistent")
}

fn snapshot() -> ProvisionalGeometrySnapshot {
    ProvisionalGeometrySnapshot::new(
        digest(SOURCE_HASH),
        "8.0.0",
        3,
        1,
        1,
        geometry_decimal("392"),
        geometry_decimal("480"),
        [
            geometry_decimal("6"),
            geometry_decimal("4"),
            geometry_decimal("2.5"),
        ],
    )
    .expect("analytic prism snapshot must be valid")
}

fn geometry_with_envelope() -> AnalyzedGeometryEvidence {
    let envelope = ExactStepEnvelopeDerivative::new(
        digest(SOURCE_HASH),
        [
            geometry_decimal("12"),
            geometry_decimal("8"),
            geometry_decimal("5"),
        ],
    )
    .expect("analytic prism envelope must be valid");
    let analysis = ProvisionalExactStepAnalysis::new(snapshot(), envelope)
        .expect("source-bound analysis must be valid");
    AnalyzedGeometryEvidence::new(
        StageStatus::Succeeded,
        Vec::new(),
        SnapshotReference::new(PROVISIONAL_EXACT_STEP_ANALYSIS_REFERENCE)
            .expect("reference must be valid"),
        digest(OUTPUT_HASH),
        512,
        ControlledGeometryResult::ExactBrepWithEnvelope(Box::new(analysis)),
        None,
        None,
    )
    .expect("analyzed evidence must be valid")
}

fn legacy_geometry() -> AnalyzedGeometryEvidence {
    AnalyzedGeometryEvidence::new(
        StageStatus::Succeeded,
        Vec::new(),
        SnapshotReference::new(PROVISIONAL_GEOMETRY_SNAPSHOT_REFERENCE)
            .expect("reference must be valid"),
        digest(OUTPUT_HASH),
        256,
        ControlledGeometryResult::ExactBrep(Box::new(snapshot())),
        None,
        None,
    )
    .expect("legacy analyzed evidence must remain valid")
}

#[test]
fn application_resolves_exact_active_catalog_evidence_into_a_reviewable_proposal() {
    let result = ExactStepStockProposalApplication.propose(
        &geometry_with_envelope(),
        &reviewed_catalog(ResourceSelectionState::ActiveForProposals),
    );

    let ValueState::Available { value } = result else {
        panic!("controlled envelope plus active catalog must produce reviewable evidence")
    };
    assert_eq!(value.source_hash().as_str(), SOURCE_HASH);
    assert_eq!(value.analysis_output_hash().as_str(), OUTPUT_HASH);
    assert_eq!(value.blank_dimensions_mm().x().value(), decimal("15"));
    assert_eq!(value.blank_dimensions_mm().y().value(), decimal("11"));
    assert_eq!(value.blank_dimensions_mm().z().value(), decimal("7"));
    assert_eq!(value.removed_volume_mm3().value(), decimal("675"));
}

#[test]
fn application_preserves_missing_derivative_and_inactive_selection_as_unavailable() {
    assert_eq!(
        ExactStepStockProposalApplication.propose(
            &legacy_geometry(),
            &reviewed_catalog(ResourceSelectionState::ActiveForProposals),
        ),
        ValueState::Unavailable {
            reason: "exact STEP envelope evidence is unavailable".to_owned(),
        }
    );
    assert_eq!(
        ExactStepStockProposalApplication.propose(
            &geometry_with_envelope(),
            &reviewed_catalog(ResourceSelectionState::Reviewed),
        ),
        ValueState::Unavailable {
            reason: "no resource selection is active for proposals".to_owned(),
        }
    );
}
