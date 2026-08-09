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
- **Product readiness:** Pre-alpha engineering foundation. A bounded Apple-Silicon internal developer GUI now runs end to end: an unsigned local bundle can use an explicitly configured, separately assembled and content-verified pinned OCCT runtime to analyze the governed STEP prism, require revision-bound review and complete inputs, and produce a deterministic provisional USD 702 estimate trace. Nothing is persisted; the runtime is not embedded/signed/distributable, and supported importer evidence, Windows/Linux native construction, full accessibility, and release acceptance remain absent
- **Current branch:** `main`; prior TASK-002/TASK-003 branch histories are consolidated and the obsolete local/remote task branches are deleted
- **Last completed task/checkpoint:** TASK-003 developer-runtime assembly is complete for Apple Silicon. A fail-closed schema-v1 assembler binds the exact OCCT source/build/host provenance, verified worker, version header, construction manifest, and all 23 observed `TK*` shared-library families into a new separately verified root; the opt-in GUI-5 real STEP-to-estimate smoke passes against that copy. GUI-5 actual-app evidence remains unchanged; see `06-quality/task-003-validation.md` and `06-quality/gui-5-validation.md`
- **Work in progress:** GUI-1 formal fixture/security review plus Windows/Linux native construction/runtime assembly and remaining TASK-003 containment are the immediate parallel gates. The GUI-4/GUI-5 slice is testable but remains session-only, developer-configured, one-platform evidence with no durable shop library, save/reopen, supported importer, viewer, signed package, or production approval
- **Next task:** Extend pinned OCCT construction, schema-v1 runtime assembly/verification, dynamic-link inspection, and launch evidence to Windows and Linux while closing remaining worker network/filesystem/resource containment. In parallel, obtain GUI-1 fixture review. Then proceed through TASK-004 mesh intake, TASK-005 durable shop-owned rate/model-review UX, and TASK-006 persistence. No user input is needed for the immediate developer engineering slice; shop/security/legal review is still required at its named gates
- **Build status:** Exact OCCT `V8_0_0` commit `d3056ef80c9668f395da40f5fd7be186cae4501f` was freshly constructed on Apple Silicon and produced verified worker SHA-256 `3d3770c0cf95016800f1524f6be8fa7d6b670bac6d394b180331854ae5648482`. A 49 MiB developer runtime containing the 23-family shared-library closure verifies with manifest SHA-256 `9532bbc04e756469db3d578b95c6a59c9033327c26aeda3e4121fcea698bd2e1`, and the configured host smoke passes against it. GitHub Actions run 31342724249 passes the default code-quality matrix on macOS, Ubuntu, and Windows at runtime-assembler implementation commit `75bc415`; this does not add three-OS native OCCT construction/launch evidence
- **Test status:** The current local macOS workspace passes 137 runtime tests and one compile-fail doctest. Strict workspace/native-host/WASM Clippy, eleven Python native-tooling tests, 142-document planning validation, fixture hashes, offline frontend build, standalone runtime verification, and the explicit configured-native GUI-5 smoke all pass. The smoke still passes after the ordinary feature-off workspace build overwrites the shared Cargo worker output, proving the dedicated runtime remains intact. Run 31342724249 passes formatting, all eleven portable tooling tests, workspace/native-host/WASM lint, all-target tests, and doctests on macOS, Ubuntu, and Windows. The actual app's prior keyboard/semantic/cancellation/failure/recovery evidence and 26 optional-native adapter/worker tests remain valid. Full screen-reader/contrast/scaling/HiDPI and three-platform native/package suites remain pending
- **Active technical debt:** The GUI has only ephemeral developer-session rate entry, not guided governance or persistence; pinned ISO 4217 registry validation, nonterminating rational-division rounding, rounding increments beyond decimal scale, and graph-wide evaluation/migration remain pending; canonical JSON remains an internal spike contract; supported STEP/STL/3MF corpora and representative machining fixtures remain pending
- **Active blockers:** No shop rates block TASK-002 mechanics because production libraries start empty and tests are synthetic; TASK-007/M0.2 still requires real category/policy/calibration review, and ADR acceptance plus broader production work still require shop/security/legal inputs and representative fixtures
- **Open architecture decisions:** ADR-0001 through ADR-0014 are In Review
- **Supported model formats:** None supported for production. An optional Apple Silicon STEP/OCCT spike handles governed synthetic evidence only. First-class plan: STEP, STL, 3MF; experimental plan: IGES, OBJ
- **Geometry capabilities:** Kernel-neutral contracts, bounded control schema v2, asset-transport manifest v2, explicit verified-private-copy selection, exact Unix descriptor and Windows HANDLE direct delivery with unrelated-resource exclusion proof, worker-side identity/type/hash/length/quota verification, immutable byte-stream adapter input, cooperative acknowledgement with forced termination, deny-by-default authorization/audit seams, root-identity-bound capability resolution, a consumed already-open grant, private per-job staging, immutable output claiming, governed storage ports, and a mandatory worker resource profile are implemented. GUI-4 intake can now compute a size-bounded SHA-256 fingerprint only after authorization/audit and rewind the same open grant for worker consumption; worker verification remains independent. The provisional native output has a validated schema and decoder that binds reference and source hash. Remaining containment and production gaps are unchanged
- **Runtime capabilities:** None implemented. Six estimation levels and validation structure documented only
- **Advanced capabilities:** None implemented. Route optimization, capacity/opportunity, uncertainty, revision/PMI/coverage, feature allocations, learning, CAM reconciliation, availability, sourcing, and bid priority are planning artifacts only
- **Current fixtures:** `FIX-MESH-001/002` cover closed/open synthetic STL cubes; `FIX-STEP-001` is a deterministic OCCT-generated AP214 10 mm cube; `FIX-STEP-002` contains an intentionally invalid entity; `FIX-STEP-003` is a manually authored AP214 faceted rectangular prism with independent analytic 12 × 8 × 5 mm ground truth, 392 mm² area, 480 mm³ volume, and `(6, 4, 2.5)` mm centroid. All hashes reproduce and native measurements pass; formal geometry/security review, broader CAD-tool independence, and licensing remain open
- **Calculation capabilities:** CALC-001/003/005 and CALC-007–018 foundations, with CALC-009 limited to exact division; empty configurable rate cards; scoped/effective/approved resolution; unavailable/blocked ambiguity states; versioned pricing/rounding; synthetic EX-01/03/12 exact itemized traces; pinned replay; typed money/currency/units; graph validation; value states; provenance/version wrappers; canonical snapshots; and a headless session-only application composition over reviewed provisional geometry/manual inputs/rates/policy are implemented as a reversible spike
- **Nearest testable GUI:** It exists now as the bounded GUI-5 Apple-Silicon developer slice: local STEP selection, real OCCT geometry facts, explicit review/manual/rate/pricing entry, and a deterministic trace run in the actual unsigned app. The worker and OCCT closure can now be kept together in one verified developer runtime, but it still is not a convenient installable product: the app and runtime remain separate explicit paths, data disappears on exit, only governed synthetic STEP evidence is validated, and there is no 3D viewport. A visible viewer still adds roughly two or three checkpoints
- **Files likely next:** GUI-1 review evidence; TASK-003 Windows/Linux native construction and runtime assembly, application embedding/signing, dynamic-link/SBOM/legal evidence, network/filesystem and remaining resource containment; TASK-004 mesh imports; TASK-005 durable rate/model-review UX; and TASK-006 persistence. The verified runbook and evidence are in `apps/estimator-desktop/README.md`, `06-quality/task-003-validation.md`, and `06-quality/gui-5-validation.md`

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
python3 scripts/assemble_native_runtime.py assemble --occt-root /approved/local/occt-install --worker /approved/local/partprobe-geometry-worker --build-manifest /approved/local/occt-build/partprobe-occt-build-manifest.json --output /approved/local/partprobe-native-runtime
python3 scripts/assemble_native_runtime.py verify --runtime-root /approved/local/partprobe-native-runtime
PARTPROBE_GEOMETRY_WORKER="$PWD/target/debug/partprobe-geometry-worker" PARTPROBE_GEOMETRY_WORKSPACE=/private/tmp/partprobe-gui-worker PARTPROBE_OCCT_ROOT=/private/tmp/partprobe-occt-install cargo test -p partprobe-estimator-desktop --features desktop-host gui5_configured_worker_runs_real_step_through_retained_estimate_session --locked -- --ignored --nocapture
git diff --check
git status --short --branch
```

## Read these first

1. [AGENTS.md](../AGENTS.md)
2. [Documentation index](INDEX.md)
3. [Decision log](08-decisions/decision-log.md)
4. [Requirements matrix](03-requirements/requirements-matrix.md)
5. The requirement, ADR, architecture, domain, UX, and quality files relevant to the task
