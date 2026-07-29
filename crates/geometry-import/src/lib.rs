//! Versioned, path-free protocol for the isolated geometry worker.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use partprobe_domain::{DomainError, SchemaVersion};
use partprobe_geometry_core::{
    AnalysisProfile, GeometryStage, GeometryStageReport, Sha256Digest, StageStatus,
};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

/// Fixed worker-local input name; no caller path crosses the protocol.
pub const WORKER_INPUT_FILENAME: &str = "partprobe-input.asset";
/// Fixed worker-local output name; the response carries only an opaque reference.
pub const WORKER_OUTPUT_FILENAME: &str = "partprobe-output.json";

macro_rules! protocol_token {
    ($(#[$attribute:meta])* $name:ident, $field:literal) => {
        $(#[$attribute])*
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Validates a bounded path-free protocol token.
            pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
                let value = value.into();
                if value.is_empty()
                    || value.len() > 256
                    || !value.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')
                    })
                {
                    return Err(DomainError::InvalidValue {
                        field: $field,
                        reason: "must be a 1–256 character ASCII token without path separators",
                    });
                }
                Ok(Self(value))
            }

            /// Returns the validated token.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

macro_rules! diagnostic_code {
    ($(#[$attribute:meta])* $name:ident, $field:literal) => {
        $(#[$attribute])*
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Validates and creates a bounded machine-readable code.
            pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
                let value = value.into();
                if value.is_empty()
                    || value.len() > 64
                    || !value
                        .bytes()
                        .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
                {
                    return Err(DomainError::InvalidValue {
                        field: $field,
                        reason: "must be 1–64 uppercase ASCII letters, digits, or underscores",
                    });
                }
                Ok(Self(value))
            }

            /// Returns the validated code.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

protocol_token!(
    /// Stable geometry job identity.
    GeometryJobId,
    "geometry job ID"
);
protocol_token!(
    /// Correlation identity safe for diagnostics.
    CorrelationId,
    "correlation ID"
);
protocol_token!(
    /// Opaque least-privilege capability resolved by the worker host.
    AssetCapability,
    "asset capability"
);
diagnostic_code!(
    /// Stable sanitized diagnostic identity.
    DiagnosticCode,
    "diagnostic code"
);
protocol_token!(
    /// Opaque controlled-store reference to an immutable snapshot.
    SnapshotReference,
    "snapshot reference"
);

/// Hard limits supplied to every worker job.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct ResourceQuotas {
    max_input_bytes: u64,
    max_output_bytes: u64,
    max_entities: u64,
    wall_time_millis: u64,
}

#[derive(Deserialize)]
struct ResourceQuotasWire {
    max_input_bytes: u64,
    max_output_bytes: u64,
    max_entities: u64,
    wall_time_millis: u64,
}

impl<'de> Deserialize<'de> for ResourceQuotas {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ResourceQuotasWire::deserialize(deserializer)?;
        Self::new(
            wire.max_input_bytes,
            wire.max_output_bytes,
            wire.max_entities,
            wire.wall_time_millis,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl ResourceQuotas {
    /// Validates strictly positive worker limits.
    pub fn new(
        max_input_bytes: u64,
        max_output_bytes: u64,
        max_entities: u64,
        wall_time_millis: u64,
    ) -> Result<Self, DomainError> {
        if [
            max_input_bytes,
            max_output_bytes,
            max_entities,
            wall_time_millis,
        ]
        .contains(&0)
        {
            return Err(DomainError::InvalidValue {
                field: "geometry worker quotas",
                reason: "every resource limit must be greater than zero",
            });
        }
        Ok(Self {
            max_input_bytes,
            max_output_bytes,
            max_entities,
            wall_time_millis,
        })
    }

    /// Returns the maximum accepted source size.
    #[must_use]
    pub const fn max_input_bytes(self) -> u64 {
        self.max_input_bytes
    }

    /// Returns the maximum controlled output size.
    #[must_use]
    pub const fn max_output_bytes(self) -> u64 {
        self.max_output_bytes
    }

    /// Returns the maximum parser entity count.
    #[must_use]
    pub const fn max_entities(self) -> u64 {
        self.max_entities
    }

    /// Returns the maximum wall-clock duration.
    #[must_use]
    pub const fn wall_time_millis(self) -> u64 {
        self.wall_time_millis
    }
}

/// Path-free versioned request sent to the isolated worker.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GeometryWorkerRequest {
    schema_version: SchemaVersion,
    job_id: GeometryJobId,
    correlation_id: CorrelationId,
    asset_capability: AssetCapability,
    expected_source_hash: Sha256Digest,
    stages: Vec<GeometryStage>,
    analysis_profile: AnalysisProfile,
    quotas: ResourceQuotas,
}

#[derive(Deserialize)]
struct GeometryWorkerRequestWire {
    schema_version: SchemaVersion,
    job_id: GeometryJobId,
    correlation_id: CorrelationId,
    asset_capability: AssetCapability,
    expected_source_hash: Sha256Digest,
    stages: Vec<GeometryStage>,
    analysis_profile: AnalysisProfile,
    quotas: ResourceQuotas,
}

impl<'de> Deserialize<'de> for GeometryWorkerRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = GeometryWorkerRequestWire::deserialize(deserializer)?;
        Self::new(
            wire.schema_version,
            wire.job_id,
            wire.correlation_id,
            wire.asset_capability,
            wire.expected_source_hash,
            wire.stages,
            wire.analysis_profile,
            wire.quotas,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl GeometryWorkerRequest {
    /// Validates a deterministic, path-free request.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        schema_version: SchemaVersion,
        job_id: GeometryJobId,
        correlation_id: CorrelationId,
        asset_capability: AssetCapability,
        expected_source_hash: Sha256Digest,
        stages: Vec<GeometryStage>,
        analysis_profile: AnalysisProfile,
        quotas: ResourceQuotas,
    ) -> Result<Self, DomainError> {
        if stages.is_empty() {
            return Err(DomainError::InvalidValue {
                field: "geometry worker stages",
                reason: "at least one stage is required",
            });
        }
        let unique = stages.iter().copied().collect::<BTreeSet<_>>();
        if unique.len() != stages.len() {
            return Err(DomainError::InvalidValue {
                field: "geometry worker stages",
                reason: "stages must not repeat",
            });
        }
        if !stages.windows(2).all(|pair| pair[0] < pair[1]) {
            return Err(DomainError::InvalidValue {
                field: "geometry worker stages",
                reason: "stages must follow canonical pipeline order",
            });
        }
        Ok(Self {
            schema_version,
            job_id,
            correlation_id,
            asset_capability,
            expected_source_hash,
            stages,
            analysis_profile,
            quotas,
        })
    }

    /// Returns the schema version.
    #[must_use]
    pub const fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }

    /// Returns the job identity.
    #[must_use]
    pub const fn job_id(&self) -> &GeometryJobId {
        &self.job_id
    }

    /// Returns the correlation identity.
    #[must_use]
    pub const fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }

    /// Returns the opaque input capability.
    #[must_use]
    pub const fn asset_capability(&self) -> &AssetCapability {
        &self.asset_capability
    }

    /// Returns the source hash expected before parsing.
    #[must_use]
    pub const fn expected_source_hash(&self) -> &Sha256Digest {
        &self.expected_source_hash
    }

    /// Returns the requested stages in canonical order.
    #[must_use]
    pub fn stages(&self) -> &[GeometryStage] {
        &self.stages
    }

    /// Returns the selected analysis profile.
    #[must_use]
    pub const fn analysis_profile(&self) -> &AnalysisProfile {
        &self.analysis_profile
    }

    /// Returns the hard resource quotas.
    #[must_use]
    pub const fn quotas(&self) -> ResourceQuotas {
        self.quotas
    }
}

/// Versioned response from the isolated worker.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GeometryWorkerResponse {
    schema_version: SchemaVersion,
    job_id: GeometryJobId,
    correlation_id: CorrelationId,
    status: StageStatus,
    stage_reports: Vec<GeometryStageReport>,
    snapshot_reference: Option<SnapshotReference>,
    diagnostic_codes: Vec<DiagnosticCode>,
}

#[derive(Deserialize)]
struct GeometryWorkerResponseWire {
    schema_version: SchemaVersion,
    job_id: GeometryJobId,
    correlation_id: CorrelationId,
    status: StageStatus,
    stage_reports: Vec<GeometryStageReport>,
    snapshot_reference: Option<SnapshotReference>,
    diagnostic_codes: Vec<DiagnosticCode>,
}

impl<'de> Deserialize<'de> for GeometryWorkerResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = GeometryWorkerResponseWire::deserialize(deserializer)?;
        Self::new(
            wire.schema_version,
            wire.job_id,
            wire.correlation_id,
            wire.status,
            wire.stage_reports,
            wire.snapshot_reference,
            wire.diagnostic_codes,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl GeometryWorkerResponse {
    /// Validates stage ordering and status/diagnostic consistency.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        schema_version: SchemaVersion,
        job_id: GeometryJobId,
        correlation_id: CorrelationId,
        status: StageStatus,
        stage_reports: Vec<GeometryStageReport>,
        snapshot_reference: Option<SnapshotReference>,
        diagnostic_codes: Vec<DiagnosticCode>,
    ) -> Result<Self, DomainError> {
        if !stage_reports
            .windows(2)
            .all(|pair| pair[0].stage() < pair[1].stage())
        {
            return Err(DomainError::InvalidValue {
                field: "geometry worker response stages",
                reason: "stage reports must be unique and follow canonical pipeline order",
            });
        }
        let mut codes = BTreeSet::new();
        if diagnostic_codes
            .iter()
            .any(|code| !codes.insert(code.clone()))
        {
            return Err(DomainError::InvalidValue {
                field: "geometry worker diagnostics",
                reason: "diagnostic codes must be unique",
            });
        }
        if status.permits_authoritative_output() && snapshot_reference.is_none() {
            return Err(DomainError::InvalidValue {
                field: "geometry worker response",
                reason: "successful completion requires a controlled snapshot reference",
            });
        }
        if matches!(
            status,
            StageStatus::FailedRecoverable | StageStatus::FailedTerminal
        ) && diagnostic_codes.is_empty()
        {
            return Err(DomainError::InvalidValue {
                field: "geometry worker response",
                reason: "failed completion requires a sanitized diagnostic code",
            });
        }
        Ok(Self {
            schema_version,
            job_id,
            correlation_id,
            status,
            stage_reports,
            snapshot_reference,
            diagnostic_codes,
        })
    }

    /// Returns the response schema version.
    #[must_use]
    pub const fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }

    /// Returns the job identity.
    #[must_use]
    pub const fn job_id(&self) -> &GeometryJobId {
        &self.job_id
    }

    /// Returns the correlation identity.
    #[must_use]
    pub const fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }

    /// Returns the overall status.
    #[must_use]
    pub const fn status(&self) -> StageStatus {
        self.status
    }

    /// Returns ordered stage reports.
    #[must_use]
    pub fn stage_reports(&self) -> &[GeometryStageReport] {
        &self.stage_reports
    }

    /// Returns the optional controlled snapshot reference.
    #[must_use]
    pub const fn snapshot_reference(&self) -> Option<&SnapshotReference> {
        self.snapshot_reference.as_ref()
    }

    /// Returns sanitized diagnostic identities.
    #[must_use]
    pub fn diagnostic_codes(&self) -> &[DiagnosticCode] {
        &self.diagnostic_codes
    }
}

