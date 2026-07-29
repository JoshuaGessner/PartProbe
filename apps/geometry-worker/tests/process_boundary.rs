use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use partprobe_domain::{RuleVersion, SchemaVersion};
use partprobe_geometry_core::{
    AnalysisProfile, AnalysisProfileId, GeometryStage, Sha256Digest, StageStatus,
};
#[cfg(feature = "native-occt")]
use partprobe_geometry_import::WORKER_OUTPUT_FILENAME;
use partprobe_geometry_import::{
    AssetCapability, AssetReadGrant, CorrelationId, GeometryJobId, GeometryWorkerRequest,
    GeometryWorkerSupervisor, ResourceQuotas, SupervisorPolicy, WORKER_INPUT_FILENAME,
};
#[cfg(feature = "native-occt")]
use partprobe_test_support::geometry_fixtures::GeometryImportFailureExpectation;

fn request() -> GeometryWorkerRequest {
    GeometryWorkerRequest::new(
        SchemaVersion::new(1).expect("schema version must be valid"),
        GeometryJobId::new("process-job-1").expect("job ID must be valid"),
        CorrelationId::new("process-correlation-1").expect("correlation ID must be valid"),
        AssetCapability::new("asset-capability-1").expect("capability must be valid"),
        Sha256Digest::new("c46f940641e08eb3cbcaed5e1d90191c089651dd8d42064ecdaaa7a8b3e069ab")
            .expect("hash must be valid"),
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

fn asset_grant(request: &GeometryWorkerRequest, source: &Path) -> AssetReadGrant {
    AssetReadGrant::new(
        request.asset_capability().clone(),
        File::open(source).expect("authorized source must open"),
    )
    .expect("authorized source must create a read grant")
}

#[cfg(feature = "native-occt")]
fn native_request(
    expected_hash: &str,
    job_id: &str,
    correlation_id: &str,
) -> GeometryWorkerRequest {
    GeometryWorkerRequest::new(
        SchemaVersion::new(1).expect("schema version must be valid"),
        GeometryJobId::new(job_id).expect("job ID must be valid"),
        CorrelationId::new(correlation_id).expect("correlation ID must be valid"),
        AssetCapability::new("native-asset-capability-1").expect("capability must be valid"),
        Sha256Digest::new(expected_hash).expect("hash must be valid"),
        vec![
            GeometryStage::Intake,
            GeometryStage::Identify,
            GeometryStage::Parse,
            GeometryStage::UnitResolution,
            GeometryStage::Validation,
            GeometryStage::BasicProperties,
        ],
        AnalysisProfile {
            id: AnalysisProfileId::new("occt-step-spike").expect("profile ID must be valid"),
            version: RuleVersion::new(1, 0, 0),
        },
        ResourceQuotas::new(1_000_000, 65_536, 100_000, 5_000).expect("quotas must be valid"),
    )
    .expect("request must be valid")
}

#[cfg(not(feature = "native-occt"))]
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

    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm_ascii.stl");
    let request = request();
    let grant = asset_grant(&request, &source);
    let response = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));

    assert_eq!(response.status(), StageStatus::FailedTerminal);
    assert_eq!(response.job_id().as_str(), "process-job-1");
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        "NATIVE_ADAPTER_UNAVAILABLE"
    );
    assert!(!job_directory.join(WORKER_INPUT_FILENAME).exists());
    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}

#[test]
fn supervisor_rejects_hash_mismatch_before_worker_launch() {
    let job_directory =
        std::env::temp_dir().join(format!("partprobe-worker-hash-test-{}", std::process::id()));
    std::fs::create_dir_all(&job_directory).expect("job directory must be created");
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from("worker-must-not-launch"),
        job_directory.clone(),
        SupervisorPolicy::new(65_536, 5).expect("policy must be valid"),
    )
    .expect("supervisor must be valid");
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/models/open_cube_10mm_ascii.stl");

    let request = request();
    let grant = asset_grant(&request, &source);
    let response = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));

    assert_eq!(response.status(), StageStatus::FailedRecoverable);
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        "ASSET_HASH_MISMATCH"
    );
    assert!(!job_directory.join(WORKER_INPUT_FILENAME).exists());
    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}

