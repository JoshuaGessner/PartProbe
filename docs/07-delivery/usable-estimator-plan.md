# Usable Estimator Delivery Plan

> **Status:** In Review
> **Last updated:** 2026-09-13
> **Related requirements:** REQ-F-002–REQ-F-010, REQ-F-014–REQ-F-018, REQ-F-032; UX-001–UX-012, UX-021–UX-025; GEO-001–GEO-015; TIME-001–TIME-008; DATA-001–DATA-017; TEST-002–TEST-007, TEST-012, TEST-014
> **Related ADRs:** ADR-0001, ADR-0002, ADR-0005–ADR-0008
> **Open questions:** Shop-approved stock allowances, material catalog/prices, machine groups, coarse runtime profiles, and acceptance tolerances
> **Dependencies:** TASK-003–TASK-007
> **Supersedes:** The sequencing, but not the evidence or safety boundaries, in `gui-vertical-slice-plan.md`

## Product shape

The next development sequence is organized around one useful primary task: choose a model and obtain a transparent estimate with the least amount of user entry that remains safe. The estimate workspace is not a rate-editor or a developer test form. Reusable shop configuration belongs in Settings, and the primary flow asks only for information that cannot be established from the authorized model or the selected governed library context.

```text
Choose model → Analyze locally → Supply vital missing facts → Review proposals → Estimate
                   ↓                         ↑
            geometry evidence       governed Settings/library context
```

The target first-use flow is:

1. choose or drop one model;
2. run local analysis and show concise geometry/warning evidence;
3. select material/condition and enter quantity plus requirement facts that cannot be derived safely;
4. generate versioned stock, route, and coarse-runtime proposals from the model and governed shop profiles;
5. resolve the applicable rate card, material price, and pricing policy automatically from Settings;
6. require one consolidated review for units, warnings, and material manufacturing assumptions; and
7. show an itemized cost and price result with model, library, algorithm, and policy traceability.

Generated stock, route, setup, and runtime values remain proposals until reviewed. A coarse deterministic runtime is not CAM simulation, and an estimate is not an approved quote.

## Settings boundary

Settings owns organization currency, immutable/effective rate cards, material and stock prices, stock allowances, machines/workcenters, coarse runtime profiles, operation templates, and pricing policies. The estimate workspace shows only configuration readiness, selected versions, conflicts/staleness, and a direct link to the affected Settings record. It does not duplicate reusable rate or pricing editors.

PartProbe still ships with no production numeric rates. The explicit `huntsville-2026q3-test-*` profile is a source-controlled, research-informed usability baseline: it loads only on request, clears every confirmation, remains unsaved until reviewed, and is never market, shop, approved, quote, or deployment authority. Its sources, assumptions, and replacement gate are documented in [Huntsville aerospace test rate baseline](../01-research/huntsville-test-rate-baseline.md). The ordinary arbitrary-model workflow must never silently apply a fixed template or describe fixed assumptions as model-derived.

## Delivery checkpoints

| Checkpoint | Outcome | Acceptance boundary |
|---|---|---|
| USE-1 Workflow guardrail | Upload-first Estimate view; rates/pricing moved to Settings; temporary manual manufacturing assumptions collapsed and explicitly labeled; stale displayed estimates clear after any input change | No formula, geometry, schema, or persistence change; exact STEP-only estimating remains session-only |
| USE-2 Durable shop setup | First-run Settings flow and SQLite-backed versioned rate/pricing/material/machine/runtime records | Empty-on-install numeric libraries; validation, effective dates, approvals, migrations, backup/replay, and missing/conflict states pass |
| USE-3 Model-derived stock and material | Exact STEP envelope evidence drives reviewable stock proposals; selected material supplies density and governed price; a native model/stock review window makes the relationship inspectable | Stock allowance, placement, editing, and selection policies versioned; model-sensitive fixtures and worked examples prove distinct geometry and reviewed stock revisions change the proposal and estimate |
| USE-4 Coarse process and runtime | Exact STEP evidence plus reviewed shop profiles proposes broad process, setup count, programming, cutting, handling, and inspection time | Low/needs-review confidence and reason codes visible; estimator adoption recorded; never described as CAM accuracy |
| USE-5 Streamlined estimate | Primary workflow reduces to model, material, quantity, vital requirement flags, consolidated review, and result for a configured shop | Same Settings with materially different governed models produces appropriately different physical/manufacturing inputs and traceable results; missing authority stays unavailable/blocked |
| USE-6 Calibration and deployable alpha | Actual estimator review, calibration, save/reopen, backup, accessibility, supported importer evidence, and signed target packages | TASK-003/004/006/007 and release-acceptance evidence pass on declared targets |

