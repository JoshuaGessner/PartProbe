use partprobe_domain::RuleVersion;
use partprobe_geometry_core::{
    AnalysisProfile, AnalysisProfileId, DISPLAY_TESSELLATION_PROFILE_REFERENCE,
    DisplayCoordinateSpace, DisplayGeometryReference, DisplayMeshChunkDescriptor,
    DisplayTessellationProfile, ExactStepEnvelopeDerivative, GEOMETRY_DISPLAY_SCENE_EVIDENCE_STATE,
    GEOMETRY_DISPLAY_SCENE_REFERENCE, GeometryConfidence, GeometryConfidenceLevel,
    GeometryConfidenceReasonCode, GeometryDisplaySceneManifest, GeometryStage, GeometryStageReport,
    GeometryWarning, GeometryWarningCode, MAX_DISPLAY_SCENE_BYTES, MAX_DISPLAY_SCENE_CHUNKS,
    MAX_DISPLAY_SCENE_TRIANGLES, MAX_DISPLAY_SCENE_VERTICES, ModelAssetDescriptor, ModelFormat,
    ModelLengthUnit, ProvisionalGeometryDecimal, ProvisionalGeometrySnapshot, RepresentationBasis,
    Sha256Digest, StageStatus, WarningSeverity,
};

fn digest() -> Sha256Digest {
    Sha256Digest::new("a".repeat(64)).expect("digest must be valid")
}

fn digest_for(character: char) -> Sha256Digest {
    Sha256Digest::new(character.to_string().repeat(64)).expect("digest must be valid")
}

fn display_profile() -> DisplayTessellationProfile {
    DisplayTessellationProfile::new(
        ProvisionalGeometryDecimal::new("0.1").expect("linear tolerance must be valid"),
        ProvisionalGeometryDecimal::new("12").expect("angular tolerance must be valid"),
    )
    .expect("profile must be valid")
}

fn display_chunk(sequence: u32) -> DisplayMeshChunkDescriptor {
    DisplayMeshChunkDescriptor::new(
        sequence,
        DisplayGeometryReference::new("body-0").expect("reference must be valid"),
        8,
        12,
        digest_for('c'),
        digest_for('d'),
    )
    .expect("chunk must be valid")
}

fn display_scene() -> GeometryDisplaySceneManifest {
    GeometryDisplaySceneManifest::new(
        digest(),
        digest_for('b'),
        RepresentationBasis::ExactBrep,
        DisplayCoordinateSpace::CanonicalMillimeters,
        ModelLengthUnit::Millimeter,
        display_profile(),
        ["10", "20", "30"]
            .map(|value| ProvisionalGeometryDecimal::new(value).expect("extent must be valid")),
        vec![display_chunk(0)],
    )
    .expect("scene must be valid")
}

#[test]
fn source_descriptor_preserves_hash_size_and_format_mismatch() {
    let descriptor =
        ModelAssetDescriptor::new(digest(), 42, Some(ModelFormat::Stl), ModelFormat::Step)
            .expect("descriptor must be valid");

    assert_eq!(descriptor.source_hash().as_str(), "a".repeat(64));
    assert_eq!(descriptor.byte_size(), 42);
    assert!(descriptor.has_format_mismatch());
}

#[test]
fn source_hash_and_empty_asset_are_rejected() {
    assert!(Sha256Digest::new("ABC").is_err());
    assert!(ModelAssetDescriptor::new(digest(), 0, None, ModelFormat::Unknown).is_err());
}

#[test]
fn deserialization_cannot_bypass_source_invariants() {
    let value = serde_json::json!({
        "source_hash": "A".repeat(64),
        "byte_size": 0,
        "claimed_format": null,
        "detected_format": "step"
    });

    assert!(serde_json::from_value::<ModelAssetDescriptor>(value).is_err());
}

#[test]
fn stage_reports_enforce_warning_consistency() {
    let warning = GeometryWarning {
        code: GeometryWarningCode::new("FORMAT_MISMATCH").expect("code must be valid"),
        stage: GeometryStage::Identify,
        severity: WarningSeverity::Warning,
    };

    assert!(
        GeometryStageReport::new(
            GeometryStage::Identify,
            StageStatus::Succeeded,
            vec![warning.clone()]
        )
        .is_err()
    );
    assert!(
        GeometryStageReport::new(
            GeometryStage::Parse,
            StageStatus::SucceededWithWarnings,
            vec![warning]
        )
        .is_err()
    );
}

#[test]
fn profile_and_status_contracts_are_serializable() {
    let profile = AnalysisProfile {
        id: AnalysisProfileId::new("step-basic").expect("ID must be valid"),
        version: RuleVersion::new(1, 0, 0),
    };
    let value = serde_json::to_value(profile).expect("profile must serialize");

    assert_eq!(value["id"], "step-basic");
    assert!(StageStatus::Succeeded.permits_authoritative_output());
    assert!(!StageStatus::NeedsUserInput.permits_authoritative_output());
}

