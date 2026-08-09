//! Versioned, path-free protocol for the isolated geometry worker.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use partprobe_domain::{AssetRootId, DomainError, SchemaVersion};
use partprobe_geometry_core::{AnalysisProfile, GeometryStage, GeometryStageReport, StageStatus};
pub use partprobe_geometry_core::{ProvisionalGeometrySnapshot, Sha256Digest};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

/// Fixed worker-local input name; no caller path crosses the protocol.
pub const WORKER_INPUT_FILENAME: &str = "partprobe-input.asset";
/// Fixed worker-local output name; the response carries only an opaque reference.
pub const WORKER_OUTPUT_FILENAME: &str = "partprobe-output.json";
/// Current schema for the supervisor-to-worker control stream.
pub const WORKER_CONTROL_SCHEMA_VERSION: u16 = 2;
/// Current schema for the process-launch asset transport manifest.
pub const WORKER_ASSET_TRANSPORT_SCHEMA_VERSION: u16 = 2;
/// Opaque reference used only by the current developer-only native snapshot schema.
pub const PROVISIONAL_GEOMETRY_SNAPSHOT_REFERENCE: &str = "geometry-snapshot-v1";

static JOB_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(1);

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

/// Platform-neutral description of how one authorized asset reaches the worker.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerAssetTransport {
    /// A Unix file descriptor delivered through an allowlisted launch channel.
    UnixDescriptor,
    /// A Windows handle delivered through an allowlisted launch channel.
    WindowsHandle,
    /// A supervisor-created, hash-verified copy in the private job workspace.
    VerifiedPrivateCopy,
}

/// Deployment choice for worker asset transport.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkerAssetTransportPolicy {
    /// Use only the currently implemented verified-copy transport.
    VerifiedCopyOnly,
    /// Prefer direct transport, recording an explicit fallback when unavailable.
    PreferDirect,
    /// Fail closed unless direct transport is implemented and allowlisted.
    RequireDirect,
}

/// Why an execution used a fallback rather than a preferred direct transport.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkerAssetFallbackReason {
    /// No reviewed launcher can yet restrict inherited descriptors or handles to an allowlist.
    DirectTransportUnavailable,
}

/// Versioned, path-free binding for the exact asset prepared for one worker launch.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WorkerAssetManifest {
    transport_schema_version: u16,
    transport: WorkerAssetTransport,
    job_id: GeometryJobId,
    correlation_id: CorrelationId,
    asset_capability: AssetCapability,
    authorized_byte_length: u64,
    expected_source_hash: Sha256Digest,
    worker_resource_id: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerAssetManifestWire {
    transport_schema_version: u16,
    transport: WorkerAssetTransport,
    job_id: GeometryJobId,
    correlation_id: CorrelationId,
    asset_capability: AssetCapability,
    authorized_byte_length: u64,
    expected_source_hash: Sha256Digest,
    worker_resource_id: Option<u64>,
}

impl<'de> Deserialize<'de> for WorkerAssetManifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = WorkerAssetManifestWire::deserialize(deserializer)?;
        if wire.transport_schema_version != WORKER_ASSET_TRANSPORT_SCHEMA_VERSION {
            return Err(serde::de::Error::custom(
                "unsupported geometry worker asset transport schema version",
            ));
        }
        if wire.authorized_byte_length == 0 {
            return Err(serde::de::Error::custom(
                "geometry worker asset length must be nonzero",
            ));
        }
        match (wire.transport, wire.worker_resource_id) {
            (WorkerAssetTransport::VerifiedPrivateCopy, None)
            | (
                WorkerAssetTransport::UnixDescriptor | WorkerAssetTransport::WindowsHandle,
                Some(1..),
            ) => {}
            _ => {
                return Err(serde::de::Error::custom(
                    "geometry worker resource ID must be absent for copies and nonzero for direct transport",
                ));
            }
        }
        Ok(Self {
            transport_schema_version: wire.transport_schema_version,
            transport: wire.transport,
            job_id: wire.job_id,
            correlation_id: wire.correlation_id,
            asset_capability: wire.asset_capability,
            authorized_byte_length: wire.authorized_byte_length,
            expected_source_hash: wire.expected_source_hash,
            worker_resource_id: wire.worker_resource_id,
        })
    }
}

impl WorkerAssetManifest {
    /// Creates the explicit verified-private-copy binding for one request.
    #[must_use]
    pub fn verified_private_copy(request: &GeometryWorkerRequest, byte_length: u64) -> Self {
        Self {
            transport_schema_version: WORKER_ASSET_TRANSPORT_SCHEMA_VERSION,
            transport: WorkerAssetTransport::VerifiedPrivateCopy,
            job_id: request.job_id().clone(),
            correlation_id: request.correlation_id().clone(),
            asset_capability: request.asset_capability().clone(),
            authorized_byte_length: byte_length,
            expected_source_hash: request.expected_source_hash().clone(),
            worker_resource_id: None,
        }
    }

    fn direct(
        request: &GeometryWorkerRequest,
        byte_length: u64,
        transport: WorkerAssetTransport,
        worker_resource_id: u64,
    ) -> Self {
        debug_assert!(matches!(
            transport,
            WorkerAssetTransport::UnixDescriptor | WorkerAssetTransport::WindowsHandle
        ));
        debug_assert_ne!(worker_resource_id, 0);
        Self {
            transport_schema_version: WORKER_ASSET_TRANSPORT_SCHEMA_VERSION,
            transport,
            job_id: request.job_id().clone(),
            correlation_id: request.correlation_id().clone(),
            asset_capability: request.asset_capability().clone(),
            authorized_byte_length: byte_length,
            expected_source_hash: request.expected_source_hash().clone(),
            worker_resource_id: Some(worker_resource_id),
        }
    }

    /// Returns the asset transport contract version.
    #[must_use]
    pub const fn transport_schema_version(&self) -> u16 {
        self.transport_schema_version
    }

    /// Returns the explicitly selected transport.
    #[must_use]
    pub const fn transport(&self) -> WorkerAssetTransport {
        self.transport
    }

    /// Returns the byte length captured from the open supervisor grant.
    #[must_use]
    pub const fn authorized_byte_length(&self) -> u64 {
        self.authorized_byte_length
    }

    /// Returns the process-local descriptor or handle identifier for direct transport.
    #[must_use]
    pub const fn worker_resource_id(&self) -> Option<u64> {
        self.worker_resource_id
    }

