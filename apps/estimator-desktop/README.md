# PartProbe desktop developer shell

> **Status:** GUI-5 internal developer checkpoint; unsigned, session-only, and not a supported product package.

The desktop composition root is split into three Rust boundaries:

- `crates/desktop-contract` owns the versioned, pathless commands/events and presentation DTOs.
- this directory owns the Leptos 0.8 CSR presentation and design-system foundations;
- `src-tauri` owns the Tauri 2 host, native dialog, retained source path, capability file, and production CSP.

The current shell can choose one local `.step` or `.stp` file and, when an external pinned developer runtime is explicitly configured and passes host-side verification, request provisional geometry facts through `DraftEstimateApplication`. The native host retains the selected path and complete `DraftEstimateSession` only for the running process. The WebView receives a session token, leaf filename, hashes, sanitized stage codes, and provisional measurements; it never receives the path or CAD bytes. After analysis, the workflow requires explicit unit/warning review, every manual estimate value, five confirmed session-only rates, and a confirmed versioned pricing policy before the native application session can return a deterministic selling price and trace. Analysis can be cancelled cooperatively. No values are prefilled as production defaults, and no data, rate approval, or estimate is saved or uploaded.

## Pinned developer tools

- Rust 1.94.1 from `rust-toolchain.toml`
- `wasm32-unknown-unknown`
- Trunk 0.21.14
- `wasm-bindgen-cli` 0.2.127
- Tauri CLI 2.11.4 for unsigned local bundles

Install the optional developer tools from their Rust sources:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --version 0.21.14 --locked --no-default-features
cargo install wasm-bindgen-cli --version 0.2.127 --locked
cargo install tauri-cli --version 2.11.4 --locked
```

## Verification and launch

From the repository root:

```sh
cargo test -p partprobe-desktop-contract -p partprobe-estimator-desktop-ui --all-targets --locked
cargo test -p partprobe-estimator-desktop --features desktop-host --all-targets --locked
cargo clippy -p partprobe-estimator-desktop-ui --target wasm32-unknown-unknown --locked -- -D warnings
cargo clippy -p partprobe-estimator-desktop --features desktop-host --all-targets --locked -- -D warnings
```

Build the local frontend without network access:

```sh
cd apps/estimator-desktop
trunk build --release --locked true --offline true
```

Run the unsigned shell from the repository root after the frontend exists. Without the explicit runtime/workspace configuration below, source selection works and analysis fails closed as unavailable:

```sh
cargo run -p partprobe-estimator-desktop --features desktop-host --locked
```

To construct the exact reviewed native root from an existing clean official source checkout, run the repository builder. The builder downloads nothing, requires OCCT tag `V8_0_0` at commit `d3056ef80c9668f395da40f5fd7be186cae4501f`, rejects a dirty/wrong source tree, records build/install fingerprints, and runs the native verification suite:

```sh
python3 scripts/build_occt.py \
  --source /approved/local/occt-source \
  --build /private/tmp/partprobe-occt-build \
  --install /private/tmp/partprobe-occt-install
```

To enable the provisional STEP analysis command, build the external worker against that root. Then assemble a separate developer runtime from the worker, the OCCT install, and the construction manifest. The assembler copies the worker plus the complete OCCT runtime closure into one new directory, records only relative launch paths, fingerprints every regular file and safe local symlink, verifies the copied root, and refuses to overwrite an existing output:

```sh
PARTPROBE_OCCT_ROOT=/approved/local/occt-install \
  cargo build -p partprobe-geometry-worker --features native-occt --locked
python3 scripts/assemble_native_runtime.py assemble \
  --occt-root /approved/local/occt-install \
  --worker "$PWD/target/debug/partprobe-geometry-worker" \
  --build-manifest /approved/local/occt-build/partprobe-occt-build-manifest.json \
  --output /approved/local/partprobe-native-runtime
python3 scripts/assemble_native_runtime.py verify \
  --runtime-root /approved/local/partprobe-native-runtime
# Linux only: require every OCCT dynamic link to stay inside the verified runtime.
python3 scripts/verify_native_runtime_links.py \
  --runtime-root /approved/local/partprobe-native-runtime
mkdir -p /private/tmp/partprobe-gui-worker
PARTPROBE_NATIVE_RUNTIME=/approved/local/partprobe-native-runtime \
PARTPROBE_GEOMETRY_WORKSPACE=/private/tmp/partprobe-gui-worker \
  cargo run -p partprobe-estimator-desktop --features desktop-host --locked
