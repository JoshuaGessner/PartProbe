use partprobe_application::{
    AnalyzedGeometryEvidence, DeveloperEstimateProposalAdoptionError,
    DeveloperEstimateProposalApplication, DeveloperEstimateProposalReview, DraftQuantityInputs,
};
use partprobe_domain::{
    ActorId, CoarseRuntimeProfile, CurrencyCode, DensityKilogramsPerCubicMeter, EffectiveDate,
    LibraryRecordState, MachineEnvelopeMillimeters, MachineProfile, MachineProfileId,
    MaterialDefinition, MaterialDefinitionId, MaterialOffer, MaterialOfferId, Money, ProcessClass,
    RecordedAt, RemovalRateCubicMillimetersPerMinute, RuntimeMinutes, RuntimeProfileId,
    ShopProfileId, ShopResourceLibrary, ShopResourceLibraryId, ShopResourceVersion,
    ShopSettingsDraft, ShopSettingsRevision, SourceKind, SourceRef, StockAllowanceMillimeters,
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
use partprobe_setup_planner::{
    DEVELOPER_ESTIMATE_INPUT_PROPOSAL_RULE_ID, DEVELOPER_ESTIMATE_INPUT_PROPOSAL_RULE_VERSION,
    DeveloperEstimateProposalReadiness, DeveloperEstimateProposalReasonCode,
    EXACT_STEP_ENVELOPE_EVIDENCE_SCHEMA_VERSION,
};
use rust_decimal::Decimal;

const SOURCE_HASH: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const OUTPUT_HASH: &str = "2222222222222222222222222222222222222222222222222222222222222222";

fn decimal(value: &str) -> Decimal {
    Decimal::from_str_exact(value).expect("test decimal must be exact")
}

fn geometry_decimal(value: &str) -> ProvisionalGeometryDecimal {
    ProvisionalGeometryDecimal::new(value).expect("geometry decimal must be valid")
}

fn digest(value: &str) -> Sha256Digest {
    Sha256Digest::new(value).expect("test hash must be valid")
}

fn source(label: &str) -> SourceRef {
    SourceRef::new(
        SourceKind::Manual,
        label,
        Some("test-v1".to_owned()),
        Some(RecordedAt::new("2026-09-10T12:00:00Z").expect("time must be valid")),
    )
    .expect("source must be valid")
}

fn snapshot(
    surface_area_mm2: &str,
    volume_mm3: &str,
    center_of_mass_mm: [&str; 3],
) -> ProvisionalGeometrySnapshot {
    ProvisionalGeometrySnapshot::new(
        digest(SOURCE_HASH),
        "8.0.0",
        3,
        1,
        1,
        geometry_decimal(surface_area_mm2),
        geometry_decimal(volume_mm3),
        center_of_mass_mm.map(geometry_decimal),
    )
    .expect("snapshot must be valid")
}

fn exact_geometry(
    extents_mm: [&str; 3],
    surface_area_mm2: &str,
    volume_mm3: &str,
    center_of_mass_mm: [&str; 3],
) -> AnalyzedGeometryEvidence {
    let snapshot = snapshot(surface_area_mm2, volume_mm3, center_of_mass_mm);
    let envelope =
        ExactStepEnvelopeDerivative::new(digest(SOURCE_HASH), extents_mm.map(geometry_decimal))
            .expect("envelope must be valid");
    let analysis = ProvisionalExactStepAnalysis::new(snapshot, envelope)
        .expect("analysis must be source-bound");
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
        ControlledGeometryResult::ExactBrep(Box::new(snapshot("600", "1000", ["5", "5", "5"]))),
        None,
        None,
    )
    .expect("legacy evidence must be valid")
}