    /// Validates that this launch manifest belongs to exactly one request.
    pub fn validate_for(&self, request: &GeometryWorkerRequest) -> Result<(), DomainError> {
        if self.job_id != *request.job_id()
            || self.correlation_id != *request.correlation_id()
            || self.asset_capability != *request.asset_capability()
            || self.expected_source_hash != *request.expected_source_hash()
        {
            return Err(DomainError::InvalidValue {
                field: "geometry worker asset manifest identity",
                reason: "job, correlation, capability, and source hash must match the request",
            });
        }
        if self.authorized_byte_length == 0
            || self.authorized_byte_length > request.quotas().max_input_bytes()
        {
            return Err(DomainError::InvalidValue {
                field: "geometry worker asset manifest length",
                reason: "authorized byte length must be nonzero and within the request quota",
            });
        }
        match (self.transport, self.worker_resource_id) {
            (WorkerAssetTransport::VerifiedPrivateCopy, None)
            | (
                WorkerAssetTransport::UnixDescriptor | WorkerAssetTransport::WindowsHandle,
                Some(1..),
            ) => {}
            _ => {
                return Err(DomainError::InvalidValue {
                    field: "geometry worker asset resource",
                    reason: "resource ID must be absent for copies and nonzero for direct transport",
                });
            }
        }
        Ok(())
    }
}

/// Reason a running worker is asked to stop cooperatively.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerCancellationReason {
    /// A user or application owner cancelled the job.
    UserRequested,
    /// The request's wall-clock deadline elapsed.
    DeadlineExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "message", rename_all = "snake_case")]
enum WorkerControlCommand {
    Execute {
        request: Box<GeometryWorkerRequest>,
        asset_manifest: WorkerAssetManifest,
    },
    Cancel {
        job_id: GeometryJobId,
        correlation_id: CorrelationId,
        reason: WorkerCancellationReason,
    },
}

#[derive(Deserialize)]
#[serde(tag = "message", rename_all = "snake_case", deny_unknown_fields)]
enum WorkerControlCommandWire {
    Execute {
        request: Box<GeometryWorkerRequest>,
        asset_manifest: WorkerAssetManifest,
    },
    Cancel {
        job_id: GeometryJobId,
        correlation_id: CorrelationId,
        reason: WorkerCancellationReason,
    },
}

/// One validated frame on the bounded supervisor-to-worker control stream.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GeometryWorkerControlMessage {
    control_schema_version: u16,
    #[serde(flatten)]
    command: WorkerControlCommand,
}

#[derive(Deserialize)]
struct GeometryWorkerControlMessageWire {
    control_schema_version: u16,
    #[serde(flatten)]
    command: WorkerControlCommandWire,
}

impl<'de> Deserialize<'de> for GeometryWorkerControlMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = GeometryWorkerControlMessageWire::deserialize(deserializer)?;
        if wire.control_schema_version != WORKER_CONTROL_SCHEMA_VERSION {
            return Err(serde::de::Error::custom(
                "unsupported geometry worker control schema version",
            ));
        }
        let command = match wire.command {
            WorkerControlCommandWire::Execute {
                request,
                asset_manifest,
            } => WorkerControlCommand::Execute {
                request,
                asset_manifest,
            },
            WorkerControlCommandWire::Cancel {
                job_id,
                correlation_id,
                reason,
            } => WorkerControlCommand::Cancel {
                job_id,
                correlation_id,
                reason,
            },
        };
        Ok(Self {
            control_schema_version: wire.control_schema_version,
            command,
        })
    }
}

impl GeometryWorkerControlMessage {
    /// Creates the first control-stream frame for one validated request.
    #[must_use]
    pub fn execute(request: GeometryWorkerRequest, asset_manifest: WorkerAssetManifest) -> Self {
        Self {
            control_schema_version: WORKER_CONTROL_SCHEMA_VERSION,
            command: WorkerControlCommand::Execute {
                request: Box::new(request),
                asset_manifest,
            },
        }
    }

    /// Creates a cancellation frame bound to the request identity.
    #[must_use]
    pub fn cancel(request: &GeometryWorkerRequest, reason: WorkerCancellationReason) -> Self {
        Self {
            control_schema_version: WORKER_CONTROL_SCHEMA_VERSION,
            command: WorkerControlCommand::Cancel {
                job_id: request.job_id().clone(),
                correlation_id: request.correlation_id().clone(),
                reason,
            },
        }
    }

    /// Returns the control protocol schema version.
    #[must_use]
    pub const fn control_schema_version(&self) -> u16 {
        self.control_schema_version
    }

    /// Consumes an execute frame and returns its request plus asset binding.
    #[must_use]
    pub fn into_execute(self) -> Option<(GeometryWorkerRequest, WorkerAssetManifest)> {
        match self.command {
            WorkerControlCommand::Execute {
                request,
                asset_manifest,
            } => Some((*request, asset_manifest)),
            WorkerControlCommand::Cancel { .. } => None,
        }
    }

