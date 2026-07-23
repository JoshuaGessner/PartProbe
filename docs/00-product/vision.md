# Product Vision

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-001–REQ-F-065, REQ-NF-001–REQ-NF-022  
> **Related ADRs:** ADR-0001–ADR-0014  
> **Open questions:** OQ-001–OQ-050  
> **Dependencies:** Shop discovery interviews  
> **Supersedes:** None

PartProbe helps a human estimator turn incomplete RFQ inputs into an explainable, editable, confidence-rated quote foundation. It combines model facts, drawing-driven requirements, shop libraries, proposed routings, deterministic calculations, and explicit uncertainty without pretending to be production CAM.

Its differentiator is shop-specific manufacturing judgment: explain how this shop could make the part, which resources and bottlenecks it would consume, what alternatives exist, what requirements and uncertainty remain, what changed between revisions, and what the shop should charge—not merely reproduce a distributed marketplace price.

## Product promise

- Model-derived facts are visibly separated from assumptions and human decisions.
- Every cost and price can be traced to inputs, rules, versions, and overrides.
- The estimator can correct every generated proposal without losing its provenance.
- Sensitive technical data remains local by default and can operate offline.
- Planned STEP, STL, and 3MF support will prove the solid-versus-mesh distinction in the first vertical slice, subject to ADR-0004 validation.
- Advanced decision support will compare routing, capacity, availability, sourcing, revision, uncertainty, and bid alternatives while keeping every recommendation reviewable and versioned.

## Experience thesis

The primary workspace is a dense engineering canvas: model viewport, model/feature tree, routing, contextual inspector, confidence warnings, and persistent totals remain synchronized. Fast keyboard paths coexist with inspectable detail and accessible alternatives.

## Product boundary

PartProbe estimates manufacturing effort and commercial risk. It does not generate production-safe G-code, certify regulatory compliance, infer drawing requirements as authoritative facts, or issue unreviewed quotes.
