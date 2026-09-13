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

    #[must_use]
    const fn is_positive(&self) -> bool {
        !(self.is_negative() || self.0.len() == 1 && self.0.as_bytes()[0] == b'0')
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

/// First additive source-axis envelope derivative for exact STEP evidence.
pub const EXACT_STEP_ENVELOPE_DERIVATIVE_SCHEMA_VERSION: u16 = 1;

/// Positive exact STEP axis-aligned extents, separate from the retained snapshot-v1 schema.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExactStepEnvelopeDerivative {
    schema_version: u16,
    evidence_state: String,
    source_hash: Sha256Digest,
    representation: RepresentationBasis,
    canonical_units: ModelLengthUnit,
    decimal_scale: u32,
    aabb_extents_mm: [ProvisionalGeometryDecimal; 3],
}

#[derive(Deserialize)]
struct ExactStepEnvelopeDerivativeWire {
    schema_version: u16,
    evidence_state: String,
    source_hash: Sha256Digest,
    representation: RepresentationBasis,
    canonical_units: ModelLengthUnit,
    decimal_scale: u32,
    aabb_extents_mm: [ProvisionalGeometryDecimal; 3],
}

impl<'de> Deserialize<'de> for ExactStepEnvelopeDerivative {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ExactStepEnvelopeDerivativeWire::deserialize(deserializer)?;
        Self::from_wire(wire).map_err(serde::de::Error::custom)
    }
}

impl ExactStepEnvelopeDerivative {
    /// Constructs the fixed-basis derivative without modifying `geometry-snapshot-v1`.
    pub fn new(
        source_hash: Sha256Digest,
        aabb_extents_mm: [ProvisionalGeometryDecimal; 3],
    ) -> Result<Self, DomainError> {
        Self::from_wire(ExactStepEnvelopeDerivativeWire {
            schema_version: EXACT_STEP_ENVELOPE_DERIVATIVE_SCHEMA_VERSION,
            evidence_state: "provisional_exact_step_envelope".to_owned(),
            source_hash,
            representation: RepresentationBasis::ExactBrep,
            canonical_units: ModelLengthUnit::Millimeter,
            decimal_scale: PROVISIONAL_GEOMETRY_DECIMAL_SCALE,
            aabb_extents_mm,
        })
    }

    fn from_wire(wire: ExactStepEnvelopeDerivativeWire) -> Result<Self, DomainError> {
        if wire.schema_version != EXACT_STEP_ENVELOPE_DERIVATIVE_SCHEMA_VERSION
            || wire.evidence_state != "provisional_exact_step_envelope"
            || wire.representation != RepresentationBasis::ExactBrep
            || wire.canonical_units != ModelLengthUnit::Millimeter
            || wire.decimal_scale != PROVISIONAL_GEOMETRY_DECIMAL_SCALE
            || wire
                .aabb_extents_mm
                .iter()
                .any(|value| !value.is_positive())
        {
            return Err(DomainError::InvalidValue {
                field: "exact STEP envelope derivative",
                reason: "schema, evidence state, representation, units, scale, or extents are unsupported",
            });
        }
        Ok(Self {
            schema_version: wire.schema_version,
            evidence_state: wire.evidence_state,
            source_hash: wire.source_hash,
            representation: wire.representation,
            canonical_units: wire.canonical_units,
            decimal_scale: wire.decimal_scale,
            aabb_extents_mm: wire.aabb_extents_mm,
        })
    }

    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    #[must_use]
    pub fn evidence_state(&self) -> &str {
        &self.evidence_state
    }

    #[must_use]
    pub const fn source_hash(&self) -> &Sha256Digest {
        &self.source_hash
    }

    #[must_use]
    pub const fn representation(&self) -> RepresentationBasis {
        self.representation
    }

    #[must_use]
    pub const fn canonical_units(&self) -> ModelLengthUnit {
        self.canonical_units
    }

    #[must_use]
    pub const fn decimal_scale(&self) -> u32 {
        self.decimal_scale
    }

    #[must_use]
    pub const fn aabb_extents_mm(&self) -> &[ProvisionalGeometryDecimal; 3] {
        &self.aabb_extents_mm
    }
}

