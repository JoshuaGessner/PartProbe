//! Additive, source-bound controlled geometry results.

use partprobe_domain::DomainError;
use partprobe_geometry_core::{
    ExactStepEnvelopeDerivative, ProvisionalGeometrySnapshot, Sha256Digest,
};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    AsciiStlMeshEvidence, ControlledWorkerOutput, PROVISIONAL_GEOMETRY_SNAPSHOT_REFERENCE,
    ThreeMfMeshEvidence, decode_provisional_geometry_snapshot,
};

/// Current schema for a provisional mesh result produced from verified worker bytes.
pub const PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_SCHEMA_VERSION: u16 = 1;
/// Opaque controlled-output reference for the provisional mesh schema.
pub const PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_REFERENCE: &str = "geometry-mesh-snapshot-v1";
/// Current additive exact-STEP analysis container schema.
pub const PROVISIONAL_EXACT_STEP_ANALYSIS_SCHEMA_VERSION: u16 = 1;
/// Opaque controlled-output reference for snapshot-v1 plus its additive AABB derivative.
pub const PROVISIONAL_EXACT_STEP_ANALYSIS_REFERENCE: &str = "geometry-step-analysis-v1";

/// Source-consistent exact STEP snapshot plus an additive envelope derivative.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProvisionalExactStepAnalysis {
    schema_version: u16,
    evidence_state: String,
    snapshot: ProvisionalGeometrySnapshot,
    envelope: ExactStepEnvelopeDerivative,
}

#[derive(Deserialize)]
struct ProvisionalExactStepAnalysisWire {
    schema_version: u16,
    evidence_state: String,
    snapshot: ProvisionalGeometrySnapshot,
    envelope: ExactStepEnvelopeDerivative,
}

impl<'de> Deserialize<'de> for ProvisionalExactStepAnalysis {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ProvisionalExactStepAnalysisWire::deserialize(deserializer)?;
        Self::from_wire(wire).map_err(serde::de::Error::custom)
    }
}

impl ProvisionalExactStepAnalysis {
    /// Constructs a new container only when both exact STEP artifacts bind to one source.
    pub fn new(
        snapshot: ProvisionalGeometrySnapshot,
        envelope: ExactStepEnvelopeDerivative,
    ) -> Result<Self, DomainError> {
        Self::from_wire(ProvisionalExactStepAnalysisWire {
            schema_version: PROVISIONAL_EXACT_STEP_ANALYSIS_SCHEMA_VERSION,
            evidence_state: "provisional_exact_step_analysis".to_owned(),
            snapshot,
            envelope,
        })
    }

    fn from_wire(wire: ProvisionalExactStepAnalysisWire) -> Result<Self, DomainError> {
        if wire.schema_version != PROVISIONAL_EXACT_STEP_ANALYSIS_SCHEMA_VERSION
            || wire.evidence_state != "provisional_exact_step_analysis"
            || wire.snapshot.source_hash() != wire.envelope.source_hash()
        {
            return Err(DomainError::InvalidValue {
                field: "provisional exact STEP analysis",
                reason: "schema, evidence state, or source binding is unsupported",
            });
        }
        Ok(Self {
            schema_version: wire.schema_version,
            evidence_state: wire.evidence_state,
            snapshot: wire.snapshot,
            envelope: wire.envelope,
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
    pub const fn snapshot(&self) -> &ProvisionalGeometrySnapshot {
        &self.snapshot
    }

    #[must_use]
    pub const fn envelope(&self) -> &ExactStepEnvelopeDerivative {
        &self.envelope
    }
}

/// Complete format-owned evidence retained by the provisional mesh result.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "format", content = "analysis", rename_all = "snake_case")]
pub enum ProvisionalMeshEvidence {
    /// Bounded ASCII or binary STL analysis with unresolved physical units.
    Stl(AsciiStlMeshEvidence),
    /// Bounded 3MF package analysis in canonical millimetres.
    ThreeMf(ThreeMfMeshEvidence),
}

/// Source-bound, non-authoritative mesh result emitted by the isolated worker.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProvisionalMeshGeometrySnapshot {
    schema_version: u16,
    evidence_state: String,
    source_hash: Sha256Digest,
    evidence: ProvisionalMeshEvidence,
}

#[derive(Deserialize)]
struct ProvisionalMeshGeometrySnapshotWire {
    schema_version: u16,
    evidence_state: String,
    source_hash: Sha256Digest,
    evidence: ProvisionalMeshEvidence,
}

impl<'de> Deserialize<'de> for ProvisionalMeshGeometrySnapshot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ProvisionalMeshGeometrySnapshotWire::deserialize(deserializer)?;
        Self::from_wire(wire).map_err(serde::de::Error::custom)
    }
}

impl ProvisionalMeshGeometrySnapshot {
    /// Creates source-bound provisional evidence for a parsed STL asset.
    #[must_use]
    pub fn from_stl(source_hash: Sha256Digest, evidence: AsciiStlMeshEvidence) -> Self {
        Self::new(source_hash, ProvisionalMeshEvidence::Stl(evidence))
    }

    /// Creates source-bound provisional evidence for a parsed 3MF asset.
    #[must_use]
    pub fn from_three_mf(source_hash: Sha256Digest, evidence: ThreeMfMeshEvidence) -> Self {
        Self::new(source_hash, ProvisionalMeshEvidence::ThreeMf(evidence))
    }

