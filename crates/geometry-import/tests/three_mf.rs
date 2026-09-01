use std::io::{Cursor, Read, Write};

use partprobe_geometry_core::{
    GeometryConfidenceLevel, ModelFormat, ModelLengthUnit, RepresentationBasis,
    UnitResolutionMethod,
};
use partprobe_geometry_import::{
    MESH_CONFIDENCE_POLICY_VERSION, MESH_SELF_INTERSECTION_ALGORITHM_VERSION,
    MESH_TOPOLOGY_POLICY_VERSION, MeshSelfIntersectionState, MeshTopologyIdentity,
    MeshWeldingStatus, THREE_MF_ANALYZER_VERSION, ThreeMfError, ThreeMfLimits, ThreeMfMeshEvidence,
    analyze_3mf,
};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

const TRANSLATED_CUBE: &[u8] = include_bytes!("../../../fixtures/models/cube_1cm_translated.3mf");
const COMPONENT_CUBE: &[u8] =
    include_bytes!("../../../fixtures/models/cube_1cm_component_scaled_translated.3mf");
const NESTED_COMPONENT_CUBE: &[u8] =
    include_bytes!("../../../fixtures/models/cube_1cm_nested_component_chain.3mf");
const METADATA_CUBE: &[u8] = include_bytes!("../../../fixtures/models/cube_10mm_3mf_metadata.3mf");
const SPLIT_INDEX_SEAM_CUBE: &[u8] =
    include_bytes!("../../../fixtures/models/cube_10mm_3mf_split_index_seam.3mf");
const MICRON_CUBE: &[u8] = include_bytes!("../../../fixtures/models/cube_10mm_3mf_micron.3mf");
const MILLIMETER_CUBE: &[u8] =
    include_bytes!("../../../fixtures/models/cube_10mm_3mf_millimeter.3mf");
const METER_CUBE: &[u8] = include_bytes!("../../../fixtures/models/cube_10mm_3mf_meter.3mf");
const INCH_CUBE: &[u8] = include_bytes!("../../../fixtures/models/cube_10mm_3mf_inch.3mf");
const FOOT_CUBE: &[u8] = include_bytes!("../../../fixtures/models/cube_10mm_3mf_foot.3mf");
const DEFAULT_MM_CUBE: &[u8] =
    include_bytes!("../../../fixtures/models/cube_10mm_3mf_default_mm.3mf");
