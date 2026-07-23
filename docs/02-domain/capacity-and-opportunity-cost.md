# Capacity and Opportunity Cost

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-043–REQ-F-046, REQ-F-065; REQ-NF-015–REQ-NF-018, REQ-NF-021–REQ-NF-022; CALC-021–CALC-026, CALC-031, CALC-033; TEST-045–TEST-050  
> **Related ADRs:** ADR-0007, ADR-0010  
> **Open questions:** Bottleneck designations, schedule source/freshness, contribution policy, and reservation authority  
> **Dependencies:** Routing alternatives, rate policy, calendars/backlog, availability, pricing  
> **Supersedes:** None

## Purpose and boundary

Capacity analysis asks whether a proposed route fits the shop's stated resources and whether scarce capacity has a more valuable competing use. It augments an estimate; it is not finite-capacity production scheduling, dispatch, ERP/MES replacement, or a promise of delivery.

Capacity-aware calculations never silently alter accounting cost, estimate cost, or selling price. Any price or route change based on capacity is an explicit, traceable human decision.

## Time and resource vocabulary

| Term | Definition |
|---|---|
| Touch time | Human effort actively attending the work, possibly overlapping machine occupancy |
| Machine occupancy | Time a machine cannot accept incompatible work, including attended and unattended intervals |
| Operator time | Labor demand by skill/resource pool; not assumed equal to occupancy |
| Setup time | Preparation demand with machine, operator, fixture, and possible programmer/quality resources represented separately |
| Programming time | Programmer demand, whether before or overlapping other route intervals |
| Inspection time | Operator or quality/CMM demand; in-cycle inspection remains classified under CALC-010 |
| Queue time | Waiting between readiness and resource start; not a costed activity unless a named policy says otherwise |
| Calendar lead time | Critical-path elapsed duration using calendars, dependencies, queues, and supplier/material lead times |
| Bottleneck time | Demand on a resource designated or calculated as constraining for the evaluated horizon |

Every operation declares resource demands and compatibility, duration basis, quantity/lot basis, precedence, overlap, attended/unattended state, and uncertainty. Parallel work is represented as intervals and dependencies, never by adding all durations. A demand may require several resources simultaneously.

## Owned records and lifecycle

The shop-library/capacity boundary owns effective-dated `CapacityCalendar`, `ResourcePool`, `BottleneckDesignation`, overtime/weekend/unattended policy, and data-source/freshness policy. The capacity engine owns immutable `CapacitySnapshot`, `CapacityEvaluation`, `DeliveryFeasibilityResult`, and `OpportunityCostAnalysis`. The estimate owns references to evaluated snapshots and any adoption/override decision.

A snapshot records horizon, timezone, shifts, closures, maintenance, planned downtime, backlog/reservations, resource availability, vendor/material/tooling lead-time observations, data as-of times, source versions, and completeness. Lifecycle is `Draft → Reviewed → Published → Superseded`; published snapshots do not change in place. A new import or calendar edit creates a successor.

A quote evaluation is immutable once attached to an approved estimate. Re-evaluation against a newer snapshot creates a comparison; it does not rewrite the historical delivery or profitability evidence.

## Distinct economic values

| Value | Meaning |
|---|---|
| Accounting cost | Cost recognized under the shop's approved accounting/rate policy |
| Incremental cost | Additional cash/resource cost caused by accepting this work over the chosen baseline |
| Fully burdened cost | Allocated direct and indirect cost under the approved burden policy |
| Opportunity cost | Estimated contribution foregone from the best credible displaced use of a constrained resource in the stated horizon |
| Risk-adjusted cost | Internal cost plus explicitly modeled accepted risk/rework treatment; never a hidden blend |
| Selling price | Commercial decision produced by the pricing policy and approved overrides |

Opportunity cost is a decision metric, not an accounting transaction and not a universal surcharge. It remains zero only when the model establishes no displacement under the selected policy; absent competing-demand evidence it is `Unavailable`, not zero.

## Calculation semantics

