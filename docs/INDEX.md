# PartProbe Documentation Index

> **Status:** In Review
> **Last updated:** 2026-08-16
> **Related requirements:** All
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** OQ-001–OQ-050
> **Dependencies:** Shop review and M0.2 technical spikes
> **Supersedes:** None

PartProbe is a planned local-first, cross-platform CAD-assisted machine-shop estimator. It turns models and explicit shop/customer inputs into a confidence-rated, editable routing and transparent cost/price foundation. It is not production CAM, an autonomous quote generator, or a compliance certification system.

## Current state

- **Phase:** 0 — Discovery and Validation
- **Milestone:** M0.2 — Evidence and decision closure, In Progress
- **Overall status:** Pre-alpha engineering foundation. TASK-001 and TASK-002 calculation/rate mechanics pass local and three-OS CI; TASK-003 has cross-platform worker contracts, partial OS containment including poll-bounded aggregate private-workspace output monitoring and Linux `x86_64`/`aarch64` parser-phase socket/process syscall denial, provisional OCCT byte-stream measurements, one manually authored analytic STEP fixture, and host-verified developer-runtime evidence on Apple Silicon, Ubuntu x86_64, and Windows x64. GUI-2 through GUI-5 establish one bounded Apple-Silicon internal developer path from native selection and real OCCT analysis through revision-bound review, complete inputs, cancellation/recovery, and deterministic USD 702 trace; Ubuntu and Windows pass equivalent configured native-host smokes, and Ubuntu also passes an unsigned extracted Debian package with verified native runtime, real retained-session STEP analysis, and bounded launch while exact-payload picker/analysis interaction is under final validation. Durable repositories, signed/distributable three-OS packaging, a supported production importer, calibrated deployment, and a release package do not exist
- **Recently completed:** Linux native-package run 31624958771 constructs exact OCCT 8.0.0 on Ubuntu 22.04 x86_64, materializes and verifies a package-safe 72-artifact runtime, builds and privately extracts the Debian package, repeats manifest and 24-binary/23-family dynamic-link verification, passes the packaged-resource STEP-to-USD-702 smoke, and keeps the exact packaged executable alive through the bounded virtual-display window. The workflow uploads no binaries. The follow-on exact-payload interaction harness now proves exact X11/AT-SPI discovery plus confirmed fixture selection and is validating the native Open action. This remains unsigned internal developer evidence, not installation, full accessibility/usability, supported importer, signing, legal approval, or distribution approval. GUI-5 and GUI-4 remain complete only for their bounded internal checkpoints; GUI-1 implementation remains ready for formal fixture/security review.
- **Documents awaiting review:** All documents marked Draft or In Review; especially ADR-0001–ADR-0014, advanced formulas/policies, calculation rules, requirements, security boundaries, and worked examples
- **Blocking decisions:** ADR acceptance blocks committing to production UI/kernel/worker/persistence choices
- **Next actions:** Close the Linux packaged accessibility/keyboard gate only when the real app exposes `Model selected` and the expected OCCT geometry state, then add interactive Windows evidence. Obtain GUI-1 fixture review; add Windows/macOS package integration, macOS/Windows parser egress denial, general filesystem containment, target dynamic-link/license/SBOM review, and signed application integration while closing remaining TASK-003 containment. TASK-004 mesh import, full TASK-005 durable desktop/rate UX, TASK-006 persistence, and TASK-007 shop calibration remain required for the broader product slice.

Implementation readiness extends through reversible TASK-001/002 calculation and rate mechanics plus the evidence-producing TASK-003 worker/OCCT spike. Architecture acceptance, controlled-data deployment, shop calibration, native packaging, runtime-accuracy claims, and product release remain blocked on M0.2/M1 evidence and review.

## Coverage summaries

