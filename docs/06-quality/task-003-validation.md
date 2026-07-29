# TASK-003 Validation Evidence

> **Status:** In Review
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-F-002–REQ-F-004, REQ-F-021–REQ-F-022; REQ-NF-002, REQ-NF-004–REQ-NF-005, REQ-NF-011–REQ-NF-014; GEO-001–GEO-007; TEST-003, TEST-021–TEST-030
> **Related ADRs:** ADR-0002, ADR-0005, ADR-0008
> **Open questions:** Reproducible three-OS OCCT builds, target-specific sandbox and resource enforcement, independent analytic STEP corpus, and distribution approval
> **Dependencies:** TASK-002 integration; OCCT 8.0.0 is selected for the spike but not approved for distribution
> **Supersedes:** None

## Implemented contract evidence

- Added `geometry-core` as a kernel-neutral crate for canonical source hashes, model formats, representation basis, model units, analysis profiles, pipeline stages, stage states, sanitized warnings, immutable source descriptors, and validated stage reports.
- Added `geometry-import` as a path-free, schema-versioned worker protocol using opaque asset capabilities, expected source hashes, ordered unique stages, analysis-profile versions, and strictly positive input/output/entity/time quotas.
- Added a local subprocess supervisor that clears the inherited environment, sets a controlled working directory, bounds stdin/stdout JSON, polls cancellation and wall time, kills and reaps failed workers, verifies response schema/job/correlation identity, and maps launch, I/O, exit, timeout, quota, malformed-response, and cancellation failures to sanitized responses.
- Added a minimal `geometry-worker` executable that proves the process boundary and returns the explicit terminal diagnostic `NATIVE_ADAPTER_UNAVAILABLE`; it does not parse CAD or claim OCCT capability.
- Added fixture expectation schema version 2 with exact decimal strings and typed `available`, `unavailable`, and `not_applicable` evidence. Closed/open mesh fixtures now distinguish authoritative volume from unavailable open-boundary volume.
- Selected official OCCT 8.0.0 commit `d3056ef80c9668f395da40f5fd7be186cae4501f` for the reversible native spike and completed a minimal shared-library source build on Apple Silicon with no external CMake dependency enabled.
- Added an optional `geometry-occt-adapter` C ABI boundary. Default builds do not link OCCT; the explicit native feature requires `PARTPROBE_OCCT_ROOT`, dynamically links shared libraries, catches all C++ exceptions, checks ABI/result bounds, and exposes stable diagnostics without native exception text or paths.
- Added one-job asset staging: open-handle regular-file validation, create-new fixed destination, streaming input quota, SHA-256 verification, read-only worker-local bytes, sanitized mismatch/staging/cleanup failures, and post-worker input removal.
- Replaced the supervisor's source-path API with a consumed `AssetReadGrant` bound to the request capability and an already-open regular file. Staging rewinds the same handle, verifies authorized length plus hash/quota, drops it before launch, and rejects capability mismatch or post-grant length drift.
- Added a cross-platform read-only local-source opener that rejects a linked final component with Unix `O_NOFOLLOW` or a Windows open-reparse-point plus attribute check and identification-only security quality of service. It returns only the one-use open-handle grant; application-service path authorization and parent containment remain separate.
- Added deterministic synthetic `FIX-STEP-001` generation, manifest hash, schema-v2 expected evidence, and analytic area/volume/centroid checks. Its same-kernel generation/read limitation is documented.
- Added project-authored adversarial `FIX-STEP-002` and a separate schema-v1 failure expectation. The optional adapter and supervised worker return sanitized recoverable `STEP_TRANSFER_FAILED`, create no snapshot/output, and remove the staged input.
- Connected the optional worker to OCCT for the exact six-stage spike profile. The subprocess writes a bounded schema-v1 `provisional_spike` snapshot and returns only an opaque reference; measurements use documented six-decimal canonicalization and remain non-authoritative.
- Custom deserialization revalidates source hashes, byte counts, stage ordering, warning consistency, IDs, and quotas.
- Default builds add no OCCT runtime or C++ linkage. The optional native feature and SHA-256 staging dependency remain isolated, while an operating-system sandbox is not yet implemented.

