# Model-fixture strategy

## Metadata

- **Status:** In Review
- **Last updated:** 2026-07-22
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

## Ground truth and tolerances

Use analytic construction or a reviewed independent CAD/calculation reference. Document coordinate system, dimensions, unit, expected tolerance and whether the value is exact or approximate. A mesh fixture records triangle count, resolution and expected error bound; a finer mesh is not automatically an equivalent fixture. Feature ground truth is annotated with geometry refs, expected candidate state, deliberate ambiguities, known false positives and known false negatives.

## Governance

Fixture owners review additions with a geometry maintainer and security reviewer for malicious/controlled sets. Track provenance in source control; store larger approved assets in an access-controlled artifact location with immutable hashes. Malformed inputs may be encrypted/restricted if sharing them increases risk; CI fetches them only in controlled environments. Deleting or changing a baseline needs a reason, replacement or explicit retirement, and a regression-history record.
