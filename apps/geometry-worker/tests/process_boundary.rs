use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
#[cfg(not(feature = "native-occt"))]
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use partprobe_domain::{RuleVersion, SchemaVersion};
use partprobe_geometry_core::{
    AnalysisProfile, AnalysisProfileId, GeometryStage, Sha256Digest, StageStatus,
};
#[cfg(feature = "native-occt")]
use partprobe_geometry_import::WORKER_OUTPUT_FILENAME;
#[cfg(all(not(feature = "native-occt"), not(any(unix, windows))))]
use partprobe_geometry_import::WorkerAssetFallbackReason;
use partprobe_geometry_import::{
    AssetCapability, AssetReadGrant, CorrelationId, GeometryJobId, GeometryWorkerRequest,
    GeometryWorkerSupervisor, ResourceQuotas, SupervisorPolicy, WORKER_INPUT_FILENAME,
    WorkerAssetTransport, WorkerAssetTransportPolicy, open_local_source_read_only,
};
#[cfg(not(feature = "native-occt"))]
use partprobe_geometry_import::{
    GeometryWorkerControlMessage, GeometryWorkerResponse, WorkerAssetManifest,
};
#[cfg(feature = "native-occt")]
use partprobe_test_support::geometry_fixtures::GeometryImportFailureExpectation;

fn request() -> GeometryWorkerRequest {
    mesh_request(
        "c46f940641e08eb3cbcaed5e1d90191c089651dd8d42064ecdaaa7a8b3e069ab",
        "process-job-1",
        "process-correlation-1",
    )
}

fn mesh_request(expected_hash: &str, job_id: &str, correlation_id: &str) -> GeometryWorkerRequest {
    GeometryWorkerRequest::new(
        SchemaVersion::new(1).expect("schema version must be valid"),
        GeometryJobId::new(job_id).expect("job ID must be valid"),
        CorrelationId::new(correlation_id).expect("correlation ID must be valid"),
        AssetCapability::new("asset-capability-1").expect("capability must be valid"),
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
            id: AnalysisProfileId::new("mesh-comparison-spike").expect("profile ID must be valid"),
            version: RuleVersion::new(1, 0, 0),
        },
        ResourceQuotas::new(1_000_000, 1_000_000, 100_000, 5_000).expect("quotas must be valid"),
    )
    .expect("request must be valid")
}

#[cfg(not(feature = "native-occt"))]
fn assert_stl_mesh_success(
    execution: &partprobe_geometry_import::GeometryWorkerExecution,
    request: &GeometryWorkerRequest,
) {
    let response = execution.response();
    assert_eq!(response.status(), StageStatus::SucceededWithWarnings);
    assert!(response.diagnostic_codes().is_empty());
    assert_eq!(
        response
            .snapshot_reference()
            .expect("mesh success must reference controlled output")
            .as_str(),
        partprobe_geometry_import::PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_REFERENCE
    );
    let output = execution
        .output()
        .expect("mesh success must return claimed controlled output");
    let snapshot = partprobe_geometry_import::decode_provisional_mesh_geometry_snapshot(
        output,
        request.expected_source_hash(),
    )
    .expect("mesh output must satisfy the source-bound schema");
    match snapshot.evidence() {
        partprobe_geometry_import::ProvisionalMeshEvidence::Stl(evidence) => {
            assert_eq!(evidence.triangle_count(), 12);
            assert!(evidence.enclosed_volume_source_units_cubed().is_some());
            assert!(evidence.center_of_mass_source_units().is_some());
            assert!(
                evidence
                    .warnings()
                    .iter()
                    .any(|warning| warning.as_str() == "UNITS_MISSING_REQUIRES_CONFIRMATION")
            );
        }
        partprobe_geometry_import::ProvisionalMeshEvidence::ThreeMf(_) => {
            panic!("STL worker request must retain STL evidence")
        }
    }
}

#[cfg(any(unix, windows))]
fn expected_direct_transport() -> WorkerAssetTransport {
    #[cfg(unix)]
    {
        WorkerAssetTransport::UnixDescriptor
    }
    #[cfg(windows)]
    {
        WorkerAssetTransport::WindowsHandle
    }
}

#[cfg(all(not(feature = "native-occt"), any(unix, windows)))]
fn direct_transport_token() -> &'static str {
    #[cfg(unix)]
    {
        "unix_descriptor"
    }
    #[cfg(windows)]
    {
        "windows_handle"
    }
}

fn cancellation_request(job_id: &str, wall_time_millis: u64) -> GeometryWorkerRequest {
    GeometryWorkerRequest::new(
        SchemaVersion::new(1).expect("schema version must be valid"),
        GeometryJobId::new(job_id).expect("job ID must be valid"),
        CorrelationId::new("cancellation-correlation").expect("correlation ID must be valid"),
        AssetCapability::new("cancellation-capability").expect("capability must be valid"),
        Sha256Digest::new("c46f940641e08eb3cbcaed5e1d90191c089651dd8d42064ecdaaa7a8b3e069ab")
            .expect("hash must be valid"),
        vec![GeometryStage::Intake],
        AnalysisProfile {
            id: AnalysisProfileId::new("cancellation-contract").expect("profile ID must be valid"),
            version: RuleVersion::new(1, 0, 0),
        },
        ResourceQuotas::new(1_000_000, 1_000_000, 100_000, wall_time_millis)
            .expect("quotas must be valid"),
    )
    .expect("request must be valid")
}

