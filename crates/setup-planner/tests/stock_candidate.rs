use partprobe_domain::{
    ActorId, CoarseRuntimeProfile, CurrencyCode, DensityKilogramsPerCubicMeter, EffectiveDate,
    LibraryRecordState, MachineEnvelopeMillimeters, MachineProfile, MachineProfileId,
    MaterialDefinition, MaterialDefinitionId, MaterialOffer, MaterialOfferId, Money, ProcessClass,
    RecordId, RecordedAt, RemovalRateCubicMillimetersPerMinute, RuntimeMinutes, RuntimeProfileId,
    ShopProfileId, ShopResourceLibrary, ShopResourceLibraryId, ShopResourceVersion,
    ShopSettingsDraft, ShopSettingsRevision, SourceKind, SourceRef, StockAllowanceMillimeters,
    StockAllowanceProfile, StockAllowanceProfileId, StockForm,
};
use partprobe_geometry_core::{
    ExactStepEnvelopeDerivative, ProvisionalGeometryDecimal, ProvisionalGeometrySnapshot,
    Sha256Digest,
};
use partprobe_geometry_import::ProvisionalExactStepAnalysis;
use partprobe_setup_planner::stock_candidate::{
    DeveloperStockCandidate, STOCK_CANDIDATE_RULE_ID, STOCK_CANDIDATE_RULE_VERSION,
    STOCK_CANDIDATE_SCHEMA_VERSION, StockCandidateEdit, StockCandidateError,
    StockCandidateReasonCode,
};
use partprobe_setup_planner::{
    AxisAlignedEnvelopeMillimeters, CanonicalMillimeterLength, DeveloperEstimateProposalReadiness,
    ExactStepEnvelopeEvidence, StockProposalError, propose_developer_estimate_inputs,
};
use rust_decimal::Decimal;
fn decimal(value: &str) -> Decimal {
    Decimal::from_str_exact(value).unwrap()
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

fn envelope(
    x: &str,
    y: &str,
    z: &str,
    part_volume: &str,
    source: &str,
    output: &str,
) -> ExactStepEnvelopeEvidence {
    let source_hash = Sha256Digest::new(source.repeat(64)).expect("source hash must be valid");
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
        Sha256Digest::new(output.repeat(64)).expect("output hash must be valid"),
    )
    .expect("test envelope must enclose its part")
}