fn library(
    record_state: LibraryRecordState,
    stock_form: StockForm,
    process_class: ProcessClass,
    machine_envelope_mm: [&str; 3],
) -> ShopResourceLibrary {
    let material_version = ShopResourceVersion::new(2).expect("version must be valid");
    let offer_version = ShopResourceVersion::new(3).expect("version must be valid");
    let allowance_version = ShopResourceVersion::new(4).expect("version must be valid");
    let machine_version = ShopResourceVersion::new(5).expect("version must be valid");
    let runtime_version = ShopResourceVersion::new(6).expect("version must be valid");
    let material_id = MaterialDefinitionId::new("al-6061-t6").expect("ID must be valid");
    let machine_id = MachineProfileId::new("vmc-1").expect("ID must be valid");
    let material = MaterialDefinition::new(
        material_id.clone(),
        material_version,
        "Aluminum",
        "6061",
        None,
        Some("T6".to_owned()),
        DensityKilogramsPerCubicMeter::new(decimal("2700")).expect("density must be valid"),
        source("material-test"),
        record_state,
    )
    .expect("material must be valid");
    let offer = MaterialOffer::new(
        MaterialOfferId::new("al-offer").expect("ID must be valid"),
        offer_version,
        material_id.clone(),
        material_version,
        "test supplier",
        Money::new(
            decimal("8.50"),
            CurrencyCode::new("USD").expect("currency must be valid"),
        ),
        EffectiveDate::new("2026-09-10").expect("date must be valid"),
        source("offer-test"),
        record_state,
    )
    .expect("offer must be valid");
    let allowance = StockAllowanceProfile::new(
        StockAllowanceProfileId::new("rectangular-default").expect("ID must be valid"),
        allowance_version,
        stock_form,
        StockAllowanceMillimeters::new(decimal("3")).expect("allowance must be valid"),
        StockAllowanceMillimeters::new(decimal("3")).expect("allowance must be valid"),
        StockAllowanceMillimeters::new(decimal("2")).expect("allowance must be valid"),
        source("allowance-test"),
        record_state,
    );
    let machine = MachineProfile::new(
        machine_id.clone(),
        machine_version,
        "Test VMC",
        process_class,
        MachineEnvelopeMillimeters::new(decimal(machine_envelope_mm[0]))
            .expect("envelope must be valid"),
        MachineEnvelopeMillimeters::new(decimal(machine_envelope_mm[1]))
            .expect("envelope must be valid"),
        MachineEnvelopeMillimeters::new(decimal(machine_envelope_mm[2]))
            .expect("envelope must be valid"),
        source("machine-test"),
        record_state,
    )
    .expect("machine must be valid");
    let runtime = CoarseRuntimeProfile::new(
        RuntimeProfileId::new("vmc-al").expect("ID must be valid"),
        runtime_version,
        machine_id,
        machine_version,
        material_id,
        material_version,
        RemovalRateCubicMillimetersPerMinute::new(decimal("16000"))
            .expect("removal rate must be valid"),
        RuntimeMinutes::new(decimal("60")).expect("setup must be valid"),
        RuntimeMinutes::new(decimal("45")).expect("programming must be valid"),
        RuntimeMinutes::new(decimal("3")).expect("handling must be valid"),
        RuntimeMinutes::new(decimal("15")).expect("quality must be valid"),
        source("runtime-test"),
        record_state,
    );
    ShopResourceLibrary::new(
        ShopResourceLibraryId::new("starter-library").expect("ID must be valid"),
        ShopResourceVersion::new(9).expect("version must be valid"),
        CurrencyCode::new("USD").expect("currency must be valid"),
        material,
        offer,
        allowance,
        machine,
        runtime,
    )
    .expect("library must be internally consistent")
}

fn settings(resources: Option<ShopResourceLibrary>) -> ShopSettingsDraft {
    ShopSettingsDraft::new_with_resources(
        ShopProfileId::new("test-shop").expect("profile must be valid"),
        ShopSettingsRevision::new(7).expect("revision must be valid"),
        CurrencyCode::new("USD").expect("currency must be valid"),
        None,
        None,
        resources,
        ActorId::new("test-editor").expect("actor must be valid"),
        RecordedAt::new("2026-09-10T13:00:00Z").expect("time must be valid"),
        "persisted developer-demo resource draft",
    )
    .expect("settings must be valid")
}

fn valid_library() -> ShopResourceLibrary {
    library(
        LibraryRecordState::Draft,
        StockForm::Rectangular,
        ProcessClass::Milling,
        ["500", "500", "500"],
    )
}

