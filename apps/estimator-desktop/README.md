# PartProbe desktop developer shell

> **Status:** GUI-3 developer checkpoint; unsigned, session-only, and not a supported product package.

The desktop composition root is split into three Rust boundaries:

- `crates/desktop-contract` owns the versioned, pathless commands/events and presentation DTOs.
- this directory owns the Leptos 0.8 CSR presentation and design-system foundations;
- `src-tauri` owns the Tauri 2 host, native dialog, retained source path, capability file, and production CSP.

The current shell can choose one local `.step` or `.stp` file. It does not read, analyze, estimate, save, or upload that model. The native host retains the path only for the running process and returns a session token plus the leaf filename to the webview. GUI-4 will adapt that retained selection to `DraftEstimateApplication`; it must not move geometry or estimate authority into the UI.

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
cargo test -p partprobe-desktop-contract -p partprobe-estimator-desktop-ui -p partprobe-estimator-desktop --locked
cargo clippy -p partprobe-estimator-desktop-ui --target wasm32-unknown-unknown --locked -- -D warnings
cargo clippy -p partprobe-estimator-desktop --features desktop-host --all-targets --locked -- -D warnings
```

Build the local frontend without network access:

```sh
cd apps/estimator-desktop
trunk build --release --locked true --offline true
```

Run the unsigned shell from the repository root after the frontend exists:

```sh
cargo run -p partprobe-estimator-desktop --features desktop-host --locked
```

For a macOS smoke-test bundle:

```sh
cd apps/estimator-desktop
cargo tauri build --debug --features desktop-host --bundles app --no-sign
open ../../target/debug/bundle/macos/PartProbe.app
```

The `.app` is a disposable local test artifact. It is not signed, notarized, installed, supported, or evidence for Windows/Linux packaging.
