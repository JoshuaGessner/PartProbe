# Linux Desktop Package Validation

> **Status:** In Review
> **Last updated:** 2026-08-12
> **Related requirements:** REQ-NF-001, REQ-NF-003, REQ-NF-006, REQ-NF-010; TEST-008, TEST-012, TEST-019
> **Related ADRs:** ADR-0001, ADR-0003
> **Open questions:** Supported Linux baseline, signing identity/policy, package format support, GPU/display matrix, and native-runtime embedding
> **Dependencies:** GUI-3 through GUI-5 shell evidence; external native runtime remains separate
> **Supersedes:** None

## Evidence boundary

The opt-in `Linux Desktop Package Evidence` workflow builds the current Tauri/Leptos desktop shell on explicit Ubuntu 22.04 x86_64 using Rust 1.94.1, Trunk 0.21.14, wasm-bindgen-cli 0.2.127, and Tauri CLI 2.11.4. It installs the reviewed Tauri Debian/Ubuntu prerequisites, builds one unsigned Debian package, inspects it locally, extracts it into an owner-only ephemeral runner directory, records the package and contained executable SHA-256 values in the job log, and requires that exact Debian-contained executable to remain alive for a bounded ten-second virtual-display smoke.

The workflow grants read-only repository permission, disables persisted checkout credentials, uploads no package/cache/artifact, uses no CAD fixture or native OCCT runtime, and confirms the checkout remains unchanged. The virtual-display smoke starts with native analysis unavailable because the separately verified developer runtime and external workspace are intentionally absent. That fail-closed configuration is not a STEP-analysis test.

## Acceptance gate

The checkpoint passes only when all of the following occur in one clean job:

- the pinned frontend and desktop build tools install successfully;
- bundled local frontend construction and the `desktop-host` build complete;
- one Debian package exists in the expected Tauri bundle directory;
- `dpkg-deb` and `file` can inspect the package metadata, contents, and type;
- the expected executable exists and is executable after extraction into the private runner directory;
- package and contained-executable SHA-256 values are produced without uploading either artifact;
- that extracted Debian-contained executable does not crash or exit during the bounded virtual-display window; and
- the source checkout remains unchanged.

## Limitations

This is an internal packaging spike. It executes the extracted package payload without installing or registering the package. It does not inspect every transitive file/license, run the native file dialog interactively, analyze a model, embed the OCCT runtime, exercise a physical GPU/display/session, prove older/newer distribution compatibility, sign anything, publish an artifact, or establish install/upgrade/uninstall, accessibility, security, legal, or release acceptance. Ubuntu 22.04 is an evidence host, not yet a supported-product promise.

Initial run 31610449968 passes the pinned toolchain and prerequisite setup, builds the bundled-content desktop executable, and produces `PartProbe_0.1.0_amd64.deb`. AppImage construction then fails closed before inspection or launch because the existing square icon files were not explicitly listed in `bundle.icon`. Its log also shows the Tauri bundler downloading AppImage helper scripts/binaries from moving `master`/`continuous` locations, which does not meet this repository's pinned-input evidence standard. The corrected workflow therefore limits this checkpoint to Debian; AppImage remains deferred until helper sources and hashes are explicitly governed. The production config still pins the reviewed 32/128/256/512 PNG plus ICNS/ICO icon set, with a host-contract regression that keeps bundling disabled by default while requiring that exact metadata. Run 31612272327 tests the narrowed build, inspection, and build-tree shell launch at commit `0ef897f`, but it predates the final requirement that the smoke execute the extracted Debian payload. One final corrected run remains required. Do not cite this document as passing Linux package evidence until a run ID, commit, elapsed time, package and executable metadata/hashes, and bounded extracted-payload launch result are recorded here.
