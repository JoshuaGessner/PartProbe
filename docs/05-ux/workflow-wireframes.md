# Workflow Wireframes

## Metadata

- **Status:** Draft
- **Last updated:** 2026-09-11
- **Related requirement IDs:** UX-001 through UX-012, UX-021 through UX-045, REQ-F-001, REQ-F-006 through REQ-F-065
- **Related architecture decision IDs:** ADR-0001
- **Open questions:** Validate compact pane widths and total-strip contents with estimators.
- **Dependencies:** [Screen inventory](screen-inventory.md), [model review workflow](model-review-workflow.md)
- **Supersedes / superseded by:** None

These are structural wireframes, not visual designs. They specify hierarchy, visible evidence, and interaction regions; the [design system](design-system.md) defines appearance.

## Basic rate setup

```text
┌ Rate Setup ─ Organization policy ──────────────────────────────────────────┐
│ Currency [USD ▼] (explicit, not inferred)  Rounding [Half-even / 2 places] │
│ Required: 8  Approved: 5  Missing: 2  Conflict: 1   [Import preview]       │
├──────────────┬─────────┬──────────┬─────────────┬───────────┬──────────────┤
│ Category     │ Amount  │ Basis    │ Scope       │ Effective │ State/source │
│ Setup labor  │ 125.00  │ USD/hour│ Workcenter… │ 2026-01-01│ Approved     │
│ Machine      │ —       │ USD/hour│ Machine…    │ —         │ Missing      │
│ Inspection   │ 95.00 ! │ USD/hour│ Org         │ 2026-01-01│ Conflict (2) │
├──────────────┴─────────┴──────────┴─────────────┴───────────┴──────────────┤
│ Selected row inspector: formula preview / evidence / history / affected use │
│ [Save draft] [Submit review]              [Resolve errors before approval] │
└────────────────────────────────────────────────────────────────────────────┘
```

CSV and bulk-paste actions first open a bounded dry-run table with row-level errors; no imported row becomes authoritative until validation and explicit acceptance. A missing/conflicting-rate link from an estimate opens this screen with the affected category and scope selected.

The current contract-v10 checkpoint separates the upload/analyze Estimate workspace from durable local Settings. Rate/pricing drafts and one optional typed material/offer/stock/machine/runtime starter bundle persist as immutable revisions; reload clears confirmations. An explicit proposal command now applies the saved starter bundle to exact STEP bounds, returns review-only model-sensitive stock/material/coarse-runtime inputs, and requires review plus named limitation acceptance before estimate adoption. The ordinary workflow never silently loads test values. Settings offers an explicit Huntsville research-informed testing profile that remains unconfirmed and unsaved until reviewed; it is not verified market or deployment pricing.

## Shop resource catalog

```text
┌ Shop Settings ─ Resources ─ Catalog v… ─ Draft/identity status ───────────┐
│ [Materials] [Offers] [Stock] [Machines] [Runtime]   [Search records…]     │
├───────────────┬──────────────────────────┬─────────────────────────────────┤
│ Category      │ Records                  │ Selected record                 │
│ Materials  12 │ 6061-T6        Reviewed │ Identity / immutable version    │
│ Offers     18 │ 7075-T6        Draft    │ Editable successor fields       │
│ Stock       6 │ 1018 CRS       Reviewed │ Units / references / source     │
│ Machines    4 │ [+ New material]         │ Validation / lifecycle impact   │
│ Runtime     7 │                          │ [History / affected references] │
├───────────────┴──────────────────────────┴─────────────────────────────────┤
│ Unsaved: 1 successor | Activation will return to Reviewed                 │
│ [Discard local changes]                         [Save catalog draft]       │
└────────────────────────────────────────────────────────────────────────────┘
```

Only one record detail is expanded at a time. Categories and records retain selection while the user reviews another record, but unsaved changes remain explicit and never autosave into authority. A deep link from Estimate opens the affected category and record, then returns to the same estimate context. Contract v9 supplies the native save boundary; ordinary startup is identity-unavailable, so the editor remains a planned disabled/empty-state UI until a trusted native identity/session source is configured.

