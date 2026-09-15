# Usable Estimator Delivery Plan

> **Status:** In Review
> **Last updated:** 2026-09-15
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

The USE-3 engine checkpoint implements `partprobe-rectangular-stock-envelope-policy` v1.0.0 and the distinct internal `partprobe-developer-estimate-input-proposal` v1.0.0 in `crates/setup-planner`. ABI-v4 supplies one-solid exact STEP AABB evidence. The internal proposal resolves an exact schema-v2 starter Draft into model-sensitive blank, material mass/cost, machine-fit, and coarse-runtime evidence; `partprobe-developer-estimate-proposal-adoption` v1.0.0 requires explicit review, named limitation acceptance, quantity, native actor/time, and reason before the existing deterministic engine runs. Contract v12 preserves v10's path-free evidence/exclusion trace and v11's Model & stock workspace command, then adds one path-free standard-view command. Analytic/native cube/prism tests prove model sensitivity. VIS-1 phases A/B prove the isolated native GPU scene, visual language, and a same-window Metal screen with return navigation, resize, four semantic keyboard views, fragment-lit normals, and a path-free text equivalent. The three workspace destinations now expose page-current state and focus the new heading only after a successful transition; live assistive-technology capture remains open. VIS-2 carries governed native cube/prism display artifacts through the application session and internal desktop's validated indexed renderer path. `proposed-stock-display-placement-v1` now adds the matching proposal blank as source-axis, centered, equal-per-side, review-only stock while leaving standard size/availability unresolved. The packaged host can create its own private temporary supervisor root, and the internal viewer build no longer requires a process flag. A fresh direct-launch package rehearsal now passes through Settings, STEP analysis, automatic proposal, combined reviews, explicit-zero test pieces, estimate, and visible source/stock review. The viewer additionally requests one fallback adapter, detects device loss, retries a lost/outdated surface once, and preserves accepted work through full-width text-only failover. Child-WebView API acceptance/replacement, forced fallback/device-loss platform evidence, live VoiceOver/Narrator/Orca evidence, governed editable stock, standard-size/availability, detailed runtime/adders, and ABI-v4/display-ABI Linux/Windows refresh remain next.

### Model and stock visualizer/editor increment

The 2026-09-15 lifecycle slice guards all GPU-touching viewer operations against already-observed loss and proves the production callback using headless API-induced Metal device destruction. This closes an entry-point safety gap, not the live no-adapter/driver-loss platform gate. The package and screen-reader evidence must still be refreshed independently; full in-session recreation and immutable stock editing remain the next decisions/slices.

The source-bound application viewer is now an implemented internal slice of USE-3, not a supported product capability or decorative preview. It remains inside the existing architecture: the worker creates a separately versioned and bounded display derivative; the native `model-viewer` owns validated source-bound indexed GPU buffers, lighting, camera, and the display-only proposed-stock layer; the application owns selection/analysis identity and proposal calculation; and the WebView receives only path-free lifecycle/view state plus already-reviewed analysis/proposal facts through contract v12. Raw CAD, paths, and vertex/index buffers do not cross into the WebView. Editable candidates, standard-size resolution, and availability remain planned.

| Increment | Outcome | Exit evidence |
|---|---|---|
| VIS-1 renderer spike — In Progress | Phase A pins/reviews `wgpu 30.0.1` and proves a bounded offscreen native scene on Metal; phase B proves a build-profile-gated same-window Tauri screen and path-free contract-v12 workspace/standard-view control, while Tauri's child-WebView API remains feature-gated unstable | Complete locally: GPU-lit model normals, translucent stock/edge overlay, four semantic keyboard-operable standard views, path-free model/proposal text equivalent, page-current workspace navigation, success-bound heading focus, bounded offscreen/surface dimensions, viewport containment, one fallback-adapter request, device-loss observation, one bounded lost/outdated surface retry, text-only failover, Metal presentation inside the existing window, return navigation, live resize, and dependency record. Open: explicit API acceptance or stable replacement, forced loss/no-adapter platform evidence, full-device-recreation decision, live VoiceOver/Narrator/Orca acceptance, broader HiDPI, Windows/Linux and signed package evidence |
| VIS-2 governed display derivative — In Progress | The native-only manifest/framing/decoder bind source and analysis identity, profile, bounds, layouts, references, chunks, hashes, and limits. Additive OCCT display ABI v1 tessellates the same immutable STEP bytes after measurement, worker schema v2 emits one source/analysis-bound artifact, the supervisor claims both fixed outputs under one quota, the application retains only validated arrays, and the developer desktop replaces the matching native scene | Complete: manifest/artifact rejection suites, protocol migration, shared-quota dual claim, native cube/prism tessellation/emission, hard limit/cancellation, real supervisor/application retention, indexed renderer conversion, stale-scene clearing, configured desktop retention, accurate pending state, and live source-bound cube framing on Apple Silicon. Open: representative performance and Linux/Windows/package evidence |
| VIS-3 model/stock review — In Progress | **Model & stock** opens the source-bound native viewport, standard views, textual model/proposal evidence, warnings, provenance, and the `proposed-stock-display-placement-v1` translucent centered blank; scene tree, edges/stock toggles, editable candidates, and standard-size/availability remain planned | Contract-v12 command/permission parity, lighting/stock buffers, enclosure rejection, stale clearing, keyboard/textual evidence, page-current navigation, success-bound heading focus, text-only GPU-unavailable fallback logic, and fresh packaged Apple-Silicon source-plus-stock inspection pass; forced platform fallback, live screen-reader acceptance, and remaining controls remain |
| VIS-4 governed stock revision — In Progress | The final-X/Y/Z engine and bounded application history enforce exact latest ID/revision/settings, retain review, and now adopt/evaluate sealed coarse candidate inputs as a distinct snapshot without replacing the original baseline | Fourteen engine, nine session-review, and seven candidate-adoption tests pass, including quantity/evidence pins, stale adoption, geometry/rate/pricing gates, retained unavailable baseline state, and unchanged original result. Authorization/audit, native selection-token composition, DTO/permission migration, numeric editor, and renderer integration remain; no drag-only workflow |
| VIS-5 platform acceptance | Harden Metal/Vulkan/D3D12 packages and representative model behavior | Three-OS package, accessibility, scaling, memory, cancellation, device-loss, and visual-reference evidence |

