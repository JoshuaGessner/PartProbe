# TASK-004 validation evidence

> **Status:** In Review
> **Last updated:** 2026-08-22
> **Related requirements:** REQ-F-002–REQ-F-004; GEO-002–GEO-004, GEO-006, GEO-010, GEO-013; TEST-003, TEST-022, TEST-025
> **Related ADRs:** ADR-0004, ADR-0005
> **Open questions:** 3MF package/XML dependency and quotas; mesh self-intersection policy; reviewed confidence ceiling; production vertex-welding tolerance
> **Dependencies:** `model-fixture-strategy.md`, `geometry-validation.md`, `fixtures/manifest.yaml`
> **Supersedes:** None

## Checkpoint

The first two TASK-004 comparison slices implement path-free, dependency-free ASCII and binary STL analyzers in `partprobe-geometry-import`. Callers provide an already-authorized byte slice plus positive byte and triangle ceilings; the analyzer does not open a path, resolve external content, infer model units, or mutate application state. The combined entry point selects binary only when the 80-byte header, little-endian triangle count, and exact 50-byte record length agree; otherwise it applies strict ASCII grammar.

Versions `partprobe-ascii-stl-spike-v1` and `partprobe-binary-stl-spike-v1` produce provisional mesh evidence for:

- ASCII grammar or exact binary record framing identified from content and representation identified as mesh;
- unresolved/unknown source units requiring explicit confirmation;
- triangle count, axis-aligned extents, and surface area in source-coordinate units;
- undirected-edge manifold and watertight classification;
- paired-edge winding consistency; and
- enclosed volume and centroid only when the mesh is watertight, consistently wound, and has nonzero finite signed volume.

Binary records require exact framing, finite normal/vertex components, and a zero attribute field; unsupported per-facet attribute/color payloads fail closed rather than being silently discarded. Failures expose stable content-free diagnostic codes for invalid limits, input/triangle quota breaches, invalid text/structure/numbers, unsupported attribute data, degenerate triangles, and empty meshes. Non-finite intermediate measurements fail closed rather than entering evidence.

## Governed regression evidence

| Fixture | Expected result | Automated result |
|---|---|---|
| `FIX-MESH-001` closed cube | 12 triangles; manifold/watertight/consistent; extents 10 × 10 × 10; area 600; volume 1,000; centroid 5,5,5; units unresolved | Pass |
| `FIX-MESH-002` open cube | 10 triangles; manifold but not watertight; extents 10 × 10 × 10; area 500; volume/centroid unavailable; units unresolved | Pass |
| `FIX-MESH-003` binary closed cube | Exact binary framing; same 12-triangle analytic geometry and unresolved-unit evidence as `FIX-MESH-001` | Pass |

The binary fixture is deterministically generated from `FIX-MESH-001`; its check mode proves the committed 684-byte artifact is reproducible before tests consume it. Focused validation passes seven STL tests plus three generator-tooling tests covering all three golden fixtures, content-based ASCII/binary selection, deterministic warnings, positive parser limits, exact binary length/count framing, byte/triangle quotas, unsupported attributes, malformed/non-finite input, empty input, and degenerate/overflowing geometry. Strict crate-wide Clippy also passes.

## Evidence boundary

This is an internal comparison spike, not a supported importer. Binary evidence currently covers one simple zero-attribute analytic fixture, not color/attribute extensions or a representative malformed corpus. The spike does not implement 3MF, recognize self-intersections, assign an authoritative mesh-confidence score, confirm a physical unit, convert source-coordinate measurements, enter the geometry-worker/desktop flow, persist a model, or validate representative machining parts. Exact floating-point vertex identity is sufficient for these analytic fixtures but is not yet the reviewed production welding/tolerance policy. TASK-004 remains In Progress until the full STL/3MF unit/validity/failure matrix and acceptance evidence exist.