    /// Validates a cancellation frame against the running request.
    pub fn cancellation_reason_for(
        &self,
        request: &GeometryWorkerRequest,
    ) -> Result<WorkerCancellationReason, DomainError> {
        let WorkerControlCommand::Cancel {
            job_id,
            correlation_id,
            reason,
        } = &self.command
        else {
            return Err(DomainError::InvalidValue {
                field: "geometry worker control message",
                reason: "a running worker accepts only a cancellation frame",
            });
        };
        if job_id != request.job_id() || correlation_id != request.correlation_id() {
            return Err(DomainError::InvalidValue {
                field: "geometry worker cancellation identity",
                reason: "job and correlation IDs must match the running request",
            });
        }
        Ok(*reason)
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
    /// A cancelled worker did not exit within the configured grace interval.
    CancellationGraceExceeded,
    /// A timed-out worker did not exit within the configured grace interval.
    TimeoutGraceExceeded,
    /// Protocol input/output failed.
    ProtocolIo,
    /// Controlled source could not be staged safely.
    AssetStageFailed,
    /// Open source grant did not match the request's opaque capability.
    AssetGrantMismatch,
    /// Worker asset manifest did not match the request identity or source contract.
    AssetManifestMismatch,
    /// Requested direct asset transport is unavailable and fallback was forbidden.
    AssetDirectTransportUnavailable,
    /// Worker could not validate the explicitly selected asset transport.
    AssetTransportInvalid,
    /// Staged source bytes did not match the request.
    AssetHashMismatch,
    /// Controlled source cleanup failed.
    AssetCleanupFailed,
    /// A private per-job workspace could not be created.
    WorkspacePrepareFailed,
    /// Worker output could not be claimed and validated.
    OutputClaimFailed,
    /// Worker output could not be removed from the filesystem namespace.
    OutputCleanupFailed,
    /// The private per-job workspace could not be removed.
    WorkspaceCleanupFailed,
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
        WorkerTermination::CancellationGraceExceeded => "WORKER_CANCEL_FORCE_TERMINATED",
        WorkerTermination::TimeoutGraceExceeded => "WORKER_TIMEOUT_FORCE_TERMINATED",
        WorkerTermination::ProtocolIo => "WORKER_PROTOCOL_IO",
        WorkerTermination::AssetStageFailed => "ASSET_STAGE_FAILED",
        WorkerTermination::AssetGrantMismatch => "ASSET_GRANT_MISMATCH",
        WorkerTermination::AssetManifestMismatch => "ASSET_MANIFEST_MISMATCH",
        WorkerTermination::AssetDirectTransportUnavailable => "ASSET_DIRECT_TRANSPORT_UNAVAILABLE",
        WorkerTermination::AssetTransportInvalid => "ASSET_TRANSPORT_INVALID",
        WorkerTermination::AssetHashMismatch => "ASSET_HASH_MISMATCH",
        WorkerTermination::AssetCleanupFailed => "ASSET_CLEANUP_FAILED",
        WorkerTermination::WorkspacePrepareFailed => "WORKSPACE_PREPARE_FAILED",
        WorkerTermination::OutputClaimFailed => "OUTPUT_CLAIM_FAILED",
        WorkerTermination::OutputCleanupFailed => "OUTPUT_CLEANUP_FAILED",
        WorkerTermination::WorkspaceCleanupFailed => "WORKSPACE_CLEANUP_FAILED",
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
    cancellation_grace_millis: u64,
    max_worker_memory_bytes: NonZeroU64,
    max_worker_cpu_millis: NonZeroU64,
}

impl SupervisorPolicy {
    /// Validates protocol and polling limits.
    pub fn new(
        max_protocol_bytes: usize,
        poll_interval_millis: u64,
        cancellation_grace_millis: u64,
        max_worker_memory_bytes: u64,
        max_worker_cpu_millis: u64,
    ) -> Result<Self, DomainError> {
        let Some(max_worker_memory_bytes) = NonZeroU64::new(max_worker_memory_bytes) else {
            return Err(DomainError::InvalidValue {
                field: "geometry supervisor policy",
                reason: "worker memory limit must be greater than zero",
            });
        };
        let Some(max_worker_cpu_millis) = NonZeroU64::new(max_worker_cpu_millis) else {
            return Err(DomainError::InvalidValue {
                field: "geometry supervisor policy",
                reason: "worker CPU limit must be greater than zero",
            });
        };
        if max_protocol_bytes == 0 || poll_interval_millis == 0 || cancellation_grace_millis == 0 {
            return Err(DomainError::InvalidValue {
                field: "geometry supervisor policy",
                reason: "protocol, polling, and cancellation-grace limits must be greater than zero",
            });
        }
        Ok(Self {
            max_protocol_bytes,
            poll_interval_millis,
            cancellation_grace_millis,
            max_worker_memory_bytes,
            max_worker_cpu_millis,
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

    /// Returns the cooperative cancellation grace interval.
    #[must_use]
    pub const fn cancellation_grace_millis(self) -> u64 {
        self.cancellation_grace_millis
    }

    /// Returns the hard per-worker memory ceiling configured by the deployment.
    #[must_use]
    pub const fn max_worker_memory_bytes(self) -> u64 {
        self.max_worker_memory_bytes.get()
    }

    /// Returns the hard per-worker CPU-time ceiling configured by the deployment.
    #[must_use]
    pub const fn max_worker_cpu_millis(self) -> u64 {
        self.max_worker_cpu_millis.get()
    }
}

/// One-job read grant that binds an opaque capability to an already-open source file.
#[derive(Debug)]
#[must_use = "asset read grants must be consumed by one supervisor execution"]
pub struct AssetReadGrant {
    asset_capability: AssetCapability,
    source: File,
    authorized_byte_length: u64,
}

impl AssetReadGrant {
    /// Validates an already-open regular, nonempty source without resolving a pathname.
    pub fn new(asset_capability: AssetCapability, source: File) -> Result<Self, DomainError> {
        let metadata = source.metadata().map_err(|_| DomainError::InvalidValue {
            field: "geometry asset read grant",
            reason: "source metadata must be readable from the open handle",
        })?;
        if !metadata.is_file() || metadata.len() == 0 {
            return Err(DomainError::InvalidValue {
                field: "geometry asset read grant",
                reason: "source handle must identify a nonempty regular file",
            });
        }
        Ok(Self {
            asset_capability,
            source,
            authorized_byte_length: metadata.len(),
        })
    }

    /// Returns the opaque capability bound to this grant.
    #[must_use]
    pub const fn asset_capability(&self) -> &AssetCapability {
        &self.asset_capability
    }

    /// Returns the source length captured when the grant was created.
    #[must_use]
    pub const fn authorized_byte_length(&self) -> u64 {
        self.authorized_byte_length
    }

    /// Computes a bounded SHA-256 fingerprint from the already-open authorized source.
    ///
    /// The source is rewound before and after fingerprinting so the same one-use grant can be
    /// passed directly to the geometry supervisor. The worker still independently verifies the
    /// exact length and digest before parsing.
    pub fn fingerprint_sha256(
        &mut self,
        max_input_bytes: u64,
    ) -> Result<Sha256Digest, DomainError> {
        if max_input_bytes == 0 || self.authorized_byte_length > max_input_bytes {
            return Err(DomainError::InvalidValue {
                field: "geometry asset fingerprint",
                reason: "authorized source exceeds the nonzero fingerprint limit",
            });
        }

        self.source
            .seek(SeekFrom::Start(0))
            .map_err(|_| DomainError::InvalidValue {
                field: "geometry asset fingerprint",
                reason: "authorized source could not be rewound",
            })?;

        let fingerprint = fingerprint_open_source(
            &mut self.source,
            self.authorized_byte_length,
            max_input_bytes,
        );
        let rewind = self.source.seek(SeekFrom::Start(0));
        if rewind.is_err() {
            return Err(DomainError::InvalidValue {
                field: "geometry asset fingerprint",
                reason: "authorized source could not be rewound after fingerprinting",
            });
        }
        fingerprint
    }
}

fn fingerprint_open_source(
    source: &mut File,
    authorized_byte_length: u64,
    max_input_bytes: u64,
) -> Result<Sha256Digest, DomainError> {
    let mut hasher = Sha256::new();
    let mut total = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = source
            .read(&mut buffer)
            .map_err(|_| DomainError::InvalidValue {
                field: "geometry asset fingerprint",
                reason: "authorized source could not be read",
            })?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(u64::try_from(read).map_err(|_| DomainError::InvalidValue {
                field: "geometry asset fingerprint",
                reason: "authorized source length overflowed",
            })?)
            .ok_or(DomainError::InvalidValue {
                field: "geometry asset fingerprint",
                reason: "authorized source length overflowed",
            })?;
        if total > max_input_bytes {
            return Err(DomainError::InvalidValue {
                field: "geometry asset fingerprint",
                reason: "authorized source exceeds the fingerprint limit",
            });
        }
        hasher.update(&buffer[..read]);
    }
    if total != authorized_byte_length {
        return Err(DomainError::InvalidValue {
            field: "geometry asset fingerprint",
            reason: "authorized source length changed during fingerprinting",
        });
    }

    let mut digest = String::with_capacity(64);
    for byte in hasher.finalize() {
        write!(&mut digest, "{byte:02x}").expect("writing to a String cannot fail");
    }
    Sha256Digest::new(digest)
}

/// Immutable worker-owned bytes verified against one launch manifest and request.
#[derive(Debug)]
#[must_use = "verified worker asset bytes must be consumed by the configured adapter"]
pub struct VerifiedWorkerAsset {
    transport: WorkerAssetTransport,
    content_hash: Sha256Digest,
    bytes: Box<[u8]>,
}

impl VerifiedWorkerAsset {
    /// Returns the explicit transport that supplied these bytes.
    #[must_use]
    pub const fn transport(&self) -> WorkerAssetTransport {
        self.transport
    }

    /// Returns the independently recomputed worker-side digest.
    #[must_use]
    pub const fn content_hash(&self) -> &Sha256Digest {
        &self.content_hash
    }

    /// Returns the exact immutable bytes that adapters must parse.
    #[must_use]
    pub const fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Pathless, read-only worker output claimed and verified by the supervisor.
#[derive(Debug)]
#[must_use = "controlled worker output must be persisted or deliberately discarded"]
pub struct ControlledWorkerOutput {
    snapshot_reference: SnapshotReference,
    content_hash: Sha256Digest,
    byte_length: u64,
    bytes: Box<[u8]>,
}

impl ControlledWorkerOutput {
    /// Revalidates claimed immutable output parts for alternate supervisor transports and tests.
    pub fn from_claimed_parts(
        snapshot_reference: SnapshotReference,
        content_hash: Sha256Digest,
        bytes: Box<[u8]>,
    ) -> Result<Self, DomainError> {
        if bytes.is_empty() {
            return Err(DomainError::InvalidValue {
                field: "controlled worker output",
                reason: "must not be empty",
            });
        }
        let byte_length = u64::try_from(bytes.len()).map_err(|_| DomainError::InvalidValue {
            field: "controlled worker output",
            reason: "byte length exceeds the supported range",
        })?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let mut actual_hash = String::with_capacity(64);
        for byte in hasher.finalize() {
            write!(&mut actual_hash, "{byte:02x}").expect("writing to a String cannot fail");
        }
        if actual_hash != content_hash.as_str() {
            return Err(DomainError::InvalidValue {
                field: "controlled worker output",
                reason: "claimed content hash must match the supplied bytes",
            });
        }
        Ok(Self {
            snapshot_reference,
            content_hash,
            byte_length,
            bytes,
        })
    }

    /// Returns the opaque snapshot identity bound to these bytes.
    #[must_use]
    pub const fn snapshot_reference(&self) -> &SnapshotReference {
        &self.snapshot_reference
    }

    /// Returns the supervisor-computed SHA-256 digest.
    #[must_use]
    pub const fn content_hash(&self) -> &Sha256Digest {
        &self.content_hash
    }

    /// Returns the verified byte length.
    #[must_use]
    pub const fn byte_length(&self) -> u64 {
        self.byte_length
    }

    /// Returns the supervisor-owned immutable bytes.
    #[must_use]
    pub const fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Decodes the current provisional native snapshot and binds it to the expected source.
pub fn decode_provisional_geometry_snapshot(
    output: &ControlledWorkerOutput,
    expected_source_hash: &Sha256Digest,
) -> Result<ProvisionalGeometrySnapshot, DomainError> {
    if output.snapshot_reference().as_str() != PROVISIONAL_GEOMETRY_SNAPSHOT_REFERENCE {
        return Err(DomainError::InvalidValue {
            field: "provisional geometry snapshot",
            reason: "snapshot reference does not identify the provisional geometry schema",
        });
    }
    let snapshot: ProvisionalGeometrySnapshot =
        serde_json::from_slice(output.bytes()).map_err(|_| DomainError::InvalidValue {
            field: "provisional geometry snapshot",
            reason: "bytes must satisfy the versioned provisional geometry schema",
        })?;
    if snapshot.source_hash() != expected_source_hash {
        return Err(DomainError::InvalidValue {
            field: "provisional geometry snapshot",
            reason: "snapshot source hash must match the authorized source",
        });
    }
    Ok(snapshot)
}

/// One supervised execution with an optional pathless controlled output.
#[derive(Debug)]
#[must_use = "worker execution results and controlled output must be inspected"]
pub struct GeometryWorkerExecution {
    response: GeometryWorkerResponse,
    output: Option<ControlledWorkerOutput>,
    asset_transport: Option<WorkerAssetTransport>,
    fallback_reason: Option<WorkerAssetFallbackReason>,
}

impl GeometryWorkerExecution {
    fn new(
        response: GeometryWorkerResponse,
        output: Option<ControlledWorkerOutput>,
        asset_transport: Option<WorkerAssetTransport>,
        fallback_reason: Option<WorkerAssetFallbackReason>,
    ) -> Self {
        Self {
            response,
            output,
            asset_transport,
            fallback_reason,
        }
    }

    /// Returns the validated worker response.
    #[must_use]
    pub const fn response(&self) -> &GeometryWorkerResponse {
        &self.response
    }

    /// Returns output claimed for the response's optional snapshot reference.
    #[must_use]
    pub const fn output(&self) -> Option<&ControlledWorkerOutput> {
        self.output.as_ref()
    }

    /// Returns the transport selected for this execution, when preparation reached that stage.
    #[must_use]
    pub const fn asset_transport(&self) -> Option<WorkerAssetTransport> {
        self.asset_transport
    }

    /// Returns the explicit reason a preferred direct transport used a fallback.
    #[must_use]
    pub const fn fallback_reason(&self) -> Option<WorkerAssetFallbackReason> {
        self.fallback_reason
    }

    /// Transfers the response and optional controlled output.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        GeometryWorkerResponse,
        Option<ControlledWorkerOutput>,
        Option<WorkerAssetTransport>,
        Option<WorkerAssetFallbackReason>,
    ) {
        (
            self.response,
            self.output,
            self.asset_transport,
            self.fallback_reason,
        )
    }
}

/// Capability-scoped local directory used to resolve asset paths without escaping its root.
///
/// Opening this root opts into ambient filesystem authority exactly once. The application
/// service must authorize the root, actor, project, classification, and asset capability before
/// constructing or using it. This type enforces filesystem containment; it does not make an
/// authorization decision.
#[derive(Debug)]
pub struct LocalAssetRoot {
    asset_root_id: AssetRootId,
    directory: cap_std::fs::Dir,
}

impl LocalAssetRoot {
    /// Opens an application-authorized directory as the root of subsequent relative lookups.
    pub fn open(asset_root_id: AssetRootId, root_path: &Path) -> Result<Self, DomainError> {
        let directory = cap_std::fs::Dir::open_ambient_dir(root_path, ambient_authority())
            .map_err(|_| DomainError::InvalidValue {
                field: "geometry local asset root",
                reason: "authorized root must identify an accessible directory",
            })?;
        Ok(Self {
            asset_root_id,
            directory,
        })
    }

    /// Returns the stable identity bound to this open directory capability.
    #[must_use]
    pub const fn asset_root_id(&self) -> &AssetRootId {
        &self.asset_root_id
    }

    /// Resolves a relative path beneath this root and returns a one-use read grant.
    ///
    /// Parent traversal, absolute paths, root escape through parent symlinks, and a linked final
    /// component are rejected. Parent symlinks that remain contained beneath the root are allowed.
    pub fn grant_read(
        &self,
        asset_capability: AssetCapability,
        relative_path: &Path,
    ) -> Result<AssetReadGrant, DomainError> {
        validate_asset_relative_path(relative_path)?;
        let mut options = cap_std::fs::OpenOptions::new();
        options.read(true);
        options.follow(FollowSymlinks::No);
        let source = self
            .directory
            .open_with(relative_path, &options)
            .map_err(|_| DomainError::InvalidValue {
                field: "geometry local asset path",
                reason: "must resolve to an accessible contained file whose final component is not a link",
            })?
            .into_std();
        AssetReadGrant::new(asset_capability, source)
    }
}

fn validate_asset_relative_path(relative_path: &Path) -> Result<(), DomainError> {
    let mut components = relative_path.components();
    if components.next().is_none()
        || !relative_path
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
    {
        return Err(DomainError::InvalidValue {
            field: "geometry local asset path",
            reason: "must be a nonempty normalized relative path without parent traversal",
        });
    }
    Ok(())
}

/// Opens one local source read-only without following a final-component link, then grants it.
///
/// The caller remains responsible for authorizing the path and safely resolving parent
/// components. This function protects the final component and returns no path in the grant.
pub fn open_local_source_read_only(
    asset_capability: AssetCapability,
    source_path: &Path,
) -> Result<AssetReadGrant, DomainError> {
    let source =
        open_final_component_read_only(source_path).map_err(|_| DomainError::InvalidValue {
            field: "geometry local source",
            reason: "must be an accessible regular file whose final component is not a link",
        })?;
    AssetReadGrant::new(asset_capability, source)
}

#[cfg(unix)]
fn open_final_component_read_only(source_path: &Path) -> std::io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(source_path)
}

#[cfg(windows)]
fn open_final_component_read_only(source_path: &Path) -> std::io::Result<File> {
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    const SECURITY_IDENTIFICATION: u32 = 0x0001_0000;

    let source = OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .security_qos_flags(SECURITY_IDENTIFICATION)
        .open(source_path)?;
    if source.metadata()?.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "final source component is a reparse point",
        ));
    }
    Ok(source)
}

