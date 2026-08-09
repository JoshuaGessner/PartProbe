# GUI-3 Secure Desktop Shell Validation

> **Status:** Complete
> **Last updated:** 2026-08-09
> **Related requirements:** REQ-NF-001, REQ-NF-002, REQ-NF-004, REQ-NF-006, REQ-NF-010, REQ-NF-017; SEC-001, SEC-002; UX-001, UX-007–UX-010; TEST-008, TEST-011, TEST-012
> **Related ADRs:** ADR-0001
> **Open questions:** Three-OS host/package evidence, full accessibility matrix, GUI-4 application-service integration
> **Dependencies:** `crates/desktop-contract`, `apps/estimator-desktop`, `apps/estimator-desktop/src-tauri`
> **Supersedes:** None

## Scope and result

GUI-3 is complete for its bounded developer-checkpoint acceptance criteria. It adds a locally runnable Tauri 2.11.5 host and Leptos 0.8.20 CSR shell, one versioned shared bridge contract, host-owned native STEP selection, an exact main-window capability, a production local-assets-only CSP, deliberate design-system styling, and explicit provisional/session-only/unavailable states.

This evidence does not accept ADR-0001, complete TASK-005, establish a supported desktop application, or prove geometry analysis. The shell does not yet call `DraftEstimateApplication`, launch the geometry worker, edit estimate inputs, persist data, render a model, or produce a price.

## Boundary evidence

- The webview can invoke only `desktop_contract` and `select_model_source`. `tauri_build::AppManifest::commands` generates the two application permissions; `capabilities/main.json` grants only those permissions plus event listen/unlisten to the `main` window.
- No shell, HTTP, filesystem, opener, updater, upload, or frontend dialog permission is granted. The dialog plugin is called from Rust; its transitive filesystem path type does not grant the webview filesystem access.
- Production content loads from the bundled `tauri://localhost` origin. CSP allows local styles/scripts/fonts/images, local WASM fetch, and Tauri IPC; it denies objects, frames, form submission, remote sources, and inline script/style. Prototype freezing is enabled.
- The native dialog callback is asynchronous. The first smoke attempt exposed that `blocking_pick_file` froze the host; a regression test now prohibits that API.
- The native host retains the selected `PathBuf` behind a session token. The serialized result contains only `selection_id`, leaf `display_name`, STEP format, `not_started`, and `session_only`; a test proves parent directory text cannot cross the bridge.
- A cancelled replacement selection preserves the prior selection. Errors carry a bounded code, user-safe message, and non-sensitive diagnostic ID.
- The estimate panel remains visibly `Unavailable`; no missing geometry, unit review, rate basis, or selling price is converted to zero.

## Automated evidence

Validated locally on Apple Silicon macOS with Rust 1.94.1:

```sh
cargo test -p partprobe-desktop-contract -p partprobe-estimator-desktop-ui -p partprobe-estimator-desktop --locked
cargo clippy -p partprobe-estimator-desktop-ui --target wasm32-unknown-unknown --locked -- -D warnings
cargo clippy -p partprobe-estimator-desktop --features desktop-host --all-targets --locked -- -D warnings
trunk build --release --locked true --offline true
cargo tauri build --debug --features desktop-host --bundles app --no-sign
```

The focused native test set contains 13 tests: two shared-contract tests, three presentation-state tests, and eight host/configuration/path tests. The WASM frontend, desktop-host feature, offline release frontend bundle, and unsigned debug `.app` bundle all compile successfully. Repository-wide formatting, strict workspace Clippy, 119 macOS runtime tests, one compile-fail doctest, six native-tooling tests, 140-document planning validation, fixture hashes, and diff checks pass locally. Three-OS GUI-3 CI remains pending this change's push and is not part of the bounded local GUI-3 exit criterion.

## Manual macOS smoke evidence

On 2026-08-09, the unsigned debug `PartProbe.app` launched from the local build output and exposed a semantic accessibility tree with a level-one workspace heading, level-two model/estimate headings, skip link, named button, named application state, and description lists for source and estimate facts.

The smoke path:

1. launched the local bundle with no server and rendered the bundled CSR UI;
2. opened the macOS native file picker without freezing the window;
3. selected the synthetic `fixtures/models/cube_10mm.step` file;
4. returned only `cube_10mm.step`, `STEP`, `Not started`, and `Session only` to the visible UI;
5. displayed visible keyboard focus on `Choose STEP model`;
6. opened the native picker with Return and cancelled with Escape.

This is a one-platform developer smoke, not the TEST-012 accessibility suite. Narrator/Accessibility Insights, VoiceOver/Accessibility Inspector, Orca/AT-SPI, contrast automation, browser/webview version coverage, installers, signing, and all three supported OS packages remain open.

## GUI-4 handoff

GUI-4 must consume the retained session token through a native adapter that reconstructs the authorized `DraftEstimateApplication` request. It must preserve explicit unit/warning review, missing/blocked rate states, pinned policy/rate context, and `DraftEstimateResult`. It may not pass the raw path into Leptos, parse CAD in the webview, duplicate formulas, or imply that selecting a source completed analysis.
