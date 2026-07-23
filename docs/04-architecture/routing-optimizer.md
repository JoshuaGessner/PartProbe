# Routing Optimizer Architecture

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-006, REQ-F-027, REQ-F-040–REQ-F-042, REQ-F-065; REQ-NF-004, REQ-NF-015–REQ-NF-018, REQ-NF-021–REQ-NF-022  
> **Related ADRs:** ADR-0007–ADR-0009  
> **Open questions:** Search budget, shop feasibility rule ownership, and approved scoring profiles  
> **Dependencies:** Domain routing model, geometry/setup/runtime engines, shop libraries, capacity and uncertainty ports  
> **Supersedes:** None

## Boundary and ownership

The routing optimizer is a UI-independent, headlessly testable application/domain service. It produces versioned route candidates and comparisons; it does not mutate estimates, approve routes, reserve resources, call vendors, generate G-code, or promise manufacturability.

The proposed workspace boundary is `crates/routing-optimizer` or a clearly isolated module adjacent to `setup-planner`. It owns candidate generation orchestration, feasibility evaluation, comparison vectors, scoring profiles, dominance analysis, and explanation traces. It consumes immutable DTOs and ports; it does not import UI, SQL, native kernel, or deployment types.

```text
part/requirement evidence + shop snapshot + quantity scenario
                  │
        route strategy providers
                  ↓
   candidate normalization/deduplication
                  ↓
   hard feasibility rules → rejected/unknown evidence
                  ↓
 runtime + cost + risk + capacity + uncertainty evaluators
                  ↓
 comparison vector / Pareto facts / named profile explanations
                  ↓
            human adoption use case
```

## Inputs and ports

| Port | Consumes | Constraint |
|---|---|---|
| `RouteStrategyProvider` | Geometry/features, requirements, templates, process knowledge | Returns proposals and provenance; never raw kernel objects |
| `CapabilityEvaluator` | Machine/process/material/workholding/tool requirements | Separates `Unknown` from `Eligible` |
| `RuntimeEstimator` | Ordered operations and selected method | Returns TIME-labeled result/trace |
| `CostEvaluator` | Route, quantity, rate/library snapshots | Uses typed calculation graph and fixed-decimal currency |
| `RiskEvaluator` | Evidence and assumptions | Keeps risk ledger explicit |
| `CapacityEvaluator` | Resource-demand plan plus optional capacity snapshot | May return `NotEvaluated`; cannot alter accounting cost |
| `UncertaintyEvaluator` | Deterministic graph plus approved uncertain inputs | Optional; deterministic route remains available |
| `RouteRepository` | Alternative-set revisions and decisions | Append-preserving approved history |

Strategy providers can cover manual/template routes, 3/4/5-axis approaches, turning/live-tool/mill-turn/Swiss, manual/blanking/EDM/grinding, near-net inputs, and partial/full outsourcing. Unsupported strategies return explicit non-applicability; plugin failure cannot erase other candidates.

## Pipeline semantics

1. Validate source snapshot IDs, quantity, requirements, and shop-library compatibility.
2. Ask enabled/versioned strategies for candidates within declared process and search limits.
3. Normalize ordering/units and deduplicate by a canonical material-route signature while retaining provenance.
4. Evaluate hard constraints independently and retain rejection/unknown reasons.
5. Evaluate time, cost, risk, capacity, and uncertainty only where prerequisites allow.
6. Produce a comparison vector and non-dominated set.
7. Apply an optional named scoring profile for ordering, retaining raw metrics, excluded dimensions, normalization, weights, tie-breaks, and reasons.
8. Return a proposal set. A separate authorized application command records adoption.

Scores are not stored as intrinsic route facts. They are results of `(route metrics, scoring profile version, comparison population, policy context)`. Reordering or filtering must not rewrite route evidence.

## Determinism, limits, and failures

The same canonical inputs and version bundle produce the same candidate ordering and evaluation digests. Any heuristic randomness pins algorithm and seed. Tie-breaking uses stable IDs only after material comparison dimensions tie.

Execution enforces candidate count, branching depth, wall/CPU/memory, per-provider, and output-size limits. Cancellation returns completed candidate evidence with an `Incomplete` run status; partial output cannot be called exhaustive. Provider errors are isolated and sanitized. Combinatorial search uses staged expansion and dominance pruning only where the pruning rule is versioned and cannot remove a candidate needed to explain a hard constraint.

Missing schedule, price, tool, or uncertainty evidence yields partial comparison dimensions. A scoring profile declares whether an unavailable dimension blocks ranking or is shown outside the ranking; it never silently assigns zero or an average.

## Versions, persistence, and immutability

`RoutingRunManifest` pins input digests; geometry/feature/setup/runtime/calculation/capability/library/rate/pricing/capacity/uncertainty versions; enabled providers; search/configuration versions; seed; resource limits; and output schema. Candidate and comparison revisions are immutable once reviewed or adopted. Approved estimates retain route revision, run manifest, comparison/adoption decision, and required evidence digests.

Repository adapters persist aggregates defined in [routing alternatives](../02-domain/routing-alternatives.md). Re-running creates a sibling run and explicit diff. Schema or algorithm upgrades cannot recalculate historical candidates in place.

## Explainability and security

Each candidate reports how it was generated, applicable/failed/unknown rules, primary metric differences, dominant risks, unavailable evidence, and why a named profile ranked it. An explanation uses structured reason codes plus user-facing text; it cannot be only a scalar score.

All computation is local/in-boundary by default. Ports accept value DTOs, not filesystem paths or unrestricted handles. Strategy providers and future plugins receive the minimum classified evidence needed and have no network capability by default. Logs exclude geometry, requirements text, customer identity, prices, route notes, and file paths.

## Staged delivery and acceptance

Foundation defines contracts and manual alternatives. Initial production persists one adopted route and shows manual comparison. Intermediate delivery adds approved template strategies and deterministic multi-route comparison. Automated combinatorial optimization, advanced sourcing, capacity-adjusted scoring, and learned strategies remain advanced/research.

Acceptance requires expert-reviewed route fixtures; determinism; correct hard/unknown screening; stable adoption/override history; ranking explanations; missing-data behavior; bounded cancellation; cross-platform replay; authorization tests; and proof that no optimizer action changes an approved estimate without human adoption.
