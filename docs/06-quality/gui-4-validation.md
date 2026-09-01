# GUI-4 Analysis and Estimate Workspace Validation

> **Status:** Complete for the bounded workspace checkpoint
> **Last updated:** 2026-09-01
> **Related requirements:** REQ-F-002–REQ-F-010, REQ-F-032; REQ-NF-004, REQ-NF-005, REQ-NF-017; SEC-003, SEC-004; TEST-003, TEST-011, TEST-014, TEST-021, TEST-030
> **Related ADRs:** ADR-0001, ADR-0002, ADR-0005, ADR-0007, ADR-0008
> **Open questions:** Native worker packaging and durable authorization/audit policy
> **Dependencies:** GUI-1 evidence, GUI-2 application service, GUI-3 shell
> **Supersedes:** None

## Implemented scope

GUI-4 is complete for its bounded, session-only workspace checkpoint. `DraftGeometryRequestTemplate` carries validated path-free job, correlation, capability, stages, profile, and quota intent without falsely claiming a source hash. `DraftEstimateApplication::start_session_from_unfingerprinted_source` authorizes and audits the selected relative source, obtains one already-open grant, fingerprints its bytes within the request's maximum-input quota, constructs the source-bound worker request, and passes the rewound same grant to the configured geometry port.

`AssetReadGrant::fingerprint_sha256` checks the captured file length, enforces the nonzero maximum input size, computes typed SHA-256, and rewinds the open handle before return. This fingerprint is not accepted as worker truth: the existing supervisor/worker transport independently rechecks regular-file identity, length, quota, and hash before parsing.

The desktop integration now uses contract v4. It retains v3's exact five commands and one event while changing the session-only analysis DTO into a tagged exact-B-rep/mesh result. `analyze_model_source` accepts only an opaque selection token and runs blocking application/worker work outside the UI task. `cancel_model_analysis` signals only the matching active selection's cooperative token; supervisor cleanup remains bounded by the existing forced-termination path. GUI-4 originally required three explicit worker/workspace/library paths. The later TASK-003 checkpoint narrows current production composition to `PARTPROBE_NATIVE_RUNTIME` plus `PARTPROBE_GEOMETRY_WORKSPACE`; the host derives worker/library paths only after verifying the runtime manifest. Missing or invalid configuration remains `AnalysisUnavailable`. A narrow developer-session policy and append-only in-memory audit govern the selected local root, the application service constructs the request and session, and native state retains the complete `DraftEstimateSession` including the validated controlled result. The WebView receives only path-free selection/analysis/session IDs, hashes, sanitized stage/warning evidence, and content-minimized geometry facts.

For exact STEP, v4 preserves the existing canonical-millimetre fields and deterministic estimate behavior. For STL/3MF, it exposes detected format, optional STL encoding, source units and resolution, source-coordinate versus canonical-millimetre measurement basis, decimal-text extents/area, optional volume/centroid, topology booleans, three-state self-intersection, categorical confidence/reasons, algorithm/policy versions, topology identity, explicit not-applied welding, warnings, and source/output lineage. STL values are never labeled millimetres. Missing mesh measurements remain absent and display as withheld. The application rejects every mesh estimate as `Unavailable` before review or supplied estimate inputs can authorize calculation; the frontend also does not render the estimate form for mesh. A selected extension/content mismatch fails visibly. Full format-owned 3MF component/metadata provenance remains native and does not cross the bridge.

After successful analysis, the Leptos workspace requires explicit canonical-unit and complete-warning-set confirmation, exact text for all manual stock/density/quantity/time/material/operation/base/risk inputs, five explicit confirmed developer-session hourly rates with card/date/scope context, and a confirmed versioned markup/rounding policy. `evaluate_draft_estimate` parses and validates those fields in the native adapter, constructs typed governance objects, updates the retained session only after the full request is valid, and calls `DraftEstimateSession::evaluate`. The path-free result preserves Available, Unavailable, and Blocked states and returns the selling price, itemized cost trace, selected-rate evidence, pricing version/rounding context, and rule IDs. The UI contains no CAD parsing, hashing, rate resolution, pricing, or estimate formulas, and no production numeric defaults.

