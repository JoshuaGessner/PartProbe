use std::fmt::Write as _;
use std::sync::atomic::AtomicBool;

use partprobe_geometry_core::{
    DisplayCoordinateSpace, DisplayGeometryReference, DisplayMeshChunkDescriptor,
    DisplayTessellationProfile, GeometryDisplaySceneManifest, ModelLengthUnit,
    ProvisionalGeometryDecimal, RepresentationBasis, Sha256Digest,
};
use partprobe_geometry_import::{
    ControlledWorkerOutput, DisplayMeshChunkPayload, DisplaySceneDecodeError, SnapshotReference,
    decode_controlled_display_scene_artifact, decode_display_scene, encode_display_scene_artifact,
};
use sha2::{Digest, Sha256};

fn hash(bytes: &[u8]) -> Sha256Digest {
    let mut value = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(&mut value, "{byte:02x}").expect("writing to a String cannot fail");
    }
    Sha256Digest::new(value).expect("hash must be valid")
}

fn source_hash() -> Sha256Digest {
    Sha256Digest::new("a".repeat(64)).expect("hash must be valid")
}

fn analysis_hash() -> Sha256Digest {
    Sha256Digest::new("b".repeat(64)).expect("hash must be valid")
}

fn cube_payload() -> (Vec<u8>, Vec<u8>) {
    let positions = [
        [0.0_f32, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0],
        [0.0, 10.0, 0.0],
        [0.0, 0.0, 10.0],
        [10.0, 0.0, 10.0],
        [10.0, 10.0, 10.0],
        [0.0, 10.0, 10.0],
    ];
    let indices = [
        0_u32, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 1, 2, 6, 1, 6, 5, 2, 3, 7, 2, 7,
        6, 3, 0, 4, 3, 4, 7,
    ];
    let vertex_bytes = positions
        .iter()
        .flat_map(|position| position.iter().flat_map(|value| value.to_le_bytes()))
        .collect();
    let index_bytes = indices
        .iter()
        .flat_map(|index| index.to_le_bytes())
        .collect();
    (vertex_bytes, index_bytes)
}

fn scene_for(vertex_bytes: &[u8], index_bytes: &[u8]) -> GeometryDisplaySceneManifest {
    let vertex_count = u32::try_from(vertex_bytes.len() / 12).expect("count must fit");
    let triangle_count = u32::try_from(index_bytes.len() / 12).expect("count must fit");
    let chunk = DisplayMeshChunkDescriptor::new(
        0,
        DisplayGeometryReference::new("body-0").expect("reference must be valid"),
        vertex_count,
        triangle_count,
        hash(vertex_bytes),
        hash(index_bytes),
    )
    .expect("descriptor must be valid");
    GeometryDisplaySceneManifest::new(
        source_hash(),
        analysis_hash(),
        RepresentationBasis::ExactBrep,
        DisplayCoordinateSpace::CanonicalMillimeters,
        ModelLengthUnit::Millimeter,
        DisplayTessellationProfile::new(
            ProvisionalGeometryDecimal::new("0.1").expect("value must be valid"),
            ProvisionalGeometryDecimal::new("12").expect("value must be valid"),
        )
        .expect("profile must be valid"),
        ["10", "10", "10"]
            .map(|value| ProvisionalGeometryDecimal::new(value).expect("extent must be valid")),
        vec![chunk],
    )
    .expect("manifest must be valid")
}

fn decode(
    manifest: GeometryDisplaySceneManifest,
    vertex_bytes: Vec<u8>,
    index_bytes: Vec<u8>,
) -> Result<partprobe_geometry_import::ValidatedGeometryDisplayScene, DisplaySceneDecodeError> {
    decode_display_scene(
        &source_hash(),
        &analysis_hash(),
        manifest,
        vec![DisplayMeshChunkPayload::new(0, vertex_bytes, index_bytes)],
        None,
    )
}

fn controlled_output(reference: &str, bytes: Box<[u8]>) -> ControlledWorkerOutput {
    ControlledWorkerOutput::from_claimed_parts(
        SnapshotReference::new(reference).expect("reference must be valid"),
        hash(&bytes),
        bytes,
    )
    .expect("controlled output must be valid")
}

#[test]
fn exact_cube_payload_decodes_to_native_renderer_arrays() {
    let (vertex_bytes, index_bytes) = cube_payload();
    let scene = decode(
        scene_for(&vertex_bytes, &index_bytes),
        vertex_bytes,
        index_bytes,
    )
    .expect("valid scene must decode");

    assert_eq!(scene.manifest().source_hash(), &source_hash());
    assert_eq!(scene.chunks().len(), 1);
    assert_eq!(scene.chunks()[0].sequence(), 0);
    assert_eq!(scene.chunks()[0].geometry_reference().as_str(), "body-0");
    assert_eq!(scene.chunks()[0].positions().len(), 8);
    assert_eq!(scene.chunks()[0].triangle_indices().len(), 36);
    assert_eq!(scene.chunks()[0].positions()[6], [10.0, 10.0, 10.0]);
}

