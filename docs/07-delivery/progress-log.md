# Progress Log

> **Status:** In Review
> **Last updated:** 2026-07-29
> **Related requirements:** All
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** OQ-001–OQ-050
> **Dependencies:** None
> **Supersedes:** None

## 2026-07-29 — TASK-003 geometry-worker contract started

- Added dependency-free `geometry-core` and `geometry-import` crates before selecting a native kernel build.
- Added validated lowercase SHA-256 evidence, source byte/format descriptors, model-unit and representation labels, versioned analysis profiles, ordered pipeline stages, explicit stage states, and sanitized warning contracts.
- Added a schema-versioned, path-free worker request using opaque asset capabilities, expected source hashes, canonical stage ordering, and explicit input/output/entity/wall-time quotas.
- Added recoverable mappings for worker exit, timeout, quota breach, malformed response, and cancellation; no raw CAD contents, unrestricted paths, geometry coordinates, or model names enter the protocol.
- Ten new tests pass with the full local workspace gate, for 51 runtime tests total plus one compile-fail doctest; GitHub Actions run 30463022276 also passes Windows, Linux, and macOS.
- OCCT is not installed or added. Native adapter, IPC transport, process sandbox, analytic STEP fixtures, dependency/license review, packaging, and Windows/Linux/macOS benchmarks remain TASK-003 work.

## 2026-07-29 — TASK-003 bounded worker and fixture contract

- Added a real local subprocess supervisor with a controlled working directory, cleared inherited environment, bounded request/response JSON, cancellation and timeout polling, hard-kill/reap behavior, response identity verification, and sanitized launch/I/O/exit/protocol failures.
- Added a minimal `geometry-worker` executable and process-boundary integration test. The worker explicitly returns `NATIVE_ADAPTER_UNAVAILABLE`; it does not parse CAD or present the process boundary as native-kernel evidence.
- Upgraded the two bootstrap fixture expectations to schema version 2 with exact decimal strings and explicit available/unavailable/not-applicable states. The open cube cannot expose enclosed volume as authoritative.
- Recorded the fixture-only migration policy: version 1 is rejected and requires reviewed explicit conversion; missing evidence is never guessed as zero.
- Four additional tests bring the workspace to 55 runtime tests plus one compile-fail doctest; formatting and strict Clippy pass locally and in GitHub Actions run 30465722294 on Windows, Linux, and macOS.
- No new third-party package was added. Serde JSON's existing reviewed role now includes the bounded internal worker protocol.
- Remaining containment work is explicit: capability-to-handle resolution, cooperative cancellation grace, OS sandbox/network denial, CPU/memory/descendant limits, cleanup evidence, OCCT selection, analytic STEP fixtures, and three-OS native packaging/benchmarks.

## 2026-07-29 — TASK-003 exact OCCT spike baseline

- Selected official OCCT 8.0.0 tag `V8_0_0`, commit `d3056ef80c9668f395da40f5fd7be186cae4501f`, instead of Homebrew's older platform-specific package.
- Configured and built shared C++17 libraries from source on Apple Silicon with PCH, TBB, FreeType, all top-level modules, and external CMake dependencies disabled; requested only `TKDESTEP`, `TKShHealing`, and `TKMesh`.
- Recorded that OCCT resolves a broader internal CAF/XCAF and visualization-adjacent toolkit graph even under the minimal request; local installed shared libraries are approximately 46 MiB.
- Added optional `geometry-occt-adapter` with a project-owned C ABI, dynamic linking, stable ABI version, bounded result layout, finite/nonnegative result checks, caught C++ exceptions, and sanitized diagnostics.
- Added exact build dependency `cc 1.4.0` with `find-msvc-tools 0.1.9` and `shlex 2.0.1`; all are build-only and MIT OR Apache-2.0.
- Default builds do not link or fetch OCCT. The explicit local native feature passes link/ABI and missing-file sanitization tests; legal approval, capability resolution, analytic STEP evidence, and three-OS build/package fingerprints remain mandatory.
- GitHub Actions run 30467602461 passes all 56 default runtime tests, strict Clippy, formatting, and documentation tests on Windows, Linux, and macOS; this validates feature-off portability, not native OCCT artifacts.

## 2026-07-29 — TASK-003 controlled STEP measurement path

