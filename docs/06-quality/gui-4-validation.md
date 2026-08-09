# GUI-4 Analysis and Estimate Workspace Validation

> **Status:** In Progress
> **Last updated:** 2026-08-09
> **Related requirements:** REQ-F-002–REQ-F-010, REQ-F-032; REQ-NF-004, REQ-NF-005, REQ-NF-017; SEC-003, SEC-004; TEST-003, TEST-011, TEST-014, TEST-021, TEST-030
> **Related ADRs:** ADR-0001, ADR-0002, ADR-0005, ADR-0007, ADR-0008
> **Open questions:** Native worker packaging, desktop cancellation ownership, durable authorization/audit policy, complete estimate-input/result DTO
> **Dependencies:** GUI-1 evidence, GUI-2 application service, GUI-3 shell
> **Supersedes:** None

## Current implemented slice

GUI-4 is not complete. Its first blocking application seam is implemented: `DraftGeometryRequestTemplate` carries validated path-free job, correlation, capability, stages, profile, and quota intent without falsely claiming a source hash. `DraftEstimateApplication::start_session_from_unfingerprinted_source` authorizes and audits the selected relative source, obtains one already-open grant, fingerprints its bytes within the request's maximum-input quota, constructs the source-bound worker request, and passes the rewound same grant to the configured geometry port.

`AssetReadGrant::fingerprint_sha256` checks the captured file length, enforces the nonzero maximum input size, computes typed SHA-256, and rewinds the open handle before return. This fingerprint is not accepted as worker truth: the existing supervisor/worker transport independently rechecks regular-file identity, length, quota, and hash before parsing.

The second slice connects that seam to desktop contract v2. `analyze_model_source` accepts only an opaque selection token and runs blocking application/worker work outside the UI task. The native adapter requires explicit `PARTPROBE_GEOMETRY_WORKER`, `PARTPROBE_GEOMETRY_WORKSPACE`, and `PARTPROBE_OCCT_ROOT` configuration; missing or invalid paths remain `AnalysisUnavailable`. A narrow developer-session policy and append-only in-memory audit govern the selected local root, the application service constructs the request and session, and native state retains the complete `DraftEstimateSession`. The WebView receives only the selection/analysis IDs, hashes, stage names/status/warning codes, provisional canonical-millimeter exact-B-rep facts, engine/ABI evidence, and an explicit `Unavailable` estimate reason.

The Leptos workspace now renders not-started, running, safe failure, and provisional-evidence states. It labels all work session-only and provisional, keeps review/rate/price states unavailable, and does not contain CAD parsing, hashing, rate resolution, pricing, or estimate formulas.

## Focused evidence

- A bounded synthetic source fingerprints to its known SHA-256 value, and reading the grant immediately afterward returns the complete original bytes from offset zero.
- An allowed application request appends one authorization audit event, produces the expected path-free request digest at the analysis port, and returns a provisional session that remains `Unavailable` before review/manual/rate/policy input.
- A denied request names a nonexistent relative source yet returns the policy failure, records one decision event, and performs no analysis. This proves policy/audit precede filesystem resolution and fingerprinting.
- A positive desktop adapter test analyzes the independently authored rectangular-prism fixture through a deterministic `GeometryAnalysisPort`, records one authorization event, retains a session whose estimate is still `Unavailable`, returns 392 mm² / 480 mm³ / `(6, 4, 2.5)` mm path-free facts, and proves serialized output contains no source path.
- Host tests prove stale tokens fail before analysis, missing worker configuration remains unavailable, the Tauri manifest/capability expose exactly the three intended commands, analysis uses a background task, and the WebView request carries no `PathBuf`. Contract/UI tests preserve provisional and failure language without implying an estimate.

Focused command:

```sh
cargo test -p partprobe-geometry-import -p partprobe-application --locked
cargo test -p partprobe-desktop-contract -p partprobe-estimator-desktop-ui --all-targets --locked
cargo test -p partprobe-estimator-desktop --features desktop-host --all-targets --locked
```

The local macOS default workspace passes 128 runtime tests. The GUI-4 focused contract/native-host/UI set passes 20 tests, strict native-host and WASM Clippy pass, and the offline release frontend bundle plus a 1280 × 900 semantic/visual preview pass without browser diagnostics. Main run 31331776813 is the latest pushed three-OS evidence and predates this continuation. No pinned OCCT installation was present in this session, so the actual configured desktop-to-native-worker success path remains a GUI-5 smoke prerequisite; existing optional-native adapter/supervisor tests remain the native execution evidence.

## Remaining GUI-4 exit work

GUI-4 still needs cancellation and explicit canonical-unit/warning review, complete manual stock/material/time/cost/quantity inputs, pinned rate-card/effective-date/ordered-scope context, pinned pricing policy, safe result/trace DTOs, and the corresponding Leptos forms. The code can now analyze a selected STEP source only when the external developer worker and pinned OCCT paths are configured; it still cannot produce an estimate. No calculation behavior, geometry interpretation, worker protocol/schema, native ABI, persisted/customer schema, production rate, third-party dependency, or support claim changed in this slice.
