use partprobe_geometry_core::{
    GeometryConfidenceLevel, ModelFormat, ModelLengthUnit, RepresentationBasis,
    UnitResolutionMethod,
};
use partprobe_geometry_import::{
    ASCII_STL_ANALYZER_VERSION, BINARY_STL_ANALYZER_VERSION, MESH_CONFIDENCE_POLICY_VERSION,
    MESH_SELF_INTERSECTION_ALGORITHM_VERSION, MESH_TOPOLOGY_POLICY_VERSION,
    MeshSelfIntersectionState, MeshTopologyIdentity, MeshWeldingStatus, StlEncoding, StlError,
    StlLimits, StlMeshEvidence, analyze_ascii_stl, analyze_binary_stl, analyze_stl,
};

const CLOSED_CUBE: &[u8] = include_bytes!("../../../fixtures/models/cube_10mm_ascii.stl");
const BINARY_CUBE: &[u8] = include_bytes!("../../../fixtures/models/cube_10mm_binary.stl");
const ADVERSARIAL_ASCII_STL: [(&[u8], StlError); 4] = [
    (
        include_bytes!("../../../fixtures/models/adversarial_ascii_stl_invalid_utf8.stl"),
        StlError::InvalidText,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_ascii_stl_malformed_facet.stl"),
        StlError::InvalidStructure,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_ascii_stl_empty_solid.stl"),
        StlError::EmptyMesh,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_ascii_stl_degenerate_triangle.stl"),
        StlError::DegenerateTriangle,
    ),
];
const ADVERSARIAL_BINARY_STL: [(&[u8], StlError); 4] = [
    (
        include_bytes!("../../../fixtures/models/adversarial_binary_stl_truncated_record.stl"),
        StlError::InvalidStructure,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_binary_stl_attribute_data.stl"),
        StlError::UnsupportedAttributeData,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_binary_stl_non_finite_normal.stl"),
        StlError::InvalidNumber,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_binary_stl_triangle_count_limit.stl"),
        StlError::TriangleLimitExceeded,
    ),
];
const OPEN_CUBE: &[u8] = include_bytes!("../../../fixtures/models/open_cube_10mm_ascii.stl");
const SELF_INTERSECTING_TETRAHEDRA: &[u8] =
    include_bytes!("../../../fixtures/models/self_intersecting_tetrahedra_ascii.stl");
const REVERSED_FACET_CUBE: &[u8] =
    include_bytes!("../../../fixtures/models/cube_10mm_ascii_reversed_facet.stl");
const NON_MANIFOLD_SHARED_EDGE: &[u8] =
    include_bytes!("../../../fixtures/models/two_tetrahedra_shared_edge_ascii.stl");
const COPLANAR_OVERLAP: &[u8] =
    include_bytes!("../../../fixtures/models/coplanar_overlap_ascii.stl");

fn limits() -> StlLimits {
    StlLimits::new(64 * 1024, 1_000).expect("test limits must be valid")
}

fn warning_codes(evidence: &StlMeshEvidence) -> Vec<&str> {
    evidence
        .warnings()
        .iter()
        .map(|warning| warning.as_str())
        .collect()
}

fn confidence_reasons(evidence: &StlMeshEvidence) -> Vec<&str> {
    evidence
        .confidence()
        .reasons()
        .iter()
        .map(|reason| reason.as_str())
        .collect()
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1.0e-9,
        "{actual} != {expected}"
    );
}

