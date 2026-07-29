# ADR-0007 — Calculation Representation and Versioning

> **Status:** In Review
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-F-005–REQ-F-010; REQ-NF-003, REQ-NF-009; CALC-001–CALC-020
> **Related ADRs:** ADR-0006, ADR-0008
> **Open questions:** Organization-specific rounding/pricing calibration and production contract review
> **Dependencies:** Calculation spike
> **Supersedes:** None

## Decision proposed

Represent estimates as typed acyclic calculation graphs. Use fixed-precision decimal for authoritative currency, explicit dimensional quantities, immutable published results, semantic rule versions, and complete input/intermediate/output traces.

`rust_decimal` is the initial spike candidate: its documentation describes a fixed-precision decimal representation intended for financial calculations, with bounded scale/precision ([crate documentation](https://docs.rs/rust_decimal/latest/rust_decimal/)). Adoption depends on overflow, serialization, scale, and cross-platform golden tests.

## Consequences

More storage and explicit rule design are accepted in exchange for replay, auditability, targeted recalculation, and migration safety. Rule behavior cannot change in place; old evaluators/results remain replayable for the supported retention policy.

## Acceptance gate

Synthetic worked examples reconcile exactly; missing and ambiguous rates remain unavailable/blocked; rate selection, cycle and unit mismatch are deterministic; money golden/property tests pass on three OSes; serialization is canonical; rate/rule/policy upgrades leave approved results unchanged. Real-shop calibration is an M0.2 product-validation gate, not a prerequisite for testing the calculation representation.

## Spike evidence

TASK-001 implemented the proposed boundary with `rust_decimal 1.42.1`, project-owned typed units, validated deserialization, a typed DAG validator, provenance/version wrappers, exact arithmetic with rounding-required failures, and normalized deterministic snapshot JSON with intermediate traces. Formatting, strict Clippy, 23 runtime tests, and one compile-fail doctest pass locally and in the Windows/Linux/macOS matrix; see [the evidence record](../../06-quality/task-001-validation.md).

TASK-002 locally implements empty rate cards, scoped effective-dated entries, lifecycle/source/governance validation, deterministic selection and ambiguity blocking, versioned pricing/rounding policies, CALC-007–CALC-018 foundations with exact-only CALC-009 amortization, isolated EX-01/03/12 synthetic golden traces, and pinned selector/rate replay. Strict Clippy, 41 total runtime tests, the compile-fail doctest, and planning validation pass locally; see [the evidence record](../../06-quality/task-002-validation.md). ADR-0007 remains **In Review** because cross-platform TASK-002 evidence, governed nonterminating rational-division rounding, broader graph evaluation/migration, organization-level dependency review, and real-shop calibration/review remain open.