USE-2 and the initial USE-3 domain contracts may proceed in parallel, but persisted values cannot become calculation authority until migration/replay and governance behavior are proven. Mesh results remain non-authoritative for estimating until a separately documented calculation policy, confidence rule, fixtures, and migration decision exist.

The USE-3 engine checkpoint implements `partprobe-rectangular-stock-envelope-policy` v1.0.0 and the distinct internal `partprobe-developer-estimate-input-proposal` v1.0.0 in `crates/setup-planner`. ABI-v4 supplies one-solid exact STEP AABB evidence. The internal proposal resolves an exact schema-v2 starter Draft into model-sensitive blank, material mass/cost, machine-fit, and coarse-runtime evidence; `partprobe-developer-estimate-proposal-adoption` v1.0.0 requires explicit review, named limitation acceptance, quantity, native actor/time, and reason before the existing deterministic engine runs. Contract v11 preserves v10's path-free evidence/exclusion trace and adds only a path-free Model & stock workspace command. Analytic/native cube/prism tests prove model sensitivity. VIS-1 phases A/B prove the isolated native GPU scene, visual language, and a live developer-gated same-window Metal screen with return navigation and resize. VIS-2 carries governed native cube/prism display artifacts through the application session and developer desktop's validated indexed renderer path; live Apple-Silicon inspection now proves selection clearing, accurate pending state, exact cube analysis, and complete source-bound Metal framing. Child-WebView API acceptance/replacement and governed stock placement remain open. Final packaged/visual validation, governed catalog replacement, standard-size/availability, detailed runtime/adders, and ABI-v4/display-ABI Linux/Windows refresh remain next.

### Model and stock visualizer/editor increment

The source-bound application viewer is now an implemented developer slice of USE-3, not a supported product capability or decorative preview. It remains inside the existing architecture: the worker creates a separately versioned and bounded display derivative; the native `model-viewer` owns validated source-bound indexed GPU buffers; the application owns selection/analysis identity and validation; and the WebView receives only path-free lifecycle text through unchanged contract v11. Raw CAD, paths, and vertex/index buffers do not cross into the WebView. Real stock editing and display remain planned because the current proposal does not carry governed placement.

| Increment | Outcome | Exit evidence |
|---|---|---|
| VIS-1 renderer spike — In Progress | Phase A pins/reviews `wgpu 30.0.1` and proves a bounded offscreen native scene on Metal; phase B proves a feature/flag-gated same-window Tauri screen and path-free contract-v11 workspace control, while Tauri's child-WebView API remains feature-gated unstable | Complete locally: synthetic opaque model/translucent stock/edge overlay, four standard views, bounded offscreen/surface dimensions, viewport containment, live Retina Metal presentation inside the existing window, return navigation, live resize, and dependency record. Open: explicit API acceptance or stable replacement, device loss/fallback, complete keyboard/accessibility, broader HiDPI, Windows/Linux and normal package evidence |
| VIS-2 governed display derivative — In Progress | The native-only manifest/framing/decoder bind source and analysis identity, profile, bounds, layouts, references, chunks, hashes, and limits. Additive OCCT display ABI v1 tessellates the same immutable STEP bytes after measurement, worker schema v2 emits one source/analysis-bound artifact, the supervisor claims both fixed outputs under one quota, the application retains only validated arrays, and the developer desktop replaces the matching native scene | Complete: manifest/artifact rejection suites, protocol migration, shared-quota dual claim, native cube/prism tessellation/emission, hard limit/cancellation, real supervisor/application retention, indexed renderer conversion, stale-scene clearing, configured desktop retention, accurate pending state, and live source-bound cube framing on Apple Silicon. Open: representative performance and Linux/Windows/package evidence |
| VIS-3 model/stock review | `View model & stock` opens model tree, native viewport, standard views, edges/stock toggles, textual inspector, warnings, and provenance | Contract-v11 command/permission parity, GPU-unavailable fallback, accessible keyboard alternative, packaged Apple-Silicon smoke |
| VIS-4 governed stock revision | Numeric X/Y/Z blank or allowance edits create immutable session candidate revisions and recompute dependent proposal values | Original/new values, actor/time/reason, enclosure/machine-fit, stale-session and exact-adoption tests; no drag-only workflow |
| VIS-5 platform acceptance | Harden Metal/Vulkan/D3D12 packages and representative model behavior | Three-OS package, accessibility, scaling, memory, cancellation, device-loss, and visual-reference evidence |

