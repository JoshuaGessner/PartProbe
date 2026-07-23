# Feature-recognition pipeline architecture

## Metadata

- **Status:** In Review
- **Last updated:** 2026-07-22
- **Related requirement IDs:** REQ-F-023–REQ-F-024, FEAT-001–FEAT-021, GEO-010–GEO-014, TEST-031–TEST-039
- **Related architecture decision IDs:** ADR-0002, ADR-0005
- **Open questions:** Recognition tolerance/profile ownership? Initial shop feature taxonomy? How to version cross-revision feature matches?
- **Dependencies:** Validated geometry snapshot, tool/machine libraries, feature model, visual review UX, annotated fixtures
- **Supersedes / superseded by:** None / none

## Design

The pipeline converts validated geometry into evidence-backed candidates, then hands control to the estimator. It runs independently from UI and routing calculation. Exact B-rep and mesh inputs branch immediately and never share a confidence policy.

```text
GeometryAnalysisSnapshot
  → eligibility gate (unit, validity, representation)
  → topology / mesh-region normalization
  → primitive and adjacency classification
  → candidate generators (hole | prismatic | turning | access)
  → conflict consolidation and measurements
  → manufacturing candidate rules
  → confidence + warnings
  → immutable FeatureAnalysisSnapshot → visual/user review → routing inputs
```

## Eligibility and deterministic behavior

- Exact path requires confirmed units and a representation supported by the selected recognizer. Failed/partial imports may yield `needs-review` observations but no high-confidence features.
- Mesh path requires a recorded mesh-validation state and resolution metrics; it generates only explicitly enabled approximate primitives.
- Each recognizer receives a `RecognitionProfile` (length/angular tolerances, supported topology, candidate limits and version). Profiles are approved configuration, not hidden constants.
- Results sort deterministically by body, geometric reference and rule order. A stable candidate ID is a UUID derived from snapshot ID + recognizer/rule version + canonical evidence fingerprint; collision resolution is recorded.
- Generators may overlap. Consolidation retains source candidates and declares the merge rule, so a UI can explain each final proposal.

## Candidate generators

| Generator | Evidence | Output / guardrail |
|---|---|---|
| Hole family | cylinder/cone axis, end loops, adjacency, depth / diameter | through/blind/counterbore/countersink/spotface candidate; no thread claim |
| Prismatic | planar boundaries, concavity/convexity, boundary openness, depth direction | pocket, open pocket, slot, step, boss candidate; blends cause ambiguity warning |
| Turning | fitted/revolved axis, profile regions, coaxial cylinders/cones | OD/ID/face/shoulder/groove/bore/taper indicators; not a route decision |
| Access / reach | candidate approach directions, envelope/neighbor collision approximation, feature dimensions | blocked/long-reach/deep/small-tool risk; excludes fixtures and full tool/holder collision |
| Process indicators | aggregate geometry, access, body state | milling/turning/mill-turn/EDM/Swiss *alternatives*; never sole automatic selection |

## Confidence and stopping rules

The pipeline emits reasons as structured codes, displays a label, and applies a representation ceiling: exact valid solid can be high; exact but healed/ambiguous is medium; mesh is at most low; unresolved units, invalid geometry, unsupported entity, or candidate conflict is needs-review. No aggregation upgrades a child beyond the weakest critical input. A candidate limit, timeout or unsupported condition creates a visible incomplete-analysis warning—not an absent-feature inference.

## Review and downstream integration

The viewer highlights geometry references; the feature inspector shows evidence, dimensions, warnings, suggested tools/processes/setups, and confidence. Accept/reject/edit/merge/split/manual actions generate an append-only review event. Routing consumes only accepted or user-defined feature versions selected by the estimator, and stores their originating snapshot/version. Reanalysis proposes a new snapshot and presents a diff; it cannot overwrite an approved routing.

## Observability and privacy

Record counts, rule timings, result states, profile/algorithm versions and safe error codes. Do not export models, vertices, topology labels, file names, or customer identifiers to telemetry. Debug geometry capture is a local, explicit, permissioned support operation with retention controls.
