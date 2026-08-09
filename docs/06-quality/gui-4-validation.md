# GUI-4 Analysis and Estimate Workspace Validation

> **Status:** In Progress
> **Last updated:** 2026-08-09
> **Related requirements:** REQ-F-002–REQ-F-010, REQ-F-032; REQ-NF-004, REQ-NF-005, REQ-NF-017; SEC-003, SEC-004; TEST-003, TEST-011, TEST-014, TEST-021, TEST-030
> **Related ADRs:** ADR-0001, ADR-0002, ADR-0005, ADR-0007, ADR-0008
> **Open questions:** Native worker configuration/packaging, desktop cancellation ownership, developer-session authorization policy, complete safe workspace DTO
> **Dependencies:** GUI-1 evidence, GUI-2 application service, GUI-3 shell
> **Supersedes:** None

## Current implemented slice

GUI-4 is not complete. Its first blocking application seam is implemented: `DraftGeometryRequestTemplate` carries validated path-free job, correlation, capability, stages, profile, and quota intent without falsely claiming a source hash. `DraftEstimateApplication::start_session_from_unfingerprinted_source` authorizes and audits the selected relative source, obtains one already-open grant, fingerprints its bytes within the request's maximum-input quota, constructs the source-bound worker request, and passes the rewound same grant to the configured geometry port.

`AssetReadGrant::fingerprint_sha256` checks the captured file length, enforces the nonzero maximum input size, computes typed SHA-256, and rewinds the open handle before return. This fingerprint is not accepted as worker truth: the existing supervisor/worker transport independently rechecks regular-file identity, length, quota, and hash before parsing.

## Focused evidence

- A bounded synthetic source fingerprints to its known SHA-256 value, and reading the grant immediately afterward returns the complete original bytes from offset zero.
- An allowed application request appends one authorization audit event, produces the expected path-free request digest at the analysis port, and returns a provisional session that remains `Unavailable` before review/manual/rate/policy input.
- A denied request names a nonexistent relative source yet returns the policy failure, records one decision event, and performs no analysis. This proves policy/audit precede filesystem resolution and fingerprinting.

Focused command:

```sh
cargo test -p partprobe-geometry-import -p partprobe-application --locked
```

Repository-wide formatting, warnings-as-errors Clippy, 122 local macOS runtime tests, one compile-fail doctest, six native-tooling tests, 141-document planning validation, fixture hashes, and diff checks pass. Consolidated-main run 31327747010 predates this GUI-4 change and passes macOS/Ubuntu; its Windows CPU fixture reached the former 5-second wall deadline before accumulating the 100 ms user-CPU ceiling. The approved test-only fix increases that fixture deadline to 30 seconds without changing the hard CPU limit, production behavior, or expected `WORKER_EXIT`; replacement evidence is pending.

## Remaining GUI-4 exit work

The Tauri runtime still exposes no analysis command and retains no `DraftEstimateSession`. It needs an explicit developer-session authorization/audit adapter, supervised-worker/native-library configuration, async execution and cancellation, safe geometry/review/input/rate/result DTOs, Leptos forms, and deterministic result rendering. No GUI can yet analyze the selected STEP model or produce an estimate. No calculation behavior, geometry interpretation, worker protocol/schema, native ABI, persisted/customer schema, production rate, dependency, or support claim changed in this slice.