#[test]
fn confidence_requires_unique_explicit_reasons() {
    let ceiling = GeometryConfidenceReasonCode::new("MESH_REPRESENTATION_CEILING")
        .expect("reason must be valid");
    let confidence = GeometryConfidence::new(GeometryConfidenceLevel::Low, vec![ceiling.clone()])
        .expect("confidence must be valid");

    assert_eq!(confidence.level(), GeometryConfidenceLevel::Low);
    assert_eq!(confidence.reasons().len(), 1);
    assert_eq!(
        confidence.reasons()[0].as_str(),
        "MESH_REPRESENTATION_CEILING"
    );
    assert!(GeometryConfidence::new(GeometryConfidenceLevel::Low, Vec::new()).is_err());
    assert!(
        GeometryConfidence::new(
            GeometryConfidenceLevel::NeedsReview,
            vec![ceiling.clone(), ceiling]
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<GeometryConfidence>(serde_json::json!({
            "level": "low",
            "reasons": ["MESH_REPRESENTATION_CEILING", "MESH_REPRESENTATION_CEILING"]
        }))
        .is_err()
    );
}

#[test]
fn provisional_snapshot_round_trips_the_existing_worker_schema() {
    let snapshot = ProvisionalGeometrySnapshot::new(
        digest(),
        "8.0.0",
        3,
        1,
        1,
        ProvisionalGeometryDecimal::new("600").expect("area must be canonical"),
        ProvisionalGeometryDecimal::new("1000").expect("volume must be canonical"),
        ["5", "5", "-5.25"].map(|value| {
            ProvisionalGeometryDecimal::new(value).expect("centroid must be canonical")
        }),
    )
    .expect("snapshot must be valid");
    let value = serde_json::to_value(&snapshot).expect("snapshot must serialize");
    let decoded: ProvisionalGeometrySnapshot =
        serde_json::from_value(value.clone()).expect("snapshot must deserialize");

    assert_eq!(decoded, snapshot);
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["evidence_state"], "provisional_spike");
    assert_eq!(value["representation"], "exact_brep");
    assert_eq!(value["canonical_units"], "millimeter");
    assert_eq!(decoded.schema_version(), 1);
    assert_eq!(decoded.evidence_state(), "provisional_spike");
    assert_eq!(
        decoded.representation(),
        partprobe_geometry_core::RepresentationBasis::ExactBrep
    );
    assert_eq!(
        decoded.canonical_units(),
        partprobe_geometry_core::ModelLengthUnit::Millimeter
    );
    assert_eq!(decoded.occt_version(), "8.0.0");
    assert_eq!(decoded.adapter_abi_version(), 3);
    assert_eq!(decoded.decimal_scale(), 6);
    assert_eq!(decoded.transferred_roots(), 1);
    assert_eq!(decoded.surface_area_mm2(), "600");
    assert_eq!(decoded.enclosed_volume_mm3(), "1000");
    assert_eq!(decoded.center_of_mass_mm(), ["5", "5", "-5.25"]);
}

#[test]
fn provisional_snapshot_rejects_noncanonical_or_unsupported_evidence() {
    for value in ["", "01", "1.0", "1.1234567", "-0", "NaN", "1e3"] {
        assert!(ProvisionalGeometryDecimal::new(value).is_err(), "{value}");
    }

    let invalid = serde_json::json!({
        "schema_version": 2,
        "evidence_state": "authoritative",
        "source_hash": "a".repeat(64),
        "representation": "mesh",
        "canonical_units": "inch",
        "occt_version": "8.0.0",
        "adapter_abi_version": 0,
        "decimal_scale": 7,
        "transferred_roots": 0,
        "solid_body_count": 0,
        "surface_area_mm2": "-1",
        "enclosed_volume_mm3": "-1",
        "center_of_mass_mm": ["0", "0", "0"]
    });

    assert!(serde_json::from_value::<ProvisionalGeometrySnapshot>(invalid).is_err());
}

#[test]
fn exact_step_envelope_derivative_is_additive_source_bound_evidence() {
    let derivative = ExactStepEnvelopeDerivative::new(
        digest(),
        ["12", "8", "5"]
            .map(|value| ProvisionalGeometryDecimal::new(value).expect("extent must be canonical")),
    )
    .expect("positive exact STEP extents must be valid");
    let value = serde_json::to_value(&derivative).expect("derivative must serialize");
    let decoded: ExactStepEnvelopeDerivative =
        serde_json::from_value(value.clone()).expect("derivative must deserialize");

    assert_eq!(decoded, derivative);
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["evidence_state"], "provisional_exact_step_envelope");
    assert_eq!(value["representation"], "exact_brep");
    assert_eq!(value["canonical_units"], "millimeter");
    assert_eq!(decoded.source_hash(), &digest());
    assert_eq!(
        decoded
            .aabb_extents_mm()
            .each_ref()
            .map(|value| value.as_str()),
        ["12", "8", "5"]
    );
}

