# Estimating Model

> **Status:** In Review
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-F-006–REQ-F-011, REQ-F-040–REQ-F-065; CALC-001–CALC-035; DATA-001–DATA-042
> **Related ADRs:** ADR-0007–ADR-0014
> **Open questions:** OQ-012–OQ-018, OQ-027, OQ-031–OQ-050
> **Dependencies:** `calculation-rules.md`, runtime model
> **Supersedes:** None

## Calculation graph

`input package + requirement coverage + shop-library versions + user decisions → quantity scenario → routing-alternative set → per-route operation results → categorized internal cost + explicit risk reserve → pricing policy → selling price`.

Each node stores value, unit/currency, inputs, rule ID/version, selected rate-entry/card and selector versions where applicable, intermediate values, pricing/rounding policy versions, provenance, warnings, confidence reasons, and override chain. Edges are acyclic. Published estimates reference immutable node-result snapshots. Rate inputs follow [the rate-library contract](rate-library.md); missing or conflicting applicable rates never become zero.

Advanced analyses consume—not rewrite—the deterministic baseline: uncertainty wraps selected inputs/results; capacity and availability use explicit as-of snapshots; opportunity and sourcing compare typed cost bases; revision analysis compares immutable snapshots; bid scoring consumes reviewed outputs. Each produces a separate versioned result and requires explicit adoption into an estimate/quote revision.

## Quantity scenario

A scenario separates deliver quantity, make quantity, destructive-test/spare quantity, lot size, setup amortization, expected scrap, recurring cost, non-recurring cost, and lead-time assumptions. Quantity breaks are independently recalculated—not divided versions of one total.

## Review states

`Draft → Analyzed → Reviewed → Approved → Quoted`; revision produces a new draft. Rejected/withdrawn/expired are terminal commercial states. Approval requires resolved blockers or recorded, authorized acceptances.

## Confidence

Confidence is ordinal (`High`, `Medium`, `Low`, `Unknown`) per claim and aggregate dimension. The UI must list reasons and never collapse independent geometry, process, commercial, and requirement uncertainty into an unexplained score.

Confidence is evidence quality, not the same as an uncertainty distribution. A low-confidence assumption may use a wide range, but a percentile is shown only when the selected scenario/distribution method and inputs justify it. Missing data may yield `Unavailable` rather than a fabricated range.
