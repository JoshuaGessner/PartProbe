# Security Requirements

> **Status:** In Review  
> **Last updated:** 2026-09-10
> **Related requirements:** SEC-001–SEC-014; REQ-NF-002, REQ-NF-005, REQ-NF-007, REQ-NF-009, REQ-NF-017, REQ-NF-019  
> **Related ADRs:** ADR-0001, ADR-0005, ADR-0006, ADR-0012–ADR-0014  
> **Open questions:** OQ-021–OQ-024, OQ-026, OQ-029  
> **Dependencies:** Deployment-specific threat model and qualified compliance determination  
> **Supersedes:** None

| ID | Requirement |
|---|---|
| SEC-001 | Default capabilities deny network, shell, unrestricted filesystem, remote content, telemetry, and external upload. |
| SEC-002 | Logs/metrics/support bundles shall exclude technical, customer, price, path, and geometry content unless explicitly authorized and previewed. |
| SEC-003 | Authorization shall enforce role, project, classification, record state, and operation at the service/repository boundary. |
| SEC-004 | Untrusted CAD/document parsing shall use bounded intake, validation, least privilege, isolated worker where designed, and sanitized failure output. |
| SEC-005 | Authentication/session/secret/key handling shall follow the selected deployment profile and support revocation/recovery. |
| SEC-006 | Attachments and derivatives shall retain hashes, lineage, classification, access policy, and controlled retention/disposition. |
| SEC-007 | Security/business audit events shall be append-preserving, access controlled, integrity protected, and content-minimized. |
| SEC-008 | Export/print/clipboard/integration actions shall be policy controlled and audited for controlled-data profiles. |
| SEC-009 | Backups, restores, updates, and rollback artifacts shall preserve classification, integrity, least privilege, and authorization. |
| SEC-010 | Dependency/build/release provenance and vulnerability response shall be documented and tested; no security framework or compliance claim is automatic. |
| SEC-011 | PMI, CAM reports/APIs, schedules, inventory, supplier data, and revision derivatives shall be classified, bounded, validated, and prevented from resolving uncontrolled external references. |
| SEC-012 | Correction/learning data shall use least-privilege access, purpose limitation, retention policy, cohort thresholds, and safeguards against exposing individual/customer-sensitive behavior. |
| SEC-013 | Marketplace, CAM, ERP, QMS, scheduling, and vendor integrations shall not transmit controlled files or derived technical data without an approved integration-specific data-flow and authorization policy. |
| SEC-014 | Capacity, backlog, availability, supplier performance, bid scoring, and opportunity-cost data shall be treated as commercially sensitive and excluded from customer reports and diagnostics by default. |

## Current partial evidence

GUI-3 provides bounded configuration/test evidence for SEC-001 and SEC-002; GUI-4 established five analysis/estimate commands plus event listen/unlisten, and contracts v5-v7 add exactly two Settings commands without broadening frontend permissions. Contract v4 added tagged exact-B-rep/mesh response DTOs. No frontend network, shell, filesystem, dialog, opener, updater, upload, or remote-content permission is granted. Raw selected paths, worker authority, cancellation handle, complete controlled mesh provenance, the provisional analysis session, and the SQLite path remain native-owned; the WebView receives only path-free typed state. GUI-4 adds partial SEC-003/004/007 evidence: authorization and an append-only in-memory audit precede source resolution/fingerprinting, the same bounded open grant reaches the worker boundary, runtime/workspace configuration is explicit and fail-closed, cancellation is selection-scoped, selected-extension/content mismatches fail visibly, and estimate evaluation cannot bypass the typed application session or authorize mesh. USE-2 adds a separate headless catalog proposal-activation service with exact actor/profile/settings/catalog/selection/version/operation/correlation policy context, an explicit deny-all baseline, mandatory decision audit before mutation, and no-change denial/audit-failure behavior. Schema v4 implements the catalog audit port with content-minimized exact-version foreign keys, append-only triggers, normalized hash checks, correlation idempotency/collision handling, no-default migration, backup/reopen, and corruption rejection. Native Settings composes the real repository/audit adapters with a versioned policy: ordinary startup denies all, while a controlled exact-profile/exact-operator configuration denies mismatch and permits/audits only the exact pair. No WebView command or actor input was added, and successful decisions never grant estimate authority. The worker supervisor supplies portable poll-bounded aggregate job-output inspection, termination, and cleanup; Linux `x86_64`/`aarch64` adds parser syscall-denial evidence, with configured Ubuntu x86_64 compatibility. Configured Windows evidence lacks that parser-egress boundary. These requirements remain incomplete because no authenticated production identity, role/member repository, shop-reviewed shipped catalog allow configuration, separately access-controlled or externally anchored authorization audit, or general security-event store exists; rate approvals and results remain developer-only, the manifest is unsigned and mutable, and macOS/Windows parser egress denial, general filesystem and hard-storage-quota tests, Linux aarch64 native behavior, navigation interception, signed packages, full diagnostics review, and three-OS window/package evidence remain pending.
