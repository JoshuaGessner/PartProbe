use partprobe_geometry_core::RepresentationBasis;
use partprobe_test_support::geometry_fixtures::{
    ExpectedEvidence, GeometryFixtureExpectation, GeometryImportFailureExpectation,
};

const CLOSED: &str = include_str!("../../../fixtures/expected/cube_10mm.json");
const BINARY_CLOSED: &str = include_str!("../../../fixtures/expected/cube_10mm_binary.json");
const THREE_MF: &str = include_str!("../../../fixtures/expected/cube_1cm_translated_3mf.json");
const COMPONENT_THREE_MF: &str =
    include_str!("../../../fixtures/expected/cube_1cm_component_scaled_translated_3mf.json");
const NESTED_COMPONENT_THREE_MF: &str =
    include_str!("../../../fixtures/expected/cube_1cm_nested_component_chain_3mf.json");
const METADATA_THREE_MF: &str =
    include_str!("../../../fixtures/expected/cube_10mm_3mf_metadata.json");
const THREE_MF_UNITS: [(&str, &str); 6] = [
    (
        include_str!("../../../fixtures/expected/cube_10mm_3mf_micron.json"),
        "FIX-MESH-006",
    ),
    (
        include_str!("../../../fixtures/expected/cube_10mm_3mf_millimeter.json"),
        "FIX-MESH-007",
    ),
    (
        include_str!("../../../fixtures/expected/cube_10mm_3mf_meter.json"),
        "FIX-MESH-008",
    ),
    (
        include_str!("../../../fixtures/expected/cube_10mm_3mf_inch.json"),
        "FIX-MESH-009",
    ),
    (
        include_str!("../../../fixtures/expected/cube_10mm_3mf_foot.json"),
        "FIX-MESH-010",
    ),
    (
        include_str!("../../../fixtures/expected/cube_10mm_3mf_default_mm.json"),
        "FIX-MESH-011",
    ),
];
const OPEN: &str = include_str!("../../../fixtures/expected/open_cube_10mm.json");
const STEP: &str = include_str!("../../../fixtures/expected/cube_10mm_step.json");
const INDEPENDENT_STEP: &str =
    include_str!("../../../fixtures/expected/rectangular_prism_12x8x5_step.json");
const INVALID_STEP: &str =
    include_str!("../../../fixtures/expected/invalid_step_entity_rejection.json");