const ALTERNATE_OPC_CUBES: [&[u8]; 4] = [
    include_bytes!("../../../fixtures/models/cube_10mm_3mf_default_content_type.3mf"),
    include_bytes!("../../../fixtures/models/cube_10mm_3mf_alternate_model_part.3mf"),
    include_bytes!("../../../fixtures/models/cube_10mm_3mf_stored_compression.3mf"),
    include_bytes!("../../../fixtures/models/cube_10mm_3mf_package_thumbnail.3mf"),
];
const ADVERSARIAL_THREE_MF: [(&[u8], ThreeMfError); 32] = [
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_branching_components.3mf"),
        ThreeMfError::UnsupportedModelStructure,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_non_immediate_reference.3mf"),
        ThreeMfError::UnsupportedModelStructure,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_object_metadata.3mf"),
        ThreeMfError::UnsupportedModelStructure,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_relationship_traversal.3mf"),
        ThreeMfError::UnsafePackage,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_case_ambiguous_part.3mf"),
        ThreeMfError::UnsafePackage,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_build_union.3mf"),
        ThreeMfError::UnsupportedModelStructure,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_item_metadata.3mf"),
        ThreeMfError::UnsupportedModelStructure,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_vendor_metadata.3mf"),
        ThreeMfError::UnsupportedModelStructure,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_high_compression_ratio.3mf"),
        ThreeMfError::ArchiveLimitExceeded,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_unsupported_compression.3mf"),
        ThreeMfError::UnsupportedCompression,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_forward_component_reference.3mf"),
        ThreeMfError::UnsupportedModelStructure,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_unused_component_object.3mf"),
        ThreeMfError::UnsupportedModelStructure,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_material_attribute.3mf"),
        ThreeMfError::UnsupportedModelStructure,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_required_extension.3mf"),
        ThreeMfError::UnsupportedRequiredExtension,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_encrypted_entry.3mf"),
        ThreeMfError::UnsafePackage,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_absolute_entry_name.3mf"),
        ThreeMfError::UnsafePackage,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_backslash_entry_name.3mf"),
        ThreeMfError::UnsafePackage,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_directory_entry.3mf"),
        ThreeMfError::UnsafePackage,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_malformed_model_xml.3mf"),
        ThreeMfError::InvalidXml,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_document_type.3mf"),
        ThreeMfError::InvalidXml,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_entry_count_limit.3mf"),
        ThreeMfError::ArchiveLimitExceeded,
    ),
    (
        include_bytes!(
            "../../../fixtures/models/adversarial_3mf_reflected_component_transform.3mf"
        ),
        ThreeMfError::UnsupportedTransform,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_singular_component_transform.3mf"),
        ThreeMfError::UnsupportedTransform,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_non_finite_vertex.3mf"),
        ThreeMfError::InvalidNumber,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_degenerate_triangle.3mf"),
        ThreeMfError::DegenerateTriangle,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_empty_triangle_set.3mf"),
        ThreeMfError::UnsupportedModelStructure,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_missing_vertex_coordinate.3mf"),
        ThreeMfError::InvalidXml,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_out_of_range_triangle_index.3mf"),
        ThreeMfError::UnsupportedModelStructure,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_vertex_limit_exceeded.3mf"),
        ThreeMfError::EntityLimitExceeded,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_duplicate_model_relationship.3mf"),
        ThreeMfError::InvalidPackage,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_missing_model_relationship.3mf"),
        ThreeMfError::InvalidPackage,
    ),
    (
        include_bytes!("../../../fixtures/models/adversarial_3mf_external_model_relationship.3mf"),
        ThreeMfError::UnsafePackage,
    ),
];

fn limits() -> ThreeMfLimits {
    ThreeMfLimits::new(
        64 * 1024,
        16,
        64 * 1024,
        32 * 1024,
        100,
        1_000,
        4,
        3,
        8,
        100,
    )
    .expect("test limits must be valid")
}

fn warning_codes(evidence: &ThreeMfMeshEvidence) -> Vec<&str> {
    evidence
        .warnings()
        .iter()
        .map(|warning| warning.as_str())
        .collect()
}

