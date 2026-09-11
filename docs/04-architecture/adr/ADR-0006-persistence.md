# ADR-0006 — Standalone Persistence

> **Status:** In Review
> **Last updated:** 2026-09-10
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

## Current spike evidence

USE-2 schema v1 provides local evidence for an application-owned repository port, empty-on-install libraries, immutable rate-card/pricing-policy/settings revisions, optimistic stale-writer rejection, checksummed migration bootstrap, integrity-on-open, backup/reopen, and historical settings replay. Schema v2 adds immutable, hash-bound starter-resource aggregate and child-record snapshots for separately typed material/offer/stock/machine/runtime records and prevents child identity/version reuse through a new aggregate version. Schema v3 adds immutable `shop-resource-catalog-v1` aggregate and selection snapshots plus exact settings references, preserves mutually exclusive starter/catalog contexts, proves no-default forward migration from schemas v1 and v2, and replays a real schema-v2 settings payload. Schema v4 adds content-minimized, append-only catalog authorization events with exact settings/catalog/selection foreign keys, normalized hash verification, exact idempotent correlation replay, changed-correlation rejection, empty migrations from schemas v1-v3, and backup/reopen evidence. A draft application requires exact settings/catalog expectations, preserves current child bytes, accepts only correctly versioned Draft additions/successors, owns selection preservation/downgrade, and creates immutable catalog successors through the repository port. A separate catalog application requires exact versions, a policy decision, and decision audit before creating immutable activation successors. Native Settings composes both with independent repository/audit connections; ordinary startup has no authenticated actor and retains deny-all activation, while controlled native configurations prove draft save/reopen, mismatch denial, and durable activation allow/reopen behavior. Desktop contract v7 represents catalogs through path-free read-only snapshots; contract v8 adds one exact host-evidence activation request/result command, and contract v9 adds one complete bounded draft-save request without changing schema v4 or accepting WebView selections, identity/time/profile/path/storage authority. The legacy save request cannot mutate catalogs and the host retains fail-visible overwrite prevention. Contract v9 reuses matching immutable source evidence while reconstructing typed values, rejects changed content under a reused version, and has no WebView invocation or editor yet. The adapters reuse unchanged immutable evidence across settings revisions and use an exact, bundled `rusqlite` dependency so the developer spike does not depend on an ambient system SQLite version. This evidence is reversible and keeps this ADR `In Review`; it does not yet satisfy authenticated identity/roles, a shop-reviewed shipped allow configuration, external tamper protection/access control, enabled desktop catalog editing/activation UI, crash recovery, estimates/blobs, three-OS behavior, or protected-storage threat/encryption decisions.
