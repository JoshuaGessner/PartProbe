use partprobe_geometry_core::{
    ModelFormat, ModelLengthUnit, RepresentationBasis, UnitResolutionMethod,
};
use partprobe_geometry_import::{
    ASCII_STL_ANALYZER_VERSION, BINARY_STL_ANALYZER_VERSION, StlEncoding, StlError, StlLimits,
    StlMeshEvidence, analyze_ascii_stl, analyze_binary_stl, analyze_stl,
};

const CLOSED_CUBE: &[u8] = include_bytes!("../../../fixtures/models/cube_10mm_ascii.stl");
const BINARY_CUBE: &[u8] = include_bytes!("../../../fixtures/models/cube_10mm_binary.stl");
const OPEN_CUBE: &[u8] = include_bytes!("../../../fixtures/models/open_cube_10mm_ascii.stl");

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
    assert_eq!(evidence.encoding(), StlEncoding::Ascii);
    assert_eq!(evidence.detected_format(), ModelFormat::Stl);
    assert_eq!(evidence.representation(), RepresentationBasis::Mesh);
    assert_eq!(evidence.source_units(), ModelLengthUnit::Unknown);
    assert_eq!(evidence.unit_resolution(), UnitResolutionMethod::Unresolved);
    assert_eq!(evidence.triangle_count(), 12);
    assert!(evidence.manifold());
    assert!(evidence.watertight());
    assert!(evidence.consistently_wound());
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
    assert_eq!(evidence.encoding(), StlEncoding::Binary);
    assert_eq!(evidence.detected_format(), ModelFormat::Stl);
    assert_eq!(evidence.representation(), RepresentationBasis::Mesh);
    assert_eq!(evidence.source_units(), ModelLengthUnit::Unknown);
    assert_eq!(evidence.unit_resolution(), UnitResolutionMethod::Unresolved);
    assert_eq!(evidence.triangle_count(), 12);
    assert!(evidence.manifold());
    assert!(evidence.watertight());
    assert!(evidence.consistently_wound());
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
