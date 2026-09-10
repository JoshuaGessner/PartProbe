# Material Model

> **Status:** Draft
> **Last updated:** 2026-09-10
> **Related requirements:** REQ-F-005, REQ-F-008; DATA-005
> **Related ADRs:** ADR-0006
> **Open questions:** OQ-009, OQ-014
> **Dependencies:** Supplier/shop data
> **Supersedes:** None

A versioned material record includes family, alloy/grade, specification, temper/condition, hardness, density with source, machinability attributes, stock forms/sizes, restrictions, and approval state. Commercial offers follow [the canonical rate-library separation](rate-library.md) and remain distinct time-bounded records: supplier, supplier part, price basis, effective/expiry dates, minimum, cut/cert/freight charges, lead time, lot/country restrictions, and evidence attachment.

Material identity is never inferred authoritatively from geometry. Every estimate pins both the material-definition version and selected commercial offer or clearly records a manual assumption.

The current USE-2 starter bundle implements one bounded draft `MaterialDefinition` plus one separately versioned `MaterialOffer`. It retains family, grade, optional specification/condition, density in explicit kg/m³, manual source, supplier, exact USD/kg price, effective date, lifecycle state, and exact cross-record version references. It is local persistence evidence only: the record is not approved, no catalog is seeded, and no estimate resolves or applies it yet.

The `shop-resource-catalog-v1` contract admits bounded collections of exact material and offer versions. Every offer must resolve to a material version inside the same catalog and use the catalog currency. Duplicate identity/version pairs, dangling references, and mismatched reviewed selections fail validation. Schema v3 persists the exact catalog, child, selection, and settings-reference evidence; schema v4 persists a separate content-minimized authorization decision. A headless policy-and-audit-gated application transition can mark one reviewed exact chain active for future proposals, and native Settings composes that boundary with deny-all startup and controlled exact-operator evidence. Contract v7 presents the catalog path-free and read-only. No authenticated identity/roles, shop-reviewed shipped allow configuration, desktop editing/activation, material proposal, or estimate behavior exists yet.
