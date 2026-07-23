# ADR-0014 — Vendor-Neutral CAM Reconciliation Boundary

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-058–REQ-F-059, REQ-F-065; REQ-NF-017–REQ-NF-021; TEST-079–TEST-084  
> **Related ADRs:** ADR-0005–ADR-0008, ADR-0013  
> **Open questions:** Shop CAM products/versions, export formats, stable identifiers, time bases, API licensing, and machine-result sources  
> **Dependencies:** Sanitized shop CAM fixtures, canonical-contract spike, integration/security review, and actuals model  
> **Supersedes:** None

## Decision proposed

Keep CAM systems behind versioned adapters and persist an immutable, vendor-neutral canonical reconciliation model. Deliver local manual/file reconciliation first; add shop-selected report/API adapters only after fixture and security validation. Preserve original artifacts and hashes, adapter/schema/CAM/post versions, units/time bases, mapping evidence, and review decisions.

Machine observations and final job-cost actuals enter through separate ports. They may be compared with CAM results but cannot be conflated with them. The core has no required CAM vendor, CAM runtime, cloud service, or NC generation/execution dependency.

## Rationale

CAM exports vary by product, version, postprocessor, report template, and shop customization. Even when a product emits a setup sheet, the shape and semantics are product-specific; Autodesk Fusion's official workflow, for example, generates an HTML setup sheet from selected setup/NC-program/operation content ([Autodesk Fusion documentation](https://help.autodesk.com/cloudhelp/ENU/Fusion-CAM/files/MFG-SETUP-SHEET-STEPS.htm)). A canonical contract isolates that variability and allows partial, reviewable mappings.

For later equipment observations, MTConnect provides an open manufacturing-equipment semantic model and agent protocol, but it is not a CAM-plan or cost schema ([MTConnect overview](https://www.mtconnect.org/), [MTConnect standard downloads](https://www.mtconnect.org/standard-download20181)). Keeping machine evidence separate prevents the integration boundary from assigning authority it does not have.

## Alternatives considered

- **Embed one CAM vendor SDK/data model in the core:** rejected because it creates product, licensing, upgrade, deployment, and test coupling.
- **Treat postprocessed NC/G-code as the canonical plan:** rejected because vendor/control dialects and lost setup/tool/intent context make universal inference unreliable and unsafe.
- **Use CAM simulated time as production actual:** rejected because simulation scope and machine/operator reality differ.
- **Manual reconciliation forever:** acceptable for foundation but rejected as the long-term constraint because repeatable approved imports can reduce mapping effort and errors.
- **Direct cloud/API integration first:** rejected because shop vendors and controlled-data policies are unknown and offline use is required.

## Consequences

The project must maintain a canonical schema, compatibility/migration rules, adapter conformance suite, source artifact store, mapping UI/workflow, vendor-version fixtures, and clear time/value provenance. Some vendor fields will remain unsupported and visible. Adapter development is prioritized only for verified shop demand. NC program transfer/execution remains outside PartProbe.

## Required controls

- Local/offline import is the default; network adapters require destination, authentication, classification, egress, retention, retry/idempotency, and audit approval.
- Reports/HTML/XML/JSON/CSV and plugins are untrusted inputs with bounded staging, hashes, allowlists, parser/resource limits, and sanitized diagnostics.
- Unknown units, time bases, source versions, or ambiguous mappings block affected comparisons; missing values never become zero.
- Simulated, machine-observed, and production-actual values remain distinctly typed and labeled.
- Import/re-import creates immutable sibling snapshots and never mutates estimates, CAM artifacts, reconciliations, or actuals.
- Vendor/tool records are not silently promoted into approved shop libraries.

## Acceptance gate

TEST-079–TEST-084 pass across the PartProbe interchange plus at least one representative shop-selected export; before claiming vendor neutrality, exercise a second structurally different fixture family without changing the domain core. Demonstrate partial/hostile import handling, unit/time-basis validation, ambiguous mapping review, exact snapshot replay, variance classification, authorization/audit, no external fetch from report content, adapter upgrade compatibility, and no mutation of approved estimates. Licensing and support policy must be documented for every vendor adapter before ADR acceptance.

