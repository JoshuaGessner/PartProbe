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
- **Work in progress:** TASK-003 now binds open asset roots to stable identities; evaluates and audits deny-by-default access; stages verified bytes in private workspaces; claims and governs immutable output; uses a versioned execute/cancel control stream with explicit grace, validated acknowledgement, and distinct forced user/deadline termination; and wires adapter ABI v2 cancellation into OCCT STEP root transfer. The 88-test default baseline passes locally and on three OSes; native OCCT measurement/cancellation evidence remains Apple Silicon only
- **Next task:** Continue TASK-003 with direct worker descriptor/handle passing or a documented copy fallback, then OS containment/resource enforcement, independent fixtures including cancellation during a representative long transfer, and exact native-build automation; deployment policy/audit and TASK-006 durable storage remain separately evidence-gated
- **Build status:** Rust 1.94.1 default workspace passes locally and on Windows/Linux/macOS in GitHub Actions run 30492554999 at commit `ec78b15`; the optional OCCT 8.0.0 native feature passes locally on Apple Silicon only
- **Test status:** TASK-002 has 41 runtime tests plus one compile-fail doctest passing locally and on three OSes; TASK-003 has 88 default runtime tests passing locally and on three OSes in run 30492554999, plus nineteen focused optional-native adapter/worker tests passing locally; planning validator covers 137 documentation files
- **Active technical debt:** Rate entry has no UI or persistence yet; pinned ISO 4217 registry validation, nonterminating rational-division rounding, rounding increments beyond decimal scale, and graph-wide evaluation/migration remain pending; canonical JSON remains an internal spike contract; STEP/3MF and representative machining fixtures remain pending
- **Active blockers:** No shop rates block TASK-002 mechanics because production libraries start empty and tests are synthetic; TASK-007/M0.2 still requires real category/policy/calibration review, and ADR acceptance plus broader production work still require shop/security/legal inputs and representative fixtures
- **Open architecture decisions:** ADR-0001 through ADR-0014 are In Review
- **Supported model formats:** None implemented. First-class plan: STEP, STL, 3MF. Experimental plan: IGES, OBJ
- **Geometry capabilities:** Kernel-neutral contracts, bounded schema-versioned execute/cancel transport, cooperative acknowledgement with forced termination, a deny-by-default application authorization/audit seam, a root-identity-bound capability resolver, a consumed already-open supervisor grant, private per-job staging, immutable output claiming, a governed controlled-store port, explicit fixture evidence, and an optional OCCT 8.0.0 worker path produce provisional STEP area/volume/centroid evidence. Adapter ABI v2 polls cancellation during STEP root transfer; native parsing and property calls remain internally uninterruptible. No deployment allow policy/repository, durable audit/store adapter, direct worker handle grant, authoritative snapshot, independent translator corpus, OS sandbox, or network/CPU/memory enforcement is implemented
- **Runtime capabilities:** None implemented. Six estimation levels and validation structure documented only
- **Advanced capabilities:** None implemented. Route optimization, capacity/opportunity, uncertainty, revision/PMI/coverage, feature allocations, learning, CAM reconciliation, availability, sourcing, and bid priority are planning artifacts only
- **Current fixtures:** `FIX-MESH-001/002` cover closed/open synthetic STL cubes; `FIX-STEP-001` is a deterministic synthetic AP214 10 mm cube with exact expected area/volume/centroid; `FIX-STEP-002` contains an intentionally invalid entity with a recoverable-failure expectation. All hashes reproduce; same-kernel STEP bias, expert review, and licensing remain open
- **Calculation capabilities:** CALC-001/003/005 and CALC-007–018 foundations, with CALC-009 limited to exact division; empty configurable rate cards; scoped/effective/approved resolution; unavailable/blocked ambiguity states; versioned pricing/rounding; synthetic EX-01/03/12 exact itemized traces; pinned replay; typed money/currency/units; graph validation; value states; provenance/version wrappers; and canonical snapshots are implemented as a reversible spike
- **Files likely next:** TASK-003 security/application adapters, worker descriptor/handle transport, OS containment, independent analytic STEP fixtures, benchmark harness, native-build automation, dependency/license evidence, and three-OS worker CI; TASK-006 owns durable persistence, while UI and advanced engines remain separate slices

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
