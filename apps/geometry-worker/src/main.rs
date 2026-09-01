//! Minimal isolated worker host with an optional developer-only native geometry adapter.

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use partprobe_geometry_core::{
    GeometryStage, GeometryStageReport, GeometryWarning, GeometryWarningCode, StageStatus,
    WarningSeverity,
};
#[cfg(any(unix, windows))]
use partprobe_geometry_import::verify_worker_asset_direct;
use partprobe_geometry_import::{
    DiagnosticCode, GeometryWorkerControlMessage, GeometryWorkerRequest, GeometryWorkerResponse,
    ProvisionalMeshGeometrySnapshot, SnapshotReference, StlLimits, ThreeMfLimits,
    VerifiedWorkerAsset, WORKER_OUTPUT_FILENAME, WorkerAssetManifest, WorkerAssetTransport,
    WorkerCancellationReason, WorkerTermination, analyze_3mf, analyze_stl,
    recoverable_termination_response, verify_worker_asset_copy,
};

const MAX_CONTROL_MESSAGE_BYTES: usize = 1_048_576;
const CANCELLATION_NONE: u8 = 0;
const CANCELLATION_USER_REQUESTED: u8 = 1;
const CANCELLATION_DEADLINE_EXCEEDED: u8 = 2;
const CANCELLATION_PROTOCOL_INVALID: u8 = 3;
const SUPPORTED_STAGES: [GeometryStage; 6] = [
    GeometryStage::Intake,
    GeometryStage::Identify,
    GeometryStage::Parse,
    GeometryStage::UnitResolution,
    GeometryStage::Validation,
    GeometryStage::BasicProperties,
];
const THREE_MF_MAX_ENTRIES: usize = 16;
const MESH_SPIKE_MAX_INPUT_BYTES: usize = 64 * 1024;
const THREE_MF_MAX_EXPANDED_BYTES: u64 = 64 * 1024;
const THREE_MF_MAX_MODEL_XML_BYTES: usize = 32 * 1024;
const MESH_SPIKE_MAX_VERTICES: usize = 100;
const MESH_SPIKE_MAX_TRIANGLES: usize = 1_000;
const THREE_MF_MAX_OBJECTS: usize = 4;
const THREE_MF_MAX_COMPONENTS: usize = 3;
const THREE_MF_MAX_METADATA: usize = 8;
const THREE_MF_MAX_COMPRESSION_RATIO: u64 = 100;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::from(64),
    }
}

fn run() -> Result<(), ()> {
    let mut stdin = std::io::stdin();
    let message = read_control_message(&mut stdin)?.ok_or(())?;
    let (request, asset_manifest) = message.into_execute().ok_or(())?;
    let parser_containment = match partprobe_platform::prepare_worker_parser_containment() {
        Ok(containment) => containment,
        Err(_) => {
            return write_termination_response(
                &request,
                WorkerTermination::ParserContainmentFailed,
            );
        }
    };
    let cancellation = Arc::new(AtomicU8::new(CANCELLATION_NONE));
    let cancellation_reader = Arc::clone(&cancellation);
    let control_request = request.clone();
    std::thread::spawn(move || {
        watch_control_stream(&mut stdin, &control_request, &cancellation_reader);
    });

    let asset = match acquire_worker_asset(&request, &asset_manifest) {
        Ok(asset) => asset,
        Err(termination) => {
            return write_termination_response(&request, termination);
        }
    };
    if parser_containment.enforce().is_err() {
        return write_termination_response(&request, WorkerTermination::ParserContainmentFailed);
    }
    let response = build_response(&request, &asset, &cancellation)?;
    let response_bytes = serde_json::to_vec(&response).map_err(|_| ())?;
    std::io::stdout().write_all(&response_bytes).map_err(|_| ())
}

fn write_termination_response(
    request: &GeometryWorkerRequest,
    termination: WorkerTermination,
) -> Result<(), ()> {
    let response = recoverable_termination_response(
        request.schema_version(),
        request.job_id().clone(),
        request.correlation_id().clone(),
        termination,
    );
    let response_bytes = serde_json::to_vec(&response).map_err(|_| ())?;
    std::io::stdout().write_all(&response_bytes).map_err(|_| ())
}

