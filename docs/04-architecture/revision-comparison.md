# Revision Comparison Architecture

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-049–REQ-F-050, REQ-F-053–REQ-F-055, REQ-F-065; REQ-NF-015–REQ-NF-018, REQ-NF-020–REQ-NF-022  
> **Related ADRs:** ADR-0002–ADR-0008, ADR-0012  
> **Open questions:** Production tolerances and resource budgets; reliable source GUID availability; assembly and cross-format scope; comparison retention  
> **Dependencies:** Geometry worker, feature pipeline, requirement coverage, versioned estimation graph, viewer, policy/audit services  
> **Supersedes:** None

## Boundary and responsibilities

Revision comparison is an orchestrated, versioned analysis across existing boundaries. It does not add a second geometry authority or allow the viewer to compute authoritative measurements.

| Component | Responsibility | Must not do |
|---|---|---|
| `RevisionComparisonCoordinator` | validate side identities; pin snapshots/profiles; sequence stages; persist outcomes; cancellation/resume | interpret CAD bytes or mutate either side |
| Geometry worker comparison adapter | unit/frame normalization evidence; property/region/topology candidates; bounded difference artifacts | set manufacturing meaning or access repositories directly |
| `CorrespondenceEngine` | confidence-rated body/region/topology mapping and ambiguity sets | assert durable face identity without evidence |
| Feature/requirement differ | compare immutable reviewed/proposed snapshots; mark stale links | accept candidates or carry requirements automatically |
| `ImpactAnalyzer` | walk typed dependencies and propose affected stock/routing/tool/inspection/risk nodes | calculate customer price or edit approved routing |
| Estimation engine | replay target draft with pinned rule/rate/input versions; reconcile deltas | rewrite historical graph results |
| Viewer presenter | consume sanitized tessellation, overlays, selections and explanations | serve as measurement/diff authority |
| Policy/audit service | project/classification access, exception/approval checks, immutable events | expose content in ordinary logs |

## Versioned contract

`ComparisonRequest` pins baseline/target source hashes and revision/occurrence IDs; geometry, feature, requirement, routing, estimate and quote snapshot IDs where present; comparison profile/version; units/frames; tolerance policy; requested stages; actor/project; and security classification.

`ComparisonResult` records request hash, stage outcomes, engine/kernel/importer/feature/calculation versions, OS/architecture, transforms, mappings, changes, unknown/unmatched regions, warnings, resource metrics, derived-artifact hashes, review state, and schema version. Results are immutable. A changed input, dependency, tolerance, reviewer mapping, or algorithm produces a sibling result linked by supersession—not an in-place update.

Stage output uses `succeeded`, `succeeded_with_warnings`, `needs_user_input`, `failed_recoverable`, `failed_terminal`, or `not_run`. A downstream stage cannot convert missing upstream evidence into “no change.”

## Pipeline

1. **Authorize and identify.** Enforce access to both sides, classification compatibility, source hashes, part/occurrence/revision scope, and retention policy.
2. **Preflight.** Compare format/schema, units, representation, body/assembly structure, validity/healing and available snapshots. Block ambiguous units or identity.
3. **Register.** Default to identical declared part coordinate frames. A proposed rigid alignment is previewed and recorded; never auto-align in a way that hides an intentional placement/orientation change.
4. **Global compare.** Hash, body counts, envelope, volume, area, centroid/center-of-mass basis and other eligible properties using explicit tolerances.
5. **Map and localize.** Use validated persistent IDs first, then confidence-rated signatures/adjacency/overlap. Compute exact-solid common/added/removed candidates where valid; degrade explicitly to approximate regional/mesh evidence.
6. **Compare interpretation.** Diff feature and requirement snapshots; separate recognizer-output drift from source-design change; mark affected links stale.
7. **Propagate impact.** Produce a typed, acyclic proposal from accepted change evidence to stock, process, setup, machine, tool, runtime, inspection, supplier, risk, lead-time, and calculation nodes.
8. **Recalculate draft.** Only on user request, clone the target context into a new draft and replay selected affected nodes. Require explicit override carry-forward decisions.
9. **Review and publish.** Persist correspondences, accepted/rejected findings, unknown acknowledgements, explanations and approval scope. Quote readiness and quote approval remain separate gates.