fn cancellation_fixture_supervisor(grace_millis: u64) -> GeometryWorkerSupervisor {
    cancellation_fixture_supervisor_in(grace_millis, std::env::temp_dir())
}

fn cancellation_fixture_supervisor_in(
    grace_millis: u64,
    working_directory: PathBuf,
) -> GeometryWorkerSupervisor {
    GeometryWorkerSupervisor::new(
        PathBuf::from(env!("CARGO_BIN_EXE_partprobe-cancellation-worker-fixture")),
        working_directory,
        SupervisorPolicy::new(65_536, 5, grace_millis, 2 * 1024 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid")
}

fn resource_fixture_supervisor(
    max_memory_bytes: u64,
    max_cpu_millis: u64,
) -> GeometryWorkerSupervisor {
    GeometryWorkerSupervisor::new(
        PathBuf::from(env!("CARGO_BIN_EXE_partprobe-cancellation-worker-fixture")),
        std::env::temp_dir(),
        SupervisorPolicy::new(65_536, 5, 100, max_memory_bytes, max_cpu_millis)
            .expect("resource policy must be valid"),
    )
    .expect("resource supervisor must be valid")
}

fn execute_resource_fixture(
    job_id: &str,
    supervisor: &GeometryWorkerSupervisor,
    cancellation: &AtomicBool,
) -> partprobe_geometry_import::GeometryWorkerExecution {
    execute_resource_fixture_with_wall_deadline(job_id, supervisor, cancellation, 5_000)
}

fn execute_resource_fixture_with_wall_deadline(
    job_id: &str,
    supervisor: &GeometryWorkerSupervisor,
    cancellation: &AtomicBool,
    wall_time_millis: u64,
) -> partprobe_geometry_import::GeometryWorkerExecution {
    let request = cancellation_request(job_id, wall_time_millis);
    let grant = cancellation_asset_grant(&request);
    supervisor.execute_with_grant(&request, grant, cancellation)
}

fn asset_grant(request: &GeometryWorkerRequest, source: &Path) -> AssetReadGrant {
    open_local_source_read_only(request.asset_capability().clone(), source)
        .expect("authorized source must create a read-only grant")
}

fn cancellation_asset_grant(request: &GeometryWorkerRequest) -> AssetReadGrant {
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm_ascii.stl");
    asset_grant(request, &source)
}

#[cfg(not(feature = "native-occt"))]
fn run_worker_control_message(
    worker_directory: &Path,
    message: &GeometryWorkerControlMessage,
) -> GeometryWorkerResponse {
    let mut child = Command::new(env!("CARGO_BIN_EXE_partprobe-geometry-worker"))
        .env_clear()
        .current_dir(worker_directory)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("worker must launch");
    let mut frame = serde_json::to_vec(message).expect("control message must serialize");
    frame.push(b'\n');
    child
        .stdin
        .as_mut()
        .expect("worker stdin must exist")
        .write_all(&frame)
        .expect("control frame must be written");
    let output = child
        .wait_with_output()
        .expect("worker process must be reaped");
    assert!(output.status.success(), "worker must return a response");
    serde_json::from_slice(&output.stdout).expect("worker response must deserialize")
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

#[cfg(feature = "native-occt")]
fn native_library_directory(occt_root: &Path) -> PathBuf {
    occt_root.join(if cfg!(windows) { "bin" } else { "lib" })
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
        SupervisorPolicy::new(65_536, 5, 250, 2 * 1024 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid");

    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm_ascii.stl");
    let request = request();
    let grant = asset_grant(&request, &source);
    let execution = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));
    let response = execution.response();

    assert_stl_mesh_success(&execution, &request);
    assert_eq!(response.job_id().as_str(), "process-job-1");
    assert_eq!(
        execution.asset_transport(),
        Some(WorkerAssetTransport::VerifiedPrivateCopy)
    );
    assert_eq!(execution.fallback_reason(), None);
    assert!(execution.output().is_some());
    assert!(!job_directory.join(WORKER_INPUT_FILENAME).exists());
    assert!(
        std::fs::read_dir(&job_directory)
            .expect("job root must be readable")
            .next()
            .is_none()
    );
    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}

#[cfg(not(feature = "native-occt"))]
#[test]
fn supervised_worker_emits_source_bound_three_mf_mesh_evidence() {
    let job_directory =
        std::env::temp_dir().join(format!("partprobe-worker-3mf-test-{}", std::process::id()));
    std::fs::create_dir_all(&job_directory).expect("job directory must be created");
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from(env!("CARGO_BIN_EXE_partprobe-geometry-worker")),
        job_directory.clone(),
        SupervisorPolicy::new(65_536, 5, 250, 2 * 1024 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid");
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/models/cube_10mm_3mf_millimeter.3mf");
    let request = mesh_request(
        "009ec4eb6b93fc87d2a50f998de6ee0d45965171fddf5675a707927fc8914320",
        "three-mf-process-job-1",
        "three-mf-process-correlation-1",
    );
    let grant = asset_grant(&request, &source);

    let execution = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));
    let response = execution.response();

    assert_eq!(response.status(), StageStatus::SucceededWithWarnings);
    assert_eq!(
        response
            .snapshot_reference()
            .expect("3MF success must reference controlled output")
            .as_str(),
        partprobe_geometry_import::PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_REFERENCE
    );
    let output = execution
        .output()
        .expect("3MF success must return claimed controlled output");
    let snapshot = partprobe_geometry_import::decode_provisional_mesh_geometry_snapshot(
        output,
        request.expected_source_hash(),
    )
    .expect("3MF output must satisfy the source-bound mesh schema");
    match snapshot.evidence() {
        partprobe_geometry_import::ProvisionalMeshEvidence::ThreeMf(evidence) => {
            assert_eq!(evidence.triangle_count(), 12);
            assert_eq!(evidence.surface_area_mm2(), 600.0);
            assert_eq!(evidence.enclosed_volume_mm3(), Some(1_000.0));
            assert!(evidence.center_of_mass_mm().is_some());
        }
        partprobe_geometry_import::ProvisionalMeshEvidence::Stl(_) => {
            panic!("3MF worker request must retain 3MF evidence")
        }
    }
    assert!(
        std::fs::read_dir(&job_directory)
            .expect("job root must be readable")
            .next()
            .is_none()
    );
    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}

#[cfg(not(feature = "native-occt"))]
#[test]
fn supervised_worker_preserves_withheld_open_mesh_measurements() {
    let job_directory = std::env::temp_dir().join(format!(
        "partprobe-worker-open-mesh-test-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&job_directory).expect("job directory must be created");
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from(env!("CARGO_BIN_EXE_partprobe-geometry-worker")),
        job_directory.clone(),
        SupervisorPolicy::new(65_536, 5, 250, 2 * 1024 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid");
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/models/open_cube_10mm_ascii.stl");
    let request = mesh_request(
        "f78bc469609e5805ced85c710be08b985443a21e1b5b3da9c7f19ffa008ed55d",
        "open-mesh-process-job-1",
        "open-mesh-process-correlation-1",
    );
    let grant = asset_grant(&request, &source);

    let execution = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));
    assert_eq!(
        execution.response().status(),
        StageStatus::SucceededWithWarnings
    );
    let snapshot = partprobe_geometry_import::decode_provisional_mesh_geometry_snapshot(
        execution
            .output()
            .expect("open mesh success must return controlled output"),
        request.expected_source_hash(),
    )
    .expect("open mesh output must satisfy the source-bound schema");
    match snapshot.evidence() {
        partprobe_geometry_import::ProvisionalMeshEvidence::Stl(evidence) => {
            assert!(!evidence.watertight());
            assert_eq!(evidence.enclosed_volume_source_units_cubed(), None);
            assert_eq!(evidence.center_of_mass_source_units(), None);
        }
        partprobe_geometry_import::ProvisionalMeshEvidence::ThreeMf(_) => {
            panic!("open STL request must retain STL evidence")
        }
    }
    assert!(
        std::fs::read_dir(&job_directory)
            .expect("job root must be readable")
            .next()
            .is_none()
    );
    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}

#[cfg(not(feature = "native-occt"))]
#[test]
fn supervised_worker_rejects_malformed_mesh_without_output() {
    let job_directory = std::env::temp_dir().join(format!(
        "partprobe-worker-malformed-mesh-test-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&job_directory).expect("job directory must be created");
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from(env!("CARGO_BIN_EXE_partprobe-geometry-worker")),
        job_directory.clone(),
        SupervisorPolicy::new(65_536, 5, 250, 2 * 1024 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid");
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/models/adversarial_ascii_stl_invalid_utf8.stl");
    let request = mesh_request(
        "53b117ca3ae61779a158287188df630c42179ff1aaf4ebc84ad39a40a75741fc",
        "malformed-mesh-process-job-1",
        "malformed-mesh-process-correlation-1",
    );
    let grant = asset_grant(&request, &source);

    let execution = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));
    let response = execution.response();

    assert_eq!(response.status(), StageStatus::FailedRecoverable);
    assert_eq!(response.diagnostic_codes()[0].as_str(), "STL_INVALID_TEXT");
    assert!(response.snapshot_reference().is_none());
    assert!(execution.output().is_none());
    assert!(
        std::fs::read_dir(&job_directory)
            .expect("job root must be readable")
            .next()
            .is_none()
    );
    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}

#[cfg(not(feature = "native-occt"))]
#[test]
fn step_dispatch_still_requires_the_native_adapter() {
    let job_directory = std::env::temp_dir().join(format!(
        "partprobe-worker-step-feature-off-test-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&job_directory).expect("job directory must be created");
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from(env!("CARGO_BIN_EXE_partprobe-geometry-worker")),
        job_directory.clone(),
        SupervisorPolicy::new(65_536, 5, 250, 2 * 1024 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid");
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm.step");
    let request = mesh_request(
        "031304b3a6d9dd55a97b3329e7238286ccfdaa7f13030bbe6e5c4c5744fcc8a2",
        "step-feature-off-job-1",
        "step-feature-off-correlation-1",
    );
    let grant = asset_grant(&request, &source);

    let execution = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));
    let response = execution.response();

    assert_eq!(response.status(), StageStatus::FailedTerminal);
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        "NATIVE_ADAPTER_UNAVAILABLE"
    );
    assert!(execution.output().is_none());
    assert!(
        std::fs::read_dir(&job_directory)
            .expect("job root must be readable")
            .next()
            .is_none()
    );
    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}

#[cfg(not(feature = "native-occt"))]
#[test]
fn preferred_direct_transport_selects_the_platform_supported_mode() {
    let job_directory = std::env::temp_dir().join(format!(
        "partprobe-worker-fallback-test-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&job_directory).expect("job directory must be created");
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from(env!("CARGO_BIN_EXE_partprobe-geometry-worker")),
        job_directory.clone(),
        SupervisorPolicy::new(65_536, 5, 250, 2 * 1024 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid")
    .with_asset_transport_policy(WorkerAssetTransportPolicy::PreferDirect);
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm_ascii.stl");
    let request = request();
    let grant = asset_grant(&request, &source);

    let execution = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));

    assert_stl_mesh_success(&execution, &request);
    #[cfg(any(unix, windows))]
    {
        assert_eq!(
            execution.asset_transport(),
            Some(expected_direct_transport())
        );
        assert_eq!(execution.fallback_reason(), None);
    }
    #[cfg(not(any(unix, windows)))]
    {
        assert_eq!(
            execution.asset_transport(),
            Some(WorkerAssetTransport::VerifiedPrivateCopy)
        );
        assert_eq!(
            execution.fallback_reason(),
            Some(WorkerAssetFallbackReason::DirectTransportUnavailable)
        );
    }
    assert!(
        std::fs::read_dir(&job_directory)
            .expect("job root must be readable")
            .next()
            .is_none()
    );
    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}

#[cfg(all(not(feature = "native-occt"), any(unix, windows)))]
#[test]
fn required_direct_transport_executes_without_a_staged_copy() {
    let job_directory = std::env::temp_dir().join(format!(
        "partprobe-worker-required-direct-test-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&job_directory).expect("job directory must be created");
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from(env!("CARGO_BIN_EXE_partprobe-geometry-worker")),
        job_directory.clone(),
        SupervisorPolicy::new(65_536, 5, 250, 2 * 1024 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid")
    .with_asset_transport_policy(WorkerAssetTransportPolicy::RequireDirect);
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm_ascii.stl");
    let request = request();
    let grant = asset_grant(&request, &source);

    let execution = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));

    assert_stl_mesh_success(&execution, &request);
    assert_eq!(
        execution.asset_transport(),
        Some(expected_direct_transport())
    );
    assert_eq!(execution.fallback_reason(), None);
    assert!(!job_directory.join(WORKER_INPUT_FILENAME).exists());
    assert!(
        std::fs::read_dir(&job_directory)
            .expect("job root must be readable")
            .next()
            .is_none()
    );
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
        SupervisorPolicy::new(65_536, 5, 250, 2 * 1024 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid");
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/models/open_cube_10mm_ascii.stl");

    let request = request();
    let grant = asset_grant(&request, &source);
    let execution = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));
    let response = execution.response();

    assert_eq!(response.status(), StageStatus::FailedRecoverable);
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        "ASSET_HASH_MISMATCH"
    );
    assert!(execution.output().is_none());
    assert!(!job_directory.join(WORKER_INPUT_FILENAME).exists());
    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}

#[cfg(not(feature = "native-occt"))]
#[test]
fn worker_rejects_manifest_identity_mismatch_before_adapter_dispatch() {
    let worker_directory = std::env::temp_dir().join(format!(
        "partprobe-worker-manifest-mismatch-test-{}",
        std::process::id()
    ));
    std::fs::create_dir(&worker_directory).expect("worker directory must be created");
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm_ascii.stl");
    let bytes = std::fs::read(source).expect("fixture must be readable");
    std::fs::write(worker_directory.join(WORKER_INPUT_FILENAME), &bytes)
        .expect("private copy must be written");
    let request = request();
    let manifest = WorkerAssetManifest::verified_private_copy(
        &request,
        u64::try_from(bytes.len()).expect("fixture length must fit"),
    );
    let message = GeometryWorkerControlMessage::execute(request, manifest);
    let mut value = serde_json::to_value(message).expect("control message must serialize");
    value["asset_manifest"]["job_id"] = serde_json::json!("other-job");
    let message: GeometryWorkerControlMessage =
        serde_json::from_value(value).expect("mismatched frame remains structurally valid");

    let response = run_worker_control_message(&worker_directory, &message);

    assert_eq!(response.status(), StageStatus::FailedRecoverable);
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        "ASSET_MANIFEST_MISMATCH"
    );
    std::fs::remove_file(worker_directory.join(WORKER_INPUT_FILENAME))
        .expect("private copy must be removable");
    std::fs::remove_dir(worker_directory).expect("worker directory must be removable");
}

#[cfg(not(feature = "native-occt"))]
#[test]
fn worker_recomputes_private_copy_length_and_hash_before_adapter_dispatch() {
    let worker_directory = std::env::temp_dir().join(format!(
        "partprobe-worker-copy-revalidation-test-{}",
        std::process::id()
    ));
    std::fs::create_dir(&worker_directory).expect("worker directory must be created");
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm_ascii.stl");
    let fixture = std::fs::read(source).expect("fixture must be readable");
    let request = request();
    let manifest = WorkerAssetManifest::verified_private_copy(
        &request,
        u64::try_from(fixture.len()).expect("fixture length must fit"),
    );
    let message = GeometryWorkerControlMessage::execute(request.clone(), manifest.clone());
    let staged_path = worker_directory.join(WORKER_INPUT_FILENAME);
    std::fs::write(&staged_path, vec![0_u8; fixture.len()])
        .expect("same-length tampered copy must be written");

    let hash_response = run_worker_control_message(&worker_directory, &message);

    assert_eq!(hash_response.status(), StageStatus::FailedRecoverable);
    assert_eq!(
        hash_response.diagnostic_codes()[0].as_str(),
        "ASSET_HASH_MISMATCH"
    );
    std::fs::write(&staged_path, &fixture[..fixture.len() - 1])
        .expect("short tampered copy must be written");
    let length_message = GeometryWorkerControlMessage::execute(request, manifest);

    let length_response = run_worker_control_message(&worker_directory, &length_message);

    assert_eq!(length_response.status(), StageStatus::FailedRecoverable);
    assert_eq!(
        length_response.diagnostic_codes()[0].as_str(),
        "ASSET_TRANSPORT_INVALID"
    );
    std::fs::remove_file(staged_path).expect("private copy must be removable");
    std::fs::remove_dir(worker_directory).expect("worker directory must be removable");
}

#[cfg(all(not(feature = "native-occt"), any(unix, windows)))]
#[test]
fn worker_rejects_an_unavailable_direct_resource_before_adapter_dispatch() {
    let worker_directory = std::env::temp_dir().join(format!(
        "partprobe-worker-direct-descriptor-test-{}",
        std::process::id()
    ));
    std::fs::create_dir(&worker_directory).expect("worker directory must be created");
    let request = request();
    let manifest = WorkerAssetManifest::verified_private_copy(&request, 1);
    let message = GeometryWorkerControlMessage::execute(request, manifest);
    let mut value = serde_json::to_value(message).expect("control message must serialize");
    value["asset_manifest"]["transport"] = serde_json::json!(direct_transport_token());
    value["asset_manifest"]["worker_resource_id"] = serde_json::json!(999_999);
    let message: GeometryWorkerControlMessage =
        serde_json::from_value(value).expect("direct frame remains structurally valid");

    let response = run_worker_control_message(&worker_directory, &message);

    assert_eq!(response.status(), StageStatus::FailedRecoverable);
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        "ASSET_TRANSPORT_INVALID"
    );
    std::fs::remove_dir(worker_directory).expect("worker directory must be removable");
}

#[test]
fn running_worker_acknowledges_user_cancellation_within_grace() {
    let supervisor = cancellation_fixture_supervisor(2_000);
    #[cfg(any(unix, windows))]
    let supervisor =
        supervisor.with_asset_transport_policy(WorkerAssetTransportPolicy::RequireDirect);
    let request = cancellation_request("cooperative-user-cancel", 5_000);
    let cancellation = Arc::new(AtomicBool::new(false));
    let cancellation_signal = Arc::clone(&cancellation);
    let signal = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(25));
        cancellation_signal.store(true, Ordering::Release);
    });
    let started = Instant::now();

    let grant = cancellation_asset_grant(&request);
    let execution = supervisor.execute_with_grant(&request, grant, &cancellation);
    let response = execution.response();

    signal.join().expect("cancellation signal must complete");
    assert_eq!(response.status(), StageStatus::FailedRecoverable);
    assert_eq!(response.diagnostic_codes()[0].as_str(), "WORKER_CANCELLED");
    assert_eq!(
        response.diagnostic_codes()[1].as_str(),
        "WORKER_CANCELLATION_ACKNOWLEDGED"
    );
    assert!(started.elapsed() < Duration::from_secs(2));
    #[cfg(any(unix, windows))]
    assert_eq!(
        execution.asset_transport(),
        Some(expected_direct_transport())
    );
}

