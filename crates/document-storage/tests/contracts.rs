use partprobe_document_storage::{
    ArtifactMediaType, ArtifactSchemaId, ArtifactSchemaRef, ControlledDerivativeWrite,
    DerivativeGovernance, DerivativeIdentity, DerivativeIntegrityState, DerivativeReference,
    DocumentLocator, ImmutableBlob, RetentionPolicyId, RetentionPolicyRef, StoredDerivative,
};
use partprobe_domain::{
    ActorId, DataClassificationId, RecordId, RecordVersionId, RecordedAt, SchemaVersion,
};
use partprobe_security::{
    AuditCorrelationId, SecurityPolicyId, SecurityPolicyRef, SecurityPolicyVersion,
};

const BYTES: &[u8] = b"controlled-output";
const HASH: &str = "c2aa13ac2ee8062adde83a6469537d6f71c686d669f91cb1ab8e07712fdb2797";

fn write() -> ControlledDerivativeWrite {
    let blob = ImmutableBlob::from_claimed_sha256(HASH, 17, Box::from(BYTES))
        .expect("claimed blob must be valid");
    ControlledDerivativeWrite::new(
        DerivativeIdentity::new(
            RecordId::new("artifact-1").expect("artifact ID must be valid"),
            RecordVersionId::new("artifact-v1").expect("artifact version must be valid"),
            RecordId::new("source-1").expect("source ID must be valid"),
            RecordVersionId::new("source-v3").expect("source version must be valid"),
            DerivativeReference::new("snapshot-1").expect("reference must be valid"),
        ),
        DerivativeGovernance::new(
            DataClassificationId::new("organization-defined")
                .expect("classification must be valid"),
            SecurityPolicyRef::new(
                SecurityPolicyId::new("asset-read").expect("policy ID must be valid"),
                SecurityPolicyVersion::new(2).expect("policy version must be valid"),
            ),
            RetentionPolicyRef::new(
                RetentionPolicyId::new("quote-evidence").expect("retention ID must be valid"),
                4,
            )
            .expect("retention policy must be valid"),
            AuditCorrelationId::new("authorization-1").expect("correlation ID must be valid"),
            ActorId::new("actor-1").expect("actor ID must be valid"),
            RecordedAt::new("2026-07-29T16:00:00-05:00").expect("timestamp must be valid"),
        ),
        ArtifactSchemaRef::new(
            ArtifactSchemaId::new("partprobe.geometry-snapshot").expect("schema ID must be valid"),
            SchemaVersion::new(1).expect("schema version must be valid"),
        ),
        ArtifactMediaType::new("application/vnd.partprobe.geometry-snapshot+json")
            .expect("media type must be valid"),
        blob,
    )
}

#[test]
fn immutable_blob_revalidates_claimed_hash_and_length() {
    assert!(ImmutableBlob::from_claimed_sha256(HASH, 17, Box::from(BYTES)).is_ok());
    assert!(ImmutableBlob::from_claimed_sha256(HASH, 16, Box::from(BYTES)).is_err());
    assert!(ImmutableBlob::from_claimed_sha256(&"0".repeat(64), 17, Box::from(BYTES)).is_err());
    assert!(ImmutableBlob::from_claimed_sha256(HASH, 0, Box::from(&b""[..])).is_err());
}

#[test]
fn media_type_requires_a_canonical_type_and_subtype() {
    assert!(ArtifactMediaType::new("application/json").is_ok());
    assert!(ArtifactMediaType::new("application").is_err());
    assert!(ArtifactMediaType::new("Application/JSON").is_err());
    assert!(ArtifactMediaType::new("application/json; charset=utf-8").is_err());
}

#[test]
fn manifest_preserves_lineage_classification_policies_and_schema() {
    let write = write();
    let manifest = write.manifest();

    assert_eq!(manifest.identity().artifact_id().as_str(), "artifact-1");
    assert_eq!(manifest.identity().source_record_id().as_str(), "source-1");
    assert_eq!(
        manifest.identity().derivative_reference().as_str(),
        "snapshot-1"
    );
    assert_eq!(
        manifest.governance().classification_id().as_str(),
        "organization-defined"
    );
    assert_eq!(manifest.governance().access_policy().version().value(), 2);
    assert_eq!(manifest.governance().retention_policy().version(), 4);
    assert_eq!(
        manifest
            .governance()
            .authorization_correlation_id()
            .as_str(),
        "authorization-1"
    );
    assert_eq!(manifest.schema().version().value(), 1);
    assert_eq!(
        manifest.media_type().as_str(),
        "application/vnd.partprobe.geometry-snapshot+json"
    );
    assert_eq!(manifest.content_address().sha256(), HASH);
    assert_eq!(manifest.byte_length(), 17);
    assert_eq!(
        manifest.integrity_state(),
        DerivativeIntegrityState::Verified
    );
}

#[test]
fn storage_receipt_must_match_the_requested_manifest() {
    let write = write();
    let receipt = StoredDerivative::new(
        write.manifest().clone(),
        DocumentLocator::new("local:sha256:54e0132a").expect("locator must be valid"),
    );

    assert!(receipt.matches(&write));
}
