# PartProbe

> **Status:** In Review
> **Last updated:** 2026-09-13
> **Related requirements:** REQ-F-001–REQ-F-065; REQ-NF-001–REQ-NF-022
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** OQ-001–OQ-050
> **Dependencies:** Planning review and technical spikes
> **Supersedes:** None

PartProbe is a local-first, cross-platform CAD-assisted estimator being built for specialty machine shops. Its target workflow imports engineering models, extracts explainable geometry facts, lets estimators review manufacturing assumptions, and produces traceable cost and price foundations. It is an estimating assistant—not production CAM, an autonomous quoting system, or accounting software.

## Production status

PartProbe is currently a **pre-alpha engineering foundation**, not an installable end-user product. Phase 0 evidence and M1 foundation implementation are running together while the architecture decisions remain In Review.

| Area | Current evidence | Production limitation |
|---|---|---|
| Calculations | Typed money/units, deterministic rules, itemized traces, versioned snapshots/replay, a headless GUI-2 session service, and a contract-v10 developer proposal that makes exact STEP bounds affect stock/material/coarse-runtime inputs before explicit adoption | No durable estimate, shop calibration, standard-stock/availability resolution, detailed route/runtime/adders, or approved quote workflow |
| Rates and resources | Empty-on-install, user-owned rate cards; durable schema-v2 starter Settings plus governed catalog evidence; contract-v10 path-free exact STEP bounds, model-sensitive stock/material/coarse-runtime proposal, and explicit session-only adoption trace | The live-test bridge requires per-estimate review and lists unmodeled exclusions; it is not CAM, catalog activation, a complete manufacturing estimate, or production authority; ordinary catalog editing/activation remains identity-unavailable and deny-all; PartProbe supplies no production rates |
| Geometry worker | Bounded control schema v2/transport manifest v2, explicit verified-copy transport, exact Unix descriptor and Windows HANDLE direct allowlisting with unrelated-resource exclusion, worker-side identity/type/hash/length/quota verification, private workspaces, cancellation grace/acknowledgement, forced termination, aggregate-output monitoring, Linux parser socket/process syscall denial including a configured native-OCCT smoke, partial CPU/file/process containment, audit/security seams, and governed derivative handoff | No general filesystem sandbox, macOS/Windows parser egress denial, complete cross-platform resource containment, or durable controlled store |
| STEP/OCCT | Optional OCCT 8.0 measurement ABI v4 parses exact verified bytes and emits `geometry-step-analysis-v1` with the unchanged provisional snapshot plus precise source-axis AABB extents. Additive display ABI v1 tessellates the same immutable source under a governed profile and emits a source/analysis-bound native display artifact. Fresh Apple-Silicon cube/prism evidence passes through the real supervisor/application session, configured desktop native-scene retention, and a live source-bound Metal frame; earlier ABI-v3 developer runtimes remain evidenced on Ubuntu x86_64 and Windows x64 | Not a supported product importer; display ABI evidence on Linux/Windows/packages, formal fixture review, broader accuracy/performance corpus, signed distribution, Windows/macOS package integration, and legal review remain open |
| Desktop and storage | GUI-2 supplies the headless application use case; GUI-3 supplies the restrictive shell; GUI-4/GUI-5 provide an actual Apple-Silicon session-only path through real OCCT geometry, revision-bound review, complete inputs, cancellation/recovery, and a deterministic result. USE-1 makes model intake primary; USE-2 adds host-owned SQLite save/reopen for Settings with optimistic revisions and immutable replay. Settings separates rate/pricing review from reusable resource review through two focused work areas. Contract v12 and the developer viewer provide a clickable same-window Model & stock screen; the exact feature/flag configuration carries the selected STEP's validated display derivative into the native indexed renderer while keeping geometry outside the WebView. Four semantic keyboard-operable view controls and a path-free model/stock text equivalent now accompany the native viewport. Live Apple-Silicon inspection verifies pending-state clearing, exact cube analysis, source-bound Metal rendering, complete projected-extents framing, and keyboard Front/Top camera changes. Ubuntu and Windows pass configured native-host smokes; Ubuntu also passes an unsigned extracted Debian package with verified runtime and exact-payload picker/analysis evidence | Governed stock placement, device-loss fallback, and complete screen-reader evidence remain open; models and estimates remain session-only; multi-entry Settings editing, governed standard stock/material sourcing, detailed runtime/adders, signed native runtime, supported installer, interactive Windows acceptance, and three-OS native GUI/package evidence remain open |

Current local evidence includes the full portable workspace, native-host and WASM lint/test gates, 74 Python tooling tests, sixty-nine fixture hashes, ABI-v4 Apple-Silicon native cube/prism measurements, additive display-ABI-v1 cube/prism tessellation, a live source-bound Metal cube, and a verified embedded-runtime packaged STEP smoke. Contract v12 preserves contract v11's path-free viewer workspace and v10's model-sensitive proposal/adoption path while adding one bounded standard-view request. Cube and prism tests prove that different governed geometry changes blank cost and volumetric cutting evidence; VIS-2 carries their bounded source/analysis-bound display artifacts through the real worker, supervisor, decoder, application session, desktop selection binding, and native indexed scene conversion. Live inspection confirms that selecting a new source clears the old scene, the pending inspector stays accurate, the corrected projected-extents fit keeps the complete cube visible, and semantic Front/Top controls change the native camera by mouse or keyboard. The inspector presents reviewed model facts and explicit `Not available` stock values; source mode hides stock until placement is governed. Ordinary catalog editing/activation remains native-identity-unavailable and deny-all. Details and limitations are in [PROJECT_STATE](docs/PROJECT_STATE.md), [TASK-003 validation evidence](docs/06-quality/task-003-validation.md), [VIS-1 renderer evidence](docs/06-quality/vis-1-renderer-validation.md), and [VIS-2 display-scene evidence](docs/06-quality/vis-2-display-scene-validation.md); these results are internal engineering evidence, not production estimating accuracy or release readiness.

