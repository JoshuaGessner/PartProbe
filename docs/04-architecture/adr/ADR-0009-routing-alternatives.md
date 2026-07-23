# ADR-0009 — Routing Alternatives and Selection Boundary

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-040–REQ-F-042, REQ-F-065; REQ-NF-015–REQ-NF-018, REQ-NF-021–REQ-NF-022; TEST-040–TEST-044  
> **Related ADRs:** ADR-0007, ADR-0008, ADR-0010, ADR-0011  
> **Open questions:** Approved strategy providers, hard constraints, scoring profiles, and route-approval roles  
> **Dependencies:** Route corpus and estimator/manufacturing review  
> **Supersedes:** None

## Context

One part can have several defensible manufacturing approaches. A single generated routing hides tradeoffs, while unconstrained optimization can create implausible plans and an opaque scalar score can replace rather than support judgment. Approved human routings must also survive later analysis.

## Decision proposed

Represent alternatives as immutable revisions in an estimate-owned `RoutingAlternativeSet`. Use a bounded, provider-based, deterministic pipeline that separates generation, hard-feasibility screening, metric evaluation, comparison, and human adoption.

The authoritative comparison is a labeled metric vector with unavailable values and evidence. Named scoring profiles may order eligible routes, but must preserve raw metrics, hard filters, normalization, weights, tie-breaks, exclusions, policy version, and reasons. Show non-dominated alternatives where helpful. Never let the optimizer approve a routing, issue a quote, reserve resources, or overwrite an adopted route.

Start with manual and template alternatives. Add automatic strategy generation only after route-corpus validation. Keep production CAM, safety validation, and shop-floor release outside this boundary.

## Options considered

| Option | Disposition |
|---|---|
| One generated “best” route | Rejected: conceals alternatives, uncertainty, and shop preference |
| One weighted score as domain truth | Rejected: weights and unavailable data can obscure material tradeoffs |
| Full automated optimization in the first release | Rejected: corpus, feasibility rules, and shop policies are not validated |
| Versioned alternatives + vector comparison + explicit adoption | Proposed: preserves explanation, staged delivery, and human authority |

## Consequences

The model and UI carry more candidates, versions, partial states, and decision evidence. Comparisons can show ties and no clear winner. Search providers require resource limits and expert-reviewed feasibility tests. In exchange, route decisions are replayable and can incorporate cost, delivery, capacity, risk, confidence, and readiness without conflating them.

## Immutability, security, and missing data

An adopted route is pinned by the approved estimate; editing/re-analysis creates siblings. Missing hard evidence is `Unknown` or conditionally eligible, not a silent pass. Missing comparison metrics remain unavailable rather than becoming zero. Providers run locally/in-boundary with least data and no default network; route and commercial content is excluded from diagnostics.

## Acceptance gate

- Expert-reviewed fixtures cover distinct milling, turning, hybrid, inspection, and sourcing alternatives.
- Hard constraint, unknown, rejection, deduplication, ranking, tie, and Pareto behavior is deterministic and explained.
- Manual creation, rejection, override, adoption, replacement, and re-analysis preserve full history.
- Bounded execution/cancellation and cross-platform replay pass.
- Authorization and log-redaction tests pass.
- Usability testing shows estimators can identify why a route ranks differently and can adopt a non-leading route with a recorded reason.

Only authorized architecture/product/manufacturing reviewers may change this ADR to Accepted.
