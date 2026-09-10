# Usable Estimator Delivery Plan

> **Status:** In Review
> **Last updated:** 2026-09-10
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

PartProbe still ships with no production numeric rates. Internal tests may use named synthetic fixtures in isolated test code, but the ordinary arbitrary-model workflow must never apply one fixed synthetic estimate template and describe the result as model-derived.

## Delivery checkpoints

| Checkpoint | Outcome | Acceptance boundary |
|---|---|---|
| USE-1 Workflow guardrail | Upload-first Estimate view; rates/pricing moved to Settings; temporary manual manufacturing assumptions collapsed and explicitly labeled; stale displayed estimates clear after any input change | No formula, geometry, schema, or persistence change; exact STEP-only estimating remains session-only |
| USE-2 Durable shop setup | First-run Settings flow and SQLite-backed versioned rate/pricing/material/machine/runtime records | Empty-on-install numeric libraries; validation, effective dates, approvals, migrations, backup/replay, and missing/conflict states pass |
| USE-3 Model-derived stock and material | Exact STEP envelope/orientation evidence drives editable stock proposals; selected material supplies density and governed price | Stock allowance and selection policy versioned; model-sensitive fixtures and worked examples prove distinct geometry changes the proposal and estimate |
| USE-4 Coarse process and runtime | Exact STEP evidence plus reviewed shop profiles proposes broad process, setup count, programming, cutting, handling, and inspection time | Low/needs-review confidence and reason codes visible; estimator adoption recorded; never described as CAM accuracy |
| USE-5 Streamlined estimate | Primary workflow reduces to model, material, quantity, vital requirement flags, consolidated review, and result for a configured shop | Same Settings with materially different governed models produces appropriately different physical/manufacturing inputs and traceable results; missing authority stays unavailable/blocked |
| USE-6 Calibration and deployable alpha | Actual estimator review, calibration, save/reopen, backup, accessibility, supported importer evidence, and signed target packages | TASK-003/004/006/007 and release-acceptance evidence pass on declared targets |

USE-2 and the initial USE-3 domain contracts may proceed in parallel, but persisted values cannot become calculation authority until migration/replay and governance behavior are proven. Mesh results remain non-authoritative for estimating until a separately documented calculation policy, confidence rule, fixtures, and migration decision exist.

### USE-2 implementation status

The USE-2 foundation and first desktop slices are implemented behind typed application services and a repository port. Schema v1 stores organization currency plus optional immutable `RateCard` and `PricingPolicy` drafts, actor/time/reason evidence, optimistic concurrency, checksummed migration, integrity-on-open, backup/reopen, and historical replay. Schema v2 adds one optional bounded starter resource snapshot; schema v3 persists the bounded `shop-resource-catalog-v1` multi-entry aggregate, exact child/selection snapshots, and settings reference. Schema v4 persists content-minimized, exact-version-bound catalog authorization decisions with append-only triggers, normalized hash verification, idempotent exact correlation retry, changed-correlation rejection, no-default migrations, and backup/reopen evidence. A headless catalog application verifies exact current settings/catalog/reviewed-selection versions, evaluates a deployment policy, requires decision audit, and creates immutable successor versions. The native Settings state now composes independent repository and schema-v4 audit connections with a versioned policy adapter. Ordinary startup remains deny-all; a controlled exact-profile/exact-operator configuration passes mismatch/allow/reopen tests but requires a future trusted identity/session source before deployment use. Contract v7 preserves the exact seven-command set and adds complete path-free read-only catalog review to Settings; it exposes no catalog mutation fields and the legacy save path remains disabled/blocked for catalog-backed revisions. Persisted or active-for-proposals records remain non-authoritative and are not consumed by estimate evaluation. Remaining USE-2 work includes a separately versioned governed catalog request surface with host-owned identity, a reviewed shipped allow configuration, catalog editing/activation UI, Settings accessibility, broader recovery, and three-OS persistence evidence.

## Immediate implementation order

1. Preserve the completed USE-1 upload-first guardrail and deterministic application-service boundary.
2. Extend the contract-v7 read-only catalog boundary through a separately versioned path-free edit/activation request whose actor/time/correlation evidence is derived by the native host, with exact command permissions and regression tests. Keep ordinary startup deny-all until authenticated identity/role input and a shop-reviewed allow configuration exist. Numeric values remain explicitly test/shop owned, and active-for-proposals remains distinct from estimate authority.
3. Add a versioned exact-STEP stock-envelope proposal contract and analytic model-sensitive fixtures that consume only an explicitly reviewed resource snapshot before connecting it to pricing.
4. Add material selection and price resolution, then coarse runtime proposals, each with separate calculation rules, worked examples, and tests.
5. Replace the temporary manual-assumption panel only after its governed replacement supplies every required value or preserves a visible unavailable/blocked state.

## What this refactor does not change

- The existing accepted deterministic estimate remains the baseline calculation layer.
- The UI continues to call typed application services and never calculates prices itself.
- Native paths and CAD bytes remain host-owned and outside the WebView.
- The OCCT worker remains partially OS-contained and explicitly configured.
- Rate, route, runtime, material, and pricing authority remains human-governed and version-pinned.
- Existing TASK-003/004 containment, format, and fixture work remains required; it no longer obscures the shortest path to an internally useful estimator.
