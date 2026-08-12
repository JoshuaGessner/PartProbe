# Deployment Models

> **Status:** Draft  
> **Last updated:** 2026-08-12
> **Related requirement IDs:** REQ-NF-001, REQ-NF-002, SEC-001–SEC-010  
> **Related architecture decision IDs:** ADR-0001, ADR-0005, ADR-0006  
> **Open questions:** Target networks, air-gap/offline needs, identity source, backup target, update process, and supported operating systems  
> **Dependencies:** security model, persistence/document storage, permissions matrix, backup/recovery policy  
> **Supersedes:** None

## Profiles

| Attribute | Standalone | Team/LAN | Controlled-data environment |
| --- | --- | --- | --- |
| Topology | one desktop, local database/files | desktop clients + internal service, central database/document store | approved restricted network/service boundary; may be disconnected/air-gapped |
| Identity | OS/local application account policy | central authenticated identity and project membership | strong approved identity, MFA/session policy where required, explicitly authorized user scope |
| Data | local project vault and local backup | central source of truth; no required public cloud | classification-aware stores/derivatives/backups; restricted export/print/support |
| Connectivity | offline capable | LAN-first, no public cloud dependency | default no telemetry/external AI/cloud egress; allow-listed integrations only |
| Audit | local audit/revision record | central tamper-evident operational audit | centralized, protected audit with controlled review/export |
| Backup | manual guided encrypted backup/recovery test | controlled scheduled backup/restore operations | policy-approved encrypted backup, access, retention/disposition, restore drill |
| Updates | signed package, user/admin initiated | staged signed rollout | documented offline/controlled update chain and approval |

## Profile-selection guardrail

Choose the profile from actual data classification, contract terms, users, network, and operational policy—not customer industry alone. A product profile does not confer CMMC, DFARS, NIST, ITAR, EAR, or AS9100 compliance. Obtain contractual/security/export advice before placing controlled technical data in any boundary.

## Developer runtime is not a deployment profile

The content-addressed native runtimes assembled for the evidenced Apple-Silicon and Ubuntu TASK-003 checkpoints are local engineering artifacts. The same manifest-bound shape is implemented for Windows x64, with canonical construction directories and app-local DLLs beside the worker, but run 31599859156 stopped at strict PartProbe bridge compilation after verifying OCCT installation/fingerprints; Windows runtime/host success remains pending active corrected run 31605188245. These runtimes require an external explicit worker workspace and do not provide installation, a bundled MSVC runtime, signing, OS sandboxing, identity, authorization repositories, durable audit, backup, update, or controlled-data operations. Do not treat construction or local verification as Standalone, Team/LAN, or controlled-data deployment acceptance.

## Common deployment invariants

- Hash original CAD/drawing imports and generated derivatives; preserve revision lineage and record unit/scale/healing/import warnings.
- Enforce authorization in the data service where a service exists; do not rely on filesystem paths or hidden UI for separation.
- Preserve policy/configuration versions with each audit event and artifact export.
- Default telemetry, cloud synchronization, and content-bearing crash reporting to disabled. Support bundles must be metadata-only unless an authorized user explicitly includes content.
- Keep database/object storage, attachments, previews, export queue, cache, logs, backups, and update cache inside the documented boundary; derivatives inherit classification by default.
- Define recovery objective, responsible owner, and restore test cadence per deployment; untested backup is not recovery evidence.

## Controlled-data minimum design package

Before enablement, produce a deployment boundary diagram, data inventory/classification decision record, contract applicability review, system security/configuration plan, named roles/approvers, secure operations and incident runbooks, export/print policy, backup/retention policy, update policy, and test evidence. NIST SP 800-171r3 identifies CUI-protecting components as within scope; contracting requirements and CUI-specified rules must be evaluated separately. [NIST SP 800-171r3 scope](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/800-171r3/NIST.SP.800-171r3.html)

## Migration and portability

Use an explicit export/import package with schema version, migration log, attachment hashes, classification metadata, and signed manifest. Migrations run transactionally and never downgrade classification or delete audit history. Crossing profiles requires a policy review and explicit authorization; it is not a routine “sync.”

References: [security model](security-model.md), [defense-data research](../01-research/defense-data-handling.md).
