# ADR-0002: Geometry kernel

## Metadata

- **Status:** In Review
- **Last updated:** 2026-08-11
- **Related requirement IDs:** REQ-F-021–REQ-F-024, REQ-NF-011–REQ-NF-014, GEO-001–GEO-014, SEC-004
- **Related architecture decision IDs:** ADR-0004, ADR-0005
- **Open questions:** Windows native construction and three-OS fixture/package results, LGPL compliance decision, packaging/signing, and broader reviewed corpus
- **Dependencies:** OCCT packaging/legal review; format and malformed-input spike
- **Supersedes / superseded by:** None / none

## Context

The estimator needs validated measurements and topology from interchange CAD while remaining Rust-first, cross-platform and local-first. Rust-native options are promising but do not yet establish the required breadth of STEP/IGES translation and healing. Commercial kernels offer support but introduce contractual, cost, redistribution, and FFI constraints.

## Proposed decision

Adopt **OCCT as the initial exact B-rep kernel, accessed only by a separate partially OS-contained native geometry worker and neutral Rust contract**. Use Rust-native mesh processing for STL/3MF/OBJ measurement/validation. Keep a provider interface so a commercial kernel/translator can later be added without changing quote-domain types.

OCCT documents STEP AP203/AP214/AP242 and IGES interfaces, with validity checking and data exchange. [OCCT overview](https://dev.opencascade.org/doc/overview/html/index.html) Its license is LGPL 2.1 with an additional exception and requires visible notices/consideration of LGPL obligations in proprietary applications. [OCCT licensing](https://dev.opencascade.org/resources/licensing)

The TASK-003 spike baseline is official OCCT 8.0.0 tag `V8_0_0`, commit `d3056ef80c9668f395da40f5fd7be186cae4501f`, built as shared C++17 libraries from source. A project-owned C ABI shim is optional, dynamically linked, and confined to `geometry-occt-adapter`; default builds contain no native adapter. This exact spike selection does not accept this ADR or approve redistribution.

Checkpoint 20 automates that exact profile on Apple Silicon with clean-source verification, a source-tree/compiler/CMake manifest, and content-addressed install verification. A manually authored AP214 faceted prism also reproduces analytic area, volume, and centroid through the adapter and supervised worker, removing the OCCT-as-generator dependency for one simple fixture. Run 31555774851 repeats exact construction, runtime assembly/verification, internal dynamic-link closure, and the configured desktop-host smoke on Ubuntu 24.04 x86_64 under the Linux parser filter. Formal fixture review, broader translator coverage, Windows construction, Linux GUI/package evidence, signing, legal review, and ADR acceptance remain open.

## Consequences

- Positive: credible open initial STEP path, no vendor lock-in in the domain model, FFI/crash isolation, reproducible source/derivative provenance.
- Negative: native worker packaging on three OSes, potential translation differences, legal notice/compliance work, explicit wrapper maintenance and IPC overhead.
- Non-consequence: this does not add native proprietary CAD support or certify model correctness.

## Approval evidence required

1. Legal review of OCCT release license/notices and shipping method; SBOM and third-party notice plan.
2. Windows/macOS/Linux worker build, signing/notarization feasibility, and isolated malformed-input behavior.
3. Reviewed format fixture scorecard with exact properties, healing reports, resource limits and deterministic reruns.
4. Rust FFI wrapper audit and worker boundary test proving worker failure cannot crash the desktop or corrupt quote data.

## Alternatives considered

- **Parasolid/Communicator:** capable commercial option, deferred until native/interoperability value justifies negotiated terms. [Siemens SDK](https://www.siemens.com/en-us/products/plm-components/parasolid/3d-modeling-sdk/)
- **ACIS/InterOp:** capable commercial option, same contract and integration gate. [Spatial ACIS](https://www.spatial.com/solutions/3d-modeling/3d-acis-modeler)
- **Rust-only kernel:** preferred long-term safety posture if coverage is demonstrated; not sufficient evidence for initial authority.
- **In-process OCCT:** rejected for initial design because untrusted parsing and FFI failure containment matter more than eliminating IPC.
