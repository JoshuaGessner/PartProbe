# Dependency Record

> **Status:** In Review  
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-NF-003, REQ-NF-010; TEST-001, TEST-020  
> **Related ADRs:** ADR-0007  
> **Open questions:** Organization-wide license approval and automated advisory/SBOM tooling  
> **Dependencies:** `Cargo.toml`, `Cargo.lock`, CI  
> **Supersedes:** None

This record covers dependencies introduced by TASK-001 and reused by TASK-002/003. Exact versions are pinned in the workspace manifest and lockfile. Presence here records engineering review; it is not legal advice or a final organization-wide license approval. TASK-003 adds `cc` for its optional native bridge, `sha2` for controlled streaming source verification, a Unix-only direct use of the already-locked `libc`, and the Bytecode Alliance capability-filesystem pair `cap-std`/`cap-fs-ext` for race-safe parent containment plus final-component no-follow behavior. OCCT 8.0.0 remains an unapproved, local-only spike dependency and is not fetched, linked, or distributed by default.

## Direct dependencies

| Package | Exact version/features | Purpose | Alternatives considered | License/source | Maintenance and removal |
|---|---|---|---|---|---|
| `rust_decimal` | 1.42.1; `default-features = false`; `std`, `serde-with-str` | Authoritative fixed-precision money and canonical decimal-as-string serialization | bespoke scaled integer; another decimal crate | MIT; crates.io/upstream GitHub | Current release retrieved 2026-07-23; owner: calculation maintainers; replace behind `Money` and quantity wrappers if precision/overflow/replay gates fail |
| `serde` | 1.0.229; `derive` | Versioned DTO/snapshot serialization | handwritten encoders | MIT OR Apache-2.0; crates.io/upstream GitHub | Mature ecosystem dependency; owner: architecture maintainers; removal requires replacing derives on persisted contracts |
| `serde_json` | 1.0.151; defaults | Compact canonical snapshots, fixture expectations, and bounded internal worker protocol JSON for reversible spikes | custom canonical encoder; another canonical binary/text format | MIT OR Apache-2.0; crates.io/upstream GitHub | Owners: calculation and geometry maintainers; replace behind snapshot and worker-protocol boundaries before formats become external contracts |
| `cc` | 1.4.0; default features disabled; build dependency only | Invoke the platform C++17 compiler for the project-owned optional OCCT C ABI shim | handwritten cross-platform compiler discovery; CMake-driving crate; `cxx` bridge | MIT OR Apache-2.0; crates.io/rust-lang upstream | Owner: geometry maintainers; mature Rust project; removal replaces `build.rs` compilation without changing the C ABI |
| `sha2` | 0.11.0; default features disabled | Streaming SHA-256 verification of controlled source bytes before worker launch | platform hash commands; bespoke SHA-256; OS crypto APIs | MIT OR Apache-2.0; crates.io/RustCrypto upstream | Owner: security and geometry maintainers; pure-Rust portable API; removal occurs behind asset staging without changing stored digest semantics |
| `libc` | 0.2.189; default features disabled; direct dependency on Unix targets only | Supply the platform `O_NOFOLLOW` constant used to reject a linked final source-path component while opening read-only | hard-coded OS constants; a larger filesystem crate; a race-prone metadata precheck | MIT OR Apache-2.0; crates.io/rust-lang upstream | Owner: security and geometry maintainers; already active transitively through `sha2`/`cpufeatures`, so this adds direct maintenance responsibility but no new package; remove behind the local-source opener if the standard library exposes an equivalent portable option |
| `cap-std` | 4.0.2; default features disabled | Hold an open directory capability and resolve only relative asset paths while preventing traversal or symlink escape from that root | `canonicalize`/metadata prechecks; custom Unix `openat2` plus Windows handle traversal; pre-copy validation | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT; crates.io/Bytecode Alliance upstream | Owner: security and geometry maintainers; maintained cross-platform capability API; remove behind `LocalAssetRoot` if the standard library or a future platform service provides equivalent race-safe containment |
| `cap-fs-ext` | 4.0.2; default features disabled; `std` only | Add final-component no-follow control to the capability-root open operation | accept a contained final symlink; race-prone metadata precheck; project-owned unsafe OS calls | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT; crates.io/Bytecode Alliance upstream | Owner: security and geometry maintainers; version-aligned with `cap-std`; remove with the resolver or if `cap-std` gains equivalent final-link control |

## Active dependency graph

`cargo tree --workspace --all-targets --edges normal,build` on 2026-07-29 showed:

- `rust_decimal` → `arrayvec 0.7.8`, `num-traits 0.2.19` → build dependency `autocfg 1.5.1`.
- `serde` → `serde_core 1.0.229`, `serde_derive 1.0.229` → proc-macro chain `proc-macro2 1.0.107`, `quote 1.0.47`, `syn 3.0.3`, `unicode-ident 1.0.24`.
- `serde_json` → `itoa 1.0.18`, `memchr 2.8.3`, `zmij 1.0.23`, plus Serde.
- `cc 1.4.0` → `find-msvc-tools 0.1.9` and `shlex 2.0.1`; all report MIT OR Apache-2.0. These execute only during builds, discover the platform compiler, and parse compiler flags.
- `sha2 0.11.0` → `cfg-if 1.0.4`, `cpufeatures 0.3.0` → `libc 0.2.189`, and `digest 0.11.3` → `block-buffer 0.12.1` / `crypto-common 0.2.2` → `hybrid-array 0.4.13` → `typenum 1.20.1`. `geometry-import` also references `libc 0.2.189` directly on Unix targets. These report MIT OR Apache-2.0-compatible expressions and add no intended network behavior.
- `cap-std 4.0.2` and `cap-fs-ext 4.0.2` share `cap-primitives 4.0.2`, `ambient-authority 0.0.2`, `fs-set-times 0.20.3`, `io-extras 0.19.0`, `io-lifetimes 2.0.4/3.0.1`, `ipnet 2.12.0`, `maybe-owned 0.3.4`, and target-specific `rustix 1.1.4`, `rustix-linux-procfs 0.1.1`, `winx 0.36.4`, and `windows-sys 0.59.0/0.60.2/0.61.2` families. Their supporting target graph adds `bitflags 2.13.1`, `errno 0.3.14`, `linux-raw-sys 0.12.1`, and version-matched Windows target crates. All report MIT, Apache-2.0, or the Apache-2.0-with-LLVM-exception alternatives recorded above.

