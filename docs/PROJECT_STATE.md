# Project State

> **Status:** In Review  
> **Last updated:** 2026-07-23  
> **Related requirements:** All  
> **Related ADRs:** ADR-0001–ADR-0014  
> **Open questions:** OQ-001–OQ-050  
> **Dependencies:** M0.2 evidence and shop review  
> **Supersedes:** None

- **Current phase:** Phase 0 — Discovery and Validation
- **Current milestone:** M0.2 evidence and decision closure; calculation foundation spike validated on three OSes
- **Current branch:** `main`
- **Last completed task:** TASK-001 implementation and Windows/Linux/macOS validation: workspace, primitives, typed DAG, canonical snapshots, tests, CI, and dependency evidence
- **Work in progress:** TASK-002 input and shop-review preparation
- **Next task:** [TASK-002 — executable reviewed golden estimates](07-delivery/backlog.md)
- **Build status:** Rust 1.94.1 workspace passes on Apple Silicon macOS locally and on Windows/Linux/macOS in GitHub Actions run 30005930156
- **Test status:** Format and strict Clippy pass; 23 runtime tests and one compile-fail doctest pass; planning validator and fixture hashes pass at closeout; TEST-002–TEST-099 remain planned
- **Active technical debt:** Decimal scale/rounding and rule-upgrade replay await TASK-002; canonical JSON remains an internal spike contract; advanced fixtures are definitions only; STEP/3MF and representative machining fixtures remain pending
- **Active blockers:** TASK-002 requires shop-reviewed worked-estimate inputs and rounding policy; ADR acceptance and broader production work still require shop/security/legal inputs and representative fixtures
- **Open architecture decisions:** ADR-0001 through ADR-0014 are In Review
- **Supported model formats:** None implemented. First-class plan: STEP, STL, 3MF. Experimental plan: IGES, OBJ
- **Geometry capabilities:** None implemented. Documented pipeline and golden expectations only
- **Runtime capabilities:** None implemented. Six estimation levels and validation structure documented only
- **Advanced capabilities:** None implemented. Route optimization, capacity/opportunity, uncertainty, revision/PMI/coverage, feature allocations, learning, CAM reconciliation, availability, sourcing, and bid priority are planning artifacts only
- **Current fixtures:** `FIX-MESH-001` closed and `FIX-MESH-002` open 10 mm synthetic STL cubes with bootstrap expected records and verified hashes; expert review/licensing remain open
- **Calculation capabilities:** CALC-001/003/005/016/017, typed money/currency/units, graph validation, explicit value states, provenance/version wrappers, and canonical snapshots are implemented as a reversible spike
- **Files likely next:** TASK-002 structured golden fixtures and calculation tests, followed by state/progress/matrix updates; no advanced engine should be introduced

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
