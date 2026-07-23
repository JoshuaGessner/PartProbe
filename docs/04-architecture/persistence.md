# Persistence Architecture

> **Status:** In Review  
> **Last updated:** 2026-07-22  
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