#[test]
fn decoder_rejects_stale_source_or_analysis_and_cancellation() {
    let (vertex_bytes, index_bytes) = cube_payload();
    let manifest = scene_for(&vertex_bytes, &index_bytes);
    assert_eq!(
        decode_display_scene(
            &Sha256Digest::new("c".repeat(64)).expect("hash must be valid"),
            &analysis_hash(),
            manifest,
            vec![DisplayMeshChunkPayload::new(
                0,
                vertex_bytes.clone(),
                index_bytes.clone(),
            )],
            None,
        ),
        Err(DisplaySceneDecodeError::SourceBindingMismatch)
    );

    let manifest = scene_for(&vertex_bytes, &index_bytes);
    assert_eq!(
        decode_display_scene(
            &source_hash(),
            &Sha256Digest::new("c".repeat(64)).expect("hash must be valid"),
            manifest,
            vec![DisplayMeshChunkPayload::new(
                0,
                vertex_bytes.clone(),
                index_bytes.clone(),
            )],
            None,
        ),
        Err(DisplaySceneDecodeError::AnalysisBindingMismatch)
    );

    let cancelled = AtomicBool::new(true);
    assert_eq!(
        decode_display_scene(
            &source_hash(),
            &analysis_hash(),
            scene_for(&vertex_bytes, &index_bytes),
            vec![DisplayMeshChunkPayload::new(0, vertex_bytes, index_bytes,)],
            Some(&cancelled),
        ),
        Err(DisplaySceneDecodeError::Cancelled)
    );
}

#[test]
fn decoder_rejects_length_hash_sequence_and_chunk_count_mismatches() {
    let (vertex_bytes, index_bytes) = cube_payload();
    let manifest = scene_for(&vertex_bytes, &index_bytes);
    assert_eq!(
        decode_display_scene(&source_hash(), &analysis_hash(), manifest, Vec::new(), None,),
        Err(DisplaySceneDecodeError::ChunkCountMismatch)
    );

    let manifest = scene_for(&vertex_bytes, &index_bytes);
    assert_eq!(
        decode_display_scene(
            &source_hash(),
            &analysis_hash(),
            manifest,
            vec![DisplayMeshChunkPayload::new(
                1,
                vertex_bytes.clone(),
                index_bytes.clone(),
            )],
            None,
        ),
        Err(DisplaySceneDecodeError::ChunkSequenceMismatch)
    );

    let manifest = scene_for(&vertex_bytes, &index_bytes);
    let mut short_vertices = vertex_bytes.clone();
    short_vertices.pop();
    assert_eq!(
        decode(manifest, short_vertices, index_bytes.clone()),
        Err(DisplaySceneDecodeError::VertexLengthMismatch)
    );

    let manifest = scene_for(&vertex_bytes, &index_bytes);
    let mut short_indices = index_bytes.clone();
    short_indices.pop();
    assert_eq!(
        decode(manifest, vertex_bytes.clone(), short_indices),
        Err(DisplaySceneDecodeError::IndexLengthMismatch)
    );

    let manifest = scene_for(&vertex_bytes, &index_bytes);
    let mut changed_vertices = vertex_bytes.clone();
    changed_vertices[0] ^= 1;
    assert_eq!(
        decode(manifest, changed_vertices, index_bytes.clone()),
        Err(DisplaySceneDecodeError::VertexHashMismatch)
    );

    let manifest = scene_for(&vertex_bytes, &index_bytes);
    let mut changed_indices = index_bytes;
    changed_indices[0] ^= 1;
    assert_eq!(
        decode(manifest, vertex_bytes, changed_indices),
        Err(DisplaySceneDecodeError::IndexHashMismatch)
    );
}

#[test]
fn decoder_rejects_nonfinite_positions_and_out_of_range_indices_after_hashing() {
    let (mut vertex_bytes, index_bytes) = cube_payload();
    vertex_bytes[0..4].copy_from_slice(&f32::NAN.to_le_bytes());
    let manifest = scene_for(&vertex_bytes, &index_bytes);
    assert_eq!(
        decode(manifest, vertex_bytes, index_bytes),
        Err(DisplaySceneDecodeError::NonFinitePosition)
    );

    let (vertex_bytes, mut index_bytes) = cube_payload();
    index_bytes[0..4].copy_from_slice(&8_u32.to_le_bytes());
    let manifest = scene_for(&vertex_bytes, &index_bytes);
    assert_eq!(
        decode(manifest, vertex_bytes, index_bytes),
        Err(DisplaySceneDecodeError::IndexOutOfRange)
    );
}

