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

The repository is not yet at developer alpha. Calculation and user-owned rate mechanics exist as headless Rust foundations, and the isolated geometry-worker has exact asset delivery/revalidation, partial OS resource containment, a reproducible pinned Apple-Silicon OCCT construction path, provisional measurements for generated and manually authored analytic STEP fixtures, and a separately assembled/verified developer runtime containing its complete observed OCCT closure. GUI-2 through GUI-5 provide configured end-to-end Apple-Silicon developer evidence through selection, analysis, cancellation/recovery, explicit review/manual/session-rate/pricing inputs, and deterministic trace rendering. The release still lacks the broader durable desktop workflow, supported STEP/STL/3MF importers, representative reviewed accuracy validation, Windows/Linux native runtime evidence, save/reopen repositories and migrations, complete containment/resource controls, real-shop policy calibration, preview/report generation, signed installers, and release acceptance evidence.

The next release-critical sequence is TASK-003 worker/native completion, TASK-004 mesh import, TASK-005 desktop and guided rate setup, TASK-006 persistence/backup/replay, and TASK-007 shop calibration. Advanced routing, uncertainty, PMI, learning, CAM, capacity, and integration features remain outside the initial release gate.

For earlier engineering feedback, the [testable GUI vertical-slice plan](gui-vertical-slice-plan.md) defines an internal Apple-Silicon, STEP-only, session-only slice. GUI-2 through GUI-5 are complete for their bounded checkpoints: the configured actual app now has fixed-fixture native analysis, keyboard completion, semantic inspection, cancellation/failure recovery, and deterministic estimate evidence. GUI-1 formal fixture review remains open, and a 3D viewport adds roughly two or three separate checkpoints. The slice deliberately omits save/reopen, production format support, cross-platform native packaging, full accessibility/usability, reports, and calibration, so it is not a release stage or evidence that the repository has reached developer alpha.

## Staging

1. Internal developer alpha using isolated, visibly synthetic data that cannot seed production libraries.
2. Shop-design-partner alpha in an approved environment with governed fixtures and reviewed real rate categories, accounting treatment, pricing policy, and calibration.
3. Limited pilot after backup/restore, migration, security, usability, and calculation reviews.

No public/general release date is set. Exit follows `release-acceptance.md`; missing drawing interpretation, advanced feature recognition, team mode, and regulatory certification remain explicit limitations.
