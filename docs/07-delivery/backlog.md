# Delivery Backlog

> **Status:** In Review
> **Last updated:** 2026-08-09
> **Related requirements:** All
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** OQ-001–OQ-050
> **Dependencies:** Milestones
> **Supersedes:** None

## TASK-001 — Bootstrap the Rust workspace and calculation primitives

**Status:** Complete on 2026-07-23; local and Windows/Linux/macOS CI validation pass. See [validation evidence](../06-quality/task-001-validation.md).
**Requirements:** REQ-NF-001, REQ-NF-003, REQ-NF-004, REQ-NF-010; CALC-001, CALC-003, CALC-005, CALC-016, CALC-017; TEST-001.
**ADR constraints:** ADR-0007 (In Review; implement as a reversible spike if not yet accepted).
**Scope:** Create the root Cargo workspace, `domain`, `estimation-engine`, and `test-support` crates; typed units; fixed-precision money/currency; provenance/version wrappers; initial acyclic-node interface; CI checks for format, lint, tests on Windows/Linux/macOS.
**Acceptance:** exact money tests; unit mismatch compile/runtime boundaries; margin/markup golden tests; make-quantity and removed-volume properties; cycle rejection; canonical serialization test; dependency/license notes; `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` pass.
**Expected files:** root Cargo/config, three crates, CI workflow, dependency record, test fixtures, project-state/progress/requirements updates.
**Known risks:** decimal serialization/overflow policy, unit crate ergonomics, premature public API.

## TASK-002 — Configurable rate library and synthetic golden estimates

**Status:** Complete on 2026-07-29; local and Windows/Linux/macOS CI validation pass in run 30461694180. See [validation evidence](../06-quality/task-002-validation.md).
**Requirements:** REQ-F-008–REQ-F-010, REQ-F-014–REQ-F-018, REQ-F-032; REQ-NF-003/004/009/017/020; UX-011/012; CALC-007–CALC-018; TEST-002/014.
**ADR constraints:** ADR-0007 remains In Review; implementation remains a reversible domain/calculation spike.
**Scope:** Empty production rate card; user-owned immutable/effective/scoped/approved rates; deterministic selection and missing/conflict states; versioned pricing/rounding; synthetic EX-01/03/12 itemized traces and pinned replay. No UI, persistence, production defaults, or real-shop calibration.
**Acceptance:** strict local and three-OS checks pass; 41 total runtime tests plus one compile-fail doctest pass; synthetic fixtures are isolated and exact; missing/conflicting rates and duplicate/composite double charges cannot produce authoritative values; historical selector/rate versions replay.
**Calibration boundary:** TASK-007/M0.2—not TASK-002—validates real shop categories, accounting treatment, pricing/rounding policy, roles, and accuracy.

## Prioritized discovery/spike tasks

TASK-003 is **In Progress** on 2026-08-01. Its stacked slices add kernel-neutral contracts, bounded control/transport schemas, exact verified-copy/Unix-descriptor/Windows-HANDLE delivery, worker byte verification, cancellation, partial target resource containment, fixture expectation schema v2, optional OCCT 8.0.0 ABI-v3 parsing, and a source-bound provisional snapshot. Checkpoint 20 adds exact-commit/clean-tree OCCT construction automation with a recorded build manifest and `FIX-STEP-003`, a manually authored analytic AP214 prism verified through both adapter and supervisor. Default and tooling gates pass Windows/Linux/macOS in run 30716328418; 26 native tests pass locally on Apple Silicon. Representative mid-transfer cancellation, internally uninterruptible stream/property bounds, formal fixture review and a broader corpus, network/filesystem and remaining resource containment, legal approval, reproducible Windows/Linux construction/packaging, and three-OS native benchmarks remain required; see [TASK-003 validation](../06-quality/task-003-validation.md).

The [testable GUI vertical-slice plan](gui-vertical-slice-plan.md) defines a smaller internal, STEP-only, session-only developer path. GUI-1 implementation is awaiting fixture review; GUI-2 application orchestration and GUI-3's restrictive desktop shell are complete for their bounded checkpoint scopes. GUI-4 application/workspace integration is next. Passing the path does not complete TASK-003, TASK-005, TASK-006, M1, or M2.

