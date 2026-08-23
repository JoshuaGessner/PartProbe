# TASK-004 validation evidence

> **Status:** In Review
> **Last updated:** 2026-08-22
> **Related requirements:** REQ-F-002–REQ-F-004; GEO-002–GEO-004, GEO-006, GEO-010, GEO-013; TEST-003, TEST-022, TEST-025
> **Related ADRs:** ADR-0004, ADR-0005
> **Open questions:** Full 3MF component-graph/metadata scope; mesh self-intersection policy; reviewed confidence ceiling; production vertex-welding tolerance
> **Dependencies:** `model-fixture-strategy.md`, `geometry-validation.md`, `fixtures/manifest.yaml`
> **Supersedes:** None

## Checkpoint

The first five TASK-004 comparison slices implement path-free ASCII/binary STL and 3MF analyzers plus a persisted Core-unit corpus in `partprobe-geometry-import`. Callers provide an already-authorized byte slice plus explicit positive limits; no analyzer opens a source path or mutates application state. The STL entry point selects binary only when the 80-byte header, little-endian triangle count, and exact 50-byte record length agree; otherwise it applies strict ASCII grammar. STL parsing remains dependency-free. The 3MF path pins `quick-xml 0.41.0` and `zip 8.6.0` with ZIP defaults disabled and only Store/Deflate enabled; the dependency rationale and active graph are recorded in `../07-delivery/dependency-record.md`.

Versions `partprobe-ascii-stl-spike-v1` and `partprobe-binary-stl-spike-v1` produce provisional mesh evidence for:

- ASCII grammar or exact binary record framing identified from content and representation identified as mesh;
- unresolved/unknown source units requiring explicit confirmation;
- triangle count, axis-aligned extents, and surface area in source-coordinate units;
- undirected-edge manifold and watertight classification;
- paired-edge winding consistency; and
- enclosed volume and centroid only when the mesh is watertight, consistently wound, and has nonzero finite signed volume.

Binary records require exact framing, finite normal/vertex components, and a zero attribute field; unsupported per-facet attribute/color payloads fail closed rather than being silently discarded. Failures expose stable content-free diagnostic codes for invalid limits, input/triangle quota breaches, invalid text/structure/numbers, unsupported attribute data, degenerate triangles, and empty meshes. Non-finite intermediate measurements fail closed rather than entering evidence.

Version `partprobe-3mf-spike-v2` validates an in-memory ZIP/OPC package before reading the selected model. It requires the exact OPC content-types and root-relationships parts, resolves one internal StartPart target, verifies the model content type, and never extracts a part to disk. It rejects encrypted, non-file, unsafe-name, duplicated/case-fold-ambiguous, externally targeted, unsupported-compression, malformed XML, document-type/general-reference, unsupported-required-extension, and unsupported-model-structure inputs. Limits cover original bytes, entry count, aggregate expanded bytes, per-XML bytes, compression ratio, vertices, triangles, objects, and component references.

The bounded model contract accepts the 3MF Core namespace, every Core length unit plus its normative millimetre default, one leaf mesh object, optionally one later component object that references that mesh exactly once, and one build item. It enforces Core positive resource-ID bounds and prior-object component reference order. Vertices pass through the documented row-vector component transform, then the build transform, then canonical-millimetre conversion before entering the same format-neutral triangle measurement code as STL; this order follows the Consortium's [Core 1.4 matrix, component, and build rules](https://github.com/3MFConsortium/spec_core/blob/master/3MF%20Core%20Specification.md). Evidence distinguishes an explicit `unit` attribute from the Core default; retains leaf/component/build counts and package-local IDs; represents absent component evidence as `None`, not zero; and retains both exact 12-value source-unit transforms plus applied-state flags. Singular or reflected transforms, multiple component instances, nested component graphs, extra/unused objects, multiple build items, metadata/material extensions, and unknown structures fail visibly rather than being flattened, unioned, or ignored.

The v2 algorithm identity deliberately supersedes v1 because accepting and transforming a component changes geometry interpretation and adds provenance fields. This is a preproduction comparison API with no persisted or customer analysis records. Callers must rebuild against the expanded positive-limit constructor and must not reinterpret old v1 evidence as v2; no storage migration is required or implied.

## Governed regression evidence

| Fixture | Expected result | Automated result |
|---|---|---|
| `FIX-MESH-001` closed cube | 12 triangles; manifold/watertight/consistent; extents 10 × 10 × 10; area 600; volume 1,000; centroid 5,5,5; units unresolved | Pass |
| `FIX-MESH-002` open cube | 10 triangles; manifold but not watertight; extents 10 × 10 × 10; area 500; volume/centroid unavailable; units unresolved | Pass |
| `FIX-MESH-003` binary closed cube | Exact binary framing; same 12-triangle analytic geometry and unresolved-unit evidence as `FIX-MESH-001` | Pass |
| `FIX-MESH-004` 3MF closed cube | Explicit centimetres; one translated build item; 12 triangles; canonical extents 10 × 10 × 10 mm; area 600 mm²; volume 1,000 mm³; translated centroid 25,35,45 mm; mesh-only warning | Pass |
| `FIX-MESH-005` 3MF component cube | Explicit centimetres; mesh object 1 referenced once by component object 2; positive component X-scale plus translation before build translation; 12 triangles; canonical extents 20 × 10 × 10 mm; area 1,000 mm²; volume 2,000 mm³; centroid 60,75,95 mm; both transforms and IDs retained; mesh-only warning | Pass |
| `FIX-MESH-006`–`011` 3MF unit corpus | Persisted micron, millimetre, metre, inch, foot, and omitted-unit/default-millimetre packages; identity build; each reproduces the same canonical 10 × 10 × 10 mm, 600 mm², 1,000 mm³, `(5,5,5)` mm cube; explicit/default state retained | Pass |

The binary and 3MF fixtures are deterministically generated from `FIX-MESH-001`; their check modes prove all nine derived binary/package artifacts are reproducible before tests consume them, including decimal source coordinates for inch and foot. Focused validation passes seven STL tests, nine 3MF tests, four fixture-contract tests, and eight generator-tooling tests. Coverage includes the binary, direct, component, and persisted-unit encodings; content-based STL selection; shared deterministic mesh measurements; all six Core units plus the normative default; explicit/default distinction; identity and component-then-build transforms with retained ID provenance; positive archive/XML/entity/object/component limits; Core resource-ID bounds; exact binary framing; external/traversal relationship rejection; unsupported required extensions/material attributes/units/structures; invalid component references; multiple component rejection; singular/reflected transform rejection; unsupported binary attributes; malformed/non-finite input; empty input; and degenerate/overflowing geometry. Strict geometry-import Clippy passes.

## Evidence boundary

This is an internal comparison spike, not a supported importer. Binary evidence covers one simple zero-attribute analytic fixture. 3MF evidence now persists all six Core unit declarations plus the normative default, one project-generated translated direct mesh, and one single-reference component variant. Every unit package is another deterministic encoding of the same analytic cube, not independent geometry or CAD-tool evidence. Component graphs/unions, multiple build items, metadata/materials, reflected transforms, alternate legal OPC layouts, and a representative malformed corpus remain absent. The spike does not recognize self-intersections, assign an authoritative mesh-confidence score, enter the geometry-worker/desktop flow, persist a model, or validate representative machining parts. Exact floating-point vertex identity is sufficient for these analytic fixtures but is not yet the reviewed production welding/tolerance policy. TASK-004 remains In Progress until the broader STL/3MF validity/failure matrix and acceptance evidence exist.
