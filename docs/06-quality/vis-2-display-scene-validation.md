# VIS-2 display-scene validation

> **Status:** In Progress
> **Last updated:** 2026-09-12
> **Related requirements:** REQ-F-022–REQ-F-024, REQ-NF-011, REQ-NF-013, GEO-003, GEO-007, GEO-010, SEC-004
> **Related ADRs:** ADR-0003, ADR-0005
> **Open questions:** STEP tessellation implementation, progressive levels, and stock-placement policy
> **Dependencies:** `docs/04-architecture/model-viewer.md`, `docs/04-architecture/geometry-engine.md`, `docs/06-quality/vis-1-renderer-validation.md`
> **Supersedes:** None

## Scope

The current VIS-2 checkpoints add the metadata contract for `geometry-display-scene-v1` in `partprobe-geometry-core`; native chunk validation, deterministic controlled-artifact framing, worker request/response schema v2, and dual-output supervisor claiming in `partprobe-geometry-import`; an explicit worker fallback while tessellation is unavailable; and a native application-session retention seam. They remain native-only and display-only. The manifest does not contain paths, CAD bytes, filenames, renderer handles, stock placement, feature authority, or estimating authority, and neither the manifest nor its vertex/index buffers cross the desktop WebView contract.

The manifest binds one display derivative to both the immutable source hash and the accepted analysis-output hash. It records representation, coordinate space, units, a fixed versioned tessellation profile and its positive linear/angular tolerances, positive AABB extents, fixed binary layouts, exact totals, and ordered per-chunk bounded path-free snapshot-scoped geometry references plus buffer hashes and lengths.

## Version and migration decision

- New schema/reference: `geometry-display-scene-v1`, schema version 1.
- New binary artifact framing: display-scene artifact schema version 1.
- New worker request/response schema version 2. Schema v1 remains readable and serializes without display fields; unknown versions fail closed.
- Schema v2 may carry one optional `display_scene` request with the exact artifact reference and governed tessellation profile, plus one separately typed optional `display_scene_reference` in the response. It never replaces `snapshot_reference`.
- Evidence state: `display_only`.
- Tessellation profile: `partprobe-display-tessellation-v1`, version 1.0.0.
- Position layout: finite little-endian `f32 × 3`; index layout: little-endian `u32`; primitive topology: triangle list.
- This is additive. It does not modify or migrate `geometry-snapshot-v1`, `geometry-step-analysis-v1`, or `geometry-mesh-snapshot-v1`.
- No persisted customer/display artifact migration exists because the worker does not emit a display artifact yet. A future accepted-boundary, framing, layout, reference, coordinate, profile, error, or limit change requires a new schema/profile/artifact decision and replay tests.

## Reviewed comparison ceilings

| Limit | Schema-v1 ceiling |
|---|---:|
| Chunks | 32 |
| Vertices | 1,000,000 |
| Triangles | 2,000,000 |
| Combined vertex/index bytes | 32 MiB |

These are developer comparison ceilings, not production performance targets or evidence that representative shop models fit in memory. Every chunk must be nonempty, carry a 1–128 character path-free ASCII geometry reference, start at sequence zero, continue without gaps, and match the fixed byte lengths implied by its counts. Manifest totals must exactly reproduce the ordered descriptors. A reference identifies the snapshot-scoped body or display region represented by that chunk; it is not a pathname or a transient triangle index.

Exact-B-rep display scenes require canonical millimetres. Mesh scenes may use canonical millimetres or explicitly unresolved source coordinates with `Unknown` physical units. `Unknown` representation, false unit conversion, nonpositive extents, unsupported schema/reference/profile values, unknown fields, count/length tampering, noncontiguous chunks, and aggregate limit violations fail closed.

## Controlled artifact framing

Artifact schema v1 is one deterministic native-only file. Worker schema v2 assigns it the fixed host-owned name `partprobe-display-scene.bin`; the authoritative analysis retains `partprobe-output.json`. The supervisor reuses its no-follow claim, immutable read/hash, removal, and private-workspace cleanup controls. Its byte order and sequence are exact:

1. Eight-byte ASCII magic `PPDSCENE`.
2. Little-endian unsigned 16-bit artifact schema version `1`.
3. Little-endian unsigned 16-bit flags, required to be zero.
4. Little-endian unsigned 32-bit manifest byte length.
5. Little-endian unsigned 32-bit chunk count.
6. Exact JSON bytes for the validated schema-v1 manifest.
7. For every ordered chunk: little-endian unsigned 32-bit sequence, unsigned 64-bit vertex length, unsigned 64-bit index length, exact vertex bytes, then exact index bytes.
8. End of file; trailing bytes are rejected.

