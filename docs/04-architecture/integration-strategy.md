# Integration Strategy

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-011, REQ-F-012, REQ-F-058–REQ-F-062; REQ-NF-002, REQ-NF-010, REQ-NF-017–REQ-NF-019  
> **Related ADRs:** ADR-0005, ADR-0006, ADR-0014  
> **Open questions:** OQ-019, OQ-022, OQ-043–OQ-048  
> **Dependencies:** Shop system inventory  
> **Supersedes:** None

## Boundary design

Integration uses versioned application-level DTOs with Serde, whose trait-based data model separates Rust types from formats ([Serde overview](https://serde.rs/)). DTO schema version, exporter version, source revision, classification, manifest, and checksums travel together. Internal database rows and geometry-kernel objects never become contracts.

## Priority

1. Offline file import/export with preview and audit.
2. Read-only shop-library and actuals adapters after source-system discovery.
3. Controlled writeback only after authority, conflict, idempotency, and rollback policy.

CAM, capacity, availability, supplier, and marketplace sources first enter through bounded read-only adapters. The normalized core contract retains vendor/system, schema/API version, observation time, import time, freshness result, classification, units, mapping confidence, warnings, and payload digest. Vendor identifiers remain adapter metadata rather than domain types. A failed, stale, partial, or unknown import cannot erase the last known value or masquerade as current.

Marketplace comparison defaults to manual/local quote entry. No CAD, drawing, specification, PMI, feature geometry, customer identity, or derived controlled technical data leaves the deployment without an approved integration-specific policy and explicit action.

Each adapter documents authentication, authorization, data classification, mapping, unit/currency/time semantics, retry/idempotency, rate limits, secret storage, logging, and failure recovery. No public-cloud dependency is required for core estimating.
