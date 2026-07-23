# Observability

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-NF-002, REQ-NF-005; SEC-002, SEC-007  
> **Related ADRs:** ADR-0005, ADR-0006  
> **Open questions:** OQ-021  
> **Dependencies:** Deployment/security policy  
> **Supersedes:** None

Observability is local-first and sensitive-data-safe. Structured events may include operation name, duration bucket, result class, algorithm/version, correlation ID, format, byte-size bucket, and sanitized error code. They must not include model coordinates, filenames, customer/part identity, prices, drawing text, SQL values, paths, or attachment contents by default.

## Channels

- **Audit log:** durable security/business events; append-preserving and access controlled.
- **Diagnostic log:** rotating local technical events with configurable retention.
- **Metrics:** in-memory/local aggregates for stage timing and failures.
- **Support bundle:** explicit user action, preview/redaction, manifest, and scope record.

There is no automatic external telemetry. Debug modes cannot silently weaken classification or redaction policy.
