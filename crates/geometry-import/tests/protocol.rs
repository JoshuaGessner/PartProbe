use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use partprobe_domain::{AssetRootId, RuleVersion, SchemaVersion};
use partprobe_geometry_core::{
    AnalysisProfile, AnalysisProfileId, GeometryStage, Sha256Digest, StageStatus,
};
use partprobe_geometry_import::{
    AssetCapability, ControlledGeometryResult, ControlledWorkerOutput, CorrelationId,
    GeometryJobId, GeometryWorkerControlMessage, GeometryWorkerRequest, GeometryWorkerSupervisor,
    LocalAssetRoot, ProvisionalMeshEvidence, ProvisionalMeshGeometrySnapshot, ResourceQuotas,
    SnapshotReference, StlLimits, SupervisorPolicy, ThreeMfLimits,
    WORKER_ASSET_TRANSPORT_SCHEMA_VERSION, WORKER_CONTROL_SCHEMA_VERSION, WorkerAssetManifest,
    WorkerAssetTransport, WorkerAssetTransportPolicy, WorkerCancellationReason, WorkerTermination,
    analyze_3mf, analyze_stl, decode_controlled_geometry_result,
    decode_provisional_geometry_snapshot, decode_provisional_mesh_geometry_snapshot,
    open_local_source_read_only, recoverable_termination_response,
};
use sha2::{Digest, Sha256};

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

fn sha256_digest(bytes: &[u8]) -> Sha256Digest {
    let mut hash = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(&mut hash, "{byte:02x}").expect("writing to a String cannot fail");
    }
    Sha256Digest::new(hash).expect("content hash must be valid")
}

fn claimed_output(reference: &str, bytes: Vec<u8>) -> ControlledWorkerOutput {
    let content_hash = sha256_digest(&bytes);
    ControlledWorkerOutput::from_claimed_parts(
        SnapshotReference::new(reference).expect("reference must be valid"),
        content_hash,
        bytes.into_boxed_slice(),
    )
    .expect("claimed output must be valid")
}

#[test]
fn provisional_snapshot_decoder_binds_schema_reference_and_source_hash() {
    let source_hash = Sha256Digest::new("a".repeat(64)).expect("source hash must be valid");
    let bytes = serde_json::to_vec(&serde_json::json!({
        "schema_version": 1,
        "evidence_state": "provisional_spike",
        "source_hash": source_hash.as_str(),
        "representation": "exact_brep",
        "canonical_units": "millimeter",
        "occt_version": "8.0.0",
        "adapter_abi_version": 3,
        "decimal_scale": 6,
        "transferred_roots": 1,
        "solid_body_count": 1,
        "surface_area_mm2": "600",
        "enclosed_volume_mm3": "1000",
        "center_of_mass_mm": ["5", "5", "5"]
    }))
    .expect("snapshot fixture must serialize")
    .into_boxed_slice();
    let output = claimed_output("geometry-snapshot-v1", bytes.into_vec());

    let decoded = decode_provisional_geometry_snapshot(&output, &source_hash)
        .expect("schema and source binding must pass");
    assert_eq!(decoded.enclosed_volume_mm3(), "1000");
    assert!(matches!(
        decode_controlled_geometry_result(&output, &source_hash)
            .expect("generic controlled decoder must retain exact STEP v1"),
        ControlledGeometryResult::ExactBrep(_)
    ));
    assert!(
        decode_provisional_geometry_snapshot(
            &output,
            &Sha256Digest::new("b".repeat(64)).expect("alternate hash must be valid")
        )
        .is_err()
    );
}

#[test]
fn provisional_mesh_decoder_binds_variant_reference_and_source_hash() {
    let stl_bytes = include_bytes!("../../../fixtures/models/cube_10mm_ascii.stl");
    let stl_source_hash = sha256_digest(stl_bytes);
    let stl = analyze_stl(
        stl_bytes,
        StlLimits::new(64 * 1024, 1_000).expect("STL limits must be valid"),
    )
    .expect("governed STL fixture must analyze");
    let stl_snapshot = ProvisionalMeshGeometrySnapshot::from_stl(stl_source_hash.clone(), stl);
    let stl_output = claimed_output(
        partprobe_geometry_import::PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_REFERENCE,
        serde_json::to_vec(&stl_snapshot).expect("mesh snapshot must serialize"),
    );

    let decoded = decode_provisional_mesh_geometry_snapshot(&stl_output, &stl_source_hash)
        .expect("STL mesh schema and source binding must pass");
    assert!(matches!(
        decoded.evidence(),
        ProvisionalMeshEvidence::Stl(_)
    ));
    assert!(matches!(
        decode_controlled_geometry_result(&stl_output, &stl_source_hash)
            .expect("generic controlled decoder must accept mesh v1"),
        ControlledGeometryResult::Mesh(_)
    ));
    assert!(
        decode_provisional_mesh_geometry_snapshot(
            &stl_output,
            &Sha256Digest::new("b".repeat(64)).expect("alternate hash must be valid")
        )
        .is_err()
    );

    let wrong_reference = claimed_output(
        partprobe_geometry_import::PROVISIONAL_GEOMETRY_SNAPSHOT_REFERENCE,
        stl_output.bytes().to_vec(),
    );
    assert!(decode_controlled_geometry_result(&wrong_reference, &stl_source_hash).is_err());

    let three_mf_bytes = include_bytes!("../../../fixtures/models/cube_10mm_3mf_millimeter.3mf");
    let three_mf_source_hash = sha256_digest(three_mf_bytes);
    let three_mf = analyze_3mf(
        three_mf_bytes,
        ThreeMfLimits::new(
            64 * 1024,
            16,
            64 * 1024,
            32 * 1024,
            100,
            1_000,
            4,
            3,
            8,
            100,
        )
        .expect("3MF limits must be valid"),
    )
    .expect("governed 3MF fixture must analyze");
    let three_mf_snapshot =
        ProvisionalMeshGeometrySnapshot::from_three_mf(three_mf_source_hash.clone(), three_mf);
    let three_mf_output = claimed_output(
        partprobe_geometry_import::PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_REFERENCE,
        serde_json::to_vec(&three_mf_snapshot).expect("mesh snapshot must serialize"),
    );
    let decoded =
        decode_provisional_mesh_geometry_snapshot(&three_mf_output, &three_mf_source_hash)
            .expect("3MF mesh schema and source binding must pass");
    assert!(matches!(
        decoded.evidence(),
        ProvisionalMeshEvidence::ThreeMf(_)
    ));
}

