# Terminology

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** All  
> **Related ADRs:** All  
> **Open questions:** OQ-015  
> **Dependencies:** None  
> **Supersedes:** None

- **Analysis snapshot:** Immutable, versioned result of model analysis, including warnings and provenance.
- **Assumption:** Unverified input used to proceed; always visible and resolvable.
- **B-rep:** Boundary representation preserving topological faces, edges, and vertices of exact or tolerance-based geometry.
- **Confidence:** Ordinal, reasoned assessment of evidence quality—not a probability of correctness.
- **Cost:** Internal expected economic consumption before selling-price policy.
- **Estimate:** Versioned calculation and proposed routing for a particular input package and quantity.
- **Feature:** Geometry region interpreted as a manufacturing-relevant candidate with confidence and review state.
- **Make quantity:** Units planned for manufacture, including spares/destructive samples; never below deliver quantity.
- **Margin:** `(price - cost) / price`; distinct from markup.
- **Markup:** `(price - cost) / cost`; distinct from margin.
- **Model fact:** Measurement or structural observation produced by a named analysis version.
- **Operation:** Ordered routing step with workcenter, time, cost, source, confidence, and overrides.
- **Override:** Authorized replacement of a proposed/imported/calculated value preserving both values, actor, time, and reason.
- **Price:** Customer-facing amount after pricing policy and approvals.
- **Risk reserve:** Explicit expected allowance; never hidden inside a universal complexity multiplier.
- **Routing:** Ordered set of manufacturing, inspection, outside-process, and administrative operations.
- **Setup:** Workholding/orientation state within which one or more operations execute.
- **Stock:** Purchased or supplied starting form before manufacturing.
- **Technical data:** Models, drawings, specifications, derived geometry, manufacturing assumptions, and related controlled content.
- **Vertical slice:** End-to-end thin product capability that proves architecture and user value.

The canonical detailed glossary is [domain glossary](../02-domain/glossary.md).
