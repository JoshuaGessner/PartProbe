# Security Testing

> **Status:** Draft  
> **Last updated:** 2026-08-09
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

Current TASK-003 evidence launches a real child with one direct source resource and a deliberately inheritable sentinel on each supported OS family. Unix proves only stdio plus the intended descriptor survive `exec`; Windows uses an exact `PROC_THREAD_ATTRIBUTE_HANDLE_LIST` for child stdin/stdout, null stderr, and the source HANDLE, then proves the unrelated event HANDLE is absent and remains unsignaled in the parent. Both worker paths clear further inheritance before claiming the source exactly once. Every supervised launch also receives explicit CPU, memory-request, and output-file limits before execution. Unix tests prove hard CPU/file termination and process-group cleanup; Linux additionally proves an address-space cap. Windows creates the worker suspended, assigns it to a CPU/memory/one-process/kill-on-close Job before resume, and tests memory plus descendant containment. Process tests also cover preferred/required direct execution, malformed resource rejection, cancellation acknowledgement, and continued use of the open grant after unlink. This does not prove network denial, filesystem sandboxing, aggregate-output bounds, hard macOS memory containment, hostile Unix descendant containment, or parser safety.

Current GUI-4/GUI-5 evidence separately checks the desktop/WebView boundary. Tests require an exact five-command Tauri app manifest, an exact main-window capability with only those commands and event listen/unlisten, packaged-local asset URLs, restrictive production/development CSPs, prototype freezing, no remote URL or broad plugin permission, safe typed errors, leaf-name-only selection summaries, and path-free provisional/result DTOs. Analysis and cancellation accept only the opaque selection token; evaluation additionally binds the retained analysis ID and accepts no path or CAD content. Blocking application/worker/evaluation work runs outside the UI task, stale tokens fail closed, and cancellation can signal only the matching active analysis. GUI-5 exercised path-redacted success, cancellation, bounded failure, and recovery in the actual Apple-Silicon app. The follow-on startup verifier adds five portable valid/tamper/traversal/provenance/host cases and a passing real-worker smoke, but is unsigned and cannot prevent post-verification mutation. Run 31346159820 passes the full default build/test boundaries on macOS, Ubuntu, and Windows at startup-verifier commit `86e00b5`. This does not prove packet-level network denial, remote-navigation interception, log redaction across future adapters, Windows/Linux desktop runtime behavior, signed-package integrity, durable audit, or adversarial WebView testing.

GUI-4 intake evidence proves the application policy decision and append attempt occur before source resolution, fingerprinting, or analysis. An allowed source is hashed within its maximum-input bound from the same already-open grant later consumed by the worker, and the grant is rewound after hashing; a denied request against a nonexistent relative source returns the policy failure, appends one audit event, and never calls analysis. The desktop adapter supplies a deliberately narrow local-developer-session policy and append-only in-memory audit, requires an explicit verified runtime root plus external workspace, derives worker/native-library paths only from the checked manifest, retains the resulting draft session and cancellation handle outside the WebView, validates a complete estimate request before session mutation, and delegates calculation to the typed application service. This avoids a desktop-host CAD pre-read and calculation duplication while preserving independent worker verification. It does not establish durable identity/audit/rate approval, signed/immutable packaged runtime configuration, target containment, or configured three-OS end-to-end native UI evidence.

NIST SSDF recommends integrating security practices into development and release processes; NIST’s current incident-response guidance emphasizes integration with cybersecurity risk management. These sources guide the test program and do not certify it. [NIST SSDF](https://csrc.nist.gov/pubs/sp/800/218/final), [NIST SP 800-61r3 project](https://csrc.nist.gov/projects/incident-response)

## Release blockers

Block a release for authorization bypass, controlled-data egress by default, unaudited controlled export, parser escape from intended boundary, secrets/content in diagnostics, unsigned/unverified update acceptance, restore-integrity failure, or any unresolved critical issue under the deployment’s approved risk policy. Compliance assessment remains the responsibility of the deploying organization and its qualified assessors.

References: [security model](../04-architecture/security-model.md), [permissions matrix](../03-requirements/permissions-matrix.md), [defense-data research](../01-research/defense-data-handling.md).
