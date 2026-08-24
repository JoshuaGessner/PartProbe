# Geometry-analysis domain model

## Metadata

- **Status:** In Review
- **Last updated:** 2026-08-24
- **Related requirement IDs:** REQ-F-021–REQ-F-024, GEO-001–GEO-014, DATA-011–DATA-018, SEC-004
- **Related architecture decision IDs:** ADR-0002, ADR-0004, ADR-0005
- **Open questions:** Canonical tolerance policy? Required retention period for derivatives? First-slice maximum input size?
- **Dependencies:** `cad-file-formats.md`, `geometry-engine.md`, persistence model, security model
- **Supersedes / superseded by:** None / none

## Purpose and invariants

`GeometryAnalysisSnapshot` is an immutable, versioned interpretation of one source-model revision. It is evidence for estimating; it is not the CAD model, a drawing requirement, a released process plan, or a production CAM simulation. An analysis may be rerun, but an approved quote continues to point at the snapshot it used.

Invariants: source bytes are retained or externally content-addressed by SHA-256; all computed linear values state units and are stored canonically in millimeters; mass needs a selected material-density source; exact and mesh provenance never merge; repaired derivatives never replace the source; all results identify algorithm and dependency versions.

## Aggregate model

```text
PartRevision
 ├─ ModelAsset (source hash, format, controlled location, received metadata)
 └─ GeometryAnalysisSnapshot [0..n]
     ├─ ImportRecord / UnitResolution / HealingRecord
     ├─ GeometryRepresentation [one or more bodies]
     ├─ ValidationReport
     ├─ BasicProperties
     ├─ OrientationCandidate [0..n]
     ├─ StockEnvelopeCandidate [0..n]
     ├─ FeatureAnalysisSnapshot [0..1]
     ├─ Warning [0..n] and Diagnostic [0..n]
     └─ AnalysisProvenance / ReviewState
```

| Entity | Required fields | Notes |
|---|---|---|
| `ModelAsset` | `id`, `part_revision_id`, format detected/declared, SHA-256, byte size, source filename (display only), received timestamp, retention classification | Store controlled-path/object-store locator separately from user-visible name. |
| `UnitResolution` | imported unit, candidate units/scales, resolved unit, conversion factor to mm, method (`declared`, `confirmed`, `inferred`), confirmer, reason | Unit ambiguity blocks approval-dependent measurements. |
| `GeometryRepresentation` | body-scoped representation (`exact_brep`, `mesh`, `unknown`), body/shell counts, kernel/importer IDs, geometry reference namespace; snapshot may summarize `mixed` | Never label a mesh exact because it originated from CAD. `mixed` is aggregate metadata, never a measurement basis. |
| `ValidationReport` | state, solid validity, watertightness, manifold state, degenerate/self-intersection counts where available, transfer status | Each field has `not_applicable` / `unknown` distinct from pass. |
| `BasicProperties` | AABB, OBB candidates, volume, surface area, centroid, principal axes when available, min local thickness status, complexity indicators | Every value is `Measurement`, below. |
| `Measurement` | value, unit, derivation rule ID, representation basis, tolerance/approximation, status, confidence/reasons | `unknown` must not serialize as zero. |
| `HealingRecord` | action, parameters, before/after hashes or derivative IDs, affected refs, outcome, operator/algorithm | Healing is never silent. |
| `Warning` | stable code, severity, stage, affected refs, user-facing message, remediation, blocking flag, status | Diagnostics must exclude CAD coordinates/body names unless permitted. |
| `AnalysisProvenance` | analysis ID, started/ended, algorithm versions, kernel/library builds, OS/arch, worker profile, input and derivative hashes | Enables reproducibility and incident triage. |
| `ReviewState` | `proposed`, `needs_input`, `reviewed`, `approved_for_quote`, `superseded`, reviewer/time, decisions | Approval scopes the snapshot, not later reruns. |

## Geometry results and derivations

- **AABB:** min/max of representation coordinates after confirmed unit transform; stable and inexpensive.
- **OBB:** named algorithm and candidate orientation, never a claim of globally minimum stock unless that exact algorithm is used and recorded.
- **Exact volume/area/centroid:** only an accepted valid solid/closed exact topology. Surface area is not a machining-area estimate.
- **Mesh volume:** only a closed, consistently oriented, non-self-intersecting mesh under an explicitly named signed-volume method; label `approximate_mesh`.
- **Mass:** `volume × selected density`; density, material condition and source must be explicit. Missing density produces no mass.
- **Removed volume:** `stock volume − part volume` only if the approved stock envelope encloses the part in the recorded coordinate frame. A negative result is a validation error, not a signed manufacturing fact.

