# Estimation Engine Architecture

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-005–REQ-F-010, REQ-F-040–REQ-F-065; REQ-NF-003, REQ-NF-004, REQ-NF-015–REQ-NF-021; CALC-001–CALC-035  
> **Related ADRs:** ADR-0007, ADR-0009–ADR-0014  
> **Open questions:** OQ-012–OQ-018, OQ-031–OQ-050  
> **Dependencies:** Calculation spike and shop policy  
> **Supersedes:** None

## Model

Use a typed directed acyclic calculation graph. Node definitions declare rule ID/version, typed inputs, output dimension/currency, rounding boundary, and evaluator. Evaluation produces `ResultValue`, input/result digests, intermediate trace, warnings, provenance, and confidence reasons.

## Safety properties

- Graph construction rejects cycles and incompatible dimensions.
- Missing/ambiguous inputs yield typed `Unavailable`/`Blocked`, never zero.
- Currency paths use fixed-precision decimal; floating inputs cross through named conversion policies.
- Overrides wrap results and preserve original trace.
- Published graphs and all library/version references are immutable.
- Deterministic serialization and canonical input hashing support replay.
- Deterministic baseline, readiness adjustment, uncertainty distribution, contribution/opportunity view, and selling price remain separately typed outputs.
- Seed, distribution method, sample count, capacity snapshot, route, requirement set, policy, and adapter versions are pinned whenever they influence a result.

## Recalculation

Draft edits invalidate only dependent nodes. UI receives a calculation diff and reconciled category totals. Re-analysis creates a new input branch; approved manual routing nodes stay pinned until a user explicitly adopts alternatives.

The first spike implements CALC-001, 003, 005, 008–018 with property and golden tests before routing UI work.

## Advanced analysis boundary

Advanced engines read an immutable `EstimateBaseline` and emit proposal artifacts: route comparisons, percentile summaries, sensitivity rankings, capacity/economic views, revision deltas, readiness adjustments, sourcing alternatives, and bid-priority explanations. A proposal includes inputs, exclusions, warnings, rule versions, result digest, and adoption state. It cannot rewrite the baseline graph.

Exact decimal rules remain authoritative for money. Physical simulation may use floating point behind named conversion/tolerance policies; stochastic runs must be reproducible from a recorded seed. Missing or stale inputs propagate `Unavailable`, `Unknown`, or `Blocked` and never become zero. The initial vertical slice stores future-safe version/source fields but implements only the deterministic baseline and basic manual checks.
