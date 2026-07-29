# TASK-003 Validation Evidence

> **Status:** In Review
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-F-002–REQ-F-004, REQ-F-021–REQ-F-022; REQ-NF-002, REQ-NF-004–REQ-NF-005, REQ-NF-011–REQ-NF-014; GEO-001–GEO-007; TEST-003, TEST-021–TEST-030
> **Related ADRs:** ADR-0002, ADR-0005, ADR-0008
> **Open questions:** OCCT build/version, target-specific sandbox and resource enforcement, analytic STEP corpus, and distribution approval
> **Dependencies:** TASK-002 integration; OCCT dependency evidence is not yet approved or added
> **Supersedes:** None

## Implemented contract evidence

- Added `geometry-core` as a kernel-neutral crate for canonical source hashes, model formats, representation basis, model units, analysis profiles, pipeline stages, stage states, sanitized warnings, immutable source descriptors, and validated stage reports.
- Added `geometry-import` as a path-free, schema-versioned worker protocol using opaque asset capabilities, expected source hashes, ordered unique stages, analysis-profile versions, and strictly positive input/output/entity/time quotas.
- Added a local subprocess supervisor that clears the inherited environment, sets a controlled working directory, bounds stdin/stdout JSON, polls cancellation and wall time, kills and reaps failed workers, verifies response schema/job/correlation identity, and maps launch, I/O, exit, timeout, quota, malformed-response, and cancellation failures to sanitized responses.
- Added a minimal `geometry-worker` executable that proves the process boundary and returns the explicit terminal diagnostic `NATIVE_ADAPTER_UNAVAILABLE`; it does not parse CAD or claim OCCT capability.
- Added fixture expectation schema version 2 with exact decimal strings and typed `available`, `unavailable`, and `not_applicable` evidence. Closed/open mesh fixtures now distinguish authoritative volume from unavailable open-boundary volume.
- Custom deserialization revalidates source hashes, byte counts, stage ordering, warning consistency, IDs, and quotas.
- No OCCT, C++, FFI, hashing, operating-system sandbox, or new third-party dependency was added. Existing Serde, Serde JSON, and project-owned domain types are reused.

## Fixture schema migration

Fixture expectation schema version 2 replaces ambiguous nullable/omitted measurements with explicit evidence states and exact decimal strings. This is a preproduction, test-fixture-only schema: no customer or persisted production record is migrated. Version 1 expected files fail validation and must be deliberately upgraded with reviewed evidence; the loader does not infer unavailable data as zero or silently reinterpret old files.

## Local acceptance run

Environment: Apple Silicon macOS; Rust 1.94.1 workspace.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Pass |
| `cargo test --workspace --all-targets --locked` | Pass: 55 runtime tests |
| `cargo test --workspace --doc --locked` | Pass: one compile-fail unit-safety doctest |

TASK-003 currently adds fourteen runtime tests: five geometry-core invariants, six worker-protocol/supervisor cases, one real subprocess-boundary case, and two fixture-contract cases.

## Cross-platform evidence

GitHub Actions run 30463247931 passes formatting, strict Clippy, all 51 runtime tests, and documentation tests on Windows, Linux, and macOS for commit `fa08e53`. This proves the initial kernel-neutral contracts are cross-platform. The current 55-test subprocess/fixture slice still requires its own three-OS run; neither run proves native sandbox, OCCT, or packaging behavior.

## Remaining acceptance evidence

- Replace the current polling hard-kill behavior with a documented cooperative cancellation grace protocol where native work can acknowledge cancellation.
- Add an asset-capability resolver that grants only the requested source and controlled output locations; the current worker receives no source asset.
- Add OS-specific network denial, filesystem sandboxing, CPU/memory limits, descendant-process containment, and cleanup/retention evidence. Clearing the environment and controlling the working directory are defense-in-depth, not a sandbox.
- Select and document an exact OCCT source/build/version only after maintenance, license, transitive-native, security, packaging, update-owner, and removal-path review.
- Add legally redistributable analytic STEP fixtures with reviewed hashes, units, exact measurements, transfer/validity expectations, tolerances, and malformed cases.
- Build the OCCT adapter and worker on Windows, Linux, and macOS; record dependency fingerprints, package artifacts, elapsed/peak resources, deterministic reruns, and crash containment.
- Do not mark TASK-003 Complete or ADR-0002/0005 Accepted until accuracy, containment, packaging, and legal evidence pass.
