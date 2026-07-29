use partprobe_domain::{RuleVersion, SchemaVersion};
use partprobe_geometry_core::{
    AnalysisProfile, AnalysisProfileId, GeometryStage, Sha256Digest, StageStatus,
};
use partprobe_geometry_import::{
    AssetCapability, CorrelationId, GeometryJobId, GeometryWorkerRequest, ResourceQuotas,
    WorkerTermination, recoverable_termination_response,
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
