# ADR-0001: Use Tauri 2 with Leptos for the Desktop UI

## Metadata

- **Status:** In Review
- **Last updated:** 2026-07-22
- **Related requirement IDs:** REQ-NF-001, REQ-NF-004, REQ-NF-005, REQ-NF-007, SEC-001, UX-001 through UX-010
- **Related architecture decision IDs:** ADR-0001
- **Open questions:** Does the selected webview/viewer adapter satisfy accessibility and performance spike criteria across all supported OS versions?
- **Dependencies:** [Rust UI evaluation](../../01-research/rust-ui-evaluation.md), [desktop platform](../desktop-platform.md), [ADR-0003 model rendering](ADR-0003-model-rendering.md)
- **Supersedes / superseded by:** None

## Context

PartProbe is a Rust-first, local-first desktop estimator with dense editable forms, high-volume tables, native file import, PDF/customer quote preview, and a 3D CAD model viewer. It must run on Windows, macOS, and Linux, remain accessible and keyboard-efficient, support strict local data boundaries, and look bespoke rather than like generic native widgets. The framework must not contaminate domain or calculation crates.

## Decision

Propose **Tauri 2** as the desktop host and **Leptos CSR** as the Rust-authored DOM UI for the initial production vertical slice, pending this ADR's validation gates. Use standards-based HTML, CSS, and accessibility semantics for the business application surface. Keep all authoritative computation and sensitive filesystem access in native Rust application services behind narrow typed commands. Do not enable an unrestricted shell, network, filesystem, or remote-content capability.

Tauri was selected because its capabilities system can narrowly constrain core/plugin exposure by window, webview, and scope, but this is not automatic least privilege: Tauri documents that registered application commands are available to all windows/webviews unless the app manifest restricts them ([Tauri capabilities](https://v2.tauri.app/security/capabilities/)). It also supports platform distribution/signing ([Tauri distribution](https://v2.tauri.app/distribute/)), and its custom-titlebar support leaves an explicit path for a refined app frame ([Tauri window customization](https://v2.tauri.app/learn/window-customization/)). Leptos was selected for reactive Rust/WASM authoring while retaining DOM/CSS controls and browser accessibility behavior ([Leptos documentation](https://book.leptos.dev/)).

## Alternatives considered

| Alternative | Decision | Reason |
|---|---|---|
| Dioxus Desktop | Not selected | Similar Rust/webview benefit but a smaller purpose-built desktop capability, packaging, and testing story for this product. |
| Slint | Not selected | Strong native rendering/accessibility primitives, but more product infrastructure would be custom for complex grids, PDF/print, and rich document workflows. |
| Iced | Not selected | Attractive `wgpu`/`winit` foundation, but more immature high-density business UI and accessibility surface. |
| egui | Rejected for primary UI | Best for tooling/prototypes, not a polished, assistive-technology-friendly estimating workstation. |

Full comparative evidence and scores are maintained in the [UI evaluation](../../01-research/rust-ui-evaluation.md).

## Consequences

Positive: bespoke CSS-based design system; accessible DOM controls; fast iteration on forms/tables; strong desktop security and distribution story; Rust domain/application code; possible renderer isolation.

Negative: the frontend includes web technologies and depends on system webviews; browser-engine differences must be tested; a WASM/native bridge needs deliberate contracts; full custom titlebars can harm platform conventions. This decision does not select a geometry kernel, model renderer, table library, PDF engine, or update service.

## Guardrails

1. The desktop UI is CSR only; do not introduce a local SSR server for the application shell.
2. Domain, geometry, calculation, and persistence crates must not depend on Tauri, Leptos, DOM, or webview types.
3. Use semantic HTML before custom ARIA; test every custom composite control for keyboard and screen-reader behavior.
4. Adopt native decorations until cross-platform usability evidence approves a custom/transparent frame.
5. Viewer integration must remain behind the `model-viewer` boundary and be replaceable.
6. No production dependency is accepted without licensing, maintenance, security, and spike evidence.

## Validation and reversal

Run the bounded spike in the [UI evaluation](../../01-research/rust-ui-evaluation.md). Accept this ADR only after three-OS build/install, keyboard/screen-reader smoke tests, restricted capability tests, rendering-selection latency, and print/PDF preview tests pass. If the DOM/webview approach fails its accessibility or viewer integration exit criteria, reassess Slint or Iced with the same test suite; do not switch based on a demo alone.
