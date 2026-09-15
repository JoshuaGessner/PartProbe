# Desktop Platform Architecture

## Metadata

- **Status:** Draft
- **Last updated:** 2026-09-14
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

GUI-3 instantiated the outer two layers; GUI-4 connects the native host to `DraftEstimateApplication` and a separately configured `GeometryWorkerSupervisor` without claiming product support. USE-2 connects the host to `ShopSettingsApplication` and SQLite. `apps/estimator-desktop` owns presentation; `src-tauri` owns paths, database, native/application adapters, worker/cancellation, retained sessions, and the native viewer camera; `crates/desktop-contract` owns framework-neutral DTOs. Contract v8 added catalog activation, v9 added catalog-draft save, v10 added the proposal command, v11 added the viewer workspace command, and v12 adds one standard-view command for a total of twelve. Contract v12 preserves path-free exact STEP AABB, starter-Draft proposal, explicit adoption, result-exclusion, and viewer-lifecycle DTOs while adding only the enumerated current view. The WebView cannot calculate or alter proposal formulas or renderer state directly; the host reloads the exact settings revision and application recomputes from retained geometry before adoption. Legacy manual estimate and Settings DTOs remain accepted.

Mesh evidence is reviewable but not estimate-authoritative. STL measurements remain labeled as unresolved source coordinates; 3MF may carry canonical-millimetre measurements, but neither mesh variant may enter the deterministic calculation path. `DraftEstimateSession::evaluate` returns `Unavailable` for mesh before review, manual inputs, rates, or pricing can authorize an estimate, and the frontend withholds the estimate form as a secondary UX control. ABI-v4 exact STEP carries the unchanged `geometry-snapshot-v1` inside `geometry-step-analysis-v1`, so existing calculation rules remain unchanged. Contract v10 exposes only its three validated canonical-millimetre AABB extents and application-owned proposal/adoption trace; no other CAD content or geometry authority crosses the bridge. The selected filename extension is only a host-owned intake hint; a mismatch with content-derived exact/STL/3MF analysis fails visibly. No path, bytes, parser authority, component graph, or metadata content crosses the bridge.

GUI-4 request preparation begins inside `DraftEstimateApplication`, not the desktop host. A path-free request template carries job/correlation/capability/stage/profile/quota intent without claiming a source digest. The application first authorizes and audits the selected relative source, fingerprints the same bounded already-open grant, builds the source-bound request, rewinds the grant, and passes it to the geometry port. The worker still independently checks identity, type, length, quota, and digest. Desktop integration may invoke this use case but may not pre-read the path or substitute its own hash.

The desktop adapter is intentionally fail closed. An explicit developer `PARTPROBE_NATIVE_RUNTIME` may identify a separately assembled runtime root; otherwise a packaged host checks exactly the `partprobe-native-runtime` child of Tauri's resource directory. It verifies the manifest and complete artifact closure before deriving launch paths. `PARTPROBE_GEOMETRY_WORKSPACE` remains an explicit developer override; when absent from an internal package, the host creates one private `0700` per-process supervisor root directly beneath the OS temporary directory and retains its owner until shutdown. It removes only a proven-empty root and never recursively deletes unexpected content. Async Tauri commands move analysis, proposal, estimate, and database work off the UI task. Analysis retains the authorized source/session only if its token remains current. Proposal preparation validates that same selection/analysis, loads the host-owned immutable settings revision, returns `Unavailable`/`Blocked` rather than defaults, and may update only the native review-only stock display for the matching selection/analysis. Adoption repeats the application proposal calculation against the exact revision/library pins and requires review/limitations/quantity/reason before setting session inputs. Catalog actor/profile/time/correlation authority remains host-owned; ordinary startup supplies no catalog actor, catalog activation stays deny-all, and no environment allow switch exists. The contract-v10 proposal path uses only the already established developer-session actor for an ephemeral estimate review; it does not authenticate a shop role or mutate catalog/settings authority.

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

The first slice needs a responsive model viewer but must not bind the domain to a renderer. A `model-viewer` crate exposes scene, camera, selection, and render-result contracts. Contract v12's developer adapter exposes a native `wgpu` surface beside a bounded child WebView inside the existing PartProbe window; it sends no geometry into the WebView and creates no popup. The native host accepts only enumerated standard-view requests and returns the actual current camera, while the inspector uses reviewed path-free application facts as a text equivalent. Rendering, feature overlays, and picking remain visualizations of immutable analysis snapshots. VIS-2 now supplies the exact selected STEP's governed display derivative; the synthetic scene remains isolated pre-analysis GPU evidence.

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

GUI-3–GUI-5 provide the shell, typed native analysis/estimate, and configured worker evidence. Contract-v8/v9 tests cover exact catalog command permissions and authority minimization. Contract v12 preserves v10's path-free AABB/proposal/adoption DTOs and v11's viewer workspace command, then adds one exact path-free standard-view command, requiring twelve-command manifest/permission/capability/handler parity. Application tests prove model-sensitive cube/prism proposal and adoption behavior; VIS-1/2 tests and live Apple-Silicon inspection prove a source-bound same-window viewport, semantic keyboard standard views, and a reviewed path-free text equivalent. The viewer now requests one fallback adapter, observes device loss, retries a lost/outdated surface once, and retains the full text review after unrecovered graphics failure without changing contract v12. Packaged UI adoption and broader platform evidence remain immediate live-test gates. Settings catalog-editor/accessibility, forced fallback/device-loss evidence, complete screen-reader evidence, signed packages, PDF, estimate persistence, and update evidence remain open.