fn acquire_worker_asset(
    request: &GeometryWorkerRequest,
    manifest: &WorkerAssetManifest,
) -> Result<VerifiedWorkerAsset, WorkerTermination> {
    match manifest.transport() {
        WorkerAssetTransport::VerifiedPrivateCopy => {
            let worker_directory =
                std::env::current_dir().map_err(|_| WorkerTermination::AssetTransportInvalid)?;
            verify_worker_asset_copy(request, manifest, &worker_directory)
        }
        WorkerAssetTransport::UnixDescriptor => {
            #[cfg(unix)]
            {
                let resource_id = manifest
                    .worker_resource_id()
                    .ok_or(WorkerTermination::AssetTransportInvalid)?;
                let source = partprobe_platform::take_inherited_worker_asset(resource_id)
                    .map_err(|_| WorkerTermination::AssetTransportInvalid)?;
                verify_worker_asset_direct(request, manifest, source)
            }
            #[cfg(not(unix))]
            {
                Err(WorkerTermination::AssetTransportInvalid)
            }
        }
        WorkerAssetTransport::WindowsHandle => {
            #[cfg(windows)]
            {
                let resource_id = manifest
                    .worker_resource_id()
                    .ok_or(WorkerTermination::AssetTransportInvalid)?;
                let source = partprobe_platform::take_inherited_worker_asset(resource_id)
                    .map_err(|_| WorkerTermination::AssetTransportInvalid)?;
                verify_worker_asset_direct(request, manifest, source)
            }
            #[cfg(not(windows))]
            {
                Err(WorkerTermination::AssetTransportInvalid)
            }
        }
    }
}

fn read_control_message(
    source: &mut impl Read,
) -> Result<Option<GeometryWorkerControlMessage>, ()> {
    let mut bytes = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        match source.read(&mut byte) {
            Ok(0) if bytes.is_empty() => return Ok(None),
            Ok(0) => return Err(()),
            Ok(1) if byte[0] == b'\n' => break,
            Ok(1) => {
                if bytes.len() == MAX_CONTROL_MESSAGE_BYTES {
                    return Err(());
                }
                bytes.push(byte[0]);
            }
            Ok(_) => return Err(()),
            Err(_) => return Err(()),
        }
    }
    serde_json::from_slice(&bytes).map(Some).map_err(|_| ())
}

fn watch_control_stream(
    source: &mut impl Read,
    request: &GeometryWorkerRequest,
    cancellation: &AtomicU8,
) {
    let state = match read_control_message(source)
        .and_then(|message| message.ok_or(()))
        .and_then(|message| message.cancellation_reason_for(request).map_err(|_| ()))
    {
        Ok(WorkerCancellationReason::UserRequested) => CANCELLATION_USER_REQUESTED,
        Ok(WorkerCancellationReason::DeadlineExceeded) => CANCELLATION_DEADLINE_EXCEEDED,
        Err(()) => CANCELLATION_PROTOCOL_INVALID,
    };
    let _ = cancellation.compare_exchange(
        CANCELLATION_NONE,
        state,
        Ordering::AcqRel,
        Ordering::Acquire,
    );
}

fn cancellation_response(
    request: &GeometryWorkerRequest,
    cancellation: &AtomicU8,
) -> Result<Option<GeometryWorkerResponse>, ()> {
    let diagnostic_code = match cancellation.load(Ordering::Acquire) {
        CANCELLATION_NONE => return Ok(None),
        CANCELLATION_USER_REQUESTED => "WORKER_CANCELLED",
        CANCELLATION_DEADLINE_EXCEEDED => "WORKER_TIMEOUT",
        CANCELLATION_PROTOCOL_INVALID => return Err(()),
        _ => return Err(()),
    };
    GeometryWorkerResponse::new(
        request.schema_version(),
        request.job_id().clone(),
        request.correlation_id().clone(),
        StageStatus::FailedRecoverable,
        Vec::new(),
        None,
        vec![
            DiagnosticCode::new(diagnostic_code).map_err(|_| ())?,
            DiagnosticCode::new("WORKER_CANCELLATION_ACKNOWLEDGED").map_err(|_| ())?,
        ],
    )
    .map(Some)
    .map_err(|_| ())
}

fn build_response(
    request: &GeometryWorkerRequest,
    asset: &VerifiedWorkerAsset,
    cancellation: &AtomicU8,
) -> Result<GeometryWorkerResponse, ()> {
    if request.stages() != SUPPORTED_STAGES {
        return failed_response(request, "UNSUPPORTED_STAGE_REQUEST");
    }
    if let Some(response) = cancellation_response(request, cancellation)? {
        return Ok(response);
    }
    if looks_like_step(asset.bytes()) {
        return build_step_response(request, asset, cancellation);
    }
    build_mesh_response(request, asset, cancellation)
}

fn looks_like_step(bytes: &[u8]) -> bool {
    bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .is_some_and(|start| bytes[start..].starts_with(b"ISO-10303-21;"))
}

fn looks_like_three_mf(bytes: &[u8]) -> bool {
    bytes.starts_with(b"PK\x03\x04")
        || bytes.starts_with(b"PK\x05\x06")
        || bytes.starts_with(b"PK\x07\x08")
}

