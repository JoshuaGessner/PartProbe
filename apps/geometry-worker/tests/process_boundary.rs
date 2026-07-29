use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use partprobe_domain::{RuleVersion, SchemaVersion};
use partprobe_geometry_core::{
    AnalysisProfile, AnalysisProfileId, GeometryStage, Sha256Digest, StageStatus,
};
use partprobe_geometry_import::{
    AssetCapability, CorrelationId, GeometryJobId, GeometryWorkerRequest, GeometryWorkerSupervisor,
    ResourceQuotas, SupervisorPolicy,
};

fn request() -> GeometryWorkerRequest {
    GeometryWorkerRequest::new(
        SchemaVersion::new(1).expect("schema version must be valid"),
        GeometryJobId::new("process-job-1").expect("job ID must be valid"),
        CorrelationId::new("process-correlation-1").expect("correlation ID must be valid"),
        AssetCapability::new("asset-capability-1").expect("capability must be valid"),
        Sha256Digest::new("c".repeat(64)).expect("hash must be valid"),
        vec![
            GeometryStage::Intake,
            GeometryStage::Identify,
            GeometryStage::Parse,
        ],
        AnalysisProfile {
            id: AnalysisProfileId::new("step-contract").expect("profile ID must be valid"),
            version: RuleVersion::new(1, 0, 0),
        },
        ResourceQuotas::new(1_000_000, 1_000_000, 100_000, 5_000).expect("quotas must be valid"),
    )
    .expect("request must be valid")
}

#[test]
fn supervisor_executes_the_path_free_worker_contract() {
    let job_directory =
        std::env::temp_dir().join(format!("partprobe-worker-test-{}", std::process::id()));
    std::fs::create_dir_all(&job_directory).expect("job directory must be created");
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from(env!("CARGO_BIN_EXE_partprobe-geometry-worker")),
        job_directory.clone(),
        SupervisorPolicy::new(65_536, 5).expect("policy must be valid"),
    )
    .expect("supervisor must be valid");

    let response = supervisor.execute(&request(), &AtomicBool::new(false));

    assert_eq!(response.status(), StageStatus::FailedTerminal);
    assert_eq!(response.job_id().as_str(), "process-job-1");
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        "NATIVE_ADAPTER_UNAVAILABLE"
    );
    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}
