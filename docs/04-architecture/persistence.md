# Persistence Architecture

> **Status:** In Review  
> **Last updated:** 2026-09-09
> **Related requirements:** REQ-F-001, REQ-F-010–REQ-F-012; REQ-NF-002, REQ-NF-007, REQ-NF-009  
> **Related ADRs:** ADR-0006  
> **Open questions:** OQ-021, OQ-022  
> **Dependencies:** SQLite/repository spike  
> **Supersedes:** None

Standalone uses one local SQLite database plus content-addressed local document storage. SQLite provides transactional local persistence; foreign keys must be explicitly enabled on every connection per the [SQLite documentation](https://sqlite.org/foreignkeys.html). Backups must treat the database and any WAL state correctly; SQLite warns the WAL is persistent state that must remain with its database when copied ([WAL documentation](https://sqlite.org/wal.html)).

## Rules

- Application services own transactions through repository/unit-of-work ports.
- Migrations are ordered, checksummed, forward-tested, and restore-tested from supported prior versions.
- Published/audit records are append-preserving; mutable drafts use optimistic concurrency.
- Attachments are staged, hashed, fsynced according to platform policy, atomically manifested, and integrity-checked on open/backup.
- `PRAGMA foreign_keys=ON`, integrity checks, busy policy, journal mode, and durability settings are explicit and tested.
- Never open a database from another trust domain without defensive treatment; see SQLite's [defensive guidance](https://sqlite.org/security.html).

Team/LAN mode uses a service and central database/object store behind the same application ports. It must not place the standalone SQLite file on a shared network filesystem.

## Current implementation boundary

TASK-003 implements only the persistence-neutral controlled-derivative port and application handoff. The in-memory write contract independently verifies immutable bytes and retains lineage, classification, exact access/retention policy references, authorization correlation, actor/time, schema/media type, integrity state, and an opaque adapter locator. It is not a durable schema and does not prove atomicity, fsync behavior, encryption, integrity-on-open, backup/restore, concurrency, or retention/disposition enforcement.

USE-2 now begins TASK-006 with a bounded SQLite adapter for immutable shop-settings drafts. Schema v1 persists validated `RateCard` and `PricingPolicy` snapshots, immutable settings revisions and change events, plus one optimistic-concurrency current pointer behind an application repository port. Schema v2 adds immutable, hash-bound snapshots for each material/offer/stock/machine/runtime child record, the bounded `ShopResourceLibrary` aggregate, and its exact settings-revision reference. This prevents a changed child from reusing its identity/version through a newly versioned aggregate, while an unchanged snapshot is reused when only the settings revision changes. A schema-v1 database migrates forward without inserting resource or numeric rows, and prior settings payloads without the optional bundle remain readable. Migration SQL is ordered and checksummed; foreign keys, WAL, full synchronous durability, a bounded busy timeout, defensive trusted-schema configuration, integrity-on-open, consistent backup, save/reopen, historical replay, and stale-writer rejection are executable evidence. A new database contains no rate, policy, resource, or numeric seed rows.

This is a preproduction persistence foundation, not automatic calculation authority and not TASK-006 completion. Desktop contract v6 preserves the seven-command v5 set and loads/saves the USD rate/pricing draft plus the optional starter resource bundle through `ShopSettingsApplication`; the Tauri host owns the application-data directory and SQLite lifecycle, and the WebView receives no path or database authority. Reopened rate/pricing values clear session confirmations, and reopened resource values clear their draft confirmation. Additional catalog entries, approval/activation, model/estimate persistence, attachments, retention/disposition, encryption/key management, crash-recovery fixtures, a second prior-version migration, three-OS backup evidence, and content-store atomicity remain open. Unknown newer versions or altered migration checksums fail closed.