fn confidence_reasons(evidence: &ThreeMfMeshEvidence) -> Vec<&str> {
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

fn rewrite_part(
    package: &[u8],
    part_name: &str,
    rewrite: impl FnOnce(String) -> String,
) -> Vec<u8> {
    let mut archive = ZipArchive::new(Cursor::new(package)).expect("fixture package must open");
    let mut parts = Vec::new();
    let mut rewrite = Some(rewrite);
    for index in 0..archive.len() {
        let mut part = archive.by_index(index).expect("fixture part must open");
        let name = part.name().to_owned();
        let mut bytes = Vec::new();
        part.read_to_end(&mut bytes)
            .expect("fixture part must read");
        if name == part_name {
            let text = String::from_utf8(bytes).expect("governed XML must be UTF-8");
            bytes = rewrite.take().expect("target part must occur once")(text).into_bytes();
        }
        parts.push((name, bytes));
    }
    assert!(rewrite.is_none(), "target part must exist");

    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for (name, bytes) in parts {
        writer
            .start_file(name, options)
            .expect("test part must start");
        writer.write_all(&bytes).expect("test part must write");
    }
    writer
        .finish()
        .expect("test package must finish")
        .into_inner()
}

fn append_part(package: &[u8], part_name: &str, contents: &[u8]) -> Vec<u8> {
    let mut archive = ZipArchive::new(Cursor::new(package)).expect("fixture package must open");
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for index in 0..archive.len() {
        let mut part = archive.by_index(index).expect("fixture part must open");
        let name = part.name().to_owned();
        let mut bytes = Vec::new();
        part.read_to_end(&mut bytes)
            .expect("fixture part must read");
        writer
            .start_file(name, options)
            .expect("test part must start");
        writer.write_all(&bytes).expect("test part must write");
    }
    writer
        .start_file(part_name, options)
        .expect("additional test part must start");
    writer
        .write_all(contents)
        .expect("additional test part must write");
    writer
        .finish()
        .expect("test package must finish")
        .into_inner()
}

#[test]
fn centimeter_cube_applies_build_transform_and_returns_canonical_mm() {
    let evidence = analyze_3mf(TRANSLATED_CUBE, limits()).expect("3MF cube must parse");

    assert_eq!(evidence.algorithm_version(), THREE_MF_ANALYZER_VERSION);
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
        MeshTopologyIdentity::SourceVertexIndices
    );
    assert_eq!(evidence.welding_status(), MeshWeldingStatus::NotApplied);
    let wire = serde_json::to_value(&evidence).expect("3MF evidence must serialize");
    assert_eq!(
        wire["topology_policy_version"],
        MESH_TOPOLOGY_POLICY_VERSION
    );
    assert_eq!(wire["topology_identity"], "source_vertex_indices");
    assert_eq!(wire["welding_status"], "not_applied");
    assert_eq!(evidence.detected_format(), ModelFormat::ThreeMf);
    assert_eq!(evidence.representation(), RepresentationBasis::Mesh);
    assert_eq!(evidence.source_units(), ModelLengthUnit::Centimeter);
    assert_eq!(evidence.unit_resolution(), UnitResolutionMethod::Declared);
    assert!(evidence.unit_was_explicit());
    assert_eq!(evidence.model_metadata_count(), 0);
    assert_eq!(evidence.preserved_model_metadata_count(), 0);
    assert_eq!(evidence.mesh_object_count(), 1);
    assert_eq!(evidence.mesh_object_id(), 1);
    assert_eq!(evidence.component_object_count(), 0);
    assert!(evidence.component_chain().is_empty());
    assert_eq!(evidence.build_item_count(), 1);
    assert_eq!(evidence.build_object_id(), 1);
    assert_eq!(
        evidence.build_transform_source_units(),
        [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0]
    );
    assert!(evidence.build_transform_applied());
    assert_eq!(evidence.triangle_count(), 12);
    assert!(evidence.manifold());
    assert!(evidence.watertight());
    assert!(evidence.consistently_wound());
    assert_eq!(
        evidence.self_intersection(),
        MeshSelfIntersectionState::NotDetected
    );
    assert_eq!(evidence.confidence().level(), GeometryConfidenceLevel::Low);
    assert_eq!(
        evidence.confidence().reasons()[0].as_str(),
        "MESH_REPRESENTATION_CEILING"
    );
    assert_eq!(evidence.aabb_extents_mm().components(), [10.0; 3]);
    assert_close(evidence.surface_area_mm2(), 600.0);
    assert_close(
        evidence
            .enclosed_volume_mm3()
            .expect("closed volume must be available"),
        1_000.0,
    );
    assert_eq!(
        evidence
            .center_of_mass_mm()
            .expect("closed centroid must be available")
            .components(),
        [25.0, 35.0, 45.0]
    );
    assert_eq!(warning_codes(&evidence), ["MESH_NOT_EXACT_BREP"]);
}

