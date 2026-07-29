# Nonfunctional Requirements

> **Status:** In Review
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-NF-001–REQ-NF-022; SEC-001–SEC-014; UX-001–UX-012, UX-021–UX-045
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** OQ-021–OQ-029
> **Dependencies:** Threat model, performance budgets, platform spikes
> **Supersedes:** None

| ID | Requirement | Acceptance evidence | Status |
|---|---|---|---|
| REQ-NF-001 | Core workflows shall run on supported Windows, Linux, and macOS targets without platform-specific domain behavior. | TEST-008 CI/package matrix | Draft |
| REQ-NF-002 | Standalone workflows shall operate without public network access; telemetry and external upload are off and absent by default. | TEST-011 network isolation | Draft |
| REQ-NF-003 | Calculation results shall be reproducible from stored inputs plus calculation, rate-card/entry/selector, pricing, and rounding versions; authoritative currency shall not use binary floating point. | TEST-001/002/014 deterministic cross-platform golden tests | Draft |
| REQ-NF-004 | Domain, geometry, runtime, and estimation engines shall be UI-independent and headlessly testable. | Architecture dependency tests | Draft |
| REQ-NF-005 | Malformed/untrusted imports shall fail recoverably without exposing sensitive geometry in logs. | TEST-003/011 malformed corpus | Draft |
| REQ-NF-006 | The UI shall remain usable with keyboard-only input, visible focus, non-color-only states, scalable text, and documented accessibility semantics. | TEST-012 accessibility suite | Draft |
| REQ-NF-007 | Every persisted schema change shall be migrated, backup/restore tested, and assessed for backward compatibility. | TEST-007 migration/restore suite | Draft |
| REQ-NF-008 | The product shall not claim production CAM, guaranteed feature recognition/setup, or regulatory compliance. | Release-content review | Draft |
| REQ-NF-009 | Approved revisions, rate/policy versions, and audit events shall be append-preserving; re-analysis or library updates shall not overwrite approved human decisions. | TEST-006/007/014 audit tests | Draft |
| REQ-NF-010 | Dependencies shall have documented purpose, maintenance condition, license, provenance, and security review. | Dependency gate | Draft |
| REQ-NF-011 | Geometry and model-rendering results shall be reproducible within documented numeric/tessellation tolerances and representation limits. | TEST-024–TEST-030 | Draft |
| REQ-NF-012 | Import/analysis shall enforce configurable file-size, entity/triangle, memory, CPU/wall-time, recursion, and output limits. | TEST-023, TEST-030, TEST-011 | Draft |
| REQ-NF-013 | The viewport shall meet measured interaction, memory, correctness, and HiDPI budgets on the supported GPU/backend matrix. | TEST-012, TEST-028 | Draft |
| REQ-NF-014 | Native dependencies and application packages shall build, install, launch, update/rollback, and uninstall under a documented support/signing policy on every supported OS. | TEST-008, TEST-029 | Draft |
| REQ-NF-015 | Advanced scoring, capacity, uncertainty, comparison, and learning outputs shall be replayable from immutable inputs, policies, versions, and seeds where applicable. | TEST-044, TEST-050, TEST-055, TEST-060, TEST-078 | Draft |
| REQ-NF-016 | Probabilistic outputs shall maintain valid range/percentile ordering, prohibit negative time/cost where physically impossible, and avoid probability language unsupported by the chosen method. | TEST-051–TEST-055 | Draft |
| REQ-NF-017 | Missing, stale, conflicting, unapproved, or low-quality rate, pricing-policy, capacity, availability, PMI, requirement, CAM, and learning evidence shall be explicit and shall not silently become a neutral/default value. | TEST-002/014, TEST-047, TEST-064, TEST-068, TEST-077, TEST-082, TEST-087 | Draft |
| REQ-NF-018 | Potentially expensive optimization, simulation, revision, and reconciliation jobs shall have deterministic limits, progress, cancellation, caching policy, and resource-isolation tests. | TEST-019, TEST-043, TEST-054, TEST-059, TEST-083 | Draft |
| REQ-NF-019 | Core domain contracts shall remain independent of individual CAM, ERP, QMS, scheduling, marketplace, or native-CAD vendors. | TEST-018, TEST-081, TEST-094 | Draft |
| REQ-NF-020 | Recalculation, rate/policy updates, comparison, learning, or algorithm upgrades shall never rewrite issued quotes, approved routings, prior model analyses, or historical actuals. | TEST-006/014, TEST-060, TEST-078, TEST-084 | Draft |
| REQ-NF-021 | Recommendations and scores shall expose reasons, uncertainty, missing evidence, policy version, and override path and shall not be represented as authoritative management or manufacturing decisions. | TEST-042, TEST-055, TEST-077, TEST-098 | Draft |
| REQ-NF-022 | Advanced UX shall use progressive disclosure and accessible tabular alternatives so the normal estimating path remains usable without enabling advanced analyses. | TEST-012, TEST-070–TEST-073 | Draft |

Performance targets remain provisional until representative-model benchmarks establish percentiles and hardware profiles.
