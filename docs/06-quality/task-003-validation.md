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
- Selected official OCCT 8.0.0 commit `d3056ef80c9668f395da40f5fd7be186cae4501f` for the reversible native spike and completed a minimal shared-library source build on Apple Silicon with no external CMake dependency enabled.
- Added an optional `geometry-occt-adapter` C ABI boundary. Default builds do not link OCCT; the explicit native feature requires `PARTPROBE_OCCT_ROOT`, dynamically links shared libraries, catches all C++ exceptions, checks ABI/result bounds, and exposes stable diagnostics without native exception text or paths.
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
| `cargo test --workspace --all-targets --locked` | Pass: 56 runtime tests |
| `cargo test --workspace --doc --locked` | Pass: one compile-fail unit-safety doctest |
| `PARTPROBE_OCCT_ROOT=… cargo test -p partprobe-geometry-occt-adapter --features native-occt` | Pass: three focused native-link/ABI/failure-sanitization tests |

TASK-003 currently adds fifteen default runtime tests: five geometry-core invariants, six worker-protocol/supervisor cases, one real subprocess-boundary case, two fixture-contract cases, and one default-disabled adapter case. The explicit local native feature adds two more focused cases.

## Cross-platform evidence

GitHub Actions run 30467602461 passes formatting, strict Clippy, all 56 default runtime tests, and documentation tests on Windows, Linux, and macOS for commit `c31fafb`. This proves the kernel-neutral contracts, bounded subprocess transport, fixture schema, optional adapter crate/build script, locked `cc` toolchain, and feature-off behavior are cross-platform. It does not prove native OCCT linking, sandbox, or packaging behavior; the focused native-feature evidence is Apple Silicon only.

## Remaining acceptance evidence

- Replace the current polling hard-kill behavior with a documented cooperative cancellation grace protocol where native work can acknowledge cancellation.
- Add an asset-capability resolver that grants only the requested source and controlled output locations; the current worker receives no source asset.
- Add OS-specific network denial, filesystem sandboxing, CPU/memory limits, descendant-process containment, and cleanup/retention evidence. Clearing the environment and controlling the working directory are defense-in-depth, not a sandbox.
- Complete legal review of OCCT 8.0.0 notices, source offer/relinking approach, shared-library packaging, and third-party/transitive native inventory before distribution.
- Add reproducible OCCT 8.0.0 build automation and artifact fingerprints for Windows, Linux, and macOS; the current native source-build evidence is Apple Silicon only.
- Connect the optional adapter only after the worker resolves an opaque asset capability to a fixed worker-local source; never add source paths to the IPC contract.
- Add legally redistributable analytic STEP fixtures with reviewed hashes, units, exact measurements, transfer/validity expectations, tolerances, and malformed cases.
- Build the OCCT adapter and worker on Windows, Linux, and macOS; record dependency fingerprints, package artifacts, elapsed/peak resources, deterministic reruns, and crash containment.
- Do not mark TASK-003 Complete or ADR-0002/0005 Accepted until accuracy, containment, packaging, and legal evidence pass.