fn build_mesh_response(
    request: &GeometryWorkerRequest,
    asset: &VerifiedWorkerAsset,
    cancellation: &AtomicU8,
) -> Result<GeometryWorkerResponse, ()> {
    let max_input_bytes = usize::try_from(request.quotas().max_input_bytes())
        .map_err(|_| ())?
        .min(MESH_SPIKE_MAX_INPUT_BYTES);
    let max_entities = usize::try_from(request.quotas().max_entities()).map_err(|_| ())?;
    let (snapshot, warning_codes) = if looks_like_three_mf(asset.bytes()) {
        let limits = ThreeMfLimits::new(
            max_input_bytes,
            THREE_MF_MAX_ENTRIES,
            request
                .quotas()
                .max_input_bytes()
                .min(THREE_MF_MAX_EXPANDED_BYTES),
            max_input_bytes.min(THREE_MF_MAX_MODEL_XML_BYTES),
            max_entities.min(MESH_SPIKE_MAX_VERTICES),
            max_entities.min(MESH_SPIKE_MAX_TRIANGLES),
            THREE_MF_MAX_OBJECTS,
            THREE_MF_MAX_COMPONENTS,
            THREE_MF_MAX_METADATA,
            THREE_MF_MAX_COMPRESSION_RATIO,
        )
        .map_err(|_| ())?;
        let evidence = match analyze_3mf(asset.bytes(), limits) {
            Ok(evidence) => evidence,
            Err(error) => return failed_response(request, error.diagnostic_code()),
        };
        let warning_codes = evidence.warnings().to_vec();
        (
            ProvisionalMeshGeometrySnapshot::from_three_mf(
                request.expected_source_hash().clone(),
                evidence,
            ),
            warning_codes,
        )
    } else {
        let limits = StlLimits::new(max_input_bytes, max_entities.min(MESH_SPIKE_MAX_TRIANGLES))
            .map_err(|_| ())?;
        let evidence = match analyze_stl(asset.bytes(), limits) {
            Ok(evidence) => evidence,
            Err(error) => return failed_response(request, error.diagnostic_code()),
        };
        let warning_codes = evidence.warnings().to_vec();
        (
            ProvisionalMeshGeometrySnapshot::from_stl(
                request.expected_source_hash().clone(),
                evidence,
            ),
            warning_codes,
        )
    };
    if let Some(response) = cancellation_response(request, cancellation)? {
        return Ok(response);
    }
    let warnings = warning_codes
        .into_iter()
        .map(mesh_warning)
        .collect::<Vec<_>>();
    write_snapshot_response(
        request,
        cancellation,
        serde_json::to_vec(&snapshot).map_err(|_| ())?,
        partprobe_geometry_import::PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_REFERENCE,
        warnings,
    )
}

fn mesh_warning(code: GeometryWarningCode) -> GeometryWarning {
    let stage = match code.as_str() {
        "UNITS_MISSING_REQUIRES_CONFIRMATION" => GeometryStage::UnitResolution,
        "THREE_MF_METADATA_NOT_INTERPRETED" => GeometryStage::Parse,
        _ => GeometryStage::Validation,
    };
    GeometryWarning {
        code,
        stage,
        severity: WarningSeverity::Warning,
    }
}

#[cfg(not(feature = "native-occt"))]
fn build_step_response(
    request: &GeometryWorkerRequest,
    _asset: &VerifiedWorkerAsset,
    cancellation: &AtomicU8,
) -> Result<GeometryWorkerResponse, ()> {
    if let Some(response) = cancellation_response(request, cancellation)? {
        return Ok(response);
    }
    GeometryWorkerResponse::new(
        request.schema_version(),
        request.job_id().clone(),
        request.correlation_id().clone(),
        StageStatus::FailedTerminal,
        Vec::new(),
        None,
        vec![
            DiagnosticCode::new("NATIVE_ADAPTER_UNAVAILABLE")
                .expect("static diagnostic code must be valid"),
        ],
    )
    .map_err(|_| ())
}

