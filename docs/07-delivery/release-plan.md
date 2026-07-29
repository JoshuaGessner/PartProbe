# Initial Release Plan

> **Status:** Draft
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-F-001–REQ-F-011; REQ-NF-001–REQ-NF-010
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** Supported OS versions, signing, quote format, accuracy thresholds
> **Dependencies:** M0.2 and M1
> **Supersedes:** None

The first releasable vertical slice proves: RFQ/customer/one part revision; STEP/STL/3MF intake; unit confirmation; model view/warnings/AABB/volume/area; material mass and rectangular/round stock; removed volume; broad milling/turning process; rough setup/runtime proposal; editable assumptions; an empty-on-install rate library with guided shop-owned setup; blocked missing/conflicting rates; selected-rate and rounding traces; added cost/risk categories; quantity breaks; transparent cost/pricing; overrides; save/reopen; source hash/analysis version; internal and customer previews; Windows/Linux/macOS.

## Staging

1. Internal developer alpha using isolated, visibly synthetic data that cannot seed production libraries.
2. Shop-design-partner alpha in an approved environment with governed fixtures and reviewed real rate categories, accounting treatment, pricing policy, and calibration.
3. Limited pilot after backup/restore, migration, security, usability, and calculation reviews.

No public/general release date is set. Exit follows `release-acceptance.md`; missing drawing interpretation, advanced feature recognition, team mode, and regulatory certification remain explicit limitations.