#[cfg(not(any(unix, windows)))]
fn open_final_component_read_only(_source_path: &Path) -> std::io::Result<File> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "no final-component no-follow opener is available for this target",
    ))
}

/// Local process supervisor for the isolated geometry worker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeometryWorkerSupervisor {
    executable: PathBuf,
    working_directory: PathBuf,
    policy: SupervisorPolicy,
    native_library_directory: Option<PathBuf>,
    asset_transport_policy: WorkerAssetTransportPolicy,
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
            asset_transport_policy: WorkerAssetTransportPolicy::VerifiedCopyOnly,
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

    /// Selects the deployment policy for direct transport or explicit verified-copy fallback.
    #[must_use]
    pub const fn with_asset_transport_policy(
        mut self,
        asset_transport_policy: WorkerAssetTransportPolicy,
    ) -> Self {
        self.asset_transport_policy = asset_transport_policy;
        self
    }

    fn execute_in(
        &self,
        request: &GeometryWorkerRequest,
        asset_manifest: WorkerAssetManifest,
        direct_asset: Option<partprobe_platform::DirectWorkerAsset>,
        cancellation: &AtomicBool,
        working_directory: &Path,
    ) -> GeometryWorkerResponse {
        if cancellation.load(Ordering::Acquire) {
            return response_for(request, WorkerTermination::Cancelled);
        }

        let request_frame = match serialize_control_frame(GeometryWorkerControlMessage::execute(
            request.clone(),
            asset_manifest,
        )) {
            Ok(bytes) if bytes.len() <= self.policy.max_protocol_bytes => bytes,
            _ => return response_for(request, WorkerTermination::QuotaExceeded),
        };

        let resource_limits = partprobe_platform::WorkerResourceLimits::new(
            self.policy.max_worker_memory_bytes,
            NonZeroU64::new(
                request
                    .quotas()
                    .wall_time_millis()
                    .min(self.policy.max_worker_cpu_millis()),
            )
            .expect("validated worker CPU limits must be nonzero"),
            NonZeroU64::new(request.quotas().max_output_bytes())
                .expect("validated request output limit must be nonzero"),
        );
        let mut command = partprobe_platform::WorkerCommand::new(&self.executable, resource_limits);
        command.current_dir(working_directory);
        if let Some(directory) = &self.native_library_directory {
            configure_native_library_path(&mut command, directory);
        }
        if let Some(direct_asset) = direct_asset {
            command.direct_asset(direct_asset);
        }
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(_) => return response_for(request, WorkerTermination::LaunchFailed),
        };
        let Some(stdin) = child.take_stdin() else {
            terminate_and_reap(&mut child);
            return response_for(request, WorkerTermination::ProtocolIo);
        };
        let (control_tx, control_writer) = spawn_control_writer(stdin, request_frame);

        let Some(stdout) = child.take_stdout() else {
            terminate_and_reap(&mut child);
            close_control_writer(control_tx, control_writer);
            return response_for(request, WorkerTermination::ProtocolIo);
        };
        let response_limit = self.policy.max_protocol_bytes;
        let reader = thread::spawn(move || read_bounded(stdout, response_limit));

        let timeout = Duration::from_millis(request.quotas().wall_time_millis());
        let poll_interval = Duration::from_millis(self.policy.poll_interval_millis);
        let cancellation_grace = Duration::from_millis(self.policy.cancellation_grace_millis);
        let started = Instant::now();
        let mut shutdown = None;
        let status = loop {
            if shutdown.is_none() {
                let reason = if cancellation.load(Ordering::Acquire) {
                    Some(WorkerCancellationReason::UserRequested)
                } else if started.elapsed() >= timeout {
                    Some(WorkerCancellationReason::DeadlineExceeded)
                } else {
                    None
                };
                if let Some(reason) = reason {
                    let _ = control_tx.send(ControlWriterCommand::Send(Box::new(
                        GeometryWorkerControlMessage::cancel(request, reason),
                    )));
                    shutdown = Some((reason, Instant::now()));
                }
            }
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {
                    if let Some((reason, grace_started)) = shutdown
                        && grace_started.elapsed() >= cancellation_grace
                    {
                        terminate_and_reap(&mut child);
                        close_control_writer(control_tx, control_writer);
                        let _ = reader.join();
                        let termination = match reason {
                            WorkerCancellationReason::UserRequested => {
                                WorkerTermination::CancellationGraceExceeded
                            }
                            WorkerCancellationReason::DeadlineExceeded => {
                                WorkerTermination::TimeoutGraceExceeded
                            }
                        };
                        return response_for(request, termination);
                    }
                    thread::sleep(poll_interval);
                }
                Err(_) => {
                    terminate_and_reap(&mut child);
                    close_control_writer(control_tx, control_writer);
                    let _ = reader.join();
                    return response_for(request, WorkerTermination::ProtocolIo);
                }
            }
        };

        if !close_control_writer(control_tx, control_writer) {
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
        let reported_cancellation = response.diagnostic_codes().iter().find_map(|code| {
            matches!(code.as_str(), "WORKER_CANCELLED" | "WORKER_TIMEOUT").then(|| code.as_str())
        });
        let acknowledges_cancellation = response
            .diagnostic_codes()
            .iter()
            .any(|code| code.as_str() == "WORKER_CANCELLATION_ACKNOWLEDGED");
        let expected_cancellation = shutdown.map(|(reason, _)| match reason {
            WorkerCancellationReason::UserRequested => "WORKER_CANCELLED",
            WorkerCancellationReason::DeadlineExceeded => "WORKER_TIMEOUT",
        });
        let cancellation_evidence_is_valid = match (
            expected_cancellation,
            reported_cancellation,
            acknowledges_cancellation,
        ) {
            (None, None, false) | (Some(_), None, false) => true,
            (Some(expected), Some(reported), true) => expected == reported,
            _ => false,
        };
        if !cancellation_evidence_is_valid {
            return response_for(request, WorkerTermination::MalformedResponse);
        }
        response
    }

    /// Runs one grant in a private workspace and returns only claimed pathless output.
    pub fn execute_with_grant(
        &self,
        request: &GeometryWorkerRequest,
        mut grant: AssetReadGrant,
        cancellation: &AtomicBool,
    ) -> GeometryWorkerExecution {
        if cancellation.load(Ordering::Acquire) {
            return failed_execution(request, WorkerTermination::Cancelled, None, None);
        }
        let available_direct_transport = available_direct_worker_transport();
        let (asset_transport, fallback_reason) = match self.asset_transport_policy {
            WorkerAssetTransportPolicy::VerifiedCopyOnly => {
                (WorkerAssetTransport::VerifiedPrivateCopy, None)
            }
            WorkerAssetTransportPolicy::PreferDirect => match available_direct_transport {
                Some(transport) => (transport, None),
                None => (
                    WorkerAssetTransport::VerifiedPrivateCopy,
                    Some(WorkerAssetFallbackReason::DirectTransportUnavailable),
                ),
            },
            WorkerAssetTransportPolicy::RequireDirect => {
                let Some(transport) = available_direct_transport else {
                    return failed_execution(
                        request,
                        WorkerTermination::AssetDirectTransportUnavailable,
                        None,
                        None,
                    );
                };
                (transport, None)
            }
        };
        let job_directory = match create_job_directory(&self.working_directory) {
            Ok(path) => path,
            Err(_) => {
                return failed_execution(
                    request,
                    WorkerTermination::WorkspacePrepareFailed,
                    Some(asset_transport),
                    fallback_reason,
                );
            }
        };
        let staged_path = job_directory.join(WORKER_INPUT_FILENAME);
        let output_path = job_directory.join(WORKER_OUTPUT_FILENAME);
        let prepared = prepare_worker_asset(request, &mut grant, asset_transport, &staged_path);
        let (asset_manifest, direct_asset) = match prepared {
            Ok(prepared) => prepared,
            Err(termination) => {
                drop(grant);
                let cleanup = remove_staged_asset(&staged_path)
                    .and_then(|()| std::fs::remove_dir(&job_directory));
                return failed_execution(
                    request,
                    if cleanup.is_ok() {
                        termination
                    } else {
                        WorkerTermination::WorkspaceCleanupFailed
                    },
                    Some(asset_transport),
                    fallback_reason,
                );
            }
        };
        drop(grant);

        let response = self.execute_in(
            request,
            asset_manifest,
            direct_asset,
            cancellation,
            &job_directory,
        );
        let output = reconcile_worker_output(request, &response, &output_path);
        let asset_cleanup = remove_staged_asset(&staged_path);
        let workspace_cleanup = std::fs::remove_dir(&job_directory);

        if asset_cleanup.is_err() {
            return failed_execution(
                request,
                WorkerTermination::AssetCleanupFailed,
                Some(asset_transport),
                fallback_reason,
            );
        }
        let output = match output {
            Ok(output) => output,
            Err(termination) => {
                return failed_execution(
                    request,
                    termination,
                    Some(asset_transport),
                    fallback_reason,
                );
            }
        };
        if workspace_cleanup.is_err() {
            return failed_execution(
                request,
                WorkerTermination::WorkspaceCleanupFailed,
                Some(asset_transport),
                fallback_reason,
            );
        }
        GeometryWorkerExecution::new(response, output, Some(asset_transport), fallback_reason)
    }
}

