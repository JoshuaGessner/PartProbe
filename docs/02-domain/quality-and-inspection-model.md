# Quality and Inspection Domain Model

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirement IDs:** REQ-F-013, REQ-F-018, REQ-F-029, DATA-009, DATA-013  
> **Related architecture decision IDs:** ADR-0006, ADR-0008  
> **Open questions:** Customer clause taxonomy, retention schedule, QMS integration, and which inspection systems are in scope  
> **Dependencies:** part revisions, document storage, audit trail, user/role, routing, outside-vendor models  
> **Supersedes:** None

## Scope

Model quality work, evidence, traceability, and uncertainty for estimating. It is not an electronic QMS or a conformance decision engine.

## Entities and relationships

| Entity | Purpose / key fields |
| --- | --- |
| `RequirementSource` | PO, drawing, specification, customer clause, internal procedure, or manual entry; revision/hash, page/section, owner, review state |
| `QualityRequirement` | type, wording/reference, applicability, formal-vs-assumption status, required evidence, due phase, disposition, reviewer |
| `InspectionPlan` | versioned tasks tied to part revision, operation/setup, feature/characteristic, method, equipment/skill, sampling/first-piece, estimate basis |
| `InspectionTask` | labor/machine time, program/fixture need, cost owner, status, confidence, blocking flag, evidence links |
| `TraceabilityLink` | links RFQ/quote/part revision to material lot, operation, inspection record, certificate, vendor, and shipment as applicable |
| `EvidenceRecord` | content hash, classification, document revision, issuer, received/verified time, verifier, retention/disposition policy |
| `QualityOverride` | estimate change with original/new values, actor, time, reason, approval |

Use stable UUIDs and immutable approved snapshots. Source documents remain attachments with hashes; extracted data is a reviewed derivative, not a replacement for the source.

## Requirement states

`unreviewed` → `needs-clarification` | `applicable` | `not-applicable-with-rationale` → `estimated` → `confirmed` → `completed/evidence-linked`.

Only approved workflow roles may mark a requirement confirmed or evidence verified. An `unreviewed`/`needs-clarification` required item blocks any workflow that the shop policy says needs full requirement review.

## Estimation semantics

- Separate inspection effort from CMM programming and CMM runtime; distinguish internal labor from outside vendor cost.
- Record an FAI as an externally required/approved work package with standard/customer revision and deliverable scope. Do not infer it from “aerospace.”
- Model certificates, traceability, special-process qualification, source inspection, serialization, and retention as separate checklist/tasks because their applicability differs.
- Drawing/GD&T, key characteristics, finish, material condition, and customer clauses can increase setup, fixturing, runtime, inspection, and risk; link the rationale to each cost contribution.

## Audit and controlled data

Every change to requirement applicability, inspection plan, estimate basis, evidence link, approval, or export produces append-only audit metadata: actor, event time, before/after reference, reason, and classification context. Access to an evidence record is authorized at project + classification + role level and audit logged.

References: [research: aerospace quality](../01-research/aerospace-quality.md), [research: defense data](../01-research/defense-data-handling.md).