/// First schema for a bounded native-only display derivative manifest.
pub const GEOMETRY_DISPLAY_SCENE_SCHEMA_VERSION: u16 = 1;
/// Stable controlled-output reference for the first display derivative manifest.
pub const GEOMETRY_DISPLAY_SCENE_REFERENCE: &str = "geometry-display-scene-v1";
/// Fixed evidence label distinguishing visualization data from measurement authority.
pub const GEOMETRY_DISPLAY_SCENE_EVIDENCE_STATE: &str = "display_only";
/// Reviewed tessellation profile implemented by the first display derivative.
pub const DISPLAY_TESSELLATION_PROFILE_REFERENCE: &str = "partprobe-display-tessellation-v1";
/// Version of the reviewed first display tessellation profile.
pub const DISPLAY_TESSELLATION_PROFILE_VERSION: RuleVersion = RuleVersion::new(1, 0, 0);
/// Maximum number of independently hashed chunks in one developer display scene.
pub const MAX_DISPLAY_SCENE_CHUNKS: usize = 32;
/// Maximum vertex count in one developer display scene.
pub const MAX_DISPLAY_SCENE_VERTICES: u64 = 1_000_000;
/// Maximum triangle count in one developer display scene.
pub const MAX_DISPLAY_SCENE_TRIANGLES: u64 = 2_000_000;
/// Maximum combined vertex and index bytes in one developer display scene.
pub const MAX_DISPLAY_SCENE_BYTES: u64 = 32 * 1024 * 1024;

const DISPLAY_VERTEX_STRIDE_BYTES: u64 = 3 * size_of::<f32>() as u64;
const DISPLAY_TRIANGLE_STRIDE_BYTES: u64 = 3 * size_of::<u32>() as u64;

/// Coordinate basis retained by a display-only scene.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayCoordinateSpace {
    /// Canonical millimetres derived from resolved model units.
    CanonicalMillimeters,
    /// Source coordinates whose physical unit is still unresolved.
    UnresolvedSourceCoordinates,
}

/// Binary position layout required by `geometry-display-scene-v1`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayPositionEncoding {
    /// Three finite little-endian IEEE-754 `f32` values per vertex.
    Float32x3LittleEndian,
}

/// Binary index layout required by `geometry-display-scene-v1`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayIndexEncoding {
    /// One little-endian `u32` per index.
    Uint32LittleEndian,
}

/// Primitive topology required by `geometry-display-scene-v1`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayPrimitiveTopology {
    /// Every consecutive three indices form one triangle.
    TriangleList,
}

/// Stable snapshot-scoped body or region reference carried into native selection.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct DisplayGeometryReference(String);

impl DisplayGeometryReference {
    /// Validates a bounded opaque reference that cannot be confused with a path.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 128
            || matches!(value.as_str(), "." | "..")
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err(DomainError::InvalidValue {
                field: "display geometry reference",
                reason: "must be a 1–128 character path-free ASCII token",
            });
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for DisplayGeometryReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Reviewed and replayable tessellation inputs for one display derivative.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DisplayTessellationProfile {
    reference: String,
    version: RuleVersion,
    linear_deflection: ProvisionalGeometryDecimal,
    angular_deflection_degrees: ProvisionalGeometryDecimal,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DisplayTessellationProfileWire {
    reference: String,
    version: RuleVersion,
    linear_deflection: ProvisionalGeometryDecimal,
    angular_deflection_degrees: ProvisionalGeometryDecimal,
}

impl<'de> Deserialize<'de> for DisplayTessellationProfile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = DisplayTessellationProfileWire::deserialize(deserializer)?;
        Self::from_wire(wire).map_err(serde::de::Error::custom)
    }
}

impl DisplayTessellationProfile {
    /// Constructs the current reviewed display tessellation profile.
    pub fn new(
        linear_deflection: ProvisionalGeometryDecimal,
        angular_deflection_degrees: ProvisionalGeometryDecimal,
    ) -> Result<Self, DomainError> {
        Self::from_wire(DisplayTessellationProfileWire {
            reference: DISPLAY_TESSELLATION_PROFILE_REFERENCE.to_owned(),
            version: DISPLAY_TESSELLATION_PROFILE_VERSION,
            linear_deflection,
            angular_deflection_degrees,
        })
    }

    fn from_wire(wire: DisplayTessellationProfileWire) -> Result<Self, DomainError> {
        if wire.reference != DISPLAY_TESSELLATION_PROFILE_REFERENCE
            || wire.version != DISPLAY_TESSELLATION_PROFILE_VERSION
            || !wire.linear_deflection.is_positive()
            || !wire.angular_deflection_degrees.is_positive()
        {
            return Err(DomainError::InvalidValue {
                field: "display tessellation profile",
                reason: "reference, version, or positive tolerances are unsupported",
            });
        }
        Ok(Self {
            reference: wire.reference,
            version: wire.version,
            linear_deflection: wire.linear_deflection,
            angular_deflection_degrees: wire.angular_deflection_degrees,
        })
    }

