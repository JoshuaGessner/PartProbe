# Capacity Engine Architecture

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-043–REQ-F-046, REQ-F-065; REQ-NF-003–REQ-NF-004, REQ-NF-015–REQ-NF-018, REQ-NF-021–REQ-NF-022; CALC-021–CALC-026, CALC-031, CALC-033  
> **Related ADRs:** ADR-0007, ADR-0010  
> **Open questions:** Calendar/backlog adapters, planning horizon, finite-capacity depth, and conflict authority  
> **Dependencies:** Routing demand model, shop calendars, availability sources, calculation engine  
> **Supersedes:** None

## Boundary and proposed module

The capacity engine evaluates immutable routing demand against an immutable shop-state scenario. It owns resource/time evaluation, delivery-feasibility evidence, bottleneck metrics, and opportunity-cost scenarios. It does not own rate accounting, price calculation, source system schedules, job dispatch, employee performance, automatic reservations, or ERP/MES truth.

Use a UI-independent `crates/capacity-engine` (or equivalent isolated package) behind application ports. The deterministic estimation engine remains authoritative for accounting and pricing nodes. Capacity results are typed advisory inputs that a pricing or approval policy may reference explicitly.

```text
route resource-demand graph ─┐
requirements/due date ───────┼→ scenario builder → evaluator → feasibility/metrics/trace
capacity snapshot ───────────┤                         │
policy/version bundle ───────┘                         └→ optional route comparison
```

## Contracts

- `ResourceDemandPlan`: precedence DAG of typed intervals, quantities/lots, required resource pools, simultaneous-resource constraints, overlap/parallel rules, calendars, and confidence.
- `CapacitySnapshot`: versioned resource calendars, reservations/backlog, downtime, staffing/skills, external/material/tool lead-time observations, freshness and coverage.
- `CapacityPolicy`: horizon/timezone, rounding, overtime/weekend/unattended rules, setup compatibility, queue assumptions, bottleneck definitions, freshness/blocking thresholds, and scenario mode.
- `CapacityEvaluation`: status, critical path, completion window, slack, overtime, utilization/demand by resource, limiting resources, bottleneck metrics, coverage, warnings, and trace.
- `OpportunityCostScenario`: candidate job, credible displaced alternatives/portfolio, contribution basis, rearrangement policy, horizon, and sensitivity results.

Application adapters may import calendars/backlog through versioned DTOs. Imported state remains a source snapshot; evaluation cannot write back. Reservation/export comes later through a separate audited command contract with optimistic concurrency and source-system ownership rules.

## Evaluation rules

The first useful evaluator is scenario-based and coarse. It validates the demand DAG, expands recurring quantities/lots, intersects simultaneous resource calendars, respects precedence and explicit overlap, and computes a transparent critical path. It may use reviewed queue allowances before a full finite-capacity placement algorithm exists.

Parallel work is represented by graph structure. Machine occupancy and operator touch are separate demands; unattended running does not free the machine. Setup requiring machine + operator + fixture reserves all stated resources for that interval. Outside processing uses lead-time and vendor-capacity evidence without fabricating an internal machine reservation.

Feasibility is four-state and always accompanied by coverage and assumptions. A calculated completion date is not a promise. Comparing alternative routes reuses the same snapshot and policy unless the UI explicitly compares scenarios.

Opportunity cost is evaluated only against a declared credible displaced-work set or approved shadow-value policy. It never derives from the accounting machine rate. Results identify the displaced alternative, contribution definition, scarce resource/hours, rearrangement considered, and sensitivity. If that evidence is absent, return `Unavailable`.

## Failure and missing-data semantics

No capacity snapshot returns a successful baseline estimate plus `CapacityStatus::NotEvaluated`. Partial coverage returns supported resource results and `Unknown` delivery if missing evidence can affect the critical path. Stale input is retained with as-of time and warning/block status. Ambiguous timezone, conflicting calendars/reservations, invalid cycles, negative duration/capacity, or incompatible unit/basis returns a typed error; the engine never resolves them silently.

An overload can produce `Infeasible` or `FeasibleWithConditions` only under a named overtime/reallocation policy. No automatic overtime, weekend, split-lot, outsourcing, or priority assumption is permitted.

## Reproducibility, immutability, and performance

An evaluation manifest pins route revision and demand digest, capacity snapshot, policy, calculation, pricing/contribution policy, algorithm/schema, timezone database/version where material, and optional scenario seed. Canonical ordering and fixed tie-breaking make identical inputs reproducible. Published results are immutable; new shop state creates a successor evaluation and diff.

Resource and horizon limits bound evaluation. Long runs support cancellation and return only explicitly incomplete diagnostics. Advanced search remains off the first-slice critical path. Cache keys include all manifest inputs; caches are derived and never the historical authority.

## Authorization, privacy, and observability

Adapters enforce role, project, classification, and resource scope. A user may receive an aggregate `resource unavailable` result without seeing another customer's identity, job value, employee schedule, or controlled part details. What-if evaluation cannot create reservations. Publishing a capacity snapshot or exporting a proposed reservation is separately authorized and audited.

Logs contain correlation ID, resource type, count buckets, algorithm/version, duration, status, and sanitized reason codes. They exclude customer/job names, employee identities, exact schedule contents, prices, and model/route content. No public network is required.

## Staged delivery and acceptance

- Foundation: contracts, time/resource taxonomy, immutable manual snapshots, missing states.
- Initial production: manual availability/lead-time flags outside live capacity evaluation.
- Intermediate: snapshot import/manual entry, transparent coarse feasibility, bottleneck demand and per-hour metrics.
- Advanced: alternative comparisons, governed opportunity cost, uncertainty integration, and richer multi-resource placement.
- Research: scheduling optimization and transactional reservation integration.

Acceptance covers overlapping/simultaneous resources, maintenance and shifts, overtime conditions, critical path, current/strategic bottlenecks, missing/stale/conflicting data, opportunity-cost separation, deterministic replay, access-limited explanations, and preservation of approved historical evaluations.
