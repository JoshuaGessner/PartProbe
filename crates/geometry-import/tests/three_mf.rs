use std::io::{Cursor, Read, Write};

use partprobe_geometry_core::{
    ModelFormat, ModelLengthUnit, RepresentationBasis, UnitResolutionMethod,
};
use partprobe_geometry_import::{
    THREE_MF_ANALYZER_VERSION, ThreeMfError, ThreeMfLimits, ThreeMfMeshEvidence, analyze_3mf,
};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

const TRANSLATED_CUBE: &[u8] = include_bytes!("../../../fixtures/models/cube_1cm_translated.3mf");
const COMPONENT_CUBE: &[u8] =
    include_bytes!("../../../fixtures/models/cube_1cm_component_scaled_translated.3mf");
const MICRON_CUBE: &[u8] = include_bytes!("../../../fixtures/models/cube_10mm_3mf_micron.3mf");
const MILLIMETER_CUBE: &[u8] =
    include_bytes!("../../../fixtures/models/cube_10mm_3mf_millimeter.3mf");
const METER_CUBE: &[u8] = include_bytes!("../../../fixtures/models/cube_10mm_3mf_meter.3mf");
const INCH_CUBE: &[u8] = include_bytes!("../../../fixtures/models/cube_10mm_3mf_inch.3mf");
const FOOT_CUBE: &[u8] = include_bytes!("../../../fixtures/models/cube_10mm_3mf_foot.3mf");
const DEFAULT_MM_CUBE: &[u8] =
    include_bytes!("../../../fixtures/models/cube_10mm_3mf_default_mm.3mf");

fn limits() -> ThreeMfLimits {
    ThreeMfLimits::new(64 * 1024, 16, 64 * 1024, 32 * 1024, 100, 1_000, 2, 1, 100)
        .expect("test limits must be valid")
}

fn warning_codes(evidence: &ThreeMfMeshEvidence) -> Vec<&str> {
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
    assert_eq!(evidence.detected_format(), ModelFormat::ThreeMf);
    assert_eq!(evidence.representation(), RepresentationBasis::Mesh);
    assert_eq!(evidence.source_units(), ModelLengthUnit::Centimeter);
    assert_eq!(evidence.unit_resolution(), UnitResolutionMethod::Declared);
    assert!(evidence.unit_was_explicit());
    assert_eq!(evidence.mesh_object_count(), 1);
    assert_eq!(evidence.mesh_object_id(), 1);
    assert_eq!(evidence.component_object_count(), 0);
    assert_eq!(evidence.component_object_id(), None);
    assert_eq!(evidence.component_mesh_object_id(), None);
    assert_eq!(evidence.component_transform_source_units(), None);
    assert!(!evidence.component_transform_applied());
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
fn component_then_build_transform_is_applied_and_retained() {
    let evidence = analyze_3mf(COMPONENT_CUBE, limits()).expect("component 3MF cube must parse");

    assert_eq!(evidence.algorithm_version(), THREE_MF_ANALYZER_VERSION);
    assert_eq!(evidence.source_units(), ModelLengthUnit::Centimeter);
    assert_eq!(evidence.mesh_object_count(), 1);
    assert_eq!(evidence.mesh_object_id(), 1);
    assert_eq!(evidence.component_object_count(), 1);
    assert_eq!(evidence.component_object_id(), Some(2));
    assert_eq!(evidence.component_mesh_object_id(), Some(1));
    assert_eq!(
        evidence.component_transform_source_units(),
        Some([2.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 3.0])
    );
    assert!(evidence.component_transform_applied());
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
fn component_limits_references_and_transform_policy_fail_closed() {
    let object_limited =
        ThreeMfLimits::new(64 * 1024, 16, 64 * 1024, 32 * 1024, 100, 1_000, 1, 1, 100).unwrap();
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
    ] {
        assert_eq!(
            analyze_3mf(&invalid_reference, limits()),
            Err(ThreeMfError::UnsupportedModelStructure)
        );
    }

    for unsupported_transform in [
        rewrite_part(COMPONENT_CUBE, "3D/3dmodel.model", |xml| {
            xml.replace(
                "transform=\"2 0 0 0 1 0 0 0 1 1 2 3\"",
                "transform=\"-2 0 0 0 1 0 0 0 1 1 2 3\"",
            )
        }),
        rewrite_part(COMPONENT_CUBE, "3D/3dmodel.model", |xml| {
            xml.replace(
                "transform=\"2 0 0 0 1 0 0 0 1 1 2 3\"",
                "transform=\"0 0 0 0 1 0 0 0 1 1 2 3\"",
            )
        }),
    ] {
        assert_eq!(
            analyze_3mf(&unsupported_transform, limits()),
            Err(ThreeMfError::UnsupportedTransform)
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
fn archive_xml_and_entity_limits_fail_closed() {
    for (limited, expected) in [
        (
            ThreeMfLimits::new(8, 16, 64 * 1024, 32 * 1024, 100, 1_000, 2, 1, 100).unwrap(),
            ThreeMfError::InputLimitExceeded,
        ),
        (
            ThreeMfLimits::new(64 * 1024, 2, 64 * 1024, 32 * 1024, 100, 1_000, 2, 1, 100).unwrap(),
            ThreeMfError::ArchiveLimitExceeded,
        ),
        (
            ThreeMfLimits::new(64 * 1024, 16, 1, 32 * 1024, 100, 1_000, 2, 1, 100).unwrap(),
            ThreeMfError::ArchiveLimitExceeded,
        ),
        (
            ThreeMfLimits::new(64 * 1024, 16, 64 * 1024, 8, 100, 1_000, 2, 1, 100).unwrap(),
            ThreeMfError::InvalidXml,
        ),
        (
            ThreeMfLimits::new(64 * 1024, 16, 64 * 1024, 32 * 1024, 1, 1_000, 2, 1, 100).unwrap(),
            ThreeMfError::EntityLimitExceeded,
        ),
        (
            ThreeMfLimits::new(64 * 1024, 16, 64 * 1024, 32 * 1024, 100, 1, 2, 1, 100).unwrap(),
            ThreeMfError::EntityLimitExceeded,
        ),
        (
            ThreeMfLimits::new(64 * 1024, 16, 64 * 1024, 32 * 1024, 100, 1_000, 2, 1, 1).unwrap(),
            ThreeMfError::ArchiveLimitExceeded,
        ),
    ] {
        assert_eq!(analyze_3mf(TRANSLATED_CUBE, limited), Err(expected));
    }
}

#[test]
fn invalid_limits_and_non_packages_have_sanitized_failures() {
    assert_eq!(
        ThreeMfLimits::new(0, 1, 1, 1, 1, 1, 1, 1, 1),
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

    let non_finite = rewrite_part(TRANSLATED_CUBE, "3D/3dmodel.model", |xml| {
        xml.replacen("x=\"0\"", "x=\"NaN\"", 1)
    });
    assert_eq!(
        analyze_3mf(&non_finite, limits()),
        Err(ThreeMfError::InvalidNumber)
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
