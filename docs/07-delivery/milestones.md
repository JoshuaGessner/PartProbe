# Milestones

> **Status:** In Review
> **Last updated:** 2026-08-09
> **Related requirements:** All
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** OQ-001–OQ-050
> **Dependencies:** Roadmap
> **Supersedes:** None

## M0.1 Planning baseline

Exit: navigable documentation, stable requirement IDs, research/ADRs ready for review, domain/calculation/data models, examples, fixture strategy, quality plan, advanced planning enhancement, risks, open questions, and exact next task. Status: **Baseline established; review remains open**.

## M0.2 Evidence and decision closure

Exit: interviews complete; private fixture/data governance agreed; UI, geometry, worker, persistence, and calculation spikes measured; TASK-002 synthetic calculation mechanics plus TASK-007 real rate-category/pricing-policy calibration reviewed; ADR-0001–0014 accepted/rejected or explicitly deferred; route/capacity/uncertainty/PMI/CAM/availability data sources assessed; baseline accuracy and performance budgets set.

Status: **In Progress.** Calculation/rate mechanics, a substantial worker/geometry spike, and GUI-3's bounded Apple-Silicon Tauri/Leptos shell/security subset now have executable evidence. Interviews, shop calibration, the remaining UI/accessibility/viewer/PDF spike, persistence spike, representative private-fixture governance, ADR closure, supported-platform native packaging, and accuracy/performance budgets remain open.

## M1 Foundation

Exit: Cargo workspace; domain/units/money/provenance; migrations/repositories; worker protocol; initial import/measurement corpus; application shell/design system; cross-platform CI; future-safe identifiers/snapshots for route sets, requirements, revisions, availability, and correction events without implementing advanced engines.

Status: **In Progress.** TASK-001 supplies the calculation workspace/primitives and passing cross-platform CI evidence. TASK-002 supplies configurable rate, rounding, pricing, synthetic-golden, and replay mechanics with passing three-OS evidence. TASK-003 supplies kernel-neutral geometry contracts, an isolated control-schema-v2 worker, explicit verified-copy transport, exact Unix descriptor and Windows HANDLE direct-resource allowlisting with unrelated-resource exclusion, worker-side revalidation, controlled intake/output seams, cancellation behavior, and provisional Apple Silicon OCCT ABI-v3 byte-stream evidence. GUI-3 supplies an unsigned local desktop shell, native picker, typed pathless bridge, and initial design/security boundary. Production importer accuracy/containment/packaging, connected estimate UI, full accessibility, persistence, mesh support, and remaining foundation components are pending.

## M2 Vertical slice

Exit: all items in `release-plan.md` and `functional-requirements.md` for initial slice pass linked acceptance evidence on supported targets.

The M2 scope remains one approved route and coarse runtime. It adds guided shop-owned rate/pricing setup, selected-rate traceability, and only basic manual requirement coverage, availability, make/buy, and revision preservation needed to avoid redesign; automatic optimization, probabilistic simulation, structured PMI authority, live CAM/schedule integrations, and learning recommendations are excluded.

## M3 Decision-support expansion

Exit: multiple reviewed routing alternatives, basic capacity feasibility, revision comparison, feature-level cost/risk visualization, correction capture, and manual/CAM-report reconciliation pass TEST-040–084 as applicable.

## M7 Advanced optimization evidence

Exit: uncertainty method, bounded optimizer, opportunity-cost policy, structured PMI scope, controlled learning, and capacity-adjusted scoring pass accepted ADR gates and representative shop validation before production enablement.
