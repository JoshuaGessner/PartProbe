use std::cell::RefCell;
use std::rc::Rc;

use partprobe_application::{
    GeometryDerivativeMetadata, GeometryDerivativePersistenceFailure,
    GeometryDerivativePersistenceService,
};
use partprobe_document_storage::{
    ArtifactMediaType, ArtifactSchemaId, ArtifactSchemaRef, ControlledDerivativeStore,
    ControlledDerivativeWrite, DerivativeStoreError, DocumentLocator, RetentionPolicyId,
    RetentionPolicyRef, StoredDerivative,
};
use partprobe_domain::{
    ActorId, DataClassificationId, RecordId, RecordVersionId, RecordedAt, SchemaVersion,
};
use partprobe_geometry_import::{ControlledWorkerOutput, Sha256Digest, SnapshotReference};
use partprobe_security::{
    AuditCorrelationId, SecurityPolicyId, SecurityPolicyRef, SecurityPolicyVersion,
};

const BYTES: &[u8] = b"controlled-output";
const HASH: &str = "c2aa13ac2ee8062adde83a6469537d6f71c686d669f91cb1ab8e07712fdb2797";

#[derive(Clone, Copy, Debug)]
enum StoreMode {
    Success,
    Unavailable,
    MismatchedReceipt,
}

#[derive(Debug)]
struct RecordingStore {
    mode: StoreMode,
    writes: Rc<RefCell<Vec<ControlledDerivativeWrite>>>,
}

impl ControlledDerivativeStore for RecordingStore {
    fn persist(
        &self,
        write: &ControlledDerivativeWrite,
    ) -> Result<StoredDerivative, DerivativeStoreError> {
        self.writes.borrow_mut().push(write.clone());
        match self.mode {
            StoreMode::Success => Ok(StoredDerivative::new(
                write.manifest().clone(),
                DocumentLocator::new("controlled:sha256:c2aa13ac").expect("locator must be valid"),
            )),
            StoreMode::Unavailable => Err(DerivativeStoreError::Unavailable),
            StoreMode::MismatchedReceipt => Ok(StoredDerivative::new(
                alternate_write().manifest().clone(),
                DocumentLocator::new("controlled:sha256:other").expect("locator must be valid"),
            )),
        }
    }
}

fn output() -> ControlledWorkerOutput {
    ControlledWorkerOutput::from_claimed_parts(
        SnapshotReference::new("snapshot-1").expect("snapshot reference must be valid"),
        Sha256Digest::new(HASH).expect("hash must be valid"),
        Box::from(BYTES),
    )
    .expect("claimed output must be valid")
}

fn metadata(artifact_id: &str) -> GeometryDerivativeMetadata {
    GeometryDerivativeMetadata::new(
        RecordId::new(artifact_id).expect("artifact ID must be valid"),
        RecordVersionId::new("artifact-v1").expect("artifact version must be valid"),
        RecordId::new("source-1").expect("source ID must be valid"),
        RecordVersionId::new("source-v3").expect("source version must be valid"),
        DataClassificationId::new("organization-defined").expect("classification must be valid"),
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
        ArtifactSchemaRef::new(
            ArtifactSchemaId::new("partprobe.geometry-snapshot").expect("schema ID must be valid"),
            SchemaVersion::new(1).expect("schema version must be valid"),
        ),
        ArtifactMediaType::new("application/vnd.partprobe.geometry-snapshot+json")
            .expect("media type must be valid"),
        ActorId::new("actor-1").expect("actor ID must be valid"),
        RecordedAt::new("2026-07-29T16:00:00-05:00").expect("timestamp must be valid"),
    )
}

fn alternate_write() -> ControlledDerivativeWrite {
    let writes = Rc::new(RefCell::new(Vec::new()));
    let store = RecordingStore {
        mode: StoreMode::Success,
        writes,
    };
    let service = GeometryDerivativePersistenceService::new(store);
    let receipt = service
        .persist(output(), metadata("artifact-other"))
        .expect("alternate write must persist");
    let blob =
        partprobe_document_storage::ImmutableBlob::from_claimed_sha256(HASH, 17, Box::from(BYTES))
            .expect("blob must be valid");
    ControlledDerivativeWrite::new(
        receipt.manifest().identity().clone(),
        receipt.manifest().governance().clone(),
        receipt.manifest().schema().clone(),
        receipt.manifest().media_type().clone(),
        blob,
    )
}

#[test]
fn service_persists_complete_governed_lineage_after_independent_validation() {
    let writes = Rc::new(RefCell::new(Vec::new()));
    let service = GeometryDerivativePersistenceService::new(RecordingStore {
        mode: StoreMode::Success,
        writes: Rc::clone(&writes),
    });

    let stored = service
        .persist(output(), metadata("artifact-1"))
        .expect("governed output must persist");

    assert_eq!(writes.borrow().len(), 1);
    assert_eq!(
        stored.manifest().identity().artifact_id().as_str(),
        "artifact-1"
    );
    assert_eq!(
        stored
            .manifest()
            .identity()
            .source_record_version_id()
            .as_str(),
        "source-v3"
    );
    assert_eq!(
        stored.manifest().governance().classification_id().as_str(),
        "organization-defined"
    );
    assert_eq!(
        stored
            .manifest()
            .governance()
            .authorization_correlation_id()
            .as_str(),
        "authorization-1"
    );
    assert_eq!(
        stored.manifest().media_type().as_str(),
        "application/vnd.partprobe.geometry-snapshot+json"
    );
    assert_eq!(stored.manifest().content_address().sha256(), HASH);
}

#[test]
fn store_failure_returns_the_complete_governed_bytes_for_disposition() {
    let service = GeometryDerivativePersistenceService::new(RecordingStore {
        mode: StoreMode::Unavailable,
        writes: Rc::new(RefCell::new(Vec::new())),
    });

    let failure = service
        .persist(output(), metadata("artifact-1"))
        .expect_err("unavailable store must fail");

    let GeometryDerivativePersistenceFailure::Store { write, error } = failure else {
        panic!("expected store failure");
    };
    assert_eq!(error, DerivativeStoreError::Unavailable);
    assert_eq!(write.blob().bytes(), BYTES);
    assert_eq!(
        write.manifest().governance().retention_policy().version(),
        4
    );
}

#[test]
fn mismatched_store_receipt_is_rejected_without_losing_the_write() {
    let service = GeometryDerivativePersistenceService::new(RecordingStore {
        mode: StoreMode::MismatchedReceipt,
        writes: Rc::new(RefCell::new(Vec::new())),
    });

    let failure = service
        .persist(output(), metadata("artifact-1"))
        .expect_err("mismatched receipt must fail");

    let GeometryDerivativePersistenceFailure::Store { write, error } = failure else {
        panic!("expected receipt integrity failure");
    };
    assert_eq!(error, DerivativeStoreError::IntegrityConflict);
    assert_eq!(
        write.manifest().identity().artifact_id().as_str(),
        "artifact-1"
    );
    assert_eq!(write.blob().bytes(), BYTES);
}

#[test]
fn alternate_output_transport_rejects_a_forged_hash() {
    let result = ControlledWorkerOutput::from_claimed_parts(
        SnapshotReference::new("snapshot-1").expect("snapshot reference must be valid"),
        Sha256Digest::new("0".repeat(64)).expect("hash shape must be valid"),
        Box::from(BYTES),
    );

    assert!(result.is_err());
}
