# Project State

> **Status:** In Review
> **Last updated:** 2026-08-01
> **Related requirements:** All
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** OQ-001–OQ-050
> **Dependencies:** M0.2 evidence and shop review
> **Supersedes:** None

- **Current phase:** Phase 0 — Discovery and Validation
- **Current milestone:** M0.2 evidence and decision closure; TASK-001 and TASK-002 are validated on three OSes
- **Product readiness:** Pre-alpha engineering foundation; no installable desktop application, durable production repository, calibrated shop deployment, or releasable package exists
- **Current branch:** `codex/task-003-geometry-worker-contract`, stacked on TASK-002 PR #1
- **Last completed task/checkpoint:** TASK-002 configurable rate mechanics are validated; TASK-003 Checkpoint 18 adds a mandatory pre-execution resource profile, Unix CPU/file/core limits and process-group cleanup, Linux address-space limits, and suspended Windows Job assignment with CPU/memory/process-tree limits
- **Work in progress:** TASK-003 binds open roots/grants to stable identities; emits control schema v2 plus asset-transport manifest v2; selects explicit `verified_private_copy` everywhere by default, `unix_descriptor` under direct policy on Unix, or `windows_handle` under direct policy on Windows; independently verifies request/manifest identity, regular-file type, exact length, quota, and SHA-256 inside the worker; applies resource controls before worker execution; preserves cancellation/kill/reap and deterministic cleanup; and governs claimed output. The 99-test macOS default baseline passes locally; Linux/Windows add one memory-limit regression; 21 focused Apple Silicon native tests pass locally
- **Next task:** Continue TASK-003 with deployment-specific network/filesystem sandboxing, macOS hard-memory design, hostile Unix descendant containment, representative mid-transfer cancellation, independent fixtures, and exact native-build automation; deployment policy/audit and TASK-006 durable storage remain separately evidence-gated
- **Build status:** Rust 1.94.1 Checkpoint 18 passes locally and in GitHub Actions run 30706523670 at fix commit `c71c0e9` on Windows/Linux/macOS. The optional OCCT 8.0.0 ABI-v3 feature passes locally on Apple Silicon only
- **Test status:** TASK-002 has 41 runtime tests plus one compile-fail doctest passing locally and on three OSes; TASK-003 has 99 default runtime tests on macOS and 100 on Linux/Windows because their hard-memory regression is enabled, plus one compile-fail doctest, passing in run 30706523670; 21 focused optional-native adapter/worker tests pass locally; the planning validator covers 137 documentation files
- **Active technical debt:** Rate entry has no UI or persistence yet; pinned ISO 4217 registry validation, nonterminating rational-division rounding, rounding increments beyond decimal scale, and graph-wide evaluation/migration remain pending; canonical JSON remains an internal spike contract; STEP/3MF and representative machining fixtures remain pending
- **Active blockers:** No shop rates block TASK-002 mechanics because production libraries start empty and tests are synthetic; TASK-007/M0.2 still requires real category/policy/calibration review, and ADR acceptance plus broader production work still require shop/security/legal inputs and representative fixtures
- **Open architecture decisions:** ADR-0001 through ADR-0014 are In Review
- **Supported model formats:** None supported for production. An optional Apple Silicon STEP/OCCT spike handles governed synthetic evidence only. First-class plan: STEP, STL, 3MF; experimental plan: IGES, OBJ
- **Geometry capabilities:** Kernel-neutral contracts, bounded control schema v2, asset-transport manifest v2, explicit verified-private-copy selection, exact Unix descriptor and Windows HANDLE direct delivery with unrelated-resource exclusion proof, worker-side identity/type/hash/length/quota verification, immutable byte-stream adapter input, cooperative acknowledgement with forced termination, deny-by-default authorization/audit seams, root-identity-bound capability resolution, a consumed already-open grant, private per-job staging, immutable output claiming, governed storage ports, and a mandatory worker resource profile are implemented. Unix hard-limits CPU/regular-file/core output and uses a killable process group; Linux additionally hard-limits address space. Windows assigns every suspended worker to a Job with CPU, committed-memory, one-process, and kill-on-close limits before resume. macOS hard memory, hostile Unix descendant escape prevention, aggregate output, network/filesystem sandboxing, and production deployment controls remain open. Optional OCCT 8.0.0 ABI-v3 evidence produces provisional STEP area/volume/centroid results; stream parsing/property calls remain internally uninterruptible
- **Runtime capabilities:** None implemented. Six estimation levels and validation structure documented only
- **Advanced capabilities:** None implemented. Route optimization, capacity/opportunity, uncertainty, revision/PMI/coverage, feature allocations, learning, CAM reconciliation, availability, sourcing, and bid priority are planning artifacts only
- **Current fixtures:** `FIX-MESH-001/002` cover closed/open synthetic STL cubes; `FIX-STEP-001` is a deterministic synthetic AP214 10 mm cube with exact expected area/volume/centroid; `FIX-STEP-002` contains an intentionally invalid entity with a recoverable-failure expectation. All hashes reproduce; same-kernel STEP bias, expert review, and licensing remain open
- **Calculation capabilities:** CALC-001/003/005 and CALC-007–018 foundations, with CALC-009 limited to exact division; empty configurable rate cards; scoped/effective/approved resolution; unavailable/blocked ambiguity states; versioned pricing/rounding; synthetic EX-01/03/12 exact itemized traces; pinned replay; typed money/currency/units; graph validation; value states; provenance/version wrappers; and canonical snapshots are implemented as a reversible spike
- **Files likely next:** TASK-003 network/filesystem and remaining resource containment decisions, independent analytic STEP fixtures, benchmark harness, native-build automation, dependency/license evidence, and three-OS native CI; TASK-004 owns mesh imports, TASK-005 owns the desktop/rate UX, and TASK-006 owns durable persistence

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