#[test]
fn staging_conflict_preserves_the_preexisting_file() {
    let job_directory = std::env::temp_dir().join(format!(
        "partprobe-worker-conflict-test-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&job_directory).expect("job directory must be created");
    let staged_path = job_directory.join(WORKER_INPUT_FILENAME);
    std::fs::write(&staged_path, b"preexisting-controlled-file")
        .expect("preexisting file must be created");
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from("worker-must-not-launch"),
        job_directory.clone(),
        SupervisorPolicy::new(65_536, 5).expect("policy must be valid"),
    )
    .expect("supervisor must be valid");
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm_ascii.stl");

    let request = request();
    let grant = asset_grant(&request, &source);
    let response = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));

    assert_eq!(response.status(), StageStatus::FailedRecoverable);
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        "ASSET_STAGE_FAILED"
    );
    assert_eq!(
        std::fs::read(&staged_path).expect("preexisting file must remain"),
        b"preexisting-controlled-file"
    );
    std::fs::remove_file(staged_path).expect("preexisting file must be removable");
    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}

#[test]
fn supervisor_rejects_a_grant_bound_to_another_capability() {
    let job_directory = std::env::temp_dir().join(format!(
        "partprobe-worker-grant-mismatch-test-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&job_directory).expect("job directory must be created");
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from("worker-must-not-launch"),
        job_directory.clone(),
        SupervisorPolicy::new(65_536, 5).expect("policy must be valid"),
    )
    .expect("supervisor must be valid");
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm_ascii.stl");
    let grant = AssetReadGrant::new(
        AssetCapability::new("different-asset-capability").expect("capability must be valid"),
        File::open(source).expect("authorized source must open"),
    )
    .expect("authorized source must create a read grant");

    let response = supervisor.execute_with_grant(&request(), grant, &AtomicBool::new(false));

    assert_eq!(response.status(), StageStatus::FailedRecoverable);
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        "ASSET_GRANT_MISMATCH"
    );
    assert!(!job_directory.join(WORKER_INPUT_FILENAME).exists());
    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}

#[test]
fn supervisor_rejects_source_length_drift_after_grant_creation() {
    let job_directory = std::env::temp_dir().join(format!(
        "partprobe-worker-grant-drift-test-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&job_directory).expect("job directory must be created");
    let source = job_directory.join("authorized-source.asset");
    std::fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm_ascii.stl"),
        &source,
    )
    .expect("source fixture must copy");
    let request = request();
    let grant = asset_grant(&request, &source);
    OpenOptions::new()
        .append(true)
        .open(&source)
        .expect("source must reopen for drift simulation")
        .write_all(b"drift")
        .expect("source drift must be written");
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from("worker-must-not-launch"),
        job_directory.clone(),
        SupervisorPolicy::new(65_536, 5).expect("policy must be valid"),
    )
    .expect("supervisor must be valid");

    let response = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));

    assert_eq!(response.status(), StageStatus::FailedRecoverable);
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        "ASSET_STAGE_FAILED"
    );
    assert!(!job_directory.join(WORKER_INPUT_FILENAME).exists());
    std::fs::remove_file(source).expect("drift source must be removable");
    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}

#[cfg(not(feature = "native-occt"))]
#[test]
fn open_grant_remains_authoritative_after_its_source_path_is_removed() {
    let job_directory = std::env::temp_dir().join(format!(
        "partprobe-worker-open-grant-test-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&job_directory).expect("job directory must be created");
    let source = job_directory.join("authorized-source.asset");
    std::fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm_ascii.stl"),
        &source,
    )
    .expect("source fixture must copy");
    let request = request();
    let grant = asset_grant(&request, &source);
    std::fs::remove_file(source).expect("source path must be removable after grant creation");
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from(env!("CARGO_BIN_EXE_partprobe-geometry-worker")),
        job_directory.clone(),
        SupervisorPolicy::new(65_536, 5).expect("policy must be valid"),
    )
    .expect("supervisor must be valid");

    let response = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));

    assert_eq!(response.status(), StageStatus::FailedTerminal);
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        "NATIVE_ADAPTER_UNAVAILABLE"
    );
    assert!(!job_directory.join(WORKER_INPUT_FILENAME).exists());
    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}

