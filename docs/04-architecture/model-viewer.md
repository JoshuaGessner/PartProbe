# Model-viewer architecture

## Metadata

- **Status:** In Review
- **Last updated:** 2026-09-13
- **Related requirement IDs:** REQ-F-022–REQ-F-024, REQ-F-049–REQ-F-055, REQ-NF-011, REQ-NF-013, UX-021–UX-027, UX-035–UX-037, SEC-004
- **Related architecture decision IDs:** ADR-0003, ADR-0005
- **Open questions:** Acceptance of Tauri's feature-gated child-WebView composition; GPU fallback quality; accessibility evidence for selection
- **Dependencies:** Geometry worker tessellation API, selected UI framework, native-surface integration, visual fixtures
- **Supersedes / superseded by:** None / none

## Recommendation

Render worker-produced, versioned tessellations in a Rust `wgpu` component owned by the desktop process. `wgpu` is documented as a cross-platform, safe, pure-Rust graphics API with native Vulkan, Metal, D3D12 and OpenGL backends. [wgpu documentation](https://docs.rs/wgpu/latest/wgpu/) The viewer is a visualization and selection layer, never the source of geometry measurements.

The VIS-1/VIS-2 checkpoints now provide `crates/model-viewer` with exact `wgpu 30.0.1`, platform-scoped Metal/D3D12/Vulkan+GLES features, a bounded `synthetic-viewer-spike-v1` offscreen renderer, and a safe owned native-surface renderer. The Apple-Silicon host proof renders the synthetic model/stock scene through Metal in the existing PartProbe window. The renderer can now also consume only an already-validated native `ValidatedGeometryDisplayScene`, build bounded indexed GPU buffers, compute display-only vertex shading, and fit the selected model deterministically from its decoded finite bounds. Source-bound mode deliberately emits no stock or stock edges until placement semantics are governed. A new selection clears the prior scene, and only a matching retained selection/analysis revision may replace it. CAD, source paths, estimate rules, and geometry buffers remain outside the WebView.

Contract v11 preserves the v10 estimating DTOs/commands and adds one exact path-free visible/hidden workspace request plus a path-free developer-preview availability value. No wire-schema migration is required for the source-bound checkpoint: the existing optional `scene_reference` now reports the fixed `geometry-display-scene-v1` reference when native replacement succeeds, or no reference plus a content-minimized unavailable notice when it does not. The adapter is compiled only by `viewer-spike` and enabled only with the compatibility flag `--vis1-synthetic-viewer`; that exact combination also activates worker schema v2 and native scene handoff. Its permission/capability/manifest parity does not authorize geometry payloads or normal production-package behavior. Tauri 2.11.5 exposes the child-WebView composition used inside the existing native window behind its documented `unstable` feature. ADR-0003 must explicitly accept that API or replace it with a stable equivalent before product integration.

## Responsibilities

| Component | Owns | Must not own |
|---|---|---|
| Geometry worker | Tessellation from exact/mesh source, per-primitive/body IDs, display tolerance, bounding data | UI input, quote persistence, GPU device |
| Model-viewer crate | Validated display-scene consumption, GPU resources, camera, clipping, selection, highlight overlays, render modes, context-loss recovery | CAD parsing, exact measures, feature recognition, stock inference |
| UI adapter | Panels, accessibility equivalents, commands, keyboard focus, selection synchronization | GPU or geometry lifecycle details |
| Application service | Snapshot/version selection and mapping of geometry/feature/setup IDs | Rendering API details |

The worker output is a native-only `geometry-display-scene-v1`, separate from exact `geometry-snapshot-v1` and mesh evidence. Its manifest pins source and accepted analysis-output hashes, representation, coordinate space/units, exact tessellation profile/tolerances, positive AABB extents, fixed position/index/topology layouts, bounded path-free snapshot-scoped geometry references, exact aggregate counts/bytes, and contiguous chunk hashes/lengths. It rejects unknown fields and enforces developer comparison ceilings of 32 chunks, 1,000,000 vertices, 2,000,000 triangles, and 32 MiB. `geometry-import` wraps and decodes the controlled artifact into deliberately nonserializable native arrays. The developer desktop requests that exact profile, retrieves the validated scene only from the retained application session, and hands it directly to `partprobe-model-viewer`; this internal dependency adds no third-party package and prevents the renderer from accepting unvalidated arrays. Replacement failure clears the renderer, marks display unavailable, and does not change the accepted analysis or estimate state. WebView DTOs contain only lifecycle state, the fixed scene reference, and textual notices. See [VIS-2 validation](../06-quality/vis-2-display-scene-validation.md).

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
