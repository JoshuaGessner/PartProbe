# Aerospace Quality Research

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirement IDs:** REQ-F-013, REQ-F-018, REQ-F-029, SEC-006–SEC-009  
> **Related architecture decision IDs:** ADR-0006, ADR-0008  
> **Open questions:** Applicable customer clauses, AS9100 certification scope, FAI/CMM policy, source inspection, and approved special-process vendors  
> **Dependencies:** drawing/PO ingestion, quality plan, traceability policy, inspection and document-storage models  
> **Supersedes:** None

## Boundary

PartProbe is an estimating and evidence-management aid. It must not claim that a shop, quote, part, or deployment is AS9100, AS9102, Nadcap, or customer-clause compliant. Applicability and conformance require the organization’s quality system, contract review, controlled procedures, and qualified personnel.

## Source status

| Source | What it establishes | Product implication |
| --- | --- | --- |
| IAQG 9100 | Aerospace QMS requirements standard; the public IAQG page describes an ISO 9001-based standard with aerospace/space/defense additions. | Treat as external governing context only when applicable; track standard/customer revision and review evidence. [IAQG 9100](https://iaqg.org/standard/9100-qms-requirements-for-aviation-space-and-defense-organizations/) |
| SAE AS9102C | SAE identifies AS9102C (2023-06-28) as the current revision and describes documentation requirements for FAI. The full normative text is controlled/purchased. | Record a customer/quality-approved FAI requirement and revision; do not embed or recreate copyrighted forms/rules as authoritative. [SAE AS9102 listing](https://saemobilus.sae.org/standards/as9102-aerospace-first-article-inspection-requirement) |
| Nadcap/PRI | Industry-managed accreditation/audit program for critical processes; whether it is required is customer/process specific. | Vendor qualification is an explicit requirement/approval check, not a label inferred from operation type. [PRI Nadcap accreditation](https://www.p-r-i.org/nadcap/accreditation) |

## Estimable quality work packages

Capture each as an editable task with basis, rate/time, evidence deliverable, customer flow-down, and owner.

- Contract and drawing review; key-characteristic/datum/tolerance and surface-finish extraction or manual entry.
- Receiving material verification and material test report/certificate handling; traceability/batch linkage where contractually required.
- First-piece/in-process/final inspection, gage planning, CMM programming and execution, calibrated-equipment availability, and report preparation.
- First article inspection planning and FAIR package preparation/rework; source/customer inspection scheduling and standby.
- Process certifications, special-process routing and vendor document review, serialization/lot traceability, packing/labeling, and records retention.

The application should mark a task `not_estimated`, `allowance`, `quoted`, `confirmed`, or `not_applicable-with-rationale`. A model alone cannot determine FAI, sampling plan, CMM feature count, cert, serialization, or source-inspection requirements.

## Quality/inspection model recommendations

1. Create requirements from a reviewed source: PO, drawing, customer quality clause, internal procedure, or manual entry. Preserve document revision/hash and reviewer.
2. Link each requirement to an inspection/record task, cost element, and evidence location; distinguish requirement from the estimator’s planning assumption.
3. Preserve traceability chains: RFQ → quote revision → part revision → material/lot → routing/operation → inspection result/document → shipment when the project scope requires it.
4. Use immutable approved-estimate and approved-requirement snapshots. Corrections are new revisions with rationale.
5. Gate quote approval on unresolved quality requirements, not on a generic “aerospace” tag.

## Human-review triggers

No drawing; incomplete/mismatched model revision; GD&T/key characteristics; tight or non-default tolerance; surface finish; heat treatment/coating/NDT; material certification; customer/source inspection; special process; FAI; serialization; controlled technical data; or customer-specific flow-down.

## Research basis

- IAQG explains that its certification scheme applies 9100-series standards through accredited certification bodies; this describes a certification ecosystem, not software compliance. [IAQG certification](https://iaqg.org/certification/)
- PRI states that Nadcap covers critical processes including heat treating, chemical processing, NDT, materials testing, measurement/inspection, and conventional machining. It remains an accreditation/vendor requirement only where contractually applicable. [PRI Nadcap accreditation](https://www.p-r-i.org/nadcap/accreditation)
- SAE’s public listing identifies AS9102C but does not expose full normative requirements; defer to the licensed standard and customer flow-down for formal interpretation. [SAE AS9102](https://saemobilus.sae.org/standards/as9102-aerospace-first-article-inspection-requirement)
