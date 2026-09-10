//! SQLite adapter for durable, immutable shop-settings drafts.

use std::fmt::Write as _;
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::path::Path;
use std::time::Duration;

use partprobe_application::{
    ShopResourceCatalogAuthorizationAuditSink, ShopResourceCatalogAuthorizationEvent,
    ShopResourceCatalogOperation, ShopSettingsDraftRepository, ShopSettingsStoreError,
};
use partprobe_domain::{
    ActorId, PricingPolicy, RateCard, RecordedAt, ResourceSelection, ResourceSelectionId,
    ResourceSelectionState, SHOP_RESOURCE_CATALOG_SCHEMA, ShopProfileId, ShopResourceCatalog,
    ShopResourceCatalogId, ShopResourceLibrary, ShopResourceVersion, ShopSettingsDraft,
    ShopSettingsRevision,
};
use partprobe_security::{
    AuditAppendError, AuditCorrelationId, AuthorizationOutcome, AuthorizationReasonCode,
    SecurityPolicyId, SecurityPolicyVersion,
};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use sha2::{Digest, Sha256};

const CURRENT_SCHEMA_VERSION: u32 = 4;
const MIGRATIONS: [(u32, &str, &str); 4] = [
    (
        1,
        "0001_shop_settings",
        include_str!("../migrations/0001_shop_settings.sql"),
    ),
    (
        2,
        "0002_shop_resources",
        include_str!("../migrations/0002_shop_resources.sql"),
    ),
    (
        3,
        "0003_shop_resource_catalog",
        include_str!("../migrations/0003_shop_resource_catalog.sql"),
    ),
    (
        4,
        "0004_catalog_authorization_audit",
        include_str!("../migrations/0004_catalog_authorization_audit.sql"),
    ),
];

const CATALOG_AUTHORIZATION_EVENT_SCHEMA: &str = "shop-resource-catalog-authorization-event-v1";

type StoredDraftRow = (
    String,
    u32,
    String,
    Option<String>,
    Option<u32>,
    Option<String>,
    Option<u32>,
    String,
    String,
    String,
    String,
    String,
);

struct StoredCatalogAuthorizationEvent {
    correlation_id: String,
    profile_id: String,
    settings_revision: u32,
    catalog_id: String,
    catalog_version: u32,
    selection_id: String,
    selection_version: u32,
    actor_id: String,
    recorded_at: String,
    operation: String,
    outcome: String,
    policy_id: String,
    policy_version: i64,
    reason_code: String,
    event_schema: String,
    event_sha256: String,
    event_json: String,
}

/// SQLite-backed implementation of the application shop-settings port.
pub struct SqliteShopSettingsRepository {
    connection: Connection,
}

/// Durable append-only adapter for content-minimized catalog authorization decisions.
pub struct SqliteShopResourceCatalogAuthorizationAudit {
    connection: Connection,
}

impl SqliteShopSettingsRepository {
    /// Opens or creates a database, configures durability, and verifies its migration history.
    pub fn open(path: &Path) -> Result<Self, ShopSettingsStoreError> {
        open_connection(path).map(|connection| Self { connection })
    }

    /// Copies a consistent snapshot to a new file without overwriting an existing backup.
    pub fn backup_to(&self, destination: &Path) -> Result<(), ShopSettingsStoreError> {
        let reservation = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)
            .map_err(|error| {
                if error.kind() == ErrorKind::AlreadyExists {
                    ShopSettingsStoreError::IntegrityConflict
                } else {
                    ShopSettingsStoreError::Unavailable
                }
            })?;
        drop(reservation);
        match self.connection.backup("main", destination, None) {
            Ok(()) => Ok(()),
            Err(_) => {
                let _ = std::fs::remove_file(destination);
                Err(ShopSettingsStoreError::Unavailable)
            }
        }
    }
}

impl SqliteShopResourceCatalogAuthorizationAudit {
    /// Opens the same host-owned database used by shop Settings and verifies all prior events.
    pub fn open(path: &Path) -> Result<Self, ShopSettingsStoreError> {
        open_connection(path).map(|connection| Self { connection })
    }
}

impl ShopResourceCatalogAuthorizationAuditSink for SqliteShopResourceCatalogAuthorizationAudit {
    fn append(&self, event: ShopResourceCatalogAuthorizationEvent) -> Result<(), AuditAppendError> {
        let context = event.context();
        let decision = event.decision();
        let operation = catalog_operation(context.operation());
        let outcome = authorization_outcome(decision.outcome());
        let policy_version =
            i64::try_from(decision.policy().version().value()).map_err(|_| AuditAppendError)?;
        let payload = catalog_authorization_payload(
            context.correlation_id().as_str(),
            context.profile_id().as_str(),
            context.settings_revision().value(),
            context.catalog_id().as_str(),
            context.catalog_version().value(),
            context.selection_id().as_str(),
            context.selection_version().value(),
            context.actor_id().as_str(),
            context.recorded_at().as_str(),
            operation,
            outcome,
            decision.policy().id().as_str(),
            decision.policy().version().value(),
            decision.reason_code().as_str(),
        )
        .map_err(|_| AuditAppendError)?;
        let payload_hash = sha256(payload.as_bytes());
        let inserted = self
            .connection
            .execute(
                "INSERT OR IGNORE INTO shop_resource_catalog_authorization_events (
                    correlation_id, profile_id, settings_revision,
                    catalog_id, catalog_version, selection_id, selection_version,
                    actor_id, recorded_at, operation, outcome,
                    policy_id, policy_version, reason_code,
                    event_schema, event_sha256, event_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                           ?12, ?13, ?14, ?15, ?16, ?17)",
                params![
                    context.correlation_id().as_str(),
                    context.profile_id().as_str(),
                    context.settings_revision().value(),
                    context.catalog_id().as_str(),
                    context.catalog_version().value(),
                    context.selection_id().as_str(),
                    context.selection_version().value(),
                    context.actor_id().as_str(),
                    context.recorded_at().as_str(),
                    operation,
                    outcome,
                    decision.policy().id().as_str(),
                    policy_version,
                    decision.reason_code().as_str(),
                    CATALOG_AUTHORIZATION_EVENT_SCHEMA,
                    payload_hash,
                    payload,
                ],
            )
            .map_err(|_| AuditAppendError)?;
        if inserted == 1 {
            return Ok(());
        }

        verify_catalog_authorization_events(&self.connection).map_err(|_| AuditAppendError)?;
        let existing = self
            .connection
            .query_row(
                "SELECT event_sha256, event_json
                 FROM shop_resource_catalog_authorization_events
                 WHERE correlation_id = ?1",
                [context.correlation_id().as_str()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|_| AuditAppendError)?
            .ok_or(AuditAppendError)?;
        if existing == (payload_hash, payload) {
            Ok(())
        } else {
            Err(AuditAppendError)
        }
    }
}

