use partprobe_domain::RuleVersion;
use partprobe_geometry_core::{
    AnalysisProfile, AnalysisProfileId, GeometryStage, GeometryStageReport, GeometryWarning,
    GeometryWarningCode, ModelAssetDescriptor, ModelFormat, Sha256Digest, StageStatus,
    WarningSeverity,
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
