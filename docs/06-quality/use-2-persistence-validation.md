# USE-2 Persistence Foundation Validation

> **Status:** In Review
> **Last updated:** 2026-09-09
> **Related requirements:** REQ-F-015, REQ-F-016, REQ-F-032; REQ-NF-007, REQ-NF-009, REQ-NF-020; DATA-005, DATA-006, DATA-008, DATA-011, DATA-017; TEST-007, TEST-014
> **Related ADRs:** ADR-0006, ADR-0007
> **Open questions:** Protected-storage policy, later aggregate migrations, and three-OS evidence
> **Dependencies:** `crates/domain`, `crates/application`, `crates/persistence-sqlite`, `crates/desktop-contract`, `apps/estimator-desktop`
> **Supersedes:** None

## Evidence boundary

This checkpoint establishes and activates the first durable USE-2 foundation. `ShopSettingsDraft` validates profile identity, positive revision, currency-compatible optional rate/pricing/resource snapshots, and actor/time/reason evidence without creating numeric defaults. `ShopSettingsApplication` preserves explicit `NotConfigured` first-run state and is the desktop entry; `ShopSettingsDraftRepository` keeps persistence behind it at the application boundary. The SQLite adapter implements immutable snapshots and revisions, an append-preserving change event, one mutable optimistic-concurrency pointer, schema/hash validation on read, and historical revision lookup.

Desktop contract v5 preserved all five GUI-4 analysis/estimate commands and added only `load_shop_settings` and `save_shop_settings`. Contract v6 keeps that exact seven-command set and adds an optional path-free Settings resource DTO. The Tauri host resolves the database beneath its platform application-data directory, opens it during setup, executes storage work off the UI thread, and exposes only path-free DTOs. The WebView never receives a database path or SQLite handle. First run remains visibly `not_configured`; save requires rate/pricing confirmations plus actor and reason, and any included starter resource bundle requires its own draft confirmation. Reopen restores exact domain values but deliberately clears every confirmation so persistence is not silently activated as estimate authority.

Schema v1 is the empty-version-0 bootstrap. Schema v2 adds immutable snapshot tables for the bounded `ShopResourceLibrary` aggregate and each typed child record, plus one immutable settings-revision reference table. The aggregate retains separately versioned material identity and supplier offer, stock-form/allowance policy, machine capability, and coarse runtime profile with typed units, source evidence, draft lifecycle, exact currency, and material/machine cross-references. Child identity/version immutability is checked independently of the aggregate version, and unchanged rate/pricing/resource evidence is reused when a later settings revision changes only its audit reason or another layer. A schema-v1 database migrates forward without seeding a record, while the additive optional payload field keeps prior v1 settings JSON readable. Every embedded migration is checksummed and recorded; a changed checksum, missing expected migration, or unknown newer schema fails closed. Every opened connection explicitly enables foreign keys, WAL, full synchronous durability, a five-second busy timeout, defensive trusted-schema behavior, and an integrity check.

Focused automated tests prove:

- a new database has no rate card, pricing policy, resource library, or current settings row;
- validated rate/pricing drafts survive save, close, reopen, and exact historical replay;
- two independently opened writers cannot overwrite a newer current revision;
- reusing an immutable policy ID/version with different bytes is rejected without changing current state;
- a consistent SQLite backup reopens with current and historical revisions; and
- a schema-v1 database migrates to v2 with no numeric/resource defaults, while an unknown newer schema or altered migration checksum is rejected;
- the host adapter returns first-run absence, appends a confirmed USD revision, rejects stale and reused-version writers, reopens the same durable values, clears confirmations on load, and preserves optional missing libraries as explicit absence;
- a complete synthetic resource bundle survives host save/reopen with material/offer/stock/machine/runtime distinctions intact, unchanged snapshots can be reused by a new settings revision, and changed child bytes under a reused child version are rejected even when the aggregate version changes; and
- the shared contract, generated application manifest, exact main-window permissions, and Tauri handler contain the same seven-command allowlist.

All filesystem-backed database handles are explicitly dropped before the repository test helper removes its owned temporary tree, preserving Windows sharing semantics.

## Not established

This does not make stored drafts automatic calculation authority or satisfy TASK-006/USE-2. The user must still review and confirm loaded rates/pricing for the current session; the starter resource bundle remains lifecycle `Draft` and is not consumed by calculation. Estimates/models remain unsaved. Multiple catalog entries, activation/approval, material selection, stock proposal, route/runtime proposal, document blobs, crash/power-loss recovery, corruption fixtures beyond integrity/hash checks, a second supported prior-version migration, retention/disposition, encryption/key management, protected-storage threat review, and three-OS backup/package evidence remain open. All calculation and geometry rules are unchanged.
