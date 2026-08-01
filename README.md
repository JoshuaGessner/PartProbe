# PartProbe

> **Status:** In Review
> **Last updated:** 2026-08-01
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
| Calculations | Typed money/units, deterministic calculation rules, itemized traces, versioned snapshots, and replay pass Windows/Linux/macOS CI | No complete estimate workflow, shop calibration, or estimator UI |
| Rates and pricing | Empty-on-install, user-owned rate cards; effective dating, scope resolution, approval state, pricing and rounding policies | No rate-entry UI or durable rate library; PartProbe supplies no production rates |
| Geometry worker | Bounded control schema v2/transport manifest v2, explicit verified-copy transport, exact Unix descriptor and Windows HANDLE direct allowlisting with unrelated-resource exclusion, worker-side identity/type/hash/length/quota verification, private workspaces, cancellation grace/acknowledgement, forced termination, partial CPU/file/process containment, audit/security seams, and governed derivative handoff | No network/filesystem sandbox, complete cross-platform resource containment, or durable controlled store |
| STEP/OCCT | Optional OCCT 8.0 Apple Silicon ABI-v3 spike parses exact verified bytes, emits a source-bound provisional snapshot, has a fail-closed pinned source-build command, and measures both an OCCT-generated cube and a manually authored analytic prism | Not a supported product importer; formal fixture review, broader accuracy corpus, Windows/Linux native construction, packaging, and legal review remain open |
| Desktop and storage | UX, design system, persistence contracts, and release workflow are documented | No desktop application, database, save/reopen workflow, installer, or signed release exists |

The default workspace currently has **102 runtime tests on macOS and 103 on Linux/Windows** plus a compile-fail doctest passing in Checkpoint 19 implementation run 30712929648 at `9479505`. Six native-tooling tests and 26 focused optional-native adapter/worker tests plus strict native-feature Clippy pass locally on Apple Silicon in Checkpoint 20. These results validate foundations and partial containment contracts; they do not establish production estimating accuracy or release readiness.

## Product boundary

- Organizations enter and govern their own labor, machine, material, outside-service, overhead, risk, and pricing inputs. Production libraries start empty.
- USD is the expected primary currency, while currency remains explicitly typed for replay and validation.
- Missing, stale, ambiguous, or unapproved rates remain unavailable or blocked; they are never converted to zero.
- Accepted estimates remain deterministic and explainable. Recommendations and advanced analysis cannot silently replace a human-approved baseline.
- CAD models, drawings, quotes, rates, and estimate data stay local by default and are not sent to external AI, telemetry, or analytics services.

## Development path

The current implementation order is:

1. Review the GUI-1 fixture evidence, start GUI-2 application orchestration, and continue TASK-003 network/filesystem and target-specific resource containment plus Windows/Linux native construction.
2. Complete TASK-004 STL/3MF mesh import comparison.
3. Build TASK-005 desktop UX, including guided shop-owned rate setup and model review.
4. Implement TASK-006 durable SQLite repositories, migrations, backup/restore, and historical replay.
5. Validate real shop categories, policies, and calibration in TASK-007 before making accuracy claims.

The nearest testable GUI is a narrower internal, provisional, session-only STEP slice. GUI-1's native seam is implemented and awaiting fixture review; roughly four focused checkpoints remain for the application use case, secure desktop shell, analysis/estimate workspace, and end-to-end smoke evidence. A 3D viewport adds roughly two or three checkpoints because no tessellation/viewer implementation exists. See the [testable GUI vertical-slice plan](docs/07-delivery/gui-vertical-slice-plan.md). The first usable cross-platform product slice still requires model intake, reviewable measurements, editable assumptions, a guided rate library, transparent estimate/pricing traces, save/reopen, previews, and packaging.

## Documentation

Start with:

- [Documentation index](docs/INDEX.md)
- [Current project state](docs/PROJECT_STATE.md)
- [Roadmap](docs/07-delivery/roadmap.md)
- [Initial release plan](docs/07-delivery/release-plan.md)
- [Testable GUI vertical-slice plan](docs/07-delivery/gui-vertical-slice-plan.md)
- [TASK-003 validation evidence](docs/06-quality/task-003-validation.md)
- [Agent rules](AGENTS.md)

## Security posture

CAD models, drawings, quotes, and shop data are treated as potentially sensitive. No project workflow may upload them to telemetry, analytics, AI, or other external services by default. The project does not claim AS9100, CMMC, ITAR, NIST, or export-control compliance.

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
