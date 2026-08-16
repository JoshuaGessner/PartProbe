# GUI-5 Configured Native Desktop Validation

> **Status:** Complete for the bounded internal developer checkpoint
> **Last updated:** 2026-08-16
> **Related requirements:** REQ-F-002–REQ-F-010, REQ-F-032; REQ-NF-001, REQ-NF-004–REQ-NF-006, REQ-NF-010–REQ-NF-014; TEST-002, TEST-003, TEST-008, TEST-011, TEST-012, TEST-014, TEST-019, TEST-021, TEST-024, TEST-030
> **Related ADRs:** ADR-0001–ADR-0003, ADR-0005, ADR-0007, ADR-0008
> **Open questions:** Formal GUI-1 fixture approval, signed/embedded native distribution, three-platform native desktop evidence, full assistive-technology matrix
> **Dependencies:** GUI-1 implementation evidence, GUI-2 application service, GUI-3 shell, GUI-4 workspace
> **Supersedes:** None

## Acceptance boundary

GUI-5 completes the five-checkpoint **facts-and-estimate internal developer slice** on one Apple-Silicon macOS host. It is a reproducible developer validation of the actual Tauri application, pinned OCCT worker, one governed synthetic STEP fixture, explicit review/manual/rate/pricing inputs, and the existing deterministic estimate engine.

This evidence does not create a supported STEP importer, embedded/signed/distributable OCCT runtime, durable estimate, production rate library, approved quote, three-platform desktop capability, full TEST-012 accessibility result, or release acceptance. The application remains provisional, session-only, unsaved, and developer-configured. No 3D viewport exists.

## Reproducible native environment

The validation host was Apple Silicon (`arm64`) on macOS 26.5.2. The exact official OCCT `V8_0_0` source was checked out at commit `d3056ef80c9668f395da40f5fd7be186cae4501f`, tree `b3ffb8a91468845b63675057957209032b5806b1`, with a clean detached worktree. `scripts/build_occt.py` constructed and verified the reviewed minimal profile in explicit temporary source/build/install directories. The resulting worker was:

- path: `target/debug/partprobe-geometry-worker`;
- SHA-256: `3d3770c0cf95016800f1524f6be8fa7d6b670bac6d394b180331854ae5648482`;
- status: `developer_native_step_seam_verified`.

The construction command downloaded nothing and passed strict optional-native adapter/worker Clippy, 11 adapter tests, 15 supervised-worker tests, worker construction, install fingerprinting, and OCCT 8.0.0 verification. Source, build, install, worker workspace, and unsigned bundle remain local disposable developer artifacts and are not product packaging.

The native-feature and ordinary feature-off worker builds share the same Cargo output path. The subsequent TASK-003 runtime checkpoints now assemble the verified worker, construction provenance, version header, and all 23 observed OCCT library families into a separate content-verified developer root, then repeat those checks inside the desktop host before deriving its launch paths. This is the preferred follow-on smoke/launch path because later Cargo builds cannot replace it and independent worker/library overrides are no longer accepted. Closeout still confirmed that a feature-off replacement fails safely instead of silently parsing or reporting success.

## Automated configured-host smoke

`gui5_configured_worker_runs_real_step_through_retained_estimate_session` is an ignored, `desktop-host`-gated test so ordinary CI and machines without OCCT continue to fail closed. It runs only when the explicit native-runtime root and external workspace are supplied and the runtime passes in-host verification. The test sends `FIX-STEP-003` through the real configured supervisor/OCCT worker, retains the native draft session, submits the complete synthetic golden request, and asserts:

- 392 mm² surface area, 480 mm³ volume, and `(6, 4, 2.5)` mm centroid;
- one solid body and OCCT 8.0.0 evidence;
- rounded USD 702 selling price through the existing application/engine path;
- path redaction across the presentation result.

```sh
PARTPROBE_NATIVE_RUNTIME=/approved/local/partprobe-native-runtime \
PARTPROBE_GEOMETRY_WORKSPACE=/private/tmp/partprobe-gui-worker \
  cargo test -p partprobe-estimator-desktop --features desktop-host \
    gui5_configured_worker_runs_real_step_through_retained_estimate_session \
    --locked -- --ignored --nocapture
```

Result: the explicit developer-runtime smoke passed against the original construction root during GUI-5 closeout, against the separately assembled runtime during the next TASK-003 checkpoint, and in 0.68 seconds after desktop startup enforcement was added. Ordinary desktop-host execution now passes 19 tests and reports that smoke plus the packaged-resource smoke ignored unless explicitly configured. Five portable runtime-verifier tests independently cover the valid path, worker tampering plus extra files, traversal/unreviewed configuration, wrong copied build provenance, and wrong-host rejection. The focused contract/UI/native-host set now has 31 passing tests plus those two ignored smokes; the verifier coverage is counted separately in the workspace suite. The package-aware smoke reuses the same STEP-to-USD-702 assertions, and combined Linux package run 31624958771 supplies its configured hosted evidence.

