# ADR-0003: Model rendering

## Metadata

- **Status:** In Review
- **Last updated:** 2026-09-14
- **Related requirement IDs:** REQ-F-022–REQ-F-024, REQ-NF-011, REQ-NF-013, UX-021–UX-027, SEC-004
- **Related architecture decision IDs:** ADR-0002, ADR-0005
- **Open questions:** Stable UI-shell/native-window composition, forced device-loss/platform fallback evidence, cross-platform rendering test baseline
- **Dependencies:** UI framework decision, worker tessellation contract, signed GPU dependency packages
- **Supersedes / superseded by:** None / none

## Proposed decision

Use a dedicated Rust `wgpu` model-viewer crate, supplied by geometry-worker tessellations and embedded behind the selected desktop UI adapter. `wgpu` documents native backends for Vulkan, Metal, D3D12 and OpenGL and a safe Rust API. [wgpu documentation](https://docs.rs/wgpu/latest/wgpu/)

## Rationale and consequences

This retains cross-platform GPU control, allows custom high-density engineering UI, and avoids a browser-only/CAD-viewer dependency. It adds GPU-driver/device-loss testing, surface-embedding work and an explicit fallback policy. The viewer will show geometry/feature/setup mapping, but exact computations stay in the geometry engine; rendering tessellation is non-authoritative.

## Spike evidence to date

VIS-1 phase A pins `wgpu 30.0.1` with default features disabled and only the target backend plus `std`/WGSL enabled: Metal on macOS, D3D12 on Windows, and Vulkan/GLES on Linux. The isolated `partprobe-model-viewer` crate renders `synthetic-viewer-spike-v1` offscreen with depth, GPU-lit model faces, a translucent stock envelope, edge overlay, four deterministic standard views, bounded frame sizes, and RGBA readback. Phase B adds a safe owned surface and contract v11's explicitly path-free workspace control. With the deliberate `viewer-spike` internal-test build profile, the Tauri host hides its configured top-level WebView, composes one bounded child WebView and the Metal surface inside the same existing native window, exposes **Model & stock** as a normal in-app destination, redraws after live resize, and restores the full Estimate workspace on return. No runtime flag, second OS window, CAD data, or WebView geometry payload is required. Tauri 2.11.5 feature-gates this child-WebView API behind `unstable`, so the visual composition is proven but still requires explicit acceptance or a stable replacement. VIS-2 supplies the renderer with the matching validated worker-emitted selected-STEP scene through the native application session. Contract v12 adds one path-free standard-view request; the WebView exposes four semantic keyboard controls and reviewed text facts, while the native host owns camera mutation and returns the actual view. The native renderer now carries view-space normals to a fragment shader with ambient/key/fill/rim lighting, requests one fallback adapter if the preferred adapter is unavailable, observes device loss, and retries a lost/outdated surface once. Startup or unrecovered graphics failure discards GPU geometry and expands the same workspace into explicit text-only review without changing the accepted analysis or estimate. `proposed-stock-display-placement-v1` may add only the matching reviewed proposal's rectangular dimensions, centered on source bounds with the equal-per-side assumption visible and standard size/availability unresolved. This supports continuing the proposed decision but does not resolve it; Windows/Linux, complete screen-reader/accessibility, forced device-loss/platform evidence, signed packaging, representative-model evidence, and editable governed stock candidates remain open.

## Approval evidence required

Demonstrate a STEP-derived tessellation and STL/3MF mesh on Windows, macOS and Linux; verify HiDPI, device loss, selection-to-feature mapping, text-equivalent inspector, memory limits, and no external network or asset resolution. Confirm the selected UI host can compose the component through a stable supported API, or explicitly accept and govern any unstable public API. Pin and record the evaluated `wgpu` version/backends; review direct/transitive/native-backend licenses, maintenance/advisory ownership, build provenance, and package contents.

## Alternatives

Web view/WebGL is deferred because it introduces a web-runtime bridge and broader content-security surface. Kernel-native visualization is rejected because it couples UI lifetime and interactions to unsafe/native geometry APIs. Third-party commercial viewers remain a future procurement option only.
