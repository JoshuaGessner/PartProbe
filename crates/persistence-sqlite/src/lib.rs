//! SQLite adapter for durable, immutable shop-settings drafts.

use std::fmt::Write as _;
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::path::Path;
use std::time::Duration;

use partprobe_application::{ShopSettingsDraftRepository, ShopSettingsStoreError};
use partprobe_domain::{
    PricingPolicy, RateCard, ShopProfileId, ShopResourceLibrary, ShopSettingsDraft,
    ShopSettingsRevision,
};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use sha2::{Digest, Sha256};

const CURRENT_SCHEMA_VERSION: u32 = 2;
const MIGRATIONS: [(u32, &str, &str); 2] = [
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
];

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

/// SQLite-backed implementation of the application shop-settings port.
pub struct SqliteShopSettingsRepository {
    connection: Connection,
}

impl SqliteShopSettingsRepository {
    /// Opens or creates a database, configures durability, and verifies its migration history.
    pub fn open(path: &Path) -> Result<Self, ShopSettingsStoreError> {
        let connection = Connection::open(path).map_err(|_| ShopSettingsStoreError::Unavailable)?;
        configure(&connection)?;
        migrate(&connection)?;
        verify_integrity(&connection)?;
        Ok(Self { connection })
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
    use partprobe_domain::{
        ActorId, CurrencyCode, PricingMethod, PricingPolicyId, RateCardId, RateVersion, RecordedAt,
        RoundingBoundary, RoundingMode, RoundingPolicy, RoundingPolicyId,
    };
    use partprobe_test_support::{TestDirectory, decimal};

    fn profile() -> ShopProfileId {
        ShopProfileId::new("shop-1").expect("valid profile")
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
        assert_eq!((cards, policies, resources, resource_records), (0, 0, 0, 0));
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
        assert_eq!(version, 2);
        assert_eq!((resources, resource_records), (0, 0));

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
}
