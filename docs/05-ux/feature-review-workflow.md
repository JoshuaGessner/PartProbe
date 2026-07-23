# Feature Review Workflow

## Metadata

- **Status:** Draft
- **Last updated:** 2026-07-22
- **Related requirement IDs:** UX-004, UX-006, FEAT-001 through FEAT-020, REQ-F-013 through REQ-F-018
- **Related architecture decision IDs:** ADR-0001; feature-recognition ADR pending
- **Open questions:** Initial feature classes and confidence bands; merge/split behavior for ambiguous geometry.
- **Dependencies:** [Model review workflow](model-review-workflow.md), feature-recognition pipeline (pending)
- **Supersedes / superseded by:** None

## Intent

Feature recognition is a review queue, not an assertion of manufacturing truth. The estimator can accept, reject, edit, merge, split, or define a feature while seeing geometry evidence and consequences for tools, setups, routing, time, confidence, and cost.

## Review loop

1. Filter the feature list by status, type, confidence, direction, warning, or affected operation.
2. Selecting a row focuses/highlights its referenced geometry and opens an inspector with dimensions, candidate tools/processes/setups, warnings, algorithm version, and history.
3. User accepts, rejects, edits dimensions/type, or creates a manual feature. Manual and algorithmic identities remain distinct.
4. The impact panel previews affected suggestions and marks them stale; it does not silently change a routing.
5. User applies the proposed updates individually or in a reviewed batch; retained manual choices win.

## States and signals

`Detected`, `Needs review`, `Accepted`, `Rejected`, `Modified`, `Merged`, `Split`, `Manual`, and `Superseded` are textual states with icons and accessible labels. Confidence is `High`, `Medium`, `Low`, or `Unknown`, accompanied by reasons such as `mesh-only input`, `limited tool access`, or `geometry repaired`. A rejected feature remains discoverable in history.

## Batch behavior and safeguards

Batch accept is allowed only for same-type features with no blocking warning. The preview lists count, confidence distribution, pending impact, and exception rows. Merge/split uses a guided inspector: select source features/geometry, choose resulting type, review dimensions and provenance, then commit. All operations are undoable before approval and audited after durable save.

## Accessibility alternative

Every viewport action has an equivalent list/tree action. The inspector describes the selected geometry reference in text; it does not require color or spatial perception to understand selection. Focus returns to the triggering feature row after a viewer command unless the user explicitly opens the inspector.