#[test]
fn persisted_alternate_opc_layouts_preserve_the_same_geometry_evidence() {
    for package in ALTERNATE_OPC_CUBES {
        let evidence =
            analyze_3mf(package, limits()).expect("alternate OPC fixture must parse safely");

        assert_eq!(evidence.algorithm_version(), THREE_MF_ANALYZER_VERSION);
        assert_eq!(evidence.source_units(), ModelLengthUnit::Centimeter);
        assert_eq!(evidence.unit_resolution(), UnitResolutionMethod::Declared);
        assert!(evidence.unit_was_explicit());
        assert_eq!(evidence.mesh_object_count(), 1);
        assert_eq!(evidence.component_object_count(), 0);
        assert_eq!(evidence.build_item_count(), 1);
        assert_eq!(evidence.triangle_count(), 12);
        assert!(evidence.manifold());
        assert!(evidence.watertight());
        assert!(evidence.consistently_wound());
        assert_eq!(
            evidence.self_intersection(),
            MeshSelfIntersectionState::NotDetected
        );
        assert_eq!(evidence.confidence().level(), GeometryConfidenceLevel::Low);
        assert_eq!(evidence.aabb_extents_mm().components(), [10.0; 3]);
        assert_close(evidence.surface_area_mm2(), 600.0);
        assert_close(evidence.enclosed_volume_mm3().unwrap(), 1_000.0);
        assert_eq!(
            evidence.center_of_mass_mm().unwrap().components(),
            [25.0, 35.0, 45.0]
        );
        assert_eq!(warning_codes(&evidence), ["MESH_NOT_EXACT_BREP"]);
    }
}

#[test]
fn component_then_build_transform_is_applied_and_retained() {
    let evidence = analyze_3mf(COMPONENT_CUBE, limits()).expect("component 3MF cube must parse");

    assert_eq!(evidence.algorithm_version(), THREE_MF_ANALYZER_VERSION);
    assert_eq!(evidence.source_units(), ModelLengthUnit::Centimeter);
    assert_eq!(evidence.model_metadata_count(), 0);
    assert_eq!(evidence.preserved_model_metadata_count(), 0);
    assert_eq!(evidence.mesh_object_count(), 1);
    assert_eq!(evidence.mesh_object_id(), 1);
    assert_eq!(evidence.component_object_count(), 1);
    let component = &evidence.component_chain()[0];
    assert_eq!(component.object_id(), 2);
    assert_eq!(component.referenced_object_id(), 1);
    assert_eq!(
        component.transform_source_units(),
        [2.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 3.0]
    );
    assert!(component.transform_applied());
    assert_eq!(evidence.build_item_count(), 1);
    assert_eq!(evidence.build_object_id(), 2);
    assert_eq!(
        evidence.build_transform_source_units(),
        [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 4.0, 5.0, 6.0]
    );
    assert!(evidence.build_transform_applied());
    assert_eq!(evidence.triangle_count(), 12);
    assert_eq!(evidence.aabb_extents_mm().components(), [20.0, 10.0, 10.0]);
    assert_close(evidence.surface_area_mm2(), 1_000.0);
    assert_close(
        evidence
            .enclosed_volume_mm3()
            .expect("closed volume must be available"),
        2_000.0,
    );
    assert_eq!(
        evidence
            .center_of_mass_mm()
            .expect("closed centroid must be available")
            .components(),
        [60.0, 75.0, 95.0]
    );
    assert_eq!(warning_codes(&evidence), ["MESH_NOT_EXACT_BREP"]);
}

#[test]
fn nested_linear_component_chain_is_applied_in_leaf_to_build_order() {
    let evidence =
        analyze_3mf(NESTED_COMPONENT_CUBE, limits()).expect("nested component cube must parse");

    assert_eq!(evidence.algorithm_version(), THREE_MF_ANALYZER_VERSION);
    assert_eq!(evidence.source_units(), ModelLengthUnit::Centimeter);
    assert_eq!(evidence.mesh_object_id(), 1);
    assert_eq!(evidence.component_object_count(), 2);
    assert_eq!(evidence.component_chain().len(), 2);
    assert_eq!(evidence.component_chain()[0].object_id(), 2);
    assert_eq!(evidence.component_chain()[0].referenced_object_id(), 1);
    assert_eq!(
        evidence.component_chain()[0].transform_source_units(),
        [2.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 3.0]
    );
    assert!(evidence.component_chain()[0].transform_applied());
    assert_eq!(evidence.component_chain()[1].object_id(), 3);
    assert_eq!(evidence.component_chain()[1].referenced_object_id(), 2);
    assert_eq!(
        evidence.component_chain()[1].transform_source_units(),
        [1.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 2.0]
    );
    assert!(evidence.component_chain()[1].transform_applied());
    assert_eq!(evidence.build_object_id(), 3);
    assert_eq!(evidence.triangle_count(), 12);
    assert_eq!(evidence.aabb_extents_mm().components(), [20.0, 30.0, 10.0]);
    assert_close(evidence.surface_area_mm2(), 2_200.0);
    assert_close(evidence.enclosed_volume_mm3().unwrap(), 6_000.0);
    assert_eq!(
        evidence.center_of_mass_mm().unwrap().components(),
        [70.0, 125.0, 115.0]
    );
    assert_eq!(warning_codes(&evidence), ["MESH_NOT_EXACT_BREP"]);
}