    fn new(source_hash: Sha256Digest, evidence: ProvisionalMeshEvidence) -> Self {
        Self {
            schema_version: PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_SCHEMA_VERSION,
            evidence_state: "provisional_mesh_spike".to_owned(),
            source_hash,
            evidence,
        }
    }

    fn from_wire(wire: ProvisionalMeshGeometrySnapshotWire) -> Result<Self, DomainError> {
        if wire.schema_version != PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_SCHEMA_VERSION
            || wire.evidence_state != "provisional_mesh_spike"
        {
            return Err(DomainError::InvalidValue {
                field: "provisional mesh geometry snapshot",
                reason: "schema version or evidence state is unsupported",
            });
        }
        Ok(Self {
            schema_version: wire.schema_version,
            evidence_state: wire.evidence_state,
            source_hash: wire.source_hash,
            evidence: wire.evidence,
        })
    }

    /// Returns the provisional mesh wire-schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    /// Returns the explicit non-authoritative evidence state.
    #[must_use]
    pub fn evidence_state(&self) -> &str {
        &self.evidence_state
    }

    /// Returns the authorized source digest interpreted by this result.
    #[must_use]
    pub const fn source_hash(&self) -> &Sha256Digest {
        &self.source_hash
    }

    /// Returns the complete validated format-owned mesh evidence.
    #[must_use]
    pub const fn evidence(&self) -> &ProvisionalMeshEvidence {
        &self.evidence
    }
}

/// Additive controlled-result union; the existing exact STEP schema remains unchanged.
#[derive(Clone, Debug, PartialEq)]
pub enum ControlledGeometryResult {
    /// Existing exact-B-rep developer snapshot and replay contract.
    ExactBrep(Box<ProvisionalGeometrySnapshot>),
    /// Existing snapshot-v1 carried with the new, separately versioned STEP envelope derivative.
    ExactBrepWithEnvelope(Box<ProvisionalExactStepAnalysis>),
    /// New provisional mesh result with format-owned evidence.
    Mesh(Box<ProvisionalMeshGeometrySnapshot>),
}

/// Decodes the additive exact STEP container and independently binds it to the expected source.
pub fn decode_provisional_exact_step_analysis(
    output: &ControlledWorkerOutput,
    expected_source_hash: &Sha256Digest,
) -> Result<ProvisionalExactStepAnalysis, DomainError> {
    if output.snapshot_reference().as_str() != PROVISIONAL_EXACT_STEP_ANALYSIS_REFERENCE {
        return Err(DomainError::InvalidValue {
            field: "provisional exact STEP analysis",
            reason: "snapshot reference does not identify the exact STEP analysis schema",
        });
    }
    let analysis: ProvisionalExactStepAnalysis =
        serde_json::from_slice(output.bytes()).map_err(|_| DomainError::InvalidValue {
            field: "provisional exact STEP analysis",
            reason: "bytes must satisfy the versioned exact STEP analysis schema",
        })?;
    if analysis.snapshot().source_hash() != expected_source_hash {
        return Err(DomainError::InvalidValue {
            field: "provisional exact STEP analysis",
            reason: "analysis source hash must match the authorized source",
        });
    }
    Ok(analysis)
}

/// Decodes and source-binds the provisional mesh result.
pub fn decode_provisional_mesh_geometry_snapshot(
    output: &ControlledWorkerOutput,
    expected_source_hash: &Sha256Digest,
) -> Result<ProvisionalMeshGeometrySnapshot, DomainError> {
    if output.snapshot_reference().as_str() != PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_REFERENCE {
        return Err(DomainError::InvalidValue {
            field: "provisional mesh geometry snapshot",
            reason: "snapshot reference does not identify the provisional mesh schema",
        });
    }
    let snapshot: ProvisionalMeshGeometrySnapshot = serde_json::from_slice(output.bytes())
        .map_err(|_| DomainError::InvalidValue {
            field: "provisional mesh geometry snapshot",
            reason: "bytes must satisfy the versioned provisional mesh schema",
        })?;
    if snapshot.source_hash() != expected_source_hash {
        return Err(DomainError::InvalidValue {
            field: "provisional mesh geometry snapshot",
            reason: "snapshot source hash must match the authorized source",
        });
    }
    Ok(snapshot)
}

/// Decodes the retained STEP snapshot, additive envelope-bearing STEP, or mesh result.
pub fn decode_controlled_geometry_result(
    output: &ControlledWorkerOutput,
    expected_source_hash: &Sha256Digest,
) -> Result<ControlledGeometryResult, DomainError> {
    match output.snapshot_reference().as_str() {
        PROVISIONAL_GEOMETRY_SNAPSHOT_REFERENCE => {
            decode_provisional_geometry_snapshot(output, expected_source_hash)
                .map(Box::new)
                .map(ControlledGeometryResult::ExactBrep)
        }
        PROVISIONAL_EXACT_STEP_ANALYSIS_REFERENCE => {
            decode_provisional_exact_step_analysis(output, expected_source_hash)
                .map(Box::new)
                .map(ControlledGeometryResult::ExactBrepWithEnvelope)
        }
        PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_REFERENCE => {
            decode_provisional_mesh_geometry_snapshot(output, expected_source_hash)
                .map(Box::new)
                .map(ControlledGeometryResult::Mesh)
        }
        _ => Err(DomainError::InvalidValue {
            field: "controlled geometry result",
            reason: "snapshot reference does not identify a supported geometry result",
        }),
    }
}
