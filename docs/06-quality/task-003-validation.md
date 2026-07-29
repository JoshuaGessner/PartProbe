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
- Replaced the one-shot raw request body with control-stream schema version 1: one bounded newline-framed `execute` message followed by at most one identity-bound user/deadline `cancel` message. The supervisor requires an explicit grace interval, validates the matching worker acknowledgement, preserves valid completion-wins races, and force-kills/reaps after grace with distinct cancellation/deadline diagnostics.
- Added safe worker cancellation checkpoints before native work, after the blocking OCCT adapter returns, and around output creation. This permits cooperative acknowledgement whenever Rust regains control; it does not claim cancellation inside the current blocking OCCT call, which remains bounded by supervised termination.
- Added a minimal `geometry-worker` executable that proves the process boundary and returns the explicit terminal diagnostic `NATIVE_ADAPTER_UNAVAILABLE`; it does not parse CAD or claim OCCT capability.
- Added fixture expectation schema version 2 with exact decimal strings and typed `available`, `unavailable`, and `not_applicable` evidence. Closed/open mesh fixtures now distinguish authoritative volume from unavailable open-boundary volume.
- Selected official OCCT 8.0.0 commit `d3056ef80c9668f395da40f5fd7be186cae4501f` for the reversible native spike and completed a minimal shared-library source build on Apple Silicon with no external CMake dependency enabled.
- Added an optional `geometry-occt-adapter` C ABI boundary. Default builds do not link OCCT; the explicit native feature requires `PARTPROBE_OCCT_ROOT`, dynamically links shared libraries, catches all C++ exceptions, checks ABI/result bounds, and exposes stable diagnostics without native exception text or paths.
- Added one-job asset staging: open-handle regular-file validation, create-new fixed destination, streaming input quota, SHA-256 verification, read-only worker-local bytes, sanitized mismatch/staging/cleanup failures, and post-worker input removal.
- Replaced the supervisor's source-path API with a consumed `AssetReadGrant` bound to the request capability and an already-open regular file. Staging rewinds the same handle, verifies authorized length plus hash/quota, drops it before launch, and rejects capability mismatch or post-grant length drift.
- Added a cross-platform read-only local-source opener that rejects a linked final component with Unix `O_NOFOLLOW` or a Windows open-reparse-point plus attribute check and identification-only security quality of service. It returns only the one-use open-handle grant; application-service path authorization and parent containment remain separate.
- Added `LocalAssetRoot`, which opens an application-selected directory capability once, accepts only normalized relative paths, prevents parent traversal and absolute-path access, rejects parent-symlink escape from the root, and combines resolution with final-component no-follow. The type explicitly does not make actor/project/classification authorization decisions.
- Added headless `security` and `application` crates for the local asset-read decision point. Policy input explicitly carries actor, project, record/version, deployment-defined classification and record state, protected operation, root identity, correlation, and trusted application time without a path, capability token, or file content.
- Added versioned allow/deny results, bounded content-free reason codes, an explicit deny-all baseline, and an append-only audit port. The service records the decision before filesystem resolution and fails closed when audit append fails; it does not hard-code roles or provide an unreviewed allow policy.
- Bound each `LocalAssetRoot` to a stable `AssetRootId`, preventing the application from authorizing one logical root while passing another open directory capability to the resolver.
- Added atomically created private per-job directories, owner-only Unix mode, inherited-root Windows ACL behavior, fixed-name isolation inside that directory, and deterministic input/output/directory cleanup diagnostics.
- Added controlled referenced-output claiming: no-follow regular-file validation, nonempty and quota/length bounds, immutable supervisor-owned bytes, SHA-256 and byte-length evidence bound to the opaque snapshot reference, and removal of the worker-visible pathname. Unreferenced worker output is discarded; response status remains distinct from artifact presence.
- Added `document-storage` as a persistence-neutral controlled-derivative contract. Its immutable manifest retains artifact/source record and version lineage, the producing snapshot reference, inherited classification, exact access and retention policy references, authorization/audit correlation, actor/time, versioned payload schema, declared media type, independently recomputed SHA-256/length, explicit verified integrity state, and an opaque adapter-owned locator.
- Added an application persistence service that independently revalidates claimed worker bytes, assembles the governed manifest, delegates through a controlled-store port, and rejects a receipt that does not exactly match the requested manifest. Integrity and store failures preserve the original output or complete governed write for explicit retry, quarantine, or disposition.
- Added no filesystem or database adapter. ADR-0006 and TASK-006 still own the durable schema, ordered migrations, atomic/fsync behavior, encryption decision, optimistic concurrency, integrity-on-open, backup/restore, and controlled retention/disposition evidence.
- Added deterministic synthetic `FIX-STEP-001` generation, manifest hash, schema-v2 expected evidence, and analytic area/volume/centroid checks. Its same-kernel generation/read limitation is documented.
- Added project-authored adversarial `FIX-STEP-002` and a separate schema-v1 failure expectation. The optional adapter and supervised worker return sanitized recoverable `STEP_TRANSFER_FAILED`, create no snapshot/output, and remove the staged input.
- Connected the optional worker to OCCT for the exact six-stage spike profile. The subprocess writes a bounded schema-v1 `provisional_spike` snapshot and returns only an opaque reference; measurements use documented six-decimal canonicalization and remain non-authoritative.
- Custom deserialization revalidates source hashes, byte counts, stage ordering, warning consistency, IDs, and quotas.
- Default builds add no OCCT runtime or C++ linkage. The optional native feature and SHA-256 staging dependency remain isolated, while an operating-system sandbox is not yet implemented.

