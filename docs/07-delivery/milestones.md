# Milestones

> **Status:** In Review
> **Last updated:** 2026-09-11
> **Related requirements:** All
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** OQ-001–OQ-050
> **Dependencies:** Roadmap
> **Supersedes:** None

## M0.1 Planning baseline

Exit: navigable documentation, stable requirement IDs, research/ADRs ready for review, domain/calculation/data models, examples, fixture strategy, quality plan, advanced planning enhancement, risks, open questions, and exact next task. Status: **Baseline established; review remains open**.

## M0.2 Evidence and decision closure

Exit: interviews complete; private fixture/data governance agreed; UI, geometry, worker, persistence, and calculation spikes measured; TASK-002 synthetic calculation mechanics plus TASK-007 real rate-category/pricing-policy calibration reviewed; ADR-0001–0014 accepted/rejected or explicitly deferred; route/capacity/uncertainty/PMI/CAM/availability data sources assessed; baseline accuracy and performance budgets set.

Status: **In Progress.** Calculation/rate mechanics, a substantial worker/geometry spike, GUI-2 through GUI-5's bounded Apple-Silicon STEP-to-estimate path, and separate verified developer-runtime assembly on Apple Silicon, Ubuntu x86_64, and Windows x64 now have executable evidence. A documented Huntsville test profile supports repeatable usability testing but does not replace shop calibration. Interviews, shop-approved deployment inputs, the native model/stock viewer, remaining UI/accessibility/PDF work, persistence, representative private-fixture governance, ADR closure, supported-platform signed packaging, and accuracy/performance budgets remain open.

## M1 Foundation

Exit: Cargo workspace; domain/units/money/provenance; migrations/repositories; worker protocol; initial import/measurement corpus; application shell/design system; cross-platform CI; future-safe identifiers/snapshots for route sets, requirements, revisions, availability, and correction events without implementing advanced engines.

Status: **In Progress.** TASK-001 supplies the calculation workspace/primitives and passing cross-platform CI evidence. TASK-002 supplies configurable rate, rounding, pricing, synthetic-golden, and replay mechanics with passing three-OS evidence. TASK-003 supplies kernel-neutral geometry contracts, an isolated control-schema-v2 worker, explicit verified-copy transport, exact Unix descriptor and Windows HANDLE direct-resource allowlisting with unrelated-resource exclusion, worker-side revalidation, controlled intake/output seams, cancellation behavior, Linux parser-phase socket/process syscall denial, and provisional OCCT byte-stream evidence. ABI v4 adds precise uninflated source-axis AABB output and `geometry-step-analysis-v1`; a fresh Apple-Silicon construction/runtime and final contract-v10 arm64 package pass embedded-runtime, real-STEP, and proposal/adoption smokes, while verified Ubuntu-x86_64/Windows-x64 evidence remains at ABI v3. Ubuntu and Windows earlier pass internal dynamic-link/PE auditing and configured headless STEP/STL/3MF host smokes; Ubuntu 22.04 also passes unsigned extracted Debian package-resource verification, real STEP-to-USD-702 analysis, and bounded virtual-display launch. GUI-3 supplies an unsigned local desktop shell, native picker, typed pathless bridge, and initial design/security boundary; GUI-4/GUI-5 implement the connected session-only exact analysis/manual-estimate path; contract v10 adds exact bounds plus explicit model-sensitive starter-Draft proposal/adoption. Production importer accuracy, ABI-v4 Linux/Windows refresh, governed catalog adoption, detailed routing/runtime/adders, human-driven populated-flow accessibility, windowed mesh interaction, remaining containment, signed three-OS packaging, persistence, and remaining foundation components are pending.

Near-term M1/M2 work follows the [usable estimator delivery plan](usable-estimator-plan.md). USE-1 separates the primary upload/analyze Estimate view from Settings without changing calculation authority. USE-2 persistence and governance, USE-3 stock/material proposals, and USE-4 coarse process/runtime proposals are the shortest path from the current developer harness to a useful model-sensitive estimator; existing TASK-003/004 safety and format work continues alongside that path.

## M2 Vertical slice

Exit: all items in `release-plan.md` and `functional-requirements.md` for initial slice pass linked acceptance evidence on supported targets.

The M2 scope remains one reviewed proposed route and coarse runtime. It adds guided shop-owned Settings, exact-STEP stock/material and coarse runtime proposals, selected-version traceability, and only basic manual requirement coverage, availability, make/buy, and revision preservation needed to avoid redesign. Automatic optimization, probabilistic simulation, structured PMI authority, live CAM/schedule integrations, and learning recommendations are excluded.

## M3 Decision-support expansion

Exit: multiple reviewed routing alternatives, basic capacity feasibility, revision comparison, feature-level cost/risk visualization, correction capture, and manual/CAM-report reconciliation pass TEST-040–084 as applicable.

## M7 Advanced optimization evidence

Exit: uncertainty method, bounded optimizer, opportunity-cost policy, structured PMI scope, controlled learning, and capacity-adjusted scoring pass accepted ADR gates and representative shop validation before production enablement.
