# Stock Selection Model

> **Status:** Draft  
> **Last updated:** 2026-09-10
> **Related requirements:** REQ-F-005; GEO-008; CALC-002–CALC-007  
> **Related ADRs:** ADR-0002, ADR-0004  
> **Open questions:** OQ-009, OQ-010  
> **Dependencies:** Geometry and material models  
> **Supersedes:** None

Candidate stock forms: rectangular bar, plate, round, tube/pipe, hex, sheet, forging, casting, customer-supplied, near-net, and custom preform. A candidate stores source size, orientation, dimensional allowances, supplier evidence, yield/nesting method, certifications, restrictions, lead time, cost, risk, confidence, and rejection reasons.

Ranking considers enclosure, candidate orientation, saw/facing/workholding/clamp allowances, standard sizes, kerf, remnant policy, minimum order, availability, grain direction, country-of-origin constraints, material risk, and outside blanking. It returns alternatives—not one hidden optimum. The estimator can choose or define stock and must see mass, removed volume, cost, and risk deltas.

USE-2 schema v2 now persists one optional draft `StockAllowanceProfile` with a versioned rectangular, round, or plate form and explicit nonnegative X/Y/Z millimetre allowances plus source evidence. It is configuration input for the future USE-3 proposal rule, not a selected blank, stock-volume result, automatic recommendation, or production default.

The `shop-resource-catalog-v1` contract can retain bounded exact stock-allowance versions and pin one in a reviewed resource selection. Schema v3 persists that evidence and schema v4 can persist the separate authorization decision. The draft service preserves current allowance evidence, permits only correctly versioned Draft additions/successors, and downgrades active proposal eligibility on edit; the separate catalog application can create a policy-evaluated and audit-gated active-for-proposals successor. Contract v9 maps the complete bounded draft through native identity/time evidence, but ordinary startup remains identity-unavailable and the WebView exposes no catalog action. That state means only that a future proposal service may consider the pinned allowance; it does not select stock, calculate a blank, or authorize an estimate. A governed editor, authenticated identity/roles, a reviewed shipped allow configuration, and activation UI remain pending.
