# Linux Desktop Package Validation

> **Status:** In Review
> **Last updated:** 2026-08-12
> **Related requirements:** REQ-NF-001, REQ-NF-003, REQ-NF-006, REQ-NF-010; TEST-008, TEST-012, TEST-019
> **Related ADRs:** ADR-0001, ADR-0003
> **Open questions:** Supported Linux baseline, signing identity/policy, package format support, GPU/display matrix, and passing native-runtime package evidence
> **Dependencies:** GUI-3 through GUI-5 shell evidence; pinned native runtime construction and host verification
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

Initial run 31610449968 passes the pinned toolchain and prerequisite setup, builds the bundled-content desktop executable, and produces `PartProbe_0.1.0_amd64.deb`. AppImage construction then fails closed before inspection or launch because the existing square icon files were not explicitly listed in `bundle.icon`. Its log also shows the Tauri bundler downloading AppImage helper scripts/binaries from moving `master`/`continuous` locations, which does not meet this repository's pinned-input evidence standard. The corrected workflow therefore limits this checkpoint to Debian; AppImage remains deferred until helper sources and hashes are explicitly governed. The production config still pins the reviewed 32/128/256/512 PNG plus ICNS/ICO icon set, with a host-contract regression that keeps bundling disabled by default while requiring that exact metadata. Superseded run 31612272327 was cancelled after evidence review found that it launched the adjacent build-tree executable rather than the extracted package payload.

Run [31612819644](https://github.com/JoshuaGessner/PartProbe/actions/runs/31612819644) passes the complete bounded gate at commit `b6d6ecd` in 15m40s. It produces `PartProbe_0.1.0_amd64.deb` with package `part-probe`, version `0.1.0`, architecture `amd64`, installed size `188200` KiB, and declared dependencies `libwebkit2gtk-4.1-0, libgtk-3-0`. The Debian package SHA-256 is `943852dede40a046b7f077fb7c52cf4196d42d1d7a438a2dfdc6ea387a04a270`. The privately extracted `/usr/bin/partprobe-estimator-desktop` is an x86-64 dynamically linked ELF PIE with SHA-256 `6a4efd632218978aade5e8fd28d84a5d63107b0a06f72ff19b38f120e19bf62a`; that exact file remains alive until the expected ten-second timeout under Xvfb/DBus, and the checkout remains unchanged. The headless session logs unavailable accessibility/desktop-portal services and failed portal display activation, so this is process/window-shell survival evidence only—not native-dialog, assistive-technology, portal, physical-display, or usability acceptance.

## Native-runtime package follow-up

The separate opt-in `Linux Native Desktop Package Evidence` workflow keeps the exact OCCT construction/runtime verification from the native Linux checkpoint and the private Debian extraction/executable smoke from this package checkpoint, but joins them through one fixed deployment contract: absent an explicit developer override, the host accepts only `partprobe-native-runtime` under Tauri's resolved resource directory and re-verifies it before deriving worker/library paths. The job constructs pinned OCCT on Ubuntu 22.04 x86_64, assembles and link-audits a new runtime, injects a package-specific verified copy into the Debian resource tree through a package-only Tauri configuration, privately extracts the package, repeats runtime/link verification on the extracted copy, runs the real STEP-to-retained-USD-702 test against that exact resource tree, and launches the exact package executable without `PARTPROBE_NATIVE_RUNTIME`. It uploads no binaries and confirms the checkout stays unchanged.

Initial run [31620526974](https://github.com/JoshuaGessner/PartProbe/actions/runs/31620526974) reaches the new boundary in 47m21s. It passes exact source/tool setup, OCCT construction, all native adapter/worker checks, runtime assembly/re-verification, 24-binary/23-family link closure, package configuration, and unsigned Debian construction. The package is 118,176,760 bytes with SHA-256 `489820b2e8ea2a1eb2801b97b3979e5739e8065789e9746297130a8393d2c31c`; its extracted x86-64 executable has SHA-256 `04b29bd548602d184f12ead03e85d7e3b2b88facca5152d99fda5cc3ddce75a1`. Inspection confirms the runtime is at `/usr/lib/PartProbe/partprobe-native-runtime`, then verification rejects `lib/libTKBO.so`: Debian's archive path materializes local OCCT symlinks as regular duplicate files, while the original runtime manifest correctly still declares a symlink. Analysis and application launch are skipped.

The corrected workflow does not relax extracted-runtime verification. `materialize-for-package` accepts only a completely verified runtime plus a distinct new output, dereferences only the already-approved safe same-directory aliases, rewrites those entries as exact regular-file sizes and SHA-256 hashes, and re-verifies the complete package runtime before Tauri copies it. The original developer runtime remains unchanged. A focused regression covers alias materialization and manifest regeneration; the corrected hosted run remains pending.

This follow-up remains **pending evidence** until a clean hosted run passes every gate. Even a passing result will be unsigned, extracted rather than installed, synthetic-fixture-only, headless, and internal. It will not establish portal/dialog/accessibility, physical-display/GPU, install/upgrade/uninstall, legal redistribution, SBOM completeness, signing, support, or release acceptance.