/// Supervisor-observed termination that never propagates as a desktop crash.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkerTermination {
    /// Worker executable could not be launched.
    LaunchFailed,
    /// Worker exited unsuccessfully.
    NonzeroExit,
    /// Wall-clock deadline elapsed.
    Timeout,
    /// A configured resource quota was exceeded.
    QuotaExceeded,
    /// Worker returned a malformed or version-incompatible response.
    MalformedResponse,
    /// User or supervisor cancelled the job.
    Cancelled,
    /// Protocol input/output failed.
    ProtocolIo,
    /// Controlled source could not be staged safely.
    AssetStageFailed,
    /// Staged source bytes did not match the request.
    AssetHashMismatch,
    /// Controlled source cleanup failed.
    AssetCleanupFailed,
}

/// Converts a worker termination into a safe recoverable response.
pub fn recoverable_termination_response(
    schema_version: SchemaVersion,
    job_id: GeometryJobId,
    correlation_id: CorrelationId,
    termination: WorkerTermination,
) -> GeometryWorkerResponse {
    let code = match termination {
        WorkerTermination::LaunchFailed => "WORKER_LAUNCH_FAILED",
        WorkerTermination::NonzeroExit => "WORKER_EXIT",
        WorkerTermination::Timeout => "WORKER_TIMEOUT",
        WorkerTermination::QuotaExceeded => "WORKER_QUOTA_EXCEEDED",
        WorkerTermination::MalformedResponse => "WORKER_MALFORMED_RESPONSE",
        WorkerTermination::Cancelled => "WORKER_CANCELLED",
        WorkerTermination::ProtocolIo => "WORKER_PROTOCOL_IO",
        WorkerTermination::AssetStageFailed => "ASSET_STAGE_FAILED",
        WorkerTermination::AssetHashMismatch => "ASSET_HASH_MISMATCH",
        WorkerTermination::AssetCleanupFailed => "ASSET_CLEANUP_FAILED",
    };
    GeometryWorkerResponse::new(
        schema_version,
        job_id,
        correlation_id,
        StageStatus::FailedRecoverable,
        Vec::new(),
        None,
        vec![DiagnosticCode::new(code).expect("static diagnostic code must be valid")],
    )
    .expect("supervisor-generated response must satisfy protocol invariants")
}

