# PartProbe

> **Status:** In Review
> **Last updated:** 2026-09-04
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
| Calculations | Typed money/units, deterministic rules, itemized traces, versioned snapshots/replay, a headless GUI-2 session service, and a GUI-4 desktop workflow that submits explicit model/manual/rate/policy inputs to that service | No durable estimate, shop calibration, saved rate library, or approved quote workflow |
| Rates and pricing | Empty-on-install, user-owned rate cards; effective dating, scope resolution, approval state, pricing and rounding policies; explicit session-only developer inputs in the GUI-5 slice | No guided or durable rate library; PartProbe supplies no production rates |
| Geometry worker | Bounded control schema v2/transport manifest v2, explicit verified-copy transport, exact Unix descriptor and Windows HANDLE direct allowlisting with unrelated-resource exclusion, worker-side identity/type/hash/length/quota verification, private workspaces, cancellation grace/acknowledgement, forced termination, aggregate-output monitoring, Linux parser socket/process syscall denial including a configured native-OCCT smoke, partial CPU/file/process containment, audit/security seams, and governed derivative handoff | No general filesystem sandbox, macOS/Windows parser egress denial, complete cross-platform resource containment, or durable controlled store |
| STEP/OCCT | Optional OCCT 8.0 ABI-v3 spike parses exact verified bytes, emits a source-bound provisional snapshot, measures an OCCT-generated cube and a manually authored analytic prism, and assembles host-verified developer runtimes on evidenced Apple Silicon, Ubuntu x86_64, and Windows x64 configurations | Not a supported product importer; formal fixture review, broader accuracy corpus, signed distribution, Windows/macOS package integration, and legal review remain open |
| Desktop and storage | GUI-2 supplies the headless application use case; GUI-3 supplies the restrictive shell; GUI-4/GUI-5 provide an actual Apple-Silicon session-only path through real OCCT geometry, revision-bound review, complete inputs, cancellation/recovery, and a deterministic USD 702 trace. USE-1 now makes model intake primary and places session rates/pricing behind Settings. Ubuntu and Windows pass configured native-host smokes; Ubuntu also passes an unsigned extracted Debian package with verified runtime and exact-payload picker/analysis evidence | No durable audit/database, save/reopen workflow, model-derived stock/material/runtime proposals, signed native runtime, supported installer, complete accessibility evidence, interactive Windows acceptance, or three-OS native GUI/package evidence exists |

The local workspace passes **189 runtime tests** plus one compile-fail doctest and 74 Python tooling tests. Portable tests cover desktop startup/runtime verification, fixed package-resource selection, aggregate workspace-output containment, Windows runtime/DLL/PE contracts, and exact portal-semantic Linux GUI automation; Linux CI additionally exercises parser-phase socket and descendant denial. Configured native Linux run 33546682993 and Windows run 33546698680 pass real-worker STEP/STL/3MF desktop-host evidence. The host resolves a runtime only from the fixed packaged resource child or an explicit developer override. The 174 MiB USE-1 Apple-Silicon app contains a re-verified 72-artifact OCCT/worker runtime and passes the packaged-resource STEP-to-USD-702 smoke. Details and limitations are in [PROJECT_STATE](docs/PROJECT_STATE.md) and [TASK-003 validation evidence](docs/06-quality/task-003-validation.md); these results are internal engineering evidence, not production estimating accuracy or release readiness.

## Product boundary

- Organizations enter and govern their own labor, machine, material, outside-service, overhead, risk, and pricing inputs. Production libraries start empty.
- USD is the expected primary currency, while currency remains explicitly typed for replay and validation.
- Missing, stale, ambiguous, or unapproved rates remain unavailable or blocked; they are never converted to zero.
- Accepted estimates remain deterministic and explainable. Recommendations and advanced analysis cannot silently replace a human-approved baseline.
- CAD models, drawings, quotes, rates, and estimate data stay local by default and are not sent to external AI, telemetry, or analytics services.

## Development path

The [usable estimator delivery plan](docs/07-delivery/usable-estimator-plan.md) now defines the current implementation order:

1. Complete USE-1's upload-first workflow guardrail.
2. Implement USE-2 durable governed shop Settings and historical replay.
3. Add USE-3 exact-STEP stock/material proposals with model-sensitive fixtures.
4. Add USE-4 coarse process/runtime proposals, then streamline USE-5 around vital unresolved facts and a transparent result.
5. Validate real shop categories, policies, calibration, accessibility, import support, and packaging at USE-6/TASK-007 gates.

TASK-003 containment/native packaging and TASK-004 format evidence continue in parallel. They remain mandatory for support, but do not replace the missing stock, material, and runtime derivation layers.

The nearest testable GUI is a narrow internal, provisional, session-only slice. The actual unsigned Apple-Silicon app selects STEP, runs local OCCT, requires explicit review and inputs, and renders the deterministic trace; contract v4 also presents governed STL/3MF facts while keeping mesh estimating unavailable. USE-1 makes upload/analyze primary, moves rates/pricing behind Settings, and clearly labels the remaining manual manufacturing assumptions. It is not yet a low-effort model-derived manufacturing estimator because stock, material price, process, runtime, and most operation costs are not generated from model plus shop configuration. All work disappears on exit, GUI-1 formal fixture review remains open, and this is not developer alpha or importer support. See the [usable estimator plan](docs/07-delivery/usable-estimator-plan.md), [GUI evidence plan](docs/07-delivery/gui-vertical-slice-plan.md), [GUI-5 evidence](docs/06-quality/gui-5-validation.md), and [desktop runbook](apps/estimator-desktop/README.md).

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
