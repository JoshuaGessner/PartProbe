//! Minimal isolated worker host; native geometry adapters are not configured yet.

use std::io::{Read, Write};
use std::process::ExitCode;

#[cfg(feature = "native-occt")]
use std::fs::OpenOptions;

use partprobe_geometry_core::StageStatus;
#[cfg(feature = "native-occt")]
use partprobe_geometry_core::{GeometryStage, GeometryStageReport};
use partprobe_geometry_import::{DiagnosticCode, GeometryWorkerRequest, GeometryWorkerResponse};
#[cfg(feature = "native-occt")]
use partprobe_geometry_import::{SnapshotReference, WORKER_INPUT_FILENAME, WORKER_OUTPUT_FILENAME};
#[cfg(feature = "native-occt")]
use serde::Serialize;

const MAX_REQUEST_BYTES: u64 = 1_048_576;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::from(64),
    }
}

fn run() -> Result<(), ()> {
    let mut request_bytes = Vec::new();
    std::io::stdin()
        .take(MAX_REQUEST_BYTES + 1)
        .read_to_end(&mut request_bytes)
        .map_err(|_| ())?;
    if request_bytes.len() > usize::try_from(MAX_REQUEST_BYTES).map_err(|_| ())? {
        return Err(());
    }
    let request: GeometryWorkerRequest = serde_json::from_slice(&request_bytes).map_err(|_| ())?;
    let response = build_response(&request)?;
    let response_bytes = serde_json::to_vec(&response).map_err(|_| ())?;
    std::io::stdout().write_all(&response_bytes).map_err(|_| ())
}

#[cfg(not(feature = "native-occt"))]
fn build_response(request: &GeometryWorkerRequest) -> Result<GeometryWorkerResponse, ()> {
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
fn build_response(request: &GeometryWorkerRequest) -> Result<GeometryWorkerResponse, ()> {
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

    let source = std::env::current_dir()
        .map_err(|_| ())?
        .join(WORKER_INPUT_FILENAME);
    let metadata = std::fs::symlink_metadata(&source).map_err(|_| ())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > request.quotas().max_input_bytes()
    {
        return failed_response(request, "ASSET_GRANT_INVALID");
    }

    let properties = match partprobe_geometry_occt_adapter::analyze_step(&source) {
        Ok(properties) => properties,
        Err(error) => return failed_response(request, error.diagnostic_code()),
    };
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
