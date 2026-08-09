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

GUI-3 provides bounded configuration/test evidence for SEC-001 and SEC-002: the main webview is limited to an exact two-command application manifest, an exact capability with those commands plus event listen/unlisten, and local-content CSPs; no frontend network, shell, filesystem, dialog, opener, updater, upload, or remote-content permission is granted. Raw selected paths remain in native session state and only a leaf display name crosses the bridge. GUI-4 intake adds partial SEC-003/004 evidence that authorization/audit precede source resolution/fingerprinting and that the same bounded open grant reaches the worker boundary. These requirements remain incomplete because runtime deployment policy, durable audit, OS-level egress tests, navigation interception, future adapter review, signed packages, full diagnostics review, and three-OS evidence are pending.
