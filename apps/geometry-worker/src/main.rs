//! Minimal isolated worker host; native geometry adapters are not configured yet.

use std::io::{Read, Write};
use std::process::ExitCode;

use partprobe_geometry_core::StageStatus;
use partprobe_geometry_import::{DiagnosticCode, GeometryWorkerRequest, GeometryWorkerResponse};

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
    let response = GeometryWorkerResponse::new(
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
    .map_err(|_| ())?;
    let response_bytes = serde_json::to_vec(&response).map_err(|_| ())?;
    std::io::stdout().write_all(&response_bytes).map_err(|_| ())
}