Reported package licenses are MIT, MIT OR Apache-2.0, Apache-2.0 OR MIT, `memchr`'s Unlicense OR MIT, and `unicode-ident`'s combined MIT/Apache-2.0 and Unicode-3.0 expression. The Cargo 1.94 lockfile also records packages reachable through disabled optional features; the active graph above—not every lockfile entry—is the TASK-001 build surface.

## Security, build, and runtime review

- Project crates set `unsafe_code = "forbid"`. This does not prove that dependencies contain no unsafe code; an automated unsafe/advisory review remains a release gate.
- The active graph is Rust-only and showed no native linker dependency. It executes Serde derive procedural macros and the `autocfg` build dependency during builds.
- The selected libraries perform no intended runtime network access. Cargo requires registry/network access only to fetch uncached packages; `Cargo.lock` and `--locked` make resolution reproducible.
- TASK-003 moves the already-reviewed Serde JSON package into the `geometry-import` runtime graph for bounded local protocol encoding/decoding; this changes its runtime purpose but adds no package or network behavior.
- The optional native bridge compiles only when `native-occt` is selected, requires an explicit local `PARTPROBE_OCCT_ROOT`, dynamically links shared OCCT libraries, and never downloads native code from `build.rs`.
- SHA-256 is computed incrementally while copying into a create-new worker-local file. Default `sha2` allocation/OID features are disabled; CPU feature detection may select a supported backend and falls back to portable software.
- Local source opening is read-only and rejects a linked final component with `O_NOFOLLOW` on Unix or `FILE_FLAG_OPEN_REPARSE_POINT` plus a reparse-point attribute check on Windows. The Windows open also requests identification-only security quality of service. This uses safe standard-library APIs and no network access or project `unsafe`; parent-component authorization and containment remain application-service work.
- `LocalAssetRoot` deliberately opts into ambient authority only while opening an application-selected root, then performs relative capability-based lookups. Syntactic traversal and absolute paths are rejected before resolution; `cap-std` prevents parent symlinks from escaping the open root; `cap-fs-ext` rejects a linked final component. A root is not an authorization decision: actor, project, classification, record-state, audit, and root-selection policy remain application/security-service responsibilities.
- Standard-library `canonicalize` or `symlink_metadata` checks were rejected because a path component can change between validation and open. A custom cross-platform resolver was rejected because it would duplicate maintained Unix/Windows handle traversal and require project-owned `unsafe`, which workspace policy forbids.
- The capability crates and their OS abstraction dependencies use platform-specific code, including upstream-reviewed unsafe where required to call operating-system APIs; the workspace's `unsafe_code = "forbid"` still applies to PartProbe crates. No runtime network behavior is intended. Automated advisory, source-integrity, and unsafe-code review remain release gates.
- Default features are disabled for `rust_decimal` to reduce optional surface. Decimal overflow, scale, serialization, and cross-platform behavior are exercised by TEST-001 but still require representative golden estimates in TASK-002.
- CI pins the Rust toolchain and the checkout action commit. A future dependency gate must add automated advisory, SBOM, source-integrity, and full transitive-license evidence before release.
- TASK-003 may not add or distribute OCCT until an exact source/build/version, native transitive graph, LGPL-2.1-with-additional-exception obligations, notices/relinking approach, advisories, reproducible three-OS packaging, update owner, and adapter removal/replacement path are recorded and reviewed.

## Native candidate evidence — not distribution approval

| Candidate | Exact source/build | Local evidence | Remaining gate |
|---|---|---|---|
| Open CASCADE Technology | 8.0.0; official tag `V8_0_0`; commit `d3056ef80c9668f395da40f5fd7be186cae4501f`; C++17 shared Release libraries; PCH/TBB/FreeType off; requested `TKDESTEP`, `TKShHealing`, `TKMesh` | Apple Silicon source configure/build/install passes; optional C ABI link and sanitized missing-file failure pass | Legal approval; cross-platform resolved inventory and license hashes; three-OS reproducible build/package fingerprints; notices/source/relink instructions; advisories/SBOM; signed artifacts |

The Apple Silicon build resolved 23 OCCT shared libraries: `TKernel`, `TKMath`, `TKG2d`, `TKG3d`, `TKGeomBase`, `TKBRep`, `TKGeomAlgo`, `TKTopAlgo`, `TKPrim`, `TKBO`, `TKShHealing`, `TKMesh`, `TKHLR`, `TKService`, `TKCDF`, `TKLCAF`, `TKCAF`, `TKV3d`, `TKVCAF`, `TKXCAF`, `TKDE`, `TKXSBase`, and `TKDESTEP`. This is observed spike evidence, not yet a cross-platform package manifest; each target artifact still needs a generated fingerprint and dynamic-link audit.

## Verification commands

```sh
cargo tree --workspace --all-targets --edges normal,build
cargo metadata --format-version 1 --locked
cargo test --workspace --all-targets --locked
```