#[test]
fn provisional_mesh_schema_rejects_false_policy_and_measurement_authority() {
    let source_bytes = include_bytes!("../../../fixtures/models/open_cube_10mm_ascii.stl");
    let source_hash = sha256_digest(source_bytes);
    let evidence = analyze_stl(
        source_bytes,
        StlLimits::new(64 * 1024, 1_000).expect("STL limits must be valid"),
    )
    .expect("governed open STL fixture must analyze");
    assert!(evidence.enclosed_volume_source_units_cubed().is_none());
    let snapshot = ProvisionalMeshGeometrySnapshot::from_stl(source_hash.clone(), evidence);
    let mut value = serde_json::to_value(snapshot).expect("mesh snapshot must serialize");
    value["evidence"]["analysis"]["enclosed_volume_source_units_cubed"] =
        serde_json::json!(1_000.0);
    value["evidence"]["analysis"]["center_of_mass_source_units"] =
        serde_json::json!({"x": 5.0, "y": 5.0, "z": 5.0});
    let false_measurements = claimed_output(
        partprobe_geometry_import::PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_REFERENCE,
        serde_json::to_vec(&value).expect("tampered snapshot must serialize"),
    );
    assert!(decode_provisional_mesh_geometry_snapshot(&false_measurements, &source_hash).is_err());

    value["evidence"]["analysis"]["enclosed_volume_source_units_cubed"] = serde_json::Value::Null;
    value["evidence"]["analysis"]["center_of_mass_source_units"] = serde_json::Value::Null;
    value["evidence"]["analysis"]["topology_policy_version"] = serde_json::json!("unreviewed");
    let false_policy = claimed_output(
        partprobe_geometry_import::PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_REFERENCE,
        serde_json::to_vec(&value).expect("tampered snapshot must serialize"),
    );
    assert!(decode_provisional_mesh_geometry_snapshot(&false_policy, &source_hash).is_err());
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
    let manifest = WorkerAssetManifest::verified_private_copy(&active_request, 42);
    let execute = GeometryWorkerControlMessage::execute(active_request.clone(), manifest.clone());
    let value = serde_json::to_value(&execute).expect("execute frame must serialize");

    assert_eq!(
        execute.control_schema_version(),
        WORKER_CONTROL_SCHEMA_VERSION
    );
    assert_eq!(value["message"], "execute");
    assert_eq!(
        value["asset_manifest"]["transport"],
        "verified_private_copy"
    );
    assert_eq!(
        value["asset_manifest"]["transport_schema_version"],
        WORKER_ASSET_TRANSPORT_SCHEMA_VERSION
    );
    assert!(value.get("path").is_none());
    assert!(value.get("descriptor").is_none());
    assert!(value.get("handle").is_none());
    assert!(value["asset_manifest"]["worker_resource_id"].is_null());
    let decoded: GeometryWorkerControlMessage =
        serde_json::from_value(value.clone()).expect("execute frame must deserialize");
    let (decoded_request, decoded_manifest) = decoded
        .into_execute()
        .expect("execute frame must contain a request");
    assert_eq!(decoded_request, active_request);
    assert_eq!(decoded_manifest, manifest);
    decoded_manifest
        .validate_for(&decoded_request)
        .expect("matching manifest must validate");

    let mut unsupported = value;
    unsupported["control_schema_version"] = serde_json::json!(1);
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
fn asset_manifest_mismatch_and_direct_resource_shape_fail_closed() {
    let active_request = request();
    let manifest = WorkerAssetManifest::verified_private_copy(&active_request, 42);
    let mut mismatched = serde_json::to_value(&manifest).expect("manifest must serialize");
    mismatched["job_id"] = serde_json::json!("other-job");
    let mismatched: WorkerAssetManifest =
        serde_json::from_value(mismatched).expect("mismatched manifest remains structurally valid");
    assert!(mismatched.validate_for(&active_request).is_err());

    let mut direct = serde_json::to_value(&manifest).expect("manifest must serialize");
    direct["transport"] = serde_json::json!("unix_descriptor");
    assert!(serde_json::from_value::<WorkerAssetManifest>(direct.clone()).is_err());

    direct["worker_resource_id"] = serde_json::json!(3);
    let direct: WorkerAssetManifest =
        serde_json::from_value(direct).expect("direct transport must bind one worker resource");
    assert_eq!(direct.transport(), WorkerAssetTransport::UnixDescriptor);
    assert_eq!(direct.worker_resource_id(), Some(3));
    direct
        .validate_for(&active_request)
        .expect("matching direct manifest must validate");
}

#[test]
fn supervisor_policy_requires_an_explicit_nonzero_cancellation_grace() {
    assert!(SupervisorPolicy::new(4_096, 1, 50, 512 * 1024 * 1024, 60_000).is_ok());
    assert!(SupervisorPolicy::new(4_096, 1, 0, 512 * 1024 * 1024, 60_000).is_err());
    assert!(SupervisorPolicy::new(4_096, 1, 50, 0, 60_000).is_err());
    assert!(SupervisorPolicy::new(4_096, 1, 50, 512 * 1024 * 1024, 0).is_err());
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
        (
            WorkerTermination::WorkspaceOutputLimitExceeded,
            "WORKSPACE_OUTPUT_LIMIT_EXCEEDED",
        ),
        (
            WorkerTermination::WorkspaceInspectionFailed,
            "WORKSPACE_INSPECTION_FAILED",
        ),
        (
            WorkerTermination::ParserContainmentFailed,
            "WORKER_CONTAINMENT_FAILED",
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
        SupervisorPolicy::new(4_096, 1, 50, 512 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid");

    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm_ascii.stl");
    let mut request_value = serde_json::to_value(request()).expect("request must serialize");
    request_value["expected_source_hash"] =
        serde_json::json!("c46f940641e08eb3cbcaed5e1d90191c089651dd8d42064ecdaaa7a8b3e069ab");
    let launch_request: GeometryWorkerRequest =
        serde_json::from_value(request_value).expect("fixture request must deserialize");
    let launch_grant =
        open_local_source_read_only(launch_request.asset_capability().clone(), &source)
            .expect("fixture grant must open");
    let cancel_grant =
        open_local_source_read_only(launch_request.asset_capability().clone(), &source)
            .expect("fixture grant must open");
    let launch =
        supervisor.execute_with_grant(&launch_request, launch_grant, &AtomicBool::new(false));
    let cancelled =
        supervisor.execute_with_grant(&launch_request, cancel_grant, &AtomicBool::new(true));

    assert_eq!(
        launch.response().diagnostic_codes()[0].as_str(),
        "WORKER_LAUNCH_FAILED"
    );
    assert_eq!(
        cancelled.response().diagnostic_codes()[0].as_str(),
        "WORKER_CANCELLED"
    );
}

#[test]
fn require_direct_transport_never_creates_a_copy_fallback() {
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/cube_10mm_ascii.stl");
    let mut request_value = serde_json::to_value(request()).expect("request must serialize");
    request_value["expected_source_hash"] =
        serde_json::json!("c46f940641e08eb3cbcaed5e1d90191c089651dd8d42064ecdaaa7a8b3e069ab");
    let active_request: GeometryWorkerRequest =
        serde_json::from_value(request_value).expect("fixture request must deserialize");
    let grant = open_local_source_read_only(active_request.asset_capability().clone(), &source)
        .expect("fixture grant must open");
    let supervisor = GeometryWorkerSupervisor::new(
        PathBuf::from("__partprobe_worker_must_not_launch__"),
        std::env::temp_dir(),
        SupervisorPolicy::new(4_096, 1, 50, 512 * 1024 * 1024, 60_000)
            .expect("policy must be valid"),
    )
    .expect("supervisor must be valid")
    .with_asset_transport_policy(WorkerAssetTransportPolicy::RequireDirect);

    let execution = supervisor.execute_with_grant(&active_request, grant, &AtomicBool::new(false));

    #[cfg(any(unix, windows))]
    {
        assert_eq!(
            execution.response().diagnostic_codes()[0].as_str(),
            "WORKER_LAUNCH_FAILED"
        );
        assert_eq!(
            execution.asset_transport(),
            Some(expected_direct_transport())
        );
    }
    #[cfg(not(any(unix, windows)))]
    {
        assert_eq!(
            execution.response().diagnostic_codes()[0].as_str(),
            "ASSET_DIRECT_TRANSPORT_UNAVAILABLE"
        );
        assert!(execution.asset_transport().is_none());
    }
    assert!(execution.fallback_reason().is_none());
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