#[test]
fn exact_step_envelope_derivative_rejects_zero_or_false_authority() {
    assert!(
        ExactStepEnvelopeDerivative::new(
            digest(),
            ["10", "0", "5"].map(|value| {
                ProvisionalGeometryDecimal::new(value).expect("test decimal must be canonical")
            }),
        )
        .is_err()
    );

    let invalid = serde_json::json!({
        "schema_version": 2,
        "evidence_state": "authoritative",
        "source_hash": "a".repeat(64),
        "representation": "mesh",
        "canonical_units": "inch",
        "decimal_scale": 7,
        "aabb_extents_mm": ["10", "10", "10"]
    });
    assert!(serde_json::from_value::<ExactStepEnvelopeDerivative>(invalid).is_err());
}

#[test]
fn display_scene_manifest_is_source_and_analysis_bound_without_geometry_payloads() {
    let scene = display_scene();
    let value = serde_json::to_value(&scene).expect("scene must serialize");
    let decoded: GeometryDisplaySceneManifest =
        serde_json::from_value(value.clone()).expect("scene must deserialize");

    assert_eq!(decoded, scene);
    assert_eq!(decoded.reference(), GEOMETRY_DISPLAY_SCENE_REFERENCE);
    assert_eq!(
        decoded.evidence_state(),
        GEOMETRY_DISPLAY_SCENE_EVIDENCE_STATE
    );
    assert_eq!(decoded.source_hash(), &digest());
    assert_eq!(decoded.analysis_output_hash(), &digest_for('b'));
    assert_eq!(decoded.total_vertex_count(), 8);
    assert_eq!(decoded.total_triangle_count(), 12);
    assert_eq!(decoded.total_byte_length(), 240);
    assert_eq!(decoded.chunks()[0].vertex_byte_length(), 96);
    assert_eq!(decoded.chunks()[0].index_byte_length(), 144);
    assert_eq!(decoded.chunks()[0].geometry_reference().as_str(), "body-0");
    assert_eq!(
        decoded.tessellation_profile().reference(),
        DISPLAY_TESSELLATION_PROFILE_REFERENCE
    );
    assert_eq!(value["position_encoding"], "float32x3_little_endian");
    assert_eq!(value["index_encoding"], "uint32_little_endian");
    assert_eq!(value["primitive_topology"], "triangle_list");

    let serialized = serde_json::to_string(&scene).expect("scene must serialize");
    for forbidden in [
        "source_path",
        "directory",
        "file_name",
        "positions",
        "indices",
        "vertex_buffer",
        "index_buffer",
    ] {
        assert!(!serialized.contains(forbidden), "found {forbidden}");
    }
    assert!(DisplayGeometryReference::new("../private/model.step").is_err());
    assert!(DisplayGeometryReference::new("C:private-model").is_err());
    assert!(DisplayGeometryReference::new("..").is_err());
}

#[test]
fn display_scene_manifest_rejects_false_authority_and_unit_mismatch() {
    assert!(
        GeometryDisplaySceneManifest::new(
            digest(),
            digest_for('b'),
            RepresentationBasis::ExactBrep,
            DisplayCoordinateSpace::UnresolvedSourceCoordinates,
            ModelLengthUnit::Unknown,
            display_profile(),
            ["10", "20", "30"].map(|value| {
                ProvisionalGeometryDecimal::new(value).expect("extent must be valid")
            }),
            vec![display_chunk(0)],
        )
        .is_err()
    );

    let mutations = [
        ("schema_version", serde_json::json!(2)),
        ("reference", serde_json::json!("geometry-display-scene-v2")),
        ("evidence_state", serde_json::json!("measurement_authority")),
        ("representation", serde_json::json!("unknown")),
        ("units", serde_json::json!("unknown")),
        ("aabb_extents", serde_json::json!(["10", "0", "30"])),
    ];
    for (field, replacement) in mutations {
        let mut invalid = serde_json::to_value(display_scene()).expect("scene must serialize");
        invalid[field] = replacement;
        assert!(
            serde_json::from_value::<GeometryDisplaySceneManifest>(invalid).is_err(),
            "accepted invalid {field}"
        );
    }

    let mut unexpected_payload =
        serde_json::to_value(display_scene()).expect("scene must serialize");
    unexpected_payload["source_path"] = serde_json::json!("/private/model.step");
    assert!(serde_json::from_value::<GeometryDisplaySceneManifest>(unexpected_payload).is_err());

    let mut invalid_profile =
        serde_json::to_value(display_profile()).expect("profile must serialize");
    invalid_profile["version"] = serde_json::json!({"major": 2, "minor": 0, "patch": 0});
    assert!(serde_json::from_value::<DisplayTessellationProfile>(invalid_profile).is_err());
}

