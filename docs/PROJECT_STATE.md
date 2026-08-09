# Project State

> **Status:** In Review
> **Last updated:** 2026-08-09
> **Related requirements:** All
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** OQ-001–OQ-050
> **Dependencies:** M0.2 evidence and shop review
> **Supersedes:** None

- **Current phase:** Phase 0 — Discovery and Validation
- **Current milestone:** M0.2 evidence and decision closure; TASK-001 and TASK-002 are validated on three OSes
- **Product readiness:** Pre-alpha engineering foundation. An unsigned Apple-Silicon developer shell can be built and launched locally; with an explicitly configured pinned worker/OCCT installation it can now request provisional STEP geometry facts, but it cannot yet complete review inputs, calculate an estimate, persist work, or constitute an installable/supported release
- **Current branch:** `main`; prior TASK-002/TASK-003 branch histories are consolidated and the obsolete local/remote task branches are deleted
- **Last completed task/checkpoint:** GUI-3 is complete for its bounded shell scope. A Tauri 2.11.5/Leptos 0.8.20 CSR shell, versioned typed desktop contract, asynchronous native STEP picker, pathless frontend summary, restrictive command manifest/capability/CSP, deliberate design-system foundation, and unsigned local macOS bundle are implemented. Thirteen focused contract/host/UI tests plus strict native and WASM Clippy pass locally; see `06-quality/gui-3-validation.md`
- **Work in progress:** GUI-4 is In Progress. Desktop contract v2 now exposes an asynchronous opaque-token analysis command; the native adapter requires explicit worker/workspace/OCCT paths, applies a session-scoped authorization policy and in-memory append-only audit, invokes `DraftEstimateApplication`, retains the returned `DraftEstimateSession`, and returns only path-free provisional facts/stage codes/hashes. Leptos renders running, safe failure, provisional geometry, and still-unavailable estimate states. Review/input/rate/policy/result commands and cancellation remain next. GUI-1 fixture review and TASK-003 containment/native packaging remain open in parallel
- **Next task:** Complete GUI-4's explicit canonical-unit/warning review and complete manual stock/material/time/cost/quantity, pinned rate-card/scope/date, pricing-policy, deterministic result-trace, and cancellation DTO/command/form flow without duplicating formulas or collapsing missing states. Then complete GUI-5 configured-worker smoke/accessibility evidence. In parallel, obtain GUI-1 fixture review and continue TASK-003 containment/native construction. The session-only developer slice does not replace TASK-003/005/006 acceptance
- **Build status:** The release Trunk bundle, 1280 × 900 semantic/visual preview, focused native-host and WASM strict Clippy, and focused tests pass locally for the GUI-4 analysis adapter. GitHub Actions run 31334097970 passes the complete code-quality matrix on macOS, Ubuntu, and Windows at implementation commit `8cd2a84`. No OCCT installation was available in this session, so configured native desktop analysis was not run end to end
- **Test status:** The local macOS default workspace passes 128 runtime tests; GUI-4's focused contract/native-host/UI set passes 20 tests, including token-only async command registration, exact capability scope, fail-closed configuration, post-authorization path-free adapter evidence, native draft-session creation, source-path redaction, explicit unavailable estimate state, and safe failure presentation. Strict workspace/native-host/WASM Clippy and the offline release frontend build pass locally; run 31334097970 passes formatting, six native-tooling tests, those three lint surfaces, all-target tests, and doctests on all three CI operating systems. Twenty-six focused optional-native adapter/worker tests remain the latest Apple-Silicon native evidence
- **Active technical debt:** Rate entry has no UI or persistence yet; pinned ISO 4217 registry validation, nonterminating rational-division rounding, rounding increments beyond decimal scale, and graph-wide evaluation/migration remain pending; canonical JSON remains an internal spike contract; STEP/3MF and representative machining fixtures remain pending
- **Active blockers:** No shop rates block TASK-002 mechanics because production libraries start empty and tests are synthetic; TASK-007/M0.2 still requires real category/policy/calibration review, and ADR acceptance plus broader production work still require shop/security/legal inputs and representative fixtures
- **Open architecture decisions:** ADR-0001 through ADR-0014 are In Review
- **Supported model formats:** None supported for production. An optional Apple Silicon STEP/OCCT spike handles governed synthetic evidence only. First-class plan: STEP, STL, 3MF; experimental plan: IGES, OBJ
- **Geometry capabilities:** Kernel-neutral contracts, bounded control schema v2, asset-transport manifest v2, explicit verified-private-copy selection, exact Unix descriptor and Windows HANDLE direct delivery with unrelated-resource exclusion proof, worker-side identity/type/hash/length/quota verification, immutable byte-stream adapter input, cooperative acknowledgement with forced termination, deny-by-default authorization/audit seams, root-identity-bound capability resolution, a consumed already-open grant, private per-job staging, immutable output claiming, governed storage ports, and a mandatory worker resource profile are implemented. GUI-4 intake can now compute a size-bounded SHA-256 fingerprint only after authorization/audit and rewind the same open grant for worker consumption; worker verification remains independent. The provisional native output has a validated schema and decoder that binds reference and source hash. Remaining containment and production gaps are unchanged
- **Runtime capabilities:** None implemented. Six estimation levels and validation structure documented only
- **Advanced capabilities:** None implemented. Route optimization, capacity/opportunity, uncertainty, revision/PMI/coverage, feature allocations, learning, CAM reconciliation, availability, sourcing, and bid priority are planning artifacts only
- **Current fixtures:** `FIX-MESH-001/002` cover closed/open synthetic STL cubes; `FIX-STEP-001` is a deterministic OCCT-generated AP214 10 mm cube; `FIX-STEP-002` contains an intentionally invalid entity; `FIX-STEP-003` is a manually authored AP214 faceted rectangular prism with independent analytic 12 × 8 × 5 mm ground truth, 392 mm² area, 480 mm³ volume, and `(6, 4, 2.5)` mm centroid. All hashes reproduce and native measurements pass; formal geometry/security review, broader CAD-tool independence, and licensing remain open
- **Calculation capabilities:** CALC-001/003/005 and CALC-007–018 foundations, with CALC-009 limited to exact division; empty configurable rate cards; scoped/effective/approved resolution; unavailable/blocked ambiguity states; versioned pricing/rounding; synthetic EX-01/03/12 exact itemized traces; pinned replay; typed money/currency/units; graph validation; value states; provenance/version wrappers; canonical snapshots; and a headless session-only application composition over reviewed provisional geometry/manual inputs/rates/policy are implemented as a reversible spike
- **Nearest testable GUI:** GUI-4 now has the code path for a configured Apple-Silicon, STEP-only, session-only GUI to show numeric provisional geometry facts. It has not been exercised end to end in this session because the pinned local OCCT runtime is absent, and it still cannot collect all GUI-2 prerequisites or display a deterministic estimate. One remaining GUI-4 workflow checkpoint plus GUI-5 configured-worker smoke/accessibility evidence stand between the current code and that internal test slice. A visible 3D viewport adds roughly two or three checkpoints because no tessellation or viewer implementation exists. This is not developer alpha or production support
- **Files likely next:** GUI-4 review/input/rate/policy/result and cancellation contracts, native commands, Leptos forms, and host/UI tests; GUI-1/GUI-2 review evidence; TASK-003 network/filesystem and remaining resource containment decisions; benchmark/legal evidence; and Windows/Linux native construction automation. The developer path is defined in `07-delivery/gui-vertical-slice-plan.md`; TASK-004 owns mesh imports, TASK-005 owns the full desktop/rate UX, and TASK-006 owns durable persistence

## Commands currently available

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo clippy -p partprobe-estimator-desktop-ui --target wasm32-unknown-unknown --locked -- -D warnings
cargo clippy -p partprobe-estimator-desktop --features desktop-host --all-targets --locked -- -D warnings
(cd apps/estimator-desktop && trunk build --release --locked true --offline true)
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
