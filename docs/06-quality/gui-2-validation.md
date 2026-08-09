# GUI-2 Application-Service Validation Evidence

> **Status:** Complete
> **Last updated:** 2026-08-09
> **Related requirements:** REQ-F-003–REQ-F-005, REQ-F-007–REQ-F-009, REQ-F-014; REQ-NF-003–REQ-NF-006, REQ-NF-010, REQ-NF-017, REQ-NF-020; TEST-002, TEST-003, TEST-006, TEST-014, TEST-021, TEST-030
> **Related ADRs:** ADR-0001, ADR-0002, ADR-0005, ADR-0007, ADR-0008
> **Open questions:** None for the session-only service; production unit-resolution, persistence, shop calibration, and desktop acceptance remain outside GUI-2
> **Dependencies:** TASK-001/002 calculation evidence and TASK-003 provisional worker evidence
> **Supersedes:** None

## Scope

GUI-2 adds a headless, ephemeral draft-estimate application service. It does not add a desktop shell, persistent estimate, approved quote, production importer, shop rate library, or accuracy claim.

`DraftEstimateApplication` composes the existing deny-by-default `LocalAssetReadService` with a `GeometryAnalysisPort`. The concrete `GeometryWorkerSupervisor` implementation consumes the already-open pathless grant, accepts only successful controlled output, decodes the pinned provisional snapshot reference, and binds the decoded evidence to the request's expected source hash.

`DraftEstimateSession` requires all of the following before returning an available value:

- explicit review of canonical units and the worker warning set;
- explicit stock volume and density;
- explicit deliver, spare, and destructive-sample quantities;
- explicit setup, programming, cutting, non-cutting, load/unload, in-cycle inspection, and post-cycle inspection times;
- explicit material, operation, base, risk, and expected-rework money inputs;
- one pinned rate card, effective date, and ordered scope list resolving setup, programming, run-labor, machine, and inspection rates; and
- one pinned pricing and rounding policy.

The application crate calls CALC-001/003/005/007/008/010–015/018 through `partprobe-estimation-engine`; no UI or application-owned duplicate formula was added. The result retains the provisional source/worker evidence, complete manual inputs, every selected immutable rate and selector trace, the pricing policy, rule IDs, intermediates, total internal cost, and rounded selling price.

## Executable evidence

| Test | Evidence |
|---|---|
| `authorized_analysis_starts_ephemeral_session_with_no_numeric_defaults` | Authorization/audit precede analysis; a successful provisional session begins unavailable until review and manual inputs exist |
| `denied_source_never_reaches_geometry_analysis` | A deny decision is audited and the geometry port is never called |
| `missing_and_conflicting_rates_never_collapse_to_zero` | An empty card yields `Unavailable`; equally applicable approved setup rates yield `Blocked` |
| `deterministic_recalculation_matches_golden_engine_trace_and_tracks_edits` | Two evaluations are equal, EX-01 reconciles exactly to material 100, operation 260, total 520, and price 702; changing setup from three to four hours deterministically produces setup 100, total 545, and price 735.75 with the edited input retained in trace; zero delivery blocks |

Focused commands:

```sh
cargo test -p partprobe-application --locked
cargo clippy -p partprobe-application --all-targets --locked -- -D warnings
```

Repository-wide formatting, strict Clippy, 106 default macOS runtime tests, the compile-fail doctest, 139-document planning validation, fixture hashes, six native-tooling tests, and diff checks pass locally. GitHub Actions run 30717609229 passes formatting, native tooling, all targets, and documentation on Windows, Linux, and macOS at implementation commit `ecbb0be`.

## Boundaries and remaining work

- The test analyzer is a headless port substitute. The concrete supervisor adapter is compiled here, while worker launch, controlled-output reconciliation, cancellation, containment, and native fixture behavior remain covered by TASK-003 tests.
- Geometry evidence is the existing `provisional_spike` exact-B-rep snapshot. GUI-2 changes no geometry interpretation, worker schema, adapter ABI, fixture expectation, or tolerance.
- GUI-2 changes no calculation rule or pricing behavior. It composes already documented rules and reuses the synthetic TASK-002 rate/pricing fixture; this is software reconciliation evidence, not shop accuracy or valid production pricing.
- The session is deliberately memory-only. Setters replace unapproved session inputs and are not the governed override/audit workflow required by REQ-F-010. Save/reopen, approval, replay persistence, migrations, and historical audit remain TASK-006 work.
- No third-party package was added. The application crate now directly depends on existing workspace crates `estimation-engine` and `geometry-core`, plus the already-pinned workspace decimal library, because it owns orchestration and typed result composition.
- GUI-3 now supplies the restrictive typed desktop shell but deliberately does not expose this service. GUI-4 must add the native application adapter, display provisional/ephemeral labels, and preserve `Unavailable`/`Blocked` states without reimplementing calculations.