The manifest is capped at 64 KiB. The complete artifact is capped at 33,620,628 bytes: the 32 MiB raw-payload ceiling plus the maximum manifest, fixed header, and 32 chunk headers. Every addition/span and host-size conversion is checked before slicing or reserving memory. The artifact is accepted only through `ControlledWorkerOutput` with the exact `geometry-display-scene-v1` reference, whose constructor or supervisor claim already verifies the complete-file SHA-256.

The supervisor claims the authoritative output first and subtracts its exact claimed byte length before claiming the display artifact, so the two files share the request's one aggregate output quota. Any claim failure removes both worker-visible names and returns no partial execution. A display file without its exact response reference is removed rather than retained. Successful schema-v2 responses must either supply the exact requested display reference or return `succeeded_with_warnings` plus `DISPLAY_SCENE_UNAVAILABLE`; unsolicited, contradictory, mismatched, or silently omitted display evidence is malformed.

## Native payload decoder

`geometry-import::decode_display_scene` accepts already-owned native chunk bytes plus the exact current source and analysis-output hashes. Before returning renderer-ready arrays it requires manifest/payload chunk parity, exact sequence/length/hash evidence, finite little-endian positions, indices within the same chunk's vertex range, retained geometry references, and an uncancelled revision. Hashing and element decoding poll the cooperative cancellation flag. Its validated scene and chunk types deliberately do not implement serialization.

The decoder does not read files, reopen paths, allocate GPU objects, accept renderer handles, or expose its arrays to the desktop contract. When a display artifact is supplied, `DraftEstimateApplication` decodes it against the current source hash and the exact claimed analysis-output hash, then retains only the validated native scene in the ephemeral session. Successful worker diagnostics are also retained so explicit display unavailability cannot disappear at the application boundary.

## Executable evidence

```sh
cargo test -p partprobe-geometry-core --locked
cargo clippy -p partprobe-geometry-core --all-targets --locked -- -D warnings
cargo test -p partprobe-geometry-import --test display_scene --locked
cargo clippy -p partprobe-geometry-import --all-targets --locked -- -D warnings
```

Thirteen focused geometry-core contract tests pass, including three display-scene tests. They prove a valid source/analysis-bound manifest round trip, preservation of the stable `body-0` test reference, rejection of a path-like reference, absence of raw geometry/path fields, fixed layout metadata, exact derived lengths/totals, and rejection of false authority, unit mismatch, profile/schema/unknown-field mutation, zero extents, length/sequence/total tampering, and every aggregate ceiling.

Seven focused native decoder/artifact tests use a deterministic analytic 10 mm cube payload. They prove exact artifact and renderer-array round trips with the `body-0` reference, and reject stale source/analysis identity, cancellation, chunk count/sequence mismatches, both vertex/index length and hash mismatches, non-finite positions, out-of-range indices, wrong controlled reference, invalid magic, malformed manifest, truncation, trailing bytes, and oversized declared spans. This in-memory decoder fixture does not replace the governed persisted STEP fixture or establish worker tessellation accuracy.

Twenty protocol tests cover v1 compatibility, path-free schema-v2 requests, exact separately typed response references, unknown-version/reference rejection, explicit-unavailability semantics, and existing control/transport behavior. Five internal supervisor tests include dual output claiming, combined-quota rejection with cleanup, and unreported-display removal. Twenty-five default worker process tests include a real schema-v2 STL request that preserves and validates the authoritative mesh analysis while returning explicit display unavailability and no display file. The application suite proves the display request survives authorized source fingerprinting; the retained-scene decode path compiles behind the typed application service and remains unactivated by the desktop adapter.

## Evidence not established

The running worker accepts schema-v2 display requests but deliberately emits no display artifact yet. Its explicit-unavailability result is negative integration evidence, not tessellation support. Dual-file supervisor claim behavior is proven with bounded synthetic worker-owned bytes; native OCCT still must tessellate the governed STEP fixtures and produce exact cube/prism scenes under the pinned profile. The ordinary desktop adapter still creates schema-v1 requests, so no selected model reaches the viewer. No native worker-emitted display output, source-bound GPU frame, renderer handoff, progressive loading, stock placement, accessibility, device-loss, three-OS package, or representative-corpus evidence is established here.

## Next gate

Implement native STEP tessellation under the exact requested profile, emit the already-framed display artifact, and exercise governed cube/prism output plus cancellation and malformed-artifact failures through the real supervisor/application path. Then activate schema v2 in the native desktop adapter and connect only the fully validated native derivative to the same-window renderer. Govern stock AABB placement, orientation, and per-side allowance interpretation before replacing the synthetic stock overlay.
