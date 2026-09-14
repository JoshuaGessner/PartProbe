# Model Review Workflow

## Metadata

- **Status:** Draft
- **Last updated:** 2026-09-13
- **Related requirement IDs:** UX-003, UX-004, UX-006, GEO-001 through GEO-012, REQ-F-006 through REQ-F-012
- **Related architecture decision IDs:** ADR-0001, ADR-0003, ADR-0004
- **Open questions:** Exact first-slice geometry warning taxonomy and default confidence thresholds.
- **Dependencies:** [Geometry engine](../04-architecture/geometry-engine.md), [feature review workflow](feature-review-workflow.md), [accessibility](accessibility.md)
- **Supersedes / superseded by:** None

## Goal

Turn an imported model into a reviewed analysis snapshot without implying that geometry alone communicates drawing requirements or a safe manufacturing plan.

## Stages

1. **Intake.** User selects/drops a model. Show filename, hash progress, format, revision association, and local-data notice.
2. **Unit and integrity gate.** Show imported or inferred units, scale preview, validity/watertightness/body warnings, and a clear choice: confirm, correct, continue with warning, or replace file. Do not infer confirmation from clicking away.
3. **Geometry evidence.** Present viewport, model tree, properties (AABB/OBB, volume, surface area, mass when material selected), analysis provenance, and warning list. Every measurement includes a unit and derivation/source link.
4. **Requirements gap.** A persistent prompt distinguishes model facts from missing drawing-driven requirements: tolerance, GD&T, finish, thread, material spec, inspection, process, certification, and customer clauses. It links to package attachments and a checklist.
5. **Stock/process proposal.** Show editable candidate stock forms and broad process hypotheses, confidence, constraints, and alternatives. No proposal is auto-approved.
6. **Snapshot.** Accepting creates a versioned analysis snapshot. Re-analysis creates a new candidate and compares differences; it never overwrites accepted routing/overrides.

## Workspace composition

Left: model tree and selection filters. Center: viewport with orientation cube, view controls, legend, measure/status overlay. Right: contextual inspector for selected body/face/feature plus analysis/warning tabs. Bottom (resizable): geometry properties, import log, requirements checklist. The total strip is omitted here unless opened inside a quote workspace; avoid competing summary hierarchy.

For the first stock-review increment, open this workspace from the Estimate screen with `View model & stock`. Limit the tree to Model and Proposed stock, the viewport to standard views/fit/edges/stock visibility, and the inspector to model size, proposed stock size, total X/Y/Z allowance, material/mass/removal, availability, machine fit, and proposal evidence. The stock overlay is translucent with a high-contrast boundary; enclosure violations also have explicit text. Initial edits use labeled numeric fields with units and a reason, not pointer-only handles. `Use this stock candidate` is the single primary action and creates or adopts a review-bound candidate revision; it never edits source geometry.

The ordinary UI remains facts-only. A separately feature-gated contract-v12 developer build opens the same-window native viewport and replaces its pre-analysis synthetic model with the matching validated selected-STEP display derivative, while the Estimate workspace shows exact STEP measurements and a model-sensitive rectangular stock proposal. Its current inspector provides semantic Isometric/Front/Top/Right controls with actual-view feedback plus path-free representation, units, dimensions, warnings, confidence, analysis reference, and proposal facts. Before proposal preparation, stock remains explicitly unavailable. A matching proposal adds its exact rectangular blank under the documented source-axis, centered/equal-per-side `proposed-stock-display-placement-v1`; distinct GPU lighting and translucent amber stock make the source/blank relationship visible. This is review-only developer display evidence, not measurement, purchasable-stock resolution, CAM, or estimate authority.

## Viewport behavior

Default camera fits model with a clearly labeled axis and unit. Selection is high-contrast outline plus inspector update; hover is subtle and does not replace selection. Hide/isolate/section/measure are explicit modes with persistent mode labels and a keyboard exit. The model tree remains a complete alternative to pointer picking. Loading shows a simplified/progress state; an unavailable GPU renderer offers model facts and a retry/diagnostic path.

## Completion rule

The stage is complete only when unit state, warnings, property confidence, source hash, algorithm version, and user decision are recorded. `Reviewed with warnings` is a valid explicit outcome; `Ready for feature review` does not mean drawing requirements are complete.
