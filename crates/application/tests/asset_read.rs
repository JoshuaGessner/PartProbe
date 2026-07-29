use std::cell::{Cell, RefCell};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use partprobe_application::{AssetReadServiceError, AssetReadSubject, LocalAssetReadService};
use partprobe_domain::{
    ActorId, AssetRootId, DataClassificationId, ProjectId, RecordId, RecordStateId,
    RecordVersionId, RecordedAt,
};
use partprobe_geometry_import::{AssetCapability, LocalAssetRoot};
use partprobe_security::{
    AuditAppendError, AuditCorrelationId, AuthorizationAuditEvent, AuthorizationAuditSink,
    AuthorizationDecision, AuthorizationOutcome, AuthorizationPolicy, AuthorizationReasonCode,
    DenyAllAuthorizationPolicy, SecurityPolicyId, SecurityPolicyRef, SecurityPolicyVersion,
};

static TEST_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(1);

fn policy_ref() -> SecurityPolicyRef {
    SecurityPolicyRef::new(
        SecurityPolicyId::new("test-policy").expect("policy ID must be valid"),
        SecurityPolicyVersion::new(1).expect("policy version must be valid"),
    )
}

fn subject() -> AssetReadSubject {
    AssetReadSubject::new(
        ActorId::new("actor-1").expect("actor ID must be valid"),
        ProjectId::new("project-1").expect("project ID must be valid"),
        RecordId::new("asset-1").expect("record ID must be valid"),
        RecordVersionId::new("revision-1").expect("record version must be valid"),
        DataClassificationId::new("organization-defined").expect("classification ID must be valid"),
        RecordStateId::new("draft").expect("record state ID must be valid"),
        AuditCorrelationId::new("correlation-1").expect("correlation ID must be valid"),
        RecordedAt::new("2026-07-29T15:00:00-05:00").expect("timestamp must be valid"),
    )
}

fn capability() -> AssetCapability {
    AssetCapability::new("asset-capability-1").expect("asset capability must be valid")
}

#[derive(Debug)]
struct AllowPolicy;

impl AuthorizationPolicy for AllowPolicy {
    fn evaluate(
        &self,
        _context: &partprobe_security::AuthorizationContext,
    ) -> AuthorizationDecision {
        AuthorizationDecision::allow(
            policy_ref(),
            AuthorizationReasonCode::new("ASSIGNED_SCOPE").expect("reason must be valid"),
        )
    }
}

#[derive(Debug, Default)]
struct RecordingAudit {
    events: RefCell<Vec<AuthorizationAuditEvent>>,
    fail: Cell<bool>,
}

impl AuthorizationAuditSink for RecordingAudit {
    fn append(&self, event: AuthorizationAuditEvent) -> Result<(), AuditAppendError> {
        if self.fail.get() {
            return Err(AuditAppendError);
        }
        self.events.borrow_mut().push(event);
        Ok(())
    }
}

#[test]
fn deny_all_records_the_decision_and_never_opens_the_asset() {
    let test_directory = create_test_root("deny");
    let root = LocalAssetRoot::open(
        AssetRootId::new("root-deny").expect("root ID must be valid"),
        &test_directory,
    )
    .expect("asset root must open");
    let policy = DenyAllAuthorizationPolicy::new(
        policy_ref(),
        AuthorizationReasonCode::new("POLICY_NOT_CONFIGURED").expect("reason must be valid"),
    );
    let service = LocalAssetReadService::new(policy, RecordingAudit::default());

    let result =
        service.authorize_and_open(subject(), &root, capability(), Path::new("source.asset"));

    assert_eq!(
        result.expect_err("deny-all policy must deny"),
        AssetReadServiceError::Denied(
            AuthorizationReasonCode::new("POLICY_NOT_CONFIGURED").expect("reason must be valid")
        )
    );
    let events = service_audit_events(&service);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].decision().outcome(), AuthorizationOutcome::Denied);
    assert_eq!(events[0].context().asset_root_id().as_str(), "root-deny");
    assert!(!format!("{:?}", events[0]).contains("source.asset"));
    drop(events);
    drop(root);
    remove_test_root(&test_directory);
}

#[test]
fn allowed_read_is_audited_before_a_pathless_grant_is_returned() {
    let test_directory = create_test_root("allow");
    let root = LocalAssetRoot::open(
        AssetRootId::new("root-allow").expect("root ID must be valid"),
        &test_directory,
    )
    .expect("asset root must open");
    let service = LocalAssetReadService::new(AllowPolicy, RecordingAudit::default());

    let grant = service
        .authorize_and_open(subject(), &root, capability(), Path::new("source.asset"))
        .expect("allowed contained asset must return a grant");

    assert_eq!(grant.authorized_byte_length(), 16);
    let events = service_audit_events(&service);
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].decision().outcome(),
        AuthorizationOutcome::Allowed
    );
    drop(events);
    drop(grant);
    drop(root);
    remove_test_root(&test_directory);
}

#[test]
fn audit_failure_fails_closed_before_filesystem_resolution() {
    let test_directory = create_test_root("audit-failure");
    let root = LocalAssetRoot::open(
        AssetRootId::new("root-audit-failure").expect("root ID must be valid"),
        &test_directory,
    )
    .expect("asset root must open");
    let audit = RecordingAudit::default();
    audit.fail.set(true);
    let service = LocalAssetReadService::new(AllowPolicy, audit);

    let result =
        service.authorize_and_open(subject(), &root, capability(), Path::new("missing.asset"));

    assert_eq!(
        result.expect_err("audit failure must stop the read"),
        AssetReadServiceError::AuditUnavailable
    );
    assert!(service_audit_events(&service).is_empty());
    drop(root);
    remove_test_root(&test_directory);
}

fn create_test_root(label: &str) -> std::path::PathBuf {
    let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "partprobe-asset-read-{label}-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("test directory must be created");
    std::fs::write(directory.join("source.asset"), b"contained-source")
        .expect("test asset must be written");
    directory
}

fn remove_test_root(directory: &Path) {
    std::fs::remove_file(directory.join("source.asset")).expect("test asset must be removable");
    std::fs::remove_dir(directory).expect("test directory must be removable");
}

fn service_audit_events<P>(
    service: &LocalAssetReadService<P, RecordingAudit>,
) -> std::cell::Ref<'_, Vec<AuthorizationAuditEvent>>
where
    P: AuthorizationPolicy,
{
    service.audit().events.borrow()
}