#[test]
fn bounded_model_metadata_is_counted_but_never_retained_or_interpreted() {
    let evidence = analyze_3mf(METADATA_CUBE, limits()).expect("metadata 3MF cube must parse");

    assert_eq!(evidence.algorithm_version(), THREE_MF_ANALYZER_VERSION);
    assert_eq!(evidence.source_units(), ModelLengthUnit::Millimeter);
    assert_eq!(evidence.model_metadata_count(), 3);
    assert_eq!(evidence.preserved_model_metadata_count(), 1);
    assert_eq!(evidence.aabb_extents_mm().components(), [10.0; 3]);
    assert_close(evidence.surface_area_mm2(), 600.0);
    assert_close(evidence.enclosed_volume_mm3().unwrap(), 1_000.0);
    assert_eq!(
        warning_codes(&evidence),
        ["MESH_NOT_EXACT_BREP", "THREE_MF_METADATA_NOT_INTERPRETED"]
    );
    let debug = format!("{evidence:?}");
    assert!(!debug.contains("Governed 10 mm cube"));
    assert!(!debug.contains("PartProbe fixture generator"));

    let metadata_limited = ThreeMfLimits::new(
        64 * 1024,
        16,
        64 * 1024,
        32 * 1024,
        100,
        1_000,
        2,
        1,
        2,
        100,
    )
    .unwrap();
    assert_eq!(
        analyze_3mf(METADATA_CUBE, metadata_limited),
        Err(ThreeMfError::EntityLimitExceeded)
    );

    for unsupported in [
        rewrite_part(METADATA_CUBE, "3D/3dmodel.model", |xml| {
            xml.replace(
                "  <metadata name=\"Title\">Governed 10 mm cube</metadata>",
                "  <metadata name=\"Title\">First</metadata>\n  <metadata name=\"Title\">Second</metadata>",
            )
        }),
        rewrite_part(METADATA_CUBE, "3D/3dmodel.model", |xml| {
            xml.replace("preserve=\"true\"", "preserve=\"yes\"")
        }),
        rewrite_part(METADATA_CUBE, "3D/3dmodel.model", |xml| {
            xml.replace("name=\"Title\"", "name=\"ShopSecret\"")
        }),
        rewrite_part(METADATA_CUBE, "3D/3dmodel.model", |xml| {
            xml.replace("name=\"Title\"", "name=\"Title\" type=\"xs:string\"")
        }),
        rewrite_part(METADATA_CUBE, "3D/3dmodel.model", |xml| {
            xml.replace(
                "      <mesh>",
                "      <metadatagroup><metadata name=\"Title\">Nested</metadata></metadatagroup>\n      <mesh>",
            )
        }),
    ] {
        assert_eq!(
            analyze_3mf(&unsupported, limits()),
            Err(ThreeMfError::UnsupportedModelStructure)
        );
    }
}