    #[must_use]
    pub fn reference(&self) -> &str {
        &self.reference
    }

    #[must_use]
    pub const fn version(&self) -> RuleVersion {
        self.version
    }

    #[must_use]
    pub const fn linear_deflection(&self) -> &ProvisionalGeometryDecimal {
        &self.linear_deflection
    }

    #[must_use]
    pub const fn angular_deflection_degrees(&self) -> &ProvisionalGeometryDecimal {
        &self.angular_deflection_degrees
    }
}

/// Hash- and length-bound native buffer descriptor for one display chunk.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DisplayMeshChunkDescriptor {
    sequence: u32,
    geometry_reference: DisplayGeometryReference,
    vertex_count: u32,
    triangle_count: u32,
    vertex_byte_length: u64,
    index_byte_length: u64,
    vertex_sha256: Sha256Digest,
    index_sha256: Sha256Digest,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DisplayMeshChunkDescriptorWire {
    sequence: u32,
    geometry_reference: DisplayGeometryReference,
    vertex_count: u32,
    triangle_count: u32,
    vertex_byte_length: u64,
    index_byte_length: u64,
    vertex_sha256: Sha256Digest,
    index_sha256: Sha256Digest,
}

impl<'de> Deserialize<'de> for DisplayMeshChunkDescriptor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = DisplayMeshChunkDescriptorWire::deserialize(deserializer)?;
        Self::from_wire(wire).map_err(serde::de::Error::custom)
    }
}

impl DisplayMeshChunkDescriptor {
    /// Constructs a descriptor using the schema's fixed vertex and triangle layouts.
    pub fn new(
        sequence: u32,
        geometry_reference: DisplayGeometryReference,
        vertex_count: u32,
        triangle_count: u32,
        vertex_sha256: Sha256Digest,
        index_sha256: Sha256Digest,
    ) -> Result<Self, DomainError> {
        Self::from_wire(DisplayMeshChunkDescriptorWire {
            sequence,
            geometry_reference,
            vertex_count,
            triangle_count,
            vertex_byte_length: u64::from(vertex_count) * DISPLAY_VERTEX_STRIDE_BYTES,
            index_byte_length: u64::from(triangle_count) * DISPLAY_TRIANGLE_STRIDE_BYTES,
            vertex_sha256,
            index_sha256,
        })
    }

    fn from_wire(wire: DisplayMeshChunkDescriptorWire) -> Result<Self, DomainError> {
        let expected_vertex_bytes = u64::from(wire.vertex_count) * DISPLAY_VERTEX_STRIDE_BYTES;
        let expected_index_bytes = u64::from(wire.triangle_count) * DISPLAY_TRIANGLE_STRIDE_BYTES;
        if wire.vertex_count == 0
            || wire.triangle_count == 0
            || wire.vertex_byte_length != expected_vertex_bytes
            || wire.index_byte_length != expected_index_bytes
        {
            return Err(DomainError::InvalidValue {
                field: "display mesh chunk descriptor",
                reason: "counts must be positive and byte lengths must match the fixed layouts",
            });
        }
        Ok(Self {
            sequence: wire.sequence,
            geometry_reference: wire.geometry_reference,
            vertex_count: wire.vertex_count,
            triangle_count: wire.triangle_count,
            vertex_byte_length: wire.vertex_byte_length,
            index_byte_length: wire.index_byte_length,
            vertex_sha256: wire.vertex_sha256,
            index_sha256: wire.index_sha256,
        })
    }

    #[must_use]
    pub const fn sequence(&self) -> u32 {
        self.sequence
    }

    #[must_use]
    pub const fn geometry_reference(&self) -> &DisplayGeometryReference {
        &self.geometry_reference
    }

    #[must_use]
    pub const fn vertex_count(&self) -> u32 {
        self.vertex_count
    }

    #[must_use]
    pub const fn triangle_count(&self) -> u32 {
        self.triangle_count
    }

    #[must_use]
    pub const fn vertex_byte_length(&self) -> u64 {
        self.vertex_byte_length
    }

    #[must_use]
    pub const fn index_byte_length(&self) -> u64 {
        self.index_byte_length
    }

    #[must_use]
    pub const fn vertex_sha256(&self) -> &Sha256Digest {
        &self.vertex_sha256
    }

    #[must_use]
    pub const fn index_sha256(&self) -> &Sha256Digest {
        &self.index_sha256
    }
}