#[test]
fn cancellation_response_without_acknowledgement_is_rejected() {
    let supervisor = cancellation_fixture_supervisor(2_000);
    let request = cancellation_request("unacknowledged-user-cancel", 5_000);
    let cancellation = Arc::new(AtomicBool::new(false));
    let cancellation_signal = Arc::clone(&cancellation);
    let signal = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(25));
        cancellation_signal.store(true, Ordering::Release);
    });

    let grant = cancellation_asset_grant(&request);
    let execution = supervisor.execute_with_grant(&request, grant, &cancellation);
    let response = execution.response();

    signal.join().expect("cancellation signal must complete");
    assert_eq!(response.status(), StageStatus::FailedRecoverable);
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        "WORKER_MALFORMED_RESPONSE"
    );
}

#[test]
fn uncooperative_worker_is_force_terminated_after_grace() {
    let job_root = std::env::temp_dir().join(format!(
        "partprobe-cancellation-force-test-{}",
        std::process::id()
    ));
    std::fs::create_dir(&job_root).expect("job root must be created");
    let supervisor = cancellation_fixture_supervisor_in(50, job_root.clone());
    let request = cancellation_request("uncooperative-user-cancel", 5_000);
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm_ascii.stl");
    let grant = asset_grant(&request, &source);
    let cancellation = Arc::new(AtomicBool::new(false));
    let cancellation_signal = Arc::clone(&cancellation);
    let signal = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(25));
        cancellation_signal.store(true, Ordering::Release);
    });
    let started = Instant::now();

    let execution = supervisor.execute_with_grant(&request, grant, &cancellation);
    let response = execution.response();

    signal.join().expect("cancellation signal must complete");
    assert_eq!(response.status(), StageStatus::FailedRecoverable);
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        "WORKER_CANCEL_FORCE_TERMINATED"
    );
    assert!(execution.output().is_none());
    assert!(
        std::fs::read_dir(&job_root)
            .expect("job root must be readable")
            .next()
            .is_none()
    );
    assert!(started.elapsed() < Duration::from_secs(1));
    std::fs::remove_dir(job_root).expect("empty job root must be removed");
}