## Fixture schema migration

Fixture expectation schema version 2 replaces ambiguous nullable/omitted measurements with explicit evidence states and exact decimal strings. This is a preproduction, test-fixture-only schema: no customer or persisted production record is migrated. Version 1 successful-geometry files fail validation and must be deliberately upgraded with reviewed evidence; the loader does not infer unavailable data as zero or silently reinterpret old files. The additive import-failure expectation schema starts at version 1 and carries only controlled failure/artifact outcomes; it does not migrate or reinterpret successful-geometry records, and unsupported versions fail validation. Worker control-stream schema version 1 replaces the preproduction one-shot raw request JSON; old supervisors/workers are intentionally incompatible and fail closed, so deployment must update both signed components together. The `GeometryWorkerExecution`, `LocalAssetRoot`, authorization context/decision, audit event, controlled derivative write, and store receipt are preproduction in-memory APIs; the new access/storage IDs are additive validated primitives but are not used by a persisted record schema. There are no customer records to migrate. Durable authorization, audit, manifest, and locator schemas must start with explicit versions and migration/replay policies rather than treating these Rust layouts as storage contracts.

## Local acceptance run

Environment: Apple Silicon macOS; Rust 1.94.1 workspace.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Pass |
| `cargo test --workspace --all-targets --locked` | Pass: 88 runtime tests |
| `cargo test --workspace --doc --locked` | Pass: one compile-fail unit-safety doctest |
| `PARTPROBE_OCCT_ROOT=… cargo test -p partprobe-geometry-occt-adapter --features fixture-tools` | Pass: six native-link/ABI/failure/measurement/generator-reproduction tests |
| `PARTPROBE_OCCT_ROOT=… cargo test -p partprobe-geometry-worker --features native-occt --test process_boundary` | Pass: eleven grant/staging/hash/cancellation/measurement/invalid-entity process tests |

TASK-003 currently adds forty-seven default runtime tests: four access/security invariants, three application authorization/audit cases, four application derivative-persistence cases, four document-storage contract cases, five geometry-core invariants, eleven protocol/supervisor/output/containment cases, eleven subprocess-boundary cases, four fixture-contract cases, and one default-disabled adapter case. Seventeen focused tests pass across the explicit local native adapter/worker commands.

## Cross-platform evidence

GitHub Actions run 30490605289 passes formatting, strict Clippy, all 81 default runtime tests, and documentation tests on Windows, Linux, and macOS for commit `b45f159`. This proves the kernel-neutral contracts, bounded subprocess transport, one-use open source grants, capability/length/hash checks, deleted-path operation, fixed-name staging, fixture schemas, locked Rust dependencies, feature-off behavior, capability-root traversal/absolute-path/parent-symlink-escape/final-link rejection, root-identity binding, versioned deny-all decisions, content-minimized decision audit, audit-unavailable fail-closed behavior, private job-directory creation/cleanup, immutable output claim/hash/unlink behavior, independently verified governed derivative writes, failure-byte preservation, and exact receipt validation are cross-platform. Existing evidence does not prove a deployment-specific role/project/classification policy, identity/session enforcement, durable audit persistence, direct worker handle transport, native OCCT linking, OS sandbox, a durable controlled-store adapter, retention enforcement, or packaging behavior; the focused native-feature evidence is Apple Silicon only.

## Remaining acceptance evidence

- Add an OCCT progress/cancellation hook where supported so a blocking native operation can observe the validated control request before the supervisor's grace expires; retain forced termination as the bounded fallback.
- Implement and approve deployment-specific role/project-membership/classification/record-state policy and repositories plus durable append-preserving audit persistence; implement the controlled-store adapter with versioned schema/migrations, atomic durability, integrity-on-open, backup/restore, encryption decision, and retention/disposition enforcement.
- Add OS-specific network denial, filesystem sandboxing, CPU/memory limits, descendant-process containment, and cleanup/retention evidence. Clearing the environment and controlling the working directory are defense-in-depth, not a sandbox.
- Complete legal review of OCCT 8.0.0 notices, source offer/relinking approach, shared-library packaging, and third-party/transitive native inventory before distribution.
- Add reproducible OCCT 8.0.0 build automation and artifact fingerprints for Windows, Linux, and macOS; the current native source-build evidence is Apple Silicon only.
- Pass the granted descriptor/handle directly into the sandboxed worker where practical, retain a documented verified-copy fallback, and add OS sandbox, no-network, CPU/memory/descendant limits, and durable-output retention/disposition enforcement.
- Add independently authored analytic STEP plus broader malformed, alternate-schema, assembly, partial-transfer, and resource-limit fixtures before treating the same-kernel cube as accuracy evidence.
- Add legally redistributable analytic STEP fixtures with reviewed hashes, units, exact measurements, transfer/validity expectations, tolerances, and malformed cases.
- Build the OCCT adapter and worker on Windows, Linux, and macOS; record dependency fingerprints, package artifacts, elapsed/peak resources, deterministic reruns, and crash containment.
- Do not mark TASK-003 Complete or ADR-0002/0005 Accepted until accuracy, containment, packaging, and legal evidence pass.
