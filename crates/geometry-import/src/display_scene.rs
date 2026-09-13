use std::fmt;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};

use partprobe_geometry_core::{
    DisplayGeometryReference, DisplayMeshChunkDescriptor, GEOMETRY_DISPLAY_SCENE_REFERENCE,
    GeometryDisplaySceneManifest, MAX_DISPLAY_SCENE_BYTES, MAX_DISPLAY_SCENE_CHUNKS, Sha256Digest,
};
use sha2::{Digest, Sha256};

use crate::ControlledWorkerOutput;

const CANCELLATION_POLL_ITEMS: usize = 4_096;
const HASH_POLL_BYTES: usize = 64 * 1024;
const DISPLAY_SCENE_ARTIFACT_MAGIC: [u8; 8] = *b"PPDSCENE";
const DISPLAY_SCENE_ARTIFACT_HEADER_BYTES: u64 = 20;
const DISPLAY_SCENE_CHUNK_HEADER_BYTES: u64 = 20;

/// First deterministic binary framing version for a controlled display scene.
pub const DISPLAY_SCENE_ARTIFACT_SCHEMA_VERSION: u16 = 1;
/// Maximum JSON manifest bytes embedded in one display-scene artifact.
pub const MAX_DISPLAY_SCENE_MANIFEST_BYTES: u64 = 64 * 1024;
/// Maximum complete framed artifact size accepted before native decoding.
pub const MAX_DISPLAY_SCENE_ARTIFACT_BYTES: u64 = DISPLAY_SCENE_ARTIFACT_HEADER_BYTES
    + MAX_DISPLAY_SCENE_MANIFEST_BYTES
    + DISPLAY_SCENE_CHUNK_HEADER_BYTES * MAX_DISPLAY_SCENE_CHUNKS as u64
    + MAX_DISPLAY_SCENE_BYTES;

/// Native-owned encoded bytes for one manifest-described display chunk.
#[derive(Debug, Eq, PartialEq)]
pub struct DisplayMeshChunkPayload {
    sequence: u32,
    vertex_bytes: Box<[u8]>,
    index_bytes: Box<[u8]>,
}

impl DisplayMeshChunkPayload {
    /// Creates an unvalidated payload. `decode_display_scene` is the authority boundary.
    pub fn new(
        sequence: u32,
        vertex_bytes: impl Into<Box<[u8]>>,
        index_bytes: impl Into<Box<[u8]>>,
    ) -> Self {
        Self {
            sequence,
            vertex_bytes: vertex_bytes.into(),
            index_bytes: index_bytes.into(),
        }
    }
}

#[derive(Clone, Copy)]
struct DisplayMeshChunkPayloadRef<'a> {
    sequence: u32,
    vertex_bytes: &'a [u8],
    index_bytes: &'a [u8],
}

impl DisplayMeshChunkPayload {
    fn as_ref(&self) -> DisplayMeshChunkPayloadRef<'_> {
        DisplayMeshChunkPayloadRef {
            sequence: self.sequence,
            vertex_bytes: &self.vertex_bytes,
            index_bytes: &self.index_bytes,
        }
    }
}

/// Content-minimized reason a native display payload was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisplaySceneDecodeError {
    /// The caller cancelled before validation completed.
    Cancelled,
    /// The manifest does not belong to the current authorized source.
    SourceBindingMismatch,
    /// The manifest does not belong to the current accepted analysis output.
    AnalysisBindingMismatch,
    /// A controlled output reference does not identify the display-scene artifact.
    ControlledReferenceMismatch,
    /// The complete framed artifact exceeds its reviewed limit.
    ArtifactTooLarge,
    /// The artifact magic, schema, flags, or declared counts are unsupported.
    ArtifactHeaderInvalid,
    /// The embedded manifest is absent, oversized, malformed, or unsupported.
    ManifestInvalid,
    /// Framed bytes end before a declared field or payload is complete.
    ArtifactTruncated,
    /// Bytes remain after the exact declared artifact payload.
    ArtifactTrailingBytes,
    /// A bounded native allocation could not be reserved.
    AllocationFailed,
    /// The payload and manifest contain different numbers of chunks.
    ChunkCountMismatch,
    /// A payload sequence does not match its manifest descriptor.
    ChunkSequenceMismatch,
    /// Vertex bytes do not match the exact declared length.
    VertexLengthMismatch,
    /// Index bytes do not match the exact declared length.
    IndexLengthMismatch,
    /// Vertex bytes do not match their declared digest.
    VertexHashMismatch,
    /// Index bytes do not match their declared digest.
    IndexHashMismatch,
    /// A decoded coordinate is NaN or infinite.
    NonFinitePosition,
    /// A triangle index does not identify a vertex in the same chunk.
    IndexOutOfRange,
}

