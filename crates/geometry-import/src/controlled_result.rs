//! Additive, source-bound controlled geometry results.

use partprobe_domain::DomainError;
use partprobe_geometry_core::{ProvisionalGeometrySnapshot, Sha256Digest};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    AsciiStlMeshEvidence, ControlledWorkerOutput, PROVISIONAL_GEOMETRY_SNAPSHOT_REFERENCE,
    ThreeMfMeshEvidence, decode_provisional_geometry_snapshot,
};

/// Current schema for a provisional mesh result produced from verified worker bytes.
pub const PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_SCHEMA_VERSION: u16 = 1;
/// Opaque controlled-output reference for the provisional mesh schema.
pub const PROVISIONAL_MESH_GEOMETRY_SNAPSHOT_REFERENCE: &str = "geometry-mesh-snapshot-v1";

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
    /// New provisional mesh result with format-owned evidence.
    Mesh(Box<ProvisionalMeshGeometrySnapshot>),
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

/// Decodes either the retained STEP v1 snapshot or the additive mesh v1 snapshot.
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