#[test]
fn closed_cube_matches_governed_mesh_measurements_without_unit_authority() {
    let evidence = analyze_ascii_stl(CLOSED_CUBE, limits()).expect("closed cube must parse");

    assert_eq!(evidence.algorithm_version(), ASCII_STL_ANALYZER_VERSION);
    assert_eq!(
        evidence.self_intersection_algorithm_version(),
        MESH_SELF_INTERSECTION_ALGORITHM_VERSION
    );
    assert_eq!(
        evidence.confidence_policy_version(),
        MESH_CONFIDENCE_POLICY_VERSION
    );
    assert_eq!(
        evidence.topology_policy_version(),
        MESH_TOPOLOGY_POLICY_VERSION
    );
    assert_eq!(
        evidence.topology_identity(),
        MeshTopologyIdentity::ExactSourceCoordinates
    );
    assert_eq!(evidence.welding_status(), MeshWeldingStatus::NotApplied);
    let wire = serde_json::to_value(&evidence).expect("STL evidence must serialize");
    assert_eq!(
        wire["topology_policy_version"],
        MESH_TOPOLOGY_POLICY_VERSION
    );
    assert_eq!(wire["topology_identity"], "exact_source_coordinates");
    assert_eq!(wire["welding_status"], "not_applied");
    assert_eq!(evidence.encoding(), StlEncoding::Ascii);
    assert_eq!(evidence.detected_format(), ModelFormat::Stl);
    assert_eq!(evidence.representation(), RepresentationBasis::Mesh);
    assert_eq!(evidence.source_units(), ModelLengthUnit::Unknown);
    assert_eq!(evidence.unit_resolution(), UnitResolutionMethod::Unresolved);
    assert_eq!(evidence.triangle_count(), 12);
    assert!(evidence.manifold());
    assert!(evidence.watertight());
    assert!(evidence.consistently_wound());
    assert_eq!(
        evidence.self_intersection(),
        MeshSelfIntersectionState::NotDetected
    );
    assert_eq!(
        evidence.confidence().level(),
        GeometryConfidenceLevel::NeedsReview
    );
    assert_eq!(
        confidence_reasons(&evidence),
        ["MESH_REPRESENTATION_CEILING", "UNITS_UNRESOLVED"]
    );
    assert_eq!(evidence.aabb_extents_source_units().components(), [10.0; 3]);
    assert_close(evidence.surface_area_source_units_squared(), 600.0);
    assert_close(
        evidence
            .enclosed_volume_source_units_cubed()
            .expect("closed cube volume must be available"),
        1_000.0,
    );
    assert_eq!(
        evidence
            .center_of_mass_source_units()
            .expect("closed cube centroid must be available")
            .components(),
        [5.0; 3]
    );
    assert_eq!(
        warning_codes(&evidence),
        ["UNITS_MISSING_REQUIRES_CONFIRMATION", "MESH_NOT_EXACT_BREP"]
    );
}

#[test]
fn binary_cube_matches_governed_mesh_measurements_without_unit_authority() {
    let evidence = analyze_binary_stl(BINARY_CUBE, limits()).expect("binary cube must parse");

    assert_eq!(evidence.algorithm_version(), BINARY_STL_ANALYZER_VERSION);
    assert_eq!(
        evidence.topology_policy_version(),
        MESH_TOPOLOGY_POLICY_VERSION
    );
    assert_eq!(
        evidence.topology_identity(),
        MeshTopologyIdentity::ExactSourceCoordinates
    );
    assert_eq!(evidence.welding_status(), MeshWeldingStatus::NotApplied);
    assert_eq!(evidence.encoding(), StlEncoding::Binary);
    assert_eq!(evidence.detected_format(), ModelFormat::Stl);
    assert_eq!(evidence.representation(), RepresentationBasis::Mesh);
    assert_eq!(evidence.source_units(), ModelLengthUnit::Unknown);
    assert_eq!(evidence.unit_resolution(), UnitResolutionMethod::Unresolved);
    assert_eq!(evidence.triangle_count(), 12);
    assert!(evidence.manifold());
    assert!(evidence.watertight());
    assert!(evidence.consistently_wound());
    assert_eq!(
        evidence.self_intersection(),
        MeshSelfIntersectionState::NotDetected
    );
    assert_eq!(
        evidence.confidence().level(),
        GeometryConfidenceLevel::NeedsReview
    );
    assert_eq!(evidence.aabb_extents_source_units().components(), [10.0; 3]);
    assert_close(evidence.surface_area_source_units_squared(), 600.0);
    assert_close(
        evidence
            .enclosed_volume_source_units_cubed()
            .expect("closed binary cube volume must be available"),
        1_000.0,
    );
    assert_eq!(
        evidence
            .center_of_mass_source_units()
            .expect("closed binary cube centroid must be available")
            .components(),
        [5.0; 3]
    );
    assert_eq!(
        warning_codes(&evidence),
        ["UNITS_MISSING_REQUIRES_CONFIRMATION", "MESH_NOT_EXACT_BREP"]
    );
}