- **Requirements:** REQ-F-001–065, REQ-NF-001–022, UX-001–012/021–045, SEC-001–014, CALC-001–035, GEO-001–015, FEAT-001–021, TIME-001–008, DATA-001–042, and TEST-001–099 are allocated and traced in the [requirements matrix](03-requirements/requirements-matrix.md). TASK-001/002 and partial TASK-003 provide cross-platform executable evidence; no end-to-end product requirement is yet Complete.
- **Geometry:** Intake/units/hash/healing/validity/basic measurements/stock/feature/access/setup stages are specified. Two STL, one OCCT-generated analytic STEP, one manually authored analytic STEP, and one invalid-entity STEP fixture exist. The optional local worker produces provisional exact-B-rep measurements through OCCT; developer runtimes can be assembled and content-verified independently of Cargo output on evidenced Apple-Silicon, Ubuntu-x86_64, and Windows-x64 configurations. Reviewed corpus accuracy, sandboxing, signed three-OS packaging, and product support remain absent.
- **Roadmap:** Phases 0–8 cover discovery, foundation, vertical slice, feature depth, libraries, aerospace/quality, actuals, advanced planning, and team deployment.
- **Highest risks:** Native geometry containment/fidelity, false precision in runtime/uncertainty/allocation, implausible routes, stale capacity/availability, revision ambiguity, biased learning, vendor coupling, UI complexity, sensitive-data handling, and historical replay; see [risk register](07-delivery/risk-register.md).
- **Formats:** First-class target: STEP, STL, 3MF. Experimental/secondary: IGES, OBJ. Proprietary/native formats: conversion or future licensed translator only. Current production support: none; the optional STEP worker is provisional spike evidence only.

## Authoritative-document map

| Subject | Authority |
|---|---|
| Mission/scope/non-goals | `00-product/vision.md`, `scope.md`, `non-goals.md` |
| Vocabulary | `00-product/terminology.md`; detailed additions in `02-domain/glossary.md` |
| Requirements/status | `03-requirements/*`, especially `requirements-matrix.md` |
| Calculation behavior | `02-domain/calculation-rules.md`; rate and pricing inputs in `02-domain/rate-library.md` and `pricing-model.md` |
| Domain/data boundaries | `02-domain/domain-overview.md`, `04-architecture/data-model.md` |
| Architecture decisions | `04-architecture/adr/*`; `08-decisions/decision-log.md` records status |
| UX/design | `05-ux/design-system.md` plus interaction/accessibility documents |
| Validation | `06-quality/*` and fixture manifest |
| Schedule/tasks/risks | `07-delivery/roadmap.md`, `backlog.md`, `risk-register.md` |
| Current handoff state | `PROJECT_STATE.md` |
| Advanced route/capacity/uncertainty behavior | `02-domain/routing-alternatives.md`, `capacity-and-opportunity-cost.md`, `probabilistic-estimation.md` |
| Revision, requirement, PMI, and feature cost behavior | `02-domain/revision-cost-analysis.md`, `requirement-coverage.md`, `feature-cost-allocation.md`; `01-research/pmi-and-mbd.md` |
| Learning, CAM, readiness, sourcing, and bid behavior | `02-domain/estimator-learning.md`, `availability-model.md`, `sourcing-and-make-buy.md`, `bid-and-quote-priority.md`; `04-architecture/cam-reconciliation.md` |

## Required technical spikes

