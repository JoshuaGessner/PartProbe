# Geometry-engine architecture

## Metadata

- **Status:** In Review
- **Last updated:** 2026-07-29
- **Related requirement IDs:** REQ-F-021–REQ-F-024, REQ-NF-011–REQ-NF-014, GEO-001–GEO-014, SEC-004
- **Related architecture decision IDs:** ADR-0002, ADR-0004, ADR-0005
- **Open questions:** Target-specific sandbox/resource enforcement? Canonical tolerance defaults? Which exact fixtures establish release tolerances?
- **Dependencies:** OCCT worker spike, mesh parser, persistence, feature-recognition pipeline, desktop model viewer
- **Supersedes / superseded by:** None / none

## Architecture

The geometry engine is a UI-independent application service backed by a sandboxed `geometry-worker`. The desktop process submits a content-addressed asset and job options; it receives a versioned `GeometryAnalysisSnapshot` plus a derived display mesh. The UI never links directly to a geometry kernel, and the kernel never writes quote data.

```text
desktop UI ── application service ── job store / snapshot repository
                                      │
                                      ▼
                             geometry-worker process
 intake → hash → identify → units → parse → heal? → validate → measure
                         → orient → stock → feature job → tessellate
                                      │
                         versioned results / controlled derivative store
```

## Pipeline and contracts

| Stage | Inputs / output | Failure and audit behavior |
|---|---|---|
| Intake and hash | controlled file descriptor/path → SHA-256, byte count, MIME/sniff result | reject unreadable, size-limited and mismatched files; preserve no unchecked external reference |
| Identify / parse | bytes + declared extension → format report / neutral representation | parser errors are recoverable diagnostics; extension is not authority |
| Unit resolution | declared format unit, geometry scale signals, user choice → canonical-mm transform | unknown/ambiguous unit is `needs_user_input`; do not compute approval-grade measures |
| Healing (optional) | imported representation → derivative + action log | no in-place mutation; original and derivative hashes, configuration, topology-change warning |
| Validation | representation → exact-solid or mesh report | distinguish invalid, unknown, and not-applicable; preserve partial preview where safe |
| Basic properties | validated representation + transform → measurements | tag exact vs approximate mesh derivation and tolerance |
| Orient / stock | measurements + candidate axes → ranked envelope candidates | proposals only, include clamp/saw/facing assumptions and alternatives |
| Feature extraction | snapshot ref → `FeatureAnalysisSnapshot` | separate job/version; failure cannot invalidate base measurements |
| Tessellation | geometry ref + visual tolerance → display mesh | visual mesh never replaces analysis geometry |
| Snapshot | all outcomes → immutable record | store stage timing, versions, warnings, safe diagnostics and user decisions |

## Process and API boundary

The first transport implementation uses one bounded JSON request on worker stdin and one bounded JSON response on stdout. The supervisor launches the executable in a controlled working directory with a cleared inherited environment, polls cancellation and wall time, kills and reaps failed workers, and verifies schema, job, and correlation identity before accepting a response. Requests contain only: asset capability, job ID, requested stages, analysis/tessellation options, quotas, and correlation ID. Responses contain status, references to controlled outputs, structured measurements/warnings, stage timing, versions, and safe diagnostic codes. They do not include unrestricted file paths, raw CAD contents, database credentials, or renderer handles.

The current reversible spike supports pre-launch/polled cancellation and hard-kill; cooperative worker acknowledgement and a measured grace interval remain required before native work. The supervisor treats launch, protocol I/O, worker exit, timeout, quota breach, malformed response, or cancellation as sanitized failure. The target design requires no worker network, least-privilege source/output handles, isolated temporary storage, CPU/memory/descendant limits, and derivative cleanup under retention policy. Clearing the environment and controlling the working directory do not satisfy those OS-sandbox requirements.

## Kernel adapter boundary

`geometry-core` owns neutral types, units, measurement/validation rules, and pure algorithms. `geometry-import` owns format sniffing and worker protocol. `geometry-occt-adapter` owns all C++ calls, allocation/lifetime conversion and translation/healing reports. `mesh-analysis` owns mesh validation and approximate properties. No domain crate imports OCCT or a renderer. This keeps a commercial replacement possible and confines unsafe FFI; Rust identifies FFI declarations as unsafe. [Rust `extern`](https://doc.rust-lang.org/std/keyword.extern.html)

## Performance and reliability expectations

Stream or preflight inputs where format permits; apply configured byte, entity/triangle, archive-expansion, recursion, CPU, memory, and wall-clock quotas. Cache outputs by `(source hash, analysis profile, engine version, dependency fingerprint)` only; a different tolerance, kernel version or unit decision is a different result. Capture stage timings and resource high-water marks without geometry content. Benchmark on all target OSes and representative fixtures before selecting defaults.

## Explicit limitations

The engine estimates geometry facts and candidates, not drawing intent or production-safe tool motion. A successful import is not validation of customer requirements. Healing, tessellation, stock envelopes, local thickness and tool access are tolerance- and algorithm-dependent. “No warning” is not a guarantee. See the canonical data semantics in `../02-domain/geometry-analysis-model.md`.
