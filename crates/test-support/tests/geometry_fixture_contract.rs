use partprobe_geometry_core::RepresentationBasis;
use partprobe_test_support::geometry_fixtures::{
    ExpectedEvidence, GeometryFixtureExpectation, GeometryImportFailureExpectation,
};

const CLOSED: &str = include_str!("../../../fixtures/expected/cube_10mm.json");
const BINARY_CLOSED: &str = include_str!("../../../fixtures/expected/cube_10mm_binary.json");
const THREE_MF: &str = include_str!("../../../fixtures/expected/cube_1cm_translated_3mf.json");
const COMPONENT_THREE_MF: &str =
    include_str!("../../../fixtures/expected/cube_1cm_component_scaled_translated_3mf.json");
const OPEN: &str = include_str!("../../../fixtures/expected/open_cube_10mm.json");
const STEP: &str = include_str!("../../../fixtures/expected/cube_10mm_step.json");
const INDEPENDENT_STEP: &str =
    include_str!("../../../fixtures/expected/rectangular_prism_12x8x5_step.json");
const INVALID_STEP: &str =
    include_str!("../../../fixtures/expected/invalid_step_entity_rejection.json");

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
    let step: GeometryFixtureExpectation =
        serde_json::from_str(STEP).expect("STEP fixture expectation must be valid");
    let independent_step: GeometryFixtureExpectation = serde_json::from_str(INDEPENDENT_STEP)
        .expect("independently authored STEP fixture expectation must be valid");

    assert_eq!(closed.fixture_id(), "FIX-MESH-001");
    assert_eq!(open.fixture_id(), "FIX-MESH-002");
    assert_eq!(binary_closed.fixture_id(), "FIX-MESH-003");
    assert_eq!(three_mf.fixture_id(), "FIX-MESH-004");
    assert_eq!(component_three_mf.fixture_id(), "FIX-MESH-005");
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
fn invalid_step_expectation_requires_controlled_recoverable_failure() {
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