| Spike | Status | Evidence gate |
|---|---|---|
| Tauri 2 + Leptos desktop shell, dense table, accessibility, wgpu/PDF integration | GUI-5 configured Apple-Silicon runtime, actual-app keyboard flow, semantic trace, cancellation/recovery, explicit estimate input, and deterministic result pass; dense table, full accessibility matrix, wgpu/PDF integration, three-OS package evidence, signing, and production adoption remain pending | ADR-0001/0003 and TASK-005 |
| OCCT STEP import/measurement/healing/package corpus | Exact 8.0.0 Apple-Silicon, Ubuntu-x86_64, and Windows-x64 construction plus supervised generated-cube and manually authored-prism measurements pass; fixture review, broader independent corpus, three-OS artifacts, legal/package evidence pending | ADR-0002 and TASK-003 |
| Isolated geometry-worker IPC/crash/resource boundary | Bounded subprocess, consumed already-open grant, identity-bound manifest v2, exact resource allowlisting, worker byte verification, controlled workspaces/output, cooperative/forced cancellation, poll-bounded aggregate workspace-output monitoring, Linux parser socket/process syscall denial, and partial Unix/Linux/Windows resource controls pass. The validated provisional snapshot binds claimed bytes to the source. Local Apple-Silicon plus hosted Ubuntu-x86_64 and Windows-x64 tooling constructs/fingerprints the pinned OCCT root before strict native checks; Ubuntu and Windows pass runtime/link verification and configured host smokes, with Linux alone adding seccomp. Ubuntu 22.04 additionally passes extracted Debian package-resource verification and real retained-session native analysis, with bounded exact-payload picker interaction under final validation. Fixture review, representative mid-transfer proof, internally uninterruptible phase bounds, macOS/Windows parser egress denial, general filesystem sandboxing and hard filesystem quotas, hard macOS memory, hostile macOS descendants, Windows/macOS package integration, signing, broader Linux accessibility/portal behavior, and interactive Windows acceptance remain pending | ADR-0005 and TASK-003 |
| STL/3MF mesh import/validation | Not started | ADR-0004 and TASK-004 |
| SQLite/blob migrations, crash, backup/restore, concurrency | Not started | ADR-0006 and TASK-006 |
| Typed decimal/unit calculation DAG, configurable rates and replay | TASK-001 and TASK-002 pass on three OSes; later shop calibration remains pending | ADR-0007, TASK-002 and TASK-007 |
| Analysis snapshot version/diff/migration | Not started | ADR-0008 and TASK-003/006 |
| Route feasibility/comparison and adoption | Not started | ADR-0009, TEST-040–044, TASK-011 |
| Capacity/economic definitions and stale-source behavior | Not started | ADR-0010, TEST-045–050, TASK-012 |
| Uncertainty distributions, reproducibility and comprehension | Not started | ADR-0011, TEST-051–055, TASK-013 |
| Revision mapping, PMI/coverage and cost-delta basis | Not started | ADR-0012, TEST-056–073, TASK-014/015 |
| Correction learning governance and CAM normalization | Not started | ADR-0013/0014, TEST-074–084, TASK-016 |
| Availability, sourcing and bid policy | Not started | TEST-085–099, TASK-017/018 |

## Complete document map

### Product

[Vision](00-product/vision.md) · [Users/personas](00-product/users-and-personas.md) · [Scope](00-product/scope.md) · [Non-goals](00-product/non-goals.md) · [Success metrics](00-product/success-metrics.md) · [Terminology](00-product/terminology.md)

### Research

[Plan](01-research/research-plan.md) · [Machining estimation](01-research/machining-estimation.md) · [CAD formats](01-research/cad-file-formats.md) · [Geometry kernels](01-research/geometry-kernels.md) · [Feature recognition](01-research/feature-recognition.md) · [Runtime](01-research/runtime-estimation.md) · [Feeds/speeds](01-research/feeds-and-speeds.md) · [Setup/orientation](01-research/setup-and-orientation-planning.md) · [PMI/MBD](01-research/pmi-and-mbd.md) · [Aerospace quality](01-research/aerospace-quality.md) · [Defense data](01-research/defense-data-handling.md) · [Competitors](01-research/competitor-analysis.md) · [Rust UI](01-research/rust-ui-evaluation.md) · [Sources](01-research/sources.md)

### Domain

