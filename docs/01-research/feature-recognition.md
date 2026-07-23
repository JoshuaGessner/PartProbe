# Feature-recognition research

## Metadata

- **Status:** In Review
- **Last updated:** 2026-07-22
- **Related requirement IDs:** REQ-F-023, REQ-F-024, GEO-010–GEO-014, FEAT-001–FEAT-021, TEST-031–TEST-039
- **Related architecture decision IDs:** ADR-0002, ADR-0005
- **Open questions:** Which feature classes matter most to estimators? What reviewed fixtures and annotation process can establish ground truth? What false-positive rate is tolerable?
- **Dependencies:** Validated geometry artifact, feature model, fixture strategy, setup planner, shop taxonomy
- **Supersedes / superseded by:** None / none

## Practical position

Feature recognition is a deterministic, explainable **candidate-generation and review** system. It is not CAD history recovery, drawing interpretation, tolerance extraction, or automatic CAM. Start with exact-B-rep planar/cylindrical topology and simple through/blind hole candidates; postpone pockets, turning profiles, undercuts, and multi-axis accessibility until their fixture-based error behavior is measured. Mesh recognition is a separate, lower-confidence pipeline and must never be presented as equivalent.

## Recognition roadmap

| Phase | Exact B-rep capability | Mesh-only capability | Estimator use |
|---|---|---|---|
| 0: vertical slice | Body class, AABB/OBB, cylindrical-envelope and rotational-symmetry *indicator* | AABB, watertightness, approximate envelope; no authoritative features | Broad process / stock suggestion only |
| 1: high-value primitives | Planar faces, cylinders, cones; through/blind/coaxial hole, counterbore/countersink/spotface candidates; external faces | Optional approximate hole candidate after validated fit | Editable feature list and coarse runtime inputs |
| 2: prismatic | Closed/open pockets, slots, steps, bosses, chamfers/fillets, thin walls, deep cavities, simple access directions | Approximate planar/cylindrical regions only | Setup/tool candidate suggestions |
| 3: turning | Revolved profile, OD/ID, faces, shoulders, grooves, bores, tapers; cross-hole and milled-flat candidates | Axis-fit indicator only | Turning/mill-turn alternative proposals |
| 4: advanced | Undercut/accessibility, multi-direction clusters, difficult reach, five-axis / EDM / Swiss indicators | No automatic claim unless a specific mesh algorithm is validated | Risk flags and alternatives, never definitive routing |

## Algorithm shape

1. Require a validated geometry artifact and select exact or mesh pipeline.
2. Normalize units and tolerance policy; classify topology/surfaces; build adjacency and candidate axes.
3. Generate candidates from explicit geometric predicates—e.g., cylindrical faces plus bounded loops and coaxiality—not opaque labels.
4. Consolidate overlapping candidates, retain all evidence, measure dimensions, and associate stable geometry references.
5. Run manufacturability checks (depth/diameter, open boundaries, approach cone, neighboring obstruction, thin-wall risk) to produce warnings and process/tool *candidates*.
6. Score and classify uncertainty; present visual highlight plus dimensions and evidence; estimator may accept, reject, merge, split, or redefine.
7. Freeze the reviewed feature snapshot and link routing operations to feature IDs. Reanalysis creates a new proposal; it never overwrites approved work.

## Confidence behavior

Confidence is a labelled assessment, not probability or estimate accuracy. Use `high`, `medium`, `low`, and `needs-review`, plus reason codes. It is capped by provenance:

| Situation | Maximum | Required explanation |
|---|---|---|
| Valid exact solid; canonical topology, dimensions, and unambiguous classifier | High | Rule version and geometry evidence |
| Exact shells/healed model or ambiguous topology | Medium | Healing/ambiguity and affected references |
| Mesh with validated closed manifold topology and strong primitive fit | Low | Mesh resolution, fit residual, unit certainty |
| Open/non-manifold mesh, unknown units, self-intersection, unsupported entity, or feature conflict | Needs review | Why dimensions/semantics are unreliable |

No downstream setup or runtime confidence may exceed the weakest unresolved critical input. UI must show reason codes, not a percentage. A user acceptance means “accepted for this quote,” not “algorithm correct.”

## Known false-positive / false-negative classes

- A drilled hole, cast void, thread representation, cosmetic cylinder, and clearance feature can share cylinder geometry.
- Pocket boundaries may be split by blends, imports, or healing; a face color/layer is not reliable intent.
- Fillets/chamfers may be decorative, datum-related, or tooling-driven; do not infer finish/tool choice from radius alone.
- Rotational symmetry does not prove lathe accessibility, bar stock, spindle transfer, or that milling is inferior.
- A mesh’s facets can imitate small radii/slots and erase holes below resolution; triangulation changes candidate results.
- Occlusion and local collision tests cannot establish fixturing, clamp placement, tolerances, or safe production toolpaths.

## Evidence and implementation constraints

The feature pipeline relies on the kernel only for geometry/topology queries; its own rules and versions remain application-owned. OCCT’s data-exchange layer supports translated STEP/IGES shapes and validation, not semantic manufacturing features. [OCCT overview](https://dev.opencascade.org/doc/overview/html/index.html) Rust-native B-rep projects expose topology and STEP/STL/OBJ-oriented components, useful for experiments but insufficient proof for production interoperability. [Monstertruck documentation](https://docs.rs/monstertruck/latest/monstertruck/)

Every new recognizer requires an annotated fixture, expected detections and dimensions, known false positives/negatives, confidence criteria, visual-review test, regression result, and a feature-algorithm version increase. See `../06-quality/feature-recognition-validation.md`.