impl ShopSettingsDraftRepository for SqliteShopSettingsRepository {
    fn save(
        &mut self,
        draft: &ShopSettingsDraft,
        expected_current_revision: Option<ShopSettingsRevision>,
    ) -> Result<(), ShopSettingsStoreError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| ShopSettingsStoreError::Unavailable)?;

        let current = current_revision(&transaction, draft.profile_id())?;
        if current != expected_current_revision {
            return Err(ShopSettingsStoreError::RevisionConflict);
        }
        let expected_next = current
            .map(|revision| revision.value().checked_add(1))
            .unwrap_or(Some(1))
            .ok_or(ShopSettingsStoreError::IntegrityConflict)?;
        if draft.revision().value() != expected_next {
            return Err(ShopSettingsStoreError::RevisionConflict);
        }

        let rate_reference = draft
            .rate_card()
            .map(|card| persist_rate_card(&transaction, card))
            .transpose()?;
        let pricing_reference = draft
            .pricing_policy()
            .map(|policy| persist_pricing_policy(&transaction, policy))
            .transpose()?;
        let resource_reference = draft
            .resource_library()
            .map(|library| persist_resource_library(&transaction, library))
            .transpose()?;
        let catalog_reference = draft
            .resource_catalog()
            .map(|catalog| persist_resource_catalog(&transaction, catalog))
            .transpose()?;
        let payload = serde_json::to_string(draft)
            .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
        let payload_hash = sha256(payload.as_bytes());

        transaction
            .execute(
                "INSERT INTO shop_settings_revisions (
                    profile_id, revision, currency,
                    rate_card_id, rate_card_version,
                    pricing_policy_id, pricing_policy_version,
                    changed_by, changed_at, change_reason,
                    payload_schema, payload_sha256, payload_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                           'shop-settings-draft-v1', ?11, ?12)",
                params![
                    draft.profile_id().as_str(),
                    draft.revision().value(),
                    draft.currency().as_str(),
                    rate_reference.as_ref().map(|value| value.0.as_str()),
                    rate_reference.as_ref().map(|value| value.1),
                    pricing_reference.as_ref().map(|value| value.0.as_str()),
                    pricing_reference.as_ref().map(|value| value.1),
                    draft.changed_by().as_str(),
                    draft.changed_at().as_str(),
                    draft.change_reason(),
                    payload_hash,
                    payload,
                ],
            )
            .map_err(|_| ShopSettingsStoreError::IntegrityConflict)?;
        if let Some((library_id, library_version)) = resource_reference {
            transaction
                .execute(
                    "INSERT INTO shop_settings_resource_refs
                        (profile_id, revision, library_id, library_version)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![
                        draft.profile_id().as_str(),
                        draft.revision().value(),
                        library_id,
                        library_version,
                    ],
                )
                .map_err(|_| ShopSettingsStoreError::IntegrityConflict)?;
        }
        if let Some((catalog_id, catalog_version)) = catalog_reference {
            transaction
                .execute(
                    "INSERT INTO shop_settings_catalog_refs
                        (profile_id, revision, catalog_id, catalog_version)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![
                        draft.profile_id().as_str(),
                        draft.revision().value(),
                        catalog_id,
                        catalog_version,
                    ],
                )
                .map_err(|_| ShopSettingsStoreError::IntegrityConflict)?;
        }
        transaction
            .execute(
                "INSERT INTO settings_change_events
                    (profile_id, revision, changed_by, changed_at, reason)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    draft.profile_id().as_str(),
                    draft.revision().value(),
                    draft.changed_by().as_str(),
                    draft.changed_at().as_str(),
                    draft.change_reason(),
                ],
            )
            .map_err(|_| ShopSettingsStoreError::IntegrityConflict)?;
        transaction
            .execute(
                "INSERT INTO shop_settings_current (profile_id, revision) VALUES (?1, ?2)
                 ON CONFLICT(profile_id) DO UPDATE SET revision = excluded.revision",
                params![draft.profile_id().as_str(), draft.revision().value()],
            )
            .map_err(|_| ShopSettingsStoreError::IntegrityConflict)?;
        transaction
            .commit()
            .map_err(|_| ShopSettingsStoreError::Unavailable)
    }

    fn current(
        &self,
        profile_id: &ShopProfileId,
    ) -> Result<Option<ShopSettingsDraft>, ShopSettingsStoreError> {
        let revision = self
            .connection
            .query_row(
                "SELECT revision FROM shop_settings_current WHERE profile_id = ?1",
                [profile_id.as_str()],
                |row| row.get::<_, u32>(0),
            )
            .optional()
            .map_err(|_| ShopSettingsStoreError::Unavailable)?;
        revision
            .map(|value| {
                let revision = ShopSettingsRevision::new(value)
                    .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
                self.revision(profile_id, revision)?
                    .ok_or(ShopSettingsStoreError::InvalidStoredRecord)
            })
            .transpose()
    }

    fn revision(
        &self,
        profile_id: &ShopProfileId,
        revision: ShopSettingsRevision,
    ) -> Result<Option<ShopSettingsDraft>, ShopSettingsStoreError> {
        let row: Option<StoredDraftRow> = self
            .connection
            .query_row(
                "SELECT profile_id, revision, currency,
                        rate_card_id, rate_card_version,
                        pricing_policy_id, pricing_policy_version,
                        changed_by, changed_at, change_reason,
                        payload_sha256, payload_json
                 FROM shop_settings_revisions
                 WHERE profile_id = ?1 AND revision = ?2",
                params![profile_id.as_str(), revision.value()],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                        row.get(10)?,
                        row.get(11)?,
                    ))
                },
            )
            .optional()
            .map_err(|_| ShopSettingsStoreError::Unavailable)?;
        row.map(|stored| {
            let draft = validate_stored_draft(profile_id, revision, stored)?;
            validate_referenced_snapshots(&self.connection, &draft)?;
            Ok(draft)
        })
        .transpose()
    }
}

fn open_connection(path: &Path) -> Result<Connection, ShopSettingsStoreError> {
    let connection = Connection::open(path).map_err(|_| ShopSettingsStoreError::Unavailable)?;
    configure(&connection)?;
    migrate(&connection)?;
    verify_integrity(&connection)?;
    verify_catalog_authorization_events(&connection)?;
    Ok(connection)
}

fn configure(connection: &Connection) -> Result<(), ShopSettingsStoreError> {
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .and_then(|()| connection.pragma_update(None, "journal_mode", "WAL"))
        .and_then(|()| connection.pragma_update(None, "synchronous", "FULL"))
        .and_then(|()| connection.pragma_update(None, "trusted_schema", "OFF"))
        .map_err(|_| ShopSettingsStoreError::Unavailable)?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|_| ShopSettingsStoreError::Unavailable)
}

fn migrate(connection: &Connection) -> Result<(), ShopSettingsStoreError> {
    let user_version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| ShopSettingsStoreError::Unavailable)?;
    if user_version > CURRENT_SCHEMA_VERSION {
        return Err(ShopSettingsStoreError::UnsupportedSchema);
    }
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY CHECK (version > 0),
                name TEXT NOT NULL UNIQUE,
                checksum_sha256 TEXT NOT NULL CHECK (length(checksum_sha256) = 64)
             ) STRICT;",
        )
        .map_err(|_| ShopSettingsStoreError::Unavailable)?;

    let migration_count: u32 = connection
        .query_row("SELECT count(*) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .map_err(|_| ShopSettingsStoreError::Unavailable)?;
    if migration_count != user_version {
        return Err(ShopSettingsStoreError::UnsupportedSchema);
    }

    for (version, name, sql) in MIGRATIONS {
        let expected_hash = sha256(sql.as_bytes());
        if version <= user_version {
            let existing = connection
                .query_row(
                    "SELECT name, checksum_sha256 FROM schema_migrations WHERE version = ?1",
                    [version],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                )
                .optional()
                .map_err(|_| ShopSettingsStoreError::Unavailable)?;
            match existing {
                Some((stored_name, checksum))
                    if stored_name == name && checksum == expected_hash => {}
                _ => return Err(ShopSettingsStoreError::UnsupportedSchema),
            }
            continue;
        }

        let transaction = connection
            .unchecked_transaction()
            .map_err(|_| ShopSettingsStoreError::Unavailable)?;
        transaction
            .execute_batch(sql)
            .map_err(|_| ShopSettingsStoreError::UnsupportedSchema)?;
        transaction
            .execute(
                "INSERT INTO schema_migrations (version, name, checksum_sha256)
                 VALUES (?1, ?2, ?3)",
                params![version, name, expected_hash],
            )
            .map_err(|_| ShopSettingsStoreError::UnsupportedSchema)?;
        transaction
            .pragma_update(None, "user_version", version)
            .map_err(|_| ShopSettingsStoreError::Unavailable)?;
        transaction
            .commit()
            .map_err(|_| ShopSettingsStoreError::Unavailable)?;
    }
    Ok(())
}

fn verify_integrity(connection: &Connection) -> Result<(), ShopSettingsStoreError> {
    let result: String = connection
        .pragma_query_value(None, "integrity_check", |row| row.get(0))
        .map_err(|_| ShopSettingsStoreError::Unavailable)?;
    if result == "ok" {
        Ok(())
    } else {
        Err(ShopSettingsStoreError::InvalidStoredRecord)
    }
}

fn verify_catalog_authorization_events(
    connection: &Connection,
) -> Result<(), ShopSettingsStoreError> {
    let mut statement = connection
        .prepare(
            "SELECT correlation_id, profile_id, settings_revision,
                    catalog_id, catalog_version, selection_id, selection_version,
                    actor_id, recorded_at, operation, outcome,
                    policy_id, policy_version, reason_code,
                    event_schema, event_sha256, event_json
             FROM shop_resource_catalog_authorization_events",
        )
        .map_err(|_| ShopSettingsStoreError::Unavailable)?;
    let events = statement
        .query_map([], |row| {
            Ok(StoredCatalogAuthorizationEvent {
                correlation_id: row.get(0)?,
                profile_id: row.get(1)?,
                settings_revision: row.get(2)?,
                catalog_id: row.get(3)?,
                catalog_version: row.get(4)?,
                selection_id: row.get(5)?,
                selection_version: row.get(6)?,
                actor_id: row.get(7)?,
                recorded_at: row.get(8)?,
                operation: row.get(9)?,
                outcome: row.get(10)?,
                policy_id: row.get(11)?,
                policy_version: row.get(12)?,
                reason_code: row.get(13)?,
                event_schema: row.get(14)?,
                event_sha256: row.get(15)?,
                event_json: row.get(16)?,
            })
        })
        .map_err(|_| ShopSettingsStoreError::Unavailable)?;
    for event in events {
        let event = event.map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
        validate_catalog_authorization_event(&event)?;
        let expected_payload = catalog_authorization_payload(
            &event.correlation_id,
            &event.profile_id,
            event.settings_revision,
            &event.catalog_id,
            event.catalog_version,
            &event.selection_id,
            event.selection_version,
            &event.actor_id,
            &event.recorded_at,
            &event.operation,
            &event.outcome,
            &event.policy_id,
            u64::try_from(event.policy_version)
                .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
            &event.reason_code,
        )?;
        if event.event_schema != CATALOG_AUTHORIZATION_EVENT_SCHEMA
            || event.event_sha256 != sha256(expected_payload.as_bytes())
            || event.event_json != expected_payload
        {
            return Err(ShopSettingsStoreError::InvalidStoredRecord);
        }
    }
    Ok(())
}

