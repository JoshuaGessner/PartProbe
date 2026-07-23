# Feature-recognition validation

## Metadata

- **Status:** In Review
- **Last updated:** 2026-07-22
- **Related requirement IDs:** REQ-F-023–REQ-F-024, FEAT-001–FEAT-021, GEO-010–GEO-014, TEST-031–TEST-039
- **Related architecture decision IDs:** ADR-0002, ADR-0005
- **Open questions:** Minimum precision/recall thresholds by feature family? Human-review sample size? Who adjudicates ambiguous manufacturing semantics?
- **Dependencies:** Annotated fixtures, geometry validation, feature model, UI review tests, shop subject-matter review
- **Supersedes / superseded by:** None / none

## Objective and scope

Validate deterministic, explainable feature candidate behavior and its confidence limits. Success is not “perfect recognition”; it is correct evidence, safe uncertainty, stable review behavior and no unannounced regression. Manufacturing route accuracy is evaluated separately with estimator review.

## Expected-result schema

For each fixture pin: source and geometry-snapshot hashes, recognition profile/suite version, expected candidates with type/subtype, geometry references, dimensions and tolerance, evidence rule, confidence label/reasons, candidate process/setup/tool suggestions, known false positives/negatives, ambiguous regions, expected warnings, and visual selection target. Explicitly record expected **absence only where the fixture guarantees it**; a timeout/unsupported condition is “incomplete,” not absence.

## Test matrix

| Test class | Assertions |
|---|---|
| Primitive geometry | Hole axis/diameter/depth; planar/cylindrical/conical classifications; axis orientation under transforms |
| Topological pattern | Through vs blind, counterbore/countersink/spotface, open/closed pocket, slot/boss, fillet/chamfer boundaries, concentric turning profile |
| Ambiguity | Cylindrical cosmetic/thread-like surfaces, blended pocket edges, casting void, disjoint cylinders, mesh facets—must warn/cap confidence rather than overclassify |
| Representation | Same nominal part as B-rep and mesh: results preserve distinct provenance; mesh never reaches high/medium ceiling defined for exact topology |
| Robustness | Invalid/healed/unknown-unit geometry yields needs-review; candidate limits/timeouts produce incomplete warning |
| Determinism | Repeated / cross-platform run produces stable evidence ordering and IDs within snapshot contract |
| Review UX | Highlight resolves to expected geometry; accept/reject/edit/merge/split/manual preserves original evidence and audit history |
| Downstream | Only selected reviewed features feed routing; reanalysis creates diff and cannot overwrite approved routing |

## Metrics and interpretation

Report precision and recall only against a fixture taxonomy and human-annotated ground truth, separately for exact and mesh pipelines and by feature subtype. Include dimension error, false-positive/false-negative counts, `needs-review` rate, accepted/edit/reject rate in internal usability studies, elapsed time and incomplete-analysis rate. Do not combine them into a deceptive universal accuracy percentage. A lower automation rate may be correct when uncertainty increases.

## Confidence tests

Tests assert reason codes, not just a label: valid exact solid + unambiguous evidence can reach high; healing or ambiguous boundaries cap at medium; mesh caps at low; unit uncertainty/topology invalidity/conflict forces needs-review. A manually accepted candidate retains the system confidence and adds a review decision. Any change to a cap, reason taxonomy, threshold or aggregation rule is a documented feature-behavior/version change with updated fixtures and worked examples.

## Release criteria

No release may increase a known high-severity false positive, elevate a mesh/invalid feature above its ceiling, drop required warning/evidence, break stable references, or alter reviewed fixture output without approval. New recognizers need representative positives, negatives and ambiguous fixtures; an estimator/manufacturing reviewer must sign off that labels and explanations are not misleading. Feature recognition remains advisory until real shop estimate-versus-actual calibration supports a constrained use case.
