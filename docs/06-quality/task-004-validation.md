# TASK-004 validation evidence

> **Status:** In Review
> **Last updated:** 2026-08-22
> **Related requirements:** REQ-F-002–REQ-F-004; GEO-002–GEO-004, GEO-006, GEO-010, GEO-013; TEST-003, TEST-022, TEST-025
> **Related ADRs:** ADR-0004, ADR-0005
> **Open questions:** Binary STL parser choice; 3MF package/XML dependency and quotas; mesh self-intersection policy; reviewed confidence ceiling
> **Dependencies:** `model-fixture-strategy.md`, `geometry-validation.md`, `fixtures/manifest.yaml`
> **Supersedes:** None

## Checkpoint

The first TASK-004 comparison slice implements a path-free, dependency-free ASCII STL analyzer in `partprobe-geometry-import`. Callers provide an already-authorized byte slice plus positive byte and triangle ceilings; the analyzer does not open a path, resolve external content, infer model units, or mutate application state.

Version `partprobe-ascii-stl-spike-v1` produces provisional mesh evidence for:

- content grammar identified as ASCII STL and representation identified as mesh;
- unresolved/unknown source units requiring explicit confirmation;
- triangle count, axis-aligned extents, and surface area in source-coordinate units;
- undirected-edge manifold and watertight classification;
- paired-edge winding consistency; and
- enclosed volume and centroid only when the mesh is watertight, consistently wound, and has nonzero finite signed volume.

Failures expose stable content-free diagnostic codes for invalid limits, input/triangle quota breaches, invalid text/structure/numbers, degenerate triangles, and empty meshes. Non-finite intermediate measurements fail closed rather than entering evidence.

## Governed regression evidence

| Fixture | Expected result | Automated result |
|---|---|---|
| `FIX-MESH-001` closed cube | 12 triangles; manifold/watertight/consistent; extents 10 × 10 × 10; area 600; volume 1,000; centroid 5,5,5; units unresolved | Pass |
| `FIX-MESH-002` open cube | 10 triangles; manifold but not watertight; extents 10 × 10 × 10; area 500; volume/centroid unavailable; units unresolved | Pass |

Focused validation passes four tests covering both golden fixtures, deterministic warnings, positive parser limits, byte/triangle quotas, malformed/non-finite input, empty input, and degenerate/overflowing geometry. Strict crate-wide Clippy also passes.

## Evidence boundary

This is an internal comparison spike, not a supported importer. It does not implement binary STL or 3MF, recognize self-intersections, assign an authoritative mesh-confidence score, confirm a physical unit, convert source-coordinate measurements, enter the geometry-worker/desktop flow, persist a model, or validate representative machining parts. Exact floating-point vertex identity is sufficient for these analytic fixtures but is not yet the reviewed production welding/tolerance policy. TASK-004 remains In Progress until the full STL/3MF unit/validity/failure matrix and acceptance evidence exist.
