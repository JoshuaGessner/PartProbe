# Changelog

> **Status:** In Review
> **Last updated:** 2026-08-09
> **Related requirements:** All
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** None
> **Dependencies:** Release governance
> **Supersedes:** None

All notable project changes are recorded here. This project follows Keep a Changelog structure once releases begin.

## [Unreleased]

### Added

- Phase 0 repository governance and planning documentation baseline.
- Research, domain, requirements, architecture, UX, quality, delivery, and decision records.
- Initial fixture manifest and validation expectations.
- Planning metadata/link validator and fixture hash utility.
- Advanced planning for routing, capacity/economics, uncertainty, revision/PMI/coverage, feature explanation, learning, CAM, availability, sourcing, and bid priority.
- ADR-0009–ADR-0014, expanded traceability through REQ-F-065/DATA-042/TEST-099, staged roadmap, and governed human-adoption/versioning boundaries.
- Rust 1.94.1 workspace with domain, estimation-engine, and test-support crates.
- Typed units, fixed-precision money/currency, provenance/version wrappers, explicit value states, typed DAG validation, and canonical calculation snapshots.
- Executable TASK-001 tests, passing three-OS CI evidence, dependency record, and validation evidence.
- User-owned effective-dated rate cards, explicit approval/source/effective-period contracts, deterministic missing/ambiguity behavior, and no numeric production defaults.
- Versioned rounding and pricing policies, CALC-007–CALC-018 deterministic foundations, and isolated synthetic EX-01/03/12 golden/replay tests.
- Kernel-neutral geometry contracts, bounded schema-v2 worker control and transport, verified-copy plus exact Unix descriptor/Windows HANDLE direct delivery, worker-side source verification, cancellation/forced termination, private staging, controlled output claiming, and governed derivative handoff.
- Optional OCCT 8.0.0 ABI-v3 Apple Silicon byte-stream parsing with provisional synthetic STEP measurements and cancellation polling; default builds remain native-feature-off.
- Partial worker resource containment: Unix CPU/file/core/process-group controls, Linux memory limits, and suspended Windows Job CPU/memory/one-process/tree-kill controls.
- A canonical testable-GUI plan separating the five-checkpoint internal STEP/session-only slice, optional viewport increment, cross-platform alpha, and release acceptance.
- A validated schema for the existing provisional native geometry evidence, a decoder binding claimed output to its reference and authorized source, and a native-root fingerprint/strict-check/worker-build entry point for GUI-1.
- Fail-closed OCCT 8.0.0 source construction with exact commit/tag/clean-tree checks, fixed minimal CMake options, a compiler/generator/source-tree manifest, native artifact fingerprints, and focused Python tooling tests.
- `FIX-STEP-003`, a manually authored AP214 faceted 12 × 8 × 5 mm rectangular prism with governed hash and analytic area/volume/centroid evidence through both the native adapter and supervised worker.
- GUI-2 headless draft-estimate orchestration: governed source authorization, a concrete supervised-geometry adapter, explicit provisional-evidence review and manual inputs, pinned rate/policy resolution, deterministic itemized results, replay trace, and focused missing/conflict/recalculation tests.
- GUI-3 restrictive Tauri 2/Leptos developer shell with a versioned framework-neutral desktop contract, asynchronous host-owned STEP picker, native-only raw path retention, leaf-name-only frontend summary, exact command/capability/CSP regression tests, deliberate design-system foundations, unsigned Apple-Silicon bundle evidence, and a developer runbook.
- GUI-4 intake foundation: an application-owned request template now authorizes and audits the selected source before deriving a bounded SHA-256 fingerprint from the same already-open grant subsequently consumed and independently verified by the geometry worker.
- GUI-4 native analysis workspace: desktop contract v2 adds a token-only asynchronous analysis command, explicit fail-closed worker/workspace/OCCT configuration, developer-session authorization and in-memory audit, native `DraftEstimateSession` retention, path-free provisional exact-B-rep evidence, and deliberately unavailable estimate states in the Leptos workspace.
- GUI-4 estimate workspace: desktop contract v3 adds token-bound cooperative cancellation and native draft-estimate evaluation; the deliberate Leptos workflow now requires explicit unit/warning review, complete manual quantities/times/costs, confirmed session-only rates, and a confirmed versioned pricing policy before showing the deterministic itemized result and trace. The host delegates all authoritative evaluation to `DraftEstimateSession` and supplies no production defaults or durable approval.

### Security

- Established local-first, no-external-upload-by-default policy for technical and quote data.
- Added deny-by-default authorization/audit seams, capability-root containment, exact child-resource allowlists, and explicit documentation that partial worker controls are not an OS sandbox.
- Added exact desktop application-command and main-window capability allowlists, packaged-local-content CSPs, prototype freezing, pathless bridge DTOs, and no frontend shell/network/filesystem/opener/updater/upload/dialog permission.
