# Geometry-kernel research

## Metadata

- **Status:** In Review
- **Last updated:** 2026-08-11
- **Related requirement IDs:** REQ-F-022, REQ-NF-011, REQ-NF-014, GEO-001–GEO-014, SEC-004
- **Related architecture decision IDs:** ADR-0002, ADR-0005
- **Open questions:** Does the chosen OCCT build meet target-platform packaging and performance goals? Is commercial native-format translation commercially justified?
- **Dependencies:** CAD fixture corpus, legal licensing review, worker-process spike, Rust FFI wrapper audit
- **Supersedes / superseded by:** None / none

## Recommendation

Use **OCCT behind a narrow, out-of-process geometry adapter** for the initial exact-B-rep path; pair it with Rust-owned mesh parsing/analysis and `wgpu` rendering. This maximizes practical STEP/IGES interoperability without putting C++ ABI, parser faults, or global kernel state inside the desktop process. It remains **In Review** until fixture, legal, and cross-platform packaging evidence is collected.

OCCT provides modular data exchange for STEP, IGES, STL and other formats, plus translation and validity checking. [OCCT overview](https://dev.opencascade.org/doc/overview/html/index.html) Its public license is LGPL 2.1 with an additional exception, not “license-free.” [OCCT licensing](https://dev.opencascade.org/resources/licensing)

## TASK-003 exact spike baseline

The reversible native spike pins official OCCT **8.0.0**, tag `V8_0_0`, commit `d3056ef80c9668f395da40f5fd7be186cae4501f`, released 2026-05-07. Version 8.0.0 is the current official release and requires C++17; the project will not substitute Homebrew's currently older 7.9.3 package because a platform-specific version would undermine replay and cross-platform comparison. [Official release](https://github.com/Open-Cascade-SAS/OCCT/releases/tag/V8_0_0) [Build requirements](https://dev.opencascade.org/doc/overview/html/build_upgrade__building_occt.html)

The reviewed source-build profile uses shared Release libraries, PCH off, TBB and FreeType off, all top-level modules off, and `BUILD_ADDITIONAL_TOOLKITS=TKDESTEP;TKShHealing;TKMesh`. CMake resolves the required internal toolkit graph, which includes CAF/XCAF and visualization-adjacent libraries despite disabling those top-level modules. `scripts/build_occt.py` enforces the exact clean source commit, records source tree `b3ffb8a91468845b63675057957209032b5806b1`, CMake/generator/compiler/options/jobs, and invokes content-addressed native verification. Fresh Apple-Silicon and Ubuntu-24.04-x86_64 constructions pass. The Ubuntu workflow also assembles/re-verifies the runtime, audits all OCCT dynamic links, and passes the configured desktop-host smoke under seccomp without retaining binaries. Windows construction, Linux GUI/package behavior, legal approval, and distribution packaging remain open.

The Rust bridge candidate is a project-owned C ABI shim compiled with `cc 1.4.0`, dynamically linked to OCCT, and enabled only by an explicit Cargo feature plus `PARTPROBE_OCCT_ROOT`. It catches C++ exceptions before they cross the ABI, returns stable content-free diagnostics, and exposes no OCCT type to Rust. This is engineering spike evidence only: OCCT is not shipped, enabled by default, or legally approved for product distribution.

| Candidate | Strengths | Material risks | Recommendation |
|---|---|---|---|
| OCCT via C ABI / worker | Mature B-rep, STEP AP203/214/242, IGES, tessellation, shape healing, physical-property operations; source available | C++/FFI memory and ABI boundary; LGPL notices/obligations; translation variability; native packaging | **Primary spike and planned initial kernel** |
| Siemens Parasolid / Communicator | Industrial component with solid, facet, lattice and model-interrogation offerings | Commercial contract, SDK/platform entitlement, confidential cost, nontrivial FFI and packaging | Evaluate only if native XT/format quality becomes paid requirement |
| Spatial ACIS + InterOp | Mature surface/solid modeling, validity/physical-property queries, commercial support | Commercial licensing/redistribution and integration cost; FFI and platform dependencies | Alternate commercial escalation, not initial dependency |
| Rust-native (Truck/Monstertruck, BrepRs) | Rust ownership, easier safety/reproducibility, useful for experiments | No demonstrated equivalence to OCCT’s industrial import/healing breadth; projects are evolving | Use for focused research/prototypes, not estimator authority initially |
| CGAL or custom mesh stack | Strong computational-geometry / mesh possibilities | License or integration burden, no direct STEP interchange solution, complexity | Mesh algorithms only when a measured need exists |

Siemens presents Parasolid as an SDK available in designer/editor/communicator forms, including solid and facet support. [Siemens SDK](https://www.siemens.com/en-us/products/plm-components/parasolid/3d-modeling-sdk/) Spatial describes ACIS as supporting surface/solid creation, model validity and physical-properties analysis. [Spatial ACIS](https://www.spatial.com/solutions/3d-modeling/3d-acis-modeler) These are vendor claims, not comparative benchmark evidence.

## Kernel contract and non-negotiable rules

- The Rust domain consumes a **neutral analysis artifact**, not OCCT topology objects: canonical millimeter values, exact/mesh provenance, measurements, warnings, stable geometric references, tessellation handles, and diagnostics.
- The worker receives immutable files by controlled path/descriptor and returns versioned structured data. It may not access network, user home directories, or the quote database.
- No importer, healer, or feature recognizer mutates the source. A healed derivative has its own hash, action list, kernel/importer configuration and explicit user-visible status.
- OCCT/FFI calls are confined to an audited adapter. Rust documents FFI as inherently unsafe because declarations may be wrong and can yield undefined behavior. [Rust `extern`](https://doc.rust-lang.org/std/keyword.extern.html)
- No geometry function makes a process-plan assertion. Exact geometry supports measurements; material, tolerances, threads, finish, datum intent, and process requirements remain separately reviewed.

## Required kernel spike scorecard

Run identical, legally redistributable STEP, IGES, STL and 3MF fixtures on Windows, macOS, and Linux. Score: successful parse/translation, body/shell counts, exact volume/area against reviewed reference, healing actions, tessellation quality, elapsed/peak memory, crash containment, determinism, package size, notices, and reproducible build method. A kernel may be selected only if it passes hard requirements: no desktop crash from malformed fixture, source hash preserved, unit provenance preserved, and reviewed measurements within documented fixture tolerance.

## Explicit limitations

An exact B-rep can still be unsuitable or incomplete. Healing may alter topology; imported color, names, assembly structure, and attributes are not manufacturing intent. A triangulated rendering is not calculation geometry. Mesh methods must never be reported as exact B-rep feature detection. Neither OCCT nor a commercial kernel establishes CAD native-format support without a tested translator.