#[test]
fn content_framing_detects_ascii_and_binary_without_an_extension() {
    assert_eq!(
        analyze_stl(CLOSED_CUBE, limits())
            .expect("ASCII cube must be detected")
            .encoding(),
        StlEncoding::Ascii
    );
    assert_eq!(
        analyze_stl(BINARY_CUBE, limits())
            .expect("binary cube must be detected")
            .encoding(),
        StlEncoding::Binary
    );
}

#[test]
fn open_cube_keeps_volume_and_centroid_unavailable() {
    let evidence = analyze_ascii_stl(OPEN_CUBE, limits()).expect("open cube must parse");

    assert_eq!(evidence.triangle_count(), 10);
    assert!(evidence.manifold());
    assert!(!evidence.watertight());
    assert!(!evidence.consistently_wound());
    assert_eq!(
        evidence.self_intersection(),
        MeshSelfIntersectionState::NotDetected
    );
    assert_eq!(
        evidence.confidence().level(),
        GeometryConfidenceLevel::NeedsReview
    );
    assert_eq!(
        confidence_reasons(&evidence),
        [
            "MESH_REPRESENTATION_CEILING",
            "UNITS_UNRESOLVED",
            "OPEN_BOUNDARY",
        ]
    );
    assert_eq!(evidence.aabb_extents_source_units().components(), [10.0; 3]);
    assert_close(evidence.surface_area_source_units_squared(), 500.0);
    assert_eq!(evidence.enclosed_volume_source_units_cubed(), None);
    assert_eq!(evidence.center_of_mass_source_units(), None);
    assert_eq!(
        warning_codes(&evidence),
        [
            "UNITS_MISSING_REQUIRES_CONFIRMATION",
            "MESH_NOT_EXACT_BREP",
            "OPEN_BOUNDARY",
            "CLOSED_VOLUME_UNAVAILABLE",
        ]
    );
}

#[test]
fn reversed_facet_withholds_closed_measurements_and_requires_review() {
    let evidence =
        analyze_ascii_stl(REVERSED_FACET_CUBE, limits()).expect("reversed cube must parse");

    assert_eq!(evidence.triangle_count(), 12);
    assert!(evidence.manifold());
    assert!(evidence.watertight());
    assert!(!evidence.consistently_wound());
    assert_eq!(
        evidence.self_intersection(),
        MeshSelfIntersectionState::NotDetected
    );
    assert_eq!(evidence.aabb_extents_source_units().components(), [10.0; 3]);
    assert_close(evidence.surface_area_source_units_squared(), 600.0);
    assert_eq!(evidence.enclosed_volume_source_units_cubed(), None);
    assert_eq!(evidence.center_of_mass_source_units(), None);
    assert_eq!(
        confidence_reasons(&evidence),
        [
            "MESH_REPRESENTATION_CEILING",
            "UNITS_UNRESOLVED",
            "INCONSISTENT_WINDING",
        ]
    );
    assert_eq!(
        warning_codes(&evidence),
        [
            "UNITS_MISSING_REQUIRES_CONFIRMATION",
            "MESH_NOT_EXACT_BREP",
            "INCONSISTENT_WINDING",
            "CLOSED_VOLUME_UNAVAILABLE",
        ]
    );
}

#[test]
fn non_manifold_shared_edge_withholds_closed_measurements_and_requires_review() {
    let evidence = analyze_ascii_stl(NON_MANIFOLD_SHARED_EDGE, limits())
        .expect("non-manifold tetrahedra must parse");

    assert_eq!(evidence.triangle_count(), 8);
    assert!(!evidence.manifold());
    assert!(!evidence.watertight());
    assert!(!evidence.consistently_wound());
    assert_eq!(
        evidence.self_intersection(),
        MeshSelfIntersectionState::NotDetected
    );
    assert_eq!(
        evidence.aabb_extents_source_units().components(),
        [2.0, 4.0, 4.0]
    );
    assert_close(
        evidence.surface_area_source_units_squared(),
        6.0 + 2.0 * 3.0_f64.sqrt() + 2.0 * 5.0_f64.sqrt() + 11.0_f64.sqrt(),
    );
    assert_eq!(evidence.enclosed_volume_source_units_cubed(), None);
    assert_eq!(evidence.center_of_mass_source_units(), None);
    assert_eq!(
        confidence_reasons(&evidence),
        [
            "MESH_REPRESENTATION_CEILING",
            "UNITS_UNRESOLVED",
            "NON_MANIFOLD_EDGE",
            "OPEN_BOUNDARY",
        ]
    );
    assert_eq!(
        warning_codes(&evidence),
        [
            "UNITS_MISSING_REQUIRES_CONFIRMATION",
            "MESH_NOT_EXACT_BREP",
            "NON_MANIFOLD_EDGE",
            "OPEN_BOUNDARY",
            "CLOSED_VOLUME_UNAVAILABLE",
        ]
    );
}

