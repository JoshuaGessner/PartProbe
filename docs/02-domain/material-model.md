# Material Model

> **Status:** Draft
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-F-005, REQ-F-008; DATA-005
> **Related ADRs:** ADR-0006
> **Open questions:** OQ-009, OQ-014
> **Dependencies:** Supplier/shop data
> **Supersedes:** None

A versioned material record includes family, alloy/grade, specification, temper/condition, hardness, density with source, machinability attributes, stock forms/sizes, restrictions, and approval state. Commercial offers follow [the canonical rate-library separation](rate-library.md) and remain distinct time-bounded records: supplier, supplier part, price basis, effective/expiry dates, minimum, cut/cert/freight charges, lead time, lot/country restrictions, and evidence attachment.

Material identity is never inferred authoritatively from geometry. Every estimate pins both the material-definition version and selected commercial offer or clearly records a manual assumption.
