# Model-viewer architecture

## Metadata

- **Status:** In Review
- **Last updated:** 2026-09-12
- **Related requirement IDs:** REQ-F-022–REQ-F-024, REQ-F-049–REQ-F-055, REQ-NF-011, REQ-NF-013, UX-021–UX-027, UX-035–UX-037, SEC-004
- **Related architecture decision IDs:** ADR-0003, ADR-0005
- **Open questions:** Acceptance of Tauri's feature-gated child-WebView composition; GPU fallback quality; accessibility evidence for selection
- **Dependencies:** Geometry worker tessellation API, selected UI framework, native-surface integration, visual fixtures
- **Supersedes / superseded by:** None / none

## Recommendation

Render worker-produced, versioned tessellations in a Rust `wgpu` component owned by the desktop process. `wgpu` is documented as a cross-platform, safe, pure-Rust graphics API with native Vulkan, Metal, D3D12 and OpenGL backends. [wgpu documentation](https://docs.rs/wgpu/latest/wgpu/) The viewer is a visualization and selection layer, never the source of geometry measurements.

The first VIS-1 checkpoints now provide `crates/model-viewer` with exact `wgpu 30.0.1`, platform-scoped Metal/D3D12/Vulkan+GLES features, a bounded `synthetic-viewer-spike-v1` offscreen renderer, and a safe owned native-surface renderer. The Apple-Silicon host proof renders an opaque model, translucent offset stock box, explicit stock edges, all four planned standard views, and multiple frame sizes through Metal. A developer-gated Tauri adapter presents the same scene on a Retina display inside the existing PartProbe window: a child WebView becomes a 400-logical-pixel inspector while the native surface occupies the remaining viewport, follows live resize, and returns to the full Estimate workspace without creating another OS window. It accepts no CAD, path, estimate, worker geometry, or WebView geometry data and therefore establishes only the GPU/scene/same-window lifecycle slice—not complete keyboard/accessibility, device-loss recovery, a display derivative, governed stock placement, normal packaged behavior, or a source-bound application viewer.

Contract v11 preserves the v10 estimating DTOs/commands and adds one exact path-free visible/hidden workspace request plus a path-free developer-preview availability value. The phase-B adapter is compiled only by `viewer-spike` and enabled only with `--vis1-synthetic-viewer`; its exact permission/capability/manifest parity does not authorize geometry payloads or normal production-package behavior. Tauri 2.11.5 exposes the child-WebView composition used inside the existing native window behind its documented `unstable` feature. This is the intended visual layout, but ADR-0003 must explicitly accept that API or replace it with a stable equivalent before product integration. A WebGL/WebGPU fallback would contradict the present no-geometry-in-WebView boundary and requires a separate ADR/security decision.

## Responsibilities

| Component | Owns | Must not own |
|---|---|---|
| Geometry worker | Tessellation from exact/mesh source, per-primitive/body IDs, display tolerance, bounding data | UI input, quote persistence, GPU device |
| Model-viewer crate | GPU resources, camera, clipping, selection, highlight overlays, render modes, context-loss recovery | CAD parsing, exact measures, feature recognition |
| UI adapter | Panels, accessibility equivalents, commands, keyboard focus, selection synchronization | GPU or geometry lifecycle details |
| Application service | Snapshot/version selection and mapping of geometry/feature/setup IDs | Rendering API details |

The worker output is a new native-only `geometry-display-scene-v1`, separate from exact `geometry-snapshot-v1` and mesh evidence. Its manifest pins source and accepted analysis-output hashes, representation, coordinate space/units, exact tessellation profile/tolerances, positive AABB extents, fixed position/index/topology layouts, bounded path-free snapshot-scoped geometry references, exact aggregate counts/bytes, and contiguous chunk hashes/lengths. It rejects unknown fields and enforces developer comparison ceilings of 32 chunks, 1,000,000 vertices, 2,000,000 triangles, and 32 MiB. Exact-B-rep scenes require canonical millimetres; mesh scenes may instead retain explicitly unresolved source coordinates. `geometry-import` wraps those values in one deterministic, versioned, bounded binary artifact and decodes a hash-verified `ControlledWorkerOutput`; it rechecks exact current source/analysis bindings, framing, count/sequence/length/hash evidence, finite positions, in-chunk indices, references, and cancellation, then produces deliberately nonserializable native arrays. Worker request/response schema v2 preserves the authoritative analysis reference while adding a distinct optional display request/reference. The supervisor owns and claims both fixed output names under one aggregate quota, removes unreported/failed outputs, and the application retains only a fully validated native scene. Additive OCCT display ABI v1 now tessellates the same immutable authorized STEP bytes after measurement and the worker emits a bound flattened-root artifact; fresh Apple-Silicon cube/prism evidence reaches the application session. The desktop still uses schema v1 and renders its synthetic scene. WebView DTOs contain only opaque viewer/candidate IDs, textual facts, lifecycle state, warnings, and typed stock-edit results. See [VIS-2 validation](../06-quality/vis-2-display-scene-validation.md).

## Interaction and data mapping

Viewer selections use stable snapshot-scoped `GeometryReference` values returned by the worker, not triangle indices leaked from a mutable buffer. The same reference maps to feature evidence, requirement coverage, revision mapping, setup orientation, tool-access warning, and cost/routing explanation. Supported modes: shaded, shaded-with-edges, wireframe when feasible, body/isolate, section/clipping, orientation views, warning/feature/setup/revision/cost-risk overlays, fit, undoable visibility/filter controls, and screenshot export subject to controlled-data policy.

Cost/risk overlays consume sanitized allocation DTOs from the application service. They include the estimate/route/allocation versions, amount or band, confidence, and an explicit unallocated remainder. The viewer never calculates or reallocates cost, and ambiguous or many-to-many revision mappings remain visually and textually distinct from exact mappings.

The viewer displays a persistent representation badge: `Exact B-rep source (tessellated for display)` or `Mesh source`. It shows confirmed units, analysis state and warning count. Measurements shown in overlays must link to analysis snapshot derivations; picking and apparent pixel distances are never authoritative.

The first model/stock workspace keeps Estimate primary and opens from `View model & stock`. It shows Model and Proposed stock in a keyboard-selectable scene tree; Fit/Isometric/Front/Top/Right, edges, stock visibility, and reset controls; an opaque model plus translucent high-contrast stock envelope; and a textual inspector with dimensions, total allowances, material/mass/removal, standard-size/availability state, machine fit, and proposal pins. Enclosure failures require text as well as color. The viewer never implies workholding, fixture position, tool access, or CAM feasibility.

Initial editing is numeric rather than drag-only. Each change creates an immutable session stock-candidate revision with original/new values, source analysis and settings/library pins, recomputed blank/removal/mass/material/runtime evidence, actor/time/reason, review state, and unresolved facts. A new versioned placement rule is required before drawing the stock envelope because the current proposal has no AABB minimum/maximum, stock transform, or per-side placement. Re-analysis or Settings changes make the candidate stale; estimate adoption references the exact reviewed candidate revision.

## Robustness, performance, and accessibility

Use progressive loading: coarse mesh first, requested quality only after interaction settles, cancellation on snapshot change, GPU-resource budgets, and deterministic camera framing from analysis bounds. Handle absent/unsupported GPU by a visible software/limited mode or a clear import-review message; do not silently omit warnings. Test device-loss/context recreation, HiDPI, resize, long labels, huge assemblies, and cross-platform color/selection contrast.

Provide keyboard commands for focus, standard views, isolate/reset, next/previous selected feature and selection clearing. Mirror selected entity name/type/dimensions/warnings in a textual inspector, and do not encode confidence or warning state only by color. The primary inspector remains usable when a model cannot render.

## Security and scope

Viewer inputs are controlled in-memory derivative buffers; it loads no external textures, URLs, scripts, shaders, or CAD files. Screenshot/export defaults follow attachment classification and audit policy. No cloud rendering, analytics, or model telemetry is enabled by default. The component does not claim to validate geometry or provide CAM collision avoidance.