#[test]
fn distinct_vertex_indices_at_equal_coordinates_form_an_open_reviewable_seam() {
    let evidence = analyze_3mf(SPLIT_INDEX_SEAM_CUBE, limits())
        .expect("bounded split-index seam must remain reviewable geometry evidence");

    assert_eq!(evidence.algorithm_version(), THREE_MF_ANALYZER_VERSION);
    assert_eq!(evidence.triangle_count(), 12);
    assert!(evidence.manifold());
    assert!(!evidence.watertight());
    assert!(!evidence.consistently_wound());
    assert_eq!(
        evidence.self_intersection(),
        MeshSelfIntersectionState::Detected
    );
    assert_eq!(
        evidence.confidence().level(),
        GeometryConfidenceLevel::NeedsReview
    );
    assert_eq!(evidence.aabb_extents_mm().components(), [10.0; 3]);
    assert_close(evidence.surface_area_mm2(), 600.0);
    assert_eq!(evidence.enclosed_volume_mm3(), None);
    assert_eq!(evidence.center_of_mass_mm(), None);
    assert_eq!(
        confidence_reasons(&evidence),
        [
            "MESH_REPRESENTATION_CEILING",
            "OPEN_BOUNDARY",
            "SELF_INTERSECTION_DETECTED",
        ]
    );
    assert_eq!(
        warning_codes(&evidence),
        [
            "MESH_NOT_EXACT_BREP",
            "OPEN_BOUNDARY",
            "SELF_INTERSECTION_DETECTED",
            "CLOSED_VOLUME_UNAVAILABLE",
        ]
    );
}

#[test]
fn component_limits_references_and_transform_policy_fail_closed() {
    let object_limited = ThreeMfLimits::new(
        64 * 1024,
        16,
        64 * 1024,
        32 * 1024,
        100,
        1_000,
        1,
        1,
        8,
        100,
    )
    .unwrap();
    assert_eq!(
        analyze_3mf(COMPONENT_CUBE, object_limited),
        Err(ThreeMfError::EntityLimitExceeded)
    );

    let second_component = rewrite_part(COMPONENT_CUBE, "3D/3dmodel.model", |xml| {
        xml.replace(
            "      </components>",
            "        <component objectid=\"1\" />\n      </components>",
        )
    });
    assert_eq!(
        analyze_3mf(&second_component, limits()),
        Err(ThreeMfError::UnsupportedModelStructure)
    );

    let component_limited = ThreeMfLimits::new(
        64 * 1024,
        16,
        64 * 1024,
        32 * 1024,
        100,
        1_000,
        4,
        1,
        8,
        100,
    )
    .unwrap();
    assert_eq!(
        analyze_3mf(NESTED_COMPONENT_CUBE, component_limited),
        Err(ThreeMfError::EntityLimitExceeded)
    );

    for invalid_reference in [
        rewrite_part(COMPONENT_CUBE, "3D/3dmodel.model", |xml| {
            xml.replace("<component objectid=\"1\"", "<component objectid=\"7\"")
        }),
        rewrite_part(COMPONENT_CUBE, "3D/3dmodel.model", |xml| {
            xml.replace("<object id=\"2\"", "<object id=\"1\"")
        }),
        rewrite_part(COMPONENT_CUBE, "3D/3dmodel.model", |xml| {
            xml.replace("<item objectid=\"2\"", "<item objectid=\"1\"")
        }),
        rewrite_part(NESTED_COMPONENT_CUBE, "3D/3dmodel.model", |xml| {
            xml.replace("<component objectid=\"2\"", "<component objectid=\"1\"")
        }),
    ] {
        assert_eq!(
            analyze_3mf(&invalid_reference, limits()),
            Err(ThreeMfError::UnsupportedModelStructure)
        );
    }

    let oversized_resource_id = rewrite_part(COMPONENT_CUBE, "3D/3dmodel.model", |xml| {
        xml.replace("<object id=\"1\"", "<object id=\"2147483648\"")
    });
    assert_eq!(
        analyze_3mf(&oversized_resource_id, limits()),
        Err(ThreeMfError::InvalidNumber)
    );
}

