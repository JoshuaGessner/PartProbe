//! Versioned, path-free protocol for the isolated geometry worker.

use std::collections::BTreeSet;

use partprobe_domain::{DomainError, SchemaVersion};
use partprobe_geometry_core::{
    AnalysisProfile, GeometryStage, GeometryStageReport, Sha256Digest, StageStatus,
};
use serde::{Deserialize, Deserializer, Serialize};

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
}

/// Converts a worker termination into a safe recoverable response.
pub fn recoverable_termination_response(
    schema_version: SchemaVersion,
    job_id: GeometryJobId,
    correlation_id: CorrelationId,
    termination: WorkerTermination,
) -> GeometryWorkerResponse {
    let code = match termination {
        WorkerTermination::NonzeroExit => "WORKER_EXIT",
        WorkerTermination::Timeout => "WORKER_TIMEOUT",
        WorkerTermination::QuotaExceeded => "WORKER_QUOTA_EXCEEDED",
        WorkerTermination::MalformedResponse => "WORKER_MALFORMED_RESPONSE",
        WorkerTermination::Cancelled => "WORKER_CANCELLED",
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