## Focused evidence

- A bounded synthetic source fingerprints to its known SHA-256 value, and reading the grant immediately afterward returns the complete original bytes from offset zero.
- An allowed application request appends one authorization audit event, produces the expected path-free request digest at the analysis port, and returns a provisional session that remains `Unavailable` before review/manual/rate/policy input.
- A denied request names a nonexistent relative source yet returns the policy failure, records one decision event, and performs no analysis. This proves policy/audit precede filesystem resolution and fingerprinting.
- A positive desktop adapter test analyzes the independently authored rectangular-prism fixture through a deterministic `GeometryAnalysisPort`, records one authorization event, returns 392 mm² / 480 mm³ / `(6, 4, 2.5)` mm path-free facts, then submits complete explicit golden inputs/rates/policy through the retained session and reconciles the expected rounded USD 702 selling price through the existing engine.
- Host tests prove stale tokens fail before analysis/evaluation, missing runtime configuration remains unavailable, cancellation is token-bound and idempotent for stale selections, the Tauri manifest/capability expose exactly five intended commands, analysis/evaluation use background tasks, and WebView requests carry no `PathBuf`. Contract/UI tests preserve provisional, failure, blocked, and result language without implying an approved quote.
- Native composition tests prove unconfirmed rates and pricing fail closed; form fields provide no shop-owned numeric defaults, and users must explicitly enter zero where intended.
- Contract-v4 tests preserve the same five-command/capability boundary, recognize only STEP/STL/3MF extensions, round-trip tagged exact and mesh DTOs, keep withheld volume/centroid as `null`, and prove no source path crosses serialization.
- Application and native-adapter tests retain governed STL evidence in a reviewable session, preserve unknown units/topology/confidence/warnings, and prove that a complete explicit estimate request still returns `Unavailable`. UI tests verify unknown-unit and withheld-volume accessibility language without implying cubic millimetres.

Focused command:

```sh
cargo test -p partprobe-geometry-import -p partprobe-application --locked
cargo test -p partprobe-desktop-contract -p partprobe-estimator-desktop-ui --all-targets --locked
cargo test -p partprobe-estimator-desktop --features desktop-host --all-targets --locked
```

The current focused contract/application/native-host/UI set passes 53 tests, including seven draft-application, twenty-two native-host, and twelve frontend-state cases. Seventy-four Python native-tooling tests and strict focused native/WASM Clippy pass. Corrected GitHub Actions run 33546668622 passes Windows, Ubuntu, and macOS at exact contract-v4 commit `7f2c378`, including Windows teardown after the application mesh test drops its session/root before shared cleanup. GUI-5 also supplied exact pinned OCCT construction, one passing opt-in real-worker STEP host smoke, and actual-app keyboard/semantic/cancellation/failure/recovery evidence; see [GUI-5 validation](gui-5-validation.md). The configured real-worker desktop mesh smoke is implemented, locally compiled, and wired into the opt-in Linux and Windows workflows; those exact-commit native executions remain in progress.

## Acceptance conclusion and remaining product evidence

GUI-4 is complete only for the deliberately bounded exact-STEP workspace checkpoint because GUI-5 ran the full configured native fixture workflow, exercised live cooperative cancellation and malformed-worker recovery, completed the estimate by keyboard, inspected semantic labels/roles/results, and verified review confirmations reset on re-analysis. Contract v4 adds internal mesh presentation evidence but does not expand that completion claim. Full TEST-012 assistive-technology/contrast/scaling/HiDPI evidence, configured real-worker desktop mesh evidence, durable rate/persistence workflows, supported importer validation, and release acceptance remain open. No calculation formula or mesh geometry interpretation changed; no new dependency, worker protocol/schema, native ABI, persisted/customer schema, production rate, durable approval, or support claim was introduced.

Contract v4 is an intentionally incompatible developer-bridge schema replacement for v3. Host and bundled UI are built and shipped together and discover the version before use; no v3 persisted/customer record or independent supported client exists, so no data migration is required. Historical v3 binaries are not relabeled as v4, and any future incompatible bridge change requires a new contract version plus an explicit compatibility/migration decision.
