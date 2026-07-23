# ADR-0007 — Calculation Representation and Versioning

> **Status:** In Review  
> **Last updated:** 2026-07-23  
> **Related requirements:** REQ-F-005–REQ-F-010; REQ-NF-003, REQ-NF-009; CALC-001–CALC-020  
> **Related ADRs:** ADR-0006, ADR-0008  
> **Open questions:** Decimal scale/rounding policy approval  
> **Dependencies:** Calculation spike  
> **Supersedes:** None

## Decision proposed

Represent estimates as typed acyclic calculation graphs. Use fixed-precision decimal for authoritative currency, explicit dimensional quantities, immutable published results, semantic rule versions, and complete input/intermediate/output traces.

`rust_decimal` is the initial spike candidate: its documentation describes a fixed-precision decimal representation intended for financial calculations, with bounded scale/precision ([crate documentation](https://docs.rs/rust_decimal/latest/rust_decimal/)). Adoption depends on overflow, serialization, scale, and cross-platform golden tests.

## Consequences

More storage and explicit rule design are accepted in exchange for replay, auditability, targeted recalculation, and migration safety. Rule behavior cannot change in place; old evaluators/results remain replayable for the supported retention policy.

## Acceptance gate

Worked examples reconcile exactly; cycle and unit mismatch are rejected; money golden/property tests pass on three OSes; serialization is canonical; rule upgrade leaves approved results unchanged.

## Spike evidence

TASK-001 implemented the proposed boundary with `rust_decimal 1.42.1`, project-owned typed units, validated deserialization, a typed DAG validator, provenance/version wrappers, exact arithmetic with rounding-required failures, and normalized deterministic snapshot JSON with intermediate traces. Formatting, strict Clippy, 23 runtime tests, and one compile-fail doctest pass locally and in the Windows/Linux/macOS matrix; see [the evidence record](../../06-quality/task-001-validation.md). ADR-0007 remains **In Review** because reviewed worked estimates, the general decimal scale/rounding policy, rule-upgrade replay, and organization-level dependency review are still open.
