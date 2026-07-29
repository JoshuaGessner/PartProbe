# Dependency Record

> **Status:** In Review  
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-NF-003, REQ-NF-010; TEST-001, TEST-020  
> **Related ADRs:** ADR-0007  
> **Open questions:** Organization-wide license approval and automated advisory/SBOM tooling  
> **Dependencies:** `Cargo.toml`, `Cargo.lock`, CI  
> **Supersedes:** None

This record covers dependencies introduced by TASK-001 and reused by TASK-002/003. Exact versions are pinned in the workspace manifest and lockfile. Presence here records engineering review; it is not legal advice or a final organization-wide license approval. TASK-003's `geometry-core`, `geometry-import`, fixture-contract, and worker-process slices add no external dependency; OCCT remains an unapproved candidate.

## Direct dependencies

| Package | Exact version/features | Purpose | Alternatives considered | License/source | Maintenance and removal |
|---|---|---|---|---|---|
| `rust_decimal` | 1.42.1; `default-features = false`; `std`, `serde-with-str` | Authoritative fixed-precision money and canonical decimal-as-string serialization | bespoke scaled integer; another decimal crate | MIT; crates.io/upstream GitHub | Current release retrieved 2026-07-23; owner: calculation maintainers; replace behind `Money` and quantity wrappers if precision/overflow/replay gates fail |
| `serde` | 1.0.229; `derive` | Versioned DTO/snapshot serialization | handwritten encoders | MIT OR Apache-2.0; crates.io/upstream GitHub | Mature ecosystem dependency; owner: architecture maintainers; removal requires replacing derives on persisted contracts |
| `serde_json` | 1.0.151; defaults | Compact canonical snapshots, fixture expectations, and bounded internal worker protocol JSON for reversible spikes | custom canonical encoder; another canonical binary/text format | MIT OR Apache-2.0; crates.io/upstream GitHub | Owners: calculation and geometry maintainers; replace behind snapshot and worker-protocol boundaries before formats become external contracts |

## Active transitive build graph

`cargo tree --workspace --all-targets --edges normal,build` on 2026-07-23 showed:

- `rust_decimal` → `arrayvec 0.7.8`, `num-traits 0.2.19` → build dependency `autocfg 1.5.1`.
- `serde` → `serde_core 1.0.229`, `serde_derive 1.0.229` → proc-macro chain `proc-macro2 1.0.107`, `quote 1.0.47`, `syn 3.0.3`, `unicode-ident 1.0.24`.
- `serde_json` → `itoa 1.0.18`, `memchr 2.8.3`, `zmij 1.0.23`, plus Serde.

Reported package licenses are MIT, MIT OR Apache-2.0, Apache-2.0 OR MIT, `memchr`'s Unlicense OR MIT, and `unicode-ident`'s combined MIT/Apache-2.0 and Unicode-3.0 expression. The Cargo 1.94 lockfile also records packages reachable through disabled optional features; the active graph above—not every lockfile entry—is the TASK-001 build surface.

## Security, build, and runtime review

- Project crates set `unsafe_code = "forbid"`. This does not prove that dependencies contain no unsafe code; an automated unsafe/advisory review remains a release gate.
- The active graph is Rust-only and showed no native linker dependency. It executes Serde derive procedural macros and the `autocfg` build dependency during builds.
- The selected libraries perform no intended runtime network access. Cargo requires registry/network access only to fetch uncached packages; `Cargo.lock` and `--locked` make resolution reproducible.
- TASK-003 moves the already-reviewed Serde JSON package into the `geometry-import` runtime graph for bounded local protocol encoding/decoding; this changes its runtime purpose but adds no package or network behavior.
- Default features are disabled for `rust_decimal` to reduce optional surface. Decimal overflow, scale, serialization, and cross-platform behavior are exercised by TEST-001 but still require representative golden estimates in TASK-002.
- CI pins the Rust toolchain and the checkout action commit. A future dependency gate must add automated advisory, SBOM, source-integrity, and full transitive-license evidence before release.
- TASK-003 may not add or distribute OCCT until an exact source/build/version, native transitive graph, LGPL-2.1-with-additional-exception obligations, notices/relinking approach, advisories, reproducible three-OS packaging, update owner, and adapter removal/replacement path are recorded and reviewed.

## Verification commands

```sh
cargo tree --workspace --all-targets --edges normal,build
cargo metadata --format-version 1 --locked
cargo test --workspace --all-targets --locked
```
