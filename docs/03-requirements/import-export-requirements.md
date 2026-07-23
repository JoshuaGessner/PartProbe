# Import and Export Requirements

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-001, REQ-F-011; REQ-NF-002, REQ-NF-005; SEC-006–SEC-009  
> **Related ADRs:** ADR-0004, ADR-0006  
> **Open questions:** OQ-019, OQ-022, OQ-025  
> **Dependencies:** Data-classification and integration decisions  
> **Supersedes:** None

- Imports use allowlisted formats, bounded resources, safe paths, source hashes, malware policy hooks, and recoverable staging before domain acceptance.
- Structured import validates schema/version and rejects unknown authoritative fields unless an explicit compatibility rule exists.
- Exports are generated from a named immutable revision and template version; classification/watermark policy is deployment-configurable.
- Customer exports omit internal rates, margins, risk detail, and restricted notes unless an authorized template explicitly includes them.
- Bulk and integration exports require scope preview, authorization, audit event, destination policy, and deterministic manifest/checksums.
- No export or integration silently sends technical data to a public service.

Candidate structured interchange uses versioned Serde DTOs, not persistence-row serialization. ERP/CAM/QMS integration is deferred pending shop discovery.