All metrics identify currency, quantity scenario, time basis, horizon, snapshot, and policy version.

- `contribution_margin = selling_price - policy_defined_incremental_cost`.
- `contribution_per_spindle_hour = contribution_margin / constrained spindle cutting hours`.
- `contribution_per_machine_occupancy_hour = contribution_margin / constrained machine occupancy hours`.
- `contribution_per_bottleneck_hour = contribution_margin / demand hours on the named bottleneck`.
- `revenue_per_bottleneck_hour = selling_price / named bottleneck hours`.
- `gross_profit_per_bottleneck_hour = policy_defined_gross_profit / named bottleneck hours`.
- `opportunity_cost = max(credible contribution of displaced alternative - contribution retained after feasible rearrangement, 0)` for the declared comparison set and horizon.

Undefined denominators return `Unavailable`. Metrics with different denominators must not be relabeled or compared as equivalent. Setup, programming, inspection, supplier, and machine bottlenecks use their own resource hours; “spindle hour” applies only when spindle use is the constrained basis.

Delivery feasibility is `Feasible`, `FeasibleWithConditions`, `Infeasible`, or `Unknown`, with earliest/latest modeled completion, required overtime, critical path, limiting resources, slack, and assumptions. Schedule risk is separate from feasibility and includes freshness, utilization, dependency, and uncertainty evidence. A capacity-adjusted quote score is explainable policy output, not price and not approval.

## Bottlenecks and horizons

The shop may designate strategic/long-term bottlenecks and current/horizon-specific bottlenecks. A calculated bottleneck is a proposal requiring review. Examples include five-axis or Swiss machines, senior programming, CMM, grinding, assembly, and outside heat treatment. Each designation records resource, horizon, rationale, utilization/queue evidence, owner, status, and version.

Do not optimize one bottleneck by hiding overload on another. Results list every material constraint and sensitivity to alternative bottleneck designations.

## Missing, stale, and conflicting data

The engine always preserves a baseline route estimate independent of live capacity.

- No calendar/backlog: capacity and opportunity cost are `NotEvaluated`; theoretical capability and baseline cost remain visible.
- Partial resource data: calculate only supported dimensions and label coverage; delivery stays `Unknown` when the critical path cannot be established.
- Stale data: retain result with as-of timestamp and `Stale` warning or block approval per policy.
- Conflicting reservations/calendars: do not pick silently; return a conflict with source records.
- Unknown overtime/weekend/unattended policy: do not assume availability.
- Missing competing jobs or contribution values: opportunity cost is unavailable; do not substitute the workcenter rate.

Users may enter an explicit planning scenario, but it is `Manual`, pins its assumptions, and does not masquerade as current shop state.

## Approval, security, and retention

Only authorized schedule owners publish capacity snapshots or reservations; estimate users may run non-mutating what-if evaluations. Adopting a capacity-influenced route, delivery commitment, overtime assumption, or price adjustment records the original decision, result, actor, time, reason, and required approvals.

Capacity data may reveal customers, jobs, staffing, downtime, prices, and strategic constraints. Apply project/role/classification policy; expose only aggregated competing demand where detailed access is not authorized. No public network or third-party scheduler is required. Diagnostic output excludes job names, employees, prices, and schedule content.

Retain the exact snapshot and policy/result digests used for approved estimates. Reservation synchronization or schedule export is deferred behind an integration contract, authorization, audit, and conflict policy.

## Release placement

- Early foundation: resource/time types, snapshot/version references, manual bottleneck designation, and `NotEvaluated` states.
- Initial production: baseline route economics and manual availability/lead-time facts; no live opportunity-cost claim.
- Intermediate: imported/manual calendar and backlog snapshots, coarse delivery feasibility, basic bottleneck metrics, and explicit what-if comparisons.
- Advanced: governed opportunity-cost scenarios, multi-resource capacity scoring, and availability-aware alternative ranking.
- Research: optimization-grade scheduling, probabilistic queues, and automatic reservation/dispatch.
