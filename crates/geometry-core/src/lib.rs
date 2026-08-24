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
diagnostic_code!(
    /// Stable machine-readable reason for a geometry-confidence level.
    GeometryConfidenceReasonCode,
    "geometry confidence reason code"
);

/// Categorical geometry confidence; percentages are deliberately excluded.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometryConfidenceLevel {
    /// Reviewed exact-representation evidence without a known reduction.
    High,
    /// Exact-representation evidence with a bounded reduction.
    Medium,
    /// Validated mesh evidence at its representation ceiling.
    Low,
    /// Evidence has a condition requiring explicit human review.
    NeedsReview,
}

/// Validated confidence level plus deterministic reason codes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GeometryConfidence {
    level: GeometryConfidenceLevel,
    reasons: Vec<GeometryConfidenceReasonCode>,
}

#[derive(Deserialize)]
struct GeometryConfidenceWire {
    level: GeometryConfidenceLevel,
    reasons: Vec<GeometryConfidenceReasonCode>,
}

impl<'de> Deserialize<'de> for GeometryConfidence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = GeometryConfidenceWire::deserialize(deserializer)?;
        Self::new(wire.level, wire.reasons).map_err(serde::de::Error::custom)
    }
}

impl GeometryConfidence {
    /// Constructs confidence evidence with at least one unique reason.
    pub fn new(
        level: GeometryConfidenceLevel,
        reasons: Vec<GeometryConfidenceReasonCode>,
    ) -> Result<Self, DomainError> {
        if reasons.is_empty() {
            return Err(DomainError::InvalidValue {
                field: "geometry confidence reasons",
                reason: "at least one reason is required",
            });
        }
        let mut unique = BTreeSet::new();
        if reasons.iter().any(|reason| !unique.insert(reason.clone())) {
            return Err(DomainError::InvalidValue {
                field: "geometry confidence reasons",
                reason: "reason codes must be unique",
            });
        }
        Ok(Self { level, reasons })
    }

    /// Returns the categorical confidence level.
    #[must_use]
    pub const fn level(&self) -> GeometryConfidenceLevel {
        self.level
    }

    /// Returns the ordered reason codes.
    #[must_use]
    pub fn reasons(&self) -> &[GeometryConfidenceReasonCode] {
        &self.reasons
    }
}

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

/// Current schema for the bounded developer-only native geometry evidence.
pub const PROVISIONAL_GEOMETRY_SNAPSHOT_SCHEMA_VERSION: u16 = 1;
/// Fixed display/test decimal scale used by the current provisional native spike.
pub const PROVISIONAL_GEOMETRY_DECIMAL_SCALE: u32 = 6;

/// Canonical decimal text retained by the provisional geometry evidence schema.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ProvisionalGeometryDecimal(String);

impl ProvisionalGeometryDecimal {
    /// Validates a non-exponent decimal with at most the provisional six-place scale.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if !is_canonical_provisional_decimal(&value) {
            return Err(DomainError::InvalidValue {
                field: "provisional geometry decimal",
                reason: "must be canonical decimal text with at most six fractional places",
            });
        }
        Ok(Self(value))
    }

    /// Returns the exact canonical decimal text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    const fn is_negative(&self) -> bool {
        self.0.as_bytes()[0] == b'-'
    }
}

impl<'de> Deserialize<'de> for ProvisionalGeometryDecimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

fn is_canonical_provisional_decimal(value: &str) -> bool {
    if value.is_empty() || value.len() > 64 || value == "-0" {
        return false;
    }
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let mut parts = unsigned.split('.');
    let integer = parts.next().unwrap_or_default();
    let fraction = parts.next();
    if parts.next().is_some()
        || integer.is_empty()
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || (integer.len() > 1 && integer.starts_with('0'))
    {
        return false;
    }
    fraction.is_none_or(|digits| {
        !digits.is_empty()
            && digits.len() <= PROVISIONAL_GEOMETRY_DECIMAL_SCALE as usize
            && digits.bytes().all(|byte| byte.is_ascii_digit())
            && !digits.ends_with('0')
    })
}