const THREE_MF_FAILURES: [(&str, &str, &str); 18] = [
    (
        include_str!(
            "../../../fixtures/expected/adversarial_3mf_branching_components_rejection.json"
        ),
        "FIX-MESH-014",
        "THREE_MF_UNSUPPORTED_MODEL_STRUCTURE",
    ),
    (
        include_str!(
            "../../../fixtures/expected/adversarial_3mf_non_immediate_reference_rejection.json"
        ),
        "FIX-MESH-015",
        "THREE_MF_UNSUPPORTED_MODEL_STRUCTURE",
    ),
    (
        include_str!("../../../fixtures/expected/adversarial_3mf_object_metadata_rejection.json"),
        "FIX-MESH-016",
        "THREE_MF_UNSUPPORTED_MODEL_STRUCTURE",
    ),
    (
        include_str!(
            "../../../fixtures/expected/adversarial_3mf_relationship_traversal_rejection.json"
        ),
        "FIX-MESH-017",
        "THREE_MF_UNSAFE_PACKAGE",
    ),
    (
        include_str!(
            "../../../fixtures/expected/adversarial_3mf_case_ambiguous_part_rejection.json"
        ),
        "FIX-MESH-018",
        "THREE_MF_UNSAFE_PACKAGE",
    ),
    (
        include_str!("../../../fixtures/expected/adversarial_3mf_build_union_rejection.json"),
        "FIX-MESH-019",
        "THREE_MF_UNSUPPORTED_MODEL_STRUCTURE",
    ),
    (
        include_str!("../../../fixtures/expected/adversarial_3mf_item_metadata_rejection.json"),
        "FIX-MESH-020",
        "THREE_MF_UNSUPPORTED_MODEL_STRUCTURE",
    ),
    (
        include_str!("../../../fixtures/expected/adversarial_3mf_vendor_metadata_rejection.json"),
        "FIX-MESH-021",
        "THREE_MF_UNSUPPORTED_MODEL_STRUCTURE",
    ),
    (
        include_str!(
            "../../../fixtures/expected/adversarial_3mf_high_compression_ratio_rejection.json"
        ),
        "FIX-MESH-022",
        "THREE_MF_ARCHIVE_LIMIT_EXCEEDED",
    ),
    (
        include_str!(
            "../../../fixtures/expected/adversarial_3mf_unsupported_compression_rejection.json"
        ),
        "FIX-MESH-023",
        "THREE_MF_UNSUPPORTED_COMPRESSION",
    ),
    (
        include_str!(
            "../../../fixtures/expected/adversarial_3mf_forward_component_reference_rejection.json"
        ),
        "FIX-MESH-024",
        "THREE_MF_UNSUPPORTED_MODEL_STRUCTURE",
    ),
    (
        include_str!(
            "../../../fixtures/expected/adversarial_3mf_unused_component_object_rejection.json"
        ),
        "FIX-MESH-025",
        "THREE_MF_UNSUPPORTED_MODEL_STRUCTURE",
    ),
    (
        include_str!(
            "../../../fixtures/expected/adversarial_3mf_material_attribute_rejection.json"
        ),
        "FIX-MESH-026",
        "THREE_MF_UNSUPPORTED_MODEL_STRUCTURE",
    ),
    (
        include_str!(
            "../../../fixtures/expected/adversarial_3mf_required_extension_rejection.json"
        ),
        "FIX-MESH-027",
        "THREE_MF_UNSUPPORTED_REQUIRED_EXTENSION",
    ),
    (
        include_str!("../../../fixtures/expected/adversarial_3mf_encrypted_entry_rejection.json"),
        "FIX-MESH-028",
        "THREE_MF_UNSAFE_PACKAGE",
    ),
    (
        include_str!(
            "../../../fixtures/expected/adversarial_3mf_absolute_entry_name_rejection.json"
        ),
        "FIX-MESH-029",
        "THREE_MF_UNSAFE_PACKAGE",
    ),
    (
        include_str!(
            "../../../fixtures/expected/adversarial_3mf_backslash_entry_name_rejection.json"
        ),
        "FIX-MESH-030",
        "THREE_MF_UNSAFE_PACKAGE",
    ),
    (
        include_str!("../../../fixtures/expected/adversarial_3mf_directory_entry_rejection.json"),
        "FIX-MESH-031",
        "THREE_MF_UNSAFE_PACKAGE",
    ),
];
const BINARY_STL_FAILURES: [(&str, &str, &str); 2] = [
    (
        include_str!(
            "../../../fixtures/expected/adversarial_binary_stl_truncated_record_rejection.json"
        ),
        "FIX-MESH-032",
        "STL_INVALID_STRUCTURE",
    ),
    (
        include_str!(
            "../../../fixtures/expected/adversarial_binary_stl_attribute_data_rejection.json"
        ),
        "FIX-MESH-033",
        "STL_UNSUPPORTED_ATTRIBUTE_DATA",
    ),
];

