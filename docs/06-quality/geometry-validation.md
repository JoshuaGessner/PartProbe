# Geometry-validation strategy

## Metadata

- **Status:** In Review
- **Last updated:** 2026-09-01
- **Related requirement IDs:** GEO-001–GEO-014, REQ-NF-011–REQ-NF-014, TEST-021–TEST-030, SEC-004
- **Related architecture decision IDs:** ADR-0002, ADR-0004, ADR-0005
- **Open questions:** Reviewed tolerance table by format/property? Fixture-owner assignments? Release resource budgets?
- **Dependencies:** Fixture strategy, geometry worker, golden-result schema, CI runners for all target OSes
- **Supersedes / superseded by:** None / none

## Objective

Prove that geometry claims are reproducible, correctly labelled by representation, and fail safely. Tests validate the analysis contract—not that any imported model is manufacturing-complete or that a kernel’s result is absolute truth.

Current TASK-003 evidence includes analytic STEP measurements on Apple Silicon and Ubuntu 24.04 x86_64. The Ubuntu checkpoint constructs the exact pinned kernel, verifies the runtime/link closure, and passes the governed prism through the configured desktop host under Linux parser containment. That is one cross-platform comparison point, not the three-OS corpus, tolerance scorecard, packaging result, or production importer acceptance required below.

Current TASK-004 evidence adds path-free, quota-bounded ASCII/binary STL and ZIP/OPC-bounded 3MF comparison analyzers over twenty-two successful public synthetic mesh fixtures plus thirty-seven persisted rejected mesh inputs. Successful governed STL/3MF inputs now also cross verified-copy/direct worker transport, independent asset verification, parser containment, content-derived dispatch, controlled-output claiming, source/reference-bound decoding, and cleanup under additive `geometry-mesh-snapshot-v1`. All accepted formats share deterministic triangle bounds/area/manifold/closure/winding, versioned exact-predicate self-intersection state, categorical reason-coded confidence, and conditional-volume measurement while topology identity remains format-aware and every successful result names policy v1 plus not-applied welding status. STL units remain explicitly unresolved and therefore `NeedsReview`; validated unit-resolved 3MF mesh evidence reaches only `Low`. `FIX-MESH-039`, `047`–`049`, and `059` separately prove detected intersection, inconsistent winding, non-manifold/open topology, coplanar `indeterminate` state, and a 3MF index-defined open seam with volume/centroid withheld. Parser v8 retains 3MF vertex indices for edge/winding/adjacency decisions rather than welding equal coordinates; STL retains its explicit exact-source-coordinate/no-tolerance spike rule. Schema v4 pins the detector's exact three-state result instead of collapsing uncertainty into a boolean. Fifteen successful 3MF fixtures persist all six Core declarations/default, direct through two-link linear component-chain provenance, ordered transforms, bounded model metadata, four isolated legal OPC variations including a standard internal package thumbnail, index-defined topology, and canonical measurements. Twenty-nine adversarial 3MF packages pin exact no-evidence diagnostics for selected graph/reference/union/metadata/material/extension/archive/XML/resource threats, the unsupported singular/reflected-transform boundary, incomplete/non-finite vertex data, repeated/out-of-range triangle indices, an empty triangle structure, and the vertex-count ceiling; the reflected case is a deliberately narrower PartProbe capability limit, not Core-invalid input. The empty triangle structure pins `THREE_MF_UNSUPPORTED_MODEL_STRUCTURE`, not the analyzer's later `THREE_MF_EMPTY_MESH` branch. Four binary STL inputs pin truncated framing, unsupported facet attributes, non-finite numbers, and triangle-quota rejection; and four ASCII STL inputs pin invalid-text, malformed-grammar, empty-mesh, and degenerate-geometry rejection. The main format corpus still derives from one analytic cube; the tetrahedral and coplanar cases are additional project-authored analytic shapes, not independent CAD-tool or representative machining evidence. Correct negative-determinant orientation/volume handling, broader representative malformed entity/geometry cases, supported general DAG/material semantics, broader alternate legal package layouts, production welding/tolerance policy, application/desktop integration, representative geometry, and supported-importer acceptance remain open; see `task-004-validation.md`.

