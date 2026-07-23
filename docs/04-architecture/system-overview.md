# System Architecture Overview

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-001–REQ-F-065; REQ-NF-001–REQ-NF-022  
> **Related ADRs:** ADR-0001–ADR-0014  
> **Open questions:** OQ-021, OQ-022, OQ-031–OQ-050  
> **Dependencies:** Architecture spikes  
> **Supersedes:** None

## Recommended shape

PartProbe is a Cargo workspace with a Tauri 2 desktop shell and Leptos CSR UI, Rust application/domain services, a deterministic estimation graph, SQLite standalone persistence, content-addressed local document storage, and an isolated native geometry worker using OCCT-based STEP B-rep translation and analysis for successfully translated, validated solids. Invalid, partial, or healed results remain explicitly limited. A `wgpu` renderer consumes a sanitized render scene rather than kernel objects. These choices remain **In Review** until their ADR spike gates pass.

```text
Leptos UI + wgpu viewport
          │ typed commands/events
Tauri desktop/application boundary
          ├── domain + estimation/runtime libraries
          ├── advanced analysis (routes, uncertainty, capacity, revision, coverage)
          ├── governed evidence (corrections, CAM, availability, sourcing, priority)
          ├── repositories ── SQLite + document store
          └── geometry client ── bounded IPC ── geometry worker (OCCT/mesh import)
```

## Architectural invariants

- Domain and calculation crates know nothing about UI, SQL, or native geometry types.
- Original files and published analysis/estimate snapshots are immutable.
- Every boundary uses explicit versioned DTOs, units, provenance, and recoverable errors.
- Geometry worker privileges, resource budgets, and accessible paths are narrower than the desktop process.
- Standalone mode has no required server or public network.
- Team mode reuses application/repository contracts; it is not SQLite on a network share.
- Advanced engines consume immutable baseline snapshots and produce versioned proposals or views; they do not mutate approved estimates.
- Capacity snapshots, availability observations, CAM imports, PMI evidence, and correction events cross bounded adapter contracts and retain source, freshness, classification, and version.
- Rules, suggestions, routes, coverage exceptions, and bid outcomes require explicit human authority at the application boundary.

See the ADRs for rationale and evidence gates.