Before drawing stock around a part, add a versioned placement rule. The current proposal contains source-axis extents and total dimensional additions but no AABB minimum/maximum, stock transform, orientation, or per-side placement. VIS-4 must never silently assume a centered blank. A first display-placement policy may split each total-axis allowance equally around the analyzed AABB only if it records that assumption as review-required display evidence, not workholding or routing authority. Re-analysis or a Settings change makes the viewer candidate stale, and an estimate must pin the exact reviewed candidate revision.

### USE-2 implementation status

The USE-2 foundation and first desktop slices are implemented behind typed application services and a repository port. Schemas v1-v4 persist immutable rate/pricing, starter-resource, multi-entry catalog/selection, and append-only authorization-decision evidence without numeric defaults. Separate catalog applications govern immutable Draft creation/editing and policy/audit-gated proposal eligibility. Contract v7 adds path-free catalog review, v8 adds native-owned activation, and v9 adds a complete bounded native-owned catalog-draft save command. The native adapter maps all five record kinds, preserves matching source evidence across equivalent decimal formatting, rejects changed content under reused versions, and executes storage off the UI task. Ordinary startup supplies no authenticated actor, returns explicit unavailability for both catalog commands, and retains deny-all activation; the WebView invokes neither. Persisted drafts or active-for-proposals records remain non-authoritative and are not consumed by estimate evaluation. Remaining USE-2 work includes the governed category/list/detail editor, record review, authenticated identity/roles, a reviewed shipped allow configuration, activation UI, Settings accessibility, broader recovery, and three-OS persistence evidence.

## Immediate implementation order

1. Preserve the completed USE-1 upload-first guardrail and deterministic application-service boundary.
2. Build the governed Settings editor on contract v9 using category navigation, a searchable record list, one selected-record inspector/editor, and a persistent save/review footer. Keep ordinary startup catalog actions identity-unavailable and activation deny-all until authenticated identity/role input and a shop-reviewed allow configuration exist. Numeric values remain explicitly test/shop owned, and active-for-proposals remains distinct from estimate authority.
3. Run the source-bound viewer through live same-window Metal inspection, then finish the live-test UI cleanup, fresh macOS package, and complete human Settings→STEP→proposal→estimate rehearsal without changing `geometry-snapshot-v1` or treating display tessellation as measurement authority.
4. Finish VIS-3 keyboard/textual and device-fallback behavior, then implement VIS-4's governed stock placement/revision path before calling USE-3 complete; do not replace analysis output, infer centered stock placement, or move geometry buffers into the WebView.
5. Replace the starter-Draft live-test bridge with governed catalog selection/adoption, standard-size/availability handling, and detailed runtime/manual-addition inputs.
6. Remove the temporary manual-assumption fallback only after its governed replacement supplies every required value or preserves a visible unavailable/blocked state.

## What this refactor does not change

- The existing accepted deterministic estimate remains the baseline calculation layer.
- The UI continues to call typed application services and never calculates prices itself.
- Native paths and CAD bytes remain host-owned and outside the WebView.
- The OCCT worker remains partially OS-contained and explicitly configured.
- Rate, route, runtime, material, and pricing authority remains human-governed and version-pinned.
- Existing TASK-003/004 containment, format, and fixture work remains required; it no longer obscures the shortest path to an internally useful estimator.
