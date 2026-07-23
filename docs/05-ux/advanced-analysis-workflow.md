# Advanced Analysis Workflow

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-040–REQ-F-065; UX-028–UX-045  
> **Related ADRs:** ADR-0009–ADR-0014  
> **Open questions:** OQ-031–OQ-050  
> **Dependencies:** routing comparison, requirement coverage, revision comparison, estimation baseline  
> **Supersedes:** None

## Interaction contract

Advanced analysis is a set of optional lenses over a pinned deterministic baseline. The baseline total, quote revision, route, freshness, blockers, and version manifest remain visible while a lens is open. A result is labeled `Proposal`, `Scenario`, `Advisory`, `Unavailable`, `Stale`, or `Adopted`; calculation output alone never grants authority.

```text
┌ Analysis ─ Baseline $… / …h ─ Route R1 ─ Versions … ─ Blockers 2 ─────────┐
│ Routes | Availability | Uncertainty | Capacity | Sourcing | Bid priority    │
├───────────────────────────────┬──────────────────────────────────────────────┤
│ Comparison                    │ Explanation                                  │
│ baseline always first         │ assumptions / exclusions / freshness         │
│ alternatives or scenarios     │ cost bases / sensitivity / provenance        │
│ filters do not hide blockers  │ warning and unknown states                   │
├───────────────────────────────┴──────────────────────────────────────────────┤
│ [Retain baseline] [Save scenario] [Adopt selected…]  human + reason required │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Progressive disclosure

Level 1 shows the baseline, decision question, blockers, and a small comparison vector. Level 2 expands components, freshness, assumptions, and confidence/uncertainty distinctions. Level 3 exposes calculation traces, distributions, capacity snapshots, mappings, rule versions, and audit events. Expert density is supported with saved column views, but hidden columns never suppress a blocking warning from the summary.

## Cross-linked evidence

- Route rows link to operation/setup/feature evidence and explain infeasibility before score.
- Availability adjusts readiness without changing theoretical capability.
- Capacity and economics show accounting, contribution, opportunity, risk, and price in separately labeled groups.
- Uncertainty shows input ranges, method, percentiles, sensitivity, and deterministic baseline; confidence is not a percentile.
- Revision and requirement views link geometry, PMI/drawing evidence, coverage state, manufacturing consequence, and named cost basis.
- Sourcing shows eligibility/constraints before landed-cost ranking and never initiates an external upload by default.
- Bid priority shows blockers before components and records the accountable human decision.

## Adoption and history

Adoption opens a diff summarizing changed route/inputs, affected costs, unresolved evidence, and pinned versions. It requires actor, time, reason, and target draft. Published/approved estimates cannot be targeted. Rule suggestions use a separate governance workflow and cannot be activated here. Historical views render the saved snapshot with a banner when a newer method exists; they do not silently recompute.

## Accessibility and failure states

All charts have equivalent tables; all overlays have a list/inspector path; color is redundant; focus order follows comparison then explanation then actions. Loading, partial, timeout, stale, forbidden, malformed-import, and not-evaluated states have distinct messages and recovery actions. Unknown values display `Unknown` or `Unavailable`, never blank or zero.

## Usability gates

Tests must show that estimators can distinguish baseline from proposal, confidence from uncertainty, accounting cost from opportunity cost, and theoretical capability from readiness; locate why a route was excluded; identify stale inputs; trace a revision delta; and retain the baseline without accidental adoption.