#[test]
fn display_scene_manifest_rejects_chunk_and_total_tampering_or_limits() {
    let mut invalid_chunk = serde_json::to_value(display_chunk(0)).expect("chunk must serialize");
    invalid_chunk["vertex_byte_length"] = serde_json::json!(95);
    assert!(serde_json::from_value::<DisplayMeshChunkDescriptor>(invalid_chunk).is_err());

    let mut invalid_sequence = serde_json::to_value(display_scene()).expect("scene must serialize");
    invalid_sequence["chunks"][0]["sequence"] = serde_json::json!(1);
    assert!(serde_json::from_value::<GeometryDisplaySceneManifest>(invalid_sequence).is_err());

    let mut invalid_total = serde_json::to_value(display_scene()).expect("scene must serialize");
    invalid_total["total_triangle_count"] = serde_json::json!(13);
    assert!(serde_json::from_value::<GeometryDisplaySceneManifest>(invalid_total).is_err());

    assert!(
        DisplayMeshChunkDescriptor::new(
            0,
            DisplayGeometryReference::new("body-0").expect("reference must be valid"),
            u32::try_from(MAX_DISPLAY_SCENE_VERTICES + 1).expect("test ceiling must fit u32"),
            1,
            digest_for('c'),
            digest_for('d'),
        )
        .and_then(|chunk| GeometryDisplaySceneManifest::new(
            digest(),
            digest_for('b'),
            RepresentationBasis::Mesh,
            DisplayCoordinateSpace::CanonicalMillimeters,
            ModelLengthUnit::Millimeter,
            display_profile(),
            ["10", "20", "30"].map(|value| {
                ProvisionalGeometryDecimal::new(value).expect("extent must be valid")
            }),
            vec![chunk],
        ))
        .is_err()
    );

    let too_many_chunks = (0..=MAX_DISPLAY_SCENE_CHUNKS)
        .map(|sequence| display_chunk(u32::try_from(sequence).expect("sequence must fit u32")))
        .collect();
    assert!(
        GeometryDisplaySceneManifest::new(
            digest(),
            digest_for('b'),
            RepresentationBasis::Mesh,
            DisplayCoordinateSpace::CanonicalMillimeters,
            ModelLengthUnit::Millimeter,
            display_profile(),
            ["10", "20", "30"].map(|value| {
                ProvisionalGeometryDecimal::new(value).expect("extent must be valid")
            }),
            too_many_chunks,
        )
        .is_err()
    );

    assert!(
        DisplayMeshChunkDescriptor::new(
            0,
            DisplayGeometryReference::new("body-0").expect("reference must be valid"),
            1,
            u32::try_from(MAX_DISPLAY_SCENE_TRIANGLES + 1).expect("test ceiling must fit u32"),
            digest_for('c'),
            digest_for('d'),
        )
        .and_then(|chunk| GeometryDisplaySceneManifest::new(
            digest(),
            digest_for('b'),
            RepresentationBasis::Mesh,
            DisplayCoordinateSpace::CanonicalMillimeters,
            ModelLengthUnit::Millimeter,
            display_profile(),
            ["10", "20", "30"].map(|value| {
                ProvisionalGeometryDecimal::new(value).expect("extent must be valid")
            }),
            vec![chunk],
        ))
        .is_err()
    );

    let bytes_over_limit_but_counts_within_limits = DisplayMeshChunkDescriptor::new(
        0,
        DisplayGeometryReference::new("body-0").expect("reference must be valid"),
        900_000,
        2_000_000,
        digest_for('c'),
        digest_for('d'),
    )
    .expect("individual descriptor must be valid");
    assert!(
        bytes_over_limit_but_counts_within_limits.vertex_byte_length()
            + bytes_over_limit_but_counts_within_limits.index_byte_length()
            > MAX_DISPLAY_SCENE_BYTES
    );
    assert!(
        GeometryDisplaySceneManifest::new(
            digest(),
            digest_for('b'),
            RepresentationBasis::Mesh,
            DisplayCoordinateSpace::CanonicalMillimeters,
            ModelLengthUnit::Millimeter,
            display_profile(),
            ["10", "20", "30"].map(|value| {
                ProvisionalGeometryDecimal::new(value).expect("extent must be valid")
            }),
            vec![bytes_over_limit_but_counts_within_limits],
        )
        .is_err()
    );
}
