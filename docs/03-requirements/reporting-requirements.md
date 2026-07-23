# Reporting Requirements

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-009, REQ-F-011, REQ-F-012; REQ-NF-003  
> **Related ADRs:** ADR-0001, ADR-0006  
> **Open questions:** OQ-023, OQ-025  
> **Dependencies:** Quote template and approval policy  
> **Supersedes:** None

## Internal estimate

Show revision/input hashes, quantities, routing, cost categories, risk, assumptions, warnings, confidence reasons, overrides, calculation/library/analysis versions, approval state, and reconciliation to total.

## Customer quote

Show customer/RFQ identity, part/revision, quantities and unit/extended price, NRE/setup presentation, lead-time assumptions, validity, terms, exclusions, customer-visible notes, and approval identity—without leaking internal-only evidence.

## Variance report

Compare original estimate versus actual by category and operation; preserve both version contexts and reason codes; distinguish estimating, geometry, planning, purchasing, and execution variance.

Reports need deterministic snapshot/golden tests, locale/currency rules, pagination/accessibility review, and print/PDF evaluation. A preview is not an approval.
