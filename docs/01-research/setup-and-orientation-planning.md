# Setup and Orientation Planning Research

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirement IDs:** REQ-F-006, REQ-F-024, REQ-F-027, FEAT-006, FEAT-013, FEAT-021  
> **Related architecture decision IDs:** ADR-0005, ADR-0008  
> **Open questions:** Available workholding, machine envelopes/kinematics, fourth/fifth-axis capability, and shop datum conventions  
> **Dependencies:** geometry/feature confidence, machine and fixture libraries, drawing/GD&T review  
> **Supersedes:** None

## Position

Propose reviewable setup candidates; never assert manufacturability or generate a production fixture plan. The selected orientation, datum/locating proposal, clamp assumptions, and inaccessible regions are estimator inputs with confidence and warnings.

NIST notes that process planning may be automatic, manual, or hybrid and that feature-based planning is commonly manual at cell/workstation level. This supports a human-controlled planner. [NIST feature-based process planning](https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=901352)

## Candidate generation

1. Normalize units and validate geometry; retain source-model hash, healing actions, and model type.
2. Generate candidate approach directions from principal faces, feature axes, rotational axes, and user-specified directions. Mesh-derived normals have a lower confidence ceiling.
3. For each candidate, determine visible/approachable feature regions against a tool-and-holder envelope and a conservative fixture exclusion volume.
4. Test machine envelope, axis/rotary reach, travel, table/fixture capacity, and prohibited approach directions.
5. Score candidate groups by accessible work, setup count, datum continuity, re-clamp risk, expected rigidity, tool reach, collision uncertainty, and machine suitability.
6. Present ranked alternatives plus exclusion reasons. A user selects/edits a plan; the score is not a safety clearance.

## Required setup record

| Field | Rule |
| --- | --- |
| Orientation | Named coordinate transform plus machine/workholding relation, not an image alone. |
| Feature/operation coverage | Each operation links to one setup and has `accessible`, `uncertain`, or `not_accessible` status. |
| Datum and location assumption | Proposed primary/secondary/tertiary or turning reference; mark drawing-driven relationships unavailable when no drawing is present. |
| Workholding | stock grip, jaws/fixture/supports, clamp zones, soft-jaw/fixture need, and a rigidity confidence. |
| Tool envelope | tool, holder, maximum reach/stickout, approach direction, collision uncertainty. |
| Transfer/reorientation | explicit operation and time; preserve WCS/probe/re-reference requirements. |
| Evidence | geometry revision, algorithm version, machine/fixture-library versions, override trail. |

## Hard exclusions and warnings

- No stock overlap or valid holding region; required feature lies inside conservative fixture/tool-holder collision volume; machine envelope or axis limit is exceeded; or selected process is incompatible with the machine profile.
- Part has thin walls, low clamp area, unsupported deep cavities, long tool reach, undercuts, freeform surfaces, or drawing-driven datum/tolerance requirements that the model does not express.
- Five-axis, mill-turn, Swiss, and transfer plans require a capable machine profile and manual confirmation; present alternatives instead of collapsing them to a three-axis setup count.

## Cost implications

Setup count alone is insufficient. Estimate each setup’s loading, location/probing, jaw/fixture preparation, first-off/prove-out, transfer, inspection/re-reference, and teardown separately. Flag reusable fixtures versus job-specific work. A second setup may reduce tool reach or tolerance risk, so the planner must expose trade-offs rather than minimize count blindly.

## Research basis

- NIST’s reference data distinguishes machining options, tool catalog/inventory, machining features, and a process plan; setup planning should depend on these separately versioned inputs. [NIST feature-based control](https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=820582)
- Peer-reviewed setup-planning literature identifies access direction, fixture locating surfaces, operation precedence, machine capability, and tolerances as coupled constraints. This is research evidence, not a formal requirement. [Machine/fixture-constrained setup generation](https://www.sciencedirect.com/science/article/pii/S0924013604001074), [integrated setup/fixture planning](https://www.tandfonline.com/doi/full/10.1080/00207543.2010.543172)
