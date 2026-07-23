# Progress Log

> **Status:** In Review  
> **Last updated:** 2026-07-23  
> **Related requirements:** All  
> **Related ADRs:** ADR-0001–ADR-0014  
> **Open questions:** OQ-001–OQ-050  
> **Dependencies:** None  
> **Supersedes:** None

## 2026-07-23 — TASK-001 calculation foundation

- Created a Cargo resolver-v3 workspace pinned to Rust 1.94.1 with `domain`, `estimation-engine`, and `test-support` crates.
- Added project-owned typed volume/density/mass/quantity and fixed-precision money/currency primitives, explicit value states, provenance, schema versions, and semantic rule references.
- Implemented CALC-001, CALC-003, CALC-005, CALC-016, and CALC-017 plus a typed DAG definition/validation boundary that rejects duplicate/missing/type-incompatible/cyclic nodes.
- Added deterministic snapshot JSON with ordered maps, decimal strings, schema/rule versions, provenance, warnings, and explicit result states.
- Added exact/property/boundary tests: 23 runtime tests and one compile-fail unit-safety doctest pass locally. Formatting and strict Clippy also pass.
- Independent review found and drove fixes for deserialization invariant bypass, omitted intermediate traces, scale/negative-zero canonicalization, and implicit precision loss in physical and monetary arithmetic.
- Added a Windows/Linux/macOS GitHub Actions matrix and pinned checkout action/toolchain. The matrix has not run because the repository has no Git remote.
- Added exact dependency and active-transitive evidence for `rust_decimal`, Serde, and Serde JSON. ADR-0007 remains In Review; canonical JSON remains an internal spike contract.
- No UI, geometry, persistence, runtime estimator, external integration, routing optimizer, or advanced-analysis engine was added.

## 2026-07-22 — Advanced planning enhancement opened

- **Documents reviewed:** `AGENTS.md`, `docs/INDEX.md`, `docs/PROJECT_STATE.md`, product vision/scope/non-goals/success metrics, roadmap/milestones/backlog/risks, estimating/geometry/feature/runtime/pricing/quality/security/actuals models, requirements matrix, data architecture, and ADR-0001–ADR-0008. No ADR is Approved; all remain In Review.
- **Existing requirements affected:** REQ-F-006–REQ-F-007, REQ-F-010–REQ-F-013, REQ-F-016–REQ-F-020, REQ-F-022–REQ-F-025, REQ-F-027–REQ-F-030, REQ-F-032–REQ-F-039; REQ-NF-002–REQ-NF-010; CALC-010–CALC-020; DATA-004, DATA-006–DATA-012, DATA-015–DATA-018; TIME-001–TIME-008; UX-001–UX-010 and UX-021–UX-027; SEC-001–SEC-010.
- **New systems in scope:** routing alternatives, capacity/opportunity cost, probabilistic estimates, geometric revision/cost difference, requirement coverage, PMI/MBD, feature-cost/risk visualization, correction learning, CAM reconciliation, availability-aware estimation, make/buy sourcing, and bid/no-bid priority scoring.
- **Expected ADRs:** routing/scoring boundary, capacity/opportunity-cost semantics, uncertainty representation, revision/PMI comparison, learning governance, and CAM integration contracts (provisionally ADR-0009–ADR-0014).
- **Major unresolved decisions:** shop bottlenecks and schedule data sources; uncertainty method and appetite; usable AP242/PMI translator coverage; CAM vendors/formats; inventory freshness ownership; requirement-readiness exception authority; contribution/opportunity-cost policy; correction-learning approval cohort.
- **Documents expected to change:** the mandatory files named by the enhancement prompt plus focused domain, architecture, UX, quality, ADR, source/index/state, risk, roadmap, milestone, and backlog documents. No production implementation is authorized in this session.

## 2026-07-22 — Advanced planning enhancement integrated

- Added planning for all twelve requested systems: alternative routing; capacity/opportunity economics; probabilistic estimation; revision/cost difference; manufacturing requirement coverage; correction learning; CAM reconciliation; availability-aware estimation; make/buy; feature-level cost/risk explanation; structured PMI/MBD; and bid/quote priority.
- Allocated REQ-F-040–065, REQ-NF-015–022, SEC-011–014, UX-028–045, CALC-021–035, DATA-019–042, and TEST-040–099 without marking them Complete.
- Added ADR-0009–ADR-0014 as In Review. Human adoption, baseline immutability, explicit unavailable/stale states, source/version pinning, vendor-neutral adapters, and no automatic learning activation are cross-cutting invariants.
- Refined Phases 0–8 and TASK-011–TASK-018. Phase 2 remains intentionally narrow: one route, basic requirement checklist, manual readiness/availability, and manual make/buy; uncertainty, capacity optimization, broad PMI, and governed learning are later gates.
- Expanded the risk register to RISK-036, open questions to OQ-050, assumptions to A-020, and added dedicated domain/architecture/UX/quality plans and research for PMI/MBD.
- Preserved the deterministic estimate as the accepted baseline and distinguished capability from readiness, confidence from uncertainty, accounting cost from opportunity cost, and recommendation from authority.
- Planning-only change: no Cargo workspace, production module, dependency, integration, or external data transmission was added. TASK-001 remains the exact next implementation task.

## 2026-07-22 — Phase 0 planning baseline

- Inspected the prompt-only repository and initialized Git on `main`.
- Established root governance and the requested documentation hierarchy.
- Researched current CAD/geometry, Rust desktop UI/rendering, machining/runtime, quality, controlled-data, security, persistence, and versioning topics with primary-source links.
- Proposed Tauri 2 + Leptos, OCCT in an isolated worker, wgpu rendering, STEP/STL/3MF initial support, SQLite standalone persistence, and typed deterministic calculation graphs; all architectural ADRs remain In Review pending spikes.
- Defined domain/data/calculation/geometry/runtime models, requirements and test traceability, eighteen synthetic worked scenarios, roadmap/backlog, risks, assumptions, and open shop questions.
- Added two public synthetic STL fixtures, SHA-256 manifest entries, and bootstrap expected-result records for harness bootstrap.
- Completed independent integration reviews across UI, CAD/geometry, machining/runtime, and security. Reconciled provisional IDs; corrected decision-status wording, representation provenance, inspection-time taxonomy, Tauri/WebView security controls, fixture policy, and usability target hierarchy.
- Verified 102 documentation files for required metadata and local links; `git diff --check` passed; both fixture hashes were reproduced. No production tests exist.
- No production modules or external data integrations were created.
- Next: complete M0.2 evidence tasks, beginning with TASK-001 while running the architecture spikes in parallel as staffing permits.
