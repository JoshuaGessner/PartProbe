# Model Analysis Requirements

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-004–REQ-F-007, REQ-F-021–REQ-F-024; GEO-008–GEO-015; FEAT-001–FEAT-021  
> **Related ADRs:** ADR-0002, ADR-0005, ADR-0008  
> **Open questions:** OQ-006–OQ-011  
> **Dependencies:** Geometry fixtures  
> **Supersedes:** None

| ID | Rule |
|---|---|
| GEO-008 | Produce explicit-unit AABB; OBB is a separate versioned result with orientation and method. |
| GEO-009 | Produce volume, area, center of mass, body/shell count, and validity where representation supports them; otherwise return unavailable plus reason. |
| GEO-010 | Mesh analysis reports watertightness, manifold/degenerate findings, and measurement uncertainty. |
| GEO-011 | Stock candidates record form, dimensions, orientation, allowances, enclosure test, and confidence. |
| GEO-012 | Thin-section, local thickness, surface complexity, symmetry, and access results are indicators with method/limitations—not facts beyond evidence. |
| GEO-013 | Every stage returns structured result, warnings, confidence reasons, timing, algorithm version, and recoverable error. |
| GEO-014 | Approved snapshots are immutable and comparable to later analyses. |
| GEO-015 | Geometry identifiers used by feature/setup/routing results remain stable within a snapshot and map visibly to the viewer. |
| FEAT-001 | Every candidate has stable ID, referenced geometry, type, dimensions, tools/processes/orientations, confidence, warnings, and review state. |
| FEAT-002 | Users can accept, reject, merge, split, or redefine detections with history. |
| FEAT-003 | False-positive/negative expectations and confidence criteria are fixture-specific. |
| FEAT-004 | Initial slice does not require automatic hole/pocket recognition; architecture must accept later detectors. |
| FEAT-005 | Mesh-based features use separate methods and confidence ceilings from B-rep features. |
| FEAT-006 | Tool access and setup results are proposals with alternatives and unresolved collision/workholding limitations. |
| FEAT-007 | Planar, cylindrical, conical, and rotational evidence is represented before compound manufacturing features. |
| FEAT-008 | Hole candidates distinguish through/blind/counterbore/countersink/spotface evidence and never infer authoritative threads. |
| FEAT-009 | Pocket, open-pocket, slot, step, boss, chamfer, fillet, and thin-wall candidates retain supporting topology/measurements. |
| FEAT-010 | Deep cavity, small-radius, long-reach, undercut, and difficult-access indicators create explicit risk/warning evidence. |
| FEAT-011 | Turning candidates distinguish OD, ID, face, shoulder, groove, bore, taper, cutoff, and thread candidate evidence. |
| FEAT-012 | Cross-hole, milled-flat, live-tool, transfer, and Swiss indicators remain process alternatives, not classifications of fact. |
| FEAT-013 | Five-axis, mill-turn, EDM, grinding, deep-hole, and custom-tool indicators state tested access assumptions and limitations. |
| FEAT-014 | Recognizers operate against immutable geometry snapshots and record recognizer/configuration versions. |
| FEAT-015 | Detection order/parallelism shall not change canonical results within documented tolerances. |
| FEAT-016 | Overlapping/conflicting candidates are preserved for adjudication rather than silently discarded. |
| FEAT-017 | A reviewer action records actor/time/reason and links original and resulting candidates. |
| FEAT-018 | Manual candidates are labeled user-defined and carry no fabricated algorithm confidence. |
| FEAT-019 | Expected fixture results include accepted candidates, ambiguous regions, known false positives/negatives, and confidence reasons. |
| FEAT-020 | Cross-revision candidate matching is non-authoritative and never substitutes renderer IDs for geometry references. |
| FEAT-021 | A feature candidate cannot become an approved routing operation without explicit user/application workflow adoption. |
