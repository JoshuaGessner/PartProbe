# ADR-0006 — Standalone Persistence

> **Status:** In Review
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-F-001, REQ-F-010–REQ-F-012, REQ-F-032; REQ-NF-002, REQ-NF-007, REQ-NF-009, REQ-NF-020
> **Related ADRs:** ADR-0005, ADR-0007, ADR-0008
> **Open questions:** Encryption/key policy; team-service database
> **Dependencies:** Persistence/backup spike
> **Supersedes:** None

## Decision proposed

Use SQLite for standalone structured data, a content-addressed local document store for large source/derived files, and repository/unit-of-work interfaces at the application boundary.

## Rationale

SQLite fits a single-workstation, offline, transactional deployment and documents atomic commit behavior ([official documentation](https://sqlite.org/atomiccommit.html)). Separating blobs avoids database bloat and makes source integrity manifests explicit. Ports preserve a path to a team service.

## Consequences

We must define WAL-safe backup, foreign-key activation, migrations, concurrency, attachment atomicity, encryption-at-rest decision, corruption recovery, and no-network-share policy. A team deployment is a service, not shared-file SQLite.

## Acceptance gate

Concurrent draft test, crash/recovery test, migration from two fixtures, immutable rate-entry/card/selector/pricing/rounding version persistence and prior-estimate replay, database+blob backup/restore/integrity test on three OSes, threat review, and an explicit encryption-at-rest/key-management decision for each deployment profile before claiming protected storage.
