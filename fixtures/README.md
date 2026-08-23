# Public Synthetic Fixtures

These fixtures are deliberately simple and contain no customer or shop data. They bootstrap the test harness; they do not validate production CAD coverage.

- `models/cube_10mm_ascii.stl`: closed triangular cube, unitless by STL definition; fixture convention proposes millimetres but import must require/record confirmation.
- `models/open_cube_10mm_ascii.stl`: same cube with one face missing, intended to trigger non-watertight/open-boundary diagnostics.
- `expected/cube_10mm.json`: reviewed mathematical expectations and tolerances, not parser output.

The bounded `partprobe-ascii-stl-spike-v1` comparison analyzer reproduces the two mesh fixtures' source-coordinate triangle, bounds, area, closure, and winding evidence. It deliberately leaves STL units unresolved and exposes enclosed volume/centroid only for the closed, consistently wound cube. This is ASCII-only developer evidence, not a supported importer.

`manifest.yaml` is authoritative for fixture provenance, hashes, classification, parser limits, and expected-result files. Project redistribution licensing, domain-expert review, binary STL/3MF fixtures, and representative machining parts remain required; do not manufacture questionable hand-written fixtures and call them representative.
