//! Policy and content-minimized audit contracts for sensitive application operations.

use partprobe_domain::{
    ActorId, AssetRootId, DataClassificationId, DomainError, ProjectId, RecordId, RecordStateId,
    RecordVersionId, RecordedAt,
};

macro_rules! security_token {
    ($(#[$attribute:meta])* $name:ident, $field:literal) => {
        $(#[$attribute])*
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Validates a bounded machine-readable security token.
            pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
                let value = value.into();
                if value.is_empty()
                    || value.len() > 128
                    || !value.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':')
                    })
                {
                    return Err(DomainError::InvalidValue {
                        field: $field,
                        reason: "must be a 1-128 byte ASCII machine token",
                    });
                }
                Ok(Self(value))
            }

            /// Returns the validated token.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

security_token!(
    /// Stable identity of an authorization policy.
    SecurityPolicyId,
    "security policy ID"
);
security_token!(
    /// Content-free reason code for an authorization outcome.
    AuthorizationReasonCode,
    "authorization reason code"
);
security_token!(
    /// Correlation identity safe for a content-minimized audit event.
    AuditCorrelationId,
    "audit correlation ID"
);

/// Monotonic version of deployed authorization-policy behavior.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SecurityPolicyVersion(u64);

impl SecurityPolicyVersion {
    /// Creates a nonzero policy version.
    pub fn new(value: u64) -> Result<Self, DomainError> {
        if value == 0 {
            return Err(DomainError::InvalidValue {
                field: "security policy version",
                reason: "must be greater than zero",
            });
        }
        Ok(Self(value))
    }

    /// Returns the numeric version.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Exact deployed policy identity recorded with every decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityPolicyRef {
    id: SecurityPolicyId,
    version: SecurityPolicyVersion,
}

impl SecurityPolicyRef {
    /// Creates a policy reference.
    #[must_use]
    pub const fn new(id: SecurityPolicyId, version: SecurityPolicyVersion) -> Self {
        Self { id, version }
    }

    /// Returns the stable policy identity.
    #[must_use]
    pub const fn id(&self) -> &SecurityPolicyId {
        &self.id
    }

    /// Returns the exact policy version.
    #[must_use]
    pub const fn version(&self) -> SecurityPolicyVersion {
        self.version
    }
}

/// Sensitive operation evaluated by the authorization boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtectedOperation {
    /// Open a local CAD/model asset for isolated geometry intake.
    ReadGeometryAsset,
}

/// Policy inputs for one protected operation; contains no filesystem path or file content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationContext {
    actor_id: ActorId,
    project_id: ProjectId,
    record_id: RecordId,
    record_version_id: RecordVersionId,
    classification_id: DataClassificationId,
    record_state_id: RecordStateId,
    asset_root_id: AssetRootId,
    operation: ProtectedOperation,
    correlation_id: AuditCorrelationId,
    recorded_at: RecordedAt,
}

impl AuthorizationContext {
    /// Creates explicit policy input for one protected operation.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        actor_id: ActorId,
        project_id: ProjectId,
        record_id: RecordId,
        record_version_id: RecordVersionId,
        classification_id: DataClassificationId,
        record_state_id: RecordStateId,
        asset_root_id: AssetRootId,
        operation: ProtectedOperation,
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
            asset_root_id,
            operation,
            correlation_id,
            recorded_at,
        }
    }

    /// Returns the actor identity.
    #[must_use]
    pub const fn actor_id(&self) -> &ActorId {
        &self.actor_id
    }

    /// Returns the project identity.
    #[must_use]
    pub const fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    /// Returns the protected record identity.
    #[must_use]
    pub const fn record_id(&self) -> &RecordId {
        &self.record_id
    }

    /// Returns the immutable record version identity.
    #[must_use]
    pub const fn record_version_id(&self) -> &RecordVersionId {
        &self.record_version_id
    }

    /// Returns the deployment-defined classification.
    #[must_use]
    pub const fn classification_id(&self) -> &DataClassificationId {
        &self.classification_id
    }

    /// Returns the deployment-defined record state.
    #[must_use]
    pub const fn record_state_id(&self) -> &RecordStateId {
        &self.record_state_id
    }

    /// Returns the application-approved root identity.
    #[must_use]
    pub const fn asset_root_id(&self) -> &AssetRootId {
        &self.asset_root_id
    }

    /// Returns the protected operation.
    #[must_use]
    pub const fn operation(&self) -> ProtectedOperation {
        self.operation
    }

    /// Returns the content-minimized correlation identity.
    #[must_use]
    pub const fn correlation_id(&self) -> &AuditCorrelationId {
        &self.correlation_id
    }

    /// Returns the trusted application timestamp.
    #[must_use]
    pub const fn recorded_at(&self) -> &RecordedAt {
        &self.recorded_at
    }
}

