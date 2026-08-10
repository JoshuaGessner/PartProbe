# Security Requirements

> **Status:** In Review  
> **Last updated:** 2026-08-09
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

GUI-3 provides bounded configuration/test evidence for SEC-001 and SEC-002; GUI-4 extends the exact application manifest/capability to five commands (`desktop_contract`, `select_model_source`, token-only `analyze_model_source`, token-bound `cancel_model_analysis`, and retained-session `evaluate_draft_estimate`) plus event listen/unlisten. No frontend network, shell, filesystem, dialog, opener, updater, upload, or remote-content permission is granted. Raw selected paths, worker authority, cancellation handle, and the complete provisional analysis session remain in native process state; the webview receives only a leaf display name, opaque selection/session/analysis IDs, hashes, sanitized stage/warning codes, provisional measurements, explicit estimate inputs, and path-free result traces. GUI-4 adds partial SEC-003/004/007 evidence: authorization and an append-only in-memory audit precede source resolution/fingerprinting, the same bounded open grant reaches the worker boundary, runtime/workspace configuration is explicit and fail-closed, cancellation is selection-scoped, and estimate evaluation cannot bypass the typed application session. The follow-on desktop checkpoint re-verifies the pinned runtime manifest, provenance, host, and full artifact closure at startup and accepts only manifest-derived worker/library paths. GUI-5 exercised the configured boundary in the actual Apple-Silicon app, including path-redacted success, cancellation, bounded failure, and recovery; the new composition is additionally covered by the real-worker host smoke. These requirements remain incomplete because policy, audit, rate approvals, and results are developer-session-only; the manifest is unsigned and mutable; runtime deployment policy, durable integrity-protected audit, OS-level egress tests, navigation interception, signed packages, full diagnostics review, and three-OS native evidence are pending.