#[cfg(feature = "native-occt")]
#[test]
fn supervised_native_worker_measures_the_analytic_step_cube() {
    let job_directory = std::env::temp_dir().join(format!(
        "partprobe-native-worker-test-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&job_directory).expect("job directory must be created");
    let occt_root =
        PathBuf::from(std::env::var_os("PARTPROBE_OCCT_ROOT").expect("OCCT root must be set"));
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from(env!("CARGO_BIN_EXE_partprobe-geometry-worker")),
        job_directory.clone(),
        SupervisorPolicy::new(65_536, 5).expect("policy must be valid"),
    )
    .expect("supervisor must be valid")
    .with_native_library_directory(occt_root.join("lib"))
    .expect("native library directory must be valid");
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm.step");

    let request = native_request(
        "031304b3a6d9dd55a97b3329e7238286ccfdaa7f13030bbe6e5c4c5744fcc8a2",
        "native-process-job-1",
        "native-process-correlation-1",
    );
    let grant = asset_grant(&request, &source);
    let response = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));

    assert_eq!(response.status(), StageStatus::Succeeded);
    assert_eq!(
        response
            .snapshot_reference()
            .expect("success must reference a snapshot")
            .as_str(),
        "geometry-snapshot-v1"
    );
    let output = job_directory.join(WORKER_OUTPUT_FILENAME);
    let snapshot: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&output).expect("worker snapshot must be readable"))
            .expect("worker snapshot must be valid JSON");
    assert_eq!(snapshot["evidence_state"], "provisional_spike");
    assert_eq!(snapshot["surface_area_mm2"], "600");
    assert_eq!(snapshot["enclosed_volume_mm3"], "1000");
    assert_eq!(
        snapshot["center_of_mass_mm"],
        serde_json::json!(["5", "5", "5"])
    );
    assert!(!job_directory.join(WORKER_INPUT_FILENAME).exists());

    std::fs::remove_file(output).expect("snapshot must be removable");
    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}

#[cfg(feature = "native-occt")]
#[test]
fn supervised_native_worker_rejects_invalid_step_entity_without_output() {
    let expectation: GeometryImportFailureExpectation = serde_json::from_str(include_str!(
        "../../../fixtures/expected/invalid_step_entity_rejection.json"
    ))
    .expect("failure expectation must be valid");
    assert_eq!(expectation.fixture_id(), "FIX-STEP-002");

    let job_directory = std::env::temp_dir().join(format!(
        "partprobe-invalid-step-worker-test-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&job_directory).expect("job directory must be created");
    let occt_root =
        PathBuf::from(std::env::var_os("PARTPROBE_OCCT_ROOT").expect("OCCT root must be set"));
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from(env!("CARGO_BIN_EXE_partprobe-geometry-worker")),
        job_directory.clone(),
        SupervisorPolicy::new(65_536, 5).expect("policy must be valid"),
    )
    .expect("supervisor must be valid")
    .with_native_library_directory(occt_root.join("lib"))
    .expect("native library directory must be valid");
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/invalid_entity.step");
    let request = native_request(
        expectation.source_sha256().as_str(),
        "invalid-step-process-job-1",
        "invalid-step-process-correlation-1",
    );

    let grant = asset_grant(&request, &source);
    let response = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));

    assert_eq!(response.status(), expectation.expected_status());
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        expectation.expected_diagnostic_code().as_str()
    );
    assert_eq!(
        response.snapshot_reference().is_some(),
        expectation.snapshot_expected()
    );
    assert_eq!(
        job_directory.join(WORKER_OUTPUT_FILENAME).exists(),
        expectation.output_file_expected()
    );
    assert_eq!(
        job_directory.join(WORKER_INPUT_FILENAME).exists(),
        expectation.staged_input_retained()
    );
    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}
