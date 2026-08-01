# Security Model

> **Status:** Draft  
> **Last updated:** 2026-08-01
> **Related requirement IDs:** REQ-NF-002, REQ-NF-005, REQ-NF-007, REQ-NF-009, REQ-NF-015–REQ-NF-019, SEC-001–SEC-014  
> **Related architecture decision IDs:** ADR-0001, ADR-0005, ADR-0006, ADR-0012–ADR-0014  
> **Open questions:** Cryptographic/key-management provider, identity/MFA, supported classifications, threat model, and incident owner  
> **Dependencies:** permissions matrix, deployment models, document storage, import boundary, audit persistence  
> **Supersedes:** None

## Security objective

Protect estimate, CAD, drawing, pricing, and attachment confidentiality and integrity with a local-first default. The security architecture supplies configurable safeguards and evidence; it does not make a compliance claim for any deployment.

## Trust boundaries

1. **Desktop client:** untrusted file/UI input; holds the minimum decrypted data needed for the active authorized session.
2. **Import worker:** separate, least-privilege process for geometry/document parsing and thumbnail/derivative creation; receives either one explicitly allowlisted inherited resource or an explicitly selected, identity/hash/length-bound verified private copy. Unix descriptor inheritance now allowlists stdio plus the intended source and proves an unrelated inheritable descriptor is absent after `exec`; Windows HANDLE inheritance remains unavailable. OS sandbox/no-network and resource containment are still unimplemented, so “no direct access outside approved resources” remains a target control rather than a completed platform claim.
3. **Application service (team/controlled profile):** policy decision/enforcement point for identity, authorization, records, audit, and exports.
4. **Object/document storage:** classification-aware attachments and derivatives, content hashes, revision lineage, retention policy, and encryption only when required by an approved deployment/key-management decision.
5. **Backup/update/diagnostic channels:** separately authorized; no data-bearing telemetry or support upload by default.

## Controls by concern

| Concern | Architecture requirement |
| --- | --- |
| Identity/session | strong authenticated identity in team/controlled modes; server-side expiry/revocation; no secrets in project files; platform secure storage for local credentials/keys. |
| Authorization | deny by default; enforce project + role + classification + record-state rules at service/repository boundary, not only UI. |
| Attachments | hash at ingest; immutable original, versioned derivatives, type/size limits, path containment, quarantine/staging, parser isolation. |
| Encryption | choose algorithms/key hierarchy in a separate ADR; encrypt in transit for services and encrypt stored controlled data where policy requires. Support rotation/recovery without hard-coded keys. |
| Audit | append-only logical events with actor, time, action, record/version, policy outcome, and correlation ID; avoid CAD/drawing/price content in logs. |
| Export/print/clipboard | policy-controlled and audited; explicit classification inheritance for generated previews/reports; controlled data default-deny. |
| Backup/restore | classified metadata, least-privilege location, encryption when required by the approved profile/key decision, integrity test, retention/disposition, documented restore exercise. |
| WebView/content | packaged local content only by default; restrictive CSP/connect/script/navigation allowlists; no remote script or content-bearing request; external links hand off only after explicit policy/user action without sensitive query/referrer data. |
| Updates | signed artifacts, verification before install, rollback/revocation strategy, reproducible provenance/SBOM decision, no auto-update in restricted profiles unless authorized. |
| Dependencies | locked/reviewed dependencies, vulnerability response process, license/maintenance record, and supply-chain evidence. |
| Advanced imports | bounded local staging, declared source/schema version, parser isolation, classification inheritance, external-reference rejection, and no silent network retrieval. |
| Learning evidence | least-privilege access to correction/CAM/actual cohorts; purpose/retention/data-rights policy; small-cohort controls; de-identification where approved; immutable activation audit. |
| Commercial state | capacity, readiness, supplier quotes, bid scores, and margins are sensitive business data with role-scoped display/export and content-free logs. |

## Threat-driven requirements

Mitigate malformed CAD/document imports, path traversal, unsafe previews, remote WebView navigation/script/fetch/WebSocket egress, unsafe external-link handoff, unauthorized project sharing, stale sessions, accidental export, data leakage through logs/telemetry/support bundles, compromised updates/dependencies, lost devices/backups, and unauthorized administrator action. The threat model must be versioned per profile and tested; threats outside product control (facility access, staffing, network, incident response) require deployment-owner policy.

## Secure development

Adopt a documented secure-development practice aligned to NIST SSDF: define security requirements, protect development/release components, produce/test secure releases, and respond to vulnerabilities. NIST describes SSDF as high-level practices that can be integrated into an SDLC; it is guidance, not certification. [NIST SP 800-218](https://csrc.nist.gov/pubs/sp/800/218/final)

For CUI-bound systems, scope controls to components that process/store/transmit CUI or protect those components, and document the boundary. NIST SP 800-171r3 is the relevant published guidance; contract and CUI-specific controls may exceed or alter the baseline. [NIST SP 800-171r3](https://csrc.nist.gov/pubs/sp/800/171/r3/final)

## Required security evidence

Architecture/configuration version; threat model; authorization tests; audit/export samples without content; dependency/SBOM provenance; signed-update verification; backup/restore drill; parser-fuzz/crash-containment results; incident runbook/tabletop; and controlled-data policy review. Formal assessment/compliance evidence is owned by the deploying organization.

## Current implementation boundary

The headless `security` and `application` crates now define versioned allow/deny decisions, an explicit deny-all baseline, content-minimized authorization context, an append-only audit port, and a local geometry-asset read service that fails closed when decision audit cannot be appended. The open directory capability is bound to a stable root ID, so policy evaluation and filesystem containment use the same root identity. The narrow `platform` crate now implements Unix descriptor duplication, exec-time allowlisting, and one-shot worker claim with focused unsafe review; this is not an OS sandbox. No role catalog, project-membership repository, classification policy, identity provider, durable audit store, deployment-specific allow policy, Windows HANDLE allowlist, or CPU/memory/network containment is implemented; those remain required adapters and approval evidence rather than inferred defaults.

References: [permissions matrix](../03-requirements/permissions-matrix.md), [defense-data research](../01-research/defense-data-handling.md), [security testing](../06-quality/security-testing.md).
