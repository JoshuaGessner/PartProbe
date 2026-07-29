use partprobe_domain::{
    ActorId, AssetRootId, DataClassificationId, ProjectId, RecordId, RecordStateId,
    RecordVersionId, RecordedAt,
};
use partprobe_security::{
    AuditCorrelationId, AuthorizationContext, AuthorizationOutcome, AuthorizationPolicy,
    AuthorizationReasonCode, DenyAllAuthorizationPolicy, ProtectedOperation, SecurityPolicyId,
    SecurityPolicyRef, SecurityPolicyVersion,
};

fn policy_ref() -> SecurityPolicyRef {
    SecurityPolicyRef::new(
        SecurityPolicyId::new("local-deny-all").expect("policy ID must be valid"),
        SecurityPolicyVersion::new(1).expect("policy version must be valid"),
    )
}

fn context() -> AuthorizationContext {
    AuthorizationContext::new(
        ActorId::new("actor-1").expect("actor ID must be valid"),
        ProjectId::new("project-1").expect("project ID must be valid"),
        RecordId::new("asset-1").expect("record ID must be valid"),
        RecordVersionId::new("revision-1").expect("record version must be valid"),
        DataClassificationId::new("organization-defined").expect("classification ID must be valid"),
        RecordStateId::new("draft").expect("record state ID must be valid"),
        AssetRootId::new("root-1").expect("asset root ID must be valid"),
        ProtectedOperation::ReadGeometryAsset,
        AuditCorrelationId::new("correlation-1").expect("correlation ID must be valid"),
        RecordedAt::new("2026-07-29T15:00:00-05:00").expect("timestamp must be valid"),
    )
}

#[test]
fn deny_all_policy_is_versioned_and_content_free() {
    let policy = DenyAllAuthorizationPolicy::new(
        policy_ref(),
        AuthorizationReasonCode::new("POLICY_NOT_CONFIGURED").expect("reason must be valid"),
    );

    let decision = policy.evaluate(&context());

    assert_eq!(decision.outcome(), AuthorizationOutcome::Denied);
    assert_eq!(decision.policy().id().as_str(), "local-deny-all");
    assert_eq!(decision.policy().version().value(), 1);
    assert_eq!(decision.reason_code().as_str(), "POLICY_NOT_CONFIGURED");
}

#[test]
fn security_tokens_and_policy_versions_are_bounded() {
    assert!(SecurityPolicyVersion::new(0).is_err());
    assert!(SecurityPolicyId::new("").is_err());
    assert!(AuthorizationReasonCode::new("contains spaces").is_err());
    assert!(AuditCorrelationId::new("/tmp/model.step").is_err());
}
