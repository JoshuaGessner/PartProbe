# PMI and Model-Based Definition Research

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-051–REQ-F-054, REQ-F-065; REQ-NF-015–REQ-NF-018, REQ-NF-020–REQ-NF-022  
> **Related ADRs:** ADR-0002, ADR-0004, ADR-0005, ADR-0008, ADR-0012  
> **Open questions:** Which customers deliver usable semantic PMI; which characteristics and saved views survive the selected translator; who may confirm requirement applicability; whether a drawing remains contractually controlling  
> **Dependencies:** AP242/PMI fixture corpus, geometry-kernel spike, requirement-coverage model, customer contract review  
> **Supersedes:** None

## Conclusion

PartProbe should treat STEP AP242 PMI as a potentially valuable, structured source of manufacturing requirements, not as a promise that a STEP file is a complete or authoritative model-based definition. Initial support should detect the STEP application protocol, inventory imported PMI, preserve semantic and graphical forms separately, bind every item to its source revision and geometry references, and require human validation before it can satisfy quote readiness or affect cost.

The current published standard is ISO 10303-242:2025, edition 4, “Managed model-based 3D engineering.” Its scope includes multiple geometry representations, configuration/change data, delta change, product manufacturing information, and process-planning information. Scope in a standard does not establish that a particular exporter wrote those concepts or that a particular importer recovered them. [ISO 10303-242:2025](https://www.iso.org/standard/84300.html)

## Application protocols and practical meaning

| Protocol | Standards scope | Planning interpretation |
|---|---|---|
| AP203 | Configuration-controlled 3D mechanical parts and assemblies; later editions included tolerances, annotation, presentation, and validation properties | Common legacy exchange. Do not assume geometry-only, but do not assume complete/current PMI either. Record the declared schema and inspect content. [ISO 10303-203](https://www.iso.org/standard/39522.html) |
| AP214 | Automotive mechanical-design/product/process data, variants, configuration, approvals, and richer presentation | Common legacy exchange with names/colors/layers and some GD&T practices. Its published edition is withdrawn and revised by AP242. [ISO 10303-214:2010](https://www.iso.org/standard/43669.html) |
| AP242 | Convergence and extension of AP203/AP214 for managed model-based 3D engineering, including structured PMI and modern presentation practices | Preferred request when a supplier can provide STEP with MBD/PMI. Still validate exporter profile, edition, content, association, and importer coverage. [AP242 geometry/assembly/PMI overview](https://www.ap242.org/geometry-assembly-pmi-interoperability.html) |

An extension such as `.step` does not identify an AP or prove conformance. Preserve the Part 21 header, declared schema(s), exporter metadata, source SHA-256, and transfer diagnostics. Multiple files that render alike may differ materially in semantic content.

## Semantic and graphical PMI are different evidence

| Form | Meaning | Safe use | Limitation |
|---|---|---|---|
| **Semantic PMI / PMI representation** | Computer-interpretable entities for dimensions, dimensional and geometric tolerances, datum features, and related associations | Candidate structured requirements, applicability links, inspection planning, conflict detection, and downstream automation after validation | A syntactically imported entity may be incomplete, unsupported, mistranslated, attached to the wrong geometry, or inconsistent with another source |
| **Graphical PMI / PMI presentation** | The intended visual appearance and placement of annotation graphics | Human review, saved-view recreation, and visual corroboration | Not machine-interpretable requirement semantics; text or strokes must not be silently promoted into tolerances or cost drivers |

NIST explicitly distinguishes machine-interpretable semantic PMI from graphical PMI that preserves visual appearance, and its STEP File Analyzer reports the two independently along with validation properties. [NIST STEP File Analyzer and Viewer](https://www.nist.gov/services-resources/software/step-file-analyzer-and-viewer)

Graphical and semantic forms may both be present, one may be absent, and they may disagree. PartProbe must show that condition as a conflict or incomplete import. Optical/text extraction from graphical presentation is optional future assistance and remains unverified user-review material.

## PMI content and provenance model

The initial inventory should distinguish at least:

- dimensions and dimensional tolerances;
- geometric tolerances, datum systems, datum features, and datum targets;
- surface-texture/finish annotations when supported;
- notes, saved views/annotation planes, and graphical presentations;
- material and product metadata, explicitly separate from verified material requirements;
- association targets: part, assembly occurrence, body, face, edge, feature, or unresolved reference.

Every `PMIRecord` needs the source asset/revision, AP schema and edition when known, raw entity identifiers, normalized type/value/unit/modifiers, referenced geometry, semantic-versus-graphical form, importer/support status, warnings, source and importer versions, confidence reasons, and human review decision. Preserve unsupported entities as an inventory count plus sanitized diagnostics rather than dropping them silently.

## Translator reality

The proposed OCCT path is plausible but incomplete. Current OCCT documentation says its XDE STEP reader can translate assemblies, names, colors, layers, materials, validation properties, GD&T, and saved views. The same documentation identifies material limits: GD&T references are restricted to shapes or shape groups; not all tolerance zones are imported; graphical presentation is only partially implemented; character-based presentation is not handled; and styling and filled characters can be lost. It also calls AP214 GD&T handling obsolete in favor of AP242. [OCCT STEP translator](https://dev.opencascade.org/doc/overview/html/occt_user_guides__step.html)

Therefore a successful file read, `GDTMode` enablement, or non-empty PMI list is not a completeness test. The spike must report each supported construct and association, unsupported entity counts, lost presentation details, transfer warnings, and comparison against an independent analyzer or originating CAD result. OCCT’s reader API exposes separate modes for GD&T, validation properties, materials, metadata, and saved views, which is useful for an explicit capability report but is not evidence of end-to-end fidelity. [OCCT `STEPCAFControl_Reader`](https://dev.opencascade.org/doc/refman/html/class_s_t_e_p_c_a_f_control___reader.html)

Validation properties are source-system quantities intended to help check transfer fidelity. They are valuable comparison evidence; they do not prove geometric validity, semantic-PMI completeness, contractual authority, or manufacturing applicability. CAx-IF publishes separate recommended practices for geometry/assembly validation properties and AP242 PMI representation/presentation, reinforcing that these are distinct conformance concerns. [CAx-IF recommended practices](https://www.mbx-if.org/home/cax/recpractices/)

## Information priority and conflict policy

Priority is scoped by the fact being decided:

1. Validated exact model geometry is primary for geometric shape facts.
2. Validated semantic PMI/structured MBD is the preferred embedded source for characteristic semantics.
3. Structured RFQ fields follow for explicitly scoped commercial/manufacturing facts.
4. Verified drawing and referenced specification requirements remain first-class contractual sources.
5. User-entered requirements may clarify or add facts with identity and reason.
6. Automated drawing assistance may only propose records.
7. Human approval is required before imported or inferred records become quote-ready.

This order is not a silent conflict-resolution algorithm. Geometry cannot negate a tolerance, drawing text cannot silently rewrite model units, and newer files cannot be assumed applicable to an older RFQ. Contradictory credible sources produce a visible `RequirementConflict` and block readiness until an authorized decision records the basis.

## Revision applicability

PMI is immutable evidence bound to the source model revision and import snapshot. A later model must not inherit prior face associations merely because labels or topology indexes match. Revision comparison may propose correspondence using validated persistent identifiers or confidence-rated geometric matching; a reviewer must confirm carried requirements. If geometry changes invalidate an association, the requirement remains retained but becomes `applicability_needs_review`.

The prior model, PMI records, coverage decisions, quote, and overrides remain unchanged. Re-import with a new translator creates a sibling snapshot and explicit semantic/presentation diff; it never upgrades historical coverage in place.

## Security and controlled-data handling

PMI, saved views, names, notes, material data, and generated screenshots can expose more sensitive design intent than geometry alone. They inherit the strictest source-project classification. Import remains in the no-network geometry worker; external STEP references are disabled by default; logs use counts/codes rather than annotation text or coordinates; search indexes, thumbnails, clipboard, exports, diagnostics, and test artifacts remain policy-controlled and audited. No fixture containing customer or controlled content may be submitted to an external validator without explicit authorization.

## Staged scope and evidence gates

| Stage | Included | Deliberately excluded |
|---|---|---|
| Foundation/spike | AP/schema detection; semantic/graphical inventory; supported dimensions, GD&T/datums, saved views and validation-property report; raw provenance; NIST and authorized shop fixture comparison | Claims of complete AP242 support; autonomous costing; drawing OCR authority |
| First usable PMI scope | Review UI; requirement-record proposals; geometry association highlighting; conflict/incompleteness states; explicit confirmation; quote-readiness integration | Automatic acceptance; all tolerance-zone/modifier/surface-texture dialects; assembly-wide MBD authority |
| Advanced | Expanded AP242-edition/profile coverage; exporter-specific scorecards; richer surface texture/notes; validated revision carry-forward and inspection mappings | Native-CAD parity unless separately licensed and proven |

The gate requires NIST public PMI test models plus authorized exporter-specific fixtures, independent entity/count/value/association expectations, semantic-versus-graphical coverage, round-trip or dual-reader comparison where lawful, malformed-file containment, three-platform determinism, and manufacturing/quality reviewer sign-off. NIST provides AP242/AP203 PMI test cases expressly useful for conformance and interoperability testing. [NIST CAD/PMI test models](https://www.nist.gov/ctl/smart-connected-systems-division/smart-connected-manufacturing-systems-group/mbe-pmi-0)

## Decision

Proceed with an evidence-producing AP242/PMI spike and a conservative reviewed subset under ADR-0012. Until its gates pass, PartProbe should say “PMI detected/imported with stated coverage,” never “MBD complete” or “all requirements recognized.”