GitHub Actions run 31340568277 passes formatting, six native-tooling tests, strict workspace/native-host/WASM lint, all-target tests, and doctests on macOS, Ubuntu, and Windows at GUI-5 implementation commit `40c20a2`. The native OCCT smoke remains explicit Apple-Silicon local evidence and is not run by that feature-off default matrix.

Follow-on GitHub Actions run 31346159820 passes the current eleven-tooling-test and 142-runtime-test matrix plus all lint/doctest gates on macOS, Ubuntu, and Windows at startup-verifier implementation commit `86e00b5`. It validates the portable verifier logic but still does not construct or launch OCCT on Windows/Linux.

Later TASK-003 native run 31555774851 passes this same ignored configured-host smoke in 3.27 seconds on Ubuntu 24.04 x86_64 after exact OCCT construction, runtime assembly/re-verification, and internal dynamic-link auditing. That extends the application-service/worker path to one Linux host under seccomp, but does not repeat this document's actual Tauri-window, keyboard, semantic, cancellation, or recovery checklist on Linux.

## Actual application checklist

An unsigned debug `PartProbe.app` was originally built and launched with three explicit native environment paths. After host-side runtime enforcement, the current bundle was rebuilt and the complete success path was repeated with only `PARTPROBE_NATIVE_RUNTIME` and `PARTPROBE_GEOMETRY_WORKSPACE`. Both checklists passed; the current composition repeat specifically confirmed native selection, in-host runtime verification, real OCCT analysis, complete explicit review/input/rate/pricing submission, the semantic result trace, path redaction, and clean application exit:

- keyboard focus reached the skip link, native model picker, Analyze action, every reviewed estimate field, both review confirmations, five rate inputs and confirmation, pricing inputs and confirmation, Calculate action, and the result trace;
- Return opened the native picker, selected `rectangular_prism_12x8x5.step`, invoked analysis, and submitted the estimate without a pointer-only dependency;
- the app displayed 392 mm², 480 mm³, `(6, 4, 2.5)` mm, one solid body, canonical millimeters, zero warnings, OCCT 8.0.0, and an opaque analysis ID;
- the complete synthetic inputs reconciled to material USD 100, operation USD 260, base internal USD 485, risk reserve USD 35, total internal USD 520, and rounded selling price USD 702;
- the expanded calculation/rate trace exposed CALC rule IDs, pricing-policy identity/version, five selected rates, rate-card identity/version, and effective date;
- the accessibility tree exposed semantic headings, status regions, labels and units for every input, native checkbox/button roles, and the result description list;
- developer-alpha, provisional, session-only, not-saved, unapproved, and not-a-customer-quote boundaries remained visible;
- only the selected leaf filename and opaque native session authority appeared; no absolute source path appeared in the UI or safe diagnostics.

The current-composition repeat displayed the same 392 mm², 480 mm³, `(6, 4, 2.5)` mm, one-solid, zero-warning OCCT 8.0.0 evidence and reconciled to material USD 100, operation USD 260, base internal USD 485, risk reserve USD 35, total internal USD 520, and rounded selling price USD 702. It used the rebuilt unsigned `.app` and the separately assembled 49 MiB runtime; it does not add signing, persistence, supported-importer, Windows native, or Linux window/package evidence.

## Cancellation, failure, and recovery

The checked-in delayed cancellation worker fixture kept analysis active long enough to exercise the actual Cancel button. The UI exposed Running, enabled Cancel, issued token-bound cooperative cancellation, waited for cleanup, then displayed a distinct `Analysis cancelled` state with the selected source available for retry. A contract/UI regression prevents cancellation from being presented as a failure.

The deliberately invalid `invalid_entity.step` fixture produced a bounded `GUI4-ANALYSIS-WORKER` diagnostic, no path disclosure, no numeric result, and an enabled retry/selection path. Selecting the valid prism afterward succeeded through the real OCCT worker as a new opaque analysis. Re-analysis now clears both canonical-unit and warning-review confirmations; a focused state test and a rebuilt-app accessibility check prove those confirmations cannot silently carry across analysis revisions.

## Remaining evidence and next development gate

GUI-5 and GUI-4 are complete only for this bounded developer checkpoint. GUI-1 formal fixture/security review remains open in parallel. Linux and Windows configured-native headless evidence pass, and Linux additionally passes the verified-runtime Debian package-integration checkpoint. The next product-facing work is not another form demo: it is bounded interactive Linux/Windows evidence, Windows/macOS package integration, signed application/runtime integration, remaining worker containment, a supported importer accuracy corpus, durable shop-owned rate/policy setup, persistence/save/reopen/replay, broader STEP/STL/3MF intake, and release accessibility/usability/security evidence. A visible model adds the separate tessellation/viewer checkpoints in the vertical-slice plan.

No calculation formula, geometry interpretation, worker schema/ABI, persisted/customer schema, production rate, third-party package/version, external data transfer, or compliance/support claim changed in GUI-5.