impl fmt::Display for DisplaySceneDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Cancelled => "display-scene validation was cancelled",
            Self::SourceBindingMismatch => "display-scene source binding did not match",
            Self::AnalysisBindingMismatch => "display-scene analysis binding did not match",
            Self::ControlledReferenceMismatch => {
                "controlled output did not identify a display-scene artifact"
            }
            Self::ArtifactTooLarge => "display-scene artifact exceeded its reviewed limit",
            Self::ArtifactHeaderInvalid => "display-scene artifact header was unsupported",
            Self::ManifestInvalid => "display-scene artifact manifest was invalid",
            Self::ArtifactTruncated => "display-scene artifact was truncated",
            Self::ArtifactTrailingBytes => "display-scene artifact contained trailing bytes",
            Self::AllocationFailed => "display-scene native allocation failed",
            Self::ChunkCountMismatch => "display-scene chunk count did not match",
            Self::ChunkSequenceMismatch => "display-scene chunk sequence did not match",
            Self::VertexLengthMismatch => "display-scene vertex length did not match",
            Self::IndexLengthMismatch => "display-scene index length did not match",
            Self::VertexHashMismatch => "display-scene vertex hash did not match",
            Self::IndexHashMismatch => "display-scene index hash did not match",
            Self::NonFinitePosition => "display-scene position was not finite",
            Self::IndexOutOfRange => "display-scene index was out of range",
        })
    }
}

impl std::error::Error for DisplaySceneDecodeError {}

/// Renderer-ready native arrays validated against one display-scene chunk descriptor.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidatedDisplayMeshChunk {
    sequence: u32,
    geometry_reference: DisplayGeometryReference,
    positions: Vec<[f32; 3]>,
    triangle_indices: Vec<u32>,
}

impl ValidatedDisplayMeshChunk {
    #[must_use]
    pub const fn sequence(&self) -> u32 {
        self.sequence
    }

    #[must_use]
    pub const fn geometry_reference(&self) -> &DisplayGeometryReference {
        &self.geometry_reference
    }

    #[must_use]
    pub fn positions(&self) -> &[[f32; 3]] {
        &self.positions
    }

    #[must_use]
    pub fn triangle_indices(&self) -> &[u32] {
        &self.triangle_indices
    }
}

/// Fully validated native display scene; deliberately not serializable.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidatedGeometryDisplayScene {
    manifest: GeometryDisplaySceneManifest,
    chunks: Vec<ValidatedDisplayMeshChunk>,
}

impl ValidatedGeometryDisplayScene {
    #[must_use]
    pub const fn manifest(&self) -> &GeometryDisplaySceneManifest {
        &self.manifest
    }

    #[must_use]
    pub fn chunks(&self) -> &[ValidatedDisplayMeshChunk] {
        &self.chunks
    }
}

/// Validates native-owned payload bytes against the exact current manifest bindings.
pub fn decode_display_scene(
    expected_source_hash: &Sha256Digest,
    expected_analysis_output_hash: &Sha256Digest,
    manifest: GeometryDisplaySceneManifest,
    payloads: Vec<DisplayMeshChunkPayload>,
    cancellation: Option<&AtomicBool>,
) -> Result<ValidatedGeometryDisplayScene, DisplaySceneDecodeError> {
    let payload_refs = payloads
        .iter()
        .map(DisplayMeshChunkPayload::as_ref)
        .collect::<Vec<_>>();
    let chunks = decode_display_scene_payloads(
        expected_source_hash,
        expected_analysis_output_hash,
        &manifest,
        &payload_refs,
        cancellation,
    )?;
    Ok(ValidatedGeometryDisplayScene { manifest, chunks })
}