#[test]
fn persisted_adversarial_packages_fail_with_exact_sanitized_diagnostics() {
    for (package, expected) in ADVERSARIAL_THREE_MF {
        assert_eq!(analyze_3mf(package, limits()), Err(expected));
        assert!(expected.diagnostic_code().starts_with("THREE_MF_"));
        assert!(!expected.to_string().contains("3D/"));
    }
}

#[test]
fn archive_xml_and_entity_limits_fail_closed() {
    for (limited, expected) in [
        (
            ThreeMfLimits::new(8, 16, 64 * 1024, 32 * 1024, 100, 1_000, 2, 1, 8, 100).unwrap(),
            ThreeMfError::InputLimitExceeded,
        ),
        (
            ThreeMfLimits::new(64 * 1024, 2, 64 * 1024, 32 * 1024, 100, 1_000, 2, 1, 8, 100)
                .unwrap(),
            ThreeMfError::ArchiveLimitExceeded,
        ),
        (
            ThreeMfLimits::new(64 * 1024, 16, 1, 32 * 1024, 100, 1_000, 2, 1, 8, 100).unwrap(),
            ThreeMfError::ArchiveLimitExceeded,
        ),
        (
            ThreeMfLimits::new(64 * 1024, 16, 64 * 1024, 8, 100, 1_000, 2, 1, 8, 100).unwrap(),
            ThreeMfError::InvalidXml,
        ),
        (
            ThreeMfLimits::new(64 * 1024, 16, 64 * 1024, 32 * 1024, 1, 1_000, 2, 1, 8, 100)
                .unwrap(),
            ThreeMfError::EntityLimitExceeded,
        ),
        (
            ThreeMfLimits::new(64 * 1024, 16, 64 * 1024, 32 * 1024, 100, 1, 2, 1, 8, 100).unwrap(),
            ThreeMfError::EntityLimitExceeded,
        ),
        (
            ThreeMfLimits::new(64 * 1024, 16, 64 * 1024, 32 * 1024, 100, 1_000, 2, 1, 8, 1)
                .unwrap(),
            ThreeMfError::ArchiveLimitExceeded,
        ),
    ] {
        assert_eq!(analyze_3mf(TRANSLATED_CUBE, limited), Err(expected));
    }
}

#[test]
fn invalid_limits_and_non_packages_have_sanitized_failures() {
    assert_eq!(
        ThreeMfLimits::new(0, 1, 1, 1, 1, 1, 1, 1, 1, 1),
        Err(ThreeMfError::InvalidLimits)
    );
    assert_eq!(
        ThreeMfLimits::new(1, 1, 1, 1, 1, 1, 1, 1, 0, 1),
        Err(ThreeMfError::InvalidLimits)
    );
    assert_eq!(
        analyze_3mf(b"not a package", limits()),
        Err(ThreeMfError::InvalidPackage)
    );
    assert!(!ThreeMfError::InvalidPackage.to_string().contains("package"));
}

#[test]
fn external_relationships_and_traversal_targets_are_rejected() {
    let external = rewrite_part(TRANSLATED_CUBE, "_rels/.rels", |xml| {
        xml.replace(
            "<Relationship Id=\"rel0\"",
            "<Relationship TargetMode=\"External\" Id=\"rel0\"",
        )
    });
    assert_eq!(
        analyze_3mf(&external, limits()),
        Err(ThreeMfError::UnsafePackage)
    );

    let traversal = rewrite_part(TRANSLATED_CUBE, "_rels/.rels", |xml| {
        xml.replace(
            "Target=\"/3D/3dmodel.model\"",
            "Target=\"/../escape.model\"",
        )
    });
    assert_eq!(
        analyze_3mf(&traversal, limits()),
        Err(ThreeMfError::UnsafePackage)
    );

    for unsafe_package in [
        append_part(TRANSLATED_CUBE, "../escape.xml", b"unsafe"),
        append_part(TRANSLATED_CUBE, "3d/3dmodel.model", b"ambiguous"),
    ] {
        assert_eq!(
            analyze_3mf(&unsafe_package, limits()),
            Err(ThreeMfError::UnsafePackage)
        );
    }
}