/// Bounded manifest for native-owned visualization buffers.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GeometryDisplaySceneManifest {
    schema_version: u16,
    reference: String,
    evidence_state: String,
    source_hash: Sha256Digest,
    analysis_output_hash: Sha256Digest,
    representation: RepresentationBasis,
    coordinate_space: DisplayCoordinateSpace,
    units: ModelLengthUnit,
    position_encoding: DisplayPositionEncoding,
    index_encoding: DisplayIndexEncoding,
    primitive_topology: DisplayPrimitiveTopology,
    tessellation_profile: DisplayTessellationProfile,
    aabb_extents: [ProvisionalGeometryDecimal; 3],
    total_vertex_count: u64,
    total_triangle_count: u64,
    total_byte_length: u64,
    chunks: Vec<DisplayMeshChunkDescriptor>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeometryDisplaySceneManifestWire {
    schema_version: u16,
    reference: String,
    evidence_state: String,
    source_hash: Sha256Digest,
    analysis_output_hash: Sha256Digest,
    representation: RepresentationBasis,
    coordinate_space: DisplayCoordinateSpace,
    units: ModelLengthUnit,
    position_encoding: DisplayPositionEncoding,
    index_encoding: DisplayIndexEncoding,
    primitive_topology: DisplayPrimitiveTopology,
    tessellation_profile: DisplayTessellationProfile,
    aabb_extents: [ProvisionalGeometryDecimal; 3],
    total_vertex_count: u64,
    total_triangle_count: u64,
    total_byte_length: u64,
    chunks: Vec<DisplayMeshChunkDescriptor>,
}

impl<'de> Deserialize<'de> for GeometryDisplaySceneManifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = GeometryDisplaySceneManifestWire::deserialize(deserializer)?;
        Self::from_wire(wire).map_err(serde::de::Error::custom)
    }
}

impl GeometryDisplaySceneManifest {
    /// Constructs a display-only scene manifest from native buffer descriptors.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_hash: Sha256Digest,
        analysis_output_hash: Sha256Digest,
        representation: RepresentationBasis,
        coordinate_space: DisplayCoordinateSpace,
        units: ModelLengthUnit,
        tessellation_profile: DisplayTessellationProfile,
        aabb_extents: [ProvisionalGeometryDecimal; 3],
        chunks: Vec<DisplayMeshChunkDescriptor>,
    ) -> Result<Self, DomainError> {
        let (total_vertex_count, total_triangle_count, total_byte_length) =
            display_scene_totals(&chunks)?;
        Self::from_wire(GeometryDisplaySceneManifestWire {
            schema_version: GEOMETRY_DISPLAY_SCENE_SCHEMA_VERSION,
            reference: GEOMETRY_DISPLAY_SCENE_REFERENCE.to_owned(),
            evidence_state: GEOMETRY_DISPLAY_SCENE_EVIDENCE_STATE.to_owned(),
            source_hash,
            analysis_output_hash,
            representation,
            coordinate_space,
            units,
            position_encoding: DisplayPositionEncoding::Float32x3LittleEndian,
            index_encoding: DisplayIndexEncoding::Uint32LittleEndian,
            primitive_topology: DisplayPrimitiveTopology::TriangleList,
            tessellation_profile,
            aabb_extents,
            total_vertex_count,
            total_triangle_count,
            total_byte_length,
            chunks,
        })
    }

    fn from_wire(wire: GeometryDisplaySceneManifestWire) -> Result<Self, DomainError> {
        let coordinates_are_valid = matches!(
            (wire.representation, wire.coordinate_space, wire.units),
            (
                RepresentationBasis::ExactBrep | RepresentationBasis::Mesh,
                DisplayCoordinateSpace::CanonicalMillimeters,
                ModelLengthUnit::Millimeter
            ) | (
                RepresentationBasis::Mesh,
                DisplayCoordinateSpace::UnresolvedSourceCoordinates,
                ModelLengthUnit::Unknown
            )
        );
        if wire.schema_version != GEOMETRY_DISPLAY_SCENE_SCHEMA_VERSION
            || wire.reference != GEOMETRY_DISPLAY_SCENE_REFERENCE
            || wire.evidence_state != GEOMETRY_DISPLAY_SCENE_EVIDENCE_STATE
            || !coordinates_are_valid
            || wire.aabb_extents.iter().any(|extent| !extent.is_positive())
        {
            return Err(DomainError::InvalidValue {
                field: "geometry display scene manifest",
                reason: "schema, binding, representation, units, or extents are unsupported",
            });
        }

        let (vertex_count, triangle_count, byte_length) = display_scene_totals(&wire.chunks)?;
        if wire.total_vertex_count != vertex_count
            || wire.total_triangle_count != triangle_count
            || wire.total_byte_length != byte_length
        {
            return Err(DomainError::InvalidValue {
                field: "geometry display scene totals",
                reason: "declared totals must exactly match the ordered chunk descriptors",
            });
        }

        Ok(Self {
            schema_version: wire.schema_version,
            reference: wire.reference,
            evidence_state: wire.evidence_state,
            source_hash: wire.source_hash,
            analysis_output_hash: wire.analysis_output_hash,
            representation: wire.representation,
            coordinate_space: wire.coordinate_space,
            units: wire.units,
            position_encoding: wire.position_encoding,
            index_encoding: wire.index_encoding,
            primitive_topology: wire.primitive_topology,
            tessellation_profile: wire.tessellation_profile,
            aabb_extents: wire.aabb_extents,
            total_vertex_count: wire.total_vertex_count,
            total_triangle_count: wire.total_triangle_count,
            total_byte_length: wire.total_byte_length,
            chunks: wire.chunks,
        })
    }

    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    #[must_use]
    pub fn reference(&self) -> &str {
        &self.reference
    }

    #[must_use]
    pub fn evidence_state(&self) -> &str {
        &self.evidence_state
    }

    #[must_use]
    pub const fn source_hash(&self) -> &Sha256Digest {
        &self.source_hash
    }

    #[must_use]
    pub const fn analysis_output_hash(&self) -> &Sha256Digest {
        &self.analysis_output_hash
    }

    #[must_use]
    pub const fn representation(&self) -> RepresentationBasis {
        self.representation
    }

    #[must_use]
    pub const fn coordinate_space(&self) -> DisplayCoordinateSpace {
        self.coordinate_space
    }

    #[must_use]
    pub const fn units(&self) -> ModelLengthUnit {
        self.units
    }

    #[must_use]
    pub const fn tessellation_profile(&self) -> &DisplayTessellationProfile {
        &self.tessellation_profile
    }

    #[must_use]
    pub const fn aabb_extents(&self) -> &[ProvisionalGeometryDecimal; 3] {
        &self.aabb_extents
    }

    #[must_use]
    pub const fn total_vertex_count(&self) -> u64 {
        self.total_vertex_count
    }

    #[must_use]
    pub const fn total_triangle_count(&self) -> u64 {
        self.total_triangle_count
    }

    #[must_use]
    pub const fn total_byte_length(&self) -> u64 {
        self.total_byte_length
    }

    #[must_use]
    pub fn chunks(&self) -> &[DisplayMeshChunkDescriptor] {
        &self.chunks
    }
}

