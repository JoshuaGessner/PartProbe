//! Kernel-neutral geometry analysis contracts.

use std::collections::BTreeSet;

use partprobe_domain::{DomainError, RuleVersion};
use serde::{Deserialize, Deserializer, Serialize};

macro_rules! non_empty_id {
    ($(#[$attribute:meta])* $name:ident, $field:literal) => {
        $(#[$attribute])*
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Validates and creates the identifier.
            pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(DomainError::InvalidValue {
                        field: $field,
                        reason: "must not be empty",
                    });
                }
                Ok(Self(value))
            }

            /// Returns the validated identifier.
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

non_empty_id!(
    /// Stable analysis-profile identity.
    AnalysisProfileId,
    "analysis profile ID"
);
diagnostic_code!(
    /// Stable machine-readable warning identity.
    GeometryWarningCode,
    "geometry warning code"
);

/// Lowercase hexadecimal SHA-256 digest of immutable source bytes.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Sha256Digest(String);

impl Sha256Digest {
    /// Validates a canonical lowercase SHA-256 digest.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.len() != 64
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(DomainError::InvalidValue {
                field: "SHA-256 digest",
                reason: "must contain exactly 64 lowercase hexadecimal characters",
            });
        }
        Ok(Self(value))
    }

    /// Returns the canonical digest.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Sha256Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Supported or recognized model container.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFormat {
    /// ISO 10303 STEP exchange.
    Step,
    /// STL triangle mesh.
    Stl,
    /// 3MF package.
    ThreeMf,
    /// Secondary IGES exchange.
    Iges,
    /// Secondary OBJ mesh.
    Obj,
    /// Content is not recognized.
    Unknown,
}

/// Representation authority carried by one geometry result.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepresentationBasis {
    /// Successfully translated exact boundary representation.
    ExactBrep,
    /// Triangle mesh evidence.
    Mesh,
    /// No authoritative representation is available.
    Unknown,
}

/// Length unit reported or confirmed for imported geometry.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelLengthUnit {
    /// Micrometre.
    Micrometer,
    /// Millimetre; PartProbe's canonical geometry unit.
    Millimeter,
    /// Centimetre.
    Centimeter,
    /// Metre.
    Meter,
    /// International inch.
    Inch,
    /// International foot.
    Foot,
    /// Unit was not declared or resolved.
    Unknown,
}

/// Evidence used to resolve source geometry units.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitResolutionMethod {
    /// Unit came from a supported file declaration.
    Declared,
    /// A user explicitly confirmed the unit.
    Confirmed,
    /// A heuristic proposed the unit; approval-grade measurements remain blocked.
    Inferred,
    /// Unit is unresolved.
    Unresolved,
}

/// Canonical pipeline stage.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometryStage {
    /// Hash and preflight immutable bytes.
    Intake,
    /// Detect the content format.
    Identify,
    /// Translate or parse a representation.
    Parse,
    /// Resolve source units and canonical scale.
    UnitResolution,
    /// Produce an optional non-destructive derivative.
    Healing,
    /// Validate the selected representation.
    Validation,
    /// Compute representation-appropriate properties.
    BasicProperties,
    /// Produce a non-authoritative display mesh.
    Tessellation,
}

/// Outcome state of an individual geometry stage or whole job.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    /// Completed without warnings.
    Succeeded,
    /// Completed with retained review warnings.
    SucceededWithWarnings,
    /// Cannot continue authoritatively without explicit input.
    NeedsUserInput,
    /// Failed safely and may be retried.
    FailedRecoverable,
    /// Failed under a condition that requires configuration or software change.
    FailedTerminal,
}

impl StageStatus {
    /// Returns whether this state may expose authoritative stage output.
    #[must_use]
    pub const fn permits_authoritative_output(self) -> bool {
        matches!(self, Self::Succeeded | Self::SucceededWithWarnings)
    }
}

/// Severity of a sanitized geometry warning.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningSeverity {
    /// Informational evidence.
    Info,
    /// Review is advised.
    Warning,
    /// Approval-dependent work is blocked.
    Blocking,
}

/// Sanitized warning without CAD payload, coordinates, names, or local paths.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeometryWarning {
    /// Stable warning code.
    pub code: GeometryWarningCode,
    /// Stage that produced the warning.
    pub stage: GeometryStage,
    /// Review severity.
    pub severity: WarningSeverity,
}

