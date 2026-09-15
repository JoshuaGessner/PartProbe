# VIS-1 native renderer validation

> **Status:** In Progress
> **Last updated:** 2026-09-14
> **Related requirements:** REQ-F-022–REQ-F-024, REQ-NF-011, REQ-NF-013; UX-021–UX-027
> **Related ADRs:** ADR-0003, ADR-0005
> **Open questions:** Acceptance or replacement of Tauri's feature-gated child-WebView API; complete screen-reader/accessibility lifecycle; forced device-loss/platform fallback evidence; Windows/Linux package behavior
> **Dependencies:** `crates/model-viewer`, `docs/04-architecture/model-viewer.md`, `docs/07-delivery/dependency-record.md`
> **Supersedes:** None

## Checkpoint boundary

VIS-1 phase A adds a native-only `partprobe-model-viewer` crate. Its `synthetic-viewer-spike-v1` scene is deliberately fixed, public, and non-authoritative. It accepts no CAD bytes, source path, worker result, analysis result, rate, estimate, application state, or WebView message. This keeps the renderer experiment reversible while proving the intended visual language: an opaque model inside a translucent, high-contrast stock envelope with a separate edge layer.

The renderer pins `wgpu 30.0.1`, disables broad defaults, and selects Metal on macOS, D3D12 on Windows, or Vulkan plus GLES on Linux. It requests no optional GPU feature. The live surface requests a compatible high-performance adapter first and then one platform fallback adapter. It registers a device-loss callback, makes exactly one surface reconfigure/reacquire attempt after loss or outdating, skips timeout/occlusion, and fails closed on validation or repeated loss. Frames are bounded to 64–2,048 pixels on each edge, use depth testing and alpha blending, support Isometric/Front/Top/Right projections, and return RGBA pixels. The example writes binary PPM using the standard library; it introduces no image decoder, external asset lookup, or network behavior.

VIS-1 phase B adds `NativeSurfaceRenderer` and an internal-test Tauri adapter. The adapter exists only when the native host is built with the deliberate `viewer-spike` profile; it no longer depends on a process flag. Contract v12 preserves v11's exact permissioned path-free visible/hidden workspace command and adds one exact standard-view command. The adapter reuses the existing PartProbe native window, replaces the feature-enabled host's top-level WebView with one child WebView, narrows that WebView to a 400-logical-pixel inspector when **Model & stock** is selected, and exposes the native surface on the right. Returning to Estimate or Settings restores the WebView to the complete window. If graphics initialization fails, or a later device/surface failure survives the one bounded renderer recovery, the adapter drops all GPU-backed geometry, keeps the application running, expands the child WebView to the full window, and returns a path-free text-only review result. View controls then disable while the polite atomic status region and model/proposal description lists remain usable. It creates no second OS window, reconfigures bounded physical dimensions on resize/scale changes, redraws after focus, mutates the native camera only for Isometric/Front/Top/Right requests, and drops the renderer with the parent window. No source path, CAD bytes, vertex/index buffer, worker geometry, estimate authority, or production-package behavior crosses the bridge. Tauri 2.11.5 exposes child-WebView composition behind its documented `unstable` Cargo feature, so this remains internal evidence pending explicit ADR acceptance or a stable replacement.

The current VIS-2 integration extends that exact internal configuration. It requests the governed worker display profile, reads only the matching fully validated scene retained by the application session, and replaces the native model buffers directly. Indexed source geometry is fitted from finite decoded bounds; view-space normals feed ambient/key/fill/rim fragment lighting instead of a single baked face color. A new selection clears the preceding model; stale analysis cannot repopulate it; and failure clears source buffers while preserving accepted analysis. When the matching application proposal becomes available, `proposed-stock-display-placement-v1` adds its exact rectangular dimensions as translucent amber faces plus explicit edges, source-axis aligned and centered on renderer-owned source bounds. The placement therefore splits each total-axis allowance equally across opposing faces and remains visibly review-only with standard size/availability unresolved. A new source or saved Settings change clears it. Contract v12 and the WebView remain geometry-free.

## Evidence