#[test]
fn wall_deadline_uses_cooperative_cancellation_before_force() {
    let supervisor = cancellation_fixture_supervisor(2_000);
    let request = cancellation_request("cooperative-deadline", 30);
    let started = Instant::now();

    let grant = cancellation_asset_grant(&request);
    let execution = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));
    let response = execution.response();

    assert_eq!(response.status(), StageStatus::FailedRecoverable);
    assert_eq!(response.diagnostic_codes()[0].as_str(), "WORKER_TIMEOUT");
    assert_eq!(
        response.diagnostic_codes()[1].as_str(),
        "WORKER_CANCELLATION_ACKNOWLEDGED"
    );
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn uncooperative_deadline_is_force_terminated_after_grace() {
    let supervisor = cancellation_fixture_supervisor(50);
    let request = cancellation_request("uncooperative-deadline", 30);
    let started = Instant::now();

    let grant = cancellation_asset_grant(&request);
    let execution = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));
    let response = execution.response();

    assert_eq!(response.status(), StageStatus::FailedRecoverable);
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        "WORKER_TIMEOUT_FORCE_TERMINATED"
    );
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn worker_cpu_time_is_hard_limited_before_the_wall_deadline() {
    let supervisor = resource_fixture_supervisor(512 * 1024 * 1024, 100);

    let execution = execute_resource_fixture_with_wall_deadline(
        &format!("resource-cpu-{}", std::process::id()),
        &supervisor,
        &AtomicBool::new(false),
        30_000,
    );

    assert_eq!(
        execution.response().status(),
        StageStatus::FailedRecoverable
    );
    assert_eq!(
        execution.response().diagnostic_codes()[0].as_str(),
        "WORKER_EXIT"
    );
}

