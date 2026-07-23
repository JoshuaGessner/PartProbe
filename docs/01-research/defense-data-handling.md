# Defense Data Handling Research

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirement IDs:** REQ-NF-002, REQ-NF-005, SEC-001–SEC-010  
> **Related architecture decision IDs:** ADR-0001, ADR-0005, ADR-0006  
> **Open questions:** Contract clauses, CUI markings/category, export jurisdiction/classification, CMMC level, SSP boundary, incident-reporting process, and user citizenship/authorization policy  
> **Dependencies:** security/deployment models, legal/export-control counsel, contract review, customer security requirements  
> **Supersedes:** None

## Non-compliance position

This research is architectural guidance, not legal advice, an export classification, a CUI determination, or a CMMC/NIST/ITAR/EAR compliance determination. Enable a customer to document and enforce its approved policy; require legal, export-control, contractual, and security review before handling controlled data.

## Formal context

- CUI is information that law, regulation, or government-wide policy requires/permitted safeguarding or dissemination controls; NARA operates the CUI Registry and its categories can have different specified controls. Classification must be based on authoritative marking/contract/category analysis, not a “defense” label. [NARA CUI overview](https://www.archives.gov/cui), [NARA registry glossary](https://www.archives.gov/cui/registry/cui-glossary.html)
- NIST SP 800-171r3 provides recommended security requirements for the confidentiality of CUI in relevant nonfederal-system components and has companion assessment procedures. It is not a product certification. [NIST SP 800-171r3](https://csrc.nist.gov/pubs/sp/800/171/r3/final)
- Current DFARS 252.204-7012 defines covered defense information and a covered contractor information system; a contract clause determines applicability. Current CMMC clause/provisions and assessment requirements also vary by solicitation and rollout. [DFARS 252.204-7012](https://www.acquisition.gov/dfars/252.204-7012-safeguarding-covered-defense-information-and-cyber-incident-reporting), [DFARS 252.204-7021](https://www.acquisition.gov/dfars/part-252-solicitation-provisions-and-contract-clauses), [DFARS 204.7504](https://www.acquisition.gov/dfars/204.7504-solicitation-provision-and-contract-clause.)
- ITAR technical data includes information required for design, development, production, manufacture, assembly, operation, repair, testing, maintenance, or modification of defense articles, subject to stated exclusions. Export-control jurisdiction/classification and authorized-release decisions require qualified review. [22 CFR 120.33](https://www.ecfr.gov/current/title-22/chapter-I/subchapter-M/part-120/subpart-C/section-120.33)

## Product data policy

### Classification states

`unclassified-commercial`, `contract-review-needed`, `CUI-basic`, `CUI-specified`, `export-controlled-review-needed`, `ITAR-confirmed-by-authority`, `EAR-confirmed-by-authority`, `classified-not-supported`, and `unknown/restricted`. Only an authorized reviewer may transition into or out of confirmed states. “Classified not supported” must block import into the ordinary application environment.

Store classification basis, source document/revision, authority, reviewer, decision date, dissemination limits, retention/disposition rule, and policy version. A classification tag is not a legal determination.

### Safeguards to design

- Local-first default: no telemetry, cloud sync, external AI submission, or content-bearing crash reports by default.
- Project and classification boundary at every attachment, derivative/preview, export, print, clipboard action, backup, log, and support bundle.
- Least-privilege roles and project membership; deny-by-default controlled exports; strong authentication/MFA as required by the deployed policy; session expiration/revocation.
- Cryptographic attachment hash, immutable audit records, revision lineage, encryption/key-management decision, secure backup/restore, and retention/disposition enforcement.
- Safe import/parser isolation, path containment, malware/content screening policy, and content-redacted diagnostics.
- Authentication, authorization, audit, configuration/change control, incident response, media protection, personnel/physical controls, and supplier risk remain organization/deployment controls; the product can provide evidence but cannot supply compliance by itself.

## Data-flow rules

1. Intake requires user classification selection or `unknown/restricted`; no automatic external processing.
2. Derivatives (thumbnail, mesh cache, measurement report, quote export, log attachment) inherit the strictest applicable classification unless an authorized policy explicitly declassifies/releases them.
3. Exports require a purpose, recipient, approval if policy requires it, watermark/marking policy, audit event, and secure delivery choice. Printing and clipboard follow the same authorization policy.
4. Backups preserve classification, access restrictions, encryption/key availability, integrity verification, retention, restore testing, and destruction evidence.
5. Support/diagnostic collection must default to metadata-only; any content inclusion requires an explicit, logged, authorized action.

## Research basis

- NIST’s CUI guidance applies only to components that process/store/transmit CUI or protect those components, which supports a documented system/deployment boundary. [NIST SP 800-171r3 scope](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/800-171r3/NIST.SP.800-171r3.html)
- NIST SP 800-218 SSDF recommends integrating secure-development practices into the SDLC; use it as development guidance, not a certification claim. [NIST SSDF](https://csrc.nist.gov/pubs/sp/800/218/final)
- NIST SP 800-61r3 is the current incident-response guidance and supersedes r2; deployment policy needs an incident process beyond product logging. [NIST incident-response project](https://csrc.nist.gov/projects/incident-response)
