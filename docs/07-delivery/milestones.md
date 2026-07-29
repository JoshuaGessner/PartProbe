# Milestones

> **Status:** In Review
> **Last updated:** 2026-07-29
> **Related requirements:** All
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** OQ-001–OQ-050
> **Dependencies:** Roadmap
> **Supersedes:** None

## M0.1 Planning baseline — current

Exit: navigable documentation, stable requirement IDs, research/ADRs ready for review, domain/calculation/data models, examples, fixture strategy, quality plan, advanced planning enhancement, risks, open questions, and exact next task. Status: **In Review**.

## M0.2 Evidence and decision closure

Exit: interviews complete; private fixture/data governance agreed; UI, geometry, worker, persistence, and calculation spikes measured; TASK-002 synthetic calculation mechanics plus TASK-007 real rate-category/pricing-policy calibration reviewed; ADR-0001–0014 accepted/rejected or explicitly deferred; route/capacity/uncertainty/PMI/CAM/availability data sources assessed; baseline accuracy and performance budgets set.

## M1 Foundation

Exit: Cargo workspace; domain/units/money/provenance; migrations/repositories; worker protocol; initial import/measurement corpus; application shell/design system; cross-platform CI; future-safe identifiers/snapshots for route sets, requirements, revisions, availability, and correction events without implementing advanced engines.

Status: **In Progress.** TASK-001 supplies the calculation workspace/primitives and passing cross-platform CI evidence. TASK-002 supplies locally passing configurable rate, rounding, pricing, synthetic-golden, and replay mechanics; cross-platform TASK-002 evidence and the remaining foundation components are pending.

## M2 Vertical slice

Exit: all items in `release-plan.md` and `functional-requirements.md` for initial slice pass linked acceptance evidence on supported targets.

The M2 scope remains one approved route and coarse runtime. It adds guided shop-owned rate/pricing setup, selected-rate traceability, and only basic manual requirement coverage, availability, make/buy, and revision preservation needed to avoid redesign; automatic optimization, probabilistic simulation, structured PMI authority, live CAM/schedule integrations, and learning recommendations are excluded.

## M3 Decision-support expansion

Exit: multiple reviewed routing alternatives, basic capacity feasibility, revision comparison, feature-level cost/risk visualization, correction capture, and manual/CAM-report reconciliation pass TEST-040–084 as applicable.

## M7 Advanced optimization evidence

Exit: uncertainty method, bounded optimizer, opportunity-cost policy, structured PMI scope, controlled learning, and capacity-adjusted scoring pass accepted ADR gates and representative shop validation before production enablement.
