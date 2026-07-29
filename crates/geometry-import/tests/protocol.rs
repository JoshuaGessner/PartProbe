use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use partprobe_domain::{AssetRootId, RuleVersion, SchemaVersion};
use partprobe_geometry_core::{
    AnalysisProfile, AnalysisProfileId, GeometryStage, Sha256Digest, StageStatus,
};
use partprobe_geometry_import::{
    AssetCapability, CorrelationId, GeometryJobId, GeometryWorkerControlMessage,
    GeometryWorkerRequest, GeometryWorkerSupervisor, LocalAssetRoot, ResourceQuotas,
    SupervisorPolicy, WORKER_CONTROL_SCHEMA_VERSION, WorkerCancellationReason, WorkerTermination,
    open_local_source_read_only, recoverable_termination_response,
};

fn request() -> GeometryWorkerRequest {
    GeometryWorkerRequest::new(
        SchemaVersion::new(1).expect("schema version must be valid"),
        GeometryJobId::new("job-1").expect("job ID must be valid"),
        CorrelationId::new("correlation-1").expect("correlation ID must be valid"),
        AssetCapability::new("capability-opaque-1").expect("capability must be valid"),
        Sha256Digest::new("b".repeat(64)).expect("digest must be valid"),
        vec![
            GeometryStage::Intake,
            GeometryStage::Identify,
            GeometryStage::Parse,
        ],
        AnalysisProfile {
            id: AnalysisProfileId::new("step-basic").expect("profile ID must be valid"),
            version: RuleVersion::new(1, 0, 0),
        },
        ResourceQuotas::new(1_000_000, 2_000_000, 100_000, 30_000).expect("quotas must be valid"),
    )
    .expect("request must be valid")
}

#[test]
fn request_is_path_free_and_versioned() {
    let value = serde_json::to_value(request()).expect("request must serialize");

    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["asset_capability"], "capability-opaque-1");
    assert!(value.get("path").is_none());
    assert!(value.get("filename").is_none());
    assert!(AssetCapability::new("/tmp/model.step").is_err());
}

#[test]
fn control_frames_are_versioned_path_free_and_identity_bound() {
    let active_request = request();
    let execute = GeometryWorkerControlMessage::execute(active_request.clone());
    let value = serde_json::to_value(&execute).expect("execute frame must serialize");

    assert_eq!(
        execute.control_schema_version(),
        WORKER_CONTROL_SCHEMA_VERSION
    );
    assert_eq!(value["message"], "execute");
    assert!(value.get("path").is_none());
    let decoded: GeometryWorkerControlMessage =
        serde_json::from_value(value.clone()).expect("execute frame must deserialize");
    assert_eq!(
        decoded
            .into_execute_request()
            .expect("execute frame must contain a request"),
        active_request
    );

    let mut unsupported = value;
    unsupported["control_schema_version"] = serde_json::json!(2);
    assert!(serde_json::from_value::<GeometryWorkerControlMessage>(unsupported).is_err());

    let cancel = GeometryWorkerControlMessage::cancel(
        &active_request,
        WorkerCancellationReason::UserRequested,
    );
    assert_eq!(
        cancel
            .cancellation_reason_for(&active_request)
            .expect("matching cancellation identity must validate"),
        WorkerCancellationReason::UserRequested
    );
    let mut other_value = serde_json::to_value(request()).expect("request must serialize");
    other_value["job_id"] = serde_json::json!("other-job");
    let other: GeometryWorkerRequest =
        serde_json::from_value(other_value).expect("other request must deserialize");
    assert!(cancel.cancellation_reason_for(&other).is_err());
}

#[test]
fn supervisor_policy_requires_an_explicit_nonzero_cancellation_grace() {
    assert!(SupervisorPolicy::new(4_096, 1, 50).is_ok());
    assert!(SupervisorPolicy::new(4_096, 1, 0).is_err());
}

