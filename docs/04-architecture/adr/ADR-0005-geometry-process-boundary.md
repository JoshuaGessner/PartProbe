# ADR-0005: Geometry process boundary

## Metadata

- **Status:** In Review
- **Last updated:** 2026-07-29
- **Related requirement IDs:** REQ-NF-011–REQ-NF-014, GEO-001–GEO-014, SEC-001–SEC-004
- **Related architecture decision IDs:** ADR-0002, ADR-0003, ADR-0004
- **Open questions:** Sandbox mechanisms per target OS, resource quota defaults, local IPC authentication and update strategy
- **Dependencies:** Desktop supervisor, geometry-worker prototype, controlled-data deployment policy
- **Supersedes / superseded by:** None / none

## Proposed decision

Run parsing, native kernel translation/healing, geometry analysis and tessellation in a **separate local geometry-worker process**, supervised by the Rust application. Communicate through a versioned local IPC contract and controlled asset/output stores. Do not allow the worker to query quote data or network services.

## Rationale

CAD input is untrusted and native kernels require FFI; Rust describes foreign calls/interfaces as inherently unsafe. [Rust `extern`](https://doc.rust-lang.org/std/keyword.extern.html) A separate process contains crashes, leaks and parser compromise better than in-process adapters, permits hard resource limits, and makes native dependency replacement practical. The cost is IPC, lifecycle complexity, target-specific sandbox work, and another signed executable.

## Required controls

- Random per-job IPC capability or authenticated local channel; schema/version validation; no path strings accepted outside an allow-listed asset service.
- No network, restricted read-only input / write-only job temp directory, low-privilege identity where supported, cleanup and retention controls.
- Hard limits for bytes, ZIP expansion, entities/triangles, CPU, memory, wall time and nested external references; cooperative cancellation then supervised termination.
- Sanitized diagnostics only; no CAD payload, customer data or full local paths in logs. Store stage timing/version/hash evidence.
- Worker crash or timeout becomes a recoverable import result; it cannot approve a snapshot, alter source bytes, or update a routing.

## Approval evidence required

Demonstrate malformed-file termination, quota enforcement, worker crash recovery, IPC fuzz/contract validation, no-network tests, filesystem escape tests, controlled-data logging review, and packaging/signing on each target OS. The security profile must degrade safely on platforms where a desired sandbox primitive is unavailable.

## Current TASK-003 evidence

The supervisor now uses bounded stdio JSON, a cleared environment, controlled working directory, timeout/kill/reap behavior, fixed-name source staging, input byte and SHA-256 checks, read-only staged bytes, and cleanup. The optional OCCT worker accepts only the fixed staged name and emits a bounded fixed-name provisional snapshot plus an opaque response reference. Hash mismatch is rejected before launch, and the analytic STEP cube completes through the real subprocess boundary. This evidence narrows the design but does not satisfy OS sandbox, no-network, descriptor-based grant, descendant-process, memory/CPU, or packaging approval gates.
