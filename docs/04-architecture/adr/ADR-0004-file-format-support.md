# ADR-0004: Initial CAD file-format support

## Metadata

- **Status:** In Review
- **Last updated:** 2026-08-27
- **Related requirement IDs:** REQ-F-021–REQ-F-024, GEO-001–GEO-009, SEC-004
- **Related architecture decision IDs:** ADR-0002, ADR-0005
- **Open questions:** Shop format distribution, target max size, assembly scope, licensed translator budget, supported negative-determinant 3MF orientation/volume policy
- **Dependencies:** Format fixture scorecard, OCCT worker, 3MF/STL parser choice, legal review
- **Supersedes / superseded by:** None / none

## Proposed decision

Propose support for **STEP, STL, and 3MF** in the first vertical slice, pending this ADR's evidence gates. STEP is the only proposed initial exact-engineering interchange format. STL and 3MF would be accepted as mesh inputs, each with format-specific unit/validation behavior and a mesh confidence ceiling. Add IGES and OBJ only after primary-format validation as secondary preview/review imports. Require conversion to STEP for native CAD, Parasolid XT and ACIS/SAT until a commercial translator/kernel proves availability, license and packaging.

3MF defines units and packaged model metadata/components; its published public specifications are royalty-free for necessary patent claims, subject to separate implementation-library review. [3MF specification](https://3mf.io/spec/) OCCT documents STEP and IGES translation. [OCCT data exchange](https://dev.opencascade.org/doc/overview/html/index.html)

## Consequences

The scope delivers useful RFQ coverage while avoiding unsupported proprietary-format promises. Users receive a clear request for STEP AP242/AP214 (and drawing) when a native file arrives. All imports retain original source hash, declared/detected format, unit resolution, transfer/healing report and algorithm versions. Format enablement is feature-gated, not extension-gated.

The current TASK-004 comparison parser is deliberately narrower than Core 1.4 for transforms: it fails closed on singular matrices and on negative determinants because it does not yet implement and validate the required orientation/volume-sign behavior. A negative determinant is not classified as malformed or Core-invalid. Enabling it requires a reviewed geometry rule, parser-version/migration decision, positive fixtures with exact oriented-volume evidence, regression coverage, and downstream confidence behavior.

## Approval evidence required

For every enabled format: benign and malformed fixture corpus; verified unit behavior; expected validity/measurements; time/memory quota results; companion-file/external-reference behavior; Windows/Linux/macOS package test; source/implementation licenses; and review UX for every warning. See `../../01-research/cad-file-formats.md` for the canonical matrix.
