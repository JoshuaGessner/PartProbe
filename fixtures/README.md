# Public Synthetic Fixtures

These fixtures are deliberately simple and contain no customer or shop data. They bootstrap the test harness; they do not validate production CAD coverage.

- `models/cube_10mm_ascii.stl`: closed triangular cube, unitless by STL definition; fixture convention proposes millimetres but import must require/record confirmation.
- `models/cube_10mm_binary.stl`: deterministic binary encoding of the same governed cube, reproduced from the ASCII source by `scripts/generate_binary_stl_fixture.py`; it carries the same unit uncertainty.
- `models/open_cube_10mm_ascii.stl`: same cube with one face missing, intended to trigger non-watertight/open-boundary diagnostics.
- `models/cube_1cm_translated.3mf`: deterministic 3MF Core/OPC package derived from the governed cube, with explicit centimetre units and one build translation to `(20, 30, 40)` mm; reproduced by `scripts/generate_3mf_fixture.py`.
- `models/cube_1cm_component_scaled_translated.3mf`: deterministic variant that references the mesh through one component object, applies a positive-determinant component scale/translation, then applies a build translation; reproduced by the same script.
- `models/cube_10mm_3mf_{micron,millimeter,meter,inch,foot,default_mm}.3mf`: deterministic direct-mesh unit corpus. Together with the explicit-centimetre fixture, these persist all six Core unit declarations plus the normative millimetre default while preserving identical canonical 10 mm geometry.
- `models/cube_10mm_3mf_metadata.3mf`: deterministic explicit-millimetre cube with three public well-known Core model metadata entries. Analysis retains only entry/preservation counts and a not-interpreted warning, never metadata names or values.
- `expected/cube_10mm.json`: reviewed mathematical expectations and tolerances, not parser output.
- `expected/cube_10mm_binary.json`: the separately identified binary-encoding expectation with the same analytic geometry.
- `expected/cube_1cm_translated_3mf.json`: canonical-millimetre expectations proving declared-unit conversion and build-transform application.
- `expected/cube_1cm_component_scaled_translated_3mf.json`: analytic 20 × 10 × 10 mm expectations proving component-then-build transform order and retained component provenance.
- `expected/cube_10mm_3mf_*.json`: separate fixture identities and canonical 10 mm expectations for each persisted unit/default package; the shared geometry does not make these independent shape fixtures.
- `expected/cube_10mm_3mf_metadata.json`: canonical cube evidence plus the required metadata-not-interpreted warning for `FIX-MESH-012`.

The bounded ASCII/binary STL comparison analyzers reproduce the governed mesh fixtures' source-coordinate triangle, bounds, area, closure, and winding evidence. They deliberately leave STL units unresolved. The bounded 3MF comparison analyzer validates the package relationship/content-type path, applies explicit archive/XML/entity/compression/object/component/model-metadata limits, converts every Core unit declaration or normative default into canonical millimetres, and accepts either one direct mesh/build item or one mesh referenced through one component object before the build item. Well-known unprefixed model metadata is counted and warned but never retained or interpreted. Both paths expose enclosed volume/centroid only for closed, consistently wound geometry. Multiple component instances, nested component graphs, object/item/vendor metadata, reflected/singular transforms, and build unions still fail visibly. This is synthetic developer evidence, not a supported importer.

`manifest.yaml` is authoritative for fixture provenance, hashes, classification, parser limits, and expected-result files. Project redistribution licensing, domain-expert review, additional binary STL edge cases, broader 3MF object/item/vendor metadata, multi-instance/adversarial fixtures, and representative machining parts remain required; do not manufacture questionable hand-written fixtures and call them representative.