## Validation layers

| Layer | What it proves | Required evidence |
|---|---|---|
| Unit / property | pure conversions, envelope/volume/mass derivations, no negative removed volume | deterministic tests and property tests |
| Format integration | parser/translator reports, units, bodies/shells, validation flags, source preservation | representative golden fixtures |
| Cross-platform | same input/profile produces accepted comparable snapshot on Windows, Linux, macOS | artifact diff with OS/library fingerprints |
| Robustness | malformed/untrusted input cannot crash desktop, escape filesystem, substitute the authorized source, inherit an unrelated resource, or exceed policy | transport identity/hash/length/quota/fallback tests plus worker termination tests |
| Visual | tessellation maps feature/body picks to snapshot references | screenshot/interaction tests, not measurement oracle |
| Regression | known issue stays diagnosed or fixed | minimized fixture, expected outcome, issue link |

## Golden-contract requirements

Each fixture expectation pins: source SHA-256, format/version, declared and confirmed units, analysis profile/version, representation, body/shell count, validity state, AABB/OBB algorithm and results, volume/area/centroid status, mesh manifold/watertight/self-intersection state where applicable, categorical confidence/reasons, warnings, healing actions, accepted tolerance, and expected stage state. Exact B-rep reference values require a reviewed independent source or analytic construction; mesh values identify approximation and mesh-resolution tolerance. Store decimal units explicitly—never compare display-rounded strings.

## Required test cases

- Analytic cuboid, cylinder, hollow tube, cone and multi-body models for exact measurements and unit transformations.
- STEP AP203/AP214/AP242 samples with names/colors/assembly and intentionally partial/invalid transfers.
- IGES surface/shell and solid samples (secondary format) to ensure warnings distinguish translation loss from validity.
- STL ASCII/binary with millimeter/inch ambiguity, open boundary, reversed normals, degenerate triangle, non-manifold edge, self-intersection and oversized triangle count; synthetic persisted cases now cover each condition, while representative exporter-derived inputs remain pending.
- 3MF with union/general-DAG cases, item/vendor metadata and material layouts, unsafe archive variants, malformed XML/entity forms, resource limits, high compression ratio, unsupported required extensions, and index-defined topology; all Core units/default, one two-link linear chain, one well-known model-metadata layout, four isolated legal OPC variations including one standard internal package thumbnail, one split-index seam, and twenty-nine selected adversarial rejections now have persisted fixtures.
- OBJ with missing/malicious MTL/texture path; the test asserts no external path traversal.
- Unitless and conflicting-scale inputs: analysis blocks approval and preserves user resolution reason.
- Healing candidate: source/derivative hashes and altered topology warning must be present.
- Worker asset transport: capability/job/correlation mismatch, deleted source, same-length hash tamper, length drift, quota breach, explicit verified-copy selection, unavailable direct mode, and unrelated descriptor/HANDLE inheritance.
- Worker crash, timeout, memory/CPU budget, malformed IPC response and cancellation: desktop remains usable and quote state unchanged.

## Acceptance rules

1. Exact values pass only when the fixture validation state supports the operation. A “computed” field never appears for invalid/unknown input.
2. Mesh volume/mass is absent unless closure, orientation and self-intersection policy passes; when present it is explicitly approximate.
3. Repeated analysis with identical bytes/profile/dependencies is deterministic within documented numeric tolerance and has the same warnings/results ordering.
4. Any kernel/importer/profile change reruns the corpus; a changed golden result requires reviewer approval, explanation, version decision and changelog entry.
5. No input can cause UI process termination, network access, external-reference following, unbounded resource use or sensitive data in diagnostic output.

## Release gate and metrics

CI reports fixture pass rate by format/OS, measurement deltas, warning-state deltas, worker crash/timeout count, p50/p95 elapsed time, peak memory, package/SBOM result, and unreviewed baseline changes. A release blocks on any source-hash mismatch, incorrect unit/representation label, unexpected high-confidence result, desktop crash, security-control failure, or unexplained exact measurement delta. See `model-fixture-strategy.md` for corpus ownership.