#[test]
fn developer_proposal_pins_evidence_and_derives_exact_cost_and_coarse_times() {
    let result = DeveloperEstimateProposalApplication.propose(
        &exact_geometry(["12", "8", "5"], "392", "480", ["6", "4", "2.5"]),
        &settings(Some(valid_library())),
    );
    let ValueState::Available { value } = result else {
        panic!("valid exact evidence and persisted draft must produce a proposal")
    };

    assert_eq!(
        value.evidence_schema_version(),
        EXACT_STEP_ENVELOPE_EVIDENCE_SCHEMA_VERSION
    );
    assert_eq!(value.rule_id(), DEVELOPER_ESTIMATE_INPUT_PROPOSAL_RULE_ID);
    assert_eq!(
        value.rule_version(),
        DEVELOPER_ESTIMATE_INPUT_PROPOSAL_RULE_VERSION
    );
    assert_eq!(value.source_hash().as_str(), SOURCE_HASH);
    assert_eq!(value.analysis_output_hash().as_str(), OUTPUT_HASH);
    assert_eq!(value.library_id().as_str(), "starter-library");
    assert_eq!(value.library_version().value(), 9);
    assert_eq!(value.material_id().as_str(), "al-6061-t6");
    assert_eq!(value.material_version().value(), 2);
    assert_eq!(value.material_offer_id().as_str(), "al-offer");
    assert_eq!(value.material_offer_version().value(), 3);
    assert_eq!(value.stock_allowance_id().as_str(), "rectangular-default");
    assert_eq!(value.stock_allowance_version().value(), 4);
    assert_eq!(value.machine_id().as_str(), "vmc-1");
    assert_eq!(value.machine_version().value(), 5);
    assert_eq!(value.runtime_id().as_str(), "vmc-al");
    assert_eq!(value.runtime_version().value(), 6);

    assert_eq!(
        value
            .total_allowance_mm()
            .map(|allowance| allowance.value()),
        [decimal("3"), decimal("3"), decimal("2")]
    );
    assert_eq!(value.blank_dimensions_mm().x().value(), decimal("15"));
    assert_eq!(value.blank_dimensions_mm().y().value(), decimal("11"));
    assert_eq!(value.blank_dimensions_mm().z().value(), decimal("7"));
    assert_eq!(value.blank_volume_mm3().value(), decimal("1155"));
    assert_eq!(value.removed_volume_mm3().value(), decimal("675"));
    assert_eq!(value.material_density_kg_per_m3().value(), decimal("2700"));
    assert_eq!(value.blank_mass_kg().value(), decimal("0.0031185"));
    assert_eq!(value.material_price_per_kg().amount(), decimal("8.50"));
    assert_eq!(
        value.unit_stock_material_cost().amount(),
        decimal("0.026507250")
    );
    assert_eq!(value.unit_stock_material_cost().currency().as_str(), "USD");
    assert_eq!(
        value.removal_rate_mm3_per_minute().value(),
        decimal("16000")
    );
    assert_eq!(value.coarse_times().setup().value(), decimal("60"));
    assert_eq!(value.coarse_times().programming().value(), decimal("45"));
    assert_eq!(value.coarse_times().cutting().value(), decimal("0.0421875"));
    assert_eq!(value.coarse_times().load_unload().value(), decimal("3"));
    assert_eq!(value.coarse_times().quality().value(), decimal("15"));
    assert_eq!(
        value.readiness(),
        DeveloperEstimateProposalReadiness::NeedsReview
    );
    assert!(
        value
            .reason_codes()
            .contains(&DeveloperEstimateProposalReasonCode::CoarseRuntimeNotCam)
    );
    assert!(
        value
            .reason_codes()
            .contains(&DeveloperEstimateProposalReasonCode::HumanReviewAndAdoptionRequired)
    );
}

