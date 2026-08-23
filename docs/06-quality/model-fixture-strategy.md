# Model-fixture strategy

## Metadata

- **Status:** In Review
- **Last updated:** 2026-08-22
- **Related requirement IDs:** GEO-001–GEO-014, FEAT-001–FEAT-021, TEST-021–TEST-039, SEC-004
- **Related architecture decision IDs:** ADR-0002, ADR-0004, ADR-0005
- **Open questions:** What anonymized customer models can be contributed? Licensing policy for external samples? Artifact-store size budget?
- **Dependencies:** Controlled fixture repository/storage, expected-result schema, legal/release review
- **Supersedes / superseded by:** None / none

## Policy

Fixtures are versioned test data, not convenient sample CAD. Public fixtures must be legally redistributable, non-controlled, minimal, documented, hashed, and paired with reviewed expected results. The current bootstrap cubes are not approved for redistribution until the project license decision and review fields are resolved. Do not commit customer, defense, export-controlled, price, drawing, or identifiable RFQ data. Reviewed synthetic analytic fixtures become the authoritative baseline for measurement accuracy.

## Corpus layout and manifest

```text
fixtures/
  manifest.yaml
  models/<descriptive-source-name>.<ext>
  expected/<fixture-id-or-name>.json
  private/  # ignored; governed shop fixtures are never assumed distributable
```

The root manifest is the canonical registry and links each source to its expected file. It includes fixture ID/title, source SHA-256, origin/license/redistribution approval, sensitivity classification, format/schema/version, declared units, canonical units, intended representation, purpose/tags, expected worker outcome, resource ceiling, owner, reviewed date, and links to issue/requirements as those fields become applicable. Expected files identify analysis/feature profile and algorithm versions. A fixture update is a code-reviewable change, never a silent replacement.

Expectation schema version 2 encodes exact numeric values as decimal strings and every potentially missing measurement as an explicit `available`, `unavailable`, or `not_applicable` evidence state. Diagnostic codes explain unavailable evidence; absence never means zero. The schema rejects mesh records without triangle counts, confirmed unknown units, duplicate warnings, negative area/volume, and authoritative enclosed volume for a known open mesh.

Version 2 is a preproduction fixture-contract migration with no customer or persisted production records. Version 1 files are rejected and require deliberate, reviewed conversion; the loader does not silently infer evidence states or reinterpret old numeric values.

## Initial suite

| Family | Mandatory cases | Primary validation |
|---|---|---|
| Analytic exact | cube, rectangular prism, cylinder, cone, tube, stepped turn profile, multi-body transform | units, volume/area/centroid/AABB/OBB and rotational indicator |
| STEP | AP203/214/242 single solid; named/color body; assembly; partial transfer; invalid shell | transfer report, attributes/provenance, exact measures |
| Mesh | watertight cube/cylinder at two resolutions; open/non-manifold/reversed/degenerate/self-intersecting inputs | mesh validation, unit prompt, approximate measurements/confidence ceiling |
| 3MF | all supported units, component transform/build, metadata, unsupported extension, unsafe archive | unit/read behavior and archive controls |
| IGES / OBJ | legacy surface/solid, missing assets, external-path attempt | secondary-format warnings and containment |
| Feature parts | through/blind hole, counterbore/countersink, pocket/open pocket, slot, boss, thin wall, turning profile, ambiguous blends | type/dimensions/evidence/confidence and visual mapping |
| Adversarial | malformed records, giant counts, recursion/compression bombs, invalid UTF/XML, worker fault injection | quotas, controlled failure, no UI crash |

`FIX-STEP-001` is a project-generated AP214 10 mm cube with declared millimetres, one solid, 600 mm² surface area, 1000 mm³ volume, and centroid `(5, 5, 5)` mm. The committed feature-gated generator uses the pinned OCCT 8.0.0 build and normalizes the volatile STEP header timestamp to `2000-01-01T00:00:00`; repeated generation produced SHA-256 `031304b3a6d9dd55a97b3329e7238286ccfdaa7f13030bbe6e5c4c5744fcc8a2`.

`FIX-STEP-002` is a small project-authored STEP envelope containing one intentionally invalid entity, SHA-256 `b0bb25521620d14798110de10dc78739e18d28cd160a10a16c4163a269ca1157`. Its separate failure-expectation schema version 1 requires recoverable `STEP_TRANSFER_FAILED`, no snapshot or output file, and removal of the staged input. It carries no geometry measurements or unit claims. This schema is additive and does not migrate or reinterpret schema-v2 successful-geometry expectations; unsupported failure-schema versions are rejected.

`FIX-STEP-003` is a project-authored ISO 10303-21 AP214 faceted B-rep rectangular prism. It is written directly rather than exported by OCCT, declares millimetres, and has analytic dimensions 12 × 8 × 5 mm, area 392 mm², volume 480 mm³, and centroid `(6, 4, 2.5)` mm. Its SHA-256 is `a3a2cceef68a98212a2b05ac376da747758cc360fb02085fee3f6db766dc2138`. Native adapter and supervised-worker tests reproduce those values within 0.000001, but `reviewed_date` remains unset until geometry and security reviewers accept the authoring/provenance record.

`FIX-MESH-001/002` exercise the bounded `partprobe-ascii-stl-spike-v1` comparison analyzer under explicit 65,536-byte and 1,000-triangle ceilings. The closed cube reproduces 12 triangles, 10 × 10 × 10 source-coordinate extents, 600 square source units, 1,000 cubic source units, and centroid `(5, 5, 5)`; the open cube reproduces 10 triangles and 500 square source units while keeping volume and centroid unavailable. `FIX-MESH-003` is a deterministic 684-byte binary encoding of the same closed analytic cube, generated from `FIX-MESH-001` and reproduced by a checked script; `partprobe-binary-stl-spike-v1` returns the same geometry and unresolved-unit evidence. These fixtures do not establish binary attribute/color-extension handling, 3MF, representative-part, desktop, or production-importer support.

Because OCCT generates and reads `FIX-STEP-001`, it proves deterministic project plumbing and analytic measurement reconciliation, not independent translator correctness. `FIX-STEP-003` removes that same-generator dependency for one simple solid, but it remains a project-authored manual file interpreted by one kernel—not a broad translator or production accuracy corpus. `FIX-STEP-002` adds one adversarial transfer/cleanup case, not a comprehensive malformed-input corpus. Release accuracy and containment still need independently reviewed CAD-tool exports plus broader malformed, alternate-schema, assembly, partial-transfer, and resource-limit cases.

## Ground truth and tolerances

Use analytic construction or a reviewed independent CAD/calculation reference. Document coordinate system, dimensions, unit, expected tolerance and whether the value is exact or approximate. A mesh fixture records triangle count, resolution and expected error bound; a finer mesh is not automatically an equivalent fixture. Feature ground truth is annotated with geometry refs, expected candidate state, deliberate ambiguities, known false positives and known false negatives.

## Governance

Fixture owners review additions with a geometry maintainer and security reviewer for malicious/controlled sets. Track provenance in source control; store larger approved assets in an access-controlled artifact location with immutable hashes. Malformed inputs may be encrypted/restricted if sharing them increases risk; CI fetches them only in controlled environments. Deleting or changing a baseline needs a reason, replacement or explicit retirement, and a regression-history record.
