# Rust Desktop UI Evaluation

## Metadata

- **Status:** Draft
- **Last updated:** 2026-07-22
- **Related requirement IDs:** REQ-NF-001, REQ-NF-004, REQ-NF-007, UX-001 through UX-010
- **Related architecture decision IDs:** ADR-0001
- **Open questions:** Can the chosen webview meet the target keyboard and screen-reader acceptance suite on all three desktop platforms? Which GPU surface ownership approach is most stable with the model viewer?
- **Dependencies:** [Desktop platform](../04-architecture/desktop-platform.md), [design system](../05-ux/design-system.md), [ADR-0003 model rendering](../04-architecture/adr/ADR-0003-model-rendering.md)
- **Supersedes / superseded by:** None

## Decision summary

Recommend **Tauri 2 with a Leptos CSR frontend**, scoped native capabilities, and a separately owned GPU model-viewer surface. This produces the most capable foundation for a dense, bespoke estimating application: standards-based HTML controls and accessibility semantics for the business UI; CSS for the deliberately custom design system; Rust in the desktop host, domain, and frontend; and an intentional boundary for native/GPU geometry viewing.

This is not a claim that the stack is entirely Rust: HTML/CSS and a small amount of JavaScript interoperability are part of the desktop surface. That trade is preferable to compromising accessible high-density forms, tables, printing, and mature browser developer tooling. Leptos supports Rust-authored reactive DOM UIs compiled to WASM; its documentation explicitly covers browser APIs through `wasm-bindgen`/`web_sys`, styling, routing, and testing ([Leptos book](https://book.leptos.dev/)). Tauri documents direct Leptos integration ([Tauri Leptos guide](https://v2.tauri.app/start/frontend/leptos/)).

## Evaluation method

Scores are 1 (poor fit) to 5 (strong fit), based on the first vertical slice: editable routing tables, calculations, PDF customer preview, native imports, high-DPI model review, keyboard-driven operation, and controlled-data deployment. A score means *fit with planned engineering*, not a vendor guarantee. Accessibility and 3D rendering receive disproportionate decision weight.

| Criterion | Tauri 2 + Leptos | Dioxus Desktop | Slint | Iced | egui |
|---|---:|---:|---:|---:|---:|
| Bespoke visual styling / themes | 5 | 4 | 5 | 4 | 3 |
| Dense editable forms and tables | 5 | 4 | 3 | 3 | 2 |
| Keyboard navigation / shortcuts | 5 | 4 | 4 | 4 | 3 |
| Accessibility path | 5 | 4 | 4 | 3 | 2 |
| Cross-platform rendering consistency | 4 | 4 | 5 | 5 | 5 |
| HiDPI / windowing | 4 | 4 | 5 | 5 | 5 |
| Native menus, dialogs, drag/drop | 5 | 4 | 4 | 4 | 3 |
| Printing and PDF preview | 5 | 4 | 2 | 2 | 2 |
| GPU 3D-viewer integration | 4 | 4 | 5 | 5 | 5 |
| Startup / memory potential | 4 | 4 | 5 | 4 | 4 |
| Packaging / signing / updates | 5 | 3 | 4 | 3 | 3 |
| Automated UI testing | 5 | 3 | 3 | 3 | 2 |
| Maintainability / hiring surface | 5 | 3 | 3 | 3 | 3 |
| Rust purity | 4 | 5 | 5 | 5 | 5 |
| Licensing / security controls | 5 | 3 | 4 | 4 | 4 |

### Candidate findings

**Tauri 2 + Leptos — recommended pending spike evidence.** Tauri supplies window controls, capability/permission configuration, native dialog plugins, a WebDriver testing route, platform installers, and signing documentation ([Tauri documentation](https://v2.tauri.app/)). Its capabilities system can constrain core/plugin exposure, but custom application commands require deliberate app-manifest restriction; Tauri documents that registered commands otherwise remain available to all windows/webviews ([Tauri capabilities](https://v2.tauri.app/security/capabilities/)). Custom title bars are supported, though Tauri warns that on macOS a fully custom titlebar loses system window-management features ([Tauri window customization](https://v2.tauri.app/learn/window-customization/)). Therefore retain native decorations by default and only experiment with a transparent integration treatment.

Leptos produces DOM controls, so semantics, browser focus behavior, native input editing, ARIA, and CSS media queries are available. The cost is a webview compatibility matrix and a WASM/client boundary: do not compile domain, CAD, filesystem, or calculation authority into the frontend. Do not use SSR in the desktop app; a local CSR bundle is simpler and avoids a local server process.

**Dioxus Desktop — viable alternate, not selected.** It provides Rust/RSX development with webview desktop rendering, giving similar DOM strengths. The current desktop renderer uses a webview and an internal native-to-webview transport ([dioxus-desktop docs](https://docs.rs/dioxus-desktop/latest/dioxus_desktop/)). Its product-specific desktop, printing, capability, signing, and test ecosystem is less mature or more fragmented than Tauri’s, increasing delivery risk for a controlled-data desktop product.

**Slint — strong visual/control candidate, not selected.** Slint’s native renderer, declarative language, direct Rust binding, built-in accessibility roles, and automated-accessibility identifiers are excellent ([Slint accessibility reference](https://docs.slint.dev/latest/docs/slint/reference/common/)). It supports declarative keyboard bindings ([Slint keyboard guide](https://docs.slint.dev/latest/docs/slint/reference/keyboard-input/overview/)) and its own consistent rendering is appealing for a premium application. The downside is the cost of building spreadsheet-grade grids, print/PDF workflows, rich document preview, and custom web-adjacent integrations. Keep it as a future thin native viewer/HMI option; do not split the main product UI across frameworks.

**Iced — credible Rust-native base, not selected.** Iced is a cross-platform, Elm-inspired GUI library built atop `wgpu`/`winit` ([Iced repository](https://github.com/iced-rs/iced)). It offers a compelling GPU story and consistent rendering but has less mature out-of-the-box data-grid, browser-accessibility, document-preview, and packaging/story than the selected stack. It would transfer too much routine business-UI infrastructure into this project.

**egui — not credible as the primary product shell.** egui is immediate-mode and intentionally simple, with first-class `wgpu`/`winit` integrations ([egui repository](https://github.com/emilk/egui)). It is excellent for an internal engineering/debug overlay, a geometry spike, or diagnostics. Its dense business forms, native text editing behavior, accessibility expectations, printing, and polished data-grid experience are not an acceptable primary route for long estimator sessions.

## Required validation spike

Timebox this to **10 engineering days** before framework lock:

1. Build an unsigned Tauri + Leptos shell on Windows, macOS, and Ubuntu with native menu, file-open, drag/drop, and a restricted capability file.
2. Implement a 500-row virtualized routing table with inline decimal editing, validation, copy/paste, column resizing, undo, and keyboard traversal. Measure input-to-painted-frame p95 at 60 Hz on the reference hardware.
3. Embed or coordinate a `wgpu` model-viewer prototype with picking; prove selection synchronization to a DOM feature list without UI thread stalls.
4. Test a 20-page PDF preview/print workflow, a generated customer quote, and save/export restrictions.
5. Run keyboard-only and screen-reader smoke tests: Windows (Narrator + Accessibility Insights), macOS (VoiceOver + Accessibility Inspector), and Linux (Orca/AT-SPI where supported). Tauri’s WebDriver path supplies the automation starting point ([Tauri WebDriver guide](https://v2.tauri.app/develop/tests/webdriver/)).
6. Produce installers and verify code-signing/update policy, including offline/no-update mode. Tauri documents platform distribution and signing targets ([Tauri distribution guide](https://v2.tauri.app/distribute/)).

**Exit criteria:** all workflows function without mouse-only steps; no uncontrolled filesystem/network capability; 3D selection stays under 100 ms p95 for a representative model; and at least one automated end-to-end test per target OS passes. If the viewer cannot coexist reliably, retain Tauri for the shell and use a separate native viewer process/window behind the model-viewer interface.

## Risks and mitigations

| Risk | Mitigation / trigger |
|---|---|
| System webviews differ | Test exact supported OS versions in CI; avoid browser-specific CSS; publish support matrix. |
| Wasm/DOM bridge becomes a calculation authority | UI issues typed commands only; Rust application service validates and calculates. |
| Custom chrome harms native behavior or accessibility | Default to native decorations; require usability evidence before enabling custom chrome. |
| Web content expands filesystem/network reach | Strict CSP, capability allowlist, no remote scripts, scoped dialog/filesystem permissions. |
| GPU viewer destabilizes webview | Isolate viewer state/process boundary; keep UI responsive and define fallback screenshot/diagnostic mode. |

## Recommendation status

Proceed with the spike and keep ADR-0001 **In Review** until its exit criteria are recorded. This recommendation is not approval to add production UI dependencies before the spike’s dependency/security review.