fn library(
    record_state: LibraryRecordState,
    stock_form: StockForm,
    process_class: ProcessClass,
    machine_envelope_mm: [&str; 3],
    removal_rate: &str,
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
        RemovalRateCubicMillimetersPerMinute::new(decimal(removal_rate))
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

fn settings(resources: Option<ShopResourceLibrary>, revision: u32) -> ShopSettingsDraft {
    ShopSettingsDraft::new_with_resources(
        ShopProfileId::new("test-shop").expect("profile must be valid"),
        ShopSettingsRevision::new(revision).expect("revision must be valid"),
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
        "16000",
    )
}
fn prism() -> ExactStepEnvelopeEvidence {
    envelope("12", "8", "5", "480", "a", "b")
}
fn dimensions(values: [&str; 3]) -> AxisAlignedEnvelopeMillimeters {
    let [x, y, z] = values.map(|v| CanonicalMillimeterLength::new(decimal(v)).unwrap());
    AxisAlignedEnvelopeMillimeters::new(x, y, z)
}
fn edit(values: [&str; 3]) -> StockCandidateEdit {
    StockCandidateEdit::new(
        dimensions(values),
        ActorId::new("local-test-estimator").unwrap(),
        RecordedAt::new("2026-09-15T12:00:00Z").unwrap(),
        "explicit synthetic blank edit",
    )
    .unwrap()
}
fn candidate() -> DeveloperStockCandidate {
    let geometry = prism();
    let settings = settings(Some(valid_library()), 7);
    let proposal =
        propose_developer_estimate_inputs(&geometry, settings.resource_library().unwrap()).unwrap();
    DeveloperStockCandidate::new(
        RecordId::new("candidate-1").unwrap(),
        &geometry,
        &settings,
        &proposal,
        edit(["20", "12", "8"]),
    )
    .unwrap()
}

#[test]
fn edited_prism_recomputes_exact_stock_cost_and_cutting_without_changing_baseline() {
    let c = candidate();
    assert_eq!(c.evidence_schema_version(), STOCK_CANDIDATE_SCHEMA_VERSION);
    assert_eq!(c.rule_id(), STOCK_CANDIDATE_RULE_ID);
    assert_eq!(c.rule_version(), STOCK_CANDIDATE_RULE_VERSION);
    assert_eq!(c.id().as_str(), "candidate-1");
    assert_eq!(c.revision(), 1);
    assert_eq!(c.previous_revision(), None);
    assert_eq!(c.previous_dimensions_mm(), dimensions(["15", "11", "7"]));
    assert_eq!(c.edit().dimensions_mm(), dimensions(["20", "12", "8"]));
    assert_eq!(
        c.total_allowance_mm().map(StockAllowanceMillimeters::value),
        [decimal("8"), decimal("4"), decimal("3")]
    );
    assert_eq!(c.blank_volume_mm3().value(), decimal("1920"));
    assert_eq!(c.removed_volume_mm3().value(), decimal("1440"));
    assert_eq!(c.blank_mass_kg().value(), decimal("0.005184"));
    assert_eq!(c.unit_stock_material_cost().amount(), decimal("0.044064"));
    assert_eq!(c.unit_stock_material_cost().currency().as_str(), "USD");
    assert_eq!(c.coarse_times().cutting().value(), decimal("0.09"));
    let original = c.original_proposal();
    assert_eq!(original.blank_volume_mm3().value(), decimal("1155"));
    assert_eq!(
        original.unit_stock_material_cost().amount(),
        decimal("0.026507250")
    );
    assert_eq!(
        original.coarse_times().cutting().value(),
        decimal("0.0421875")
    );
    assert_eq!(c.coarse_times().setup(), original.coarse_times().setup());
    assert_eq!(
        c.coarse_times().programming(),
        original.coarse_times().programming()
    );
    assert_eq!(
        c.coarse_times().load_unload(),
        original.coarse_times().load_unload()
    );
    assert_eq!(
        c.coarse_times().quality(),
        original.coarse_times().quality()
    );
    assert_eq!(
        c.readiness(),
        DeveloperEstimateProposalReadiness::NeedsReview
    );
    assert_eq!(
        c.reason_codes(),
        [
            StockCandidateReasonCode::UserEditedBlankDimensions,
            StockCandidateReasonCode::StockAllowanceProfileOverridden
        ]
    );
    assert_eq!(original.reason_codes().len(), 7);
    assert_eq!(c.settings_basis().revision().value(), 7);
    assert_eq!(c.settings_basis().profile_id().as_str(), "test-shop");
    assert_eq!(c.edit().actor().as_str(), "local-test-estimator");
    assert_eq!(c.edit().recorded_at().as_str(), "2026-09-15T12:00:00Z");
    assert_eq!(c.edit().reason(), "explicit synthetic blank edit");
}

#[test]
fn successor_retains_exact_original_and_predecessor_without_mutating_either() {
    let first = candidate();
    let saved = first.clone();
    let revised_edit = StockCandidateEdit::new(
        dimensions(["18", "10", "7"]),
        ActorId::new("second-test-estimator").unwrap(),
        RecordedAt::new("2026-09-15T13:00:00Z").unwrap(),
        "explicit smaller synthetic blank",
    )
    .unwrap();
    let next = first
        .revise(&prism(), first.settings_basis(), revised_edit.clone())
        .unwrap();
    assert_eq!(first, saved);
    assert_eq!(next.id(), first.id());
    assert_eq!(next.revision(), 2);
    assert_eq!(next.previous_revision(), Some(1));
    assert_eq!(next.previous_dimensions_mm(), first.edit().dimensions_mm());
    assert_eq!(next.original_proposal(), first.original_proposal());
    assert_eq!(next.settings_basis(), first.settings_basis());
    assert_eq!(next.edit(), &revised_edit);
    assert_ne!(next.edit().actor(), first.edit().actor());
    assert_ne!(next.edit().recorded_at(), first.edit().recorded_at());
    assert_ne!(next.edit().reason(), first.edit().reason());
    assert_eq!(next.blank_volume_mm3().value(), decimal("1260"));
    assert_eq!(next.removed_volume_mm3().value(), decimal("780"));
    assert_eq!(next.blank_mass_kg().value(), decimal("0.003402"));
    assert_eq!(
        next.unit_stock_material_cost().amount(),
        decimal("0.028917")
    );
    assert_eq!(next.coarse_times().cutting().value(), decimal("0.04875"));
}

#[test]
fn cube_can_explicitly_override_allowances_to_zero_but_remains_review_only() {
    let geometry = envelope("10", "10", "10", "1000", "a", "b");
    let settings = settings(Some(valid_library()), 7);
    let proposal =
        propose_developer_estimate_inputs(&geometry, settings.resource_library().unwrap()).unwrap();
    let c = DeveloperStockCandidate::new(
        RecordId::new("cube-candidate").unwrap(),
        &geometry,
        &settings,
        &proposal,
        edit(["10", "10", "10"]),
    )
    .unwrap();
    assert_eq!(c.removed_volume_mm3().value(), Decimal::ZERO);
    assert_eq!(c.coarse_times().cutting().value(), Decimal::ZERO);
    assert_eq!(
        c.total_allowance_mm().map(StockAllowanceMillimeters::value),
        [Decimal::ZERO; 3]
    );
    assert_eq!(c.unit_stock_material_cost().amount(), decimal("0.02295"));
    assert_eq!(
        c.readiness(),
        DeveloperEstimateProposalReadiness::NeedsReview
    );
    assert_eq!(proposal.blank_volume_mm3().value(), decimal("2028"));
}

#[test]
fn returning_to_original_dimensions_creates_a_review_only_revision_not_an_approval() {
    let first = candidate();
    let restored = first
        .revise(&prism(), first.settings_basis(), edit(["15", "11", "7"]))
        .unwrap();
    assert_eq!(restored.revision(), 2);
    assert_eq!(
        restored.previous_dimensions_mm(),
        first.edit().dimensions_mm()
    );
    assert_eq!(
        restored.blank_volume_mm3(),
        first.original_proposal().blank_volume_mm3()
    );
    assert_eq!(
        restored.unit_stock_material_cost(),
        first.original_proposal().unit_stock_material_cost()
    );
    assert_eq!(
        restored.coarse_times(),
        first.original_proposal().coarse_times()
    );
    assert_eq!(
        restored.readiness(),
        DeveloperEstimateProposalReadiness::NeedsReview
    );
    assert_eq!(restored.original_proposal(), first.original_proposal());
}

#[test]
fn large_volume_does_not_compensate_for_an_axis_that_fails_enclosure() {
    let c = candidate();
    assert_eq!(
        c.revise(&prism(), c.settings_basis(), edit(["11", "50", "50"])),
        Err(StockCandidateError::BlankDoesNotEncloseModel)
    );
}

#[test]
fn edited_blank_must_fit_corresponding_machine_axes_without_rotation() {
    let c = candidate();
    assert_eq!(
        c.revise(&prism(), c.settings_basis(), edit(["501", "12", "8"])),
        Err(StockCandidateError::Calculation(
            StockProposalError::MachineEnvelopeMismatch
        ))
    );
}

#[test]
fn no_op_is_rejected_for_first_and_later_revisions_including_decimal_formatting() {
    let c = candidate();
    assert_eq!(
        c.revise(&prism(), c.settings_basis(), edit(["20.0", "12.00", "8"])),
        Err(StockCandidateError::NoStockChange)
    );
    assert_eq!(
        DeveloperStockCandidate::new(
            RecordId::new("noop").unwrap(),
            &prism(),
            c.settings_basis(),
            c.original_proposal(),
            edit(["15", "11", "7"])
        ),
        Err(StockCandidateError::NoStockChange)
    );
}

#[test]
fn changed_source_output_or_geometry_is_stale_even_with_reused_source_identity() {
    let c = candidate();
    for changed in [
        envelope("12", "8", "5", "480", "c", "b"),
        envelope("12", "8", "5", "480", "a", "c"),
        envelope("13", "8", "5", "480", "a", "b"),
        envelope("12", "8", "5", "479", "a", "b"),
    ] {
        assert_eq!(
            c.revise(&changed, c.settings_basis(), edit(["18", "10", "7"])),
            Err(StockCandidateError::StaleBasis)
        );
    }
}

#[test]
fn settings_successor_and_reused_child_versions_with_changed_bytes_are_stale() {
    let c = candidate();
    let changed_version = settings(Some(valid_library()), 8);
    let changed_child = settings(
        Some(library(
            LibraryRecordState::Draft,
            StockForm::Rectangular,
            ProcessClass::Milling,
            ["499", "500", "500"],
            "16000",
        )),
        7,
    );
    for changed in [changed_version, changed_child] {
        assert_eq!(
            c.revise(&prism(), &changed, edit(["18", "10", "7"])),
            Err(StockCandidateError::StaleBasis)
        );
    }
}

#[test]
fn original_proposal_is_replayed_not_trusted_from_an_unrelated_analysis() {
    let c = candidate();
    let other = envelope("10", "10", "10", "1000", "a", "b");
    assert_eq!(
        DeveloperStockCandidate::new(
            RecordId::new("mismatch").unwrap(),
            &other,
            c.settings_basis(),
            c.original_proposal(),
            edit(["20", "12", "8"])
        ),
        Err(StockCandidateError::ProposalMismatch)
    );
}

#[test]
fn missing_library_stays_unavailable_instead_of_creating_a_zero_cost_candidate() {
    let c = candidate();
    assert_eq!(
        DeveloperStockCandidate::new(
            RecordId::new("missing").unwrap(),
            &prism(),
            &settings(None, 7),
            c.original_proposal(),
            edit(["20", "12", "8"])
        ),
        Err(StockCandidateError::ResourcesUnavailable)
    );
}

#[test]
fn edit_reason_and_positive_dimensions_are_required() {
    for reason in ["", " ", "bad\0reason", &"x".repeat(2049), &"é".repeat(1025)] {
        assert_eq!(
            StockCandidateEdit::new(
                dimensions(["20", "12", "8"]),
                ActorId::new("test").unwrap(),
                RecordedAt::new("2026-09-15T12:00:00Z").unwrap(),
                reason
            ),
            Err(StockCandidateError::InvalidReason)
        );
    }
    for invalid in ["0", "-1"] {
        assert_eq!(
            CanonicalMillimeterLength::new(decimal(invalid)),
            Err(StockProposalError::InvalidEnvelopeDimension)
        );
    }
}

#[test]
fn candidate_arithmetic_rejects_excessive_magnitude_without_changing_existing_values() {
    let max = "79228162514264337593543950335";
    let geometry = prism();
    let settings = settings(
        Some(library(
            LibraryRecordState::Draft,
            StockForm::Rectangular,
            ProcessClass::Milling,
            [max; 3],
            "16000",
        )),
        7,
    );
    let proposal =
        propose_developer_estimate_inputs(&geometry, settings.resource_library().unwrap()).unwrap();
    assert_eq!(
        DeveloperStockCandidate::new(
            RecordId::new("overflow").unwrap(),
            &geometry,
            &settings,
            &proposal,
            edit([max; 3])
        ),
        // The unchanged exact-product representation gate uses the rounding-required category
        // for an unrepresentable magnitude, before attempting the overflowing multiplication.
        Err(StockCandidateError::Calculation(
            StockProposalError::ArithmeticRequiresRounding
        ))
    );
    assert_eq!(proposal.blank_volume_mm3().value(), decimal("1155"));
}

#[test]
fn nonterminating_candidate_division_requires_explicit_rounding_policy() {
    // Baseline removal 675/3 is exact; edited 1000 - 480 = 520/3 is not.
    let geometry = prism();
    let settings = settings(
        Some(library(
            LibraryRecordState::Draft,
            StockForm::Rectangular,
            ProcessClass::Milling,
            ["500"; 3],
            "3",
        )),
        7,
    );
    let proposal =
        propose_developer_estimate_inputs(&geometry, settings.resource_library().unwrap()).unwrap();
    assert_eq!(
        DeveloperStockCandidate::new(
            RecordId::new("rounding").unwrap(),
            &geometry,
            &settings,
            &proposal,
            edit(["20", "10", "5"])
        ),
        Err(StockCandidateError::Calculation(
            StockProposalError::ArithmeticRequiresRounding
        ))
    );
}
