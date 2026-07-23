# TASK-001 Validation Evidence

> **Status:** In Review  
> **Last updated:** 2026-07-23  
> **Related requirements:** REQ-NF-001, REQ-NF-003, REQ-NF-004, REQ-NF-010; CALC-001, CALC-003, CALC-005, CALC-016, CALC-017; TEST-001  
> **Related ADRs:** ADR-0007  
> **Open questions:** First Windows and Linux CI run; approved decimal scale/rounding policy  
> **Dependencies:** Rust 1.94.1, Cargo.lock, GitHub Actions runner access  
> **Supersedes:** None

## Implemented evidence

- Cargo resolver-v3 workspace with `partprobe-domain`, `partprobe-estimation-engine`, and `partprobe-test-support`.
- Compile-time-distinct volume, density, mass, item quantity, money, and currency types.
- Exact decimal arithmetic for implemented physical and monetary rules, currency mismatch rejection, overflow/error boundaries, normalized decimal-as-string serialization, and a typed rounding-required result when an operation cannot be represented exactly.
- Rule/source/schema version wrappers, provenance categories, and explicit available/unavailable/blocked/unknown/stale states.
- CALC-001, CALC-003, CALC-005, CALC-016, and CALC-017 with exact or exhaustive bounded tests.
- Typed graph definitions that reject duplicates, missing dependencies, dimensional/currency type mismatches, and cycles before evaluation.
- Deterministic topological ordering and compact canonical snapshot JSON using ordered maps, fixed field order, normalized decimal strings, inputs/intermediate/output traces, schema/rule versions, and the pinned serializer.
- Validated deserialization paths prevent persisted data from bypassing currency, non-negative quantity, nonempty ID/source, schema-version, and value-type invariants.
- Three-OS CI definition for formatting, strict Clippy, all-target tests, and doctests.

## Local acceptance run

Environment: Apple Silicon macOS; `rustc 1.94.1 (e408947bf 2026-03-25)`; Cargo 1.94.1.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| `cargo test --workspace --all-targets --locked` | Pass: 23 runtime tests |
| `cargo test --workspace --doc --locked` | Pass: one compile-fail unit-safety doctest |
| `python3 scripts/check_planning.py` | Must pass at session close |
| `python3 scripts/hash_fixtures.py` | Must reproduce both fixture hashes at session close |

TEST-001 evidence includes exact money addition and pricing, currency mismatch, normalized decimal serialization, deserialization-invariant rejection, negative physical-value rejection, mass, removed-volume and make-quantity properties, physical and monetary implicit-precision-loss rejection, quantity overflow, invalid pricing boundaries, missing dependency/type/cycle rejection, deterministic graph order, and an exact canonical JSON golden with an intermediate trace.

## Remaining evidence and limits

- The GitHub Actions matrix is defined but has not run because this local repository has no configured remote. Windows and Linux results therefore remain pending; REQ-NF-001 and ADR-0007 are not Complete/Accepted.
- TASK-002 must reconcile reviewed worked estimates and settle decimal internal scale, presentation rounding, overflow, and historical replay policy before this spike API becomes a production contract.
- Canonical JSON is currently an internal, versioned spike representation—not an adopted external interchange standard.
- No UI, persistence, geometry, runtime estimator, production routing, or advanced-analysis engine was implemented.
