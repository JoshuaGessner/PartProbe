# Security Testing

> **Status:** Draft  
> **Last updated:** 2026-08-01
> **Related requirement IDs:** SEC-001–SEC-010, TEST-011, TEST-020, TEST-030  
> **Related architecture decision IDs:** ADR-0001, ADR-0005, ADR-0006  
> **Open questions:** Threat model owner, penetration-test scope, supported import formats, and deployment-specific assurance obligations  
> **Dependencies:** security model, permissions matrix, deployment profile, test data policy  
> **Supersedes:** None

## Scope and safety

Run security testing only against authorized environments and sanitized/non-controlled fixtures. Test results, screenshots, logs, and crash artifacts inherit the project’s data policy; never upload controlled CAD/drawings to third-party scanning or issue trackers without authorization.

## Required test matrix

| Area | Tests |
| --- | --- |
| Authentication/session | invalid/expired/revoked session; credential reset; MFA/identity integration where configured; no secrets in local/project exports |
| Authorization | deny-by-default; cross-project access; role escalation; record-state/classification checks; service endpoint enforcement independent of UI |
| Controlled actions | export, print, clipboard, share, support bundle, backup/restore and classification transition require authorization and produce audit evidence |
| Import/parser | malformed, oversized, polyglot, archive-bomb/path-traversal inputs; intended-resource-only transport; capability/job/correlation mismatch; unrelated descriptor/HANDLE exclusion; deleted-source behavior; source replacement/race resistance; explicit verified-copy labeling and revalidation; worker crash/timeout/memory containment; staging cleanup; no arbitrary file read/write |
| Storage/crypto | attachment hash mismatch; tampered manifest; encryption/key/rotation/recovery behavior per approved ADR; cache/preview/log classification inheritance |
| Audit/logging | required events emitted; tampering/ordering/replay behavior; no CAD/drawing/price/secret content in logs; authorized query only |
| Update/dependency | signature failure, rollback/revocation path, dependency inventory/SBOM, known-vulnerability triage and remediation evidence |
| Backup/recovery | unauthorized backup read, restore authorization, integrity verification, retention/disposition, timed restore drill |
| Network/egress | default telemetry and content-bearing diagnostics disabled; restrictive WebView CSP; blocked remote navigation/script/fetch/WebSocket and remote asset resolution; safe explicit external-link handoff; allow-list enforcement; no uncontrolled external AI/cloud upload |

## Methods and evidence

Use unit/integration tests for authorization and policy logic; property/fuzz tests for parsers and boundary formats; static/dependency review; signed-package verification; configuration review; backup restore exercises; and independent authorized assessment when risk/contract requires it. Link each test to the threat, profile/configuration, fixture classification, evidence hash, result, owner, and remediation status.

Current TASK-003 Unix evidence launches a real child with one direct source descriptor and a deliberately inheritable sentinel, then proves the source is readable and the sentinel is absent after `exec`. Process tests additionally cover required-direct execution without a staged copy, malformed descriptor rejection, cancellation acknowledgement over direct transport, and continued use of the open grant after unlink. This is descriptor-inheritance evidence only; it does not prove an OS sandbox, no-network policy, Windows HANDLE allowlisting, CPU/memory containment, or parser safety.

NIST SSDF recommends integrating security practices into development and release processes; NIST’s current incident-response guidance emphasizes integration with cybersecurity risk management. These sources guide the test program and do not certify it. [NIST SSDF](https://csrc.nist.gov/pubs/sp/800/218/final), [NIST SP 800-61r3 project](https://csrc.nist.gov/projects/incident-response)

## Release blockers

Block a release for authorization bypass, controlled-data egress by default, unaudited controlled export, parser escape from intended boundary, secrets/content in diagnostics, unsigned/unverified update acceptance, restore-integrity failure, or any unresolved critical issue under the deployment’s approved risk policy. Compliance assessment remains the responsibility of the deploying organization and its qualified assessors.

References: [security model](../04-architecture/security-model.md), [permissions matrix](../03-requirements/permissions-matrix.md), [defense-data research](../01-research/defense-data-handling.md).
