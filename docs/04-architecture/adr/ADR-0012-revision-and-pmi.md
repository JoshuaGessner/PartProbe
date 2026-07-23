# ADR-0012 — Revision Comparison and AP242 PMI Authority

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-049–REQ-F-055, REQ-F-065; REQ-NF-015–REQ-NF-018, REQ-NF-020–REQ-NF-022  
> **Related ADRs:** ADR-0002–ADR-0008  
> **Open questions:** Accepted PMI subset and exporter scorecard; comparison tolerances; exception authority; assembly/cross-format scope  
> **Dependencies:** OCCT/AP242 and comparison spikes, NIST/authorized fixtures, requirement-coverage policy, shop and quality review  
> **Supersedes:** None

## Context

PartProbe needs to compare model revisions and recognize embedded manufacturing requirements without turning translator output or geometric similarity into unreviewed manufacturing truth. STEP AP242 can carry semantic and graphical PMI, but file content and receiving-system coverage vary. CAD topology identifiers are unstable across export, healing and edits. Approved routings, estimates and quotes must remain reproducible.

## Proposed decision

Adopt two linked advisory systems:

1. **Versioned revision comparison:** compare immutable baseline/target snapshots in stages; preserve uncertainty and topology ambiguity; propagate only reviewed findings into a new draft estimate.
2. **Reviewed AP242 PMI subset:** inventory semantic and graphical PMI separately; normalize only proven translator constructs; create proposed requirement records tied to exact source revision and geometry; require human confirmation before coverage, costing, inspection planning or quote readiness.

Neither system mutates prior files, analyses, requirements, routings, overrides, estimates, quotes or prices. All outputs pin inputs, algorithms, dependencies, policies and tolerances. AP203/AP214 remain readable geometry interchange when enabled, but AP242 is the requested MBD/PMI interchange; an AP label never proves completeness.

## Authority rules

- Validated exact geometry is evidence for shape; semantic PMI is evidence for characteristic meaning; drawings/specifications/RFQ/customer clauses remain first-class sources.
- Graphical PMI supports human presentation and corroboration, not machine semantics.
- Successful import, visual similarity, validation properties, or a high mapping score does not establish contractual authority or completeness.
- Source conflicts, unsupported PMI, ambiguous correspondence and stale revision applicability remain visible and may block quote readiness.
- Human approval confirms a scoped interpretation; it does not erase importer confidence or warnings.

## Consequences

Benefits: traceable revision-cost explanations, preserved historical quotes, reusable structured requirements, explicit unknowns, and a path toward MBD-assisted inspection/estimating.

Costs: more snapshots/artifacts, translator-specific fixtures, review workload, correspondence uncertainty, and strict version/retention policy. Some files will show only geometry or graphical annotations. Early automation is intentionally limited.

## Alternatives considered

| Alternative | Disposition |
|---|---|
| Treat every AP242 file as complete MBD and auto-cost PMI | Rejected: exporter/importer coverage and contractual applicability are not guaranteed |
| Parse graphical PMI/OCR as authoritative semantics | Rejected: presentation is not a machine-interpretable requirement model |
| Use face/edge indexes as stable revision identity | Rejected: topology is unstable across translation, healing and ordinary edits |
| Compare only file hashes/global properties | Retained as foundation, rejected as the complete capability because it cannot explain local manufacturing impact |
| Automatically rerun and overwrite the prior quote | Rejected: breaks auditability, approval and calculation-versioning rules |
| Defer all PMI and revision planning | Rejected: domain/version boundaries would be costly to retrofit, though automation remains staged |

## Initial scope

- Schema/header detection and capability report.
- Supported semantic dimensions, GD&T/datums and saved views only when the spike proves entity/value/unit/modifier/association fidelity; separate graphical inventory/presentation.
- Hash/unit/frame/global-property comparison, explicit registration, exact-solid regional candidates when validated, uncertainty-aware topology/feature mapping, side-by-side/overlay views, and reviewed draft impact propagation.
- Manual requirement coverage, readiness blockers and permissioned exceptions; PMI creates proposals only.

Deferred: completeness claims, automatic drawing extraction authority, universal surface-texture/note support, native-CAD parity, unreviewed requirement carry-forward, broad assembly/cross-format matching, and autonomous quote repricing.

## Acceptance evidence

1. Public NIST and authorized shop AP242 fixtures cover semantic/graphical PMI, unsupported constructs, conflicting forms, AP203/AP214 and exporter variations; independent expectations verify values, units, modifiers, associations and saved-view presentation.
2. Revision fixtures cover identical, unit/frame-only, small/local, major, topology-renumbered, healed, mesh/B-rep, split/merge and analysis-version-only cases with reviewed ground truth.
3. Tests demonstrate no silent drop, false “no change,” unsupported high confidence, cross-revision association, override carry-forward, prior-record mutation, or quote-readiness bypass.
4. Three-platform determinism/tolerance reports, resource/containment tests, accessible review UX, classification inheritance, audit evidence, dependency/license review and manufacturing/quality sign-off pass.
5. Exact supported PMI matrix, tolerance/matching profiles, readiness exception policy, retention and rollback/version compatibility are approved.

Until these gates pass, the ADR remains **In Review** and UI language must qualify the capabilities as detected/proposed/reviewed rather than complete or authoritative.