The implemented `proposed-stock-display-placement-v1` draws only the unchanged application proposal's rectangular dimensions. The renderer derives the retained scene minimum/maximum, source-axis aligns and centers the blank, and thus splits each total-axis allowance equally around the analyzed bounds. The UI/native notice records that review-required display assumption and leaves standard size, availability, purchasing, workholding, and routing unresolved. Re-analysis or a saved Settings change clears the layer. VIS-4 now retains immutable candidate revisions, separate review, and distinct adoption/evaluation in application, but authorization/audit, native selection-token/contract/editor, and explicitly versioned candidate rendering must precede any divergence of displayed/GUI-estimated stock from the original proposal.

### USE-2 implementation status

The USE-2 foundation and first desktop slices are implemented behind typed application services and a repository port. Schemas v1-v4 persist immutable rate/pricing, starter-resource, multi-entry catalog/selection, and append-only authorization-decision evidence without numeric defaults. Separate catalog applications govern immutable Draft creation/editing and policy/audit-gated proposal eligibility. Contract v7 adds path-free catalog review, v8 adds native-owned activation, and v9 adds a complete bounded native-owned catalog-draft save command. The native adapter maps all five record kinds, preserves matching source evidence across equivalent decimal formatting, rejects changed content under reused versions, and executes storage off the UI task. Ordinary startup supplies no authenticated actor, returns explicit unavailability for both catalog commands, and retains deny-all activation; the WebView invokes neither. Persisted drafts or active-for-proposals records remain non-authoritative and are not consumed by estimate evaluation. Remaining USE-2 work includes the governed category/list/detail editor, record review, authenticated identity/roles, a reviewed shipped allow configuration, activation UI, Settings accessibility, broader recovery, and three-OS persistence evidence.

## Immediate implementation order

1. Preserve the completed USE-1 upload-first guardrail and deterministic application-service boundary.
2. Build the governed Settings editor on contract v9 using category navigation, a searchable record list, one selected-record inspector/editor, and a persistent save/review footer. Keep ordinary startup catalog actions identity-unavailable and activation deny-all until authenticated identity/role input and a shop-reviewed allow configuration exist. Numeric values remain explicitly test/shop owned, and active-for-proposals remains distinct from estimate authority.
3. Preserve the completed fresh macOS package and direct-launch Settings→STEP→automatic-proposal→estimate rehearsal without changing `geometry-snapshot-v1` or treating display tessellation as measurement authority.
4. Force and validate VIS-3 fallback on declared platforms, capture the implemented heading-focus path with declared screen readers, finish remaining accessible control behavior, and decide whether full in-session device recreation is required; then implement VIS-4's immutable governed stock-revision path before calling USE-3 complete. Do not replace analysis output, silently modify the documented display-only centered placement, or move geometry buffers into the WebView.
5. Replace the starter-Draft live-test bridge with governed catalog selection/adoption, standard-size/availability handling, and detailed runtime/manual-addition inputs.
6. Remove the temporary manual-assumption fallback only after its governed replacement supplies every required value or preserves a visible unavailable/blocked state.

## What this refactor does not change

- The existing accepted deterministic estimate remains the baseline calculation layer.
- The UI continues to call typed application services and never calculates prices itself.
- Native paths and CAD bytes remain host-owned and outside the WebView.
- The OCCT worker remains partially OS-contained and explicitly configured.
- Rate, route, runtime, material, and pricing authority remains human-governed and version-pinned.
- Existing TASK-003/004 containment, format, and fixture work remains required; it no longer obscures the shortest path to an internally useful estimator.