/// Host-side limits for protocol messages and process polling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SupervisorPolicy {
    max_protocol_bytes: usize,
    poll_interval_millis: u64,
}

impl SupervisorPolicy {
    /// Validates protocol and polling limits.
    pub fn new(max_protocol_bytes: usize, poll_interval_millis: u64) -> Result<Self, DomainError> {
        if max_protocol_bytes == 0 || poll_interval_millis == 0 {
            return Err(DomainError::InvalidValue {
                field: "geometry supervisor policy",
                reason: "protocol and polling limits must be greater than zero",
            });
        }
        Ok(Self {
            max_protocol_bytes,
            poll_interval_millis,
        })
    }

    /// Returns the maximum request or response message size.
    #[must_use]
    pub const fn max_protocol_bytes(self) -> usize {
        self.max_protocol_bytes
    }

    /// Returns the process polling interval.
    #[must_use]
    pub const fn poll_interval_millis(self) -> u64 {
        self.poll_interval_millis
    }
}

/// Local process supervisor for the isolated geometry worker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeometryWorkerSupervisor {
    executable: PathBuf,
    working_directory: PathBuf,
    policy: SupervisorPolicy,
    native_library_directory: Option<PathBuf>,
}

impl GeometryWorkerSupervisor {
    /// Creates a supervisor for one configured worker executable.
    pub fn new(
        executable: PathBuf,
        working_directory: PathBuf,
        policy: SupervisorPolicy,
    ) -> Result<Self, DomainError> {
        if executable.as_os_str().is_empty() || working_directory.as_os_str().is_empty() {
            return Err(DomainError::InvalidValue {
                field: "geometry worker process configuration",
                reason: "executable and controlled working directory must not be empty",
            });
        }
        if !working_directory.is_dir() {
            return Err(DomainError::InvalidValue {
                field: "geometry worker working directory",
                reason: "must identify an existing controlled directory",
            });
        }
        Ok(Self {
            executable,
            working_directory,
            policy,
            native_library_directory: None,
        })
    }

