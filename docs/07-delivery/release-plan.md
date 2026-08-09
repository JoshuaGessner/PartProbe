# Initial Release Plan

> **Status:** Draft
> **Last updated:** 2026-08-09
> **Related requirements:** REQ-F-001–REQ-F-011; REQ-NF-001–REQ-NF-010
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** Supported OS versions, signing, quote format, accuracy thresholds
> **Dependencies:** M0.2 and M1
> **Supersedes:** None

The first releasable vertical slice proves: RFQ/customer/one part revision; STEP/STL/3MF intake; unit confirmation; model view/warnings/AABB/volume/area; material mass and rectangular/round stock; removed volume; broad milling/turning process; rough setup/runtime proposal; editable assumptions; an empty-on-install rate library with guided shop-owned setup; blocked missing/conflicting rates; selected-rate and rounding traces; added cost/risk categories; quantity breaks; transparent cost/pricing; overrides; save/reopen; source hash/analysis version; internal and customer previews; Windows/Linux/macOS.

## Current readiness

The repository is not yet at developer alpha. Calculation and user-owned rate mechanics exist as headless Rust foundations, and the isolated geometry-worker has exact asset delivery/revalidation, partial OS resource containment, a reproducible pinned Apple Silicon OCCT construction path, and provisional measurements for generated and manually authored analytic STEP fixtures. GUI-3 adds an unsigned, local Apple-Silicon developer shell and native STEP selection, but it is not connected to analysis or estimating. The release still lacks the complete desktop workflow, supported STEP/STL/3MF importers, representative reviewed accuracy validation, durable save/reopen repositories and migrations, complete containment/resource controls, real-shop policy calibration, preview/report generation, installers/signing, and end-to-end acceptance evidence.

The next release-critical sequence is TASK-003 worker/native completion, TASK-004 mesh import, TASK-005 desktop and guided rate setup, TASK-006 persistence/backup/replay, and TASK-007 shop calibration. Advanced routing, uncertainty, PMI, learning, CAM, capacity, and integration features remain outside the initial release gate.

For earlier engineering feedback, the [testable GUI vertical-slice plan](gui-vertical-slice-plan.md) defines an internal Apple-Silicon, STEP-only, session-only slice. GUI-1 implementation awaits fixture review; GUI-2's headless use case and GUI-3's restrictive shell are complete for their bounded scopes. Approximately two focused checkpoints remain for the analysis/estimate workspace and end-to-end smoke/accessibility evidence; a 3D viewport adds roughly two or three. The slice deliberately omits save/reopen, production format support, cross-platform packaging, reports, and calibration, so it is not a release stage or evidence that the repository has reached developer alpha.

## Staging

1. Internal developer alpha using isolated, visibly synthetic data that cannot seed production libraries.
2. Shop-design-partner alpha in an approved environment with governed fixtures and reviewed real rate categories, accounting treatment, pricing policy, and calibration.
3. Limited pilot after backup/restore, migration, security, usability, and calculation reviews.

No public/general release date is set. Exit follows `release-acceptance.md`; missing drawing interpretation, advanced feature recognition, team mode, and regulatory certification remain explicit limitations.
