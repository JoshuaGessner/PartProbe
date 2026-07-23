# Release Acceptance

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirements:** All  
> **Related ADRs:** ADR-0001–ADR-0014  
> **Open questions:** Release authority and support policy  
> **Dependencies:** Release plan  
> **Supersedes:** None

A release candidate requires:

- Acceptance criteria and linked tests pass for claimed scope.
- No unresolved critical security, data-loss, calculation-integrity, or source-integrity defect.
- Three-platform core/build/package evidence for supported targets.
- Migration plus backup/restore from every supported prior release.
- Dependency license/security review and reproducible provenance manifest.
- Geometry/calculation golden changes reviewed by qualified owners.
- Accessibility/key workflows reviewed; customer report leakage check passes.
- Offline/no-telemetry test passes.
- Known limitations, upgrade/rollback, data-retention, and support notes are published.
- `PROJECT_STATE`, requirements matrix, changelog, risk register, and release notes agree.

Release acceptance never implies regulatory compliance or production-CAM fitness.