#[test]
fn unsupported_extensions_structures_and_numbers_do_not_succeed_silently() {
    let extension = rewrite_part(TRANSLATED_CUBE, "3D/3dmodel.model", |xml| {
        xml.replace(
            "<model unit=\"centimeter\"",
            "<model requiredextensions=\"foo\" xmlns:foo=\"urn:example\" unit=\"centimeter\"",
        )
    });
    assert_eq!(
        analyze_3mf(&extension, limits()),
        Err(ThreeMfError::UnsupportedRequiredExtension)
    );

    let component = rewrite_part(TRANSLATED_CUBE, "3D/3dmodel.model", |xml| {
        xml.replace("<mesh>", "<components>")
            .replace("</mesh>", "</components>")
    });
    assert_eq!(
        analyze_3mf(&component, limits()),
        Err(ThreeMfError::UnsupportedModelStructure)
    );

    let material_attribute = rewrite_part(TRANSLATED_CUBE, "3D/3dmodel.model", |xml| {
        xml.replacen("<triangle ", "<triangle pid=\"2\" ", 1)
    });
    assert_eq!(
        analyze_3mf(&material_attribute, limits()),
        Err(ThreeMfError::UnsupportedModelStructure)
    );
}

#[test]
fn core_default_millimeters_are_explicitly_distinguished_from_a_unit_attribute() {
    let default_units = rewrite_part(TRANSLATED_CUBE, "3D/3dmodel.model", |xml| {
        xml.replace(" unit=\"centimeter\"", "")
    });
    let evidence = analyze_3mf(&default_units, limits()).expect("core default units must resolve");

    assert_eq!(evidence.source_units(), ModelLengthUnit::Millimeter);
    assert_eq!(evidence.unit_resolution(), UnitResolutionMethod::Declared);
    assert!(!evidence.unit_was_explicit());
    assert_eq!(evidence.aabb_extents_mm().components(), [1.0; 3]);
    assert_eq!(
        evidence
            .center_of_mass_mm()
            .expect("default-unit centroid must be available")
            .components(),
        [2.5, 3.5, 4.5]
    );

    let unsupported = rewrite_part(TRANSLATED_CUBE, "3D/3dmodel.model", |xml| {
        xml.replace("unit=\"centimeter\"", "unit=\"parsec\"")
    });
    assert_eq!(
        analyze_3mf(&unsupported, limits()),
        Err(ThreeMfError::UnsupportedUnit)
    );
}

#[test]
fn persisted_core_unit_fixtures_resolve_to_the_same_canonical_cube() {
    for (package, unit, explicit) in [
        (MICRON_CUBE, ModelLengthUnit::Micrometer, true),
        (MILLIMETER_CUBE, ModelLengthUnit::Millimeter, true),
        (METER_CUBE, ModelLengthUnit::Meter, true),
        (INCH_CUBE, ModelLengthUnit::Inch, true),
        (FOOT_CUBE, ModelLengthUnit::Foot, true),
        (DEFAULT_MM_CUBE, ModelLengthUnit::Millimeter, false),
    ] {
        let evidence = analyze_3mf(package, limits()).expect("governed unit fixture must resolve");

        assert_eq!(evidence.source_units(), unit);
        assert_eq!(evidence.unit_was_explicit(), explicit);
        assert_eq!(
            evidence.build_transform_source_units(),
            [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,]
        );
        assert!(!evidence.build_transform_applied());
        assert_eq!(evidence.aabb_extents_mm().components(), [10.0; 3]);
        assert_close(evidence.surface_area_mm2(), 600.0);
        assert_close(
            evidence
                .enclosed_volume_mm3()
                .expect("closed volume must be available"),
            1_000.0,
        );
        assert_eq!(
            evidence
                .center_of_mass_mm()
                .expect("closed centroid must be available")
                .components(),
            [5.0; 3]
        );
    }
}
