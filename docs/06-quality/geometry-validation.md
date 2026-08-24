# Geometry-validation strategy

## Metadata

- **Status:** In Review
- **Last updated:** 2026-08-24
- **Related requirement IDs:** GEO-001–GEO-014, REQ-NF-011–REQ-NF-014, TEST-021–TEST-030, SEC-004
- **Related architecture decision IDs:** ADR-0002, ADR-0004, ADR-0005
- **Open questions:** Reviewed tolerance table by format/property? Fixture-owner assignments? Release resource budgets?
- **Dependencies:** Fixture strategy, geometry worker, golden-result schema, CI runners for all target OSes
- **Supersedes / superseded by:** None / none

## Objective

Prove that geometry claims are reproducible, correctly labelled by representation, and fail safely. Tests validate the analysis contract—not that any imported model is manufacturing-complete or that a kernel’s result is absolute truth.

Current TASK-003 evidence includes analytic STEP measurements on Apple Silicon and Ubuntu 24.04 x86_64. The Ubuntu checkpoint constructs the exact pinned kernel, verifies the runtime/link closure, and passes the governed prism through the configured desktop host under Linux parser containment. That is one cross-platform comparison point, not the three-OS corpus, tolerance scorecard, packaging result, or production importer acceptance required below.

Current TASK-004 evidence adds path-free, quota-bounded ASCII/binary STL and ZIP/OPC-bounded 3MF comparison analyzers over thirteen successful public synthetic mesh fixtures plus fifteen persisted rejected 3MF packages. All accepted formats share deterministic triangle bounds/area/manifold/closure/winding/conditional-volume measurement. STL units remain explicitly unresolved. Ten successful 3MF fixtures persist all six Core declarations/default, direct through two-link linear component-chain provenance, ordered transforms, bounded model metadata, and canonical measurements. Fifteen adversarial packages pin exact no-evidence diagnostics for branching/non-immediate/forward references, an unused component, a build union, object/item/vendor metadata, a material attribute, a required extension, relationship traversal, case ambiguity, excessive/unsupported compression, and encryption. The corpus derives from one analytic cube and is not independent representative geometry. Broader binary/malformed cases, supported general DAG/material semantics, alternate package layouts, self-intersection policy, approximate-confidence scoring, worker/desktop integration, representative geometry, and supported-importer acceptance remain open; see `task-004-validation.md`.

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

Each fixture expectation pins: source SHA-256, format/version, declared and confirmed units, analysis profile/version, representation, body/shell count, validity state, AABB/OBB algorithm and results, volume/area/centroid status, mesh manifold/watertight state where applicable, warnings, healing actions, accepted tolerance, and expected stage state. Exact B-rep reference values require a reviewed independent source or analytic construction; mesh values identify approximation and mesh-resolution tolerance. Store decimal units explicitly—never compare display-rounded strings.

## Required test cases

- Analytic cuboid, cylinder, hollow tube, cone and multi-body models for exact measurements and unit transformations.
- STEP AP203/AP214/AP242 samples with names/colors/assembly and intentionally partial/invalid transfers.
- IGES surface/shell and solid samples (secondary format) to ensure warnings distinguish translation loss from validity.
- STL ASCII/binary with millimeter/inch ambiguity, open boundary, reversed normals, degenerate triangle, non-manifold edge, self-intersection and oversized triangle count.
- 3MF with union/general-DAG cases, item/vendor metadata and material layouts, additional unsafe archive variants, high compression ratio, and unsupported required extensions; all Core units/default, one two-link linear chain, one well-known model-metadata layout, and five selected adversarial rejections now have persisted fixtures.
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
