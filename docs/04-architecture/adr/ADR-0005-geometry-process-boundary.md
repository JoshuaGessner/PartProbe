# ADR-0005: Geometry process boundary

## Metadata

- **Status:** In Review
- **Last updated:** 2026-08-01
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

- Random per-job IPC capability or authenticated local channel; schema/version validation; no path strings accepted outside an allow-listed asset service. Asset delivery is either an allowlisted inherited resource or an explicitly named verified-private-copy fallback; an unsupported direct mode fails closed rather than widening inheritance.
- No network, restricted read-only input / write-only job temp directory, low-privilege identity where supported, cleanup and retention controls.
- Hard limits for bytes, ZIP expansion, entities/triangles, CPU, memory, wall time and nested external references; cooperative cancellation then supervised termination.
- Sanitized diagnostics only; no CAD payload, customer data or full local paths in logs. Store stage timing/version/hash evidence.
- Worker crash or timeout becomes a recoverable import result; it cannot approve a snapshot, alter source bytes, or update a routing.

## Approval evidence required

Demonstrate malformed-file termination, quota enforcement, worker crash recovery, IPC fuzz/contract validation, capability/job/correlation mismatch rejection, deleted-source behavior, verified-copy labeling, intentionally inheritable unrelated descriptor/HANDLE exclusion, no-network tests, filesystem escape tests, controlled-data logging review, and packaging/signing on each target OS. The security profile must degrade safely on platforms where a desired sandbox primitive is unavailable.

## Current TASK-003 evidence

The supervisor now uses bounded stdio JSON, a cleared environment, atomically created private per-job directories, control-stream schema version 2, explicit cooperative-cancellation grace, acknowledgement validation, forced kill/reap fallback, and a consumed already-open source grant bound to the request capability. Asset-transport manifest schema version 2 binds the selected transport to job, correlation, capability, authorized length, SHA-256, and exactly one resource ID only for direct transport. `verified_private_copy` remains the default on every target. On Unix, the platform adapter duplicates the granted file close-on-exec and allowlists only that descriptor plus stdio for `exec`. On Windows, the platform-owned launcher creates controlled pipes and null stderr, duplicates the source HANDLE, and passes exactly those four inheritable HANDLEs through `PROC_THREAD_ATTRIBUTE_HANDLE_LIST` with an explicit application path, cleared Unicode environment, and controlled working directory. The worker claims the named descriptor/HANDLE once and immediately prevents descendant inheritance before independently rechecking regular-file type, manifest/request identity, exact length, quota, and hash into immutable bytes. Tests on each platform prove an intentionally inheritable unrelated resource does not survive, required direct creates no copy, malformed resource IDs fail closed, cancellation/forced kill/reap still work, and a removed pathname cannot replace the open grant. OCCT parses verified bytes through `ReadStream`, while the progress callback and forced-termination boundary remain. This evidence does not satisfy durable controlled-store persistence/retention, OS sandbox/no-network, descendant-process, memory/CPU, three-OS native, or packaging approval gates.
