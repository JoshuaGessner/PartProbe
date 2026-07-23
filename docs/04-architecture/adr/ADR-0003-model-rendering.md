# ADR-0003: Model rendering

## Metadata

- **Status:** In Review
- **Last updated:** 2026-07-22
- **Related requirement IDs:** REQ-F-022–REQ-F-024, REQ-NF-011, REQ-NF-013, UX-021–UX-027, SEC-004
- **Related architecture decision IDs:** ADR-0002, ADR-0005
- **Open questions:** UI-shell embedding, minimum GPU/fallback policy, cross-platform rendering test baseline
- **Dependencies:** UI framework decision, worker tessellation contract, signed GPU dependency packages
- **Supersedes / superseded by:** None / none

## Proposed decision

Use a dedicated Rust `wgpu` model-viewer crate, supplied by geometry-worker tessellations and embedded behind the selected desktop UI adapter. `wgpu` documents native backends for Vulkan, Metal, D3D12 and OpenGL and a safe Rust API. [wgpu documentation](https://docs.rs/wgpu/latest/wgpu/)

## Rationale and consequences

This retains cross-platform GPU control, allows custom high-density engineering UI, and avoids a browser-only/CAD-viewer dependency. It adds GPU-driver/device-loss testing, surface-embedding work and an explicit fallback policy. The viewer will show geometry/feature/setup mapping, but exact computations stay in the geometry engine; rendering tessellation is non-authoritative.

## Approval evidence required

Demonstrate a STEP-derived tessellation and STL/3MF mesh on Windows, macOS and Linux; verify HiDPI, device loss, selection-to-feature mapping, text-equivalent inspector, memory limits, and no external network or asset resolution. Confirm the selected UI host can embed the component without unstable private APIs. Pin and record the evaluated `wgpu` version/backends; review direct/transitive/native-backend licenses, maintenance/advisory ownership, build provenance, and package contents.

## Alternatives

Web view/WebGL is deferred because it introduces a web-runtime bridge and broader content-security surface. Kernel-native visualization is rejected because it couples UI lifetime and interactions to unsafe/native geometry APIs. Third-party commercial viewers remain a future procurement option only.
