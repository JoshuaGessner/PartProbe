# Routing Alternatives

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-006, REQ-F-010, REQ-F-027, REQ-F-040–REQ-F-042, REQ-F-065; REQ-NF-015–REQ-NF-018, REQ-NF-021–REQ-NF-022; TEST-040–TEST-044  
> **Related ADRs:** ADR-0007–ADR-0009  
> **Open questions:** Approved route objectives, hard feasibility rules, and route-approval roles by shop  
> **Dependencies:** Geometry/setup/runtime evidence, shop libraries, capacity snapshots, pricing policy  
> **Supersedes:** None

## Purpose and boundary

A `RoutingAlternativeSet` gives one estimate revision and quantity scenario several plausible ways to manufacture or source the same deliverable. Alternatives may vary process, machine class, stock, orientation, setup count, workholding, tooling, inspection, outside processing, availability, and capacity demand.

An alternative is an estimating plan, not production CAM, a released shop router, or proof of manufacturability. The system proposes, compares, and explains; an authorized user adopts the routing used by an estimate.

## Aggregate and records

`RoutingAlternativeSet` is owned by an estimate revision and quantity scenario. It contains immutable revisions of:

| Record | Meaning and required content |
|---|---|
| `RoutingAlternative` | Stable ID, set/revision IDs, label, origin, parent/derivation, applicability, lifecycle, and ordered operation references |
| `RouteAssumption` | Explicit claim about geometry, tolerance, process, availability, overlap, yield, or policy; source, confidence, and blocking state |
| `RouteFeasibilityResult` | `Eligible`, `ConditionallyEligible`, `Ineligible`, or `Unknown`, with evaluated rule/version and evidence |
| `RouteMetricSet` | Cost classes, price, time classes, lead time, uncertainty, risk, capacity demand, bottleneck demand, and confidence; unavailable values stay unavailable |
| `RoutingComparison` | Compared route revisions, objective/profile version, normalized display values, Pareto/dominance facts, reasons, filters, and user decision |
| `RouteAdoptionDecision` | Adopted route revision, actor, time, reason, comparison snapshot, exceptions, and approvals |

An operation retains the fields in [routing and operation model](routing-and-operation-model.md). A route also records stock form/dimensions, setup and datum strategy, workholding/fixture/soft-jaw assumptions, feature-to-operation assignments, tool families, programming/setup/prove-out/run/inspection effort, tooling/fixture lead time, scrap/rework, quality obligations, internal and external work, and resource intervals.

Do not materialize every record as an independent database table by default. Persist the aggregate header and query-critical facts normally; preserve a versioned route/comparison payload when needed for faithful replay.

## Origins and lifecycle

Origins are `Manual`, `TemplateDerived`, `SystemGenerated`, `Imported`, or `CopiedFromPriorRevision`. Origin never implies approval.

`DraftCandidate → Evaluated → Reviewed → Adopted` is the successful path. `Rejected`, `Withdrawn`, and `Superseded` retain evidence and decision reasons. Editing an evaluated, reviewed, or adopted route creates a new route revision. Re-analysis may add sibling candidates but cannot mutate or replace an adopted route.

Only one route revision is active for a given approved estimate revision and quantity scenario. Another alternative may be adopted only through an explicit replacement decision and estimate reapproval. Rejection records rule/user, reason, and whether the rejection is quote-specific or reusable evidence.

## Generation and screening

Generation is staged:

1. Early foundation stores user-created alternatives and copied/template routes.
2. Initial production supports one adopted route and manual comparisons without optimizer claims.
3. Intermediate releases generate bounded alternatives from approved templates and capability rules.
4. Advanced releases may search combinations of process, machine, stock, setup, and sourcing choices under explicit limits.
5. Machine-learning route generation remains research-only until it is explainable, governed, and independently validated.

Generation must be deterministic for the same version bundle and ordered inputs unless a recorded seed is part of the algorithm. It deduplicates materially equivalent routes and records why a combination was not generated.

Hard constraints screen safety, geometry/process applicability, machine envelope/capability, material/process restrictions, required approvals, quality/customer restrictions, and prohibited data/vendor boundaries. Missing evidence produces `Unknown` or `ConditionallyEligible`; it does not pass a hard constraint. Soft objectives may rank only candidates that survive the selected filter policy.

## Comparison and preference

The canonical comparison is a vector, not an unexplained universal score:

- accounting, incremental, fully burdened, and risk-adjusted cost;
- selling price and contribution margin;
- touch, occupancy, bottleneck, and calendar time;
- delivery feasibility and schedule risk;
- expected scrap/rework, technical/quality risk, and confidence;
- current readiness, tooling/fixture reuse, and prototype/repeat suitability.

The UI may offer named views such as lowest internal cost, fastest delivery, lowest risk, highest confidence, lowest bottleneck use, or highest contribution per bottleneck hour. Each view identifies its policy version, excluded/unavailable dimensions, hard filters, weights if used, and the leading reasons. Ties remain ties. A Pareto frontier may identify non-dominated choices without selecting a winner.

Capacity and opportunity-cost outputs are advisory dimensions from [capacity and opportunity cost](capacity-and-opportunity-cost.md). They never overwrite the route's accounting cost. Uncertainty summaries come from [probabilistic estimation](probabilistic-estimation.md) and never turn an infeasible route into a feasible one.

## Missing and stale data

- A route remains editable when optional tooling, calendar, vendor, or uncertainty data is missing.
- A missing hard-feasibility input is visible and may block adoption under shop policy.
- Missing metrics display `Unavailable` with reason; they are not zero, worst-case, or silently excluded from a composite.
- Stale capability, price, availability, calendar, and vendor data retain their as-of time and freshness result.
- When current schedule data is absent, compare baseline technical/economic routes and label capacity, opportunity cost, and delivery feasibility `Not evaluated`.

## Versioning, audit, retention, and security

Each evaluation pins source/analysis, feature/setup, route-generator, capability, tool, feed/speed, runtime, calculation, rate, pricing, capacity, uncertainty, availability, and requirement snapshots. Approved estimate and quote snapshots retain the exact adopted route and rejected alternatives used for the decision according to retention policy; archival may compact derived search traces only under a documented, verifiable policy.

Manual changes preserve old/new values, actor, time, reason, and authorization. Route comparison access follows project, classification, and commercial permissions. Vendor/export evaluations must not transmit models, drawings, requirements, prices, or route content outside the approved boundary. Logs contain IDs, versions, timings, and safe result classes—not part geometry or commercial values.

## Human approval and release gates

Adoption requires the estimator to see feasibility unknowns, materially different alternatives, requirement coverage, capacity freshness, uncertainty drivers, and overrides. Policy may require manufacturing, quality, purchasing, or commercial concurrence. The optimizer never issues a quote, reserves capacity, places an order, or changes an approved route.

Release gates are route-corpus expert review, deterministic ranking, preserved override/adoption history, missing-data tests, no accounting/opportunity-cost conflation, bounded compute/cancellation, and usable comparison behavior under [routing comparison UX](../05-ux/routing-comparison.md).
