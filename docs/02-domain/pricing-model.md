# Pricing Model

> **Status:** In Review
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-F-008–REQ-F-010, REQ-F-041, REQ-F-045, REQ-F-048, REQ-F-061–REQ-F-064; CALC-013–CALC-035
> **Related ADRs:** ADR-0007, ADR-0009–ADR-0011
> **Open questions:** OQ-012–OQ-018, OQ-034–OQ-037, OQ-048–OQ-050
> **Dependencies:** Approved shop pricing policy
> **Supersedes:** None

Cost categories remain visible through pricing. Rate, allocation, and pricing inputs follow [the rate-library contract](rate-library.md). A versioned `PricingPolicy` may apply exactly one authoritative markup or target-margin method plus floors, minimum order/line values, setup/NRE presentation, quantity breaks, customer adjustments, expedite charges, discounts, and approval thresholds.

The UI always labels margin and markup and shows both formula and resulting values. Strategic adjustments and manual prices require reason and authorization. Customer-facing aggregation may hide internal rates but must remain traceable to the immutable internal estimate.

Accounting cost, incremental/direct-cash cost, fully burdened cost, risk-adjusted cost, opportunity cost, contribution, and selling price are different typed results. Opportunity/capacity and probabilistic outputs inform route, bid and price review but do not silently change accounting cost or selling price. A sourcing resale policy and any uncertainty-informed floor are separate versioned pricing inputs with approval thresholds.