    /// Adds one controlled directory containing the worker's native shared libraries.
    pub fn with_native_library_directory(
        mut self,
        native_library_directory: PathBuf,
    ) -> Result<Self, DomainError> {
        if !native_library_directory.is_dir() {
            return Err(DomainError::InvalidValue {
                field: "geometry worker native library directory",
                reason: "must identify an existing controlled directory",
            });
        }
        self.native_library_directory = Some(native_library_directory);
        Ok(self)
    }

    /// Executes one request with bounded messages, timeout, cancellation, and sanitized failures.
    #[must_use]
    pub fn execute(
        &self,
        request: &GeometryWorkerRequest,
        cancellation: &AtomicBool,
    ) -> GeometryWorkerResponse {
        if cancellation.load(Ordering::Acquire) {
            return response_for(request, WorkerTermination::Cancelled);
        }

        let request_bytes = match serde_json::to_vec(request) {
            Ok(bytes) if bytes.len() <= self.policy.max_protocol_bytes => bytes,
            _ => return response_for(request, WorkerTermination::QuotaExceeded),
        };

        let mut command = Command::new(&self.executable);
        command
            .env_clear()
            .current_dir(&self.working_directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if let Some(directory) = &self.native_library_directory {
            configure_native_library_path(&mut command, directory);
        }
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(_) => return response_for(request, WorkerTermination::LaunchFailed),
        };

        let Some(mut stdin) = child.stdin.take() else {
            terminate_and_reap(&mut child);
            return response_for(request, WorkerTermination::ProtocolIo);
        };
        let writer = thread::spawn(move || stdin.write_all(&request_bytes));

        let Some(stdout) = child.stdout.take() else {
            terminate_and_reap(&mut child);
            let _ = writer.join();
            return response_for(request, WorkerTermination::ProtocolIo);
        };
        let response_limit = self.policy.max_protocol_bytes;
        let reader = thread::spawn(move || read_bounded(stdout, response_limit));

        let timeout = Duration::from_millis(request.quotas().wall_time_millis());
        let poll_interval = Duration::from_millis(self.policy.poll_interval_millis);
        let started = Instant::now();
        let status = loop {
            if cancellation.load(Ordering::Acquire) {
                terminate_and_reap(&mut child);
                let _ = writer.join();
                let _ = reader.join();
                return response_for(request, WorkerTermination::Cancelled);
            }
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if started.elapsed() < timeout => thread::sleep(poll_interval),
                Ok(None) => {
                    terminate_and_reap(&mut child);
                    let _ = writer.join();
                    let _ = reader.join();
                    return response_for(request, WorkerTermination::Timeout);
                }
                Err(_) => {
                    terminate_and_reap(&mut child);
                    let _ = writer.join();
                    let _ = reader.join();
                    return response_for(request, WorkerTermination::ProtocolIo);
                }
            }
        };

