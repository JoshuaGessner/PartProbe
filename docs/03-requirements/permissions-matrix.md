# Permissions Matrix

> **Status:** Draft  
> **Last updated:** 2026-07-29
> **Related requirement IDs:** REQ-F-020, REQ-F-032–REQ-F-034, SEC-003, SEC-005, SEC-007, SEC-008  
> **Related architecture decision IDs:** ADR-0006  
> **Open questions:** Identity provider, MFA policy, customer/external roles, export approver, and classification authority  
> **Dependencies:** security model, deployment model, audit model, data-classification policy  
> **Supersedes:** None

## Principles

Authorization is deny-by-default and evaluated as `role permission ∩ project membership ∩ classification authorization ∩ record state`. UI visibility is not authorization. Server/service enforcement is mandatory for team and controlled-data profiles; standalone has local-user policy boundaries but no claim of multi-user separation.

## Baseline roles

| Capability | Administrator | Estimator | Programmer/Manufacturing | Quality | Purchasing | Approver | Controlled-data custodian | Auditor |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Manage users/roles/policy | Manage | — | — | — | — | — | delegated classification policy | Read audit only |
| Read project commercial data | yes | assigned | assigned | assigned | assigned | assigned | assigned | scoped |
| Read controlled attachment | policy + need | only authorized/assigned | only authorized/assigned | only authorized/assigned | only authorized/assigned | only authorized/assigned | approve/revoke scope | audit scope |
| Create/edit draft estimate | optional | yes | routing/tooling sections | quality sections | supplier cost sections | review only | classification fields | — |
| Override calculation | policy | propose | propose | quality-only propose | cost-only propose | approve/reject where delegated | — | read trail |
| Approve quote/release revision | policy | propose | recommend | quality concurrence | cost concurrence | yes | controlled release concurrence | — |
| Change classification/retention | policy | request | request | request | request | request | authorized action | read trail |
| Export/print/share controlled data | policy | request | request | request | request | request | authorize when policy requires | audit only |
| Manage backup/restore | policy | — | — | — | — | — | classification controls | observe evidence |
| View immutable audit log | security scope | own project | own project | quality scope | own procurement scope | approval scope | controlled scope | authorized scope |

“yes” means only within assigned project/classification scope; it does not grant global access. Role names and final scope require shop policy approval.

## State-based controls

- Draft estimates: authorized contributors can edit their owned sections; all changes are revisioned.
- Submitted/under review: edits create a new proposed revision or require return-to-draft; no silent mutation.
- Approved: immutable snapshot. A change creates a new revision and re-approval path.
- Controlled/unknown-restricted: no default external sync, AI upload, content-bearing diagnostics, export, print, or clipboard transfer. Actions require permitted role + classification policy and audit event.
- Classified: unsupported; block ordinary project import/storage and direct the user to the organization’s approved process.

## Administrative requirements

1. Separate identity administration from quote approval where staffing permits.
2. Require reason, actor, timestamp, original/new value, and approval for override and classification transitions.
3. Revalidate active assignments on project closure, role change, classification change, and session revocation.
4. Audit allow and deny decisions for sensitive read/export/print/download/backup/restore events without placing content in logs.
5. The matrix is a product design baseline, not a CMMC, NIST, ITAR, EAR, or contract compliance determination.

## Current implementation boundary

The application asset-read service evaluates actor, project, record/version, classification, record state, operation, and the identity bound to the open asset root. It records the exact policy ID/version, allow/deny outcome, stable reason code, correlation ID, and trusted application timestamp before any relative file resolution. Missing deployment policy uses an explicit versioned deny-all implementation, and an unavailable audit sink fails closed. The baseline roles above remain unconfigured product guidance: no role name or organization-specific permission is hard-coded, and no requirement in this matrix is Complete.

References: [security model](../04-architecture/security-model.md), [defense-data research](../01-research/defense-data-handling.md).
