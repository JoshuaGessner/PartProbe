# Delivery Backlog

> **Status:** In Review  
> **Last updated:** 2026-07-23  
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

## Prioritized discovery/spike tasks

| ID | Task | Links | Exit |
|---|---|---|---|
| TASK-002 | Reconcile EX-01/03/12 into executable golden estimates with shop review | REQ-F-005–010; TEST-002 | Exact itemized totals and signed review record |
| TASK-003 | Benchmark OCCT STEP worker on representative corpus and three OSes | REQ-F-002–004; GEO-001–015; ADR-0002/0005; TEST-003 | Accuracy, crash containment, packaging/license evidence |
| TASK-004 | Implement STL/3MF mesh import comparison spike | REQ-F-002–004; ADR-0004; TEST-003 | Unit/mesh validity measurements and failure matrix |
| TASK-005 | Build Tauri/Leptos/wgpu UX spike | REQ-NF-001/006; UX-001–008; ADR-0001/0003; TEST-012 | Dense table, viewport picking, keyboard/a11y, PDF, package evidence |
| TASK-006 | SQLite repository/backup prototype | REQ-NF-007/009; DATA-001–012; ADR-0006; TEST-007 | Migration, crash, backup/restore, blob integrity |
| TASK-007 | Conduct shop workflow and rate interviews | REQ-F-001–012; OQ-001–030 | Reviewed maps, rate/cost policy, access decisions |
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
