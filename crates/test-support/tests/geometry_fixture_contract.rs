use partprobe_geometry_core::RepresentationBasis;
use partprobe_test_support::geometry_fixtures::{ExpectedEvidence, GeometryFixtureExpectation};

const CLOSED: &str = include_str!("../../../fixtures/expected/cube_10mm.json");
const OPEN: &str = include_str!("../../../fixtures/expected/open_cube_10mm.json");

#[test]
fn committed_mesh_expectations_satisfy_the_versioned_contract() {
    let closed: GeometryFixtureExpectation =
        serde_json::from_str(CLOSED).expect("closed fixture expectation must be valid");
    let open: GeometryFixtureExpectation =
        serde_json::from_str(OPEN).expect("open fixture expectation must be valid");

    assert_eq!(closed.fixture_id(), "FIX-MESH-001");
    assert_eq!(open.fixture_id(), "FIX-MESH-002");
    assert_eq!(closed.representation(), RepresentationBasis::Mesh);
    assert!(matches!(
        closed.enclosed_volume_mm3(),
        ExpectedEvidence::Available { .. }
    ));
    assert!(matches!(
        open.enclosed_volume_mm3(),
        ExpectedEvidence::Unavailable { .. }
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