        if !matches!(writer.join(), Ok(Ok(()))) {
            let _ = reader.join();
            return response_for(request, WorkerTermination::ProtocolIo);
        }
        let response_bytes = match reader.join() {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(ReadFailure::LimitExceeded)) => {
                return response_for(request, WorkerTermination::QuotaExceeded);
            }
            _ => return response_for(request, WorkerTermination::ProtocolIo),
        };
        if !status.success() {
            return response_for(request, WorkerTermination::NonzeroExit);
        }
        let response = match serde_json::from_slice::<GeometryWorkerResponse>(&response_bytes) {
            Ok(response) => response,
            Err(_) => return response_for(request, WorkerTermination::MalformedResponse),
        };
        if response.schema_version() != request.schema_version()
            || response.job_id() != request.job_id()
            || response.correlation_id() != request.correlation_id()
        {
            return response_for(request, WorkerTermination::MalformedResponse);
        }
        response
    }

    /// Stages one authorized source under a fixed worker-local name, executes, then removes it.
    #[must_use]
    pub fn execute_with_asset(
        &self,
        request: &GeometryWorkerRequest,
        authorized_source: &Path,
        cancellation: &AtomicBool,
    ) -> GeometryWorkerResponse {
        let staged_path = self.working_directory.join(WORKER_INPUT_FILENAME);
        let stage_result = stage_asset(request, authorized_source, &staged_path);
        if let Err(termination) = stage_result {
            return response_for(request, termination);
        }

        let response = self.execute(request, cancellation);
        if remove_staged_asset(&staged_path).is_err() {
            return response_for(request, WorkerTermination::AssetCleanupFailed);
        }
        response
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReadFailure {
    Io,
    LimitExceeded,
}

fn read_bounded(stdout: impl Read, max_protocol_bytes: usize) -> Result<Vec<u8>, ReadFailure> {
    let limit = u64::try_from(max_protocol_bytes)
        .map_err(|_| ReadFailure::LimitExceeded)?
        .saturating_add(1);
    let mut bytes = Vec::new();
    stdout
        .take(limit)
        .read_to_end(&mut bytes)
        .map_err(|_| ReadFailure::Io)?;
    if bytes.len() > max_protocol_bytes {
        return Err(ReadFailure::LimitExceeded);
    }
    Ok(bytes)
}

fn terminate_and_reap(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn configure_native_library_path(command: &mut Command, directory: &Path) {
    #[cfg(target_os = "macos")]
    command.env("DYLD_LIBRARY_PATH", directory);
    #[cfg(all(unix, not(target_os = "macos")))]
    command.env("LD_LIBRARY_PATH", directory);
    #[cfg(windows)]
    command.env("PATH", directory);
}

fn stage_asset(
    request: &GeometryWorkerRequest,
    authorized_source: &Path,
    staged_path: &Path,
) -> Result<(), WorkerTermination> {
    let source_metadata = std::fs::symlink_metadata(authorized_source)
        .map_err(|_| WorkerTermination::AssetStageFailed)?;
    if source_metadata.file_type().is_symlink()
        || !source_metadata.is_file()
        || source_metadata.len() == 0
        || source_metadata.len() > request.quotas().max_input_bytes()
    {
        return Err(WorkerTermination::AssetStageFailed);
    }

    let mut source =
        File::open(authorized_source).map_err(|_| WorkerTermination::AssetStageFailed)?;
    let mut staged = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(staged_path)
        .map_err(|_| WorkerTermination::AssetStageFailed)?;
    let copy_result = copy_and_verify(request, &mut source, &mut staged);
    drop(staged);
    if let Err(termination) = copy_result {
        let _ = std::fs::remove_file(staged_path);
        return Err(termination);
    }

    let Ok(metadata) = std::fs::metadata(staged_path) else {
        let _ = std::fs::remove_file(staged_path);
        return Err(WorkerTermination::AssetStageFailed);
    };
    let mut permissions = metadata.permissions();
    permissions.set_readonly(true);
    if std::fs::set_permissions(staged_path, permissions).is_err() {
        let _ = std::fs::remove_file(staged_path);
        return Err(WorkerTermination::AssetStageFailed);
    }
    Ok(())
}

fn copy_and_verify(
    request: &GeometryWorkerRequest,
    source: &mut File,
    staged: &mut File,
) -> Result<(), WorkerTermination> {
    let mut hasher = Sha256::new();
    let mut total = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = source
            .read(&mut buffer)
            .map_err(|_| WorkerTermination::AssetStageFailed)?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(u64::try_from(read).map_err(|_| WorkerTermination::AssetStageFailed)?)
            .ok_or(WorkerTermination::AssetStageFailed)?;
        if total > request.quotas().max_input_bytes() {
            return Err(WorkerTermination::AssetStageFailed);
        }
        hasher.update(&buffer[..read]);
        staged
            .write_all(&buffer[..read])
            .map_err(|_| WorkerTermination::AssetStageFailed)?;
    }
    staged
        .flush()
        .map_err(|_| WorkerTermination::AssetStageFailed)?;

    let mut actual_hash = String::with_capacity(64);
    for byte in hasher.finalize() {
        write!(&mut actual_hash, "{byte:02x}").map_err(|_| WorkerTermination::AssetStageFailed)?;
    }
    if actual_hash != request.expected_source_hash().as_str() {
        return Err(WorkerTermination::AssetHashMismatch);
    }
    Ok(())
}

#[cfg_attr(
    windows,
    allow(
        clippy::permissions_set_readonly_false,
        reason = "Windows requires clearing FILE_ATTRIBUTE_READONLY before deleting the staged copy"
    )
)]
fn remove_staged_asset(staged_path: &Path) -> std::io::Result<()> {
    if !staged_path.exists() {
        return Ok(());
    }
    #[cfg(windows)]
    {
        let mut permissions = std::fs::metadata(staged_path)?.permissions();
        permissions.set_readonly(false);
        std::fs::set_permissions(staged_path, permissions)?;
    }
    std::fs::remove_file(staged_path)
}

fn response_for(
    request: &GeometryWorkerRequest,
    termination: WorkerTermination,
) -> GeometryWorkerResponse {
    recoverable_termination_response(
        request.schema_version(),
        request.job_id().clone(),
        request.correlation_id().clone(),
        termination,
    )
}
