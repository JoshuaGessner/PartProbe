# VIS-2 display-scene validation

> **Status:** In Progress
> **Last updated:** 2026-09-13
> **Related requirements:** REQ-F-022–REQ-F-024, REQ-NF-011, REQ-NF-013, GEO-003, GEO-007, GEO-010, SEC-004
> **Related ADRs:** ADR-0003, ADR-0005
> **Open questions:** Progressive levels, representative performance, and stock-placement policy
> **Dependencies:** `docs/04-architecture/model-viewer.md`, `docs/04-architecture/geometry-engine.md`, `docs/06-quality/vis-1-renderer-validation.md`
> **Supersedes:** None

## Scope

The current VIS-2 checkpoints add the metadata contract for `geometry-display-scene-v1` in `partprobe-geometry-core`; native chunk validation, deterministic controlled-artifact framing, worker request/response schema v2, and dual-output supervisor claiming in `partprobe-geometry-import`; native OCCT STEP tessellation under the requested profile; worker emission of one bounded exact-B-rep display artifact; source/analysis-bound application-session retention; and developer-desktop handoff to the same-window native renderer. They remain native-only and display-only. The manifest does not contain paths, CAD bytes, filenames, renderer handles, stock placement, feature authority, or estimating authority, and neither the manifest nor its vertex/index buffers cross the desktop WebView contract.

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
- The established measurement ABI remains version 4. The new display-tessellation ABI is independently versioned as `1`, so display-only evolution cannot silently change authoritative measurement interpretation.
- No persisted customer/display artifact migration exists. The emitted derivative remains session-only and is discarded with the native analysis session. A future accepted-boundary, framing, layout, reference, coordinate, profile, error, or limit change requires a new schema/profile/artifact decision and replay tests.
- Desktop contract v11 is unchanged: its existing optional path-free `scene_reference` can report `geometry-display-scene-v1`, while absence plus a content-minimized notice represents pending or unavailable display. No command, DTO field, permission, persistence, or customer-data migration is introduced.

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

The desktop adapter requests schema v2 only when both the `viewer-spike` feature and exact `--vis1-synthetic-viewer` compatibility flag activate the developer viewer. It uses the fixed 0.1 mm/12 degree profile and the existing combined output ceiling. After analysis, native session state returns a scene only for the exact retained selection and analysis identities. The adapter hands that validated value directly to `partprobe-model-viewer`; no raw-buffer reconstruction or WebView geometry DTO exists.

The renderer concatenates the validated chunks into bounded indexed GPU buffers with checked offsets, frames the source from its actual finite decoded bounds, and derives deterministic display-only vertex shading. A new source clears the previous model before analysis. Stale results cannot replace a newer selection, and any scene-construction or render failure clears source buffers, reports display unavailable, and leaves accepted analysis/estimate state unchanged. Source-bound mode intentionally supplies no stock vertices or stock edges because current estimate evidence has no governed AABB origin, orientation, or per-side placement.

## Native STEP tessellation and emission

The optional OCCT adapter exposes an additive display ABI v1 beside measurement ABI v4. It accepts only the same already-authorized bounded immutable STEP bytes used by the worker, a validated positive profile, explicit vertex/triangle/byte ceilings, and the panic-contained cancellation probe. The bridge transfers the STEP roots, runs `BRepMesh_IncrementalMesh` with the exact linear deflection and degree-to-radian angular deflection from the request, reads each face triangulation with its location transform, preserves reversed-face orientation, and returns one flattened root mesh. Rust owns the result only long enough to copy it into bounded vectors and always releases the matching C++ allocation through the ABI free function.

The worker performs authoritative analysis first, hashes those exact controlled-output bytes, and then performs the separate display parse/tessellation pass over the same immutable source bytes. It emits one chunk named `exact-brep-root-0` in canonical millimetres, encodes finite little-endian positions and in-range triangle indices, binds the manifest to the exact source and analysis hashes, and writes the existing deterministic `PPDSCENE` artifact. The second parse is deliberate display-only work; it does not replace the already accepted shape measurements. Cancellation remains cooperative at the exposed probes, with the supervisor's forced cleanup as the bound for uninterruptible native meshing work.

Native display failure is non-authoritative: the accepted analysis remains available and the worker returns `succeeded_with_warnings` plus `DISPLAY_SCENE_UNAVAILABLE`. A combined output-quota shortage also drops the derivative instead of disguising analysis failure. A display write failure removes both fixed worker-visible outputs and fails the execution rather than returning a partial claim.

## Executable evidence

