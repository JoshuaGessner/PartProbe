# PartProbe

> **Status:** In Review
> **Last updated:** 2026-08-12
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
| Rates and pricing | Empty-on-install, user-owned rate cards; effective dating, scope resolution, approval state, pricing and rounding policies | No rate-entry UI or durable rate library; PartProbe supplies no production rates |
| Geometry worker | Bounded control schema v2/transport manifest v2, explicit verified-copy transport, exact Unix descriptor and Windows HANDLE direct allowlisting with unrelated-resource exclusion, worker-side identity/type/hash/length/quota verification, private workspaces, cancellation grace/acknowledgement, forced termination, aggregate-output monitoring, Linux parser socket/process syscall denial including a configured native-OCCT smoke, partial CPU/file/process containment, audit/security seams, and governed derivative handoff | No general filesystem sandbox, macOS/Windows parser egress denial, complete cross-platform resource containment, or durable controlled store |
| STEP/OCCT | Optional OCCT 8.0 ABI-v3 spike parses exact verified bytes, emits a source-bound provisional snapshot, measures an OCCT-generated cube and a manually authored analytic prism, and assembles host-verified developer runtimes on evidenced Apple Silicon, Ubuntu x86_64, and Windows x64 configurations | Not a supported product importer; formal fixture review, broader accuracy corpus, signed distribution, native-runtime package integration, and legal review remain open |
| Desktop and storage | GUI-2 supplies the headless application use case; GUI-3 supplies the restrictive shell; GUI-4/GUI-5 provide an actual Apple-Silicon session-only path through real OCCT geometry, revision-bound review, complete inputs, cancellation/recovery, and a deterministic USD 702 trace; Ubuntu and Windows pass configured headless native-host smokes, and Ubuntu separately passes an unsigned Debian-contained shell-process smoke | No durable audit/database, save/reopen workflow, embedded/signed native runtime, supported installer, interactive Linux/Windows acceptance, full accessibility matrix, or three-OS native GUI/package evidence exists |

The local macOS default workspace passes **144 runtime tests** plus one compile-fail doctest and 26 native-tooling tests. Portable tests cover desktop startup verification, fixed package-resource runtime selection, aggregate workspace-output containment, and the fail-closed Windows runtime/DLL/PE contract; Linux CI additionally exercises parser-phase socket and descendant denial. The focused contract/native-host/UI set passes 31 tests plus two opt-in real-worker smokes ignored by default. [Native Linux run 31555774851](https://github.com/JoshuaGessner/PartProbe/actions/runs/31555774851) constructs exact OCCT 8.0.0 on Ubuntu 24.04 x86_64, verifies the 24-binary/23-OCCT-family dynamic-link closure, and passes the configured desktop-host STEP-to-USD-702 smoke under seccomp. [Native Windows run 31613259785](https://github.com/JoshuaGessner/PartProbe/actions/runs/31613259785) passes all 11 adapter and 16 worker/process tests, assembles/re-verifies the app-local runtime, proves 23 OCCT DLL families across 24 x64 PE binaries remain inside it, and passes the same configured STEP-to-USD-702 host smoke. This is Windows Server 2022 headless developer evidence—not a windowed app, bundled MSVC runtime, package, signing, importer-support, or release claim. The host can now resolve a runtime only from the fixed packaged resource child or an explicit developer override; the first combined Linux native-package evidence run remains pending. The rebuilt unsigned Apple-Silicon app retains its full native-picker flow, cancellation, malformed-input recovery, and review-reset evidence. Details and limits are in [TASK-003 validation evidence](docs/06-quality/task-003-validation.md); these results are internal engineering evidence, not production estimating accuracy or release readiness.

## Product boundary

- Organizations enter and govern their own labor, machine, material, outside-service, overhead, risk, and pricing inputs. Production libraries start empty.
- USD is the expected primary currency, while currency remains explicitly typed for replay and validation.
- Missing, stale, ambiguous, or unapproved rates remain unavailable or blocked; they are never converted to zero.
- Accepted estimates remain deterministic and explainable. Recommendations and advanced analysis cannot silently replace a human-approved baseline.
- CAD models, drawings, quotes, rates, and estimate data stay local by default and are not sent to external AI, telemetry, or analytics services.

## Development path

The current implementation order is:

1. Review the GUI-1 fixture evidence; integrate the verified native runtimes with platform packages, add interactive Linux/Windows dialog/accessibility evidence, dynamic-link/license/SBOM inspection, signed application integration, macOS/Windows parser egress denial, general filesystem confinement, and remaining TASK-003 resource containment.
2. Complete TASK-004 STL/3MF mesh import comparison.
3. Build TASK-005 desktop UX, including guided shop-owned rate setup and model review.
4. Implement TASK-006 durable SQLite repositories, migrations, backup/restore, and historical replay.
5. Validate real shop categories, policies, and calibration in TASK-007 before making accuracy claims.

The nearest testable GUI now exists as a narrow internal, provisional, session-only STEP slice. GUI-2 through GUI-5 have passed their bounded checkpoints on the evidenced Apple-Silicon configuration: the actual unsigned app can select the governed STEP prism, analyze it through freshly constructed OCCT 8.0.0, require explicit inputs, and show a deterministic USD 702 trace; cancellation and malformed-input recovery also work. Running it still requires separate OCCT/runtime construction and two explicit environment paths; the native host now rejects changed, incomplete, unmanifested, or wrong-host runtime contents before launch. All work disappears on exit, GUI-1 formal fixture review remains open, and this is not developer alpha or importer support. A 3D viewport adds roughly two or three checkpoints because no tessellation/viewer implementation exists. See the [testable GUI vertical-slice plan](docs/07-delivery/gui-vertical-slice-plan.md), [GUI-5 evidence](docs/06-quality/gui-5-validation.md), and [desktop developer runbook](apps/estimator-desktop/README.md). The first usable cross-platform product slice still requires broader model intake, reviewable measurements, guided durable shop rates, save/reopen, native packaging, and release evidence.

## Documentation

Start with:

- [Documentation index](docs/INDEX.md)
- [Current project state](docs/PROJECT_STATE.md)
- [Roadmap](docs/07-delivery/roadmap.md)
- [Initial release plan](docs/07-delivery/release-plan.md)
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
