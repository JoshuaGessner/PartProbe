# Module Boundaries

> **Status:** In Review  
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-NF-001, REQ-NF-004, REQ-NF-010, REQ-NF-015–REQ-NF-021  
> **Related ADRs:** ADR-0001–ADR-0014  
> **Open questions:** Exact crate split after dependency-cycle spike  
> **Dependencies:** Cargo bootstrap  
> **Supersedes:** None

## Proposed workspace

| Path | Owns | May depend on |
|---|---|---|
| `crates/domain` | IDs, units, money, entities, provenance, lifecycle | minimal vetted primitives |
| `crates/geometry-core` | kernel-neutral geometry DTOs/stages | domain units/IDs |
| `crates/geometry-import` | worker client/protocol/import orchestration | geometry-core, platform |
| `crates/feature-recognition` | detector traits/results | geometry-core, domain |
| `crates/setup-planner` | proposal/alternative logic | domain, geometry-core, feature results |
| `crates/tooling` | tool candidates/compatibility | domain |
| `crates/feeds-speeds` | versioned library evaluation | domain, tooling |
| `crates/runtime-estimation` | TIME methods/results | domain, feeds-speeds |
| `crates/estimation-engine` | acyclic calculation graph | domain + pure engines |
| `crates/advanced-analysis` | route comparison, uncertainty, sensitivity, revision/cost-delta orchestration | domain + pure engines; no repositories/UI |
| `crates/capacity-analysis` | immutable capacity snapshots, occupancy and opportunity views | domain + estimation results |
| `crates/requirement-coverage` | requirement evidence, applicability, coverage/readiness rules | domain |
| `crates/learning-governance` | correction evidence, cohorts, suggestions, activation records | domain + application policy ports |
| `crates/cam-reconciliation` | vendor-neutral CAM normalization and typed variance comparison | domain + import-export ports |
| `crates/sourcing-analysis` | availability/readiness, make-buy, bid and priority proposals | domain + estimation results |
| `crates/application` | use cases, authz hooks, transactions | domain + repository/engine traits |
| `crates/persistence` | SQLite repositories/migrations | application ports, domain |
| `crates/document-storage` | hashes, immutable blobs, manifests | domain, platform |
| `crates/security` | policy, classification, audit interfaces | domain |
| `crates/import-export` | versioned external DTOs | application, domain |
| `crates/reporting` | internal/customer render models | application, domain |
| `crates/platform` | filesystem/process/dialog abstractions | OS APIs only |
| `crates/model-viewer` | render scene/picking/overlays | geometry-core, wgpu adapter |
| `crates/ui-components` | design-system components | UI framework, application DTOs |
| `crates/test-support` | builders/golden harnesses | public test APIs |
| `crates/sample-data` | synthetic non-sensitive examples | domain DTOs |
| `apps/estimator-desktop` | composition root/Tauri shell | adapters and UI |
| `apps/geometry-worker` | native parser/kernel host | geometry-core, native adapters |

No UI crate may execute SQL or receive raw kernel pointers. Advanced-analysis crates do not own approval, scheduling, purchasing, supplier transmission, or rule activation. Integration adapters depend inward on versioned ports and may not leak vendor DTOs into the domain. Dependency direction is enforced with workspace policy tests and review.

TASK-003 instantiates narrow `security`, `application`, and `document-storage` slices for local geometry-asset reads and governed derivative handoff. `security` owns policy/audit contracts and depends only on domain identities. `document-storage` owns immutable blob, manifest, policy-reference, integrity, and controlled-store port types without selecting a filesystem or database. `application` coordinates policy, audit, `geometry-import`, and the storage port; `geometry-import` remains unaware of actors, roles, classification rules, audit persistence, or durable storage. This preserves inward dependency direction while deployment-specific adapters remain unimplemented.