- Added fixed-name source staging with regular-file/symlink checks, create-new semantics, streaming byte quota, SHA-256 verification, read-only staged permissions, sanitized failures, and cleanup after worker completion.
- Added regression evidence that a create-new staging conflict preserves the pre-existing controlled file rather than claiming or deleting it.
- Added RustCrypto `sha2 0.11.0` with default features disabled and recorded its exact transitive graph.
- Added feature-gated deterministic AP214 cube generation. Timestamp normalization makes repeated 10 mm fixture generation reproduce SHA-256 `031304b3a6d9dd55a97b3329e7238286ccfdaa7f13030bbe6e5c4c5744fcc8a2`.
- Added a native regression test that regenerates the cube and byte-compares it with the committed STEP fixture.
- Added `FIX-STEP-001` manifest and schema-v2 expectations: one exact solid, 600 mm² area, 1000 mm³ volume, and `(5,5,5)` mm centroid within `0.000001`.
- Connected the optional worker to the OCCT adapter for the exact intake-through-basic-properties profile and a bounded schema-v1 provisional snapshot. The subprocess test passes staging, cleared-environment native loading, STEP transfer, measurements, opaque reference, and source cleanup.
- Documented six-decimal provisional measurement formatting after the first native run exposed `599.9999999999999`; raw native values remain internal and production tolerance policy remains open.
- Same-kernel fixture generation/read is integration evidence, not independent accuracy evidence. Descriptor-backed grants, sandbox/resource enforcement, independent STEP corpus, legal approval, and three-OS native packaging remain.
- Local default validation passes strict Clippy, 60 runtime tests, and the compile-fail doctest; ten focused native adapter/worker tests also pass.
- GitHub Actions run 30474167026 passes formatting, strict Clippy, all 58 default runtime tests, and documentation tests on Windows, Linux, and macOS at commit `ab89bab`; native OCCT evidence remains Apple Silicon only.
- Added `FIX-STEP-002`, a project-authored invalid-entity STEP fixture with a schema-v1 failure expectation. The adapter and supervised worker return recoverable `STEP_TRANSFER_FAILED`, produce no snapshot/output, and clean the staged source.
- GitHub Actions run 30475113583 passes formatting, strict Clippy, all 60 default runtime tests, and documentation tests on Windows, Linux, and macOS at commit `7dc1a81`; this validates the failure-expectation contract cross-platform, not native OCCT execution.

## 2026-07-29 — TASK-002 configurable rates and synthetic golden mechanics

- Adopted the approved product boundary: PartProbe supplies no numeric production rates; organizations configure their own effective-dated rate and pricing libraries, while synthetic values remain isolated test/demo evidence.
- Recorded USD as the initial setup suggestion for the expected common case; users must explicitly confirm or change it, and calculations retain an explicit currency rather than inferring locale.
- Added validated rate-card/entry IDs and versions, ISO 4217-style currency, typed cost category/basis/composition/scope, explicit ownership, inclusive effective periods, retained lifecycle decisions, actor/time/reason governance, source provenance, and empty production-card construction.
- Added deterministic ordered-scope resolution: exactly one approved effective match is selected and pinned with selector ID/version and the complete scope request; absence/unapproved/out-of-period yields `Unavailable`; equal matches yield `Blocked`.
- Added versioned pricing and rounding policies preserving method, thresholds, currency, named boundary, scale, mode, unrounded/rounded values, and policy identity/version.
- Implemented CALC-007–CALC-015 and CALC-018 foundations around the existing CALC-016/017 rules, including material, setup, cycle/run, operation, base, risk, rework, and price traces. CALC-009 is intentionally exact-only and rejects nonterminating setup amortization pending governed rational rounding; CALC-011 rejects duplicate and composite/component double charges.
- Added an explicitly `synthetic_test_only` EX-01/03/12 fixture with exact itemized intermediate/output traces and independent quantity breaks; values are deterministic software evidence, not production defaults or shop accuracy claims.
- Added pinned rate-card replay and serializable resolved-rate/rounding snapshot values.
- Local formatting, strict Clippy, 41 runtime tests, one compile-fail doctest, and planning validation pass. TASK-002 adds 18 runtime tests; GitHub Actions run 30461694180 passes Windows, Linux, and macOS.
- Updated product scope, requirements, data/version contracts, UX rate setup, validation, roadmap/backlog/milestones, ADR evidence, risks, assumptions, and release boundaries. TASK-007/M0.2 retains real-shop category/policy/calibration review.
- No UI framework, database, importer, geometry kernel, external integration, production defaults, or advanced engine was added.

## 2026-07-23 — TASK-001 calculation foundation

- Created a Cargo resolver-v3 workspace pinned to Rust 1.94.1 with `domain`, `estimation-engine`, and `test-support` crates.
- Added project-owned typed volume/density/mass/quantity and fixed-precision money/currency primitives, explicit value states, provenance, schema versions, and semantic rule references.
- Implemented CALC-001, CALC-003, CALC-005, CALC-016, and CALC-017 plus a typed DAG definition/validation boundary that rejects duplicate/missing/type-incompatible/cyclic nodes.
- Added deterministic snapshot JSON with ordered maps, decimal strings, schema/rule versions, provenance, warnings, and explicit result states.
- Added exact/property/boundary tests: 23 runtime tests and one compile-fail unit-safety doctest pass locally. Formatting and strict Clippy also pass.
- Independent review found and drove fixes for deserialization invariant bypass, omitted intermediate traces, scale/negative-zero canonicalization, and implicit precision loss in physical and monetary arithmetic.
- Added a Windows/Linux/macOS GitHub Actions matrix and pinned checkout action/toolchain. Run 30005930156 passes on all three platforms after adding an LF checkout contract for Windows.
- Added exact dependency and active-transitive evidence for `rust_decimal`, Serde, and Serde JSON. ADR-0007 remains In Review; canonical JSON remains an internal spike contract.
- No UI, geometry, persistence, runtime estimator, external integration, routing optimizer, or advanced-analysis engine was added.

