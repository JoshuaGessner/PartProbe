# Requirements Coverage Matrix

> **Status:** In Review
> **Last updated:** 2026-07-29
> **Related requirements:** All
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** None beyond linked requirements
> **Dependencies:** Quality plans
> **Supersedes:** None

| Requirement set | Canonical definition | Architecture/domain coverage | Planned validation | Current evidence |
|---|---|---|---|---|
| REQ-F-001–020 | Functional capability/lifecycle requirements | domain, application, persistence, estimation | TEST-001–020 as mapped | TASK-001 primitives plus TASK-002 local configurable-rate and synthetic itemized-calculation evidence; UI/persistence/shop calibration pending |
| REQ-F-021–024 | Detailed CAD/feature/setup requirements | geometry/feature engines and worker | TEST-021–039 | Research and fixtures only |
| REQ-F-025–039 | Detailed review/library/report workflow requirements | UX, application, reporting, security | TEST-006–012, TEST-014–020 | Plans only |
| REQ-F-040–048 | Alternative routing, capacity/economics, and probabilistic estimates | routing optimizer, capacity engine, uncertainty engine | TEST-040–055 | Plans and formulas only |
| REQ-F-049–055 | Revision, requirement coverage, PMI, and feature-level explanation | revision comparison, coverage domain, PMI research, viewer contract | TEST-056–073 | Plans and synthetic fixture definitions only |
| REQ-F-056–065 | Corrections, CAM reconciliation, availability, sourcing, bid priority, and version pinning | learning governance, CAM adapters, availability/sourcing/bid domain | TEST-074–099 | Plans only |
| REQ-NF-001–022 | Nonfunctional requirements | ADRs, security, deployment, quality, replay and resource budgets | TEST-001/002/014, TEST-007, TEST-008, TEST-011–013, TEST-019–030, TEST-040–099 | TASK-001 supplies three-OS REQ-NF-001/003/004/010 evidence; TASK-002 supplies local rate/policy replay and missing-data evidence; broader gates remain planned |
| UX-001–012, 021–045 | UX requirements + UX documents | desktop platform + design system + rate setup + advanced-analysis workflows | TEST-012/014, TEST-028, TEST-040–099 + usability sessions | Plans only |
| SEC-001–014 | Security model/requirements | security + deployment + adapter/learning boundaries | TEST-011, TEST-061–069, TEST-074–099 | Threat/test plans only |
| CALC-001–035 | Calculation rules | estimation, routing, capacity, uncertainty, revision, sourcing, bid domains | TEST-001, TEST-002, TEST-040–055, TEST-058–060, TEST-090–099 | CALC-001/003/005 and CALC-007–018 foundations executable, with CALC-009 exact-only; synthetic EX-01/03/12 local goldens pass; governed nonterminating division, cross-platform evidence, and real-shop calibration pending |
| GEO-001–015 | CAD/model requirements | geometry architecture | TEST-021–030 | Two mesh fixtures; exact corpus pending |
| FEAT-001–021 | Model requirements | feature pipeline/model | TEST-031–039 | No executable detector |
| TIME-001–008 | Runtime research/model | runtime architecture | TEST-005 | Synthetic examples |
| DATA-001–042 | Data model | persistence + bounded domain aggregates | TEST-007, TEST-014–018, TEST-040–099 | Conceptual schema |
| TEST-001–099 | Quality strategy and specialized plans | `docs/06-quality` | Self-audit + future executable suites | TEST-001 passes locally and on Windows/Linux/macOS; TEST-002/014 have local partial TASK-002 evidence; TEST-003–099 otherwise remain planned |

Coverage means planned traceability, not completion. [TASK-001 evidence](../06-quality/task-001-validation.md) and [TASK-002 evidence](../06-quality/task-002-validation.md) are linked, but affected requirements remain incomplete until downstream UI, persistence, cross-platform, calibration, and approval gates pass.
