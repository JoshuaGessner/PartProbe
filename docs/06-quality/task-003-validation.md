# TASK-003 Validation Evidence

> **Status:** In Review
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-F-002–REQ-F-004, REQ-F-021–REQ-F-022; REQ-NF-002, REQ-NF-004–REQ-NF-005, REQ-NF-011–REQ-NF-014; GEO-001–GEO-007; TEST-003, TEST-021–TEST-030
> **Related ADRs:** ADR-0002, ADR-0005, ADR-0008
> **Open questions:** OCCT build/version, IPC transport, target-specific sandbox, resource budgets, analytic STEP corpus, and distribution approval
> **Dependencies:** TASK-002 integration; OCCT dependency evidence is not yet approved or added
> **Supersedes:** None

## Implemented contract evidence

- Added `geometry-core` as a kernel-neutral crate for canonical source hashes, model formats, representation basis, model units, analysis profiles, pipeline stages, stage states, sanitized warnings, immutable source descriptors, and validated stage reports.
- Added `geometry-import` as a path-free, schema-versioned worker protocol using opaque asset capabilities, expected source hashes, ordered unique stages, analysis-profile versions, and strictly positive input/output/entity/time quotas.
- Worker exit, timeout, quota breach, malformed response, and cancellation map to sanitized `FailedRecoverable` responses; no raw CAD content, local paths, or geometry names are present in the protocol.
- Custom deserialization revalidates source hashes, byte counts, stage ordering, warning consistency, IDs, and quotas.
- No OCCT, C++, FFI, hashing, IPC, process-control, or sandbox dependency was added. Existing Serde and project-owned domain types are reused.

## Local acceptance run

Environment: Apple Silicon macOS; Rust 1.94.1 workspace.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Pass |
| `cargo test --workspace --all-targets --locked` | Pass: 51 runtime tests |
| `cargo test --workspace --doc --locked` | Pass: one compile-fail unit-safety doctest |

TASK-003 currently adds ten runtime contract tests: five geometry-core invariants and five worker-protocol/supervisor cases.

## Remaining acceptance evidence

- Implement the actual local IPC transport, supervisor process lifecycle, cancellation grace period, and OS-specific sandbox/resource controls.
- Select and document an exact OCCT source/build/version only after maintenance, license, transitive-native, security, packaging, update-owner, and removal-path review.
- Add legally redistributable analytic STEP fixtures with reviewed hashes, units, exact measurements, transfer/validity expectations, tolerances, and malformed cases.
- Build the OCCT adapter and worker on Windows, Linux, and macOS; record dependency fingerprints, package artifacts, elapsed/peak resources, deterministic reruns, and crash containment.
- Do not mark TASK-003 Complete or ADR-0002/0005 Accepted until accuracy, containment, packaging, and legal evidence pass.