TASK-003's OCCT worker output is explicitly a `provisional_spike` schema, not `GeometryAnalysisSnapshot` authority. When a validated response carries the provisional opaque reference, the supervisor claims its output as bounded immutable bytes, computes and exposes SHA-256 and length evidence, binds that evidence to the reference, and removes the worker-visible pathname. The decoder then validates the complete schema and requires its source hash to match the authorized source before returning typed evidence; response status remains authoritative about whether the artifact is successful, partial, or failed, and no persistence or approval follows automatically. The `occt-step-spike` 1.0.0 profile retains native `f64` values inside the adapter, rejects non-finite or negative area/volume, and serializes display/test evidence using fixed six-decimal canonical strings with trailing zeros removed. The snapshot records `decimal_scale: 6`; the analytic fixture uses absolute tolerance `0.000001`. This temporary rule prevents platform-noise strings such as `599.9999999999999` from masquerading as meaningful precision. Production measurement policy still requires dimension-specific tolerance, derivation/version evidence, and an accepted snapshot schema.

Checkpoint 19 centralizes the pre-existing schema-v1 wire shape in `geometry-core`; it does not change field names, geometry interpretation, decimal behavior, fixture expectations, or persistence. Mixed/older/unsupported evidence remains rejected rather than migrated implicitly. There are still no persisted/customer snapshot records to migrate.

Checkpoint 20 does not change that interpretation or schema. It adds construction provenance for the exact native dependency and exercises the same rules with `FIX-STEP-003`, a manually authored faceted B-rep rectangular prism whose analytic area, volume, and centroid differ from the OCCT-generated cube. This is additional provisional evidence, not acceptance of the snapshot as authoritative geometry.

## Stage outcome contract

Every geometry stage returns `StageOutcome { status, outputs, warnings, confidence, timing, algorithm_version, diagnostics }`. `status` is `succeeded`, `succeeded_with_warnings`, `needs_user_input`, `failed_recoverable`, or `failed_terminal`. Stages may preserve useful earlier outputs after a downstream failure; no failed/partial output can masquerade as authoritative.

## Confidence and review

Confidence has a level and reasons, not one arithmetic score. Each body, measurement, and derived feature starts at its own representation ceiling, then has explicit reductions for unknown units, invalidity, transfer loss, healing, ambiguity, unsupported entities, and no drawing. A mixed-snapshot aggregate is constrained by the weakest representation relevant to that aggregate; exact bodies do not upgrade mesh-derived evidence. A user can accept a low-confidence input but must supply a reason; the warning and original confidence remain. Quote approval must show all unresolved blocking warnings.

The TASK-004 comparison contract implements this rule with `GeometryConfidenceLevel` and validated, unique `GeometryConfidenceReasonCode` values. Mesh evidence can reach only `Low`, and only when units are resolved, topology is manifold/watertight/consistently wound, and the versioned self-intersection check returns `not_detected`. Unresolved units, any of those topology defects, detected intersection, or indeterminate coplanar overlap produces `NeedsReview`; no numeric score is calculated. Detector version `partprobe-exact-mesh-intersection-spike-v1` uses bounded pairwise AABB preflight and exact floating-point triangle predicates, treats ordinary shared-edge topology as adjacency, and returns `indeterminate` for overlapping-bounds coplanar pairs rather than inventing a welding tolerance. Detector or confidence uncertainty cannot upgrade evidence: detected or indeterminate intersection withholds enclosed volume and centroid. Confidence policy `partprobe-mesh-confidence-policy-v1` and the parser version are retained separately. This is deterministic synthetic-fixture evidence, not the reviewed production vertex-welding, near-contact, or tolerance policy.

## Boundary with drawings and requirements

Geometry does not author material, tolerance, GD&T, finish, thread, inspection, certification, export-control, or customer requirements. `RequirementReviewLink` may associate a geometry reference with a human-entered drawing requirement but does not infer it. This prevents a visually plausible model from suppressing drawing-driven estimating work.