fn validate_catalog_authorization_event(
    event: &StoredCatalogAuthorizationEvent,
) -> Result<(), ShopSettingsStoreError> {
    AuditCorrelationId::new(&event.correlation_id)
        .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    ShopProfileId::new(&event.profile_id)
        .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    ShopSettingsRevision::new(event.settings_revision)
        .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    ShopResourceCatalogId::new(&event.catalog_id)
        .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    ShopResourceVersion::new(event.catalog_version)
        .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    ResourceSelectionId::new(&event.selection_id)
        .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    ShopResourceVersion::new(event.selection_version)
        .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    ActorId::new(&event.actor_id).map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    RecordedAt::new(&event.recorded_at).map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    SecurityPolicyId::new(&event.policy_id)
        .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    SecurityPolicyVersion::new(
        u64::try_from(event.policy_version)
            .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
    )
    .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    AuthorizationReasonCode::new(&event.reason_code)
        .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    if event.operation != "activate_for_proposals"
        || !matches!(event.outcome.as_str(), "allowed" | "denied")
    {
        return Err(ShopSettingsStoreError::InvalidStoredRecord);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn catalog_authorization_payload(
    correlation_id: &str,
    profile_id: &str,
    settings_revision: u32,
    catalog_id: &str,
    catalog_version: u32,
    selection_id: &str,
    selection_version: u32,
    actor_id: &str,
    recorded_at: &str,
    operation: &str,
    outcome: &str,
    policy_id: &str,
    policy_version: u64,
    reason_code: &str,
) -> Result<String, ShopSettingsStoreError> {
    serde_json::to_string(&serde_json::json!({
        "actor_id": actor_id,
        "catalog_id": catalog_id,
        "catalog_version": catalog_version,
        "correlation_id": correlation_id,
        "event_schema": CATALOG_AUTHORIZATION_EVENT_SCHEMA,
        "operation": operation,
        "outcome": outcome,
        "policy_id": policy_id,
        "policy_version": policy_version,
        "profile_id": profile_id,
        "reason_code": reason_code,
        "recorded_at": recorded_at,
        "selection_id": selection_id,
        "selection_version": selection_version,
        "settings_revision": settings_revision,
    }))
    .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)
}

const fn catalog_operation(operation: ShopResourceCatalogOperation) -> &'static str {
    match operation {
        ShopResourceCatalogOperation::ActivateForProposals => "activate_for_proposals",
    }
}

const fn authorization_outcome(outcome: AuthorizationOutcome) -> &'static str {
    match outcome {
        AuthorizationOutcome::Allowed => "allowed",
        AuthorizationOutcome::Denied => "denied",
    }
}

fn current_revision(
    transaction: &Transaction<'_>,
    profile_id: &ShopProfileId,
) -> Result<Option<ShopSettingsRevision>, ShopSettingsStoreError> {
    let stored = transaction
        .query_row(
            "SELECT revision FROM shop_settings_current WHERE profile_id = ?1",
            [profile_id.as_str()],
            |row| row.get::<_, u32>(0),
        )
        .optional()
        .map_err(|_| ShopSettingsStoreError::Unavailable)?;
    stored
        .map(|value| {
            ShopSettingsRevision::new(value)
                .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)
        })
        .transpose()
}

fn persist_rate_card(
    transaction: &Transaction<'_>,
    card: &RateCard,
) -> Result<(String, u32), ShopSettingsStoreError> {
    let payload =
        serde_json::to_string(card).map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    persist_snapshot(
        transaction,
        "rate_card_snapshots",
        "card_id",
        card.id().as_str(),
        card.version().value(),
        card.currency().as_str(),
        "rate-card-v1",
        &payload,
    )?;
    Ok((card.id().as_str().to_owned(), card.version().value()))
}

fn persist_pricing_policy(
    transaction: &Transaction<'_>,
    policy: &PricingPolicy,
) -> Result<(String, u32), ShopSettingsStoreError> {
    let payload =
        serde_json::to_string(policy).map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    persist_snapshot(
        transaction,
        "pricing_policy_snapshots",
        "policy_id",
        policy.id().as_str(),
        policy.version().value(),
        policy.currency().as_str(),
        "pricing-policy-v1",
        &payload,
    )?;
    Ok((policy.id().as_str().to_owned(), policy.version().value()))
}

fn persist_resource_library(
    transaction: &Transaction<'_>,
    library: &ShopResourceLibrary,
) -> Result<(String, u32), ShopSettingsStoreError> {
    persist_resource_record(
        transaction,
        "material",
        library.material().id().as_str(),
        library.material().version().value(),
        "material-definition-v1",
        &serde_json::to_string(library.material())
            .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
    )?;
    persist_resource_record(
        transaction,
        "material_offer",
        library.material_offer().id().as_str(),
        library.material_offer().version().value(),
        "material-offer-v1",
        &serde_json::to_string(library.material_offer())
            .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
    )?;
    persist_resource_record(
        transaction,
        "stock_allowance",
        library.stock_allowance().id().as_str(),
        library.stock_allowance().version().value(),
        "stock-allowance-v1",
        &serde_json::to_string(library.stock_allowance())
            .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
    )?;
    persist_resource_record(
        transaction,
        "machine",
        library.machine().id().as_str(),
        library.machine().version().value(),
        "machine-profile-v1",
        &serde_json::to_string(library.machine())
            .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
    )?;
    persist_resource_record(
        transaction,
        "runtime",
        library.runtime().id().as_str(),
        library.runtime().version().value(),
        "coarse-runtime-profile-v1",
        &serde_json::to_string(library.runtime())
            .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
    )?;
    let payload =
        serde_json::to_string(library).map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    persist_snapshot(
        transaction,
        "shop_resource_library_snapshots",
        "library_id",
        library.id().as_str(),
        library.version().value(),
        library.currency().as_str(),
        "shop-resource-library-v1",
        &payload,
    )?;
    Ok((library.id().as_str().to_owned(), library.version().value()))
}

fn persist_resource_catalog(
    transaction: &Transaction<'_>,
    catalog: &ShopResourceCatalog,
) -> Result<(String, u32), ShopSettingsStoreError> {
    for material in catalog.materials() {
        persist_resource_record(
            transaction,
            "material",
            material.id().as_str(),
            material.version().value(),
            "material-definition-v1",
            &serde_json::to_string(material)
                .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
        )?;
    }
    for offer in catalog.material_offers() {
        persist_resource_record(
            transaction,
            "material_offer",
            offer.id().as_str(),
            offer.version().value(),
            "material-offer-v1",
            &serde_json::to_string(offer)
                .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
        )?;
    }
    for stock in catalog.stock_allowances() {
        persist_resource_record(
            transaction,
            "stock_allowance",
            stock.id().as_str(),
            stock.version().value(),
            "stock-allowance-v1",
            &serde_json::to_string(stock)
                .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
        )?;
    }
    for machine in catalog.machines() {
        persist_resource_record(
            transaction,
            "machine",
            machine.id().as_str(),
            machine.version().value(),
            "machine-profile-v1",
            &serde_json::to_string(machine)
                .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
        )?;
    }
    for runtime in catalog.runtimes() {
        persist_resource_record(
            transaction,
            "runtime",
            runtime.id().as_str(),
            runtime.version().value(),
            "coarse-runtime-profile-v1",
            &serde_json::to_string(runtime)
                .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
        )?;
    }
    for selection in catalog.selections() {
        persist_resource_selection(transaction, selection)?;
    }
    let payload =
        serde_json::to_string(catalog).map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    persist_snapshot(
        transaction,
        "shop_resource_catalog_snapshots",
        "catalog_id",
        catalog.id().as_str(),
        catalog.version().value(),
        catalog.currency().as_str(),
        SHOP_RESOURCE_CATALOG_SCHEMA,
        &payload,
    )?;
    Ok((catalog.id().as_str().to_owned(), catalog.version().value()))
}

fn persist_resource_selection(
    transaction: &Transaction<'_>,
    selection: &ResourceSelection,
) -> Result<(), ShopSettingsStoreError> {
    let payload = serde_json::to_string(selection)
        .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    let payload_hash = sha256(payload.as_bytes());
    let state = resource_selection_state(selection.state());
    let existing = transaction
        .query_row(
            "SELECT selection_state, decided_by, decided_at, reason,
                    payload_schema, payload_sha256, payload_json
             FROM shop_resource_selection_snapshots
             WHERE selection_id = ?1 AND version = ?2",
            params![selection.id().as_str(), selection.version().value()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            },
        )
        .optional()
        .map_err(|_| ShopSettingsStoreError::Unavailable)?;
    let expected = (
        state,
        selection.decided_by().as_str(),
        selection.decided_at().as_str(),
        selection.reason(),
        "resource-selection-v1",
        payload_hash.as_str(),
        payload.as_str(),
    );
    if let Some(stored) = existing {
        let stored = (
            stored.0.as_str(),
            stored.1.as_str(),
            stored.2.as_str(),
            stored.3.as_str(),
            stored.4.as_str(),
            stored.5.as_str(),
            stored.6.as_str(),
        );
        return if stored == expected {
            Ok(())
        } else {
            Err(ShopSettingsStoreError::IntegrityConflict)
        };
    }
    transaction
        .execute(
            "INSERT INTO shop_resource_selection_snapshots
                (selection_id, version, selection_state, decided_by, decided_at, reason,
                 payload_schema, payload_sha256, payload_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'resource-selection-v1', ?7, ?8)",
            params![
                selection.id().as_str(),
                selection.version().value(),
                state,
                selection.decided_by().as_str(),
                selection.decided_at().as_str(),
                selection.reason(),
                payload_hash,
                payload,
            ],
        )
        .map_err(|_| ShopSettingsStoreError::IntegrityConflict)?;
    Ok(())
}

