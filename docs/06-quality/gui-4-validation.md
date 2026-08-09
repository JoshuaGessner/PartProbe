# GUI-4 Analysis and Estimate Workspace Validation

> **Status:** In Progress
> **Last updated:** 2026-08-09
> **Related requirements:** REQ-F-002–REQ-F-010, REQ-F-032; REQ-NF-004, REQ-NF-005, REQ-NF-017; SEC-003, SEC-004; TEST-003, TEST-011, TEST-014, TEST-021, TEST-030
> **Related ADRs:** ADR-0001, ADR-0002, ADR-0005, ADR-0007, ADR-0008
> **Open questions:** Native worker packaging, durable authorization/audit policy, configured end-to-end test environment
> **Dependencies:** GUI-1 evidence, GUI-2 application service, GUI-3 shell
> **Supersedes:** None

## Implemented scope

GUI-4 is not complete. Its first blocking application seam is implemented: `DraftGeometryRequestTemplate` carries validated path-free job, correlation, capability, stages, profile, and quota intent without falsely claiming a source hash. `DraftEstimateApplication::start_session_from_unfingerprinted_source` authorizes and audits the selected relative source, obtains one already-open grant, fingerprints its bytes within the request's maximum-input quota, constructs the source-bound worker request, and passes the rewound same grant to the configured geometry port.

`AssetReadGrant::fingerprint_sha256` checks the captured file length, enforces the nonzero maximum input size, computes typed SHA-256, and rewinds the open handle before return. This fingerprint is not accepted as worker truth: the existing supervisor/worker transport independently rechecks regular-file identity, length, quota, and hash before parsing.

The desktop integration now uses contract v3. `analyze_model_source` accepts only an opaque selection token and runs blocking application/worker work outside the UI task. `cancel_model_analysis` signals only the matching active selection's cooperative token; supervisor cleanup remains bounded by the existing forced-termination path. The native adapter requires explicit `PARTPROBE_GEOMETRY_WORKER`, `PARTPROBE_GEOMETRY_WORKSPACE`, and `PARTPROBE_OCCT_ROOT` configuration; missing or invalid paths remain `AnalysisUnavailable`. A narrow developer-session policy and append-only in-memory audit govern the selected local root, the application service constructs the request and session, and native state retains the complete `DraftEstimateSession`. The WebView receives only path-free selection/analysis/session IDs, hashes, sanitized stage/warning evidence, and provisional canonical-millimeter exact-B-rep facts.

After successful analysis, the Leptos workspace requires explicit canonical-unit and complete-warning-set confirmation, exact text for all manual stock/density/quantity/time/material/operation/base/risk inputs, five explicit confirmed developer-session hourly rates with card/date/scope context, and a confirmed versioned markup/rounding policy. `evaluate_draft_estimate` parses and validates those fields in the native adapter, constructs typed governance objects, updates the retained session only after the full request is valid, and calls `DraftEstimateSession::evaluate`. The path-free result preserves Available, Unavailable, and Blocked states and returns the selling price, itemized cost trace, selected-rate evidence, pricing version/rounding context, and rule IDs. The UI contains no CAD parsing, hashing, rate resolution, pricing, or estimate formulas, and no production numeric defaults.

## Focused evidence

- A bounded synthetic source fingerprints to its known SHA-256 value, and reading the grant immediately afterward returns the complete original bytes from offset zero.
- An allowed application request appends one authorization audit event, produces the expected path-free request digest at the analysis port, and returns a provisional session that remains `Unavailable` before review/manual/rate/policy input.
- A denied request names a nonexistent relative source yet returns the policy failure, records one decision event, and performs no analysis. This proves policy/audit precede filesystem resolution and fingerprinting.
- A positive desktop adapter test analyzes the independently authored rectangular-prism fixture through a deterministic `GeometryAnalysisPort`, records one authorization event, returns 392 mm² / 480 mm³ / `(6, 4, 2.5)` mm path-free facts, then submits complete explicit golden inputs/rates/policy through the retained session and reconciles the expected rounded USD 702 selling price through the existing engine.
- Host tests prove stale tokens fail before analysis/evaluation, missing worker configuration remains unavailable, cancellation is token-bound and idempotent for stale selections, the Tauri manifest/capability expose exactly five intended commands, analysis/evaluation use background tasks, and WebView requests carry no `PathBuf`. Contract/UI tests preserve provisional, failure, blocked, and result language without implying an approved quote.
- Native composition tests prove unconfirmed rates and pricing fail closed; form fields provide no shop-owned numeric defaults, and users must explicitly enter zero where intended.

Focused command:

```sh
cargo test -p partprobe-geometry-import -p partprobe-application --locked
cargo test -p partprobe-desktop-contract -p partprobe-estimator-desktop-ui --all-targets --locked
cargo test -p partprobe-estimator-desktop --features desktop-host --all-targets --locked
```

The local macOS default workspace passes 134 runtime tests. The GUI-4 focused contract/native-host/UI set passes 26 tests, strict workspace/native-host/WASM Clippy pass, and the offline release frontend bundle plus an initial semantic/visual preview pass without browser diagnostics. GitHub Actions run 31336768681 passes formatting, six native-tooling tests, all three lint surfaces, all-target tests, and doctests on macOS, Ubuntu, and Windows at contract-v3 implementation commit `2868212`. No pinned OCCT installation was present in this session, so the configured desktop-to-native-worker-to-estimate success path and live post-analysis form remain GUI-5 smoke prerequisites; existing optional-native adapter/supervisor tests remain the native execution evidence.

## Remaining acceptance evidence

GUI-4's bounded implementation scope is complete, but the requirement remains In Progress until GUI-5 runs the full configured native fixture workflow, observes cooperative cancellation and worker-failure recovery, and completes keyboard/accessibility checks. No calculation formula or geometry interpretation changed; the existing `rust_decimal` workspace package is now a direct desktop-host dependency for exact parsing, but no new package/version, worker protocol/schema, native ABI, persisted/customer schema, production rate, durable approval, or support claim was introduced.