#[cfg(any(target_os = "linux", windows))]
#[test]
fn worker_address_space_is_hard_limited_without_exhausting_the_supervisor() {
    let supervisor = resource_fixture_supervisor(192 * 1024 * 1024, 60_000);
    let started = Instant::now();

    let execution = execute_resource_fixture(
        &format!("resource-memory-{}", std::process::id()),
        &supervisor,
        &AtomicBool::new(false),
    );

    assert_eq!(
        execution.response().status(),
        StageStatus::FailedRecoverable
    );
    assert_eq!(
        execution.response().diagnostic_codes()[0].as_str(),
        "WORKER_EXIT"
    );
    assert!(started.elapsed() < Duration::from_secs(3));
}

#[cfg(unix)]
#[test]
fn worker_regular_file_output_is_hard_limited_and_cleaned() {
    let supervisor = resource_fixture_supervisor(512 * 1024 * 1024, 60_000);

    let execution = execute_resource_fixture(
        &format!("resource-output-{}", std::process::id()),
        &supervisor,
        &AtomicBool::new(false),
    );

    assert_eq!(
        execution.response().status(),
        StageStatus::FailedRecoverable
    );
    assert_eq!(
        execution.response().diagnostic_codes()[0].as_str(),
        "WORKER_EXIT"
    );
    assert!(execution.output().is_none());
}