## Product boundary

- Organizations enter and govern their own labor, machine, material, outside-service, overhead, risk, and pricing inputs. Production libraries start empty.
- USD is the expected primary currency, while currency remains explicitly typed for replay and validation.
- Missing, stale, ambiguous, or unapproved rates remain unavailable or blocked; they are never converted to zero.
- Accepted estimates remain deterministic and explainable. Recommendations and advanced analysis cannot silently replace a human-approved baseline.
- CAD models, drawings, quotes, rates, and estimate data stay local by default and are not sent to external AI, telemetry, or analytics services.

## Development path

The [usable estimator delivery plan](docs/07-delivery/usable-estimator-plan.md) now defines the current implementation order:

1. Complete USE-1's upload-first workflow guardrail.
2. Finish USE-2 by building the governed category/list/detail Settings editor on the completed contract-v9 native catalog-draft boundary, while retaining identity-unavailable/deny-all activation until a reviewed identity/role-backed allow configuration exists.
3. Finish packaged/visual live-test validation of the USE-3 exact-STEP stock/material/coarse-runtime proposal and adoption path.
4. Add USE-4 coarse process/runtime proposals, then streamline USE-5 around vital unresolved facts and a transparent result.
5. Validate real shop categories, policies, calibration, accessibility, import support, and packaging at USE-6/TASK-007 gates.

TASK-003 containment/native packaging and TASK-004 format evidence continue in parallel. They remain mandatory for support, but do not replace the missing stock, material, and runtime derivation layers.

The nearest testable GUI is a narrow internal, provisional slice. The actual unsigned Apple-Silicon app selects STEP, runs local OCCT, requires explicit review and inputs, and renders the deterministic trace; contract v4 also presents governed STL/3MF facts while keeping mesh estimating unavailable. USE-1 makes upload/analyze primary, and USE-2 persists USD rate/pricing, a starter resource bundle, and bounded catalog evidence behind Settings. Contract v7 reviews catalog contents without host paths; contracts v8/v9 now provide native-owned activation and draft-save seams, but ordinary startup lacks the authenticated actor needed to use either and the WebView exposes neither action. Models and estimates still disappear on exit. It is not yet a low-effort model-derived manufacturing estimator because saved resources are not resolved into stock, material, process, runtime, or operation-cost proposals. GUI-1 formal fixture review remains open, and this is not developer alpha or importer support. See the [usable estimator plan](docs/07-delivery/usable-estimator-plan.md), [GUI evidence plan](docs/07-delivery/gui-vertical-slice-plan.md), [GUI-5 evidence](docs/06-quality/gui-5-validation.md), and [desktop runbook](apps/estimator-desktop/README.md).

## Documentation

Start with:

- [Documentation index](docs/INDEX.md)
- [Current project state](docs/PROJECT_STATE.md)
- [Roadmap](docs/07-delivery/roadmap.md)
- [Initial release plan](docs/07-delivery/release-plan.md)
- [Usable estimator delivery plan](docs/07-delivery/usable-estimator-plan.md)
- [Testable GUI vertical-slice plan](docs/07-delivery/gui-vertical-slice-plan.md)
- [TASK-003 validation evidence](docs/06-quality/task-003-validation.md)
- [GUI-3 validation evidence](docs/06-quality/gui-3-validation.md)
- [GUI-5 configured desktop evidence](docs/06-quality/gui-5-validation.md)
- [Agent rules](AGENTS.md)

## Security posture

CAD models, drawings, quotes, and shop data are treated as potentially sensitive. No project workflow may upload them to telemetry, analytics, AI, or other external services by default. The project does not claim AS9100, CMMC, ITAR, NIST, or export-control compliance.

This source repository is public. Only reviewed project-authored synthetic fixtures or assets with documented public redistribution rights may be committed. Customer/shop models, drawings, estimates, rates, credentials, signing material, controlled technical data, and private validation artifacts must remain outside the repository and public CI.

## Development

The repository is a Rust workspace pinned to Rust 1.94.1. Run the standard local gates with:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
python3 scripts/check_planning.py
python3 scripts/hash_fixtures.py
python3 scripts/tests/test_native_tooling.py
```

Native OCCT commands require an existing checkout of the exact pinned source commit. `scripts/build_occt.py` validates that checkout, records the construction profile, builds/installs it, and calls the native verifier; exact commands and limitations are in [TASK-003 validation evidence](docs/06-quality/task-003-validation.md) and [PROJECT_STATE.md](docs/PROJECT_STATE.md).