#[test]
fn committed_mesh_expectations_satisfy_the_versioned_contract() {
    let closed: GeometryFixtureExpectation =
        serde_json::from_str(CLOSED).expect("closed fixture expectation must be valid");
    let open: GeometryFixtureExpectation =
        serde_json::from_str(OPEN).expect("open fixture expectation must be valid");
    let binary_closed: GeometryFixtureExpectation = serde_json::from_str(BINARY_CLOSED)
        .expect("binary closed fixture expectation must be valid");
    let three_mf: GeometryFixtureExpectation =
        serde_json::from_str(THREE_MF).expect("3MF fixture expectation must be valid");
    let component_three_mf: GeometryFixtureExpectation = serde_json::from_str(COMPONENT_THREE_MF)
        .expect("component 3MF fixture expectation must be valid");
    let nested_component_three_mf: GeometryFixtureExpectation =
        serde_json::from_str(NESTED_COMPONENT_THREE_MF)
            .expect("nested component 3MF fixture expectation must be valid");
    let metadata_three_mf: GeometryFixtureExpectation = serde_json::from_str(METADATA_THREE_MF)
        .expect("metadata 3MF fixture expectation must be valid");
    let step: GeometryFixtureExpectation =
        serde_json::from_str(STEP).expect("STEP fixture expectation must be valid");
    let independent_step: GeometryFixtureExpectation = serde_json::from_str(INDEPENDENT_STEP)
        .expect("independently authored STEP fixture expectation must be valid");

    assert_eq!(closed.fixture_id(), "FIX-MESH-001");
    assert_eq!(open.fixture_id(), "FIX-MESH-002");
    assert_eq!(binary_closed.fixture_id(), "FIX-MESH-003");
    assert_eq!(three_mf.fixture_id(), "FIX-MESH-004");
    assert_eq!(component_three_mf.fixture_id(), "FIX-MESH-005");
    assert_eq!(nested_component_three_mf.fixture_id(), "FIX-MESH-013");
    assert_eq!(metadata_three_mf.fixture_id(), "FIX-MESH-012");
    assert_eq!(step.fixture_id(), "FIX-STEP-001");
    assert_eq!(independent_step.fixture_id(), "FIX-STEP-003");
    assert_eq!(closed.representation(), RepresentationBasis::Mesh);
    assert!(matches!(
        closed.enclosed_volume_mm3(),
        ExpectedEvidence::Available { .. }
    ));
    assert!(matches!(
        open.enclosed_volume_mm3(),
        ExpectedEvidence::Unavailable { .. }
    ));
    assert!(matches!(
        binary_closed.enclosed_volume_mm3(),
        ExpectedEvidence::Available { .. }
    ));
    assert_eq!(three_mf.representation(), RepresentationBasis::Mesh);
    assert!(matches!(
        three_mf.enclosed_volume_mm3(),
        ExpectedEvidence::Available { .. }
    ));
    assert_eq!(
        component_three_mf.representation(),
        RepresentationBasis::Mesh
    );
    assert!(matches!(
        component_three_mf.enclosed_volume_mm3(),
        ExpectedEvidence::Available { .. }
    ));
    assert_eq!(
        nested_component_three_mf.representation(),
        RepresentationBasis::Mesh
    );
    assert!(matches!(
        nested_component_three_mf.enclosed_volume_mm3(),
        ExpectedEvidence::Available { .. }
    ));
    assert_eq!(
        metadata_three_mf.representation(),
        RepresentationBasis::Mesh
    );
    assert!(matches!(
        metadata_three_mf.enclosed_volume_mm3(),
        ExpectedEvidence::Available { .. }
    ));
    for (source, fixture_id) in THREE_MF_UNITS {
        let unit_fixture: GeometryFixtureExpectation = serde_json::from_str(source)
            .expect("persisted 3MF unit fixture expectation must be valid");
        assert_eq!(unit_fixture.fixture_id(), fixture_id);
        assert_eq!(unit_fixture.representation(), RepresentationBasis::Mesh);
        assert!(matches!(
            unit_fixture.enclosed_volume_mm3(),
            ExpectedEvidence::Available { .. }
        ));
    }
    assert_eq!(step.representation(), RepresentationBasis::ExactBrep);
    assert!(matches!(
        step.enclosed_volume_mm3(),
        ExpectedEvidence::Available { .. }
    ));
    assert_eq!(
        independent_step.representation(),
        RepresentationBasis::ExactBrep
    );
    assert!(matches!(
        independent_step.enclosed_volume_mm3(),
        ExpectedEvidence::Available { .. }
    ));
}

#[test]
fn deserialization_rejects_false_authority_for_open_mesh_volume() {
    let mut value: serde_json::Value = serde_json::from_str(OPEN).expect("fixture JSON must parse");
    let mut false_volume = value.clone();
    false_volume["enclosed_volume_mm3"] =
        serde_json::json!({"state": "available", "value": "1000"});
    assert!(serde_json::from_value::<GeometryFixtureExpectation>(false_volume).is_err());

    value["schema_version"] = serde_json::json!(1);
    assert!(serde_json::from_value::<GeometryFixtureExpectation>(value).is_err());
}

#[test]
fn committed_failure_expectations_require_controlled_recoverable_failures() {
    let expectation: GeometryImportFailureExpectation =
        serde_json::from_str(INVALID_STEP).expect("invalid STEP expectation must be valid");

    assert_eq!(expectation.fixture_id(), "FIX-STEP-002");
    assert_eq!(
        expectation.expected_diagnostic_code().as_str(),
        "STEP_TRANSFER_FAILED"
    );
    assert!(!expectation.snapshot_expected());
    assert!(!expectation.output_file_expected());
    assert!(!expectation.staged_input_retained());

    for (source, fixture_id, diagnostic_code) in
        THREE_MF_FAILURES.into_iter().chain(BINARY_STL_FAILURES)
    {
        let expectation: GeometryImportFailureExpectation =
            serde_json::from_str(source).expect("adversarial 3MF expectation must be valid");
        assert_eq!(expectation.fixture_id(), fixture_id);
        assert_eq!(
            expectation.expected_diagnostic_code().as_str(),
            diagnostic_code
        );
        assert!(!expectation.snapshot_expected());
        assert!(!expectation.output_file_expected());
        assert!(!expectation.staged_input_retained());
    }
}

#[test]
fn failure_expectation_rejects_success_or_retained_artifacts() {
    for invalid in [
        INVALID_STEP.replace(
            "\"expected_status\": \"failed_recoverable\"",
            "\"expected_status\": \"succeeded\"",
        ),
        INVALID_STEP.replace(
            "\"snapshot_expected\": false",
            "\"snapshot_expected\": true",
        ),
    ] {
        assert!(
            serde_json::from_str::<GeometryImportFailureExpectation>(&invalid).is_err(),
            "failure expectation invariants must survive deserialization"
        );
    }
}