#[test]
fn controlled_display_scene_artifact_round_trips_exact_framing() {
    let (vertex_bytes, index_bytes) = cube_payload();
    let manifest = scene_for(&vertex_bytes, &index_bytes);
    let payloads = vec![DisplayMeshChunkPayload::new(0, vertex_bytes, index_bytes)];
    let artifact =
        encode_display_scene_artifact(&manifest, &payloads, None).expect("artifact must encode");
    let output = controlled_output("geometry-display-scene-v1", artifact);

    let scene =
        decode_controlled_display_scene_artifact(&output, &source_hash(), &analysis_hash(), None)
            .expect("claimed artifact must decode");
    assert_eq!(scene.manifest(), &manifest);
    assert_eq!(scene.chunks()[0].geometry_reference().as_str(), "body-0");
    assert_eq!(scene.chunks()[0].positions().len(), 8);
    assert_eq!(scene.chunks()[0].triangle_indices().len(), 36);
}

#[test]
fn controlled_display_scene_artifact_rejects_reference_header_and_manifest_tampering() {
    let (vertex_bytes, index_bytes) = cube_payload();
    let manifest = scene_for(&vertex_bytes, &index_bytes);
    let payloads = vec![DisplayMeshChunkPayload::new(0, vertex_bytes, index_bytes)];
    let artifact =
        encode_display_scene_artifact(&manifest, &payloads, None).expect("artifact must encode");

    let wrong_reference = controlled_output("geometry-snapshot-v1", artifact.clone());
    assert_eq!(
        decode_controlled_display_scene_artifact(
            &wrong_reference,
            &source_hash(),
            &analysis_hash(),
            None,
        ),
        Err(DisplaySceneDecodeError::ControlledReferenceMismatch)
    );

    let mut bad_magic = artifact.clone();
    bad_magic[0] ^= 1;
    let bad_magic = controlled_output("geometry-display-scene-v1", bad_magic);
    assert_eq!(
        decode_controlled_display_scene_artifact(
            &bad_magic,
            &source_hash(),
            &analysis_hash(),
            None,
        ),
        Err(DisplaySceneDecodeError::ArtifactHeaderInvalid)
    );

    let mut bad_manifest = artifact;
    bad_manifest[20] = b'!';
    let bad_manifest = controlled_output("geometry-display-scene-v1", bad_manifest);
    assert_eq!(
        decode_controlled_display_scene_artifact(
            &bad_manifest,
            &source_hash(),
            &analysis_hash(),
            None,
        ),
        Err(DisplaySceneDecodeError::ManifestInvalid)
    );
}

#[test]
fn controlled_display_scene_artifact_rejects_truncation_trailing_bytes_and_huge_spans() {
    let (vertex_bytes, index_bytes) = cube_payload();
    let manifest = scene_for(&vertex_bytes, &index_bytes);
    let payloads = vec![DisplayMeshChunkPayload::new(0, vertex_bytes, index_bytes)];
    let artifact =
        encode_display_scene_artifact(&manifest, &payloads, None).expect("artifact must encode");

    let mut truncated = artifact.clone().into_vec();
    truncated.pop();
    let truncated = controlled_output("geometry-display-scene-v1", truncated.into_boxed_slice());
    assert_eq!(
        decode_controlled_display_scene_artifact(
            &truncated,
            &source_hash(),
            &analysis_hash(),
            None,
        ),
        Err(DisplaySceneDecodeError::ArtifactTruncated)
    );

    let mut trailing = artifact.clone().into_vec();
    trailing.push(0);
    let trailing = controlled_output("geometry-display-scene-v1", trailing.into_boxed_slice());
    assert_eq!(
        decode_controlled_display_scene_artifact(&trailing, &source_hash(), &analysis_hash(), None,),
        Err(DisplaySceneDecodeError::ArtifactTrailingBytes)
    );

    let mut huge_span = artifact.into_vec();
    let manifest_length =
        u32::from_le_bytes([huge_span[12], huge_span[13], huge_span[14], huge_span[15]]) as usize;
    let vertex_length_offset = 20 + manifest_length + 4;
    huge_span[vertex_length_offset..vertex_length_offset + 8]
        .copy_from_slice(&u64::MAX.to_le_bytes());
    let huge_span = controlled_output("geometry-display-scene-v1", huge_span.into_boxed_slice());
    assert_eq!(
        decode_controlled_display_scene_artifact(
            &huge_span,
            &source_hash(),
            &analysis_hash(),
            None,
        ),
        Err(DisplaySceneDecodeError::ArtifactTooLarge)
    );
}
