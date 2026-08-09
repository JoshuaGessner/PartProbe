# GUI-5 Configured Native Desktop Validation

> **Status:** Complete for the bounded internal developer checkpoint
> **Last updated:** 2026-08-09
> **Related requirements:** REQ-F-002–REQ-F-010, REQ-F-032; REQ-NF-001, REQ-NF-004–REQ-NF-006, REQ-NF-010–REQ-NF-014; TEST-002, TEST-003, TEST-008, TEST-011, TEST-012, TEST-014, TEST-019, TEST-021, TEST-024, TEST-030
> **Related ADRs:** ADR-0001–ADR-0003, ADR-0005, ADR-0007, ADR-0008
> **Open questions:** Formal GUI-1 fixture approval, packaged native runtime, three-platform native desktop evidence, full assistive-technology matrix
> **Dependencies:** GUI-1 implementation evidence, GUI-2 application service, GUI-3 shell, GUI-4 workspace
> **Supersedes:** None

## Acceptance boundary

GUI-5 completes the five-checkpoint **facts-and-estimate internal developer slice** on one Apple-Silicon macOS host. It is a reproducible developer validation of the actual Tauri application, pinned OCCT worker, one governed synthetic STEP fixture, explicit review/manual/rate/pricing inputs, and the existing deterministic estimate engine.

This evidence does not create a supported STEP importer, packaged OCCT runtime, signed installer, durable estimate, production rate library, approved quote, three-platform desktop capability, full TEST-012 accessibility result, or release acceptance. The application remains provisional, session-only, unsaved, and developer-configured. No 3D viewport exists.

## Reproducible native environment

The validation host was Apple Silicon (`arm64`) on macOS 26.5.2. The exact official OCCT `V8_0_0` source was checked out at commit `d3056ef80c9668f395da40f5fd7be186cae4501f`, tree `b3ffb8a91468845b63675057957209032b5806b1`, with a clean detached worktree. `scripts/build_occt.py` constructed and verified the reviewed minimal profile in explicit temporary source/build/install directories. The resulting worker was:

- path: `target/debug/partprobe-geometry-worker`;
- SHA-256: `3d3770c0cf95016800f1524f6be8fa7d6b670bac6d394b180331854ae5648482`;
- status: `developer_native_step_seam_verified`.

The construction command downloaded nothing and passed strict optional-native adapter/worker Clippy, 11 adapter tests, 15 supervised-worker tests, worker construction, install fingerprinting, and OCCT 8.0.0 verification. Source, build, install, worker workspace, and unsigned bundle remain local disposable developer artifacts and are not product packaging.

The native-feature and ordinary feature-off worker builds currently share the same Cargo output path. Therefore the runbook rebuilds `partprobe-geometry-worker` with `native-occt` immediately before configured smoke/launch, or uses a separately copied verified executable. Closeout deliberately confirmed that a feature-off replacement fails safely instead of silently parsing or reporting success.

## Automated configured-host smoke

`gui5_configured_worker_runs_real_step_through_retained_estimate_session` is an ignored, `desktop-host`-gated test so ordinary CI and machines without OCCT continue to fail closed. It runs only when all three explicit native paths are supplied. The test sends `FIX-STEP-003` through the real configured supervisor/OCCT worker, retains the native draft session, submits the complete synthetic golden request, and asserts:

- 392 mm² surface area, 480 mm³ volume, and `(6, 4, 2.5)` mm centroid;
- one solid body and OCCT 8.0.0 evidence;
- rounded USD 702 selling price through the existing application/engine path;
- path redaction across the presentation result.

```sh
PARTPROBE_GEOMETRY_WORKER="$PWD/target/debug/partprobe-geometry-worker" \
PARTPROBE_GEOMETRY_WORKSPACE=/private/tmp/partprobe-gui-worker \
PARTPROBE_OCCT_ROOT=/private/tmp/partprobe-occt-install \
  cargo test -p partprobe-estimator-desktop --features desktop-host \
    gui5_configured_worker_runs_real_step_through_retained_estimate_session \
    --locked -- --ignored --nocapture
```

Result: one configured native smoke test passed in 0.24 seconds on the final closeout run. Ordinary desktop-host execution passes 17 tests and reports this test ignored unless the developer opts in. The focused contract/UI/native-host set totals 29 passing tests plus that one ignored smoke.

GitHub Actions run 31340568277 passes formatting, six native-tooling tests, strict workspace/native-host/WASM lint, all-target tests, and doctests on macOS, Ubuntu, and Windows at GUI-5 implementation commit `40c20a2`. The native OCCT smoke remains explicit Apple-Silicon local evidence and is not run by that feature-off default matrix.

## Actual application checklist

An unsigned debug `PartProbe.app` was built from the current bundled frontend and launched by its executable with the same three explicit native environment paths. The manual checklist passed:

- keyboard focus reached the skip link, native model picker, Analyze action, every reviewed estimate field, both review confirmations, five rate inputs and confirmation, pricing inputs and confirmation, Calculate action, and the result trace;
- Return opened the native picker, selected `rectangular_prism_12x8x5.step`, invoked analysis, and submitted the estimate without a pointer-only dependency;
- the app displayed 392 mm², 480 mm³, `(6, 4, 2.5)` mm, one solid body, canonical millimeters, zero warnings, OCCT 8.0.0, and an opaque analysis ID;
- the complete synthetic inputs reconciled to material USD 100, operation USD 260, base internal USD 485, risk reserve USD 35, total internal USD 520, and rounded selling price USD 702;
- the expanded calculation/rate trace exposed CALC rule IDs, pricing-policy identity/version, five selected rates, rate-card identity/version, and effective date;
- the accessibility tree exposed semantic headings, status regions, labels and units for every input, native checkbox/button roles, and the result description list;
- developer-alpha, provisional, session-only, not-saved, unapproved, and not-a-customer-quote boundaries remained visible;
- only the selected leaf filename and opaque native session authority appeared; no absolute source path appeared in the UI or safe diagnostics.

## Cancellation, failure, and recovery

The checked-in delayed cancellation worker fixture kept analysis active long enough to exercise the actual Cancel button. The UI exposed Running, enabled Cancel, issued token-bound cooperative cancellation, waited for cleanup, then displayed a distinct `Analysis cancelled` state with the selected source available for retry. A contract/UI regression prevents cancellation from being presented as a failure.

The deliberately invalid `invalid_entity.step` fixture produced a bounded `GUI4-ANALYSIS-WORKER` diagnostic, no path disclosure, no numeric result, and an enabled retry/selection path. Selecting the valid prism afterward succeeded through the real OCCT worker as a new opaque analysis. Re-analysis now clears both canonical-unit and warning-review confirmations; a focused state test and a rebuilt-app accessibility check prove those confirmations cannot silently carry across analysis revisions.

## Remaining evidence and next development gate

GUI-5 and GUI-4 are complete only for this bounded developer checkpoint. GUI-1 formal fixture/security review remains open in parallel. The next product-facing work is not another form demo: it is packaged native construction on macOS/Windows/Linux, remaining worker containment, a supported importer accuracy corpus, durable shop-owned rate/policy setup, persistence/save/reopen/replay, broader STEP/STL/3MF intake, and release accessibility/usability/security evidence. A visible model adds the separate tessellation/viewer checkpoints in the vertical-slice plan.

No calculation formula, geometry interpretation, worker schema/ABI, persisted/customer schema, production rate, third-party package/version, external data transfer, or compliance/support claim changed in GUI-5.
