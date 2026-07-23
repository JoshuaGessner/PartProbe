# Revision and Cost-Delta Domain Model

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-049–REQ-F-050, REQ-F-053–REQ-F-055, REQ-F-065; REQ-NF-015–REQ-NF-018, REQ-NF-020–REQ-NF-022  
> **Related ADRs:** ADR-0007, ADR-0008, ADR-0012  
> **Open questions:** Approved comparison tolerances; authoritative revision identity; cross-format comparison scope; cost-allocation presentation; assembly scope  
> **Dependencies:** Geometry/feature snapshots, calculation graph, requirement coverage, routing and quote lifecycle  
> **Supersedes:** None

## Purpose and invariants

Revision analysis explains what changed between two immutable model/estimate contexts and why the change may alter manufacturing. It is a proposal for review, not a CAD change-management authority or an automatic quote revision.

- Baseline and target source files, analyses, routings, estimates, quotes, prices, and overrides are immutable.
- A comparison pins both sides, units, coordinate frames, algorithms, dependencies, policies, and tolerances.
- Geometric evidence, recognized-feature interpretation, manufacturing impact, and financial impact remain separate layers.
- `unknown`, `unmatched`, and `not_comparable` are not encoded as zero/no-change.
- Only a new reviewed estimate graph can produce an authoritative cost/price delta.

## Aggregate

```text
ModelRevisionComparison
 ├─ ComparisonSide { source, geometry, feature, requirement, routing, estimate snapshots }
 ├─ RegistrationDecision
 ├─ GeometryChange [0..n]
 ├─ FeatureChange [0..n]
 ├─ RequirementChange [0..n]
 ├─ ManufacturingImpact [0..n]
 ├─ CostDelta [0..n]
 ├─ Warning / UnmatchedRegion [0..n]
 └─ ComparisonReview
```

| Record | Required content |
|---|---|
| `ComparisonSide` | part/assembly occurrence and revision identity; source hash; analysis/feature/coverage/routing/estimate snapshot IDs; representation and approval states |
| `RegistrationDecision` | source and target units/frames; transform; method (`identity`, `declared_placement`, `reviewed_rigid_alignment`); residual error; actor/reason; confidence |
| `GeometryChange` | stable comparison-local ID; `added`, `removed`, `modified`, `moved`, `unchanged`, `unmatched`, or `not_comparable`; affected refs/regions; dimensional/property delta; method/tolerance; representation; confidence/reasons |
| `FeatureChange` | baseline/target feature versions; `added`, `removed`, `modified`, `split`, `merged`, `reclassified`, `association_uncertain`; dimension/access/evidence delta; mapping method and review |
| `RequirementChange` | added/removed/modified/conflicting/carried applicability; source and coverage versions; downstream links made stale |
| `ManufacturingImpact` | change evidence to stock, setup, operation, machine, tool, runtime, inspection, outside service, risk, lead-time, or availability effect; status and rationale |
| `CostDelta` | baseline and target calculation node/result IDs; quantity/currency; internal cost, price, and lead-time deltas; allocation; rule/rate/profile versions; explanation |
| `ComparisonReview` | state, accepted/rejected correspondences, acknowledged unknowns, reviewer/time, approval scope; never approves a quote implicitly |

## Comparison tiers

1. **Identity:** hashes, filenames as display only, declared revision/part/occurrence, format/schema, source metadata.
2. **Normalization:** units, coordinate frame, body/occurrence scope, representation validity, healing state.
3. **Global properties:** AABB/OBB dimensions, volume, area, centroid/center of mass when density is comparable, body counts, symmetry indicators.
4. **Regional geometry:** added/removed material, changed regions/faces, wall thickness/radii/undercuts and access evidence with method-specific confidence.
5. **Manufacturing interpretation:** holes, pockets, slots, turning profiles and other reviewed features; stock, setup, machine/tool, inspection and risk effects.
6. **Estimate/quote:** runtime, direct/support/outside costs, price, lead time and assumptions through replayable calculation graphs.

Skipping a tier is explicit. A property delta may flag change but does not locate it; a geometric delta does not itself prove a manufacturing or cost delta.