/// Policy result kept distinct from operation success or filesystem resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationOutcome {
    /// Policy permits the requested operation.
    Allowed,
    /// Policy denies the requested operation.
    Denied,
}

/// Versioned, reason-coded authorization result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationDecision {
    outcome: AuthorizationOutcome,
    policy: SecurityPolicyRef,
    reason_code: AuthorizationReasonCode,
}

impl AuthorizationDecision {
    /// Creates an allow decision from a trusted policy implementation.
    #[must_use]
    pub const fn allow(policy: SecurityPolicyRef, reason_code: AuthorizationReasonCode) -> Self {
        Self {
            outcome: AuthorizationOutcome::Allowed,
            policy,
            reason_code,
        }
    }

    /// Creates a deny decision from a trusted policy implementation.
    #[must_use]
    pub const fn deny(policy: SecurityPolicyRef, reason_code: AuthorizationReasonCode) -> Self {
        Self {
            outcome: AuthorizationOutcome::Denied,
            policy,
            reason_code,
        }
    }

    /// Returns the allow/deny outcome.
    #[must_use]
    pub const fn outcome(&self) -> AuthorizationOutcome {
        self.outcome
    }

    /// Returns the exact policy identity and version.
    #[must_use]
    pub const fn policy(&self) -> &SecurityPolicyRef {
        &self.policy
    }

    /// Returns the content-free reason code.
    #[must_use]
    pub const fn reason_code(&self) -> &AuthorizationReasonCode {
        &self.reason_code
    }
}

/// Deployment-supplied authorization policy. Implementations must deny unless every required
/// role, project-membership, classification, record-state, and operation rule allows access.
pub trait AuthorizationPolicy {
    /// Evaluates one protected operation without performing it.
    fn evaluate(&self, context: &AuthorizationContext) -> AuthorizationDecision;
}

/// Safe baseline for deployments that have not configured an authorization policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DenyAllAuthorizationPolicy {
    policy: SecurityPolicyRef,
    reason_code: AuthorizationReasonCode,
}

impl DenyAllAuthorizationPolicy {
    /// Creates an explicit deny-all baseline with versioned evidence.
    #[must_use]
    pub const fn new(policy: SecurityPolicyRef, reason_code: AuthorizationReasonCode) -> Self {
        Self {
            policy,
            reason_code,
        }
    }
}

impl AuthorizationPolicy for DenyAllAuthorizationPolicy {
    fn evaluate(&self, _context: &AuthorizationContext) -> AuthorizationDecision {
        AuthorizationDecision::deny(self.policy.clone(), self.reason_code.clone())
    }
}

/// Content-minimized, append-preserving authorization decision event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationAuditEvent {
    context: AuthorizationContext,
    decision: AuthorizationDecision,
}

impl AuthorizationAuditEvent {
    /// Binds the exact policy decision to its content-free context.
    #[must_use]
    pub const fn new(context: AuthorizationContext, decision: AuthorizationDecision) -> Self {
        Self { context, decision }
    }

    /// Returns the audited policy context.
    #[must_use]
    pub const fn context(&self) -> &AuthorizationContext {
        &self.context
    }

    /// Returns the exact audited decision.
    #[must_use]
    pub const fn decision(&self) -> &AuthorizationDecision {
        &self.decision
    }
}

/// Append-only audit port. Persistence implementations must not update or delete prior events.
pub trait AuthorizationAuditSink {
    /// Appends one decision event. Sensitive operations fail closed when this returns an error.
    fn append(&self, event: AuthorizationAuditEvent) -> Result<(), AuditAppendError>;
}

/// Content-free failure indicating that required audit evidence could not be appended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuditAppendError;
