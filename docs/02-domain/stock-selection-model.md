# Stock Selection Model

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-005; GEO-008; CALC-002–CALC-007  
> **Related ADRs:** ADR-0002, ADR-0004  
> **Open questions:** OQ-009, OQ-010  
> **Dependencies:** Geometry and material models  
> **Supersedes:** None

Candidate stock forms: rectangular bar, plate, round, tube/pipe, hex, sheet, forging, casting, customer-supplied, near-net, and custom preform. A candidate stores source size, orientation, dimensional allowances, supplier evidence, yield/nesting method, certifications, restrictions, lead time, cost, risk, confidence, and rejection reasons.

Ranking considers enclosure, candidate orientation, saw/facing/workholding/clamp allowances, standard sizes, kerf, remnant policy, minimum order, availability, grain direction, country-of-origin constraints, material risk, and outside blanking. It returns alternatives—not one hidden optimum. The estimator can choose or define stock and must see mass, removed volume, cost, and risk deltas.