## Topology instability and correspondence

Face/edge indexes and kernel topology identities are not durable across export, healing, re-tessellation, algorithm versions, or innocuous CAD edits. Matching precedence is:

1. validated persistent source identifier/GUID with compatible revision lineage;
2. exact topological history emitted by the same controlled operation;
3. geometry signature plus adjacency, orientation, body/occurrence context, and dimensional tolerance;
4. spatial overlap/region matching;
5. unresolved human correspondence.

Every match records method, score/reasons, ambiguity set, and versions. A low-confidence or many-to-many match becomes `split`, `merged`, or `association_uncertain`; it must not appear as definite deletion/addition. Stable comparison IDs are scoped to one comparison and do not pretend to be CAD topology IDs.

## Geometry and feature deltas

For validated exact solids, an isolated worker may compute common-region and added/removed-volume candidates using recorded boolean and tolerance policy. Failed or unstable booleans fall back to regional/tessellated evidence with a lower confidence label. Mesh comparisons are always approximate and require compatible scale, registration, watertightness policy, and sampling/tessellation parameters.

Feature comparison uses versioned feature snapshots. Reviewed/manual feature identity can be proposed for carry-forward, but changed geometry, access, evidence, or association marks it stale. Recognition reruns do not define truth: candidate changes are distinguished from reviewer-confirmed feature changes.

## Manufacturing and financial propagation

Impact propagation is an explainable dependency walk, not a blanket recalculation:

```text
source/requirement change
 → geometry and feature evidence
 → stock/access/setup/machine/tool/inspection/risk proposals
 → selected routing changes
 → versioned runtime and cost nodes
 → internal cost, price and lead-time delta
```

Each explanation identifies the triggering change and intermediate decisions. Example: deeper reviewed pocket → additional axial passes + longer-reach tool candidate → selected routing update → runtime node +14.2 min → internal cost node +$38.40. The numbers become authoritative only after the target routing/estimate is reviewed.

Cost delta is stored by non-overlapping component and reconciles exactly to target minus baseline at a stated quantity/currency. Rate, pricing policy, capacity/opportunity assumptions, and quantity differences are separately classified from design-driven change. If the comparison contexts differ for unrelated reasons, the UI must not label the entire delta “revision cost.”

Manual overrides never transfer silently. The user chooses `reapply_and_revalidate`, `replace`, or `do_not_carry`; the original and target decisions retain user/time/reason and affected calculation nodes.

## Revision and approval lifecycle

States are `queued`, `preflight_blocked`, `computed`, `computed_with_unknowns`, `under_review`, `reviewed`, `approved_for_estimate_draft`, `superseded`, and `failed`. Approval only authorizes selected comparison findings to seed a new draft estimate/coverage set. Quote publication remains governed by quote readiness and normal quote approval.

Re-running an algorithm or changing a tolerance creates a sibling comparison. Comparison results may be reproduced only if both snapshots and evaluator versions are available; otherwise retain rendered evidence and mark replay unavailable. Historical cost deltas never float to current rates unless the user intentionally creates a new scenario.

## Security and retention

The comparison inherits the stricter classification and permissions of both sides. Derived overlays, difference meshes, screenshots, annotations, cost explanations, and exports inherit that boundary. Import/comparison workers have no network, no unrestricted external references, bounded resources, and content-redacted diagnostics. Cross-project or cross-customer comparison is denied by default even when hashes match.

## Staged scope

- **Foundation:** hash/unit/frame/global-property comparison; side-by-side/ghost views; manual mapping; immutable review; no automatic cost claim.
- **First revision capability:** validated exact added/removed-region candidates, reviewed feature/requirement deltas, stale-impact graph, draft estimate replay, reconciled explanations.
- **Advanced:** cross-format/assembly matching, robust split/merge lineage, expanded topology/change classes, CAM/actual integration, calibrated prioritization.

No stage may overwrite the baseline or present unreviewed inference as customer-ready. TEST-056–TEST-060 define the minimum comparison evidence; PMI and requirement integration additionally use TEST-061–TEST-069.