#[test]
fn self_intersection_withholds_closed_measurements_and_requires_review() {
    let evidence = analyze_ascii_stl(SELF_INTERSECTING_TETRAHEDRA, limits())
        .expect("governed intersecting tetrahedra must parse");

    assert_eq!(evidence.triangle_count(), 8);
    assert!(evidence.manifold());
    assert!(evidence.watertight());
    assert!(evidence.consistently_wound());
    assert_eq!(
        evidence.self_intersection(),
        MeshSelfIntersectionState::Detected
    );
    assert_eq!(evidence.aabb_extents_source_units().components(), [5.0; 3]);
    assert_close(
        evidence.surface_area_source_units_squared(),
        48.0 + 16.0 * 3.0_f64.sqrt(),
    );
    assert_eq!(evidence.enclosed_volume_source_units_cubed(), None);
    assert_eq!(evidence.center_of_mass_source_units(), None);
    assert_eq!(
        evidence.confidence().level(),
        GeometryConfidenceLevel::NeedsReview
    );
    assert_eq!(
        confidence_reasons(&evidence),
        [
            "MESH_REPRESENTATION_CEILING",
            "UNITS_UNRESOLVED",
            "SELF_INTERSECTION_DETECTED",
        ]
    );
    assert_eq!(
        warning_codes(&evidence),
        [
            "UNITS_MISSING_REQUIRES_CONFIRMATION",
            "MESH_NOT_EXACT_BREP",
            "SELF_INTERSECTION_DETECTED",
            "CLOSED_VOLUME_UNAVAILABLE",
        ]
    );
}

#[test]
fn coplanar_overlap_remains_indeterminate_without_a_tolerance_policy() {
    let evidence = analyze_ascii_stl(COPLANAR_OVERLAP, limits())
        .expect("bounded coplanar mesh must remain reviewable evidence");

    assert_eq!(evidence.triangle_count(), 2);
    assert!(evidence.manifold());
    assert!(!evidence.watertight());
    assert!(!evidence.consistently_wound());
    assert_eq!(
        evidence.self_intersection(),
        MeshSelfIntersectionState::Indeterminate
    );
    assert_eq!(
        evidence.confidence().level(),
        GeometryConfidenceLevel::NeedsReview
    );
    assert_eq!(
        evidence.aabb_extents_source_units().components(),
        [2.5, 2.5, 0.0]
    );
    assert_close(evidence.surface_area_source_units_squared(), 4.0);
    assert_eq!(evidence.enclosed_volume_source_units_cubed(), None);
    assert_eq!(evidence.center_of_mass_source_units(), None);
    assert_eq!(
        confidence_reasons(&evidence),
        [
            "MESH_REPRESENTATION_CEILING",
            "UNITS_UNRESOLVED",
            "OPEN_BOUNDARY",
            "SELF_INTERSECTION_INDETERMINATE",
        ]
    );
    assert_eq!(
        warning_codes(&evidence),
        [
            "UNITS_MISSING_REQUIRES_CONFIRMATION",
            "MESH_NOT_EXACT_BREP",
            "OPEN_BOUNDARY",
            "SELF_INTERSECTION_INDETERMINATE",
            "CLOSED_VOLUME_UNAVAILABLE",
        ]
    );

    let shared_edge_overlap = b"solid shared-edge-overlap
facet normal 0 0 1
outer loop
vertex 0 0 0
vertex 2 0 0
vertex 0 2 0
endloop
endfacet
facet normal 0 0 1
outer loop
vertex 0 0 0
vertex 2 0 0
vertex 1 1 0
endloop
endfacet
endsolid shared-edge-overlap
";
    let evidence = analyze_ascii_stl(shared_edge_overlap, limits())
        .expect("same-side shared-edge overlap must remain reviewable evidence");
    assert_eq!(
        evidence.self_intersection(),
        MeshSelfIntersectionState::Indeterminate
    );
}