/// Validated developer-only snapshot emitted by the current optional native STEP spike.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProvisionalGeometrySnapshot {
    schema_version: u16,
    evidence_state: String,
    source_hash: Sha256Digest,
    representation: RepresentationBasis,
    canonical_units: ModelLengthUnit,
    occt_version: String,
    adapter_abi_version: u32,
    decimal_scale: u32,
    transferred_roots: u64,
    solid_body_count: u64,
    surface_area_mm2: ProvisionalGeometryDecimal,
    enclosed_volume_mm3: ProvisionalGeometryDecimal,
    center_of_mass_mm: [ProvisionalGeometryDecimal; 3],
}

#[derive(Deserialize)]
struct ProvisionalGeometrySnapshotWire {
    schema_version: u16,
    evidence_state: String,
    source_hash: Sha256Digest,
    representation: RepresentationBasis,
    canonical_units: ModelLengthUnit,
    occt_version: String,
    adapter_abi_version: u32,
    decimal_scale: u32,
    transferred_roots: u64,
    solid_body_count: u64,
    surface_area_mm2: ProvisionalGeometryDecimal,
    enclosed_volume_mm3: ProvisionalGeometryDecimal,
    center_of_mass_mm: [ProvisionalGeometryDecimal; 3],
}

fn validate_provisional_schema(wire: &ProvisionalGeometrySnapshotWire) -> Result<(), DomainError> {
    if wire.schema_version != PROVISIONAL_GEOMETRY_SNAPSHOT_SCHEMA_VERSION
        || wire.evidence_state != "provisional_spike"
        || wire.representation != RepresentationBasis::ExactBrep
        || wire.canonical_units != ModelLengthUnit::Millimeter
        || wire.decimal_scale != PROVISIONAL_GEOMETRY_DECIMAL_SCALE
    {
        return Err(DomainError::InvalidValue {
            field: "provisional geometry snapshot",
            reason: "schema, evidence state, representation, units, or decimal scale is unsupported",
        });
    }
    Ok(())
}

fn validate_provisional_engine(wire: &ProvisionalGeometrySnapshotWire) -> Result<(), DomainError> {
    if wire.occt_version.is_empty()
        || wire.occt_version.len() > 64
        || !wire
            .occt_version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        || wire.adapter_abi_version == 0
    {
        return Err(DomainError::InvalidValue {
            field: "provisional geometry snapshot",
            reason: "engine version or adapter ABI is invalid",
        });
    }
    Ok(())
}

fn validate_provisional_results(wire: &ProvisionalGeometrySnapshotWire) -> Result<(), DomainError> {
    if wire.transferred_roots == 0
        || wire.solid_body_count == 0
        || wire.surface_area_mm2.is_negative()
        || wire.enclosed_volume_mm3.is_negative()
    {
        return Err(DomainError::InvalidValue {
            field: "provisional geometry snapshot",
            reason: "root/body counts must be positive and area/volume must be non-negative",
        });
    }
    Ok(())
}

impl<'de> Deserialize<'de> for ProvisionalGeometrySnapshot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ProvisionalGeometrySnapshotWire::deserialize(deserializer)?;
        Self::from_wire(wire).map_err(serde::de::Error::custom)
    }
}