#[test]
fn distinct_cube_and_prism_geometry_produce_distinct_cost_and_cutting_evidence() {
    let settings = settings(Some(valid_library()));
    let cube = DeveloperEstimateProposalApplication.propose(
        &exact_geometry(["10", "10", "10"], "600", "1000", ["5", "5", "5"]),
        &settings,
    );
    let prism = DeveloperEstimateProposalApplication.propose(
        &exact_geometry(["12", "8", "5"], "392", "480", ["6", "4", "2.5"]),
        &settings,
    );
    let (ValueState::Available { value: cube }, ValueState::Available { value: prism }) =
        (cube, prism)
    else {
        panic!("both analytic solids must produce proposals")
    };

    assert_eq!(cube.blank_volume_mm3().value(), decimal("2028"));
    assert_eq!(cube.removed_volume_mm3().value(), decimal("1028"));
    assert_eq!(cube.blank_mass_kg().value(), decimal("0.0054756"));
    assert_eq!(
        cube.unit_stock_material_cost().amount(),
        decimal("0.046542600")
    );
    assert_eq!(cube.coarse_times().cutting().value(), decimal("0.06425"));
    assert_ne!(cube.blank_volume_mm3(), prism.blank_volume_mm3());
    assert_ne!(
        cube.unit_stock_material_cost(),
        prism.unit_stock_material_cost()
    );
    assert_ne!(
        cube.coarse_times().cutting(),
        prism.coarse_times().cutting()
    );
}

#[test]
fn explicit_review_adopts_model_sensitive_inputs_and_records_excluded_categories_as_zero() {
    let proposal = match DeveloperEstimateProposalApplication.propose(
        &exact_geometry(["12", "8", "5"], "392", "480", ["6", "4", "2.5"]),
        &settings(Some(valid_library())),
    ) {
        ValueState::Available { value } => value,
        _ => panic!("valid proposal must be available"),
    };
    let adopted = DeveloperEstimateProposalApplication
        .adopt(
            proposal,
            DraftQuantityInputs {
                deliver: partprobe_domain::ItemQuantity::new(2),
                planned_spares: partprobe_domain::ItemQuantity::new(1),
                destructive_samples: partprobe_domain::ItemQuantity::new(0),
            },
            DeveloperEstimateProposalReview {
                proposal_values_reviewed: true,
                coarse_limitations_accepted: true,
                actor: ActorId::new("test-estimator").expect("actor must be valid"),
                recorded_at: RecordedAt::new("2026-09-10T14:00:00Z").expect("time must be valid"),
                reason: "reviewed for one coarse developer estimate".to_owned(),
            },
        )
        .expect("explicit review must adopt the proposal");

    assert_eq!(adopted.inputs.stock.stock_volume.value(), decimal("1155"));
    assert_eq!(adopted.inputs.stock.density.value(), decimal("0.0000027"));
    assert_eq!(
        adopted.inputs.material.purchased.amount(),
        decimal("0.079521750")
    );
    assert_eq!(adopted.inputs.times.setup_hours, decimal("1"));
    assert_eq!(adopted.inputs.times.programming_hours, decimal("0.75"));
    assert_eq!(
        adopted.inputs.times.cutting_hours_per_item,
        decimal("0.000703125")
    );
    assert_eq!(
        adopted.inputs.times.load_unload_hours_per_item,
        decimal("0.05")
    );
    assert_eq!(
        adopted.inputs.times.quality_inspection_hours,
        decimal("0.25")
    );
    assert_eq!(adopted.inputs.operation.tooling.amount(), Decimal::ZERO);
    assert_eq!(adopted.inputs.base.overhead.amount(), Decimal::ZERO);
    assert_eq!(adopted.review.actor.as_str(), "test-estimator");
}

