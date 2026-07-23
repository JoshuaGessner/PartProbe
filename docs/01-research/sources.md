# Research Source Register

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** All research-derived requirements  
> **Related ADRs:** ADR-0001–ADR-0014  
> **Open questions:** Vendor commercial terms and shop evidence  
> **Dependencies:** Individual research documents  
> **Supersedes:** None

This register classifies sources; individual research documents cite the exact page supporting each claim.

## Formal and government sources

- NIST CUI publications and CMMC program material: formal government guidance/rules; applicability requires organizational and contractual determination.
- U.S. BIS and DDTC export-control resources: formal government material; not a substitute for counsel.
- FAA/DoD quality and first-article references where cited: formal only within their stated scope.
- ISO, ASME, STEP/IGES, and 3MF specifications: normative where the cited edition is legally accessible and applicable.

## Official technical sources

- Rust and Cargo documentation, [Cargo workspaces](https://doc.rust-lang.org/stable/cargo/reference/workspaces.html).
- [Serde](https://serde.rs/) documentation.
- SQLite documentation on [atomic commit](https://sqlite.org/atomiccommit.html), [WAL](https://sqlite.org/wal.html), [foreign keys](https://sqlite.org/foreignkeys.html), and [defensive use](https://sqlite.org/security.html).
- UI framework, rendering library, geometry kernel, and file-format project documentation cited in their evaluations.
- STEP AP203/AP214/AP242, PMI/MBD, and translator/interoperability sources are cataloged with scope and claim limits in [PMI and MBD research](pmi-and-mbd.md).
- [`rust_decimal`](https://docs.rs/rust_decimal/latest/rust_decimal/) API documentation; suitability remains an engineering decision, not an independent guarantee.

## Vendor claims

Commercial translator, CAD kernel, quoting-platform, tooling, and machine-data claims are labeled as vendor claims and require contract, benchmark, and license review.

## Shop evidence still required

Anonymized quotes, actual job results, rate calculation method, materials, machines, tooling, outside-process practices, quality clauses, approval workflows, security classification decisions, and representative models.

## Citation rule

Prefer direct links to stable primary pages. Record access date for mutable vendor material. Never copy restricted standards into the repository. If a conclusion is inference, label it and name the supporting evidence.