#[test]
fn aggregate_workspace_output_is_supervised_terminated_and_cleaned() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("test clock must follow the Unix epoch")
        .as_nanos();
    let job_root = std::env::temp_dir().join(format!(
        "partprobe-workspace-budget-test-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&job_root).expect("dedicated worker root must be created");
    let supervisor = cancellation_fixture_supervisor_in(100, job_root.clone());
    let started = Instant::now();

    let execution = execute_resource_fixture(
        &format!("resource-workspace-aggregate-{}", std::process::id()),
        &supervisor,
        &AtomicBool::new(false),
    );

    assert_eq!(
        execution.response().status(),
        StageStatus::FailedRecoverable
    );
    assert_eq!(
        execution.response().diagnostic_codes()[0].as_str(),
        "WORKSPACE_OUTPUT_LIMIT_EXCEEDED"
    );
    assert!(execution.output().is_none());
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(
        std::fs::read_dir(&job_root)
            .expect("worker root must remain readable")
            .count(),
        0,
        "owned job workspace and all worker scratch files must be removed"
    );
    std::fs::remove_dir(job_root).expect("empty dedicated worker root must be removed");
}

#[cfg(target_os = "linux")]
#[test]
fn linux_parser_boundary_denies_network_socket_creation() {
    let supervisor = resource_fixture_supervisor(512 * 1024 * 1024, 60_000);

    let execution = execute_resource_fixture(
        &format!("resource-network-denial-{}", std::process::id()),
        &supervisor,
        &AtomicBool::new(false),
    );

    assert_eq!(
        execution.response().status(),
        StageStatus::FailedRecoverable
    );
    assert_eq!(
        execution.response().diagnostic_codes()[0].as_str(),
        "WORKER_NETWORK_DENIED"
    );
    assert!(execution.output().is_none());
}

#[cfg(target_os = "linux")]
#[test]
fn linux_parser_boundary_denies_descendant_creation() {
    let supervisor = resource_fixture_supervisor(512 * 1024 * 1024, 60_000);

    let execution = execute_resource_fixture(
        &format!("resource-descendant-denial-{}", std::process::id()),
        &supervisor,
        &AtomicBool::new(false),
    );

    assert_eq!(
        execution.response().status(),
        StageStatus::FailedRecoverable
    );
    assert_eq!(
        execution.response().diagnostic_codes()[0].as_str(),
        "WORKER_DESCENDANT_DENIED"
    );
    assert!(execution.output().is_none());
}

#[cfg(not(target_os = "linux"))]
#[test]
fn forced_termination_contains_normal_worker_descendants() {
    let job_id = format!("resource-descendant-{}", std::process::id());
    let marker = std::env::temp_dir().join(format!("partprobe-{job_id}-marker"));
    if marker.exists() {
        std::fs::remove_file(&marker).expect("stale resource marker must be removable");
    }
    let supervisor = resource_fixture_supervisor(512 * 1024 * 1024, 60_000);
    let cancellation = Arc::new(AtomicBool::new(false));
    let cancellation_signal = Arc::clone(&cancellation);
    let signal = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        cancellation_signal.store(true, Ordering::Release);
    });

    let execution = execute_resource_fixture(&job_id, &supervisor, &cancellation);
    signal.join().expect("cancellation signal must complete");
    std::thread::sleep(Duration::from_millis(900));

    assert_eq!(
        execution.response().status(),
        StageStatus::FailedRecoverable
    );
    assert_eq!(
        execution.response().diagnostic_codes()[0].as_str(),
        "WORKER_CANCEL_FORCE_TERMINATED"
    );
    assert!(
        !marker.exists(),
        "terminated descendants must not emit a marker"
    );
}

