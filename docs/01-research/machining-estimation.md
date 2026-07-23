# Machining Estimation Research

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirement IDs:** REQ-F-006–REQ-F-010, REQ-F-013–REQ-F-018, CALC-001–CALC-020  
> **Related architecture decision IDs:** ADR-0007  
> **Open questions:** Which rate cards, setup policies, and actuals are available per workcenter?  
> **Dependencies:** `02-domain/feeds-and-speeds-model.md`, routing model, rate-card model, historical actuals  
> **Supersedes:** None

## Evidence classification

| Type | Use in PartProbe |
| --- | --- |
| Formal requirement | A contract, purchase order, drawing, customer clause, or governing regulation. Capture as a quoted requirement with source and revision; do not infer it from part geometry. |
| Industry practice | A repeatable planning convention that a shop may adopt after review. Store it as a named, versioned shop policy. |
| Vendor claim or data | Manufacturer application guidance or machine specification. Retain source/version and treat it as a starting value, not independent truth. |
| Shop-specific assumption | Editable estimate input with owner, effective date, rationale, and approval state. |

## Recommended estimating structure

Use an operation-based calculation graph. A feature, if confidently recognized, may propose operations; it must not make them mandatory. Each operation records process, machine/workcenter, setup, tool, material condition, quantity basis, and evidence source.

`Internal cost = material + preparation + setup + programming + cutting time + non-cutting cycle time + operator attendance + tooling/consumables + inspection/documentation + outside processing + packaging/freight allowance + scrap/rework risk`.

Keep **cost**, **risk allowance**, and **selling-price rule** separate. Preserve the original automatic result and each override (old value, new value, actor, timestamp, reason). Calculate per setup and per part, then apply quantity amortization only through a named rule.

## Estimate-maturity gates

| Level | Permitted inputs and result | Use |
| --- | --- | --- |
| L0: screening | envelope, material family, process class, quantity, conservative baseline removal rate | Early RFQ triage; clearly low confidence. |
| L1: routing | editable stock, setups, operation templates, workcenter rates, setup/programming allowances | Initial vertical slice. |
| L2: feature-aware | reviewed features, tools, feeds/speeds, access directions, cut and non-cut elements | Normal structured estimate. |
| L3: validated | drawing/clauses, fixture concept, machine/tool-specific data, reviewed outside quotes and historical comparables | Approval candidate. |

L0–L3 classify overall estimate evidence and review maturity; they are distinct from the six runtime-calculation methods in [runtime requirements](../03-requirements/runtime-requirements.md). They are not claims of CAM simulation accuracy. Missing drawing requirements, uncertain units, mesh-only geometry, or inaccessible regions must reduce confidence and create visible review tasks.

## Operation cost guidance

| Cost element | Deterministic rule | Required review input |
| --- | --- | --- |
| Material | purchased-stock volume/mass × price plus yield/scrap rule; track quote date, currency, MOQ, and certification premium separately | supplier quote or dated library price |
| Saw/prep | operation template time plus stock handling and kerf allowance | stock form and cutoff method |
| Setup/prove-out | fixed or parameterized hours per setup; separate first-piece/prove-out from recurring setup | fixture/soft-jaw need, datum plan, quantity |
| Programming/CAM | editable labor estimate; may be template/complexity derived but never silently learned | process novelty and programming scope |
| Cutting/non-cut cycle | runtime engine result by operation | feeds, speeds, machine limits, approach/retract, tool changes, probing/indexing |
| Tooling | perishable tool-life allocation + special-tool/fixture purchases | tool family, life model, replacement policy |
| Quality | inspection plan tasks, CMM programming/runtime, certifications, FAI, source-inspection standby | drawing/PO/customer clauses |
| Outside process | pass-through quote or versioned allowance, lead time, minimum lot, shipping | approved vendor quote and revision |
| Risk | explicit, reviewable contingency; do not bury in cycle time | material scarcity, geometry/drawing uncertainty, first-run risk |

## Process coverage and guardrails

- Milling: estimate facing, adaptive/roughing, pockets, slots, contouring, drilling, tapping, reaming, boring, deburr, probing, and index/rotary motion as separate operations.
- Turning, live-tool, mill-turn, and Swiss: model chuck/bar/collet preparation, bar-feed or part transfer, turning passes, cross/live-tool work, cutoff, subspindle transfer, and secondary operations separately. Do not map a rotational envelope directly to a safe process plan.
- Grinding, EDM, manual machining, repair, and special processes: begin with operation templates and human-entered effort. Geometry alone cannot reliably infer process capability, blend/repair limits, or required documentation.
- Fixture, soft jaw, and special-tool purchases are non-recurring, reusable, or job-specific by a declared amortization policy; do not silently amortize across unrelated jobs.

## Calibration without rule drift

Actuals can propose a new version of a rate card, operation template, or feeds/speeds library. They must never mutate an approved estimate or baseline automatically. Store actual machine, operator-attendance, setup, programming, inspection, scrap/rework, and outside-process values separately; classify variance as geometry/import, planning, execution, purchasing, or scope change before calibration.

## Research basis

- NIST describes feature-based process planning as manual, automatic, or hybrid; it notes that most feature-based planning is manual at cell/workstation level and CAM is automated at task level. This supports suggestion-plus-review rather than autonomous routing. [NIST feature-based process planning](https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=901352)
- NIST’s feature-control reference separates shop preferences, tool catalog/inventory, machining features, and a process plan; use the same separation in the domain model. [NIST feature-based control](https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=820582)
- Sandvik and Kennametal formulas are tool-vendor technical guidance, useful for transparent equations but not a substitute for a shop-approved library or process validation. [Sandvik milling formulas](https://cdn.sandvik.coromant.com/files/sitecollectiondocuments/services/metal-cutting-e-learning/formulas-and-definitions/formulas-and-deinitions-for-milling-metric-enu.pdf), [Kennametal calculator FAQ](https://www.kennametal.com/us/en/resources/engineering-calculators/miscellaneous/speed-and-feed.html) (vendor sources accessed 2026-07-22).
