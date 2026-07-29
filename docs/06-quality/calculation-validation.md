# Calculation Validation

> **Status:** In Review
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-F-005–REQ-F-010, REQ-F-040–REQ-F-065; REQ-NF-003, REQ-NF-015–REQ-NF-021; CALC-001–CALC-035; TEST-001, TEST-002, TEST-040–TEST-099
> **Related ADRs:** ADR-0007, ADR-0009–ADR-0014
> **Open questions:** Approved rounding and shop policies
> **Dependencies:** Executable calculation graph
> **Supersedes:** None

## Test classes

- Exact examples for mass, stock/removed volume, make quantity, setup amortization, cycle/run cost, category totals, risk, markup, margin, floors, and quantity breaks.
- Empty production rate-card, effective-date, lifecycle, currency/basis, ordered-scope selection, ambiguity blocking, rate-extension, pricing-policy and named-rounding-boundary cases.
- Boundary/error cases: zero quantities/denominators, 100% scrap, negative input, overflow, currency mismatch, missing unit, undefined percentage, and blocked value propagation.
- Properties: positive components do not lower linear totals; make ≥ deliver; stock-enclosed removed volume ≥ 0; unit round-trips meet tolerance; margin/markup inverse relationships; graph order does not change result.
- Cross-platform canonical serialization and result digests.
- Migration/replay against prior calculation versions.
- Pinned rate-card/entry/selector, pricing and rounding-policy replay after later versions exist.
- TEST-005/TEST-017 reconciliation proving each in-cycle probe/inspection, post-cycle operator inspection, and quality/CMM/FAI occurrence contributes once to elapsed time and once to each applicable cost basis—never twice.
- Route fixtures that prove feasibility exclusions precede ranking, score ties are deterministic, and adoption never overwrites an approved baseline.
- Capacity fixtures covering stale/missing calendars, partial occupancy, bottleneck choice, delivery feasibility, and the strict separation of accounting cost from contribution/opportunity views.
- Seeded uncertainty fixtures for triangular/PERT inputs, percentile ordering, convergence tolerance, correlation assumptions, sensitivity ranking, and deterministic-baseline containment.
- Revision fixtures for unchanged, transform-only, topology-renumbered, tolerance-changed, and ambiguous mappings with rate/estimate-basis separation.
- Make/buy and bid fixtures proving constraint blockers run before scoring and missing external evidence stays unavailable.

Every golden fixture stores raw inputs, expected intermediate nodes, unrounded and rounded results, strategy/scale, rule/library versions, reviewer, and tolerance where non-decimal physical values are involved.

## Current evidence

[TASK-001 validation](task-001-validation.md) provides executable local and three-OS evidence for CALC-001, CALC-003, CALC-005, CALC-016, CALC-017, typed graph rejection, and canonical serialization.

[TASK-002 validation](task-002-validation.md) provides local executable evidence for empty production rate cards; user-owned scoped/effective/approved rate entries; deterministic unavailable/blocked/selected resolution; versioned pricing and rounding; CALC-007–CALC-018 foundations; synthetic EX-01/03/12 exact traces; and pinned rate replay. These synthetic fixtures prove deterministic software behavior, not estimate accuracy or shop-policy validity. Cross-platform TASK-002 evidence and TASK-007 real-shop calibration remain pending.
