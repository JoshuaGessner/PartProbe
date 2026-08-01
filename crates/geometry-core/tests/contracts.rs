use partprobe_domain::RuleVersion;
use partprobe_geometry_core::{
    AnalysisProfile, AnalysisProfileId, GeometryStage, GeometryStageReport, GeometryWarning,
    GeometryWarningCode, ModelAssetDescriptor, ModelFormat, ProvisionalGeometryDecimal,
    ProvisionalGeometrySnapshot, Sha256Digest, StageStatus, WarningSeverity,
};

fn digest() -> Sha256Digest {
    Sha256Digest::new("a".repeat(64)).expect("digest must be valid")
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