impl ProvisionalGeometrySnapshot {
    /// Creates the fixed-basis provisional snapshot used by the native developer seam.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_hash: Sha256Digest,
        occt_version: impl Into<String>,
        adapter_abi_version: u32,
        transferred_roots: u64,
        solid_body_count: u64,
        surface_area_mm2: ProvisionalGeometryDecimal,
        enclosed_volume_mm3: ProvisionalGeometryDecimal,
        center_of_mass_mm: [ProvisionalGeometryDecimal; 3],
    ) -> Result<Self, DomainError> {
        Self::from_wire(ProvisionalGeometrySnapshotWire {
            schema_version: PROVISIONAL_GEOMETRY_SNAPSHOT_SCHEMA_VERSION,
            evidence_state: "provisional_spike".to_owned(),
            source_hash,
            representation: RepresentationBasis::ExactBrep,
            canonical_units: ModelLengthUnit::Millimeter,
            occt_version: occt_version.into(),
            adapter_abi_version,
            decimal_scale: PROVISIONAL_GEOMETRY_DECIMAL_SCALE,
            transferred_roots,
            solid_body_count,
            surface_area_mm2,
            enclosed_volume_mm3,
            center_of_mass_mm,
        })
    }

    fn from_wire(wire: ProvisionalGeometrySnapshotWire) -> Result<Self, DomainError> {
        validate_provisional_schema(&wire)?;
        validate_provisional_engine(&wire)?;
        validate_provisional_results(&wire)?;
        Ok(Self {
            schema_version: wire.schema_version,
            evidence_state: wire.evidence_state,
            source_hash: wire.source_hash,
            representation: wire.representation,
            canonical_units: wire.canonical_units,
            occt_version: wire.occt_version,
            adapter_abi_version: wire.adapter_abi_version,
            decimal_scale: wire.decimal_scale,
            transferred_roots: wire.transferred_roots,
            solid_body_count: wire.solid_body_count,
            surface_area_mm2: wire.surface_area_mm2,
            enclosed_volume_mm3: wire.enclosed_volume_mm3,
            center_of_mass_mm: wire.center_of_mass_mm,
        })
    }

    /// Returns the provisional wire-schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    /// Returns the explicit non-authoritative evidence state.
    #[must_use]
    pub fn evidence_state(&self) -> &str {
        &self.evidence_state
    }

    /// Returns the source digest that the snapshot interprets.
    #[must_use]
    pub const fn source_hash(&self) -> &Sha256Digest {
        &self.source_hash
    }

    /// Returns the representation basis of the provisional measurements.
    #[must_use]
    pub const fn representation(&self) -> RepresentationBasis {
        self.representation
    }

    /// Returns the canonical length unit used by the provisional measurements.
    #[must_use]
    pub const fn canonical_units(&self) -> ModelLengthUnit {
        self.canonical_units
    }

    /// Returns the recorded OCCT version.
    #[must_use]
    pub fn occt_version(&self) -> &str {
        &self.occt_version
    }

    /// Returns the linked native adapter ABI version.
    #[must_use]
    pub const fn adapter_abi_version(&self) -> u32 {
        self.adapter_abi_version
    }

    /// Returns the declared canonical decimal scale.
    #[must_use]
    pub const fn decimal_scale(&self) -> u32 {
        self.decimal_scale
    }

    /// Returns the number of STEP roots transferred by the spike.
    #[must_use]
    pub const fn transferred_roots(&self) -> u64 {
        self.transferred_roots
    }

    /// Returns the exact provisional surface-area text in square millimeters.
    #[must_use]
    pub fn surface_area_mm2(&self) -> &str {
        self.surface_area_mm2.as_str()
    }

    /// Returns the exact provisional enclosed-volume text in cubic millimeters.
    #[must_use]
    pub fn enclosed_volume_mm3(&self) -> &str {
        self.enclosed_volume_mm3.as_str()
    }

    /// Returns the exact provisional centroid text in millimeters.
    #[must_use]
    pub fn center_of_mass_mm(&self) -> [&str; 3] {
        self.center_of_mass_mm
            .each_ref()
            .map(|value| value.as_str())
    }

    /// Returns the number of exact solid bodies reported by the spike.
    #[must_use]
    pub const fn solid_body_count(&self) -> u64 {
        self.solid_body_count
    }
}

/// Evidence used to resolve source geometry units.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitResolutionMethod {
    /// Unit came from a supported file declaration or the format's normative default.
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
