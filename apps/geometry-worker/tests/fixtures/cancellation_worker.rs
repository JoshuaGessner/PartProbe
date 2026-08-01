use std::io::{BufRead, Write};
use std::time::Duration;

use partprobe_geometry_core::StageStatus;
use partprobe_geometry_import::{
    DiagnosticCode, GeometryWorkerControlMessage, GeometryWorkerRequest, GeometryWorkerResponse,
    WorkerCancellationReason, recoverable_termination_response, verify_worker_asset_copy,
};

fn main() {
    if run().is_err() {
        std::process::exit(64);
    }
}

fn run() -> Result<(), ()> {
    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();
    let execute = read_message(&mut lines)?;
    let (request, asset_manifest) = execute.into_execute().ok_or(())?;
    let worker_directory = std::env::current_dir().map_err(|_| ())?;
    if let Err(termination) = verify_worker_asset_copy(&request, &asset_manifest, &worker_directory)
    {
        let response = recoverable_termination_response(
            request.schema_version(),
            request.job_id().clone(),
            request.correlation_id().clone(),
            termination,
        );
        let bytes = serde_json::to_vec(&response).map_err(|_| ())?;
        return std::io::stdout().write_all(&bytes).map_err(|_| ());
    }

    if request.job_id().as_str().contains("uncooperative") {
        std::thread::sleep(Duration::from_secs(5));
        return Err(());
    }

    let cancellation = read_message(&mut lines)?;
    let reason = cancellation
        .cancellation_reason_for(&request)
        .map_err(|_| ())?;
    let diagnostic = match reason {
        WorkerCancellationReason::UserRequested => "WORKER_CANCELLED",
        WorkerCancellationReason::DeadlineExceeded => "WORKER_TIMEOUT",
    };
    write_response(
        &request,
        diagnostic,
        !request.job_id().as_str().contains("unacknowledged"),
    )
}

fn read_message(
    lines: &mut impl Iterator<Item = Result<String, std::io::Error>>,
) -> Result<GeometryWorkerControlMessage, ()> {
    let line = lines.next().ok_or(())?.map_err(|_| ())?;
    if line.len() > 65_536 {
        return Err(());
    }
    serde_json::from_str(&line).map_err(|_| ())
}

fn write_response(
    request: &GeometryWorkerRequest,
    diagnostic: &str,
    acknowledge: bool,
) -> Result<(), ()> {
    let mut diagnostics = vec![DiagnosticCode::new(diagnostic).map_err(|_| ())?];
    if acknowledge {
        diagnostics.push(DiagnosticCode::new("WORKER_CANCELLATION_ACKNOWLEDGED").map_err(|_| ())?);
    }
    let response = GeometryWorkerResponse::new(
        request.schema_version(),
        request.job_id().clone(),
        request.correlation_id().clone(),
        StageStatus::FailedRecoverable,
        Vec::new(),
        None,
        diagnostics,
    )
    .map_err(|_| ())?;
    let bytes = serde_json::to_vec(&response).map_err(|_| ())?;
    std::io::stdout().write_all(&bytes).map_err(|_| ())
}