#[test]
fn stages_must_be_unique_and_canonically_ordered() {
    let mut duplicate = serde_json::to_value(request()).expect("request must serialize");
    duplicate["stages"] = serde_json::json!(["intake", "intake"]);
    let mut reversed = serde_json::to_value(request()).expect("request must serialize");
    reversed["stages"] = serde_json::json!(["parse", "identify"]);

    assert!(serde_json::from_value::<GeometryWorkerRequest>(duplicate).is_err());
    assert!(serde_json::from_value::<GeometryWorkerRequest>(reversed).is_err());
}

#[test]
fn zero_resource_quota_is_rejected_during_deserialization() {
    let mut value = serde_json::to_value(request()).expect("request must serialize");
    value["quotas"]["wall_time_millis"] = serde_json::json!(0);

    assert!(serde_json::from_value::<GeometryWorkerRequest>(value).is_err());
}

#[test]
fn response_deserialization_rejects_success_without_snapshot() {
    let value = serde_json::json!({
        "schema_version": 1,
        "job_id": "job-1",
        "correlation_id": "correlation-1",
        "status": "succeeded",
        "stage_reports": [],
        "snapshot_reference": null,
        "diagnostic_codes": []
    });

    assert!(
        serde_json::from_value::<partprobe_geometry_import::GeometryWorkerResponse>(value).is_err()
    );
}

#[test]
fn supervisor_failures_become_sanitized_recoverable_results() {
    for (termination, expected_code) in [
        (WorkerTermination::NonzeroExit, "WORKER_EXIT"),
        (WorkerTermination::Timeout, "WORKER_TIMEOUT"),
        (WorkerTermination::QuotaExceeded, "WORKER_QUOTA_EXCEEDED"),
        (
            WorkerTermination::MalformedResponse,
            "WORKER_MALFORMED_RESPONSE",
        ),
        (WorkerTermination::Cancelled, "WORKER_CANCELLED"),
        (
            WorkerTermination::CancellationGraceExceeded,
            "WORKER_CANCEL_FORCE_TERMINATED",
        ),
        (
            WorkerTermination::TimeoutGraceExceeded,
            "WORKER_TIMEOUT_FORCE_TERMINATED",
        ),
    ] {
        let response = recoverable_termination_response(
            SchemaVersion::new(1).expect("schema version must be valid"),
            GeometryJobId::new("job-1").expect("job ID must be valid"),
            CorrelationId::new("correlation-1").expect("correlation ID must be valid"),
            termination,
        );

        assert_eq!(response.status(), StageStatus::FailedRecoverable);
        assert!(response.snapshot_reference().is_none());
        assert_eq!(response.diagnostic_codes()[0].as_str(), expected_code);
    }
}

#[test]
fn supervisor_maps_launch_failure_and_precancel_without_path_leakage() {
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from("__partprobe_worker_does_not_exist__"),
        std::env::temp_dir(),
        SupervisorPolicy::new(4_096, 1, 50).expect("policy must be valid"),
    )
    .expect("supervisor must be valid");

    let launch = supervisor.execute(&request(), &AtomicBool::new(false));
    let cancelled = supervisor.execute(&request(), &AtomicBool::new(true));

    assert_eq!(
        launch.diagnostic_codes()[0].as_str(),
        "WORKER_LAUNCH_FAILED"
    );
    assert_eq!(cancelled.diagnostic_codes()[0].as_str(), "WORKER_CANCELLED");
}

