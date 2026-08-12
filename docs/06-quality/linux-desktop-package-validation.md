# Linux Desktop Package Validation

> **Status:** In Review
> **Last updated:** 2026-08-12
> **Related requirements:** REQ-NF-001, REQ-NF-003, REQ-NF-006, REQ-NF-010; TEST-008, TEST-012, TEST-019
> **Related ADRs:** ADR-0001, ADR-0003
> **Open questions:** Supported Linux baseline, signing identity/policy, package format support, GPU/display matrix, and native-runtime embedding
> **Dependencies:** GUI-3 through GUI-5 shell evidence; external native runtime remains separate
> **Supersedes:** None

## Evidence boundary

The opt-in `Linux Desktop Package Evidence` workflow builds the current Tauri/Leptos desktop shell on explicit Ubuntu 22.04 x86_64 using Rust 1.94.1, Trunk 0.21.14, wasm-bindgen-cli 0.2.127, and Tauri CLI 2.11.4. It installs the reviewed Tauri Debian/Ubuntu prerequisites, builds unsigned Debian and AppImage packages, inspects both locally, records their SHA-256 values in the ephemeral job log, and requires the bundled-content desktop executable to remain alive for a bounded ten-second virtual-display smoke.

The workflow grants read-only repository permission, disables persisted checkout credentials, uploads no package/cache/artifact, uses no CAD fixture or native OCCT runtime, and confirms the checkout remains unchanged. The virtual-display smoke starts with native analysis unavailable because the separately verified developer runtime and external workspace are intentionally absent. That fail-closed configuration is not a STEP-analysis test.

## Acceptance gate

The checkpoint passes only when all of the following occur in one clean job:

- the pinned frontend and desktop build tools install successfully;
- bundled local frontend construction and the `desktop-host` build complete;
- one Debian package and one AppImage exist in the expected Tauri bundle directories;
- `dpkg-deb` can inspect package metadata and contents, and `file` recognizes the AppImage;
- SHA-256 values are produced without uploading either artifact;
- the desktop executable does not crash or exit during the bounded virtual-display window; and
- the source checkout remains unchanged.

## Limitations

This is an internal packaging spike. It does not execute the packaged file after installation, inspect every transitive file/license, run the native file dialog interactively, analyze a model, embed the OCCT runtime, exercise a physical GPU/display/session, prove older/newer distribution compatibility, sign anything, publish an artifact, or establish install/upgrade/uninstall, accessibility, security, legal, or release acceptance. Ubuntu 22.04 is an evidence host, not yet a supported-product promise. AppImage compatibility remains tied to the build baseline and Tauri/WebKitGTK constraints.

The first workflow run is pending. Do not cite this document as passing Linux package evidence until a run ID, commit, elapsed time, package metadata/hashes, and bounded launch result are recorded here.
