# Public Synthetic Fixtures

These fixtures are deliberately simple and contain no customer or shop data. They bootstrap the test harness; they do not validate production CAD coverage.

- `models/cube_10mm_ascii.stl`: closed triangular cube, unitless by STL definition; fixture convention proposes millimetres but import must require/record confirmation.
- `models/cube_10mm_binary.stl`: deterministic binary encoding of the same governed cube, reproduced from the ASCII source by `scripts/generate_binary_stl_fixture.py`; it carries the same unit uncertainty.
- `models/open_cube_10mm_ascii.stl`: same cube with one face missing, intended to trigger non-watertight/open-boundary diagnostics.
- `models/cube_1cm_translated.3mf`: deterministic 3MF Core/OPC package derived from the governed cube, with explicit centimetre units and one build translation to `(20, 30, 40)` mm; reproduced by `scripts/generate_3mf_fixture.py`.
- `expected/cube_10mm.json`: reviewed mathematical expectations and tolerances, not parser output.
- `expected/cube_10mm_binary.json`: the separately identified binary-encoding expectation with the same analytic geometry.
- `expected/cube_1cm_translated_3mf.json`: canonical-millimetre expectations proving declared-unit conversion and build-transform application.

The bounded ASCII/binary STL comparison analyzers reproduce the governed mesh fixtures' source-coordinate triangle, bounds, area, closure, and winding evidence. They deliberately leave STL units unresolved. The bounded 3MF comparison analyzer validates the package relationship/content-type path, applies explicit archive/XML/entity/compression limits, converts the declared centimetre model into canonical millimetres, and applies its one direct build-item transform. Both paths expose enclosed volume/centroid only for closed, consistently wound geometry. This is synthetic developer evidence, not a supported importer.

`manifest.yaml` is authoritative for fixture provenance, hashes, classification, parser limits, and expected-result files. Project redistribution licensing, domain-expert review, additional binary STL edge cases, broader 3MF unit/component/adversarial fixtures, and representative machining parts remain required; do not manufacture questionable hand-written fixtures and call them representative.