/// Immutable source descriptor created before native parsing.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ModelAssetDescriptor {
    source_hash: Sha256Digest,
    byte_size: u64,
    claimed_format: Option<ModelFormat>,
    detected_format: ModelFormat,
}

#[derive(Deserialize)]
struct ModelAssetDescriptorWire {
    source_hash: Sha256Digest,
    byte_size: u64,
    claimed_format: Option<ModelFormat>,
    detected_format: ModelFormat,
}

impl<'de> Deserialize<'de> for ModelAssetDescriptor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ModelAssetDescriptorWire::deserialize(deserializer)?;
        Self::new(
            wire.source_hash,
            wire.byte_size,
            wire.claimed_format,
            wire.detected_format,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl ModelAssetDescriptor {
    /// Validates immutable source evidence.
    pub fn new(
        source_hash: Sha256Digest,
        byte_size: u64,
        claimed_format: Option<ModelFormat>,
        detected_format: ModelFormat,
    ) -> Result<Self, DomainError> {
        if byte_size == 0 {
            return Err(DomainError::InvalidValue {
                field: "model byte size",
                reason: "must be greater than zero",
            });
        }
        Ok(Self {
            source_hash,
            byte_size,
            claimed_format,
            detected_format,
        })
    }

    /// Returns the immutable source digest.
    #[must_use]
    pub const fn source_hash(&self) -> &Sha256Digest {
        &self.source_hash
    }

    /// Returns the exact source byte count.
    #[must_use]
    pub const fn byte_size(&self) -> u64 {
        self.byte_size
    }

    /// Returns the claimed format, if supplied.
    #[must_use]
    pub const fn claimed_format(&self) -> Option<ModelFormat> {
        self.claimed_format
    }

    /// Returns the content-detected format.
    #[must_use]
    pub const fn detected_format(&self) -> ModelFormat {
        self.detected_format
    }

    /// Returns whether claimed and detected formats conflict.
    #[must_use]
    pub fn has_format_mismatch(&self) -> bool {
        self.claimed_format
            .is_some_and(|claimed| claimed != self.detected_format)
    }
}

/// Versioned analysis behavior selected for a worker job.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AnalysisProfile {
    /// Stable profile identity.
    pub id: AnalysisProfileId,
    /// Immutable semantic profile version.
    pub version: RuleVersion,
}

/// Validated ordered stage report.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GeometryStageReport {
    stage: GeometryStage,
    status: StageStatus,
    warnings: Vec<GeometryWarning>,
}

#[derive(Deserialize)]
struct GeometryStageReportWire {
    stage: GeometryStage,
    status: StageStatus,
    warnings: Vec<GeometryWarning>,
}

impl<'de> Deserialize<'de> for GeometryStageReport {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = GeometryStageReportWire::deserialize(deserializer)?;
        Self::new(wire.stage, wire.status, wire.warnings).map_err(serde::de::Error::custom)
    }
}

impl GeometryStageReport {
    /// Validates status and warning consistency.
    pub fn new(
        stage: GeometryStage,
        status: StageStatus,
        warnings: Vec<GeometryWarning>,
    ) -> Result<Self, DomainError> {
        if warnings.iter().any(|warning| warning.stage != stage) {
            return Err(DomainError::InvalidValue {
                field: "geometry stage report",
                reason: "every warning must identify the report stage",
            });
        }
        let mut codes = BTreeSet::new();
        if warnings
            .iter()
            .any(|warning| !codes.insert(warning.code.clone()))
        {
            return Err(DomainError::InvalidValue {
                field: "geometry stage report",
                reason: "warning codes must be unique within a stage",
            });
        }
        if status == StageStatus::Succeeded && !warnings.is_empty() {
            return Err(DomainError::InvalidValue {
                field: "geometry stage report",
                reason: "a warning-free success state must not carry warnings",
            });
        }
        if status == StageStatus::SucceededWithWarnings && warnings.is_empty() {
            return Err(DomainError::InvalidValue {
                field: "geometry stage report",
                reason: "success-with-warnings requires at least one warning",
            });
        }
        Ok(Self {
            stage,
            status,
            warnings,
        })
    }

    /// Returns the stage.
    #[must_use]
    pub const fn stage(&self) -> GeometryStage {
        self.stage
    }

    /// Returns the outcome status.
    #[must_use]
    pub const fn status(&self) -> StageStatus {
        self.status
    }

    /// Returns the sanitized warnings.
    #[must_use]
    pub fn warnings(&self) -> &[GeometryWarning] {
        &self.warnings
    }
}