#[test]
fn parser_limits_and_malformed_inputs_fail_with_sanitized_codes() {
    let byte_limited = StlLimits::new(8, 1_000).expect("limit must be valid");
    assert_eq!(
        analyze_ascii_stl(CLOSED_CUBE, byte_limited)
            .expect_err("oversized input must fail")
            .diagnostic_code(),
        "STL_INPUT_LIMIT_EXCEEDED"
    );

    let triangle_limited = StlLimits::new(64 * 1024, 1).expect("limit must be valid");
    assert_eq!(
        analyze_ascii_stl(CLOSED_CUBE, triangle_limited)
            .expect_err("triangle quota must fail")
            .diagnostic_code(),
        "STL_TRIANGLE_LIMIT_EXCEEDED"
    );

    for (bytes, expected) in [
        (&b"not an stl"[..], StlError::InvalidStructure),
        (&b"solid empty\nendsolid empty\n"[..], StlError::EmptyMesh),
        (
            &b"solid bad\nfacet normal NaN 0 0\n"[..],
            StlError::InvalidNumber,
        ),
    ] {
        assert_eq!(analyze_ascii_stl(bytes, limits()), Err(expected));
        assert!(!expected.to_string().contains("solid"));
    }
}

#[test]
fn zero_limits_and_degenerate_triangles_fail_closed() {
    assert_eq!(StlLimits::new(0, 1), Err(StlError::InvalidLimits));
    assert_eq!(StlLimits::new(1, 0), Err(StlError::InvalidLimits));

    let degenerate = b"solid flat
facet normal 0 0 1
outer loop
vertex 0 0 0
vertex 1 0 0
vertex 2 0 0
endloop
endfacet
endsolid flat
";
    assert_eq!(
        analyze_ascii_stl(degenerate, limits()),
        Err(StlError::DegenerateTriangle)
    );

    let overflowing = b"solid huge
facet normal 0 0 1
outer loop
vertex -1e308 0 0
vertex 1e308 0 0
vertex 0 1 0
endloop
endfacet
endsolid huge
";
    assert_eq!(
        analyze_ascii_stl(overflowing, limits()),
        Err(StlError::DegenerateTriangle)
    );

    let overflowing_determinant = b"solid huge
facet normal 0 0 1
outer loop
vertex 1e200 0 0
vertex 1e200 1e70 0
vertex 1e200 0 1e70
endloop
endfacet
endsolid huge
";
    assert_eq!(
        analyze_ascii_stl(overflowing_determinant, limits()),
        Err(StlError::InvalidNumber)
    );
}

#[test]
fn binary_framing_attributes_numbers_and_quotas_fail_closed() {
    let mut truncated = BINARY_CUBE.to_vec();
    truncated.pop();
    assert_eq!(
        analyze_binary_stl(&truncated, limits()),
        Err(StlError::InvalidStructure)
    );

    let triangle_limited = StlLimits::new(64 * 1024, 11).expect("limit must be valid");
    assert_eq!(
        analyze_binary_stl(BINARY_CUBE, triangle_limited),
        Err(StlError::TriangleLimitExceeded)
    );

    let mut attributed = BINARY_CUBE.to_vec();
    attributed[132] = 1;
    assert_eq!(
        analyze_binary_stl(&attributed, limits()),
        Err(StlError::UnsupportedAttributeData)
    );

    let mut non_finite = BINARY_CUBE.to_vec();
    non_finite[84..88].copy_from_slice(&f32::NAN.to_le_bytes());
    assert_eq!(
        analyze_binary_stl(&non_finite, limits()),
        Err(StlError::InvalidNumber)
    );
}

#[test]
fn persisted_adversarial_binary_stl_fails_with_exact_sanitized_diagnostics() {
    for (source, expected) in ADVERSARIAL_BINARY_STL {
        let actual = analyze_binary_stl(source, limits()).expect_err("fixture must fail closed");
        assert_eq!(actual, expected);
        assert_eq!(actual.to_string(), expected.diagnostic_code());
    }
}

#[test]
fn persisted_adversarial_ascii_stl_fails_with_exact_sanitized_diagnostics() {
    for (source, expected) in ADVERSARIAL_ASCII_STL {
        let actual = analyze_stl(source, limits()).expect_err("fixture must fail closed");
        assert_eq!(actual, expected);
        assert_eq!(actual.to_string(), expected.diagnostic_code());
    }
}
