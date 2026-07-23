# Competitor and Market-Pattern Analysis

## Metadata

- **Status:** Draft
- **Last updated:** 2026-07-22
- **Related requirement IDs:** REQ-F-001, REQ-F-021, REQ-F-039, REQ-NF-004, UX-001, UX-006
- **Related architecture decision IDs:** ADR-0001
- **Open questions:** Which competitor workflows do target estimators actually use today? Which integrations are required at launch?
- **Dependencies:** Shop interviews; [information architecture](../05-ux/information-architecture.md); [desktop platform](../04-architecture/desktop-platform.md)
- **Supersedes / superseded by:** None

## Purpose and method

This is a product-pattern review, not an endorsement or a claim that vendors’ marketing assertions have been independently verified. Claims below are attributed to vendor material. We examine interaction and workflow lessons while avoiding copied branding, layouts, pricing models, or proprietary visual identity.

| Product | Vendor-described strengths | Useful lesson | Deliberate difference for PartProbe |
|---|---|---|---|
| Paperless Parts | 2D/3D viewer, geometry-driven routers, manufacturability warnings, configurable pricing and collaboration ([CNC machining page](https://www.paperlessparts.com/processes/cnc-machining/); [costing page](https://www.paperlessparts.com/pricing-costing-automation/)) | Put geometry and quote context together; show warnings before price approval; enable bulk actions. | Local-first desktop deployment, explicit deterministic calculation trace, no silent AI authority, and every inferred value editable with provenance. |
| Costimator | Cost estimating with/without CAD, preloaded cost models, and machining feature-based estimation claims ([Costimator](https://www.mtisystems.com/)) | Reusable shop models and familiar operations reduce estimator variance. | Surface assumptions, formulas, algorithm/version IDs, and confidence rather than presenting a single opaque answer. |
| ProShop | Template-driven estimates, margin guards, quantity breaks, and quote-to-work-order handoff ([estimating page](https://proshoperp.com/product/estimating-quoting/)) | Preserve quote-to-production continuity and protect margin at approval time. | Scope initial product as estimating assistance rather than replacing ERP/MES/QMS; repository boundaries make future handoff explicit. |
| Machine Research | Vendor describes AI estimates from 3D files, historical learning, and searchable prior parts ([Machine Research](https://machineresearch.com/)) | Similar-job retrieval and actuals are valuable estimator aids. | Never let learning rewrite approved defaults; preserve original estimate and require controlled approval for calibration. |

## Opportunity statement

The market validates several expectations: a quote begins with an RFQ package, CAD/PDF context matters, labor/material/overhead must be configurable, estimators need rapid repeatable operations, and conversion/handoff prevents duplicate entry. It also exposes a gap relevant to specialty and controlled-data shops: a transparent local workflow that links exact input geometry, detected features, setup alternatives, calculation versions, confidence reasons, and human overrides in one inspectable record.

## Product implications

1. **Make automation inspectable.** A suggested route must say why: geometry evidence, chosen rules, rate-card and feeds/speeds versions, confidence reason, and unresolved requirements. “AI-generated” is not a sufficient explanation.
2. **Keep the package visible.** The quote workspace must keep model, drawing/requirement checklist, routing, costs, and warnings reachable without a modal maze.
3. **Keep cost separate from price.** Margin floors and approval thresholds are useful patterns, but estimators must see internal cost, risk reserve, price, and manual price adjustment as distinct concepts.
4. **Support history without contamination.** Similar work and actuals are references with provenance; no automatic model training or rule alteration from a job outcome.
5. **Favor local controlled-data operation.** Cloud collaboration is useful elsewhere, but imported CAD, drawings, and quote data remain local by default. Team/LAN is a later deployment profile, not an assumption.

## Research validation plan

Interview six to ten estimators/programmers using their current system. Ask them to execute: RFQ triage, import/revision comparison, routing edit, quantity-break pricing, approval, and estimate-vs-actual review. Record time, error recovery, confidence, and information they had to retrieve elsewhere. Validate patterns against work, not screenshots.
