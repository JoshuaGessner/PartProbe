//! Headless application services that coordinate policy, audit, and pure engines.

mod draft_estimate;

pub use draft_estimate::{
    AnalyzedGeometryEvidence, DraftBaseCostInputs, DraftEstimateApplication,
    DraftEstimateApplicationError, DraftEstimateInputs, DraftEstimateResult, DraftEstimateSession,
    DraftEstimateTrace, DraftGeometryReview, DraftMaterialCostInputs, DraftOperationCostInputs,
    DraftQuantityInputs, DraftRateContext, DraftResolvedRates, DraftStockInputs, DraftTimeInputs,
    GeometryAnalysisFailure, GeometryAnalysisPort,
};

use std::path::Path;

use partprobe_document_storage::{
    ArtifactMediaType, ArtifactSchemaRef, ControlledDerivativeStore, ControlledDerivativeWrite,
    DerivativeGovernance, DerivativeIdentity, DerivativeReference, DerivativeStoreError,
    ImmutableBlob, StoredDerivative,
};
use partprobe_domain::{
    ActorId, DataClassificationId, ProjectId, RecordId, RecordStateId, RecordVersionId, RecordedAt,
};
use partprobe_geometry_import::{AssetCapability, AssetReadGrant, LocalAssetRoot};
use partprobe_security::{
    AuditCorrelationId, AuthorizationAuditEvent, AuthorizationAuditSink, AuthorizationContext,
    AuthorizationOutcome, AuthorizationPolicy, AuthorizationReasonCode, ProtectedOperation,
};

/// Identity and governed record facts supplied by a trusted application boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetReadSubject {
    actor_id: ActorId,
    project_id: ProjectId,
    record_id: RecordId,
    record_version_id: RecordVersionId,
    classification_id: DataClassificationId,
    record_state_id: RecordStateId,
    correlation_id: AuditCorrelationId,
    recorded_at: RecordedAt,
}

impl AssetReadSubject {
    /// Creates explicit authorization facts without a filesystem path or asset content.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        actor_id: ActorId,
        project_id: ProjectId,
        record_id: RecordId,
        record_version_id: RecordVersionId,
        classification_id: DataClassificationId,
        record_state_id: RecordStateId,
        correlation_id: AuditCorrelationId,
        recorded_at: RecordedAt,
    ) -> Self {
        Self {
            actor_id,
            project_id,
            record_id,
            record_version_id,
            classification_id,
            record_state_id,
            correlation_id,
            recorded_at,
        }
    }
}

/// Deny-by-default service that records a policy decision before opening a local asset.
#[derive(Debug)]
pub struct LocalAssetReadService<P, A> {
    policy: P,
    audit: A,
}

impl<P, A> LocalAssetReadService<P, A>
where
    P: AuthorizationPolicy,
    A: AuthorizationAuditSink,
{
    /// Creates a service from explicit policy and append-only audit ports.
    #[must_use]
    pub const fn new(policy: P, audit: A) -> Self {
        Self { policy, audit }
    }

    /// Returns the configured audit adapter for composition and verification.
    #[must_use]
    pub const fn audit(&self) -> &A {
        &self.audit
    }

    /// Authorizes, audits, and resolves one relative local asset into a pathless read grant.
    pub fn authorize_and_open(
        &self,
        subject: AssetReadSubject,
        root: &LocalAssetRoot,
        asset_capability: AssetCapability,
        relative_path: &Path,
    ) -> Result<AssetReadGrant, AssetReadServiceError> {
        let context = AuthorizationContext::new(
            subject.actor_id,
            subject.project_id,
            subject.record_id,
            subject.record_version_id,
            subject.classification_id,
            subject.record_state_id,
            root.asset_root_id().clone(),
            ProtectedOperation::ReadGeometryAsset,
            subject.correlation_id,
            subject.recorded_at,
        );
        let decision = self.policy.evaluate(&context);
        self.audit
            .append(AuthorizationAuditEvent::new(context, decision.clone()))
            .map_err(|_| AssetReadServiceError::AuditUnavailable)?;
        if decision.outcome() == AuthorizationOutcome::Denied {
            return Err(AssetReadServiceError::Denied(
                decision.reason_code().clone(),
            ));
        }
        root.grant_read(asset_capability, relative_path)
            .map_err(|_| AssetReadServiceError::ContainmentRejected)
    }
}

/// Content-free application errors for the protected local-asset read use case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssetReadServiceError {
    /// Policy denied access with a stable reason code.
    Denied(AuthorizationReasonCode),
    /// Required append-only audit evidence could not be recorded.
    AuditUnavailable,
    /// The relative filesystem lookup failed containment or file validation.
    ContainmentRejected,
}

