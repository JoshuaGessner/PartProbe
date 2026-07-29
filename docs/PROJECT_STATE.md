# Project State

> **Status:** In Review
> **Last updated:** 2026-07-29
> **Related requirements:** All
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** OQ-001–OQ-050
> **Dependencies:** M0.2 evidence and shop review
> **Supersedes:** None

- **Current phase:** Phase 0 — Discovery and Validation
- **Current milestone:** M0.2 evidence and decision closure; TASK-001 and TASK-002 are validated on three OSes
- **Current branch:** `codex/task-003-geometry-worker-contract`, stacked on TASK-002 PR #1
- **Last completed task:** TASK-002 configurable rate-library, pricing/rounding, synthetic-golden, replay, and Windows/Linux/macOS validation
- **Work in progress:** TASK-003 kernel-neutral contracts, bounded subprocess supervision, fixture expectation schema version 2, an exact local OCCT 8.0.0 source build, and an optional project-owned C ABI shim pass locally; asset resolution, OS sandbox/resource enforcement, analytic STEP evidence, legal approval, packaging, and three-OS native benchmarks remain
- **Next task:** Resolve opaque asset capabilities to fixed worker-local sources, add a reviewed analytic STEP fixture, and exercise the optional OCCT adapter end-to-end before native packaging
- **Build status:** Rust 1.94.1 default workspace passes locally and on Windows/Linux/macOS in GitHub Actions run 30467602461; the optional OCCT 8.0.0 native feature passes locally on Apple Silicon only
- **Test status:** TASK-002 has 41 runtime tests plus one compile-fail doctest passing locally and on three OSes; TASK-003 has 56 default runtime tests and the doctest passing on three OSes in run 30467602461, plus three focused optional-native tests passing locally; planning validator covers 137 documentation files
- **Active technical debt:** Rate entry has no UI or persistence yet; pinned ISO 4217 registry validation, nonterminating rational-division rounding, rounding increments beyond decimal scale, and graph-wide evaluation/migration remain pending; canonical JSON remains an internal spike contract; STEP/3MF and representative machining fixtures remain pending
- **Active blockers:** No shop rates block TASK-002 mechanics because production libraries start empty and tests are synthetic; TASK-007/M0.2 still requires real category/policy/calibration review, and ADR acceptance plus broader production work still require shop/security/legal inputs and representative fixtures
- **Open architecture decisions:** ADR-0001 through ADR-0014 are In Review
- **Supported model formats:** None implemented. First-class plan: STEP, STL, 3MF. Experimental plan: IGES, OBJ
- **Geometry capabilities:** Kernel-neutral contracts, bounded JSON subprocess transport, worker lifecycle control, explicit fixture evidence states, and an optional local OCCT 8.0.0 C ABI capable of raw STEP transfer/basic-property calls are implemented; no asset resolver, authoritative STEP fixture result, activated worker parser, OS sandbox, or network/resource enforcement is implemented
- **Runtime capabilities:** None implemented. Six estimation levels and validation structure documented only
- **Advanced capabilities:** None implemented. Route optimization, capacity/opportunity, uncertainty, revision/PMI/coverage, feature allocations, learning, CAM reconciliation, availability, sourcing, and bid priority are planning artifacts only
- **Current fixtures:** `FIX-MESH-001` closed and `FIX-MESH-002` open 10 mm synthetic STL cubes have verified source hashes and schema-version-2 exact expected records that distinguish available from unavailable evidence; expert review/licensing remain open
- **Calculation capabilities:** CALC-001/003/005 and CALC-007–018 foundations, with CALC-009 limited to exact division; empty configurable rate cards; scoped/effective/approved resolution; unavailable/blocked ambiguity states; versioned pricing/rounding; synthetic EX-01/03/12 exact itemized traces; pinned replay; typed money/currency/units; graph validation; value states; provenance/version wrappers; and canonical snapshots are implemented as a reversible spike
- **Files likely next:** TASK-003 kernel-neutral `geometry-core`, worker protocol/supervisor, analytic STEP fixtures, benchmark harness, dependency/license evidence, and three-OS worker CI; UI, persistence, and advanced engines remain separate slices

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
