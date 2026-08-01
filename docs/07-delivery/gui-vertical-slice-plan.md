# Testable GUI Vertical Slice Plan

> **Status:** In Review
> **Last updated:** 2026-08-01
> **Related requirements:** REQ-F-002–REQ-F-010, REQ-F-032; REQ-NF-001, REQ-NF-004–REQ-NF-006, REQ-NF-010–REQ-NF-014; UX-001–UX-012; GEO-001–GEO-007; TEST-003, TEST-012, TEST-014, TEST-021, TEST-028
> **Related ADRs:** ADR-0001–ADR-0003, ADR-0005–ADR-0008
> **Open questions:** Whether the first internal build must include a 3D viewport; which independent STEP fixture is approved for the first end-to-end test
> **Dependencies:** TASK-003 native STEP evidence and TASK-005 UI-framework spike
> **Supersedes:** None

## Purpose and boundary

The nearest honest product demonstration is an **internal developer test slice**, not developer alpha, a supported importer, or the initial release. It proves that one local STEP file can move through the isolated worker, produce provisional geometry facts, accept explicit estimator inputs, and drive the existing deterministic cost and pricing rules through a deliberately styled desktop workflow.

The first slice may be macOS/Apple-Silicon-only and session-only because that is the only platform with current native OCCT evidence. It must visibly label analysis as provisional and state that unsaved work is lost on exit. It must use isolated fixtures or explicitly approved local test files, perform no external network access, supply no production rate defaults, and make missing or conflicting inputs blocking rather than treating them as zero.

This boundary does not complete TASK-003, TASK-005, TASK-006, M1, M2, or any production-format requirement.

## What is reusable now

- Typed units, fixed-precision money, deterministic calculation rules, rate selection, pricing/rounding, itemized traces, canonical snapshots, and synthetic golden tests.
- Kernel-neutral geometry contracts and a supervised worker with bounded input/output, verified bytes, cancellation, cleanup, partial OS resource containment, and provisional Apple Silicon OCCT STEP measurements.
- Application/security seams for authorization and audit, plus persistence-neutral derivative manifests.
- Documented desktop, model-review, design-system, accessibility, persistence, security, and release boundaries.

No desktop app, Tauri/Leptos frontend, model-viewer, tessellation contract, durable repository, complete estimate-workflow application service, report renderer, supported production importer, or packaged native worker exists yet. Default feature-off builds intentionally report `NATIVE_ADAPTER_UNAVAILABLE`.

## Current checkpoint status

GUI-1 is **In Review**. Checkpoint 20 adds fail-closed construction of the exact OCCT 8.0.0 source commit, a recorded CMake/compiler/source-tree manifest, content-addressed native verification, and `FIX-STEP-003`: a manually authored AP214 faceted rectangular prism that does not use OCCT as its generator. The adapter and supervised worker reproduce its analytic 12 × 8 × 5 mm dimensions, 392 mm² area, 480 mm³ volume, and `(6, 4, 2.5)` mm centroid. Formal geometry/security fixture review remains required before GUI-1 exits; this one manual fixture is not a production accuracy corpus.

## Five-checkpoint path to the first testable GUI

| Checkpoint | Deliverable | Exit evidence |
|---|---|---|
| GUI-1 — Native test seam | Repeatable local build/discovery of the pinned OCCT worker; at least one independently authored analytic STEP fixture; bounded provisional result DTO | Fixture hash/units/measurements reviewed; success, malformed input, cancellation, and unavailable-native paths pass locally |
| GUI-2 — Application use case | One session-scoped draft-estimate service composes authorized intake, worker supervision, unit/warning review, geometry facts, manual stock/material/setup/run/quantity inputs, rate resolution, and pricing | Headless integration tests prove blocked missing data, deterministic recalculation, traceability, and no UI-owned calculation |
| GUI-3 — Secure desktop shell | Minimal Tauri 2 + Leptos CSR composition root with native file selection, restrictive capabilities/CSP, no shell/network permission, typed commands/events, and design-system foundations | The UI-framework spike runs locally; host contract tests prove only the intended commands are exposed |
| GUI-4 — Analysis and estimate workspace | Import/progress/cancel/error states, unit confirmation, geometry facts/warnings, editable manual estimate inputs/rates, quantity and itemized cost/price trace | An estimator can choose the fixture, review evidence, supply every required input, and obtain a deterministic provisional estimate; unavailable/conflicting values remain visibly blocked |
| GUI-5 — End-to-end testability | Fixed-fixture desktop smoke test, keyboard-only completion, worker-failure recovery, local-data/ephemeral/provisional labels, and a reproducible developer runbook | One automated host/UI smoke path plus manual acceptance checklist pass without a hidden service or external data transfer |

GUI-1 implementation is now ready for review, leaving approximately four focused implementation checkpoints for the facts-and-estimate slice. The critical path moves to application-level model-to-estimate composition, then the secure desktop boundary—not visual form layout alone.

## Optional 3D review increment

The first testable GUI can honestly analyze a model and present numeric geometry evidence without rendering it. If a visible, selectable 3D model is required for the first test, add two or three checkpoints before GUI-5:

1. Define and produce a bounded, sanitized tessellation/display-scene artifact linked to the immutable analysis snapshot.
2. Implement the `model-viewer` boundary and a `wgpu` adapter with fit/orbit, units/axes, warning state, and a complete textual alternative.
3. Integrate viewport loading, failure recovery, accessibility, and GPU/platform smoke evidence.

The display mesh is visual evidence only. It never replaces exact analysis geometry or confirms a manufacturing feature.

## What remains after the developer slice

A cross-platform developer alpha still requires supported and reproducibly packaged STEP plus planned STL/3MF import, independent accuracy/tolerance evidence, remaining worker network/filesystem and resource containment, three-OS native builds, durable SQLite/blob repositories with migrations and backup/restore, guided shop-owned rate/policy setup and calibration, save/reopen/replay, internal/customer previews, accessibility/usability evidence, three-platform packaging, and release security/legal review.

The largest schedule risks are cross-platform native geometry packaging, safe containment, tessellation/viewer integration when included, and shop-owned rate/runtime calibration. The desktop form work is not the dominant uncertainty.

## Adoption rule

When implementation starts, record whether the target is the five-checkpoint facts-and-estimate slice or the viewport-inclusive increment. Do not widen a developer-only shortcut into a production-support claim. Every checkpoint must update `PROJECT_STATE.md`, the progress log, affected requirement evidence, and its focused acceptance tests.