const fn resource_selection_state(state: ResourceSelectionState) -> &'static str {
    match state {
        ResourceSelectionState::Reviewed => "reviewed",
        ResourceSelectionState::ActiveForProposals => "active_for_proposals",
        ResourceSelectionState::Retired => "retired",
    }
}

fn persist_resource_record(
    transaction: &Transaction<'_>,
    kind: &str,
    id: &str,
    version: u32,
    schema: &str,
    payload: &str,
) -> Result<(), ShopSettingsStoreError> {
    let payload_hash = sha256(payload.as_bytes());
    let existing = transaction
        .query_row(
            "SELECT payload_schema, payload_sha256, payload_json
             FROM shop_resource_record_snapshots
             WHERE record_kind = ?1 AND record_id = ?2 AND version = ?3",
            params![kind, id, version],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()
        .map_err(|_| ShopSettingsStoreError::Unavailable)?;
    if let Some((stored_schema, stored_hash, stored_payload)) = existing {
        return if stored_schema == schema
            && stored_hash == payload_hash
            && stored_payload == payload
        {
            Ok(())
        } else {
            Err(ShopSettingsStoreError::IntegrityConflict)
        };
    }
    transaction
        .execute(
            "INSERT INTO shop_resource_record_snapshots
                (record_kind, record_id, version, payload_schema, payload_sha256, payload_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![kind, id, version, schema, payload_hash, payload],
        )
        .map_err(|_| ShopSettingsStoreError::IntegrityConflict)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn persist_snapshot(
    transaction: &Transaction<'_>,
    table: &str,
    id_column: &str,
    id: &str,
    version: u32,
    currency: &str,
    schema: &str,
    payload: &str,
) -> Result<(), ShopSettingsStoreError> {
    let payload_hash = sha256(payload.as_bytes());
    let query = format!(
        "SELECT currency, payload_schema, payload_sha256, payload_json
         FROM {table} WHERE {id_column} = ?1 AND version = ?2"
    );
    let existing = transaction
        .query_row(&query, params![id, version], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .optional()
        .map_err(|_| ShopSettingsStoreError::Unavailable)?;
    if let Some((existing_currency, existing_schema, existing_hash, existing_payload)) = existing {
        return if existing_currency == currency
            && existing_schema == schema
            && existing_hash == payload_hash
            && existing_payload == payload
        {
            Ok(())
        } else {
            Err(ShopSettingsStoreError::IntegrityConflict)
        };
    }
    let insert = format!(
        "INSERT INTO {table}
            ({id_column}, version, currency, payload_schema, payload_sha256, payload_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
    );
    transaction
        .execute(
            &insert,
            params![id, version, currency, schema, payload_hash, payload],
        )
        .map_err(|_| ShopSettingsStoreError::IntegrityConflict)?;
    Ok(())
}

fn validate_stored_draft(
    profile_id: &ShopProfileId,
    revision: ShopSettingsRevision,
    stored: StoredDraftRow,
) -> Result<ShopSettingsDraft, ShopSettingsStoreError> {
    let (
        stored_profile,
        stored_revision,
        currency,
        card_id,
        card_version,
        policy_id,
        policy_version,
        changed_by,
        changed_at,
        change_reason,
        hash,
        payload,
    ) = stored;
    if sha256(payload.as_bytes()) != hash {
        return Err(ShopSettingsStoreError::InvalidStoredRecord);
    }
    let draft: ShopSettingsDraft =
        serde_json::from_str(&payload).map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    let draft_card = draft
        .rate_card()
        .map(|card| (card.id().as_str(), card.version().value()));
    let draft_policy = draft
        .pricing_policy()
        .map(|policy| (policy.id().as_str(), policy.version().value()));
    if stored_profile != profile_id.as_str()
        || stored_revision != revision.value()
        || currency != draft.currency().as_str()
        || draft.profile_id() != profile_id
        || draft.revision() != revision
        || changed_by != draft.changed_by().as_str()
        || changed_at != draft.changed_at().as_str()
        || change_reason != draft.change_reason()
        || draft_card != card_id.as_deref().zip(card_version)
        || draft_policy != policy_id.as_deref().zip(policy_version)
    {
        return Err(ShopSettingsStoreError::InvalidStoredRecord);
    }
    Ok(draft)
}

fn validate_referenced_snapshots(
    connection: &Connection,
    draft: &ShopSettingsDraft,
) -> Result<(), ShopSettingsStoreError> {
    if let Some(card) = draft.rate_card() {
        let payload =
            serde_json::to_string(card).map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
        validate_snapshot(
            connection,
            "rate_card_snapshots",
            "card_id",
            card.id().as_str(),
            card.version().value(),
            card.currency().as_str(),
            "rate-card-v1",
            &payload,
        )?;
    }
    if let Some(policy) = draft.pricing_policy() {
        let payload = serde_json::to_string(policy)
            .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
        validate_snapshot(
            connection,
            "pricing_policy_snapshots",
            "policy_id",
            policy.id().as_str(),
            policy.version().value(),
            policy.currency().as_str(),
            "pricing-policy-v1",
            &payload,
        )?;
    }
    match draft.resource_library() {
        Some(library) => {
            let reference = connection
                .query_row(
                    "SELECT library_id, library_version FROM shop_settings_resource_refs
                     WHERE profile_id = ?1 AND revision = ?2",
                    params![draft.profile_id().as_str(), draft.revision().value()],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?)),
                )
                .optional()
                .map_err(|_| ShopSettingsStoreError::Unavailable)?
                .ok_or(ShopSettingsStoreError::InvalidStoredRecord)?;
            if reference != (library.id().as_str().to_owned(), library.version().value()) {
                return Err(ShopSettingsStoreError::InvalidStoredRecord);
            }
            let payload = serde_json::to_string(library)
                .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
            validate_snapshot(
                connection,
                "shop_resource_library_snapshots",
                "library_id",
                library.id().as_str(),
                library.version().value(),
                library.currency().as_str(),
                "shop-resource-library-v1",
                &payload,
            )?;
            validate_resource_record(
                connection,
                "material",
                library.material().id().as_str(),
                library.material().version().value(),
                "material-definition-v1",
                &serde_json::to_string(library.material())
                    .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
            )?;
            validate_resource_record(
                connection,
                "material_offer",
                library.material_offer().id().as_str(),
                library.material_offer().version().value(),
                "material-offer-v1",
                &serde_json::to_string(library.material_offer())
                    .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
            )?;
            validate_resource_record(
                connection,
                "stock_allowance",
                library.stock_allowance().id().as_str(),
                library.stock_allowance().version().value(),
                "stock-allowance-v1",
                &serde_json::to_string(library.stock_allowance())
                    .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
            )?;
            validate_resource_record(
                connection,
                "machine",
                library.machine().id().as_str(),
                library.machine().version().value(),
                "machine-profile-v1",
                &serde_json::to_string(library.machine())
                    .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
            )?;
            validate_resource_record(
                connection,
                "runtime",
                library.runtime().id().as_str(),
                library.runtime().version().value(),
                "coarse-runtime-profile-v1",
                &serde_json::to_string(library.runtime())
                    .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
            )?;
        }
        None => {
            let reference_exists = connection
                .query_row(
                    "SELECT 1 FROM shop_settings_resource_refs
                     WHERE profile_id = ?1 AND revision = ?2",
                    params![draft.profile_id().as_str(), draft.revision().value()],
                    |_| Ok(()),
                )
                .optional()
                .map_err(|_| ShopSettingsStoreError::Unavailable)?
                .is_some();
            if reference_exists {
                return Err(ShopSettingsStoreError::InvalidStoredRecord);
            }
        }
    }
    match draft.resource_catalog() {
        Some(catalog) => {
            let reference = connection
                .query_row(
                    "SELECT catalog_id, catalog_version FROM shop_settings_catalog_refs
                     WHERE profile_id = ?1 AND revision = ?2",
                    params![draft.profile_id().as_str(), draft.revision().value()],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?)),
                )
                .optional()
                .map_err(|_| ShopSettingsStoreError::Unavailable)?
                .ok_or(ShopSettingsStoreError::InvalidStoredRecord)?;
            if reference != (catalog.id().as_str().to_owned(), catalog.version().value()) {
                return Err(ShopSettingsStoreError::InvalidStoredRecord);
            }
            let payload = serde_json::to_string(catalog)
                .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
            validate_snapshot(
                connection,
                "shop_resource_catalog_snapshots",
                "catalog_id",
                catalog.id().as_str(),
                catalog.version().value(),
                catalog.currency().as_str(),
                SHOP_RESOURCE_CATALOG_SCHEMA,
                &payload,
            )?;
            for material in catalog.materials() {
                validate_resource_record(
                    connection,
                    "material",
                    material.id().as_str(),
                    material.version().value(),
                    "material-definition-v1",
                    &serde_json::to_string(material)
                        .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
                )?;
            }
            for offer in catalog.material_offers() {
                validate_resource_record(
                    connection,
                    "material_offer",
                    offer.id().as_str(),
                    offer.version().value(),
                    "material-offer-v1",
                    &serde_json::to_string(offer)
                        .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
                )?;
            }
            for stock in catalog.stock_allowances() {
                validate_resource_record(
                    connection,
                    "stock_allowance",
                    stock.id().as_str(),
                    stock.version().value(),
                    "stock-allowance-v1",
                    &serde_json::to_string(stock)
                        .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
                )?;
            }
            for machine in catalog.machines() {
                validate_resource_record(
                    connection,
                    "machine",
                    machine.id().as_str(),
                    machine.version().value(),
                    "machine-profile-v1",
                    &serde_json::to_string(machine)
                        .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
                )?;
            }
            for runtime in catalog.runtimes() {
                validate_resource_record(
                    connection,
                    "runtime",
                    runtime.id().as_str(),
                    runtime.version().value(),
                    "coarse-runtime-profile-v1",
                    &serde_json::to_string(runtime)
                        .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?,
                )?;
            }
            for selection in catalog.selections() {
                validate_resource_selection(connection, selection)?;
            }
        }
        None => {
            let reference_exists = connection
                .query_row(
                    "SELECT 1 FROM shop_settings_catalog_refs
                     WHERE profile_id = ?1 AND revision = ?2",
                    params![draft.profile_id().as_str(), draft.revision().value()],
                    |_| Ok(()),
                )
                .optional()
                .map_err(|_| ShopSettingsStoreError::Unavailable)?
                .is_some();
            if reference_exists {
                return Err(ShopSettingsStoreError::InvalidStoredRecord);
            }
        }
    }
    let event = connection
        .query_row(
            "SELECT changed_by, changed_at, reason FROM settings_change_events
             WHERE profile_id = ?1 AND revision = ?2",
            params![draft.profile_id().as_str(), draft.revision().value()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()
        .map_err(|_| ShopSettingsStoreError::Unavailable)?
        .ok_or(ShopSettingsStoreError::InvalidStoredRecord)?;
    if event
        != (
            draft.changed_by().as_str().to_owned(),
            draft.changed_at().as_str().to_owned(),
            draft.change_reason().to_owned(),
        )
    {
        return Err(ShopSettingsStoreError::InvalidStoredRecord);
    }
    Ok(())
}

fn validate_resource_record(
    connection: &Connection,
    kind: &str,
    id: &str,
    version: u32,
    schema: &str,
    payload: &str,
) -> Result<(), ShopSettingsStoreError> {
    let stored = connection
        .query_row(
            "SELECT payload_schema, payload_sha256, payload_json
             FROM shop_resource_record_snapshots
             WHERE record_kind = ?1 AND record_id = ?2 AND version = ?3",
            params![kind, id, version],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()
        .map_err(|_| ShopSettingsStoreError::Unavailable)?
        .ok_or(ShopSettingsStoreError::InvalidStoredRecord)?;
    if stored.0 == schema && stored.1 == sha256(payload.as_bytes()) && stored.2 == payload {
        Ok(())
    } else {
        Err(ShopSettingsStoreError::InvalidStoredRecord)
    }
}

fn validate_resource_selection(
    connection: &Connection,
    selection: &ResourceSelection,
) -> Result<(), ShopSettingsStoreError> {
    let payload = serde_json::to_string(selection)
        .map_err(|_| ShopSettingsStoreError::InvalidStoredRecord)?;
    let stored = connection
        .query_row(
            "SELECT selection_state, decided_by, decided_at, reason,
                    payload_schema, payload_sha256, payload_json
             FROM shop_resource_selection_snapshots
             WHERE selection_id = ?1 AND version = ?2",
            params![selection.id().as_str(), selection.version().value()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            },
        )
        .optional()
        .map_err(|_| ShopSettingsStoreError::Unavailable)?
        .ok_or(ShopSettingsStoreError::InvalidStoredRecord)?;
    if stored.0 == resource_selection_state(selection.state())
        && stored.1 == selection.decided_by().as_str()
        && stored.2 == selection.decided_at().as_str()
        && stored.3 == selection.reason()
        && stored.4 == "resource-selection-v1"
        && stored.5 == sha256(payload.as_bytes())
        && stored.6 == payload
    {
        Ok(())
    } else {
        Err(ShopSettingsStoreError::InvalidStoredRecord)
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_snapshot(
    connection: &Connection,
    table: &str,
    id_column: &str,
    id: &str,
    version: u32,
    currency: &str,
    schema: &str,
    payload: &str,
) -> Result<(), ShopSettingsStoreError> {
    let query = format!(
        "SELECT currency, payload_schema, payload_sha256, payload_json
         FROM {table} WHERE {id_column} = ?1 AND version = ?2"
    );
    let stored = connection
        .query_row(&query, params![id, version], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .optional()
        .map_err(|_| ShopSettingsStoreError::Unavailable)?
        .ok_or(ShopSettingsStoreError::InvalidStoredRecord)?;
    if stored.0 == currency
        && stored.1 == schema
        && stored.2 == sha256(payload.as_bytes())
        && stored.3 == payload
    {
        Ok(())
    } else {
        Err(ShopSettingsStoreError::InvalidStoredRecord)
    }
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use partprobe_application::{
        ActivateShopResourceSelectionRequest, DenyAllShopResourceCatalogAuthorizationPolicy,
        ShopResourceCatalogApplication, ShopResourceCatalogAuthorizationContext,
        ShopResourceCatalogAuthorizationPolicy,
    };
    use partprobe_domain::{
        ActorId, CurrencyCode, PricingMethod, PricingPolicyId, RateCardId, RateVersion, RecordedAt,
        ResourceSelectionId, ResourceSelectionState, RoundingBoundary, RoundingMode,
        RoundingPolicy, RoundingPolicyId, ShopResourceCatalogId, ShopResourceVersion,
    };
    use partprobe_security::{
        AuditCorrelationId, AuthorizationDecision, AuthorizationReasonCode, SecurityPolicyId,
        SecurityPolicyRef, SecurityPolicyVersion,
    };
    use partprobe_test_support::{TestDirectory, decimal, resource_catalog_fixture};

    fn profile() -> ShopProfileId {
        ShopProfileId::new("shop-1").expect("valid profile")
    }

    fn policy_ref() -> SecurityPolicyRef {
        SecurityPolicyRef::new(
            SecurityPolicyId::new("test-catalog-policy").expect("policy ID"),
            SecurityPolicyVersion::new(1).expect("policy version"),
        )
    }

    #[derive(Clone)]
    struct AllowCatalogActivationPolicy;

    impl ShopResourceCatalogAuthorizationPolicy for AllowCatalogActivationPolicy {
        fn evaluate(
            &self,
            _context: &ShopResourceCatalogAuthorizationContext,
        ) -> AuthorizationDecision {
            AuthorizationDecision::allow(
                policy_ref(),
                AuthorizationReasonCode::new("test_allow").expect("reason code"),
            )
        }
    }

    fn activation_request(actor: &str, correlation: &str) -> ActivateShopResourceSelectionRequest {
        ActivateShopResourceSelectionRequest::new(
            profile(),
            ShopSettingsRevision::new(1).expect("settings revision"),
            ShopResourceCatalogId::new("test-resource-catalog").expect("catalog ID"),
            ShopResourceVersion::new(1).expect("catalog version"),
            ResourceSelectionId::new("test-resource-selection").expect("selection ID"),
            ShopResourceVersion::new(1).expect("selection version"),
            ActorId::new(actor).expect("actor"),
            RecordedAt::new("2026-09-10T12:00:00Z").expect("recorded at"),
            "activate reviewed test selection for proposals",
            AuditCorrelationId::new(correlation).expect("correlation ID"),
        )
        .expect("activation request")
    }

    fn pricing_policy(markup: &str) -> PricingPolicy {
        let currency = CurrencyCode::new("USD").expect("valid currency");
        let version = RateVersion::new(1).expect("valid version");
        PricingPolicy::new(
            PricingPolicyId::new("pricing-1").expect("valid policy ID"),
            version,
            currency.clone(),
            PricingMethod::Markup {
                rate: decimal(markup),
            },
            None,
            None,
            RoundingPolicy::new(
                RoundingPolicyId::new("rounding-1").expect("valid rounding ID"),
                version,
                currency,
                2,
                RoundingMode::HalfEven,
                RoundingBoundary::QuoteTotal,
            )
            .expect("valid rounding policy"),
        )
        .expect("valid pricing policy")
    }

    fn draft(revision: u32, reason: &str, markup: Option<&str>) -> ShopSettingsDraft {
        let currency = CurrencyCode::new("USD").expect("valid currency");
        ShopSettingsDraft::new(
            profile(),
            ShopSettingsRevision::new(revision).expect("valid revision"),
            currency.clone(),
            Some(RateCard::empty(
                RateCardId::new("rates-1").expect("valid card ID"),
                RateVersion::new(1).expect("valid version"),
                currency,
            )),
            markup.map(pricing_policy),
            ActorId::new("operator-1").expect("valid actor"),
            RecordedAt::new(format!("2026-09-04T12:00:0{revision}Z"))
                .expect("valid timestamp evidence"),
            reason,
        )
        .expect("valid settings draft")
    }

    fn catalog_draft(
        revision: u32,
        catalog_version: u32,
        material_version: u32,
        density: &str,
        selection_version: u32,
        selection_state: ResourceSelectionState,
        reason: &str,
    ) -> ShopSettingsDraft {
        ShopSettingsDraft::new_with_catalog(
            profile(),
            ShopSettingsRevision::new(revision).expect("valid revision"),
            CurrencyCode::new("USD").expect("currency"),
            None,
            None,
            Some(resource_catalog_fixture(
                catalog_version,
                material_version,
                density,
                selection_version,
                selection_state,
            )),
            ActorId::new("operator-1").expect("valid actor"),
            RecordedAt::new(format!("2026-09-09T15:00:0{revision}Z")).expect("valid timestamp"),
            reason,
        )
        .expect("valid catalog settings draft")
    }

    #[test]
    fn fresh_database_has_no_numeric_or_library_defaults() {
        let directory = TestDirectory::create("settings-empty").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let repository = SqliteShopSettingsRepository::open(&database).expect("open database");

        let cards: u32 = repository
            .connection
            .query_row("SELECT count(*) FROM rate_card_snapshots", [], |row| {
                row.get(0)
            })
            .expect("count cards");
        let policies: u32 = repository
            .connection
            .query_row("SELECT count(*) FROM pricing_policy_snapshots", [], |row| {
                row.get(0)
            })
            .expect("count policies");
        let resources: u32 = repository
            .connection
            .query_row(
                "SELECT count(*) FROM shop_resource_library_snapshots",
                [],
                |row| row.get(0),
            )
            .expect("count resource libraries");
        let resource_records: u32 = repository
            .connection
            .query_row(
                "SELECT count(*) FROM shop_resource_record_snapshots",
                [],
                |row| row.get(0),
            )
            .expect("count resource records");
        let catalogs: u32 = repository
            .connection
            .query_row(
                "SELECT count(*) FROM shop_resource_catalog_snapshots",
                [],
                |row| row.get(0),
            )
            .expect("count resource catalogs");
        let selections: u32 = repository
            .connection
            .query_row(
                "SELECT count(*) FROM shop_resource_selection_snapshots",
                [],
                |row| row.get(0),
            )
            .expect("count resource selections");
        let catalog_refs: u32 = repository
            .connection
            .query_row(
                "SELECT count(*) FROM shop_settings_catalog_refs",
                [],
                |row| row.get(0),
            )
            .expect("count settings catalog references");
        let authorization_events: u32 = repository
            .connection
            .query_row(
                "SELECT count(*) FROM shop_resource_catalog_authorization_events",
                [],
                |row| row.get(0),
            )
            .expect("count authorization events");
        assert_eq!(
            (
                cards,
                policies,
                resources,
                resource_records,
                catalogs,
                selections,
                catalog_refs,
                authorization_events,
            ),
            (0, 0, 0, 0, 0, 0, 0, 0)
        );
        assert!(
            repository
                .current(&profile())
                .expect("load current")
                .is_none()
        );

        drop(repository);
        directory.cleanup().expect("cleanup test directory");
    }

    #[test]
    fn save_reopen_and_revision_replay_preserve_exact_drafts() {
        let directory = TestDirectory::create("settings-reopen").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let first = draft(1, "initial", Some("0.20"));
        let second = draft(2, "reviewed update", Some("0.20"));

        let mut repository = SqliteShopSettingsRepository::open(&database).expect("open database");
        repository.save(&first, None).expect("save first revision");
        repository
            .save(&second, Some(first.revision()))
            .expect("save second revision");
        drop(repository);

        let repository = SqliteShopSettingsRepository::open(&database).expect("reopen database");
        assert_eq!(
            repository.current(&profile()).expect("load current"),
            Some(second)
        );
        assert_eq!(
            repository
                .revision(&profile(), first.revision())
                .expect("load historical"),
            Some(first)
        );

        drop(repository);
        directory.cleanup().expect("cleanup test directory");
    }

    #[test]
    fn catalog_save_reopen_and_replay_preserve_exact_reviewed_selection() {
        let directory = TestDirectory::create("settings-catalog-reopen").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let first = catalog_draft(
            1,
            1,
            1,
            "2700",
            1,
            ResourceSelectionState::ActiveForProposals,
            "initial reviewed catalog",
        );
        let second = catalog_draft(
            2,
            1,
            1,
            "2700",
            1,
            ResourceSelectionState::ActiveForProposals,
            "reused exact catalog in a later settings revision",
        );
        let mut repository = SqliteShopSettingsRepository::open(&database).expect("open database");
        repository.save(&first, None).expect("save catalog");
        repository
            .save(&second, Some(first.revision()))
            .expect("reuse exact catalog");
        drop(repository);

        let repository = SqliteShopSettingsRepository::open(&database).expect("reopen database");
        let current = repository
            .current(&profile())
            .expect("load current")
            .expect("current catalog settings");
        assert_eq!(current, second);
        assert_eq!(
            current
                .resource_catalog()
                .and_then(ShopResourceCatalog::active_selection)
                .map(ResourceSelection::reason),
            Some("reviewed synthetic resource selection")
        );
        assert_eq!(
            repository
                .revision(&profile(), first.revision())
                .expect("load historical"),
            Some(first)
        );
        drop(repository);
        directory.cleanup().expect("cleanup test directory");
    }

    #[test]
    fn durable_audit_composes_with_allowed_activation_and_survives_backup() {
        let directory = TestDirectory::create("settings-catalog-audit").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let backup = directory.path().join("settings-audit-backup.sqlite3");
        let initial = catalog_draft(
            1,
            1,
            1,
            "2700",
            1,
            ResourceSelectionState::Reviewed,
            "reviewed catalog awaiting activation",
        );
        let mut repository = SqliteShopSettingsRepository::open(&database).expect("repository");
        repository
            .save(&initial, None)
            .expect("save initial catalog");
        let audit = SqliteShopResourceCatalogAuthorizationAudit::open(&database).expect("audit");
        let mut application =
            ShopResourceCatalogApplication::new(repository, AllowCatalogActivationPolicy, audit);

        let activated = application
            .activate_for_proposals(&activation_request(
                "test-activation-reviewer",
                "test-allowed-activation",
            ))
            .expect("activate reviewed selection");
        assert_eq!(activated.revision().value(), 2);
        assert_eq!(
            activated
                .resource_catalog()
                .and_then(ShopResourceCatalog::active_selection)
                .map(ResourceSelection::state),
            Some(ResourceSelectionState::ActiveForProposals)
        );

        let (repository, _, audit) = application.into_parts();
        let event: (String, String, String, u64, String) = audit
            .connection
            .query_row(
                "SELECT operation, outcome, policy_id, policy_version, reason_code
                 FROM shop_resource_catalog_authorization_events
                 WHERE correlation_id = 'test-allowed-activation'",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        u64::try_from(row.get::<_, i64>(3)?).expect("positive policy version"),
                        row.get(4)?,
                    ))
                },
            )
            .expect("stored authorization event");
        assert_eq!(
            event,
            (
                "activate_for_proposals".to_owned(),
                "allowed".to_owned(),
                "test-catalog-policy".to_owned(),
                1,
                "test_allow".to_owned(),
            )
        );
        drop(audit);
        repository.backup_to(&backup).expect("backup with audit");
        drop(repository);

        let restored = SqliteShopSettingsRepository::open(&backup).expect("restored repository");
        assert_eq!(
            restored
                .current(&profile())
                .expect("restored current")
                .expect("restored settings")
                .revision(),
            activated.revision()
        );
        let restored_audit =
            SqliteShopResourceCatalogAuthorizationAudit::open(&backup).expect("restored audit");
        let event_count: u32 = restored_audit
            .connection
            .query_row(
                "SELECT count(*) FROM shop_resource_catalog_authorization_events",
                [],
                |row| row.get(0),
            )
            .expect("restored event count");
        assert_eq!(event_count, 1);
        drop(restored_audit);
        drop(restored);
        directory.cleanup().expect("cleanup test directory");
    }

    #[test]
    fn durable_denial_is_idempotent_and_rejects_correlation_reuse() {
        let directory =
            TestDirectory::create("settings-catalog-audit-idempotent").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let initial = catalog_draft(
            1,
            1,
            1,
            "2700",
            1,
            ResourceSelectionState::Reviewed,
            "reviewed catalog awaiting authorization",
        );
        let mut repository = SqliteShopSettingsRepository::open(&database).expect("repository");
        repository
            .save(&initial, None)
            .expect("save initial catalog");
        let audit = SqliteShopResourceCatalogAuthorizationAudit::open(&database).expect("audit");
        let policy = DenyAllShopResourceCatalogAuthorizationPolicy::new(
            policy_ref(),
            AuthorizationReasonCode::new("test_denied").expect("reason code"),
        );
        let mut application = ShopResourceCatalogApplication::new(repository, policy, audit);
        let request = activation_request("test-denied-reviewer", "test-denied-activation");

        assert!(matches!(
            application.activate_for_proposals(&request),
            Err(partprobe_application::ShopResourceCatalogActivationError::Denied(_))
        ));
        assert!(matches!(
            application.activate_for_proposals(&request),
            Err(partprobe_application::ShopResourceCatalogActivationError::Denied(_))
        ));
        assert_eq!(
            application.activate_for_proposals(&activation_request(
                "another-reviewer",
                "test-denied-activation",
            )),
            Err(partprobe_application::ShopResourceCatalogActivationError::AuditUnavailable)
        );

        let (repository, _, audit) = application.into_parts();
        assert_eq!(
            repository.current(&profile()).expect("unchanged current"),
            Some(initial)
        );
        let event_count: u32 = audit
            .connection
            .query_row(
                "SELECT count(*) FROM shop_resource_catalog_authorization_events",
                [],
                |row| row.get(0),
            )
            .expect("event count");
        assert_eq!(event_count, 1);
        drop(audit);
        drop(repository);
        directory.cleanup().expect("cleanup test directory");
    }

    #[test]
    fn catalog_rejects_reused_child_or_selection_versions_without_advancing_current() {
        let directory = TestDirectory::create("settings-catalog-version").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let first = catalog_draft(
            1,
            1,
            1,
            "2700",
            1,
            ResourceSelectionState::ActiveForProposals,
            "initial reviewed catalog",
        );
        let changed_child = catalog_draft(
            2,
            2,
            1,
            "2710",
            1,
            ResourceSelectionState::ActiveForProposals,
            "invalid reused material version",
        );
        let changed_selection = catalog_draft(
            2,
            2,
            1,
            "2700",
            1,
            ResourceSelectionState::Reviewed,
            "invalid reused selection version",
        );
        let mut repository = SqliteShopSettingsRepository::open(&database).expect("open database");
        repository.save(&first, None).expect("save initial catalog");
        assert_eq!(
            repository.save(&changed_child, Some(first.revision())),
            Err(ShopSettingsStoreError::IntegrityConflict)
        );
        assert_eq!(
            repository.save(&changed_selection, Some(first.revision())),
            Err(ShopSettingsStoreError::IntegrityConflict)
        );
        assert_eq!(
            repository.current(&profile()).expect("load current"),
            Some(first)
        );
        drop(repository);
        directory.cleanup().expect("cleanup test directory");
    }

    #[test]
    fn stale_writer_is_rejected_without_changing_current_revision() {
        let directory = TestDirectory::create("settings-conflict").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let first = draft(1, "initial", None);
        let second = draft(2, "writer one", None);
        let stale = draft(2, "stale writer", None);

        let mut writer_one =
            SqliteShopSettingsRepository::open(&database).expect("open writer one");
        writer_one.save(&first, None).expect("save first revision");
        let mut writer_two =
            SqliteShopSettingsRepository::open(&database).expect("open writer two");
        writer_one
            .save(&second, Some(first.revision()))
            .expect("save writer one");
        assert_eq!(
            writer_two.save(&stale, Some(first.revision())),
            Err(ShopSettingsStoreError::RevisionConflict)
        );
        assert_eq!(
            writer_two.current(&profile()).expect("load current"),
            Some(second)
        );

        drop(writer_two);
        drop(writer_one);
        directory.cleanup().expect("cleanup test directory");
    }

    #[test]
    fn immutable_policy_identity_rejects_divergent_payload() {
        let directory = TestDirectory::create("settings-immutable").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let first = draft(1, "initial", Some("0.20"));
        let divergent = draft(2, "changed payload without new version", Some("0.25"));

        let mut repository = SqliteShopSettingsRepository::open(&database).expect("open database");
        repository.save(&first, None).expect("save first revision");
        assert_eq!(
            repository.save(&divergent, Some(first.revision())),
            Err(ShopSettingsStoreError::IntegrityConflict)
        );
        assert_eq!(
            repository.current(&profile()).expect("load current"),
            Some(first)
        );

        drop(repository);
        directory.cleanup().expect("cleanup test directory");
    }

    #[test]
    fn backup_reopens_with_current_and_historical_revisions() {
        let directory = TestDirectory::create("settings-backup").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let backup = directory.path().join("settings-backup.sqlite3");
        let first = draft(1, "initial", None);
        let second = draft(2, "second", None);
        let mut repository = SqliteShopSettingsRepository::open(&database).expect("open database");
        repository.save(&first, None).expect("save first revision");
        repository
            .save(&second, Some(first.revision()))
            .expect("save second revision");
        repository.backup_to(&backup).expect("backup database");
        assert_eq!(
            repository.backup_to(&backup),
            Err(ShopSettingsStoreError::IntegrityConflict)
        );
        drop(repository);

        let restored = SqliteShopSettingsRepository::open(&backup).expect("open backup");
        assert_eq!(
            restored.current(&profile()).expect("load current"),
            Some(second)
        );
        assert_eq!(
            restored
                .revision(&profile(), first.revision())
                .expect("load historical"),
            Some(first)
        );
        drop(restored);
        directory.cleanup().expect("cleanup test directory");
    }

    #[test]
    fn catalog_backup_reopens_with_exact_selection_and_history() {
        let directory = TestDirectory::create("settings-catalog-backup").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let backup = directory.path().join("settings-catalog-backup.sqlite3");
        let first = catalog_draft(
            1,
            1,
            1,
            "2700",
            1,
            ResourceSelectionState::ActiveForProposals,
            "initial catalog",
        );
        let second = catalog_draft(
            2,
            1,
            1,
            "2700",
            1,
            ResourceSelectionState::ActiveForProposals,
            "reused exact catalog in backed-up settings history",
        );
        let mut repository = SqliteShopSettingsRepository::open(&database).expect("open database");
        repository.save(&first, None).expect("save first catalog");
        repository
            .save(&second, Some(first.revision()))
            .expect("save second catalog");
        repository.backup_to(&backup).expect("backup database");
        drop(repository);

        let restored = SqliteShopSettingsRepository::open(&backup).expect("open backup");
        assert_eq!(
            restored.current(&profile()).expect("load current"),
            Some(second)
        );
        assert_eq!(
            restored
                .revision(&profile(), first.revision())
                .expect("load historical"),
            Some(first)
        );
        drop(restored);
        directory.cleanup().expect("cleanup test directory");
    }

    #[test]
    fn unknown_newer_schema_fails_closed() {
        let directory = TestDirectory::create("settings-newer-schema").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let connection = Connection::open(&database).expect("create database");
        connection
            .pragma_update(None, "user_version", CURRENT_SCHEMA_VERSION + 1)
            .expect("set future schema");
        drop(connection);

        assert!(matches!(
            SqliteShopSettingsRepository::open(&database),
            Err(ShopSettingsStoreError::UnsupportedSchema)
        ));
        directory.cleanup().expect("cleanup test directory");
    }

    #[test]
    fn schema_one_database_migrates_forward_without_numeric_defaults() {
        let directory = TestDirectory::create("settings-schema-one").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let connection = Connection::open(&database).expect("create database");
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations (
                    version INTEGER PRIMARY KEY CHECK (version > 0),
                    name TEXT NOT NULL UNIQUE,
                    checksum_sha256 TEXT NOT NULL CHECK (length(checksum_sha256) = 64)
                 ) STRICT;",
            )
            .expect("create migration registry");
        connection
            .execute_batch(MIGRATIONS[0].2)
            .expect("apply schema one");
        connection
            .execute(
                "INSERT INTO schema_migrations (version, name, checksum_sha256)
                 VALUES (1, ?1, ?2)",
                params![MIGRATIONS[0].1, sha256(MIGRATIONS[0].2.as_bytes())],
            )
            .expect("record schema one");
        connection
            .pragma_update(None, "user_version", 1)
            .expect("set schema one version");
        drop(connection);

        let repository = SqliteShopSettingsRepository::open(&database).expect("migrate schema");
        let version: u32 = repository
            .connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read schema version");
        let resources: u32 = repository
            .connection
            .query_row(
                "SELECT count(*) FROM shop_resource_library_snapshots",
                [],
                |row| row.get(0),
            )
            .expect("count migrated resources");
        let resource_records: u32 = repository
            .connection
            .query_row(
                "SELECT count(*) FROM shop_resource_record_snapshots",
                [],
                |row| row.get(0),
            )
            .expect("count migrated resource records");
        let catalogs: u32 = repository
            .connection
            .query_row(
                "SELECT count(*) FROM shop_resource_catalog_snapshots",
                [],
                |row| row.get(0),
            )
            .expect("count migrated catalogs");
        let authorization_events: u32 = repository
            .connection
            .query_row(
                "SELECT count(*) FROM shop_resource_catalog_authorization_events",
                [],
                |row| row.get(0),
            )
            .expect("count authorization events");
        assert_eq!(version, 4);
        assert_eq!((resources, resource_records), (0, 0));
        assert_eq!(catalogs, 0);
        assert_eq!(authorization_events, 0);

        drop(repository);
        directory.cleanup().expect("cleanup test directory");
    }

    #[test]
    fn schema_two_database_migrates_forward_and_replays_prior_payload() {
        let directory = TestDirectory::create("settings-schema-two").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let prior = ShopSettingsDraft::new(
            profile(),
            ShopSettingsRevision::new(1).expect("revision"),
            CurrencyCode::new("USD").expect("currency"),
            None,
            None,
            ActorId::new("operator-1").expect("actor"),
            RecordedAt::new("2026-09-09T16:00:00Z").expect("timestamp"),
            "schema-two settings",
        )
        .expect("settings");
        let payload = serde_json::to_string(&prior)
            .expect("serialize")
            .replace(",\"resource_catalog\":null", "");
        let connection = Connection::open(&database).expect("create database");
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations (
                    version INTEGER PRIMARY KEY CHECK (version > 0),
                    name TEXT NOT NULL UNIQUE,
                    checksum_sha256 TEXT NOT NULL CHECK (length(checksum_sha256) = 64)
                 ) STRICT;",
            )
            .expect("create migration registry");
        for migration in &MIGRATIONS[..2] {
            connection
                .execute_batch(migration.2)
                .expect("apply prior migration");
            connection
                .execute(
                    "INSERT INTO schema_migrations (version, name, checksum_sha256)
                     VALUES (?1, ?2, ?3)",
                    params![migration.0, migration.1, sha256(migration.2.as_bytes())],
                )
                .expect("record prior migration");
        }
        connection
            .execute(
                "INSERT INTO shop_settings_revisions
                    (profile_id, revision, currency, changed_by, changed_at, change_reason,
                     payload_schema, payload_sha256, payload_json)
                 VALUES (?1, 1, 'USD', ?2, ?3, ?4, 'shop-settings-draft-v1', ?5, ?6)",
                params![
                    prior.profile_id().as_str(),
                    prior.changed_by().as_str(),
                    prior.changed_at().as_str(),
                    prior.change_reason(),
                    sha256(payload.as_bytes()),
                    payload,
                ],
            )
            .expect("insert prior settings");
        connection
            .execute(
                "INSERT INTO settings_change_events
                    (profile_id, revision, changed_by, changed_at, reason)
                 VALUES (?1, 1, ?2, ?3, ?4)",
                params![
                    prior.profile_id().as_str(),
                    prior.changed_by().as_str(),
                    prior.changed_at().as_str(),
                    prior.change_reason(),
                ],
            )
            .expect("insert prior event");
        connection
            .execute(
                "INSERT INTO shop_settings_current (profile_id, revision) VALUES (?1, 1)",
                [prior.profile_id().as_str()],
            )
            .expect("insert prior current pointer");
        connection
            .pragma_update(None, "user_version", 2)
            .expect("set schema two version");
        drop(connection);

        let repository = SqliteShopSettingsRepository::open(&database).expect("migrate schema");
        let version: u32 = repository
            .connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read schema version");
        assert_eq!(version, 4);
        assert_eq!(
            repository.current(&profile()).expect("load prior settings"),
            Some(prior)
        );
        let catalogs: u32 = repository
            .connection
            .query_row(
                "SELECT count(*) FROM shop_resource_catalog_snapshots",
                [],
                |row| row.get(0),
            )
            .expect("count catalogs");
        assert_eq!(catalogs, 0);
        let authorization_events: u32 = repository
            .connection
            .query_row(
                "SELECT count(*) FROM shop_resource_catalog_authorization_events",
                [],
                |row| row.get(0),
            )
            .expect("count authorization events");
        assert_eq!(authorization_events, 0);
        drop(repository);
        directory.cleanup().expect("cleanup test directory");
    }

    #[test]
    fn schema_three_database_migrates_to_empty_authorization_audit() {
        let directory = TestDirectory::create("settings-schema-three").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let connection = Connection::open(&database).expect("create database");
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations (
                    version INTEGER PRIMARY KEY CHECK (version > 0),
                    name TEXT NOT NULL UNIQUE,
                    checksum_sha256 TEXT NOT NULL CHECK (length(checksum_sha256) = 64)
                 ) STRICT;",
            )
            .expect("create migration registry");
        for migration in &MIGRATIONS[..3] {
            connection
                .execute_batch(migration.2)
                .expect("apply prior migration");
            connection
                .execute(
                    "INSERT INTO schema_migrations (version, name, checksum_sha256)
                     VALUES (?1, ?2, ?3)",
                    params![migration.0, migration.1, sha256(migration.2.as_bytes())],
                )
                .expect("record prior migration");
        }
        connection
            .pragma_update(None, "user_version", 3)
            .expect("set schema three version");
        drop(connection);

        let repository = SqliteShopSettingsRepository::open(&database).expect("migrate schema");
        let version: u32 = repository
            .connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read schema version");
        let authorization_events: u32 = repository
            .connection
            .query_row(
                "SELECT count(*) FROM shop_resource_catalog_authorization_events",
                [],
                |row| row.get(0),
            )
            .expect("count authorization events");
        assert_eq!(version, 4);
        assert_eq!(authorization_events, 0);
        drop(repository);
        directory.cleanup().expect("cleanup test directory");
    }

    #[test]
    fn changed_migration_checksum_fails_closed() {
        let directory = TestDirectory::create("settings-checksum").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let connection = Connection::open(&database).expect("create database");
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations (
                    version INTEGER PRIMARY KEY CHECK (version > 0),
                    name TEXT NOT NULL UNIQUE,
                    checksum_sha256 TEXT NOT NULL CHECK (length(checksum_sha256) = 64)
                 ) STRICT;
                 INSERT INTO schema_migrations (version, name, checksum_sha256)
                 VALUES (1, '0001_shop_settings',
                         '0000000000000000000000000000000000000000000000000000000000000000');
                 PRAGMA user_version = 1;",
            )
            .expect("create mismatched migration history");
        drop(connection);

        assert!(matches!(
            SqliteShopSettingsRepository::open(&database),
            Err(ShopSettingsStoreError::UnsupportedSchema)
        ));
        directory.cleanup().expect("cleanup test directory");
    }

    #[test]
    fn referenced_snapshot_hash_mismatch_fails_closed() {
        let directory = TestDirectory::create("settings-snapshot-hash").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let first = draft(1, "initial", Some("0.20"));
        let mut repository = SqliteShopSettingsRepository::open(&database).expect("open database");
        repository.save(&first, None).expect("save first revision");
        repository
            .connection
            .execute_batch(
                "DROP TRIGGER pricing_policy_snapshots_no_update;
                 UPDATE pricing_policy_snapshots SET payload_json = '{}';",
            )
            .expect("simulate out-of-band payload corruption");

        assert_eq!(
            repository.current(&profile()),
            Err(ShopSettingsStoreError::InvalidStoredRecord)
        );
        drop(repository);
        directory.cleanup().expect("cleanup test directory");
    }

    #[test]
    fn referenced_catalog_selection_hash_mismatch_fails_closed() {
        let directory = TestDirectory::create("settings-selection-hash").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let first = catalog_draft(
            1,
            1,
            1,
            "2700",
            1,
            ResourceSelectionState::ActiveForProposals,
            "initial reviewed catalog",
        );
        let mut repository = SqliteShopSettingsRepository::open(&database).expect("open database");
        repository.save(&first, None).expect("save catalog");
        repository
            .connection
            .execute_batch(
                "DROP TRIGGER shop_resource_selection_snapshots_no_update;
                 UPDATE shop_resource_selection_snapshots SET payload_json = '{}';",
            )
            .expect("simulate out-of-band selection corruption");

        assert_eq!(
            repository.current(&profile()),
            Err(ShopSettingsStoreError::InvalidStoredRecord)
        );
        drop(repository);
        directory.cleanup().expect("cleanup test directory");
    }

    #[test]
    fn catalog_authorization_event_semantic_corruption_fails_closed_on_reopen() {
        let directory =
            TestDirectory::create("settings-authorization-hash").expect("test directory");
        let database = directory.path().join("settings.sqlite3");
        let initial = catalog_draft(
            1,
            1,
            1,
            "2700",
            1,
            ResourceSelectionState::Reviewed,
            "reviewed catalog awaiting authorization",
        );
        let mut repository = SqliteShopSettingsRepository::open(&database).expect("repository");
        repository
            .save(&initial, None)
            .expect("save initial catalog");
        let audit = SqliteShopResourceCatalogAuthorizationAudit::open(&database).expect("audit");
        let policy = DenyAllShopResourceCatalogAuthorizationPolicy::new(
            policy_ref(),
            AuthorizationReasonCode::new("test_denied").expect("reason code"),
        );
        let mut application = ShopResourceCatalogApplication::new(repository, policy, audit);
        assert!(matches!(
            application.activate_for_proposals(&activation_request(
                "test-denied-reviewer",
                "test-corrupted-audit",
            )),
            Err(partprobe_application::ShopResourceCatalogActivationError::Denied(_))
        ));
        let (repository, _, audit) = application.into_parts();
        drop(repository);
        let invalid_payload = catalog_authorization_payload(
            "test-corrupted-audit",
            "shop-1",
            1,
            "test-resource-catalog",
            1,
            "test-resource-selection",
            1,
            "",
            "2026-09-10T12:00:00Z",
            "activate_for_proposals",
            "denied",
            "test-catalog-policy",
            1,
            "test_denied",
        )
        .expect("serialize deliberately invalid event");
        let invalid_hash = sha256(invalid_payload.as_bytes());
        audit
            .connection
            .execute_batch("DROP TRIGGER shop_resource_catalog_authorization_events_no_update;")
            .expect("disable immutable-event trigger for corruption simulation");
        audit
            .connection
            .execute(
                "UPDATE shop_resource_catalog_authorization_events
                 SET actor_id = '', event_sha256 = ?1, event_json = ?2",
                params![invalid_hash, invalid_payload],
            )
            .expect("simulate out-of-band audit corruption");
        drop(audit);

        assert!(matches!(
            SqliteShopResourceCatalogAuthorizationAudit::open(&database),
            Err(ShopSettingsStoreError::InvalidStoredRecord)
        ));
        assert!(matches!(
            SqliteShopSettingsRepository::open(&database),
            Err(ShopSettingsStoreError::InvalidStoredRecord)
        ));
        directory.cleanup().expect("cleanup test directory");
    }
}
