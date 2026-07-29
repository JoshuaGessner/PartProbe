//! Headless application services that coordinate policy, audit, and pure engines.

use std::path::Path;

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