fn decode_display_scene_payloads(
    expected_source_hash: &Sha256Digest,
    expected_analysis_output_hash: &Sha256Digest,
    manifest: &GeometryDisplaySceneManifest,
    payloads: &[DisplayMeshChunkPayloadRef<'_>],
    cancellation: Option<&AtomicBool>,
) -> Result<Vec<ValidatedDisplayMeshChunk>, DisplaySceneDecodeError> {
    check_cancelled(cancellation)?;
    if manifest.source_hash() != expected_source_hash {
        return Err(DisplaySceneDecodeError::SourceBindingMismatch);
    }
    if manifest.analysis_output_hash() != expected_analysis_output_hash {
        return Err(DisplaySceneDecodeError::AnalysisBindingMismatch);
    }
    if manifest.chunks().len() != payloads.len() {
        return Err(DisplaySceneDecodeError::ChunkCountMismatch);
    }

    let mut chunks = Vec::new();
    chunks
        .try_reserve_exact(payloads.len())
        .map_err(|_| DisplaySceneDecodeError::AllocationFailed)?;
    for (descriptor, payload) in manifest.chunks().iter().zip(payloads.iter().copied()) {
        check_cancelled(cancellation)?;
        chunks.push(decode_chunk(descriptor, payload, cancellation)?);
    }
    Ok(chunks)
}

fn decode_chunk(
    descriptor: &DisplayMeshChunkDescriptor,
    payload: DisplayMeshChunkPayloadRef<'_>,
    cancellation: Option<&AtomicBool>,
) -> Result<ValidatedDisplayMeshChunk, DisplaySceneDecodeError> {
    if payload.sequence != descriptor.sequence() {
        return Err(DisplaySceneDecodeError::ChunkSequenceMismatch);
    }
    if u64::try_from(payload.vertex_bytes.len()).ok() != Some(descriptor.vertex_byte_length()) {
        return Err(DisplaySceneDecodeError::VertexLengthMismatch);
    }
    if u64::try_from(payload.index_bytes.len()).ok() != Some(descriptor.index_byte_length()) {
        return Err(DisplaySceneDecodeError::IndexLengthMismatch);
    }
    if !hash_matches(
        payload.vertex_bytes,
        descriptor.vertex_sha256(),
        cancellation,
    )? {
        return Err(DisplaySceneDecodeError::VertexHashMismatch);
    }
    if !hash_matches(payload.index_bytes, descriptor.index_sha256(), cancellation)? {
        return Err(DisplaySceneDecodeError::IndexHashMismatch);
    }

    let position_capacity = usize::try_from(descriptor.vertex_count())
        .map_err(|_| DisplaySceneDecodeError::AllocationFailed)?;
    let mut positions = Vec::new();
    positions
        .try_reserve_exact(position_capacity)
        .map_err(|_| DisplaySceneDecodeError::AllocationFailed)?;
    for (index, bytes) in payload.vertex_bytes.chunks_exact(12).enumerate() {
        if index % CANCELLATION_POLL_ITEMS == 0 {
            check_cancelled(cancellation)?;
        }
        let position = [
            f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            f32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
        ];
        if position.iter().any(|coordinate| !coordinate.is_finite()) {
            return Err(DisplaySceneDecodeError::NonFinitePosition);
        }
        positions.push(position);
    }

    let index_capacity = usize::try_from(u64::from(descriptor.triangle_count()) * 3)
        .map_err(|_| DisplaySceneDecodeError::IndexLengthMismatch)?;
    let mut triangle_indices = Vec::new();
    triangle_indices
        .try_reserve_exact(index_capacity)
        .map_err(|_| DisplaySceneDecodeError::AllocationFailed)?;
    for (offset, bytes) in payload.index_bytes.chunks_exact(4).enumerate() {
        if offset % CANCELLATION_POLL_ITEMS == 0 {
            check_cancelled(cancellation)?;
        }
        let index = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if index >= descriptor.vertex_count() {
            return Err(DisplaySceneDecodeError::IndexOutOfRange);
        }
        triangle_indices.push(index);
    }

    Ok(ValidatedDisplayMeshChunk {
        sequence: payload.sequence,
        geometry_reference: descriptor.geometry_reference().clone(),
        positions,
        triangle_indices,
    })
}

/// Produces the exact v1 single-file worker artifact after validating every chunk.
pub fn encode_display_scene_artifact(
    manifest: &GeometryDisplaySceneManifest,
    payloads: &[DisplayMeshChunkPayload],
    cancellation: Option<&AtomicBool>,
) -> Result<Box<[u8]>, DisplaySceneDecodeError> {
    let payload_refs = payloads
        .iter()
        .map(DisplayMeshChunkPayload::as_ref)
        .collect::<Vec<_>>();
    let validated = decode_display_scene_payloads(
        manifest.source_hash(),
        manifest.analysis_output_hash(),
        manifest,
        &payload_refs,
        cancellation,
    )?;
    drop(validated);

    let manifest_bytes =
        serde_json::to_vec(manifest).map_err(|_| DisplaySceneDecodeError::ManifestInvalid)?;
    let manifest_length = u64::try_from(manifest_bytes.len())
        .map_err(|_| DisplaySceneDecodeError::ManifestInvalid)?;
    if manifest_length == 0 || manifest_length > MAX_DISPLAY_SCENE_MANIFEST_BYTES {
        return Err(DisplaySceneDecodeError::ManifestInvalid);
    }
    let chunk_count = u32::try_from(payloads.len())
        .map_err(|_| DisplaySceneDecodeError::ArtifactHeaderInvalid)?;
    let payload_length = payloads.iter().try_fold(0_u64, |total, payload| {
        let vertex_length = u64::try_from(payload.vertex_bytes.len())
            .map_err(|_| DisplaySceneDecodeError::ArtifactTooLarge)?;
        let index_length = u64::try_from(payload.index_bytes.len())
            .map_err(|_| DisplaySceneDecodeError::ArtifactTooLarge)?;
        total
            .checked_add(DISPLAY_SCENE_CHUNK_HEADER_BYTES)
            .and_then(|value| value.checked_add(vertex_length))
            .and_then(|value| value.checked_add(index_length))
            .ok_or(DisplaySceneDecodeError::ArtifactTooLarge)
    })?;
    let artifact_length = DISPLAY_SCENE_ARTIFACT_HEADER_BYTES
        .checked_add(manifest_length)
        .and_then(|value| value.checked_add(payload_length))
        .ok_or(DisplaySceneDecodeError::ArtifactTooLarge)?;
    if artifact_length > MAX_DISPLAY_SCENE_ARTIFACT_BYTES {
        return Err(DisplaySceneDecodeError::ArtifactTooLarge);
    }

    let capacity =
        usize::try_from(artifact_length).map_err(|_| DisplaySceneDecodeError::ArtifactTooLarge)?;
    let mut artifact = Vec::new();
    artifact
        .try_reserve_exact(capacity)
        .map_err(|_| DisplaySceneDecodeError::AllocationFailed)?;
    artifact.extend_from_slice(&DISPLAY_SCENE_ARTIFACT_MAGIC);
    artifact.extend_from_slice(&DISPLAY_SCENE_ARTIFACT_SCHEMA_VERSION.to_le_bytes());
    artifact.extend_from_slice(&0_u16.to_le_bytes());
    artifact.extend_from_slice(
        &u32::try_from(manifest_bytes.len())
            .map_err(|_| DisplaySceneDecodeError::ManifestInvalid)?
            .to_le_bytes(),
    );
    artifact.extend_from_slice(&chunk_count.to_le_bytes());
    artifact.extend_from_slice(&manifest_bytes);
    for payload in payloads {
        artifact.extend_from_slice(&payload.sequence.to_le_bytes());
        artifact.extend_from_slice(
            &u64::try_from(payload.vertex_bytes.len())
                .map_err(|_| DisplaySceneDecodeError::ArtifactTooLarge)?
                .to_le_bytes(),
        );
        artifact.extend_from_slice(
            &u64::try_from(payload.index_bytes.len())
                .map_err(|_| DisplaySceneDecodeError::ArtifactTooLarge)?
                .to_le_bytes(),
        );
        artifact.extend_from_slice(&payload.vertex_bytes);
        artifact.extend_from_slice(&payload.index_bytes);
    }
    debug_assert_eq!(artifact.len(), capacity);
    Ok(artifact.into_boxed_slice())
}

/// Decodes one supervisor-claimed v1 display artifact without exposing its bytes.
pub fn decode_controlled_display_scene_artifact(
    output: &ControlledWorkerOutput,
    expected_source_hash: &Sha256Digest,
    expected_analysis_output_hash: &Sha256Digest,
    cancellation: Option<&AtomicBool>,
) -> Result<ValidatedGeometryDisplayScene, DisplaySceneDecodeError> {
    check_cancelled(cancellation)?;
    if output.snapshot_reference().as_str() != GEOMETRY_DISPLAY_SCENE_REFERENCE {
        return Err(DisplaySceneDecodeError::ControlledReferenceMismatch);
    }
    if output.byte_length() > MAX_DISPLAY_SCENE_ARTIFACT_BYTES {
        return Err(DisplaySceneDecodeError::ArtifactTooLarge);
    }

    let mut cursor = ArtifactCursor::new(output.bytes());
    if cursor.read_exact(DISPLAY_SCENE_ARTIFACT_MAGIC.len())? != DISPLAY_SCENE_ARTIFACT_MAGIC {
        return Err(DisplaySceneDecodeError::ArtifactHeaderInvalid);
    }
    let schema_version = cursor.read_u16()?;
    let flags = cursor.read_u16()?;
    let manifest_length = u64::from(cursor.read_u32()?);
    let chunk_count = usize::try_from(cursor.read_u32()?)
        .map_err(|_| DisplaySceneDecodeError::ArtifactHeaderInvalid)?;
    if schema_version != DISPLAY_SCENE_ARTIFACT_SCHEMA_VERSION
        || flags != 0
        || manifest_length == 0
        || manifest_length > MAX_DISPLAY_SCENE_MANIFEST_BYTES
        || chunk_count > MAX_DISPLAY_SCENE_CHUNKS
    {
        return Err(DisplaySceneDecodeError::ArtifactHeaderInvalid);
    }

    let manifest_bytes = cursor.read_exact(
        usize::try_from(manifest_length).map_err(|_| DisplaySceneDecodeError::ManifestInvalid)?,
    )?;
    let manifest: GeometryDisplaySceneManifest = serde_json::from_slice(manifest_bytes)
        .map_err(|_| DisplaySceneDecodeError::ManifestInvalid)?;
    let mut payloads = Vec::new();
    payloads
        .try_reserve_exact(chunk_count)
        .map_err(|_| DisplaySceneDecodeError::AllocationFailed)?;
    let mut aggregate_payload_bytes = 0_u64;
    for _ in 0..chunk_count {
        check_cancelled(cancellation)?;
        let sequence = cursor.read_u32()?;
        let vertex_length = cursor.read_u64()?;
        let index_length = cursor.read_u64()?;
        aggregate_payload_bytes = aggregate_payload_bytes
            .checked_add(vertex_length)
            .and_then(|value| value.checked_add(index_length))
            .ok_or(DisplaySceneDecodeError::ArtifactTooLarge)?;
        if aggregate_payload_bytes > MAX_DISPLAY_SCENE_BYTES {
            return Err(DisplaySceneDecodeError::ArtifactTooLarge);
        }
        let vertex_bytes = cursor.read_exact(
            usize::try_from(vertex_length)
                .map_err(|_| DisplaySceneDecodeError::ArtifactTooLarge)?,
        )?;
        let index_bytes = cursor.read_exact(
            usize::try_from(index_length).map_err(|_| DisplaySceneDecodeError::ArtifactTooLarge)?,
        )?;
        payloads.push(DisplayMeshChunkPayloadRef {
            sequence,
            vertex_bytes,
            index_bytes,
        });
    }
    if !cursor.is_finished() {
        return Err(DisplaySceneDecodeError::ArtifactTrailingBytes);
    }

    let chunks = decode_display_scene_payloads(
        expected_source_hash,
        expected_analysis_output_hash,
        &manifest,
        &payloads,
        cancellation,
    )?;
    Ok(ValidatedGeometryDisplayScene { manifest, chunks })
}

struct ArtifactCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> ArtifactCursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_exact(&mut self, length: usize) -> Result<&'a [u8], DisplaySceneDecodeError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(DisplaySceneDecodeError::ArtifactTruncated)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(DisplaySceneDecodeError::ArtifactTruncated)?;
        self.offset = end;
        Ok(value)
    }

    fn read_u16(&mut self) -> Result<u16, DisplaySceneDecodeError> {
        let bytes = self.read_exact(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32(&mut self) -> Result<u32, DisplaySceneDecodeError> {
        let bytes = self.read_exact(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u64(&mut self) -> Result<u64, DisplaySceneDecodeError> {
        let bytes = self.read_exact(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    const fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

fn hash_matches(
    bytes: &[u8],
    expected: &Sha256Digest,
    cancellation: Option<&AtomicBool>,
) -> Result<bool, DisplaySceneDecodeError> {
    let mut hasher = Sha256::new();
    for chunk in bytes.chunks(HASH_POLL_BYTES) {
        check_cancelled(cancellation)?;
        hasher.update(chunk);
    }
    let mut actual = String::with_capacity(64);
    for byte in hasher.finalize() {
        write!(&mut actual, "{byte:02x}").expect("writing to a String cannot fail");
    }
    Ok(actual == expected.as_str())
}

fn check_cancelled(cancellation: Option<&AtomicBool>) -> Result<(), DisplaySceneDecodeError> {
    if cancellation.is_some_and(|flag| flag.load(Ordering::Acquire)) {
        return Err(DisplaySceneDecodeError::Cancelled);
    }
    Ok(())
}