fn display_scene_totals(
    chunks: &[DisplayMeshChunkDescriptor],
) -> Result<(u64, u64, u64), DomainError> {
    if chunks.is_empty() || chunks.len() > MAX_DISPLAY_SCENE_CHUNKS {
        return Err(DomainError::InvalidValue {
            field: "geometry display scene chunks",
            reason: "chunk count is outside the reviewed display-scene ceiling",
        });
    }

    let mut vertices = 0_u64;
    let mut triangles = 0_u64;
    let mut bytes = 0_u64;
    for (expected_sequence, chunk) in chunks.iter().enumerate() {
        if u32::try_from(expected_sequence) != Ok(chunk.sequence) {
            return Err(DomainError::InvalidValue {
                field: "geometry display scene chunks",
                reason: "chunk sequences must be contiguous and start at zero",
            });
        }
        vertices = vertices.checked_add(u64::from(chunk.vertex_count)).ok_or(
            DomainError::InvalidValue {
                field: "geometry display scene totals",
                reason: "vertex total overflowed",
            },
        )?;
        triangles = triangles
            .checked_add(u64::from(chunk.triangle_count))
            .ok_or(DomainError::InvalidValue {
                field: "geometry display scene totals",
                reason: "triangle total overflowed",
            })?;
        bytes = bytes
            .checked_add(chunk.vertex_byte_length)
            .and_then(|total| total.checked_add(chunk.index_byte_length))
            .ok_or(DomainError::InvalidValue {
                field: "geometry display scene totals",
                reason: "byte total overflowed",
            })?;
    }

    if vertices > MAX_DISPLAY_SCENE_VERTICES
        || triangles > MAX_DISPLAY_SCENE_TRIANGLES
        || bytes > MAX_DISPLAY_SCENE_BYTES
    {
        return Err(DomainError::InvalidValue {
            field: "geometry display scene totals",
            reason: "scene exceeds a reviewed vertex, triangle, or byte ceiling",
        });
    }
    Ok((vertices, triangles, bytes))
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