| Check | Current result | What it proves |
|---|---|---|
| Ordinary crate tests | 12 passed; native GPU smoke ignored by default | Primitive counts, finite/bounded clip coordinates in four views, projected-extents source fitting across four views and multiple aspect ratios, index preservation, source normals retained for GPU lighting, no stock before a proposal, centered/enclosing proposed stock, invalid-stock rejection, distinct offscreen/surface limits, viewport containment, one bounded lost/outdated recovery attempt, transient skip classification, fatal GPU lifecycle classification, and rejection before GPU initialization |
| Strict crate Clippy | Passed with `-D warnings` | New library, tests, and example satisfy the current lint policy |
| Explicit host GPU smoke | Passed on Apple Silicon through Metal after the lighting change | Four standard views produce distinct non-background RGBA frames; a 640×360 resize also renders with exact output dimensions |
| Reproducible example | 960×640 PPM generated and converted locally to PNG for inspection | Lit cyan faces have materially distinct tones; translucent amber stock, bright edges, and the dark background remain clearly separated |
| Developer Tauri host tests | 56 passed; five configured native/package smokes remain ignored | The feature-gated adapter preserves exact contract-v12 parity, maps all four wire views to native camera views, requests the exact display profile, binds source and proposed stock to the retained selection/analysis, clears stale stock, keeps startup/runtime limited-mode notices path-free and selection-bound, and proves build-profile activation plus private host-workspace lifecycle |
| No-GPU/device-loss fallback logic | Passed deterministic renderer and host tests; live forced loss still open | A preferred-adapter miss requests one platform fallback adapter; device loss is observed; lost/outdated surfaces get one bounded retry; unrecovered GPU state discards renderer geometry and retains a full-width text-only model/proposal review without altering accepted analysis or estimate state |
| Configured desktop scene retention | Passed on Apple Silicon with the fresh verified runtime | The 12 × 8 × 5 mm prism reaches desktop-owned native state as 24 vertices, 12 triangles, and 36 indices through the real schema-v2 worker path |
| Live source-bound scene and proposed stock | Passed on Retina Apple Silicon through Metal in the fresh package | Exact native analysis of the 10 mm STEP cube reports 10 × 10 × 10 mm, 600 mm², and 1,000 mm³; selection first clears the prior scene into an accurately labeled pending state, then the validated indexed scene renders completely inside the same-window viewport. Automatic proposal preparation adds the exact 16.4 × 16.4 × 16.4 mm blank as translucent amber stock with explicit edges under `proposed-stock-display-placement-v1`; distinct lit cyan face tones make the part definition visible. |
| Live same-window navigation | Passed on Retina Apple Silicon through Metal | The 1180×760 PartProbe window exposes one accessible **Model & stock** destination, displays its WebView inspector and native scene side by side, and returns to the full Estimate workspace without opening another window |
| Live keyboard/view and text equivalent | Passed on Retina Apple Silicon through Metal | Front changes the native view by mouse; Tab followed by Return changes it to Top; Isometric restores the source-bound cube; the accessible inspector exposes representation, units, 10 × 10 × 10 mm dimensions, warnings, confidence, analysis reference, and explicit unavailable stock facts |
| Workspace focus semantics | Passed in the refreshed unsigned macOS package through the platform accessibility bridge; screen-reader comprehension remains pending | Pointer and keyboard transitions focus the exact level-two heading for Estimate, Settings, and Model & stock after rendering. Keyboard activation shows the deliberate focus ring, and the next Tab from the Estimate heading reaches Choose model. A blocked/failed native viewer transition remains designed not to move focus |
| Live resize | Passed from 1180×760 to approximately 1000×680 logical pixels | The same native window, inspector, surface viewport, camera aspect, and depth target reflow cleanly; the model and stock remain visible |
| Strict feature Clippy | Passed with `-D warnings` | The live-surface renderer and Tauri adapter satisfy current lint policy with `viewer-spike` enabled |

Commands:

```sh
cargo test -p partprobe-model-viewer --locked
cargo clippy -p partprobe-model-viewer --all-targets --locked -- -D warnings
cargo test -p partprobe-model-viewer native_renderer_covers_standard_views_and_resizing --locked -- --ignored --nocapture
cargo run -p partprobe-model-viewer --example render_vis1 --locked -- target/vis-1-model-stock.ppm
cargo test -p partprobe-estimator-desktop --features viewer-spike --lib --locked
cargo clippy -p partprobe-estimator-desktop --features viewer-spike --all-targets --locked -- -D warnings
cargo run -p partprobe-estimator-desktop --features viewer-spike
```

The reproducible local image is `target/vis-1-model-stock.png`; `target/` remains untracked build evidence rather than a shipped or governed geometry fixture.

## Evidence not established

This checkpoint proves one internal-build-gated same-window native surface on Retina Apple Silicon, path-free contract-v12 workspace/standard-view control, semantic keyboard-operable standard views, a path-free text equivalent, page-current navigation semantics, a live macOS accessibility-bridge destination-heading focus handoff, accessible navigation labels, return navigation, live resize, validated indexed source conversion, fragment-lit normals, proposal-stock buffer construction, configured desktop retention, deterministic surface/device failure classification, one bounded surface recovery attempt, and a text-only host fallback that preserves accepted work. It does not yet prove a VoiceOver/Narrator/Orca announcement and comprehension sequence, representative-model performance, stable-API acceptance, complete screen-reader equivalence, broad HiDPI/continuous-resize behavior, forced physical/driver device-loss recovery, successful software-adapter presentation on each target, in-session full-device recreation, selection/picking, editable/standard-size stock, signed production packaging, or Windows/Linux execution. The synthetic scene remains isolated GPU evidence only.

## Next gate

Complete VIS-1 by explicitly accepting the feature-gated Tauri child-WebView API in ADR-0003 or replacing it with a stable equivalent; exercise the implemented limited-mode path through forced device-loss/no-adapter platform tests and decide whether full in-session device recreation is required; capture the implemented focus path in VoiceOver/Narrator/Orca and finish remaining control accessibility evidence; and run broader scaling/resize, representative-model, and Windows/Linux package evidence. VIS-4 still owns editable candidates and standard-size/availability resolution; the current centered display is review-only.