#[test]
fn adoption_requires_both_review_decisions_and_a_reason() {
    let proposal = match DeveloperEstimateProposalApplication.propose(
        &exact_geometry(["12", "8", "5"], "392", "480", ["6", "4", "2.5"]),
        &settings(Some(valid_library())),
    ) {
        ValueState::Available { value } => value,
        _ => panic!("valid proposal must be available"),
    };
    let quantities = DraftQuantityInputs {
        deliver: partprobe_domain::ItemQuantity::new(1),
        planned_spares: partprobe_domain::ItemQuantity::new(0),
        destructive_samples: partprobe_domain::ItemQuantity::new(0),
    };
    let review = |values, limits, reason: &str| DeveloperEstimateProposalReview {
        proposal_values_reviewed: values,
        coarse_limitations_accepted: limits,
        actor: ActorId::new("test-estimator").expect("actor must be valid"),
        recorded_at: RecordedAt::new("2026-09-10T14:00:00Z").expect("time must be valid"),
        reason: reason.to_owned(),
    };

    assert_eq!(
        DeveloperEstimateProposalApplication.adopt(
            proposal.clone(),
            quantities.clone(),
            review(false, true, "reason")
        ),
        Err(DeveloperEstimateProposalAdoptionError::ProposalNotReviewed)
    );
    assert_eq!(
        DeveloperEstimateProposalApplication.adopt(
            proposal.clone(),
            quantities.clone(),
            review(true, false, "reason")
        ),
        Err(DeveloperEstimateProposalAdoptionError::LimitationsNotAccepted)
    );
    assert_eq!(
        DeveloperEstimateProposalApplication.adopt(proposal, quantities, review(true, true, "")),
        Err(DeveloperEstimateProposalAdoptionError::InvalidReviewReason)
    );
}

#[test]
fn missing_library_and_legacy_exact_geometry_remain_visibly_unavailable() {
    assert_eq!(
        DeveloperEstimateProposalApplication.propose(
            &exact_geometry(["10", "10", "10"], "600", "1000", ["5", "5", "5"]),
            &settings(None),
        ),
        ValueState::Unavailable {
            reason: "persisted starter resource-library draft is unavailable".to_owned(),
        }
    );
    assert_eq!(
        DeveloperEstimateProposalApplication
            .propose(&legacy_geometry(), &settings(Some(valid_library()))),
        ValueState::Unavailable {
            reason: "exact STEP envelope evidence is unavailable".to_owned(),
        }
    );
}

#[test]
fn non_draft_resource_and_unsupported_stock_form_fail_visibly() {
    let geometry = exact_geometry(["10", "10", "10"], "600", "1000", ["5", "5", "5"]);
    assert_eq!(
        DeveloperEstimateProposalApplication.propose(
            &geometry,
            &settings(Some(library(
                LibraryRecordState::Reviewed,
                StockForm::Rectangular,
                ProcessClass::Milling,
                ["500", "500", "500"],
            ))),
        ),
        ValueState::Blocked {
            reason: "developer estimate-input proposal is blocked: developer proposal requires draft material evidence"
                .to_owned(),
        }
    );
    assert_eq!(
        DeveloperEstimateProposalApplication.propose(
            &geometry,
            &settings(Some(library(
                LibraryRecordState::Draft,
                StockForm::Round,
                ProcessClass::Milling,
                ["500", "500", "500"],
            ))),
        ),
        ValueState::Blocked {
            reason: "developer estimate-input proposal is blocked: stock-envelope rule supports only rectangular stock"
                .to_owned(),
        }
    );
}

#[test]
fn process_and_machine_mismatches_fail_instead_of_filling_estimate_inputs() {
    let geometry = exact_geometry(["12", "8", "5"], "392", "480", ["6", "4", "2.5"]);
    assert_eq!(
        DeveloperEstimateProposalApplication.propose(
            &geometry,
            &settings(Some(library(
                LibraryRecordState::Draft,
                StockForm::Rectangular,
                ProcessClass::Turning,
                ["500", "500", "500"],
            ))),
        ),
        ValueState::Blocked {
            reason: "developer estimate-input proposal is blocked: developer proposal currently supports only coarse milling profiles"
                .to_owned(),
        }
    );
    assert_eq!(
        DeveloperEstimateProposalApplication.propose(
            &geometry,
            &settings(Some(library(
                LibraryRecordState::Draft,
                StockForm::Rectangular,
                ProcessClass::Milling,
                ["14", "10", "6"],
            ))),
        ),
        ValueState::Blocked {
            reason: "developer estimate-input proposal is blocked: proposed rectangular blank exceeds the pinned machine envelope"
                .to_owned(),
        }
    );
}
