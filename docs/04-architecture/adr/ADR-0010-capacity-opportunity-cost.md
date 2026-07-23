# ADR-0010 — Capacity and Opportunity-Cost Semantics

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-043–REQ-F-046, REQ-F-065; REQ-NF-015–REQ-NF-018, REQ-NF-021–REQ-NF-022; CALC-021–CALC-026, CALC-031, CALC-033; TEST-045–TEST-050  
> **Related ADRs:** ADR-0007, ADR-0009  
> **Open questions:** Contribution basis, displaced-work set, bottleneck/horizon policy, and schedule source authority  
> **Dependencies:** Shop finance review, capacity data assessment, and scenario fixtures  
> **Supersedes:** None

## Context

A profitable job can consume capacity that limits more valuable work, but folding a scarcity premium into the normal machine rate would corrupt accounting traceability and historical comparison. Current schedules may also be incomplete, stale, or inaccessible.

## Decision proposed

Keep accounting cost, incremental cost, fully burdened cost, opportunity cost, risk-adjusted cost, and selling price as separate typed results. Capacity evaluation consumes immutable route-demand and capacity snapshots and returns advisory feasibility, bottleneck, overtime, schedule-risk, and profitability metrics.

Define opportunity cost only for a named constrained resource, horizon, contribution policy, and credible displaced alternative/portfolio or approved shadow-value scenario. It is not an accounting cost, is not derived from a workcenter rate, and cannot silently change cost or price. When displaced-work evidence is absent, return `Unavailable`.

Preserve a baseline estimate when current schedule data is absent. Initial releases use manual availability and explicit `NotEvaluated` states; coarse capacity scenarios precede advanced placement/optimization. Automatic reservation and dispatch remain outside the engine.

## Options considered

| Option | Disposition |
|---|---|
| Increase burdened machine rates when busy | Rejected: hides scarcity inside accounting cost and cannot explain the displaced alternative |
| Treat missing backlog as spare capacity/zero opportunity cost | Rejected: creates unjustified certainty |
| Build a full scheduler before capacity metrics | Rejected: overloads scope and risks ERP replacement |
| Separate immutable capacity scenarios and advisory economics | Proposed: supports staged, auditable decisions without rewriting estimates |

## Consequences

Users see more economic categories and must choose a horizon and contribution basis. Capacity data needs ownership, freshness, and access controls. Results may legitimately be unavailable or conditional. The separation permits historical accounting replay, transparent price decisions, and route comparison under multiple shop-state scenarios.

## Immutability, security, and approval

Approved estimates pin route, capacity snapshot, policy, calculation/pricing versions, and result digest. New schedules create sibling evaluations. What-if runs are non-mutating. Detailed backlog, customer, employee, and commercial data is access controlled; explanations may aggregate competing demand. Logs contain sanitized status/version metadata only.

Any route, delivery, overtime, or pricing change based on the result requires an authorized decision with original/new values, actor, time, reason, and policy-required concurrence.

## Acceptance gate

- Finance/manufacturing reviewers approve definitions and worked examples for all six economic values.
- Fixtures prove touch/occupancy/operator/setup/programming/inspection/queue/calendar/bottleneck time remain distinct.
- Parallel/simultaneous-resource, maintenance, overtime, infeasible, missing, stale, and conflicting-data scenarios pass.
- Opportunity-cost results identify displaced work and never modify accounting nodes.
- Historical replay, authorization, redaction, and cross-platform deterministic tests pass.
- Users can explain why an attractive accounting-margin job may rank poorly on a bottleneck without calling opportunity cost a booked expense.

Only authorized architecture/product/finance/manufacturing reviewers may change this ADR to Accepted.
