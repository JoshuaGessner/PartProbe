# PartProbe desktop developer shell

> **Status:** GUI-4 developer checkpoint; unsigned, session-only, and not a supported product package.

The desktop composition root is split into three Rust boundaries:

- `crates/desktop-contract` owns the versioned, pathless commands/events and presentation DTOs.
- this directory owns the Leptos 0.8 CSR presentation and design-system foundations;
- `src-tauri` owns the Tauri 2 host, native dialog, retained source path, capability file, and production CSP.

The current shell can choose one local `.step` or `.stp` file and, when an external pinned developer worker is explicitly configured, request provisional geometry facts through `DraftEstimateApplication`. The native host retains the selected path and complete `DraftEstimateSession` only for the running process. The WebView receives a session token, leaf filename, hashes, sanitized stage codes, and provisional measurements; it never receives the path or CAD bytes. After analysis, the workflow requires explicit unit/warning review, every manual estimate value, five confirmed session-only rates, and a confirmed versioned pricing policy before the native application session can return a deterministic selling price and trace. Analysis can be cancelled cooperatively. No values are prefilled as production defaults, and no data, rate approval, or estimate is saved or uploaded.

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

Run the unsigned shell from the repository root after the frontend exists. Without the explicit worker configuration below, source selection works and analysis fails closed as unavailable:

```sh
cargo run -p partprobe-estimator-desktop --features desktop-host --locked
```

To enable the provisional STEP analysis command, first build the external worker against the separately constructed pinned OCCT 8.0.0 root, create a dedicated empty worker workspace, and supply all three paths explicitly:

```sh
PARTPROBE_OCCT_ROOT=/approved/local/occt-install \
  cargo build -p partprobe-geometry-worker --features native-occt --locked
mkdir -p /private/tmp/partprobe-gui-worker
PARTPROBE_GEOMETRY_WORKER="$PWD/target/debug/partprobe-geometry-worker" \
PARTPROBE_GEOMETRY_WORKSPACE=/private/tmp/partprobe-gui-worker \
PARTPROBE_OCCT_ROOT=/approved/local/occt-install \
  cargo run -p partprobe-estimator-desktop --features desktop-host --locked
```

The host validates that the worker executable, workspace, and OCCT `lib` directory exist. Missing configuration does not trigger ambient discovery or in-process parsing; selection remains available and analysis returns a path-free `AnalysisUnavailable` result. The current request limits are 64 MiB input, 1 MiB output, 2,000,000 entities, 30 seconds wall/CPU, 2 GiB worker memory, 1 MiB protocol frames, 10 ms polling, and 250 ms cancellation grace. These are internal developer-profile bounds, not production performance or support commitments.

The estimate form is intentionally empty for shop-owned numeric values. Enter `0` explicitly when zero is the intended value. Its confirmations create only ephemeral developer-session rate governance and pricing context; the displayed result is not saved, approved, or a customer quote. Choosing a different source requests cancellation of an active analysis and waits for bounded worker cleanup before another analysis may start.

For a macOS smoke-test bundle:

```sh
cd apps/estimator-desktop
cargo tauri build --debug --features desktop-host --bundles app --no-sign
open ../../target/debug/bundle/macos/PartProbe.app
```

The `.app` is a disposable local test artifact. It is not signed, notarized, installed, supported, or evidence for Windows/Linux packaging.
