# Model-viewer architecture

## Metadata

- **Status:** In Review
- **Last updated:** 2026-07-22
- **Related requirement IDs:** REQ-F-022–REQ-F-024, REQ-F-049–REQ-F-055, REQ-NF-011, REQ-NF-013, UX-021–UX-027, UX-035–UX-037, SEC-004
- **Related architecture decision IDs:** ADR-0003, ADR-0005
- **Open questions:** Chosen desktop UI shell and embedding path? GPU fallback quality? Accessibility approach for 3D selection?
- **Dependencies:** Geometry worker tessellation API, selected UI framework, `wgpu` spike, visual fixtures
- **Supersedes / superseded by:** None / none

## Recommendation

Render worker-produced, versioned tessellations in a Rust `wgpu` component owned by the desktop process. `wgpu` is documented as a cross-platform, safe, pure-Rust graphics API with native Vulkan, Metal, D3D12 and OpenGL backends. [wgpu documentation](https://docs.rs/wgpu/latest/wgpu/) The viewer is a visualization and selection layer, never the source of geometry measurements.

## Responsibilities

| Component | Owns | Must not own |
|---|---|---|
| Geometry worker | Tessellation from exact/mesh source, per-primitive/body IDs, display tolerance, bounding data | UI input, quote persistence, GPU device |
| Model-viewer crate | GPU resources, camera, clipping, selection, highlight overlays, render modes, context-loss recovery | CAD parsing, exact measures, feature recognition |
| UI adapter | Panels, accessibility equivalents, commands, keyboard focus, selection synchronization | GPU or geometry lifecycle details |
| Application service | Snapshot/version selection and mapping of geometry/feature/setup IDs | Rendering API details |

## Interaction and data mapping

Viewer selections use stable snapshot-scoped `GeometryReference` values returned by the worker, not triangle indices leaked from a mutable buffer. The same reference maps to feature evidence, requirement coverage, revision mapping, setup orientation, tool-access warning, and cost/routing explanation. Supported modes: shaded, shaded-with-edges, wireframe when feasible, body/isolate, section/clipping, orientation views, warning/feature/setup/revision/cost-risk overlays, fit, undoable visibility/filter controls, and screenshot export subject to controlled-data policy.

Cost/risk overlays consume sanitized allocation DTOs from the application service. They include the estimate/route/allocation versions, amount or band, confidence, and an explicit unallocated remainder. The viewer never calculates or reallocates cost, and ambiguous or many-to-many revision mappings remain visually and textually distinct from exact mappings.

The viewer displays a persistent representation badge: `Exact B-rep source (tessellated for display)` or `Mesh source`. It shows confirmed units, analysis state and warning count. Measurements shown in overlays must link to analysis snapshot derivations; picking and apparent pixel distances are never authoritative.

## Robustness, performance, and accessibility

Use progressive loading: coarse mesh first, requested quality only after interaction settles, cancellation on snapshot change, GPU-resource budgets, and deterministic camera framing from analysis bounds. Handle absent/unsupported GPU by a visible software/limited mode or a clear import-review message; do not silently omit warnings. Test device-loss/context recreation, HiDPI, resize, long labels, huge assemblies, and cross-platform color/selection contrast.

Provide keyboard commands for focus, standard views, isolate/reset, next/previous selected feature and selection clearing. Mirror selected entity name/type/dimensions/warnings in a textual inspector, and do not encode confidence or warning state only by color. The primary inspector remains usable when a model cannot render.

## Security and scope

Viewer inputs are controlled in-memory derivative buffers; it loads no external textures, URLs, scripts, shaders, or CAD files. Screenshot/export defaults follow attachment classification and audit policy. No cloud rendering, analytics, or model telemetry is enabled by default. The component does not claim to validate geometry or provide CAM collision avoidance.
