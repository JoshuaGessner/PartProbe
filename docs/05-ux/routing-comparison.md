# Routing Comparison Workflow

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-027, REQ-F-040–REQ-F-048, REQ-F-065; REQ-NF-021–REQ-NF-022; UX-001–UX-010, UX-021, UX-026; TEST-040–TEST-055  
> **Related ADRs:** ADR-0001, ADR-0009–ADR-0011  
> **Open questions:** Default comparison columns, approval thresholds, and shop-preferred named views  
> **Dependencies:** Design system, routing/capacity/uncertainty domain models, permissions policy  
> **Supersedes:** None

## Experience goal

Help an estimator answer three questions without losing the part or quote context:

1. What credible ways could we make or source this part?
2. Why does each route differ in feasibility, cost, time, capacity, risk, and uncertainty?
3. Which route am I deliberately adopting, and what remains unresolved?

The screen is a decision workspace, not a leaderboard. It never labels the first row “the answer,” and no route is adopted merely because a sort or scoring profile changes.

## Entry and layered disclosure

The Routing tab exposes `Compare alternatives` once two candidates exist. Creating a manual alternative is available before automatic generation. The quote workspace keeps the model/part context and total strip visible.

| Layer | Default content |
|---|---|
| Normal estimating | Adopted/current route, most-likely baseline, main warnings, cost breakdown, and `N alternatives` link |
| Advanced analysis | Comparison table, feasibility evidence, uncertainty range, sensitivity, capacity/bottleneck impact, readiness, and scenario controls |
| Approval | Adopted route, price/margin, opportunity cost separately, delivery feasibility, coverage, unresolved assumptions, overrides, and required concurrence |

Advanced content is available but not expanded by default. User preference may remember layout, never calculation/approval state.

## Comparison workspace

```text
┌ Routing alternatives · Part 28-441 · Qty 25 ─ [New manual] [Generate] ─────┐
│ Scenario: Baseline shop state (as of …)  View: Balanced (policy v…)         │
│ Capacity: Partial coverage  | Uncertainty: 7/9 modeled  | Requirements: …   │
├──────────────┬──────────┬──────────┬────────────┬──────────┬───────────────┤
│ Alternative  │ Feasible │ Internal │ Delivery   │ Bottleneck│ Risk / range  │
│ 3-axis / 3SU │ Cond. !  │ $…       │ P50… P90…  │ 6.2 h 5AX│ Med · $…–$…  │
│ 5-axis / 2SU │ Yes      │ $…       │ P50… P90…  │ 9.7 h 5AX│ Low · $…–$…  │
│ Outsource op │ Unknown  │ $…       │ vendor data stale    │ High · not modeled│
├──────────────┴──────────┴──────────┴────────────┴──────────┴───────────────┤
│ Why different: [Operations] [Assumptions] [Capacity] [Uncertainty] [Trace] │
│ Selected route: 2 fewer setups; +3.5 bottleneck h; lower workholding risk   │
│ [Reject…] [Edit as new revision]                     [Adopt for estimate…] │
└─────────────────────────────────────────────────────────────────────────────┘
```

The table pins identity and feasibility. Users choose columns from accounting/incremental/fully burdened/risk-adjusted cost, selling price, contribution, touch/occupancy/bottleneck hours, lead time, readiness, quality/technical/schedule risk, confidence, and uncertainty. Cost classes and time bases never share ambiguous labels.

Cells show a value plus state: current/stale/not evaluated/unavailable/blocked/overridden. Hover may summarize, but keyboard focus and the inspector expose the same explanation. Sorting unavailable values groups them separately; it does not treat them as zero.

## Named views and score explanation

Named views include lowest accounting cost, fastest modeled delivery, lowest risk, highest confidence, least current bottleneck usage, prototype, repeat production, and approved shop scoring profiles. A view changes presentation/ranking only.

When a scoring profile is used, `Why ranked here` shows hard filters, dimensions, raw values, normalization, weights, exclusions, unavailable-data treatment, tie-breaks, policy/version, and the top positive/negative drivers. Show “No clear leader” for ties or materially non-dominated alternatives. Users can restore the unranked comparison.

## Detail and synchronized evidence

Selecting a route opens a structured difference inspector:

- stock, process/machine, setup/orientation/datum/workholding, operations/features/tools;
- programming/setup/prove-out/cutting/non-cutting/inspection/outside effort;
- quality requirements and unresolved coverage;
- accounting cost and price trace;
- capacity demand and bottleneck explanation;
- low/likely/high or percentile output with dominant uncertainty inputs;
- risks, feasibility failures/unknowns, source/provenance, versions, and overrides.

Selecting an operation or feature synchronizes with model, setup, tool, cost, warning, and requirement evidence under UX-021. Where feature-level allocation is coarse, label it approximate. A route with mesh-only evidence retains the representation confidence ceiling.

## Capacity and uncertainty interaction

Capacity controls choose an immutable named snapshot/scenario and show its as-of time, coverage, horizon, and policy. `No current schedule` leaves baseline economics intact and displays capacity/opportunity cost/delivery as `Not evaluated`. What-if changes are visibly manual and cannot reserve shop capacity.

The normal view shows most likely plus a compact range/warning. Expanding reveals minimum/likely/expected/quantiles only where defined, method, source, dominant drivers, and deterministic baseline. Use “P90 modeled cost” or “P90 modeled completion,” never “90% confident.” Confidence remains a separate evidence label.

## Adoption, override, and approval

`Adopt for estimate` opens a review dialog naming the current and proposed route revisions, material cost/time/capacity/risk changes, feasibility unknowns, stale/missing data, requirement blockers, and approvals. The user supplies a reason when adopting a route other than the policy leader, accepting a conditional/unknown route, or replacing a reviewed/adopted route. Policy determines required manufacturing, quality, purchasing, or commercial concurrence.

Adoption creates a decision and estimate revision; it never edits the prior route. Rejecting a candidate records a reason and preserves it. Regeneration displays a new comparison and never changes the adopted route automatically.

## Accessibility and expert flow

- The comparison is an accessible keyboard grid with stable row/column identity, announced values/units/states, pinned active cell, column chooser, and compact/comfortable density.
- Arrow keys move cells; Enter opens the inspector; Space selects; an explicit command opens adoption. Sorting cannot trigger adoption.
- Every chart has a data table; every color has text/icon/shape. Cost ranges and Pareto plots are optional complements, not the sole representation.
- Focus remains on the edited route after recalculation and announces material changes without reading the entire grid.
- Compare up to a tested visible limit; search/filter and selected-route pinning support larger sets. Hidden rows remain counted.
- Use tabular numerals, explicit units/currency/quantity basis, and display precision justified by inputs.

## Failure, security, and empty states

Generation shows stage, candidate count, elapsed time, limits, cancellation, and completed evidence. Partial/cancelled output says it is not exhaustive. An engine failure preserves manual routes and the adopted route, names the failed dimension/provider, and offers retry or manual comparison.

Unauthorized commercial or competing-job detail is summarized without leaking identities or values. Export, print, copy, and external integration follow classification policy and preview scope. There is no automatic network call or marketplace benchmark. Diagnostic IDs are safe and content-free.

## Usability acceptance

Representative estimators must be able to create a manual alternative, compare routes, identify why rankings differ, distinguish accounting from opportunity cost, interpret not-evaluated capacity, distinguish most-likely/P90/confidence, adopt a non-leading route with reason, and verify the prior approved route remains intact. Tests include keyboard-only and supported assistive-technology workflows.