[Overview](02-domain/domain-overview.md) · [Estimating](02-domain/estimating-model.md) · [Calculation rules](02-domain/calculation-rules.md) · [Rate library](02-domain/rate-library.md) · [Geometry analysis](02-domain/geometry-analysis-model.md) · [Features](02-domain/feature-model.md) · [Stock](02-domain/stock-selection-model.md) · [Materials](02-domain/material-model.md) · [Tooling](02-domain/tooling-model.md) · [Feeds/speeds](02-domain/feeds-and-speeds-model.md) · [Machines/workcenters](02-domain/machine-and-workcenter-model.md) · [Routing/operations](02-domain/routing-and-operation-model.md) · [Routing alternatives](02-domain/routing-alternatives.md) · [Capacity/economics](02-domain/capacity-and-opportunity-cost.md) · [Probabilistic estimates](02-domain/probabilistic-estimation.md) · [Requirement coverage](02-domain/requirement-coverage.md) · [Revision cost](02-domain/revision-cost-analysis.md) · [Feature cost/risk](02-domain/feature-cost-allocation.md) · [Availability](02-domain/availability-model.md) · [Sourcing/make-buy](02-domain/sourcing-and-make-buy.md) · [Estimator learning](02-domain/estimator-learning.md) · [Bid/priority](02-domain/bid-and-quote-priority.md) · [Quality/inspection](02-domain/quality-and-inspection-model.md) · [Pricing](02-domain/pricing-model.md) · [Quote lifecycle](02-domain/quote-lifecycle.md) · [Actuals/calibration](02-domain/actuals-and-calibration.md) · [Worked examples](02-domain/worked-examples.md) · [Glossary](02-domain/glossary.md)

### Requirements

[Functional](03-requirements/functional-requirements.md) · [Nonfunctional](03-requirements/nonfunctional-requirements.md) · [UX](03-requirements/ux-requirements.md) · [Security](03-requirements/security-requirements.md) · [Runtime](03-requirements/runtime-requirements.md) · [Matrix](03-requirements/requirements-matrix.md) · [Workflows](03-requirements/workflows.md) · [Permissions](03-requirements/permissions-matrix.md) · [CAD import](03-requirements/cad-import-requirements.md) · [Model analysis](03-requirements/model-analysis-requirements.md) · [Import/export](03-requirements/import-export-requirements.md) · [Reporting](03-requirements/reporting-requirements.md)

### Architecture and ADRs

[System](04-architecture/system-overview.md) · [Modules](04-architecture/module-boundaries.md) · [Data](04-architecture/data-model.md) · [Estimation](04-architecture/estimation-engine.md) · [Routing optimizer](04-architecture/routing-optimizer.md) · [Capacity engine](04-architecture/capacity-engine.md) · [Uncertainty engine](04-architecture/uncertainty-engine.md) · [Revision comparison](04-architecture/revision-comparison.md) · [Learning governance](04-architecture/learning-governance.md) · [CAM reconciliation](04-architecture/cam-reconciliation.md) · [Geometry](04-architecture/geometry-engine.md) · [Feature pipeline](04-architecture/feature-recognition-pipeline.md) · [Viewer](04-architecture/model-viewer.md) · [Persistence](04-architecture/persistence.md) · [Desktop](04-architecture/desktop-platform.md) · [Security](04-architecture/security-model.md) · [Deployments](04-architecture/deployment-models.md) · [Integrations](04-architecture/integration-strategy.md) · [Observability](04-architecture/observability.md)

[ADR-0001 UI](04-architecture/adr/ADR-0001-ui-framework.md) · [ADR-0002 kernel](04-architecture/adr/ADR-0002-geometry-kernel.md) · [ADR-0003 rendering](04-architecture/adr/ADR-0003-model-rendering.md) · [ADR-0004 formats](04-architecture/adr/ADR-0004-file-format-support.md) · [ADR-0005 worker](04-architecture/adr/ADR-0005-geometry-process-boundary.md) · [ADR-0006 persistence](04-architecture/adr/ADR-0006-persistence.md) · [ADR-0007 calculation](04-architecture/adr/ADR-0007-calculation-versioning.md) · [ADR-0008 analysis versioning](04-architecture/adr/ADR-0008-model-analysis-versioning.md)

