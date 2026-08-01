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
- **Current branch:** `codex/task-003-geometry-worker-contract`, draft PR #2 stacked on TASK-002 PR #1
- **Last completed task/checkpoint:** TASK-002 configurable rate mechanics are validated; TASK-003 Checkpoint 20 adds a fail-closed, provenance-recorded OCCT 8.0.0 source-construction command and a manually authored analytic STEP rectangular prism that is independent of the OCCT fixture generator
- **Work in progress:** GUI-1 implementation now has repeatable pinned construction/discovery, a bounded provisional DTO, same-kernel and manually authored success fixtures, malformed-input evidence, cancellation, and unavailable-native behavior. Formal fixture review is still required before GUI-1 exits. The 102-test macOS default baseline, six native-tooling tests, and 26 focused Apple Silicon native tests plus strict native-feature Clippy pass locally; Linux/Windows retain one additional memory-limit regression
- **Next task:** Obtain fixture review and begin GUI-2's session-scoped application service that composes authorized model intake, worker evidence, explicit manual estimating inputs, rate resolution, pricing, and traceability. In parallel, TASK-003 still requires deployment-specific network/filesystem sandboxing, macOS hard-memory design, hostile Unix descendant containment, representative mid-transfer cancellation, three-OS native builds, and distribution review. The session-only developer slice does not replace TASK-003/005/006 acceptance
- **Build status:** Rust 1.94.1 Checkpoint 19 default gates pass locally and on Windows/Linux/macOS in implementation run 30712929648 at `9479505`. Checkpoint 20 freshly configured, built, installed, fingerprinted, and verified pinned OCCT 8.0.0 commit `d3056ef80c9668f395da40f5fd7be186cae4501f` on Apple Silicon; new three-OS default CI is pending
- **Test status:** TASK-002 has 41 runtime tests plus one compile-fail doctest passing locally and on three OSes; TASK-003 has 102 default runtime tests on macOS and 103 on Linux/Windows because their hard-memory regression is enabled, plus one compile-fail doctest, passing in run 30712929648. Six Python construction/fingerprint tests and 26 focused optional-native adapter/worker tests plus strict native-feature Clippy pass locally; the planning validator is rerun at closeout
- **Active technical debt:** Rate entry has no UI or persistence yet; pinned ISO 4217 registry validation, nonterminating rational-division rounding, rounding increments beyond decimal scale, and graph-wide evaluation/migration remain pending; canonical JSON remains an internal spike contract; STEP/3MF and representative machining fixtures remain pending
- **Active blockers:** No shop rates block TASK-002 mechanics because production libraries start empty and tests are synthetic; TASK-007/M0.2 still requires real category/policy/calibration review, and ADR acceptance plus broader production work still require shop/security/legal inputs and representative fixtures
- **Open architecture decisions:** ADR-0001 through ADR-0014 are In Review
- **Supported model formats:** None supported for production. An optional Apple Silicon STEP/OCCT spike handles governed synthetic evidence only. First-class plan: STEP, STL, 3MF; experimental plan: IGES, OBJ
- **Geometry capabilities:** Kernel-neutral contracts, bounded control schema v2, asset-transport manifest v2, explicit verified-private-copy selection, exact Unix descriptor and Windows HANDLE direct delivery with unrelated-resource exclusion proof, worker-side identity/type/hash/length/quota verification, immutable byte-stream adapter input, cooperative acknowledgement with forced termination, deny-by-default authorization/audit seams, root-identity-bound capability resolution, a consumed already-open grant, private per-job staging, immutable output claiming, governed storage ports, and a mandatory worker resource profile are implemented. The provisional native output now has a validated schema and decoder that binds reference and source hash. Unix hard-limits CPU/regular-file/core output and uses a killable process group; Linux additionally hard-limits address space. Windows assigns every suspended worker to a Job with CPU, committed-memory, one-process, and kill-on-close limits before resume. macOS hard memory, hostile Unix descendant escape prevention, aggregate output, network/filesystem sandboxing, and production deployment controls remain open. Optional OCCT 8.0.0 ABI-v3 evidence produces provisional STEP area/volume/centroid results; stream parsing/property calls remain internally uninterruptible
- **Runtime capabilities:** None implemented. Six estimation levels and validation structure documented only
- **Advanced capabilities:** None implemented. Route optimization, capacity/opportunity, uncertainty, revision/PMI/coverage, feature allocations, learning, CAM reconciliation, availability, sourcing, and bid priority are planning artifacts only
- **Current fixtures:** `FIX-MESH-001/002` cover closed/open synthetic STL cubes; `FIX-STEP-001` is a deterministic OCCT-generated AP214 10 mm cube; `FIX-STEP-002` contains an intentionally invalid entity; `FIX-STEP-003` is a manually authored AP214 faceted rectangular prism with independent analytic 12 × 8 × 5 mm ground truth, 392 mm² area, 480 mm³ volume, and `(6, 4, 2.5)` mm centroid. All hashes reproduce and native measurements pass; formal geometry/security review, broader CAD-tool independence, and licensing remain open
- **Calculation capabilities:** CALC-001/003/005 and CALC-007–018 foundations, with CALC-009 limited to exact division; empty configurable rate cards; scoped/effective/approved resolution; unavailable/blocked ambiguity states; versioned pricing/rounding; synthetic EX-01/03/12 exact itemized traces; pinned replay; typed money/currency/units; graph validation; value states; provenance/version wrappers; and canonical snapshots are implemented as a reversible spike
- **Nearest testable GUI:** GUI-1 implementation is ready for fixture review; approximately four further focused checkpoints produce a provisional Apple-Silicon, STEP-only, session-only GUI with numeric geometry facts, explicit manual estimating inputs, and a deterministic trace. A visible 3D viewport adds roughly two or three checkpoints because no tessellation or viewer implementation exists. This is an internal test slice, not developer alpha or production support
- **Files likely next:** GUI-2 application orchestration and headless integration tests; fixture review evidence; TASK-003 network/filesystem and remaining resource containment decisions; benchmark/legal evidence; and Windows/Linux native construction automation. The developer path is defined in `07-delivery/gui-vertical-slice-plan.md`; TASK-004 owns mesh imports, TASK-005 owns the full desktop/rate UX, and TASK-006 owns durable persistence

## Commands currently available

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
python3 scripts/check_planning.py
python3 scripts/hash_fixtures.py
python3 scripts/tests/test_native_tooling.py
python3 scripts/build_occt.py --source /approved/local/occt-source --build /private/tmp/partprobe-occt-build --install /private/tmp/partprobe-occt-install
git diff --check
git status --short --branch
```

## Read these first

1. [AGENTS.md](../AGENTS.md)
2. [Documentation index](INDEX.md)
3. [Decision log](08-decisions/decision-log.md)
4. [Requirements matrix](03-requirements/requirements-matrix.md)
5. The requirement, ADR, architecture, domain, UX, and quality files relevant to the task
