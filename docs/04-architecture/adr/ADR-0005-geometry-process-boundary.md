# ADR-0005: Geometry process boundary

## Metadata

- **Status:** In Review
- **Last updated:** 2026-08-11
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

The supervisor now uses bounded stdio JSON, a cleared environment, atomically created private per-job directories, control-stream schema version 2, explicit cooperative-cancellation grace, acknowledgement validation, forced kill/reap fallback, a mandatory host resource profile, and a consumed already-open source grant bound to the request capability. Asset-transport manifest schema version 2 binds the selected transport to job, correlation, capability, authorized length, SHA-256, and exactly one resource ID only for direct transport. `verified_private_copy` remains the default on every target. On Unix, the platform adapter duplicates the granted file close-on-exec, allowlists only that descriptor plus stdio for `exec`, installs hard CPU/regular-file/core limits, creates a process group, and kills that group on forced termination; Linux also installs a hard address-space limit. On Windows, the platform-owned launcher uses the same exact HANDLE list for copy and direct modes, creates the worker suspended, assigns it to a Job with CPU/committed-memory/one-process/kill-on-close limits, and resumes only after successful assignment. The worker claims the named descriptor/HANDLE once and immediately prevents descendant inheritance before independently rechecking regular-file type, manifest/request identity, exact length, quota, and hash into immutable bytes.

Portable supervisor polling now also verifies that a staged copy remains the same length and read-only, rejects links/special/nested entries, caps the private workspace at 64 entries, and sums every non-input regular file against the request's output-byte quota. It repeats this check after worker exit, terminates a live violator, returns a sanitized code, and recursively deletes the owned job directory. A hostile fixture proves two individually sub-limit files cannot evade the aggregate budget. On Linux `x86_64`/`aarch64`, the worker also sets `no_new_privs` before its cancellation thread and, after the authorized asset is immutable, applies a seccomp filter with `TSYNC` that denies socket, `io_uring`, process/thread-creation, and exec syscalls. Failure maps to sanitized `WORKER_CONTAINMENT_FAILED`; Linux fixtures separately prove socket and descendant creation receive `EPERM`. Run 31555774851 additionally proves the configured Ubuntu x86_64 native OCCT worker completes the governed STEP-to-estimate host smoke under this filter. Tests therefore prove exact inheritance, CPU/file limits, Linux/Windows memory limits, Windows one-process containment, normal non-Linux descendant cleanup, Linux parser-phase descendant denial, cancellation/forced kill/reap, removed-path grant authority, and poll-bounded aggregate workspace output. Current macOS rejects the available memory `setrlimit` resources; hard macOS memory, macOS parser egress/hostile-descendant containment, Windows parser egress denial, OS-enforced filesystem/storage quotas and general filesystem sandboxing, durable controlled-store persistence/retention, three-OS native packaging, and packaging approval remain open.