[ADR-0009 routing](04-architecture/adr/ADR-0009-routing-alternatives.md) · [ADR-0010 capacity/economics](04-architecture/adr/ADR-0010-capacity-opportunity-cost.md) · [ADR-0011 uncertainty](04-architecture/adr/ADR-0011-uncertainty-representation.md) · [ADR-0012 revision/PMI](04-architecture/adr/ADR-0012-revision-and-pmi.md) · [ADR-0013 learning](04-architecture/adr/ADR-0013-learning-governance.md) · [ADR-0014 CAM](04-architecture/adr/ADR-0014-cam-integration.md)

### UX

[Principles](05-ux/experience-principles.md) · [Information architecture](05-ux/information-architecture.md) · [Design system](05-ux/design-system.md) · [Interactions](05-ux/interaction-patterns.md) · [Model review](05-ux/model-review-workflow.md) · [Feature review](05-ux/feature-review-workflow.md) · [Advanced analysis](05-ux/advanced-analysis-workflow.md) · [Route comparison](05-ux/routing-comparison.md) · [Requirement coverage](05-ux/requirement-coverage-workflow.md) · [Revision comparison](05-ux/revision-comparison-workflow.md) · [Accessibility](05-ux/accessibility.md) · [Screens](05-ux/screen-inventory.md) · [Wireframes](05-ux/workflow-wireframes.md) · [Usability tests](05-ux/usability-test-plan.md)

### Quality

[Strategy](06-quality/test-strategy.md) · [Calculations](06-quality/calculation-validation.md) · [TASK-001 evidence](06-quality/task-001-validation.md) · [TASK-002 evidence](06-quality/task-002-validation.md) · [TASK-003 evidence](06-quality/task-003-validation.md) · [GUI-2 evidence](06-quality/gui-2-validation.md) · [GUI-3 evidence](06-quality/gui-3-validation.md) · [GUI-4 evidence](06-quality/gui-4-validation.md) · [GUI-5 evidence](06-quality/gui-5-validation.md) · [Linux package evidence](06-quality/linux-desktop-package-validation.md) · [Uncertainty](06-quality/uncertainty-validation.md) · [Revision comparison](06-quality/revision-comparison-validation.md) · [CAM reconciliation](06-quality/cam-reconciliation-validation.md) · [Geometry](06-quality/geometry-validation.md) · [Fixtures](06-quality/model-fixture-strategy.md) · [Features](06-quality/feature-recognition-validation.md) · [Runtime](06-quality/runtime-estimation-validation.md) · [Test data](06-quality/test-data-strategy.md) · [Security](06-quality/security-testing.md) · [Cross-platform](06-quality/cross-platform-testing.md) · [Release acceptance](06-quality/release-acceptance.md)

### Delivery and decisions

[Roadmap](07-delivery/roadmap.md) · [Milestones](07-delivery/milestones.md) · [Backlog](07-delivery/backlog.md) · [Testable GUI plan](07-delivery/gui-vertical-slice-plan.md) · [Risks](07-delivery/risk-register.md) · [Dependency policy](07-delivery/dependencies.md) · [Dependency record](07-delivery/dependency-record.md) · [Release plan](07-delivery/release-plan.md) · [Progress log](07-delivery/progress-log.md) · [Assumptions](08-decisions/assumptions.md) · [Open questions](08-decisions/open-questions.md) · [Deferred](08-decisions/deferred-items.md) · [Decision log](08-decisions/decision-log.md) · [Changelog](../CHANGELOG.md)

## Instructions for future agents

Read [AGENTS.md](../AGENTS.md) and [PROJECT_STATE.md](PROJECT_STATE.md) before changes. Follow the authoritative-document map, preserve stable IDs and source/decision history, link tasks to requirements/tests, and update project state plus progress log after meaningful work. Do not infer completion from document presence.