```sh
cargo test -p partprobe-geometry-core --locked
cargo clippy -p partprobe-geometry-core --all-targets --locked -- -D warnings
cargo test -p partprobe-geometry-import --test display_scene --locked
cargo clippy -p partprobe-geometry-import --all-targets --locked -- -D warnings
```

Thirteen focused geometry-core contract tests pass, including three display-scene tests. They prove a valid source/analysis-bound manifest round trip, preservation of the stable `body-0` test reference, rejection of a path-like reference, absence of raw geometry/path fields, fixed layout metadata, exact derived lengths/totals, and rejection of false authority, unit mismatch, profile/schema/unknown-field mutation, zero extents, length/sequence/total tampering, and every aggregate ceiling.

Seven focused native decoder/artifact tests use a deterministic analytic 10 mm cube payload. They prove exact artifact and renderer-array round trips with the `body-0` reference, and reject stale source/analysis identity, cancellation, chunk count/sequence mismatches, both vertex/index length and hash mismatches, non-finite positions, out-of-range indices, wrong controlled reference, invalid magic, malformed manifest, truncation, trailing bytes, and oversized declared spans. This in-memory decoder fixture does not replace the governed persisted STEP fixture or establish worker tessellation accuracy.

Twenty protocol tests cover v1 compatibility, path-free schema-v2 requests, exact separately typed response references, unknown-version/reference rejection, explicit-unavailability semantics, and existing control/transport behavior. Five internal supervisor tests include dual output claiming, combined-quota rejection with cleanup, and unreported-display removal. Twenty-five default worker process tests include a real schema-v2 STL request that preserves and validates the authoritative mesh analysis while returning explicit display unavailability and no display file. Ten draft-estimate application tests prove the display request survives authorized source fingerprinting and only an exactly source/analysis-bound scene can be retained.

Fresh Apple-Silicon native evidence uses exact OCCT `V8_0_0` commit `d3056ef80c9668f395da40f5fd7be186cae4501f`. Four adapter cases prove governed cube and independently authored prism tessellation, deterministic output, hard vertex-ceiling rejection, and cancellation. The cube and prism each produce 24 flattened vertices and 12 triangles; their decoded maxima reproduce 10 × 10 × 10 mm and 12 × 8 × 5 mm source bounds. Eighteen native worker process tests include source/analysis-bound display emission for both fixtures through the real supervisor. A separate real `DraftEstimateApplication` test proves that the prism derivative survives authorization, worker execution, supervisor claim, controlled decoding, and ephemeral session retention while the private workspace returns empty.

Six ordinary renderer tests now include analytic fully validated indexed source scenes, proving projected-extents fitting across all four views and multiple aspect ratios, index preservation, and the absence of invented stock. The `viewer-spike` desktop host passes 48 ordinary tests with five opt-in native/package smokes ignored. A fresh verified runtime explicitly passes the new configured-worker desktop test: the governed prism travels through desktop schema-v2 request construction and retained native state as 24 vertices, 12 triangles, and 36 indices. Contract and host regressions preserve all eleven commands and prohibit geometry payloads, popup windows, or stale-selection acceptance.

Live Retina Apple-Silicon inspection exercises the corrected external-runtime developer `.app` through the real host-owned picker and native OCCT path. Before analysis, the selected 10 mm cube produces an empty native viewport with exact **Waiting for analysis** and **No model displayed** evidence, proving that a previous source scene is not reused. Native analysis returns the expected 10 × 10 × 10 mm bounds, 600 mm² surface area, 1,000 mm³ volume, and `(5, 5, 5)` mm centroid. The same-window Metal viewport then shows the complete source-bound cube with visible margins and no stock. The inspection exposed and corrected longest-world-axis fitting, which clipped a rotated cube; the renderer now fits actual projected screen and depth extents, backed by the multi-view/aspect regression.

## Evidence not established

The native evidence is one fresh Apple-Silicon developer build, one coarse profile, two public synthetic STEP fixtures, one flattened root chunk, CPU-side renderer conversion, configured desktop retention, and one live source-bound Metal cube. It does not yet establish progressive loading, stock placement, representative-model performance, complete accessibility, failed-display/device-loss recovery, Windows/Linux ABI-v1 display behavior, package integration, signed distribution, or supported importer status. Ordinary desktop/package builds still keep the viewer unavailable and request schema v1.

## Next gate

Add the keyboard/textual equivalent and device-loss/failure fallback evidence required by VIS-1. Govern stock AABB origin, orientation, and per-side allowance interpretation before displaying any real stock envelope, and add representative-model performance plus Linux/Windows/package evidence before promotion.
