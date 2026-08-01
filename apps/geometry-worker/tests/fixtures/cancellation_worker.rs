use std::io::{BufRead, Write};
use std::time::Duration;

use partprobe_geometry_core::StageStatus;
#[cfg(any(unix, windows))]
use partprobe_geometry_import::verify_worker_asset_direct;
use partprobe_geometry_import::{
    DiagnosticCode, GeometryWorkerControlMessage, GeometryWorkerRequest, GeometryWorkerResponse,
    VerifiedWorkerAsset, WorkerAssetManifest, WorkerAssetTransport, WorkerCancellationReason,
    WorkerTermination, recoverable_termination_response, verify_worker_asset_copy,
};

fn main() {
    let mut arguments = std::env::args_os();
    let _program = arguments.next();
    if arguments.next().as_deref() == Some(std::ffi::OsStr::new("--resource-descendant")) {
        let marker = arguments.next().expect("descendant marker path must exist");
        std::thread::sleep(Duration::from_millis(750));
        let _ = std::fs::write(marker, b"escaped");
        return;
    }
    if run().is_err() {
        std::process::exit(64);
    }
}

fn run() -> Result<(), ()> {
    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();
    let execute = read_message(&mut lines)?;
    let (request, asset_manifest) = execute.into_execute().ok_or(())?;
    if let Err(termination) = acquire_worker_asset(&request, &asset_manifest) {
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

    if request.job_id().as_str().contains("resource-cpu") {
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        loop {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            std::hint::black_box(state);
        }
    }

    if request.job_id().as_str().contains("resource-memory") {
        let mut allocations = Vec::new();
        loop {
            let mut allocation = vec![0_u8; 8 * 1024 * 1024].into_boxed_slice();
            for offset in (0..allocation.len()).step_by(4_096) {
                allocation[offset] = 1;
            }
            allocations.push(allocation);
        }
    }

    if request.job_id().as_str().contains("resource-output") {
        let output = vec![b'x'; 2 * 1024 * 1024];
        std::fs::write(partprobe_geometry_import::WORKER_OUTPUT_FILENAME, output)
            .map_err(|_| ())?;
        return Err(());
    }

    if request.job_id().as_str().contains("resource-descendant") {
        let marker = resource_marker_path(&request);
        let _child = std::process::Command::new(std::env::current_exe().map_err(|_| ())?)
            .arg("--resource-descendant")
            .arg(marker)
            .spawn();
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

fn resource_marker_path(request: &GeometryWorkerRequest) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("partprobe-{}-marker", request.job_id().as_str()))
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