Each stage is idempotent for the same request/profile and content-addresses large derived artifacts. Cancellation or worker failure leaves neither side changed and records only a sanitized failed attempt.

## Comparison methods and confidence

- Byte-identical inputs are `same_source_bytes`; they can still yield analysis-version differences, which are reported separately.
- Property comparisons use canonical units and field-specific absolute/relative tolerances, never display-rounded strings.
- Exact B-rep region candidates require validated solids and an independently tested boolean/tolerance profile. Boolean failure is an unknown, not zero difference.
- Mesh distance/voxel/sampling methods name resolution and approximation. Mesh evidence cannot inherit exact-solid confidence.
- Topology mapping scores identifier lineage, geometry signature, adjacency, orientation and overlap. Competing matches are retained; thresholds and reason codes are versioned.
- An importer/healing/recognizer change is classified as `analysis_pipeline_delta` unless source evidence supports a design change.

The comparison retains both source geometry references and stable comparison-local region IDs. Viewer meshes map to those IDs through a versioned selection map.

## Cost and requirement integration

The impact graph contains evidence edges, not hidden imperative updates. Accepted changes invalidate or mark stale dependent suggestions. The estimator chooses target decisions before recalculation. The resulting `CostDelta` references exact baseline/target calculation nodes and separates:

- geometry/feature/requirement-driven changes;
- routing/tool/machine/inspection decisions;
- quantity, rate, pricing, capacity/opportunity, currency and policy changes;
- unknown or unallocated remainder.

Components reconcile exactly to target minus baseline. A requirement removed from the target is not treated as truly removed until source/revision applicability is reviewed. Imported PMI follows the same coverage gate.

## Visual-delivery contract

The worker emits sanitized baseline/target tessellations, added/removed/common region layers, uncertainty masks, comparison-local IDs, and metadata checksums. Supported presentations include side-by-side synchronized cameras, ghosting, overlays, sectional views and change lists. Snapshot-backed feature layers may show operation, setup, tool, machine, cutting/support time, cost, inspection, risk, confidence, accessibility, revision delta and removal intensity; missing or unallocable values remain explicit. Colors are configurable and always paired with pattern/outline/text labels (`Added`, `Removed`, `Modified`, `Uncertain`). Picking never substitutes a renderer-derived measurement or allocates cost.

Large comparisons stream bounded chunks and may use level-of-detail while preserving a full textual change list. A missing GPU path still exposes properties, findings, uncertainties, approvals, and artifact export under policy.

## Security and failure containment

- Both source packages and every derivative inherit the stricter project/classification policy.
- Comparison executes in the isolated no-network geometry worker with time, memory, entity and artifact quotas; external references remain disabled unless explicitly staged and authorized.
- Cache keys include tenant/project/classification boundary and hashes; no cross-project deduplication is user-visible.
- Diagnostics contain codes/counts and correlation IDs, not geometry, PMI text, filenames, customer names, coordinates, prices, or thumbnails by default.
- Comparison exports, screenshots, clipboard and support bundles require policy authorization and audit.
- Partial/failure artifacts are cleaned according to retention policy; source and approved records are never cleanup targets.

## Staging and acceptance

The foundation stage adds contracts/storage plus hash, unit/frame, global property and manual-review comparisons. The first post-vertical-slice capability adds validated B-rep regional overlays, feature/requirement diff, impact invalidation and draft cost replay. Advanced stages add cross-format/assembly matching and calibrated high-scale methods. AP242 PMI parsing is independently feature-gated: comparison can inventory unsupported PMI without pretending it is equal.

Enable each stage only after its fixture classes pass `revision-comparison-validation.md`, three-platform package evidence is available, reviewer UX is usable without color/spatial perception, and false-negative/unknown behavior is accepted by engineering and estimating reviewers.
