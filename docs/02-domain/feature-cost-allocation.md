# Feature-Level Cost and Risk Allocation

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-050, REQ-F-055, REQ-NF-021  
> **Related ADRs:** ADR-0008, ADR-0012  
> **Open questions:** OQ-039, OQ-040, OQ-050  
> **Dependencies:** feature evidence, routing, requirement coverage, estimation graph, revision comparison  
> **Supersedes:** None

## Purpose and boundary

This view explains where estimated cost, time, and risk appear to come from by connecting calculation nodes to operations, setups, features, requirements, and model regions. It is an allocation and navigation aid—not an assertion that a geometric feature alone caused a cost, a substitute for the category ledger, or a new pricing authority.

## Allocation record

Each `FeatureCostAllocation` identifies an immutable estimate snapshot, route and operation, cost/time/risk node, target feature/requirement/model region, allocation method and version, allocated amount or share, confidence, explanation, provenance, and review state. Targets may be exact, grouped, indirect, shared, or unresolved. The sum of allocations plus an explicit `UnallocatedRemainder` must reconcile to the authoritative node; rounding occurs only under the calculation rule.

Supported methods progress from direct operation ownership, to estimator-authored shares, to documented deterministic heuristics. A heuristic must never fabricate precision for shared setup, program, inspection, outside-process, material, overhead, or risk costs. Unallocated and many-to-many relationships remain visible.

## Risk and confidence

Allocation confidence answers “how reliable is this mapping?” Risk answers “what adverse estimate outcome is represented?” Neither is probabilistic uncertainty. Revision mapping confidence is separately preserved. Unknown or ambiguous geometry produces an unresolved allocation, never a zero amount.

## Lifecycle and versioning

Allocations are draft proposals until reviewed. Publishing pins the feature/requirement snapshots, route, calculation graph, allocation method, and reviewer decision. Re-analysis or a new revision creates a branch; it cannot rewrite published allocations. Revision cost views may reuse allocation evidence only when the mapping and estimate/rate basis are explicit.

## Security and UX

The overlay receives sanitized stable region IDs, display geometry, values, and classification—not raw kernel objects. Role and export policies for price, margin, customer data, and controlled technical data still apply. The UI must provide table, overlay, keyboard, and non-color equivalents; selecting a region cross-highlights its feature, requirement, operation, calculation trace, and unallocated context.

## Staged delivery

1. Phase 2: category and operation drill-down only.
2. Phase 3: reviewed direct/manual mappings and visible unallocated remainder.
3. Phase 5: requirement links and revision-aware overlays.
4. Phase 7: validated deterministic allocation suggestions and sensitivity overlays.

## Validation obligations

TEST-070–TEST-073 cover reconciliation, ambiguity, cross-highlighting/accessibility, and immutable version replay. Shop validation must determine which costs estimators consider meaningfully attributable, acceptable remainder thresholds, and whether allocation improves explanation without encouraging false precision.
