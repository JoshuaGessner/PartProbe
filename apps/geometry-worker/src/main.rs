//! Minimal isolated worker host; native geometry adapters are not configured yet.

use std::io::{Read, Write};
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

#[cfg(feature = "native-occt")]
use std::fs::OpenOptions;

use partprobe_geometry_core::StageStatus;
#[cfg(feature = "native-occt")]
use partprobe_geometry_core::{GeometryStage, GeometryStageReport};
#[cfg(unix)]
use partprobe_geometry_import::verify_worker_asset_direct;
use partprobe_geometry_import::{
    DiagnosticCode, GeometryWorkerControlMessage, GeometryWorkerRequest, GeometryWorkerResponse,
    VerifiedWorkerAsset, WorkerAssetManifest, WorkerAssetTransport, WorkerCancellationReason,
    WorkerTermination, recoverable_termination_response, verify_worker_asset_copy,
};
#[cfg(feature = "native-occt")]
use partprobe_geometry_import::{SnapshotReference, WORKER_OUTPUT_FILENAME};
#[cfg(feature = "native-occt")]
use serde::Serialize;

const MAX_CONTROL_MESSAGE_BYTES: usize = 1_048_576;
const CANCELLATION_NONE: u8 = 0;
const CANCELLATION_USER_REQUESTED: u8 = 1;
const CANCELLATION_DEADLINE_EXCEEDED: u8 = 2;
const CANCELLATION_PROTOCOL_INVALID: u8 = 3;

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
    let cancellation = Arc::new(AtomicU8::new(CANCELLATION_NONE));
    let cancellation_reader = Arc::clone(&cancellation);
    let control_request = request.clone();
    std::thread::spawn(move || {
        watch_control_stream(&mut stdin, &control_request, &cancellation_reader);
    });

    let asset = match acquire_worker_asset(&request, &asset_manifest) {
        Ok(asset) => asset,
        Err(termination) => {
            let response = recoverable_termination_response(
                request.schema_version(),
                request.job_id().clone(),
                request.correlation_id().clone(),
                termination,
            );
            let response_bytes = serde_json::to_vec(&response).map_err(|_| ())?;
            return std::io::stdout().write_all(&response_bytes).map_err(|_| ());
        }
    };
    let response = build_response(&request, &asset, &cancellation)?;
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
        WorkerAssetTransport::WindowsHandle => Err(WorkerTermination::AssetTransportInvalid),
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

#[cfg(not(feature = "native-occt"))]
fn build_response(
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
#[derive(Serialize)]
struct NativeSpikeSnapshot<'a> {
    schema_version: u16,
    evidence_state: &'static str,
    source_hash: &'a str,
    representation: &'static str,
    canonical_units: &'static str,
    occt_version: &'static str,
    adapter_abi_version: u32,
    decimal_scale: u32,
    transferred_roots: u64,
    solid_body_count: u64,
    surface_area_mm2: String,
    enclosed_volume_mm3: String,
    center_of_mass_mm: [String; 3],
}

#[cfg(feature = "native-occt")]
fn build_response(
    request: &GeometryWorkerRequest,
    asset: &VerifiedWorkerAsset,
    cancellation: &AtomicU8,
) -> Result<GeometryWorkerResponse, ()> {
    const SUPPORTED_STAGES: [GeometryStage; 6] = [
        GeometryStage::Intake,
        GeometryStage::Identify,
        GeometryStage::Parse,
        GeometryStage::UnitResolution,
        GeometryStage::Validation,
        GeometryStage::BasicProperties,
    ];
    if request.stages() != SUPPORTED_STAGES {
        return failed_response(request, "UNSUPPORTED_STAGE_REQUEST");
    }
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
    let snapshot = NativeSpikeSnapshot {
        schema_version: 1,
        evidence_state: "provisional_spike",
        source_hash: request.expected_source_hash().as_str(),
        representation: "exact_brep",
        canonical_units: "millimeter",
        occt_version: "8.0.0",
        adapter_abi_version: partprobe_geometry_occt_adapter::linked_abi_version(),
        decimal_scale: 6,
        transferred_roots: properties.transferred_roots,
        solid_body_count: properties.solid_body_count,
        surface_area_mm2: format_provisional_measurement(properties.surface_area_mm2),
        enclosed_volume_mm3: format_provisional_measurement(properties.enclosed_volume_mm3),
        center_of_mass_mm: properties
            .center_of_mass_mm
            .map(format_provisional_measurement),
    };
    let snapshot_bytes = serde_json::to_vec(&snapshot).map_err(|_| ())?;
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
        .map(|stage| GeometryStageReport::new(stage, StageStatus::Succeeded, Vec::new()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ())?;
    GeometryWorkerResponse::new(
        request.schema_version(),
        request.job_id().clone(),
        request.correlation_id().clone(),
        StageStatus::Succeeded,
        stage_reports,
        Some(SnapshotReference::new("geometry-snapshot-v1").map_err(|_| ())?),
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