#[test]
fn local_source_opener_accepts_a_regular_file_and_rejects_a_final_link() {
    let directory = std::env::temp_dir().join(format!(
        "partprobe-source-opener-test-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).expect("test directory must be created");
    let source = directory.join("source.asset");
    let link = directory.join("source-link.asset");
    std::fs::write(&source, b"authorized-source").expect("source must be written");
    create_file_symlink(&source, &link);
    let capability = || AssetCapability::new("source-opener-capability").expect("valid capability");

    let grant = open_local_source_read_only(capability(), &source)
        .expect("regular source must open read-only");
    assert_eq!(
        grant.asset_capability().as_str(),
        "source-opener-capability"
    );
    assert_eq!(grant.authorized_byte_length(), 17);
    assert!(open_local_source_read_only(capability(), &link).is_err());

    std::fs::remove_file(link).expect("link must be removable");
    std::fs::remove_file(source).expect("source must be removable");
    std::fs::remove_dir(directory).expect("test directory must be removable");
}

#[test]
fn local_asset_root_contains_parent_resolution_and_rejects_final_links() {
    let test_directory = std::env::temp_dir().join(format!(
        "partprobe-contained-root-test-{}",
        std::process::id()
    ));
    let root = test_directory.join("root");
    let nested = root.join("nested");
    let outside = test_directory.join("outside.asset");
    let outside_link = root.join("outside-link");
    let final_link = nested.join("source-link.asset");
    std::fs::create_dir_all(&nested).expect("nested test directory must be created");
    std::fs::write(nested.join("source.asset"), b"contained-source")
        .expect("contained source must be written");
    std::fs::write(&outside, b"outside-source").expect("outside source must be written");
    create_directory_symlink(&test_directory, &outside_link);
    create_file_symlink(&outside, &final_link);
    let root_capability = LocalAssetRoot::open(
        AssetRootId::new("test-root").expect("root ID must be valid"),
        &root,
    )
    .expect("asset root must open");
    let capability = || AssetCapability::new("contained-capability").expect("valid capability");

    let grant = root_capability
        .grant_read(capability(), std::path::Path::new("nested/source.asset"))
        .expect("contained regular source must open");
    assert_eq!(grant.authorized_byte_length(), 16);
    assert!(
        root_capability
            .grant_read(capability(), std::path::Path::new("../outside.asset"))
            .is_err()
    );
    assert!(root_capability.grant_read(capability(), &outside).is_err());
    assert!(
        root_capability
            .grant_read(
                capability(),
                std::path::Path::new("outside-link/outside.asset")
            )
            .is_err()
    );
    assert!(
        root_capability
            .grant_read(
                capability(),
                std::path::Path::new("nested/source-link.asset")
            )
            .is_err()
    );
    drop(grant);
    drop(root_capability);

    std::fs::remove_file(final_link).expect("final link must be removable");
    remove_directory_symlink(&outside_link);
    std::fs::remove_file(nested.join("source.asset")).expect("source must be removable");
    std::fs::remove_dir(nested).expect("nested directory must be removable");
    std::fs::remove_dir(root).expect("root directory must be removable");
    std::fs::remove_file(outside).expect("outside source must be removable");
    std::fs::remove_dir(test_directory).expect("test directory must be removable");
}

#[cfg(unix)]
fn create_file_symlink(source: &std::path::Path, link: &std::path::Path) {
    std::os::unix::fs::symlink(source, link).expect("file symlink must be created");
}

#[cfg(windows)]
fn create_file_symlink(source: &std::path::Path, link: &std::path::Path) {
    std::os::windows::fs::symlink_file(source, link).expect("file symlink must be created");
}

#[cfg(unix)]
fn create_directory_symlink(source: &std::path::Path, link: &std::path::Path) {
    std::os::unix::fs::symlink(source, link).expect("directory symlink must be created");
}

#[cfg(windows)]
fn create_directory_symlink(source: &std::path::Path, link: &std::path::Path) {
    std::os::windows::fs::symlink_dir(source, link).expect("directory symlink must be created");
}

#[cfg(unix)]
fn remove_directory_symlink(link: &std::path::Path) {
    std::fs::remove_file(link).expect("directory symlink must be removable");
}

#[cfg(windows)]
fn remove_directory_symlink(link: &std::path::Path) {
    std::fs::remove_dir(link).expect("directory symlink must be removable");
}