#[cfg(feature = "native-occt")]
fn build_step_response(
    request: &GeometryWorkerRequest,
    asset: &VerifiedWorkerAsset,
    cancellation: &AtomicU8,
) -> Result<GeometryWorkerResponse, ()> {
    if let Some(response) = cancellation_response(request, cancellation)? {
        return Ok(response);
    }

    let properties = match partprobe_geometry_occt_adapter::analyze_step_bytes_with_cancellation(
        asset.bytes(),
        &|| cancellation.load(Ordering::Acquire) != CANCELLATION_NONE,
    ) {
        Ok(properties) => properties,
        Err(error) if error.diagnostic_code() == "OCCT_CANCELLED" => {
            return cancellation_response(request, cancellation)?.ok_or(());
        }
        Err(error) => return failed_response(request, error.diagnostic_code()),
    };
    if let Some(response) = cancellation_response(request, cancellation)? {
        return Ok(response);
    }
    let snapshot = partprobe_geometry_core::ProvisionalGeometrySnapshot::new(
        request.expected_source_hash().clone(),
        "8.0.0",
        partprobe_geometry_occt_adapter::linked_abi_version(),
        properties.transferred_roots,
        properties.solid_body_count,
        provisional_decimal(properties.surface_area_mm2)?,
        provisional_decimal(properties.enclosed_volume_mm3)?,
        provisional_centroid(properties.center_of_mass_mm)?,
    )
    .map_err(|_| ())?;
    write_snapshot_response(
        request,
        cancellation,
        serde_json::to_vec(&snapshot).map_err(|_| ())?,
        partprobe_geometry_import::PROVISIONAL_GEOMETRY_SNAPSHOT_REFERENCE,
        Vec::new(),
    )
}

fn write_snapshot_response(
    request: &GeometryWorkerRequest,
    cancellation: &AtomicU8,
    snapshot_bytes: Vec<u8>,
    snapshot_reference: &str,
    warnings: Vec<GeometryWarning>,
) -> Result<GeometryWorkerResponse, ()> {
    if u64::try_from(snapshot_bytes.len()).map_err(|_| ())? > request.quotas().max_output_bytes() {
        return failed_response(request, "OUTPUT_QUOTA_EXCEEDED");
    }
    if let Some(response) = cancellation_response(request, cancellation)? {
        return Ok(response);
    }
    let output = std::env::current_dir()
        .map_err(|_| ())?
        .join(WORKER_OUTPUT_FILENAME);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .map_err(|_| ())?;
    if file.write_all(&snapshot_bytes).is_err() || file.flush().is_err() {
        let _ = std::fs::remove_file(output);
        return failed_response(request, "OUTPUT_WRITE_FAILED");
    }
    if let Some(response) = cancellation_response(request, cancellation)? {
        let _ = std::fs::remove_file(output);
        return Ok(response);
    }

    let stage_reports = request
        .stages()
        .iter()
        .copied()
        .map(|stage| {
            let stage_warnings = warnings
                .iter()
                .filter(|warning| warning.stage == stage)
                .cloned()
                .collect::<Vec<_>>();
            let status = if stage_warnings.is_empty() {
                StageStatus::Succeeded
            } else {
                StageStatus::SucceededWithWarnings
            };
            GeometryStageReport::new(stage, status, stage_warnings)
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ())?;
    let status = if warnings.is_empty() {
        StageStatus::Succeeded
    } else {
        StageStatus::SucceededWithWarnings
    };
    GeometryWorkerResponse::new(
        request.schema_version(),
        request.job_id().clone(),
        request.correlation_id().clone(),
        status,
        stage_reports,
        Some(SnapshotReference::new(snapshot_reference).map_err(|_| ())?),
        Vec::new(),
    )
    .map_err(|_| ())
}

#[cfg(feature = "native-occt")]
fn format_provisional_measurement(value: f64) -> String {
    let mut value = format!("{value:.6}");
    while value.ends_with('0') {
        value.pop();
    }
    if value.ends_with('.') {
        value.pop();
    }
    if value == "-0" {
        value.clear();
        value.push('0');
    }
    value
}

#[cfg(feature = "native-occt")]
fn provisional_decimal(
    value: f64,
) -> Result<partprobe_geometry_core::ProvisionalGeometryDecimal, ()> {
    partprobe_geometry_core::ProvisionalGeometryDecimal::new(format_provisional_measurement(value))
        .map_err(|_| ())
}

#[cfg(feature = "native-occt")]
fn provisional_centroid(
    values: [f64; 3],
) -> Result<[partprobe_geometry_core::ProvisionalGeometryDecimal; 3], ()> {
    Ok([
        provisional_decimal(values[0])?,
        provisional_decimal(values[1])?,
        provisional_decimal(values[2])?,
    ])
}

fn failed_response(
    request: &GeometryWorkerRequest,
    diagnostic_code: &str,
) -> Result<GeometryWorkerResponse, ()> {
    GeometryWorkerResponse::new(
        request.schema_version(),
        request.job_id().clone(),
        request.correlation_id().clone(),
        StageStatus::FailedRecoverable,
        Vec::new(),
        None,
        vec![DiagnosticCode::new(diagnostic_code).map_err(|_| ())?],
    )
    .map_err(|_| ())
}
