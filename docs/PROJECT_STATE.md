# Project State

> **Status:** In Review
> **Last updated:** 2026-07-29
> **Related requirements:** All
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** OQ-001–OQ-050
> **Dependencies:** M0.2 evidence and shop review
> **Supersedes:** None

- **Current phase:** Phase 0 — Discovery and Validation
- **Current milestone:** M0.2 evidence and decision closure; TASK-001 validated on three OSes and TASK-002 rate/golden mechanics validated locally
- **Current branch:** `main`
- **Last completed task:** TASK-001 implementation and Windows/Linux/macOS validation: workspace, primitives, typed DAG, canonical snapshots, tests, CI, and dependency evidence
- **Work in progress:** TASK-002 cross-platform evidence and closeout; local configurable rate-library, pricing/rounding, synthetic-golden, and replay implementation passes
- **Next task:** Complete [TASK-002 three-OS evidence](07-delivery/backlog.md), then begin TASK-003 OCCT/worker spike
- **Build status:** Rust 1.94.1 workspace passes on Apple Silicon macOS locally and on Windows/Linux/macOS in GitHub Actions run 30005930156
- **Test status:** Format and strict Clippy pass; 41 runtime tests and one compile-fail doctest pass locally; planning validator covers 136 documentation files; TASK-001 remains the latest three-OS evidence and TASK-002 CI is pending
- **Active technical debt:** Rate entry has no UI or persistence yet; pinned ISO 4217 registry validation, nonterminating rational-division rounding, rounding increments beyond decimal scale, graph-wide evaluation/migration, and cross-platform TASK-002 evidence remain pending; canonical JSON remains an internal spike contract; STEP/3MF and representative machining fixtures remain pending
- **Active blockers:** No shop rates block TASK-002 mechanics because production libraries start empty and tests are synthetic; TASK-007/M0.2 still requires real category/policy/calibration review, and ADR acceptance plus broader production work still require shop/security/legal inputs and representative fixtures
- **Open architecture decisions:** ADR-0001 through ADR-0014 are In Review
- **Supported model formats:** None implemented. First-class plan: STEP, STL, 3MF. Experimental plan: IGES, OBJ
- **Geometry capabilities:** None implemented. Documented pipeline and golden expectations only
- **Runtime capabilities:** None implemented. Six estimation levels and validation structure documented only
- **Advanced capabilities:** None implemented. Route optimization, capacity/opportunity, uncertainty, revision/PMI/coverage, feature allocations, learning, CAM reconciliation, availability, sourcing, and bid priority are planning artifacts only
- **Current fixtures:** `FIX-MESH-001` closed and `FIX-MESH-002` open 10 mm synthetic STL cubes with bootstrap expected records and verified hashes; expert review/licensing remain open
- **Calculation capabilities:** CALC-001/003/005 and CALC-007–018 foundations, with CALC-009 limited to exact division; empty configurable rate cards; scoped/effective/approved resolution; unavailable/blocked ambiguity states; versioned pricing/rounding; synthetic EX-01/03/12 exact itemized traces; pinned replay; typed money/currency/units; graph validation; value states; provenance/version wrappers; and canonical snapshots are implemented as a reversible spike
- **Files likely next:** TASK-002 three-OS CI evidence/closeout, then TASK-003 geometry-worker benchmark; no production UI, persistence, or advanced engine should be introduced through this calculation slice

## Commands currently available

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
python3 scripts/check_planning.py
python3 scripts/hash_fixtures.py
git diff --check
git status --short --branch
```

## Read these first

1. [AGENTS.md](../AGENTS.md)
2. [Documentation index](INDEX.md)
3. [Decision log](08-decisions/decision-log.md)
4. [Requirements matrix](03-requirements/requirements-matrix.md)
5. The requirement, ADR, architecture, domain, UX, and quality files relevant to the task