## Quote workspace

```text
┌ App/menu ─ Search/command ─ Save status ─ User ────────────────────────────┐
├ Nav ┬ Customer / RFQ-1042 / Quote Q-1042 Rev B       [Draft] [Review]      ┤
│Home │ Overview  Parts  Routing  Costs  Risks  Approval  Output              │
│RFQs ├───────────────────────────────────────────────────────────────────────┤
│Quotes│ Part: 28-441 Rev C  STEP + drawing  | Warnings: 2 | Confidence: Med  │
│Parts ├──────────────┬───────────────────────────────┬───────────────────────┤
│...   │ Parts/tree   │ Contextual workspace          │ Inspector             │
│      │ · Housing    │ (tab-specific content)        │ Evidence / history    │
│      │ · Bracket    │                               │ affected costs        │
├──────┴──────────────┴───────────────────────────────┴───────────────────────┤
│ Cost $… | Risk $… | Price $… | Margin … | Qty 1 / 10 / 25 | Recalculated…  │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Model intake and validation

```text
┌ Import model: Part 28-441 Rev C ────────────────────────────────────────────┐
│ [Drop STEP, STL, 3MF]  or  [Choose local file]     Files stay local          │
│ File / format / hash [working…]  | revision association                       │
├─────────────────────────────┬────────────────────────────────────────────────┤
│ Checks                      │ Preview / facts                                │
│ ✓ file identified           │ units: millimeter (imported) [Confirm]         │
│ ! scale needs review        │ AABB: …  Volume: …  Bodies: …                  │
│ ! mesh not watertight       │ warnings describe impact and safe next action  │
├─────────────────────────────┴────────────────────────────────────────────────┤
│ [Cancel] [Save reviewed-with-warnings]                 [Continue to review] │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Routing and cost explanation

```text
┌ Routing: 28-441 ────────────────────────────────────────────────────────────┐
│ [Add operation] [Apply template] [Compare approach] [Undo]                  │
├────┬──────────────┬───────────┬─────────┬─────────┬────────┬───────────────┤
│ #  │ Operation    │ Workcenter│ Setup h │ Run min │ Source │ Confidence    │
│ 10 │ Saw blank    │ Saw-01    │ 0.10    │ 2.5     │ Manual │ —             │
│ 20 │ Mill setup A │ VMC-03    │ 1.50    │ 18.2    │ Suggest│ Medium (why)  │
│ 30 │ Inspect      │ CMM       │ 0.40    │ 7.0     │ Manual │ High          │
├────┴──────────────┴───────────┴─────────┴─────────┴────────┴───────────────┤
│ Selected: Mill setup A  [Geometry] [Tools] [Formula] [Overrides]             │
│ Setup $… + run $… + tooling $… → operation cost $…  [Open cost breakdown]   │
│ Rate trace: selected card/entry/version/effective scope (why) [Change/override]│
└─────────────────────────────────────────────────────────────────────────────┘
```

## Approval

```text
┌ Approval review ─ Quote Q-1042 Rev B ───────────────────────────────────────┐
│ Ready: 12 checks  | Needs review: 2  | Blocked: 0                            │
│ Cost $…  Risk reserve $…  Selling price $…  Margin …                          │
│ [2] low-confidence feature proposals  [1] price override (reason shown)     │
│ Snapshot: model hash / rate card / feeds / algorithms / user changes         │
│ Route / requirements / availability / uncertainty / policy versions pinned   │
│ [Back to edit]                                      [Request approval]       │
└─────────────────────────────────────────────────────────────────────────────┘
```

Advanced route, capacity, uncertainty, revision, coverage, sourcing, CAM, correction, and bid interactions are specified in [advanced analysis](advanced-analysis-workflow.md), [route comparison](routing-comparison.md), [requirement coverage](requirement-coverage-workflow.md), and [revision comparison](revision-comparison-workflow.md). Each preserves a visible baseline and uses details-on-demand to prevent dense expert evidence from becoming a wall of controls.