#[test]
fn private_job_namespace_preserves_a_preexisting_parent_file() {
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
        SupervisorPolicy::new(65_536, 5, 250, 2 * 1024 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid");
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm_ascii.stl");

    let request = request();
    let grant = asset_grant(&request, &source);
    let execution = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));
    let response = execution.response();

    assert_eq!(response.status(), StageStatus::FailedRecoverable);
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        "WORKER_LAUNCH_FAILED"
    );
    assert!(execution.output().is_none());
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
        SupervisorPolicy::new(65_536, 5, 250, 2 * 1024 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid");
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm_ascii.stl");
    let grant = open_local_source_read_only(
        AssetCapability::new("different-asset-capability").expect("capability must be valid"),
        &source,
    )
    .expect("authorized source must create a read grant");

    let execution = supervisor.execute_with_grant(&request(), grant, &AtomicBool::new(false));
    let response = execution.response();

    assert_eq!(response.status(), StageStatus::FailedRecoverable);
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        "ASSET_GRANT_MISMATCH"
    );
    assert!(execution.output().is_none());
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
        SupervisorPolicy::new(65_536, 5, 250, 2 * 1024 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid");

    let execution = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));
    let response = execution.response();

    assert_eq!(response.status(), StageStatus::FailedRecoverable);
    assert_eq!(
        response.diagnostic_codes()[0].as_str(),
        "ASSET_STAGE_FAILED"
    );
    assert!(execution.output().is_none());
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
        SupervisorPolicy::new(65_536, 5, 250, 2 * 1024 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid");
    #[cfg(any(unix, windows))]
    let supervisor =
        supervisor.with_asset_transport_policy(WorkerAssetTransportPolicy::RequireDirect);

    let execution = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));
    let response = execution.response();

    assert_stl_mesh_success(&execution, &request);
    assert_eq!(response.status(), StageStatus::SucceededWithWarnings);
    assert!(execution.output().is_some());
    #[cfg(any(unix, windows))]
    assert_eq!(
        execution.asset_transport(),
        Some(expected_direct_transport())
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
        SupervisorPolicy::new(65_536, 5, 250, 2 * 1024 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid")
    .with_native_library_directory(native_library_directory(&occt_root))
    .expect("native library directory must be valid");
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm.step");

    let request = native_request(
        "031304b3a6d9dd55a97b3329e7238286ccfdaa7f13030bbe6e5c4c5744fcc8a2",
        "native-process-job-1",
        "native-process-correlation-1",
    );
    let grant = asset_grant(&request, &source);
    let execution = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));
    let response = execution.response();

    assert_eq!(response.status(), StageStatus::Succeeded);
    assert_eq!(
        response
            .snapshot_reference()
            .expect("success must reference a snapshot")
            .as_str(),
        "geometry-snapshot-v1"
    );
    let output = execution
        .output()
        .expect("success must return claimed controlled output");
    assert_eq!(
        output.snapshot_reference(),
        response
            .snapshot_reference()
            .expect("success must reference a snapshot")
    );
    assert!(output.byte_length() > 0);
    assert_eq!(output.content_hash().as_str().len(), 64);
    let snapshot = partprobe_geometry_import::decode_provisional_geometry_snapshot(
        output,
        request.expected_source_hash(),
    )
    .expect("worker snapshot must satisfy the provisional schema and source binding");
    assert_eq!(snapshot.surface_area_mm2(), "600");
    assert_eq!(snapshot.enclosed_volume_mm3(), "1000");
    assert_eq!(snapshot.center_of_mass_mm(), ["5", "5", "5"]);
    assert_eq!(snapshot.solid_body_count(), 1);
    assert!(!job_directory.join(WORKER_INPUT_FILENAME).exists());
    assert!(!job_directory.join(WORKER_OUTPUT_FILENAME).exists());
    assert!(
        std::fs::read_dir(&job_directory)
            .expect("job root must be readable")
            .next()
            .is_none()
    );

    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}