| ID | Task | Links | Exit |
|---|---|---|---|
| TASK-002 | Implement the configurable rate-library contract and reconcile EX-01/03/12 as synthetic executable golden estimates | REQ-F-005–010, REQ-F-014–018, REQ-F-032; TEST-002/014 | No production defaults; exact itemized traces; missing/ambiguous-rate, rounding and replay evidence |
| TASK-003 | Benchmark OCCT STEP worker on representative corpus and three OSes | REQ-F-002–004; GEO-001–015; ADR-0002/0005; TEST-003 | Accuracy, crash containment, packaging/license evidence |
| TASK-004 | Implement STL/3MF mesh import comparison spike | REQ-F-002–004; ADR-0004; TEST-003 | Unit/mesh validity measurements and failure matrix |
| TASK-005 | **In Progress:** build Tauri/Leptos/wgpu UX spike including basic rate setup; GUI-3 proves only the restrictive shell/native-picker subset | REQ-F-032; REQ-NF-001/006; UX-001–012; ADR-0001/0003; TEST-012/014 | Dense rate grid and formula/selection trace, viewport picking, keyboard/a11y, PDF, three-OS package evidence |
| TASK-006 | SQLite repository/backup prototype including rate/policy replay | REQ-NF-007/009/020; DATA-001–012/017; ADR-0006; TEST-007/014 | Migration, immutable rate/policy versions, crash, backup/restore, blob integrity and prior-version replay |
| TASK-007 | Conduct shop workflow, rate-category, pricing-policy and calibration interviews | REQ-F-001–018, REQ-F-032; OQ-001–030; TEST-002/014 | Reviewed maps, real rate/cost categories, accounting/pricing policy, roles, calibration and access decisions |
| TASK-008 | Establish private fixture governance and corpus | TEST-003–005/011 | Rights, classification, hashes, expected results, access |
| TASK-009 | Threat-model intake, worker, storage, previews, exports, updates | SEC-001–010; TEST-011 | Reviewed mitigations and residual risks |
| TASK-010 | Approve, reject, or explicitly defer ADR-0001–0014 from evidence | REQ-NF-010, REQ-NF-015–021 | Decision log with owners/dates/evidence |
| TASK-011 | Collect and review alternative-routing fixtures and scoring decisions | REQ-F-040–042; TEST-040–044; ADR-0009 | At least five parts with expert-reviewed feasible/rejected routes and explanations |
| TASK-012 | Audit capacity/bottleneck data and economic definitions | REQ-F-043–046; CALC-021–026/031/033; TEST-045–050; ADR-0010 | Source/freshness map, reviewed cost-basis glossary and missing-data policy |
| TASK-013 | Validate staged uncertainty methods on worked estimates | REQ-F-047–048; CALC-027–029; TEST-051–055; ADR-0011 | Reviewed three-point/scenario results and decision on simulation deferral |
| TASK-014 | Build revision-comparison fixture and tolerance study | REQ-F-049–050; CALC-030; TEST-056–060; ADR-0012 | Identical/unit/small/large/topology/mesh cases with reviewed manufacturing deltas |
| TASK-015 | Inventory requirement sources and AP242/PMI evidence | REQ-F-051–055; TEST-061–073; ADR-0012 | Source-priority/readiness policy, customer examples, translator scorecard |
| TASK-016 | Inventory correction, CAM, machine and actual data paths | REQ-F-056–059; TEST-074–084; ADR-0013/0014 | Event taxonomy, privacy/governance rules, CAM adapter candidates and fixtures |
| TASK-017 | Define availability and make/buy source ownership | REQ-F-060–062; CALC-032/034; TEST-085–094 | Source/freshness/reservation policy and reviewed comparison examples |
| TASK-018 | Facilitate bid/no-bid and quote-priority policy workshop | REQ-F-063–064; CALC-035; TEST-095–099 | Versioned draft factors/weights/blockers/approvals with sensitivity review |

No task may be marked complete without its acceptance evidence and affected document updates.