## Fixture schema migration

Fixture expectation schema version 2 replaces ambiguous nullable/omitted measurements with explicit evidence states and exact decimal strings. This is a preproduction, test-fixture-only schema: no customer or persisted production record is migrated. Version 1 successful-geometry files fail validation and must be deliberately upgraded with reviewed evidence; the loader does not infer unavailable data as zero or silently reinterpret old files. The additive import-failure expectation schema starts at version 1 and carries only controlled failure/artifact outcomes; it does not migrate or reinterpret successful-geometry records, and unsupported versions fail validation.

## Local acceptance run

Environment: Apple Silicon macOS; Rust 1.94.1 workspace.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Pass |
| `cargo test --workspace --all-targets --locked` | Pass: 64 runtime tests |
| `cargo test --workspace --doc --locked` | Pass: one compile-fail unit-safety doctest |
| `PARTPROBE_OCCT_ROOT=… cargo test -p partprobe-geometry-occt-adapter --features fixture-tools` | Pass: six native-link/ABI/failure/measurement/generator-reproduction tests |
| `PARTPROBE_OCCT_ROOT=… cargo test -p partprobe-geometry-worker --features native-occt --test process_boundary` | Pass: six grant/staging/hash/measurement/invalid-entity process tests |

TASK-003 currently adds twenty-three default runtime tests: five geometry-core invariants, seven protocol/supervisor cases, six subprocess-boundary cases, four fixture-contract cases, and one default-disabled adapter case. Twelve focused tests pass across the explicit local native adapter/worker commands.

## Cross-platform evidence

GitHub Actions run 30479210538 passes formatting, strict Clippy, all 64 default runtime tests, and documentation tests on Windows, Linux, and macOS for commit `a9b57f2`. This proves the kernel-neutral contracts, bounded subprocess transport, one-use open source grants, capability/length/hash checks, deleted-path operation, fixed-name staging, fixture schemas, locked Rust dependencies, feature-off behavior, and target-specific final-component link/reparse-point rejection are cross-platform. It does not prove application-service path authorization or parent containment, direct worker handle transport, native OCCT linking, sandbox, or packaging behavior; the focused native-feature evidence is Apple Silicon only.

## Remaining acceptance evidence

- Replace the current polling hard-kill behavior with a documented cooperative cancellation grace protocol where native work can acknowledge cancellation.
- Implement application-service authorization and parent-component containment around the final-component-safe opener; define controlled output ownership.
- Add OS-specific network denial, filesystem sandboxing, CPU/memory limits, descendant-process containment, and cleanup/retention evidence. Clearing the environment and controlling the working directory are defense-in-depth, not a sandbox.
- Complete legal review of OCCT 8.0.0 notices, source offer/relinking approach, shared-library packaging, and third-party/transitive native inventory before distribution.
- Add reproducible OCCT 8.0.0 build automation and artifact fingerprints for Windows, Linux, and macOS; the current native source-build evidence is Apple Silicon only.
- Pass the granted descriptor/handle directly into the sandboxed worker where practical, retain a documented verified-copy fallback, and add OS sandbox, no-network, CPU/memory/descendant limits, and output cleanup/retention enforcement.
- Add independently authored analytic STEP plus broader malformed, alternate-schema, assembly, partial-transfer, and resource-limit fixtures before treating the same-kernel cube as accuracy evidence.
- Add legally redistributable analytic STEP fixtures with reviewed hashes, units, exact measurements, transfer/validity expectations, tolerances, and malformed cases.
- Build the OCCT adapter and worker on Windows, Linux, and macOS; record dependency fingerprints, package artifacts, elapsed/peak resources, deterministic reruns, and crash containment.
- Do not mark TASK-003 Complete or ADR-0002/0005 Accepted until accuracy, containment, packaging, and legal evidence pass.
