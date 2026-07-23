# Feature-recognition domain model

## Metadata

- **Status:** In Review
- **Last updated:** 2026-07-22
- **Related requirement IDs:** REQ-F-023–REQ-F-024, FEAT-001–FEAT-021, GEO-010–GEO-014, DATA-015
- **Related architecture decision IDs:** ADR-0002, ADR-0005
- **Open questions:** Feature ID persistence rule across revised models? Which tool library attributes are mandatory before tool suggestions?
- **Dependencies:** Geometry-analysis snapshot, feature pipeline, review UX, routing domain model
- **Supersedes / superseded by:** None / none

## Purpose

`FeatureAnalysisSnapshot` records versioned, reviewable feature **candidates** produced from one `GeometryAnalysisSnapshot`. It intentionally separates geometric evidence from manufacturing interpretation and from human acceptance. The user can edit every candidate without mutating the automatic result.

## Core entities

| Entity | Required fields | Rules |
|---|---|---|
| `FeatureAnalysisSnapshot` | ID, geometry snapshot ID/hash, recognizer suite/version, state, created time, candidates, warnings | Immutable after creation; later recognition creates another snapshot. |
| `FeatureCandidate` | stable ID, type, geometry references, dimensions, evidence, confidence, warnings, process/tool/setup candidates, acceptance state | ID is stable within snapshot only; cross-revision matching is a separate, non-authoritative relation. |
| `FeatureType` | family + subtype | Initial: planar face, hole variants, cylinder/cone, rotational indicator. Later: pocket, slot, boss, fillet/chamfer, turning and advanced-access classes. `unknown` is valid. |
| `GeometryReference` | namespace, body path, topology ID or mesh region/triangle-set handle, derivative hash | A reference must be resolvable against the source snapshot; never use transient renderer IDs. |
| `FeatureDimension` | named quantity, `Measurement`, semantic (`diameter`, `depth`, `axis`, etc.) | Dimensions include derivation and approximation. |
| `Evidence` | rule ID, input refs, thresholds/tolerances, intermediate measurements, recognizer version | Lets an estimator see why a candidate exists. |
| `ManufacturingCandidate` | process/tool/setup/orientation suggestion, assumptions, confidence, warning codes | Suggestions have no authority until a routing is reviewed. |
| `FeatureReviewDecision` | action, actor, time, reason, before/after feature data, links to original candidate | Actions: accept, reject, edit, merge, split, define-manually. Never delete automatic evidence. |

## State and history

`acceptance_state` is `proposed`, `accepted`, `rejected`, `edited`, `merged`, `split`, `manual`, or `superseded`. “Accepted” means the estimator selected it for this quote; it does not certify process safety. An edit keeps `original_candidate_id`, original values and a reason. Merges/splits reference all source IDs and create new review-owned IDs. Manual features have no fabricated recognizer confidence and are labelled `user_defined`.

## Tool, process, and setup associations

Feature candidates link to nonbinding candidates rather than a single conclusion:

```text
FeatureCandidate → [ToolCandidate] → tool constraints / reach assumptions
                 → [ProcessCandidate] → milling | turning | EDM | grinding | review
                 → [SetupOrientationCandidate] → axis/face + access evidence
```

Each association carries restrictions (minimum tool reach, maximum depth/diameter, approach direction, collision approximation, machine capability assumptions) and may be rejected independently. Missing drawing requirements, workholding, tolerance, finish, or material data add warnings rather than synthetic precision.

## Confidence contract

`FeatureConfidence { level, reasons, representation_ceiling, reviewed_override? }` uses four levels: high, medium, low, needs-review. It records the reason chain, including source representation, unit certainty, validity, healing, classifier ambiguity, mesh fit/resolution, and access-test coverage. Mesh-derived candidates cap at low. Any unresolved unit or topology error results in needs-review. The review override adds a human assertion but never erases the original confidence.

## Explicit non-goals

Feature candidates do not represent CAD history, feature-tree names, threads, GD&T, surface finish, manufacturing sequence, safe toolpaths, or approved operations. A `thread_candidate` is an observation requiring drawing confirmation—not a thread specification or tap choice. A `rotational_indicator` is not evidence that a part can be turned.
