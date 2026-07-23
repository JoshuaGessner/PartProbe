# Desktop Platform Architecture

## Metadata

- **Status:** Draft
- **Last updated:** 2026-07-22
- **Related requirement IDs:** REQ-NF-001, REQ-NF-004, REQ-NF-005, REQ-NF-007, SEC-001 through SEC-006, UX-001 through UX-010
- **Related architecture decision IDs:** ADR-0001, ADR-0003, ADR-0005, ADR-0006
- **Open questions:** Final OS support matrix; GPU integration ownership; whether Linux packages include both AppImage and native packages; team-mode authentication boundary.
- **Dependencies:** [UI evaluation](../01-research/rust-ui-evaluation.md), [ADR-0001](adr/ADR-0001-ui-framework.md), [security model](security-model.md), [model-viewer architecture](model-viewer.md)
- **Supersedes / superseded by:** None

## Architecture decision in context

The proposed desktop application, pending ADR-0001 evidence and approval, is a **Tauri 2 host** with a **Leptos client-side-rendered UI bundle**, Rust application services, and an independently testable Rust domain/estimation layer. This architecture does not allow the UI framework to become a business-logic dependency. The domain, geometry, feature-recognition, runtime-estimation, and persistence abstractions remain native Rust crates and are testable without a window.

```text
Leptos DOM/CSS UI  <— typed commands/events —>  Tauri host / application facade
       |                                              |
  viewport adapter                             domain + estimation services
       |                                              |
GPU model viewer / overlay                   repositories / SQLite / local files
```

The UI is an untrusted presentation and interaction layer even though it ships locally. It displays view models, validates immediate input for feedback, emits typed user intents, and renders returned results. Only the application facade authorizes operations; only the domain/calculation layer produces authoritative money, units, confidence, or audit values.

## Runtime responsibilities

| Layer | Responsibilities | Must not do |
|---|---|---|
| UI (`ui-components`, desktop frontend) | Layout, accessibility semantics, local focus/selection, optimistic visual state, request rendering | Direct SQL, raw filesystem paths, authoritative calculations, parsing CAD |
| Tauri host/platform | Window lifecycle, menu, drag/drop handoff, capability configuration, typed bridge, secure update policy | Business rule duplication or broad filesystem/network access |
| Application facade | Authorization, command validation, transactions, undo/audit intents, async job orchestration | Render or own UI state |
| Domain/engines | Units, calculations, immutable analysis and quote snapshots | Depend on Tauri, Leptos, DOM, or a database |
| Viewer adapter | Rendering buffers, camera, picking, geometry selection mapping | Interpret a pick as a confirmed manufacturing feature |

## Platform UX boundaries

- Use a standard OS window frame initially. Tauri supports custom titlebars but documents macOS behavior lost with fully custom chrome; any custom treatment requires a tested transparent-titlebar approach and platform-specific safeguards ([Tauri window customization](https://v2.tauri.app/learn/window-customization/)).
- Use native menus and native file dialogs for global application actions and import/export. The document UI renders application actions in the main workspace; it does not duplicate every menu action as chrome.
- Treat drag/drop paths as sensitive. The host converts them into validated intake requests; the UI never reads arbitrary paths.
- Use the browser engine only for the application UI and local bundled assets. No CDN assets, remote fonts, remote telemetry, or remote document preview by default.

## Security model at the desktop boundary

Tauri capabilities declare which windows/webviews may use which core/plugin commands; custom application commands must also be explicitly restricted because registered commands are otherwise available to all windows/webviews ([Tauri capabilities and permissions](https://v2.tauri.app/security/capabilities/)). Configure distinct least-privilege capabilities for the main UI, any preview window, and development tooling. Maintain a restrictive CSP; allow only local application assets unless a future approved integration needs more.

1. The import capability may invoke a native dialog and submit selected paths to the intake service; it does not grant recursive read access.
2. No shell/process plugin is enabled in the production baseline.
3. Network is deny-by-default. Update checks are opt-in per deployment profile and must use signed metadata/artifacts; fully offline mode performs none. Tauri offers an updater plugin but use does not itself establish an organizational update policy ([Tauri updater](https://v2.tauri.app/plugin/updater/)).
4. Logs record IDs, hashes, timings, and error codes—not CAD content, customer pricing, or document text.
5. External URL opening requires an explicit user action and a scoped opener policy.

## 3D viewing and document preview

The first slice needs a responsive model viewer but must not bind the domain to a renderer. A `model-viewer` crate exposes scene, camera, selection, and render-result contracts. The selected framework needs an adapter that either embeds a `wgpu` canvas/surface in the webview or hosts a native sibling surface/window. Rendering, feature overlays, and picking are visualizations of immutable analysis snapshots.

PDF/drawing preview uses an isolated local viewer route or a native engine with no unrestricted script execution. Print output is generated as a controlled quote-report artifact by the reporting layer, then previewed/printed; browser `window.print()` is not the sole acceptance path. This avoids a UI-dependent customer document definition.

## Packaging and operational profiles

| Profile | Storage / network | Update stance | Package expectation |
|---|---|---|---|
| Standalone | Local SQLite and attachment store; offline | Manual signed installer permitted; automatic checks off | Windows installer, macOS notarized bundle/DMG, Linux AppImage plus selected native package |
| Team/LAN | Local UI connects only to authenticated LAN service | Administrator-controlled channel | Same desktop package; deployment configuration externalized |
| Controlled-data | Restricted local or approved internal storage; no public endpoints | Offline/manual, documented provenance | Signed, checksum-verifiable artifacts; no telemetry defaults |

Tauri documents Windows installers, macOS bundles/DMGs, Linux packages, and code-signing routes ([Tauri distribution](https://v2.tauri.app/distribute/)). CI must build each target on that OS, sign only in controlled release jobs, and retain SBOM/dependency evidence separately.

## Performance and resilience goals

- Startup to usable command center: target under 3 seconds on reference hardware; establish baseline in spike rather than presenting it as a guarantee.
- UI input may not wait on CAD import, geometry analysis, report generation, or database migration. Those tasks publish structured status/events and allow cancellation where safe.
- A renderer or parser failure preserves unsaved UI recovery data, emits a non-sensitive diagnostic ID, and never changes an approved quote.
- Large tables use virtualization; model tessellation and thumbnail generation use bounded background work; UI mutations coalesce to a frame.

## Verification plan

Run unit/application tests without the desktop host. Add host contract tests for every command/capability, automated WebDriver smoke tests for core UI flows (Tauri documents the WebDriver support route at [desktop tests](https://v2.tauri.app/develop/tests/webdriver/)), and manual platform accessibility checks. The framework spike exit criteria in the UI evaluation are the gate before committing to production UI construction.