fn available_direct_worker_transport() -> Option<WorkerAssetTransport> {
    if !partprobe_platform::direct_worker_asset_supported() {
        return None;
    }
    #[cfg(unix)]
    {
        Some(WorkerAssetTransport::UnixDescriptor)
    }
    #[cfg(windows)]
    {
        Some(WorkerAssetTransport::WindowsHandle)
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

fn prepare_worker_asset(
    request: &GeometryWorkerRequest,
    grant: &mut AssetReadGrant,
    transport: WorkerAssetTransport,
    staged_path: &Path,
) -> Result<
    (
        WorkerAssetManifest,
        Option<partprobe_platform::DirectWorkerAsset>,
    ),
    WorkerTermination,
> {
    match transport {
        WorkerAssetTransport::VerifiedPrivateCopy => {
            stage_asset(request, grant, staged_path)?;
            Ok((
                WorkerAssetManifest::verified_private_copy(request, grant.authorized_byte_length()),
                None,
            ))
        }
        WorkerAssetTransport::UnixDescriptor => {
            #[cfg(unix)]
            {
                verify_direct_asset_grant(request, grant)?;
                let direct_asset = partprobe_platform::prepare_direct_worker_asset(&grant.source)
                    .map_err(|_| WorkerTermination::AssetTransportInvalid)?;
                let manifest = WorkerAssetManifest::direct(
                    request,
                    grant.authorized_byte_length(),
                    WorkerAssetTransport::UnixDescriptor,
                    direct_asset.resource_id(),
                );
                Ok((manifest, Some(direct_asset)))
            }
            #[cfg(not(unix))]
            {
                Err(WorkerTermination::AssetTransportInvalid)
            }
        }
        WorkerAssetTransport::WindowsHandle => {
            #[cfg(windows)]
            {
                verify_direct_asset_grant(request, grant)?;
                let direct_asset = partprobe_platform::prepare_direct_worker_asset(&grant.source)
                    .map_err(|_| WorkerTermination::AssetTransportInvalid)?;
                let manifest = WorkerAssetManifest::direct(
                    request,
                    grant.authorized_byte_length(),
                    WorkerAssetTransport::WindowsHandle,
                    direct_asset.resource_id(),
                );
                Ok((manifest, Some(direct_asset)))
            }
            #[cfg(not(windows))]
            {
                Err(WorkerTermination::AssetTransportInvalid)
            }
        }
    }
}

fn verify_direct_asset_grant(
    request: &GeometryWorkerRequest,
    grant: &mut AssetReadGrant,
) -> Result<(), WorkerTermination> {
    if grant.asset_capability() != request.asset_capability() {
        return Err(WorkerTermination::AssetGrantMismatch);
    }
    if grant.authorized_byte_length() > request.quotas().max_input_bytes() {
        return Err(WorkerTermination::AssetStageFailed);
    }
    grant
        .source
        .seek(SeekFrom::Start(0))
        .map_err(|_| WorkerTermination::AssetStageFailed)?;
    copy_and_verify(
        request,
        &mut grant.source,
        grant.authorized_byte_length,
        &mut std::io::sink(),
    )?;
    grant
        .source
        .seek(SeekFrom::Start(0))
        .map_err(|_| WorkerTermination::AssetStageFailed)?;
    Ok(())
}

fn create_job_directory(root: &Path) -> std::io::Result<PathBuf> {
    const MAX_ATTEMPTS: usize = 128;

    for _ in 0..MAX_ATTEMPTS {
        let sequence = JOB_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = root.join(format!(".partprobe-job-{}-{sequence}", std::process::id()));
        let builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        let builder = {
            use std::os::unix::fs::DirBuilderExt;

            let mut builder = builder;
            builder.mode(0o700);
            builder
        };
        match builder.create(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "could not allocate a unique worker job directory",
    ))
}

fn reconcile_worker_output(
    request: &GeometryWorkerRequest,
    response: &GeometryWorkerResponse,
    output_path: &Path,
) -> Result<Option<ControlledWorkerOutput>, WorkerTermination> {
    let Some(snapshot_reference) = response.snapshot_reference().cloned() else {
        remove_worker_output(output_path).map_err(|_| WorkerTermination::OutputCleanupFailed)?;
        return Ok(None);
    };
    let claimed = claim_worker_output(
        snapshot_reference,
        output_path,
        request.quotas().max_output_bytes(),
    );
    match claimed {
        Ok(output) => Ok(Some(output)),
        Err(termination) => {
            if remove_worker_output(output_path).is_err() {
                Err(WorkerTermination::OutputCleanupFailed)
            } else {
                Err(termination)
            }
        }
    }
}

fn claim_worker_output(
    snapshot_reference: SnapshotReference,
    output_path: &Path,
    max_output_bytes: u64,
) -> Result<ControlledWorkerOutput, WorkerTermination> {
    let mut source = open_final_component_read_only(output_path)
        .map_err(|_| WorkerTermination::OutputClaimFailed)?;
    let metadata = source
        .metadata()
        .map_err(|_| WorkerTermination::OutputClaimFailed)?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > max_output_bytes {
        return Err(WorkerTermination::OutputClaimFailed);
    }

    let (content_hash, bytes) = read_worker_output(&mut source, metadata.len(), max_output_bytes)?;
    std::fs::remove_file(output_path).map_err(|_| WorkerTermination::OutputCleanupFailed)?;
    Ok(ControlledWorkerOutput {
        snapshot_reference,
        content_hash,
        byte_length: metadata.len(),
        bytes,
    })
}

fn read_worker_output(
    source: &mut File,
    expected_byte_length: u64,
    max_output_bytes: u64,
) -> Result<(Sha256Digest, Box<[u8]>), WorkerTermination> {
    source
        .seek(SeekFrom::Start(0))
        .map_err(|_| WorkerTermination::OutputClaimFailed)?;
    let mut hasher = Sha256::new();
    let mut total = 0_u64;
    let expected_capacity =
        usize::try_from(expected_byte_length).map_err(|_| WorkerTermination::OutputClaimFailed)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(expected_capacity)
        .map_err(|_| WorkerTermination::OutputClaimFailed)?;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = source
            .read(&mut buffer)
            .map_err(|_| WorkerTermination::OutputClaimFailed)?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(u64::try_from(read).map_err(|_| WorkerTermination::OutputClaimFailed)?)
            .ok_or(WorkerTermination::OutputClaimFailed)?;
        if total > max_output_bytes || total > expected_byte_length {
            return Err(WorkerTermination::OutputClaimFailed);
        }
        hasher.update(&buffer[..read]);
        bytes.extend_from_slice(&buffer[..read]);
    }
    if total != expected_byte_length {
        return Err(WorkerTermination::OutputClaimFailed);
    }

    let mut digest = String::with_capacity(64);
    for byte in hasher.finalize() {
        write!(&mut digest, "{byte:02x}").map_err(|_| WorkerTermination::OutputClaimFailed)?;
    }
    let content_hash =
        Sha256Digest::new(digest).map_err(|_| WorkerTermination::OutputClaimFailed)?;
    Ok((content_hash, bytes.into_boxed_slice()))
}

fn remove_worker_output(output_path: &Path) -> std::io::Result<()> {
    match std::fs::symlink_metadata(output_path) {
        Ok(_) => std::fs::remove_file(output_path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
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

enum ControlWriterCommand {
    Send(Box<GeometryWorkerControlMessage>),
    Close,
}

fn serialize_control_frame(
    message: GeometryWorkerControlMessage,
) -> Result<Vec<u8>, serde_json::Error> {
    let mut bytes = serde_json::to_vec(&message)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn spawn_control_writer(
    mut stdin: partprobe_platform::WorkerStdin,
    request_frame: Vec<u8>,
) -> (
    mpsc::Sender<ControlWriterCommand>,
    thread::JoinHandle<std::io::Result<()>>,
) {
    let (sender, receiver) = mpsc::channel();
    let writer = thread::spawn(move || {
        stdin.write_all(&request_frame)?;
        stdin.flush()?;
        while let Ok(command) = receiver.recv() {
            match command {
                ControlWriterCommand::Send(message) => {
                    stdin.write_all(&serialize_control_frame(*message).map_err(|_| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "control message serialization failed",
                        )
                    })?)?;
                    stdin.flush()?;
                }
                ControlWriterCommand::Close => break,
            }
        }
        Ok(())
    });
    (sender, writer)
}

fn close_control_writer(
    sender: mpsc::Sender<ControlWriterCommand>,
    writer: thread::JoinHandle<std::io::Result<()>>,
) -> bool {
    let _ = sender.send(ControlWriterCommand::Close);
    matches!(writer.join(), Ok(Ok(())))
}

fn terminate_and_reap(child: &mut partprobe_platform::WorkerChild) {
    let _ = child.kill();
    let _ = child.wait();
}

fn configure_native_library_path(
    command: &mut partprobe_platform::WorkerCommand,
    directory: &Path,
) {
    #[cfg(target_os = "macos")]
    command.env("DYLD_LIBRARY_PATH", directory);
    #[cfg(all(unix, not(target_os = "macos")))]
    command.env("LD_LIBRARY_PATH", directory);
    #[cfg(windows)]
    command.env("PATH", directory);
}

/// Opens and independently verifies the fixed private-copy input before any adapter sees bytes.
pub fn verify_worker_asset_copy(
    request: &GeometryWorkerRequest,
    manifest: &WorkerAssetManifest,
    worker_directory: &Path,
) -> Result<VerifiedWorkerAsset, WorkerTermination> {
    if manifest.transport() != WorkerAssetTransport::VerifiedPrivateCopy {
        return Err(WorkerTermination::AssetTransportInvalid);
    }
    let source_path = worker_directory.join(WORKER_INPUT_FILENAME);
    let source = open_final_component_read_only(&source_path)
        .map_err(|_| WorkerTermination::AssetTransportInvalid)?;
    verify_worker_asset_source(request, manifest, source)
}

/// Independently verifies one inherited direct descriptor or handle before adapter dispatch.
pub fn verify_worker_asset_direct(
    request: &GeometryWorkerRequest,
    manifest: &WorkerAssetManifest,
    source: File,
) -> Result<VerifiedWorkerAsset, WorkerTermination> {
    if !matches!(
        manifest.transport(),
        WorkerAssetTransport::UnixDescriptor | WorkerAssetTransport::WindowsHandle
    ) {
        return Err(WorkerTermination::AssetTransportInvalid);
    }
    verify_worker_asset_source(request, manifest, source)
}

fn verify_worker_asset_source(
    request: &GeometryWorkerRequest,
    manifest: &WorkerAssetManifest,
    mut source: File,
) -> Result<VerifiedWorkerAsset, WorkerTermination> {
    manifest
        .validate_for(request)
        .map_err(|_| WorkerTermination::AssetManifestMismatch)?;
    let metadata = source
        .metadata()
        .map_err(|_| WorkerTermination::AssetTransportInvalid)?;
    if !metadata.is_file()
        || metadata.len() != manifest.authorized_byte_length()
        || metadata.len() > request.quotas().max_input_bytes()
    {
        return Err(WorkerTermination::AssetTransportInvalid);
    }
    let (content_hash, bytes) = read_verified_worker_asset(
        &mut source,
        manifest.authorized_byte_length(),
        request.quotas().max_input_bytes(),
    )?;
    if &content_hash != request.expected_source_hash() {
        return Err(WorkerTermination::AssetHashMismatch);
    }
    Ok(VerifiedWorkerAsset {
        transport: manifest.transport(),
        content_hash,
        bytes,
    })
}

fn read_verified_worker_asset(
    source: &mut File,
    expected_byte_length: u64,
    max_input_bytes: u64,
) -> Result<(Sha256Digest, Box<[u8]>), WorkerTermination> {
    source
        .seek(SeekFrom::Start(0))
        .map_err(|_| WorkerTermination::AssetTransportInvalid)?;
    let capacity = usize::try_from(expected_byte_length)
        .map_err(|_| WorkerTermination::AssetTransportInvalid)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| WorkerTermination::AssetTransportInvalid)?;
    let limit = max_input_bytes.min(expected_byte_length).saturating_add(1);
    source
        .take(limit)
        .read_to_end(&mut bytes)
        .map_err(|_| WorkerTermination::AssetTransportInvalid)?;
    let actual_length =
        u64::try_from(bytes.len()).map_err(|_| WorkerTermination::AssetTransportInvalid)?;
    if actual_length != expected_byte_length || actual_length > max_input_bytes {
        return Err(WorkerTermination::AssetTransportInvalid);
    }
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let mut digest = String::with_capacity(64);
    for byte in hasher.finalize() {
        write!(&mut digest, "{byte:02x}").map_err(|_| WorkerTermination::AssetTransportInvalid)?;
    }
    let content_hash =
        Sha256Digest::new(digest).map_err(|_| WorkerTermination::AssetTransportInvalid)?;
    Ok((content_hash, bytes.into_boxed_slice()))
}

fn stage_asset(
    request: &GeometryWorkerRequest,
    grant: &mut AssetReadGrant,
    staged_path: &Path,
) -> Result<(), WorkerTermination> {
    if grant.asset_capability() != request.asset_capability() {
        return Err(WorkerTermination::AssetGrantMismatch);
    }
    if grant.authorized_byte_length() > request.quotas().max_input_bytes() {
        return Err(WorkerTermination::AssetStageFailed);
    }
    grant
        .source
        .seek(SeekFrom::Start(0))
        .map_err(|_| WorkerTermination::AssetStageFailed)?;
    let mut staged = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(staged_path)
        .map_err(|_| WorkerTermination::AssetStageFailed)?;
    let copy_result = copy_and_verify(
        request,
        &mut grant.source,
        grant.authorized_byte_length,
        &mut staged,
    );
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

fn copy_and_verify<W: Write>(
    request: &GeometryWorkerRequest,
    source: &mut File,
    authorized_byte_length: u64,
    staged: &mut W,
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
    if total != authorized_byte_length {
        return Err(WorkerTermination::AssetStageFailed);
    }

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

fn failed_execution(
    request: &GeometryWorkerRequest,
    termination: WorkerTermination,
    asset_transport: Option<WorkerAssetTransport>,
    fallback_reason: Option<WorkerAssetFallbackReason>,
) -> GeometryWorkerExecution {
    GeometryWorkerExecution::new(
        response_for(request, termination),
        None,
        asset_transport,
        fallback_reason,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claimed_worker_output_is_hashed_unlinked_and_immutable() {
        let job_directory =
            create_job_directory(&std::env::temp_dir()).expect("private job directory must exist");
        let output_path = job_directory.join(WORKER_OUTPUT_FILENAME);
        let expected_bytes = b"controlled-output";
        std::fs::write(&output_path, expected_bytes).expect("test output must be written");

        let output = claim_worker_output(
            SnapshotReference::new("controlled-snapshot")
                .expect("snapshot reference must be valid"),
            &output_path,
            1_024,
        )
        .expect("regular bounded output must be claimed");

        assert_eq!(output.snapshot_reference().as_str(), "controlled-snapshot");
        assert_eq!(output.byte_length(), 17);
        assert!(!output_path.exists());
        assert_eq!(output.bytes(), expected_bytes);
        assert_eq!(
            output.content_hash().as_str(),
            "c2aa13ac2ee8062adde83a6469537d6f71c686d669f91cb1ab8e07712fdb2797"
        );

        drop(output);
        std::fs::remove_dir(job_directory).expect("empty private job directory must be removed");
    }

    #[test]
    fn authorized_grant_fingerprint_is_bounded_and_rewinds_the_source() {
        let job_directory =
            create_job_directory(&std::env::temp_dir()).expect("private job directory must exist");
        let source_path = job_directory.join("fingerprint.step");
        std::fs::write(&source_path, b"synthetic-step-source")
            .expect("test source must be written");
        let source = File::open(&source_path).expect("test source must open");
        let capability =
            AssetCapability::new("fingerprint-capability").expect("capability must be valid");
        let mut grant = AssetReadGrant::new(capability, source).expect("grant must be valid");

        let digest = grant
            .fingerprint_sha256(1_024)
            .expect("bounded source must fingerprint");

        assert_eq!(
            digest.as_str(),
            "72806c44cd993f89a56810474fac5ae65710a5d72232f095b5f959b6d84f20e4"
        );
        let mut bytes = Vec::new();
        grant
            .source
            .read_to_end(&mut bytes)
            .expect("grant must be rewound for its consumer");
        assert_eq!(bytes, b"synthetic-step-source");
        assert!(grant.fingerprint_sha256(1).is_err());

        drop(grant);
        std::fs::remove_file(source_path).expect("test source must be removed");
        std::fs::remove_dir(job_directory).expect("empty private job directory must be removed");
    }
}
