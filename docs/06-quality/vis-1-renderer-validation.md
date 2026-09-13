# VIS-1 native renderer validation

> **Status:** In Progress
> **Last updated:** 2026-09-12
> **Related requirements:** REQ-F-022–REQ-F-024, REQ-NF-011, REQ-NF-013; UX-021–UX-027
> **Related ADRs:** ADR-0003, ADR-0005
> **Open questions:** Acceptance or replacement of Tauri's feature-gated child-WebView API; input/accessibility lifecycle; device-loss fallback; Windows/Linux package behavior
> **Dependencies:** `crates/model-viewer`, `docs/04-architecture/model-viewer.md`, `docs/07-delivery/dependency-record.md`
> **Supersedes:** None

## Checkpoint boundary

VIS-1 phase A adds a native-only `partprobe-model-viewer` crate. Its `synthetic-viewer-spike-v1` scene is deliberately fixed, public, and non-authoritative. It accepts no CAD bytes, source path, worker result, analysis result, rate, estimate, application state, or WebView message. This keeps the renderer experiment reversible while proving the intended visual language: an opaque model inside a translucent, high-contrast stock envelope with a separate edge layer.

The renderer pins `wgpu 30.0.1`, disables broad defaults, and selects Metal on macOS, D3D12 on Windows, or Vulkan plus GLES on Linux. It requests no optional GPU feature. Frames are bounded to 64–2,048 pixels on each edge, use depth testing and alpha blending, support Isometric/Front/Top/Right projections, and return RGBA pixels. The example writes binary PPM using the standard library; it introduces no image decoder, external asset lookup, or network behavior.

VIS-1 phase B adds `NativeSurfaceRenderer` and a developer-only Tauri adapter. The adapter exists only when the native host is built with `viewer-spike` and the process receives the exact `--vis1-synthetic-viewer` flag. Contract v11 adds one exact permissioned path-free visible/hidden workspace command and a developer-preview availability state. The adapter reuses the existing PartProbe native window, replaces the feature-enabled host's top-level WebView with one child WebView, narrows that WebView to a 400-logical-pixel inspector when **Model & stock** is selected, and exposes the native surface on the right. Returning to Estimate or Settings restores the WebView to the complete window. It creates no second OS window, reconfigures bounded physical dimensions on resize/scale changes, redraws after focus, and drops the renderer with the parent window. No source path, CAD bytes, vertex/index buffer, worker geometry, estimate authority, or production-package behavior crosses the bridge. Tauri 2.11.5 exposes child-WebView composition behind its documented `unstable` Cargo feature, so this remains developer evidence pending explicit ADR acceptance or a stable replacement.

## Evidence

| Check | Result on 2026-09-11 | What it proves |
|---|---|---|
| Ordinary crate tests | 4 passed; native GPU smoke ignored by default | Primitive counts, finite/bounded clip coordinates in four views, distinct offscreen/surface limits, viewport containment, and rejection before GPU initialization |
| Strict crate Clippy | Passed with `-D warnings` | New library, tests, and example satisfy the current lint policy |
| Explicit host GPU smoke | Passed on Apple Silicon through Metal | Four standard views produce distinct non-background RGBA frames; a 640×360 resize also renders with exact output dimensions |
| Reproducible example | 960×640 PPM generated and converted locally to PNG for inspection | Opaque cyan model, translucent amber offset stock faces, clear stock edges, and dark high-contrast background are visible |
| Developer Tauri host tests | 45 passed; four configured native/package smokes remain ignored | The feature-gated in-window adapter compiles with exact contract-v11 command/permission/capability parity, rejects popup-builder/geometry-payload regressions, and accepts only the exact launch flag |
| Live same-window navigation | Passed on Retina Apple Silicon through Metal | The 1180×760 PartProbe window exposes one accessible **Model & stock** destination, displays its WebView inspector and native scene side by side, and returns to the full Estimate workspace without opening another window |
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
cargo run -p partprobe-estimator-desktop --features viewer-spike -- --vis1-synthetic-viewer
```

The reproducible local image is `target/vis-1-model-stock.png`; `target/` remains untracked build evidence rather than a shipped or governed geometry fixture.

## Evidence not established

This checkpoint proves one developer-gated same-window native surface on Retina Apple Silicon, path-free contract-v11 workspace control, accessible navigation labels, return navigation, and one live resize. It does not prove stable-API acceptance, complete keyboard controls, screen-reader equivalence, broad HiDPI/continuous-resize behavior, device-loss recovery, software fallback, selection/picking, display-scene decoding, CAD tessellation, actual model or stock placement, normal production packaging, or Windows/Linux execution. The shown stock placement belongs only to the named synthetic scene and must never be applied to a user model. The ordinary desktop package keeps the developer preview unavailable.

## Next gate

Complete VIS-1 by explicitly accepting the feature-gated Tauri child-WebView API in ADR-0003 or replacing it with a stable equivalent; add keyboard/view controls and a complete textual accessibility-equivalent inspector; implement device-loss recreation plus a visible non-GPU/limited fallback; and run broader scaling/resize plus Windows/Linux package evidence. Current timeout/occlusion/outdated handling is bounded, while a lost or validation-failed surface becomes visibly unavailable. VIS-2 is now the critical path: its [manifest, artifact decoder, protocol migration, supervisor claim, and application-retention boundary](vis-2-display-scene-validation.md) are validated, but native tessellation/emission, desktop activation, and renderer handoff remain. Real stock cannot be shown until a versioned placement/orientation/per-side policy exists.
