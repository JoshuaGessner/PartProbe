use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use partprobe_domain::{AssetRootId, RuleVersion, SchemaVersion};
use partprobe_geometry_core::{
    AnalysisProfile, AnalysisProfileId, DisplayTessellationProfile, ExactStepEnvelopeDerivative,
    GeometryStage, ProvisionalGeometryDecimal, ProvisionalGeometrySnapshot, Sha256Digest,
    StageStatus,
};
use partprobe_geometry_import::{
    AssetCapability, ControlledGeometryResult, ControlledWorkerOutput, CorrelationId,
    DiagnosticCode, DisplaySceneArtifactReference, DisplaySceneRequest,
    GEOMETRY_WORKER_SCHEMA_VERSION, GeometryJobId, GeometryWorkerControlMessage,
    GeometryWorkerRequest, GeometryWorkerResponse, GeometryWorkerSupervisor, LocalAssetRoot,
    PROVISIONAL_EXACT_STEP_ANALYSIS_REFERENCE, ProvisionalExactStepAnalysis,
    ProvisionalMeshEvidence, ProvisionalMeshGeometrySnapshot, ResourceQuotas, SnapshotReference,
    StlLimits, SupervisorPolicy, ThreeMfLimits, WORKER_ASSET_TRANSPORT_SCHEMA_VERSION,
    WORKER_CONTROL_SCHEMA_VERSION, WorkerAssetManifest, WorkerAssetTransport,
    WorkerAssetTransportPolicy, WorkerCancellationReason, WorkerTermination, analyze_3mf,
    analyze_stl, decode_controlled_geometry_result, decode_provisional_exact_step_analysis,
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

fn display_request() -> GeometryWorkerRequest {
    let mut value = serde_json::to_value(request()).expect("request must serialize");
    value["schema_version"] = serde_json::json!(GEOMETRY_WORKER_SCHEMA_VERSION);
    let request: GeometryWorkerRequest =
        serde_json::from_value(value).expect("schema-v2 base request must deserialize");
    let decimal = |value| {
        ProvisionalGeometryDecimal::new(value).expect("display tolerance must be canonical")
    };
    request
        .with_display_scene(DisplaySceneRequest::new(
            DisplayTessellationProfile::new(decimal("0.1"), decimal("12"))
                .expect("display profile must be valid"),
        ))
        .expect("schema-v2 display request must be valid")
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
fn additive_exact_step_analysis_preserves_snapshot_v1_and_binds_envelope_source() {
    let source_hash = Sha256Digest::new("a".repeat(64)).expect("source hash must be valid");
    let decimal =
        |value| ProvisionalGeometryDecimal::new(value).expect("test decimal must be canonical");
    let snapshot = ProvisionalGeometrySnapshot::new(
        source_hash.clone(),
        "8.0.0",
        3,
        1,
        1,
        decimal("392"),
        decimal("480"),
        [decimal("6"), decimal("4"), decimal("2.5")],
    )
    .expect("snapshot must be valid");
    let envelope = ExactStepEnvelopeDerivative::new(
        source_hash.clone(),
        [decimal("12"), decimal("8"), decimal("5")],
    )
    .expect("envelope must be valid");
    let analysis =
        ProvisionalExactStepAnalysis::new(snapshot, envelope).expect("analysis must be valid");
    let output = claimed_output(
        PROVISIONAL_EXACT_STEP_ANALYSIS_REFERENCE,
        serde_json::to_vec(&analysis).expect("analysis must serialize"),
    );

    let decoded = decode_provisional_exact_step_analysis(&output, &source_hash)
        .expect("analysis and source binding must pass");
    assert_eq!(decoded.snapshot().schema_version(), 1);
    assert_eq!(decoded.snapshot().enclosed_volume_mm3(), "480");
    assert_eq!(
        decoded
            .envelope()
            .aabb_extents_mm()
            .each_ref()
            .map(|value| value.as_str()),
        ["12", "8", "5"]
    );
    assert!(matches!(
        decode_controlled_geometry_result(&output, &source_hash)
            .expect("generic decoder must retain the additive exact STEP variant"),
        ControlledGeometryResult::ExactBrepWithEnvelope(_)
    ));
    assert!(
        decode_provisional_exact_step_analysis(
            &output,
            &Sha256Digest::new("b".repeat(64)).expect("alternate hash must be valid")
        )
        .is_err()
    );
}

#[test]
fn additive_exact_step_analysis_rejects_mismatched_embedded_sources() {
    let decimal =
        |value| ProvisionalGeometryDecimal::new(value).expect("test decimal must be canonical");
    let snapshot = ProvisionalGeometrySnapshot::new(
        Sha256Digest::new("a".repeat(64)).expect("source hash must be valid"),
        "8.0.0",
        3,
        1,
        1,
        decimal("600"),
        decimal("1000"),
        [decimal("5"), decimal("5"), decimal("5")],
    )
    .expect("snapshot must be valid");
    let envelope = ExactStepEnvelopeDerivative::new(
        Sha256Digest::new("b".repeat(64)).expect("source hash must be valid"),
        [decimal("10"), decimal("10"), decimal("10")],
    )
    .expect("envelope must be valid");

    assert!(ProvisionalExactStepAnalysis::new(snapshot, envelope).is_err());
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
fn schema_v2_display_request_is_explicit_path_free_and_v1_compatible() {
    let legacy = serde_json::to_value(request()).expect("legacy request must serialize");
    assert!(legacy.get("display_scene").is_none());

    let active = display_request();
    let value = serde_json::to_value(&active).expect("display request must serialize");
    assert_eq!(value["schema_version"], GEOMETRY_WORKER_SCHEMA_VERSION);
    assert_eq!(
        value["display_scene"]["artifact_reference"],
        "geometry-display-scene-v1"
    );
    assert_eq!(
        value["display_scene"]["tessellation_profile"]["linear_deflection"],
        "0.1"
    );
    assert_eq!(
        value["display_scene"]["tessellation_profile"]["angular_deflection_degrees"],
        "12"
    );
    let wire = serde_json::to_string(&value).expect("display request must stringify");
    assert!(!wire.contains("path"));
    assert!(!wire.contains("filename"));
    let decoded: GeometryWorkerRequest =
        serde_json::from_value(value.clone()).expect("display request must round trip");
    assert_eq!(decoded, active);

    let mut forbidden_legacy_display = value.clone();
    forbidden_legacy_display["schema_version"] = serde_json::json!(1);
    assert!(serde_json::from_value::<GeometryWorkerRequest>(forbidden_legacy_display).is_err());
    let mut unknown_schema = value;
    unknown_schema["schema_version"] = serde_json::json!(3);
    assert!(serde_json::from_value::<GeometryWorkerRequest>(unknown_schema).is_err());
}

#[test]
fn schema_v2_response_keeps_analysis_and_display_references_distinct() {
    let active_request = display_request();
    let response = GeometryWorkerResponse::new(
        active_request.schema_version(),
        active_request.job_id().clone(),
        active_request.correlation_id().clone(),
        StageStatus::Succeeded,
        Vec::new(),
        Some(SnapshotReference::new("geometry-step-analysis-v1").expect("valid reference")),
        Vec::new(),
    )
    .expect("analysis response must be valid")
    .with_display_scene_reference(DisplaySceneArtifactReference::current())
    .expect("display reference must be additive");
    response
        .validate_for(&active_request)
        .expect("exact requested display reference must validate");

    let value = serde_json::to_value(&response).expect("response must serialize");
    assert_eq!(value["snapshot_reference"], "geometry-step-analysis-v1");
    assert_eq!(
        value["display_scene_reference"],
        "geometry-display-scene-v1"
    );
    let decoded: GeometryWorkerResponse =
        serde_json::from_value(value.clone()).expect("response must round trip");
    assert_eq!(decoded, response);

    let mut unknown_schema = value.clone();
    unknown_schema["schema_version"] = serde_json::json!(3);
    assert!(serde_json::from_value::<GeometryWorkerResponse>(unknown_schema).is_err());

    let mut wrong_display_reference = value;
    wrong_display_reference["display_scene_reference"] = serde_json::json!("other-display-v1");
    assert!(serde_json::from_value::<GeometryWorkerResponse>(wrong_display_reference).is_err());

    let mut base_value = serde_json::to_value(request()).expect("request must serialize");
    base_value["schema_version"] = serde_json::json!(GEOMETRY_WORKER_SCHEMA_VERSION);
    let no_display_request: GeometryWorkerRequest =
        serde_json::from_value(base_value).expect("schema-v2 base request must deserialize");
    assert!(response.validate_for(&no_display_request).is_err());
}

#[test]
fn requested_display_must_be_returned_or_explicitly_unavailable() {
    let request = display_request();
    let missing = GeometryWorkerResponse::new(
        request.schema_version(),
        request.job_id().clone(),
        request.correlation_id().clone(),
        StageStatus::Succeeded,
        Vec::new(),
        Some(SnapshotReference::new("geometry-step-analysis-v1").expect("valid reference")),
        Vec::new(),
    )
    .expect("analysis response must be valid");
    assert!(missing.validate_for(&request).is_err());

    let unavailable = GeometryWorkerResponse::new(
        request.schema_version(),
        request.job_id().clone(),
        request.correlation_id().clone(),
        StageStatus::SucceededWithWarnings,
        Vec::new(),
        Some(SnapshotReference::new("geometry-step-analysis-v1").expect("valid reference")),
        vec![DiagnosticCode::new("DISPLAY_SCENE_UNAVAILABLE").expect("valid diagnostic")],
    )
    .expect("explicitly unavailable display response must be valid");
    unavailable
        .validate_for(&request)
        .expect("explicit display unavailability must preserve analysis success");
    let mut mislabeled_unavailable =
        serde_json::to_value(&unavailable).expect("response must serialize");
    mislabeled_unavailable["status"] = serde_json::json!("succeeded");
    let mislabeled_unavailable: GeometryWorkerResponse =
        serde_json::from_value(mislabeled_unavailable).expect("response remains structural");
    assert!(mislabeled_unavailable.validate_for(&request).is_err());
    assert!(
        unavailable
            .clone()
            .with_display_scene_reference(DisplaySceneArtifactReference::current())
            .is_err()
    );

    let failed = GeometryWorkerResponse::new(
        request.schema_version(),
        request.job_id().clone(),
        request.correlation_id().clone(),
        StageStatus::FailedRecoverable,
        Vec::new(),
        None,
        vec![DiagnosticCode::new("WORKER_EXIT").expect("valid diagnostic")],
    )
    .expect("failed response must be valid");
    assert!(
        failed
            .with_display_scene_reference(DisplaySceneArtifactReference::current())
            .is_err()
    );
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