```

Cargo feature variants share the same `target/debug/partprobe-geometry-worker` path. A later ordinary feature-off workspace build can intentionally replace the native executable with the unavailable stub. The assembled runtime is the preferred developer path because it retains the verified native executable independently of later Cargo builds and includes transitive OCCT libraries such as `TKDE`, `TKXCAF`, and `TKService`, not only the thirteen libraries linked directly by the worker. The runtime manifest is content-addressed evidence, not a signature or tamper-proof installation. The Python `verify` command is an independent diagnostic, and the desktop host now repeats the bounded manifest, provenance, host, file, hash, symlink, executable, direct-library, and no-extra-artifact checks before deriving either launch path. A feature-off replacement of the Cargo output fails safely with `GUI4-ANALYSIS-WORKER`; it is not evidence that OCCT construction failed.

Run the opt-in GUI-5 host smoke before manual acceptance. It is deliberately ignored by ordinary CI and fails closed unless the runtime root and external workspace are explicit and the runtime verifies:

```sh
PARTPROBE_NATIVE_RUNTIME=/approved/local/partprobe-native-runtime \
PARTPROBE_GEOMETRY_WORKSPACE=/private/tmp/partprobe-gui-worker \
  cargo test -p partprobe-estimator-desktop --features desktop-host \
    gui5_configured_worker_runs_real_step_through_retained_estimate_session \
    --locked -- --ignored --nocapture
```

The host accepts no independent worker or library-root override. It derives both only from the schema-v1 manifest after verifying the exact pinned OCCT source/build/host provenance and every declared artifact. Missing configuration or any verification failure returns the bounded `GUI5-NATIVE-RUNTIME-VERIFY`/`AnalysisUnavailable` state without exposing a path. It does not trigger ambient discovery or in-process parsing. The current request limits are 64 MiB input, 1 MiB output, 2,000,000 entities, 30 seconds wall/CPU, 2 GiB worker memory, 1 MiB protocol frames, 10 ms polling, and 250 ms cancellation grace. These are internal developer-profile bounds, not production performance or support commitments.

The estimate form is intentionally empty for shop-owned numeric values. Enter `0` explicitly when zero is the intended value. Its confirmations create only ephemeral developer-session rate governance and pricing context; the displayed result is not saved, approved, or a customer quote. Choosing a different source requests cancellation of active work and waits for bounded worker cleanup before another analysis may start. A cancelled analysis is shown distinctly from a failure. Canonical-unit and warning confirmations are analysis-revision-bound and reset whenever analysis state changes; manual/rate/policy text may remain in the process only as draft convenience.

For a macOS smoke-test bundle:

```sh
cd apps/estimator-desktop
cargo tauri build --debug --features desktop-host --bundles app --no-sign
PARTPROBE_NATIVE_RUNTIME=/approved/local/partprobe-native-runtime \
PARTPROBE_GEOMETRY_WORKSPACE=/private/tmp/partprobe-gui-worker \
  ../../target/debug/bundle/macos/PartProbe.app/Contents/MacOS/partprobe-estimator-desktop
```

Launching the bundle executable, rather than Finder or `open`, is intentional for this developer checkpoint because the two explicit environment paths must reach the host process. The runtime directory is likewise not embedded in the `.app`, signed, notarized, installed, immutable, or approved for redistribution. Startup verification detects the checked failure classes but does not prevent a privileged actor from changing files after verification. The repository's manual `Native Linux OCCT Evidence` workflow performs exact-source Ubuntu-x86_64 construction, runtime/link verification, and this configured headless host smoke without retaining binaries. The separate `Native Windows OCCT Evidence` workflow pins Windows Server 2022/Visual Studio 2022 x64, canonical OCCT install directories, app-local runtime DLLs, explicit PE/import closure, and the same configured host smoke; run 31605188245 passes strict bridge compilation and all native adapter tests before its process harness exposes a Windows `bin` versus Unix `lib` DLL-directory error, with corrected run 31609710986 active. Neither workflow is a GUI/package test, and all targets still need signed package evidence. The completed Apple-Silicon GUI checklist is recorded in [GUI-5 validation](../../docs/06-quality/gui-5-validation.md), and native-runtime evidence is recorded in [TASK-003 validation](../../docs/06-quality/task-003-validation.md).
