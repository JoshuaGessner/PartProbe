# Mesh topology and tolerance policy

## Metadata

- **Status:** In Review
- **Last updated:** 2026-08-31
- **Related requirement IDs:** GEO-003–GEO-006, GEO-010, GEO-013, TEST-003, TEST-021–TEST-025
- **Related architecture decision IDs:** ADR-0004, ADR-0005
- **Open questions:** Production STL welding threshold and scale rule; approved near-contact diagnostic; governed repair workflow; location-reference contract
- **Dependencies:** `cad-file-formats.md`, `geometry-analysis-model.md`, `geometry-engine.md`, `task-004-validation.md`
- **Supersedes / superseded by:** None / none

## Decision boundary

PartProbe does not currently weld mesh vertices or repair near contacts. The comparison analyzers expose `partprobe-mesh-topology-policy-v1`, a typed topology identity, and `welding_status: not_applied` so a caller cannot mistake missing policy evidence for a zero tolerance or a successful repair.

- 3MF topology uses the retained Core vertex indices. Equal coordinates at distinct indices remain distinct vertices.
- STL has no source index authority. The current spike compares canonicalized exact source-coordinate bits and leaves physical units unresolved.
- Neither behavior is a production machining tolerance, exporter-compatibility policy, or authorization to mutate the source.

The 3MF Core specification defines mesh edges and orientation through vertex indices and recommends collapsing very close vertices only when appropriate; it does not define one universal numeric tolerance. Mesh-repair libraries such as CGAL expose duplicate-point merging, stitching, and orientation as explicit operations that change connectivity. PartProbe therefore keeps validation, proximity diagnostics, and repair as separately versioned steps rather than silently merging them.

## Required future policy shape

A production tolerance profile must be explicit, versioned, and applied only after physical units and scale are resolved. It must record:

- the absolute canonical-unit threshold and any relative or scale-dependent rule;
- the source, reviewer, approval state, and intended importer/material/process scope;
- whether the operation is diagnostic-only or geometry-mutating;
- topology counts and stable affected references before and after the operation;
- the number of merged vertices/edges, warnings, confidence effects, and derivative hash; and
- the original source hash, which remains authoritative and is never replaced by a repair derivative.

A near-contact diagnostic may report candidates without changing topology. A welding or stitching operation creates a derivative and requires a `HealingRecord`; it cannot inherit the diagnostic policy implicitly. Logs remain content-free. Any user-facing locations must use bounded path-free triangle, edge, or source-vertex references rather than coordinates or model names.

## Acceptance evidence required before a numeric default

The fixture matrix must include distances below, exactly at, and above the proposed threshold; equivalent millimetre/inch and scaled cases; intended open seams; distinct surfaces that must not merge; thin-feature and non-manifold counterexamples; deterministic merge ordering; and explicit before/after topology, self-intersection, confidence, and closed-measurement outcomes. Representative exporter and shop parts require legal review outside public CI.

Changing the current `not_applied` boundary requires a new topology-policy version, parser/evidence migration decision, governed fixtures, regression tests, confidence behavior, and updated worked evidence. Existing v3 STL and v8 3MF results must not be reinterpreted as welded.

## Integration consequence

The next mesh worker/application/desktop slice must carry topology-policy version, topology identity, welding status, optional measurements, unit state, confidence reasons, and sanitized warnings through a versioned typed result. STL must remain unavailable for dimension-dependent estimating until units are explicitly confirmed. The existing exact-B-rep result and calculation path remain unchanged; mesh analysis must not be coerced into its mandatory volume/centroid contract or converted to zero.

## Primary references

- [3MF Core Specification 1.4.0](https://github.com/3MFConsortium/spec_core/blob/master/3MF%20Core%20Specification.md)
- [CGAL Polygon Mesh Processing: repairing polygon soups and meshes](https://doc.cgal.org/latest/Polygon_mesh_processing/)
- [CGAL Polygon Mesh Processing mesh-repair examples](https://doc.cgal.org/latest/PMP_Mesh_repair/index.html)