/// Complete application-owned metadata required before persisting worker output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeometryDerivativeMetadata {
    artifact_id: RecordId,
    artifact_version_id: RecordVersionId,
    source_record_id: RecordId,
    source_record_version_id: RecordVersionId,
    classification_id: DataClassificationId,
    access_policy: partprobe_security::SecurityPolicyRef,
    retention_policy: partprobe_document_storage::RetentionPolicyRef,
    authorization_correlation_id: partprobe_security::AuditCorrelationId,
    schema: ArtifactSchemaRef,
    media_type: ArtifactMediaType,
    created_by: ActorId,
    created_at: RecordedAt,
}

impl GeometryDerivativeMetadata {
    /// Creates explicit lineage, classification, policy, schema, and creation evidence.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        artifact_id: RecordId,
        artifact_version_id: RecordVersionId,
        source_record_id: RecordId,
        source_record_version_id: RecordVersionId,
        classification_id: DataClassificationId,
        access_policy: partprobe_security::SecurityPolicyRef,
        retention_policy: partprobe_document_storage::RetentionPolicyRef,
        authorization_correlation_id: partprobe_security::AuditCorrelationId,
        schema: ArtifactSchemaRef,
        media_type: ArtifactMediaType,
        created_by: ActorId,
        created_at: RecordedAt,
    ) -> Self {
        Self {
            artifact_id,
            artifact_version_id,
            source_record_id,
            source_record_version_id,
            classification_id,
            access_policy,
            retention_policy,
            authorization_correlation_id,
            schema,
            media_type,
            created_by,
            created_at,
        }
    }
}

/// Application service that consumes controlled worker output only after governed persistence.
#[derive(Debug)]
pub struct GeometryDerivativePersistenceService<S> {
    store: S,
}

impl<S> GeometryDerivativePersistenceService<S>
where
    S: ControlledDerivativeStore,
{
    /// Creates a persistence service over a deployment-provided controlled store.
    #[must_use]
    pub const fn new(store: S) -> Self {
        Self { store }
    }

    /// Revalidates, governs, and persists one claimed worker output.
    pub fn persist(
        &self,
        output: partprobe_geometry_import::ControlledWorkerOutput,
        metadata: GeometryDerivativeMetadata,
    ) -> Result<StoredDerivative, GeometryDerivativePersistenceFailure> {
        let blob = match ImmutableBlob::from_claimed_sha256(
            output.content_hash().as_str(),
            output.byte_length(),
            Box::from(output.bytes()),
        ) {
            Ok(blob) => blob,
            Err(_) => {
                return Err(GeometryDerivativePersistenceFailure::OutputIntegrity(
                    output,
                ));
            }
        };
        let derivative_reference =
            match DerivativeReference::new(output.snapshot_reference().as_str()) {
                Ok(reference) => reference,
                Err(_) => {
                    return Err(GeometryDerivativePersistenceFailure::OutputIntegrity(
                        output,
                    ));
                }
            };
        let identity = DerivativeIdentity::new(
            metadata.artifact_id,
            metadata.artifact_version_id,
            metadata.source_record_id,
            metadata.source_record_version_id,
            derivative_reference,
        );
        let write = ControlledDerivativeWrite::new(
            identity,
            DerivativeGovernance::new(
                metadata.classification_id,
                metadata.access_policy,
                metadata.retention_policy,
                metadata.authorization_correlation_id,
                metadata.created_by,
                metadata.created_at,
            ),
            metadata.schema,
            metadata.media_type,
            blob,
        );
        let stored = self.store.persist(&write).map_err(|error| {
            GeometryDerivativePersistenceFailure::Store {
                write: Box::new(write.clone()),
                error,
            }
        })?;
        if !stored.matches(&write) {
            return Err(GeometryDerivativePersistenceFailure::Store {
                write: Box::new(write),
                error: DerivativeStoreError::IntegrityConflict,
            });
        }
        Ok(stored)
    }
}

/// Failed persistence with bytes retained for explicit retry, quarantine, or disposition.
#[derive(Debug)]
pub enum GeometryDerivativePersistenceFailure {
    /// Worker output failed independent validation and remains available to the caller.
    OutputIntegrity(partprobe_geometry_import::ControlledWorkerOutput),
    /// The governed write remains available after a store or receipt failure.
    Store {
        /// Complete immutable bytes and manifest attempted by the service.
        write: Box<ControlledDerivativeWrite>,
        /// Content-free failure category.
        error: DerivativeStoreError,
    },
}