#[cfg(feature = "native-occt")]
#[test]
fn supervised_native_worker_measures_the_independently_authored_step_prism() {
    let job_directory = std::env::temp_dir().join(format!(
        "partprobe-independent-native-worker-test-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&job_directory).expect("job directory must be created");
    let occt_root =
        PathBuf::from(std::env::var_os("PARTPROBE_OCCT_ROOT").expect("OCCT root must be set"));
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from(env!("CARGO_BIN_EXE_partprobe-geometry-worker")),
        job_directory.clone(),
        SupervisorPolicy::new(65_536, 5, 250, 2 * 1024 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid")
    .with_native_library_directory(native_library_directory(&occt_root))
    .expect("native library directory must be valid");
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/models/rectangular_prism_12x8x5.step");
    let request = native_request(
        "a3a2cceef68a98212a2b05ac376da747758cc360fb02085fee3f6db766dc2138",
        "native-independent-process-job-1",
        "native-independent-process-correlation-1",
    );
    let grant = asset_grant(&request, &source);

    let execution = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));
    let response = execution.response();

    assert_eq!(response.status(), StageStatus::Succeeded);
    let output = execution
        .output()
        .expect("success must return claimed controlled output");
    let snapshot = partprobe_geometry_import::decode_provisional_geometry_snapshot(
        output,
        request.expected_source_hash(),
    )
    .expect("independent worker snapshot must satisfy source-bound schema");
    assert_eq!(snapshot.surface_area_mm2(), "392");
    assert_eq!(snapshot.enclosed_volume_mm3(), "480");
    assert_eq!(snapshot.center_of_mass_mm(), ["6", "4", "2.5"]);
    assert_eq!(snapshot.solid_body_count(), 1);
    assert!(!job_directory.join(WORKER_INPUT_FILENAME).exists());
    assert!(!job_directory.join(WORKER_OUTPUT_FILENAME).exists());
    assert!(
        std::fs::read_dir(&job_directory)
            .expect("job root must be readable")
            .next()
            .is_none()
    );

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
        SupervisorPolicy::new(65_536, 5, 250, 2 * 1024 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid")
    .with_native_library_directory(native_library_directory(&occt_root))
    .expect("native library directory must be valid");
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/invalid_entity.step");
    let request = native_request(
        expectation.source_sha256().as_str(),
        "invalid-step-process-job-1",
        "invalid-step-process-correlation-1",
    );

    let grant = asset_grant(&request, &source);
    let execution = supervisor.execute_with_grant(&request, grant, &AtomicBool::new(false));
    let response = execution.response();

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
        execution.output().is_some(),
        expectation.output_file_expected()
    );
    assert_eq!(
        job_directory.join(WORKER_INPUT_FILENAME).exists(),
        expectation.staged_input_retained()
    );
    std::fs::remove_dir(job_directory).expect("empty job directory must be removed");
}