## 2026-07-22 — Advanced planning enhancement opened

- **Documents reviewed:** `AGENTS.md`, `docs/INDEX.md`, `docs/PROJECT_STATE.md`, product vision/scope/non-goals/success metrics, roadmap/milestones/backlog/risks, estimating/geometry/feature/runtime/pricing/quality/security/actuals models, requirements matrix, data architecture, and ADR-0001–ADR-0008. No ADR is Approved; all remain In Review.
- **Existing requirements affected:** REQ-F-006–REQ-F-007, REQ-F-010–REQ-F-013, REQ-F-016–REQ-F-020, REQ-F-022–REQ-F-025, REQ-F-027–REQ-F-030, REQ-F-032–REQ-F-039; REQ-NF-002–REQ-NF-010; CALC-010–CALC-020; DATA-004, DATA-006–DATA-012, DATA-015–DATA-018; TIME-001–TIME-008; UX-001–UX-010 and UX-021–UX-027; SEC-001–SEC-010.
- **New systems in scope:** routing alternatives, capacity/opportunity cost, probabilistic estimates, geometric revision/cost difference, requirement coverage, PMI/MBD, feature-cost/risk visualization, correction learning, CAM reconciliation, availability-aware estimation, make/buy sourcing, and bid/no-bid priority scoring.
- **Expected ADRs:** routing/scoring boundary, capacity/opportunity-cost semantics, uncertainty representation, revision/PMI comparison, learning governance, and CAM integration contracts (provisionally ADR-0009–ADR-0014).
- **Major unresolved decisions:** shop bottlenecks and schedule data sources; uncertainty method and appetite; usable AP242/PMI translator coverage; CAM vendors/formats; inventory freshness ownership; requirement-readiness exception authority; contribution/opportunity-cost policy; correction-learning approval cohort.
- **Documents expected to change:** the mandatory files named by the enhancement prompt plus focused domain, architecture, UX, quality, ADR, source/index/state, risk, roadmap, milestone, and backlog documents. No production implementation is authorized in this session.

## 2026-07-22 — Advanced planning enhancement integrated

- Added planning for all twelve requested systems: alternative routing; capacity/opportunity economics; probabilistic estimation; revision/cost difference; manufacturing requirement coverage; correction learning; CAM reconciliation; availability-aware estimation; make/buy; feature-level cost/risk explanation; structured PMI/MBD; and bid/quote priority.
- Allocated REQ-F-040–065, REQ-NF-015–022, SEC-011–014, UX-028–045, CALC-021–035, DATA-019–042, and TEST-040–099 without marking them Complete.
- Added ADR-0009–ADR-0014 as In Review. Human adoption, baseline immutability, explicit unavailable/stale states, source/version pinning, vendor-neutral adapters, and no automatic learning activation are cross-cutting invariants.
- Refined Phases 0–8 and TASK-011–TASK-018. Phase 2 remains intentionally narrow: one route, basic requirement checklist, manual readiness/availability, and manual make/buy; uncertainty, capacity optimization, broad PMI, and governed learning are later gates.
- Expanded the risk register to RISK-036, open questions to OQ-050, assumptions to A-020, and added dedicated domain/architecture/UX/quality plans and research for PMI/MBD.
- Preserved the deterministic estimate as the accepted baseline and distinguished capability from readiness, confidence from uncertainty, accounting cost from opportunity cost, and recommendation from authority.
- Planning-only change: no Cargo workspace, production module, dependency, integration, or external data transmission was added. TASK-001 remains the exact next implementation task.

## 2026-07-22 — Phase 0 planning baseline

- Inspected the prompt-only repository and initialized Git on `main`.
- Established root governance and the requested documentation hierarchy.
- Researched current CAD/geometry, Rust desktop UI/rendering, machining/runtime, quality, controlled-data, security, persistence, and versioning topics with primary-source links.
- Proposed Tauri 2 + Leptos, OCCT in an isolated worker, wgpu rendering, STEP/STL/3MF initial support, SQLite standalone persistence, and typed deterministic calculation graphs; all architectural ADRs remain In Review pending spikes.
- Defined domain/data/calculation/geometry/runtime models, requirements and test traceability, eighteen synthetic worked scenarios, roadmap/backlog, risks, assumptions, and open shop questions.
- Added two public synthetic STL fixtures, SHA-256 manifest entries, and bootstrap expected-result records for harness bootstrap.
- Completed independent integration reviews across UI, CAD/geometry, machining/runtime, and security. Reconciled provisional IDs; corrected decision-status wording, representation provenance, inspection-time taxonomy, Tauri/WebView security controls, fixture policy, and usability target hierarchy.
- Verified 102 documentation files for required metadata and local links; `git diff --check` passed; both fixture hashes were reproduced. No production tests exist.
- No production modules or external data integrations were created.
- Next: complete M0.2 evidence tasks, beginning with TASK-001 while running the architecture spikes in parallel as staffing permits.
