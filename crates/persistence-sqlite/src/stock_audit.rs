//! Reference-only persistence for developer stock authorization decisions, not operation receipts.

use std::path::Path;

use partprobe_application::{
    DEVELOPER_STOCK_WORKFLOW_RULE_ID, DeveloperStockAuthorizationAuditSink,
    DeveloperStockAuthorizationEvent, DeveloperStockOperation,
    MAX_DEVELOPER_STOCK_CANDIDATE_REVISIONS, ShopSettingsStoreError,
};
use partprobe_domain::{
    ActorId, DataClassificationId, ProjectId, RecordId, RecordStateId, RecordVersionId, RecordedAt,
    ShopProfileId, ShopSettingsRevision,
};
use partprobe_security::{
    AuditAppendError, AuditCorrelationId, AuthorizationReasonCode, SecurityPolicyId,
    SecurityPolicyVersion,
};
use rusqlite::{Connection, Transaction, TransactionBehavior, params};
use serde_json::{Value, json};

use crate::{authorization_outcome, open_connection, sha256};

const EVENT_SCHEMA: &str = "developer-stock-authorization-event-v1";
// The stored event-v1 reader remains pinned if the active service later advances its rule.
const WORKFLOW_VERSION: (u16, u16, u16) = (1, 0, 0);
const MAX_EVENT_BYTES: usize = 8192;
const KEYS: [&str; 23] = [
    "actor_id",
    "analysis_output_hash",
    "candidate_id",
    "classification_id",
    "correlation_id",
    "event_schema",
    "expected_revision",
    "model_record_id",
    "model_revision_id",
    "operation",
    "outcome",
    "policy_id",
    "policy_version",
    "profile_id",
    "project_id",
    "reason_code",
    "record_state_id",
    "recorded_at",
    "resulting_revision",
    "settings_revision",
    "source_hash",
    "workflow_rule_id",
    "workflow_rule_version",
];

// Exact trigger definitions are checked as well as migration checksums. Not external tamper protection.
const UPDATE_TRIGGER: &str = "CREATE TRIGGER developer_stock_authorization_events_no_update BEFORE UPDATE ON developer_stock_authorization_events BEGIN SELECT RAISE(ABORT, 'immutable stock authorization event'); END";
const DELETE_TRIGGER: &str = "CREATE TRIGGER developer_stock_authorization_events_no_delete BEFORE DELETE ON developer_stock_authorization_events BEGIN SELECT RAISE(ABORT, 'immutable stock authorization event'); END";

/// Opens a host-owned Settings database with a separate connection for required stock decisions.
/// No public export/read surface or identity/policy configuration is supplied by this adapter.
pub struct SqliteDeveloperStockAuthorizationAudit {
    connection: Connection,
}

impl SqliteDeveloperStockAuthorizationAudit {
    pub fn open(path: &Path) -> Result<Self, ShopSettingsStoreError> {
        open_connection(path).map(|connection| Self { connection })
    }

    fn append_payload(&self, payload: Value) -> Result<(), AuditAppendError> {
        validate_payload(&payload).map_err(|_| AuditAppendError)?;
        let encoded = serde_json::to_string(&payload).map_err(|_| AuditAppendError)?;
        if encoded.len() > MAX_EVENT_BYTES {
            return Err(AuditAppendError);
        }
        let transaction =
            Transaction::new_unchecked(&self.connection, TransactionBehavior::Immediate)
                .map_err(|_| AuditAppendError)?;
        verify_events(&transaction).map_err(|_| AuditAppendError)?;
        let profile = text(&payload, "profile_id").map_err(|_| AuditAppendError)?;
        let revision = number(&payload, "settings_revision").map_err(|_| AuditAppendError)?;
        let exists: bool = transaction.query_row("SELECT EXISTS(SELECT 1 FROM shop_settings_revisions WHERE profile_id = ?1 AND revision = ?2)", params![profile, revision], |row| row.get(0)).map_err(|_| AuditAppendError)?;
        if !exists {
            return Err(AuditAppendError);
        }
        // Never IGNORE/UPSERT or replay: every correlation reuse must fail, even exact payloads.
        transaction.execute("INSERT INTO developer_stock_authorization_events (correlation_id, profile_id, settings_revision, operation, outcome, event_schema, event_sha256, event_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", params![text(&payload, "correlation_id").map_err(|_| AuditAppendError)?, profile, revision, text(&payload, "operation").map_err(|_| AuditAppendError)?, text(&payload, "outcome").map_err(|_| AuditAppendError)?, EVENT_SCHEMA, sha256(encoded.as_bytes()), encoded]).map_err(|_| AuditAppendError)?;
        transaction.commit().map_err(|_| AuditAppendError)
    }
}

impl DeveloperStockAuthorizationAuditSink for SqliteDeveloperStockAuthorizationAudit {
    fn append(&self, event: DeveloperStockAuthorizationEvent) -> Result<(), AuditAppendError> {
        let c = event.context();
        let d = event.decision();
        let version = c.rule_version();
        self.append_payload(json!({
            "actor_id": c.actor_id().as_str(),
            "analysis_output_hash": c.analysis_output_hash().as_str(),
            "candidate_id": c.candidate_id().as_str(),
            "classification_id": c.classification_id().as_str(),
            "correlation_id": c.correlation_id().as_str(),
            "event_schema": EVENT_SCHEMA,
            "expected_revision": c.expected_revision(),
            "model_record_id": c.record_id().as_str(),
            "model_revision_id": c.record_version_id().as_str(),
            "operation": operation(c.operation()),
            "outcome": authorization_outcome(d.outcome()),
            "policy_id": d.policy().id().as_str(),
            "policy_version": d.policy().version().value(),
            "profile_id": c.profile_id().as_str(),
            "project_id": c.project_id().as_str(),
            "reason_code": d.reason_code().as_str(),
            "record_state_id": c.record_state_id().as_str(),
            "recorded_at": c.recorded_at().as_str(),
            "resulting_revision": c.resulting_revision(),
            "settings_revision": c.settings_revision().value(),
            "source_hash": c.source_hash().as_str(),
            "workflow_rule_id": c.rule_id(),
            "workflow_rule_version": [version.0, version.1, version.2],
        }))
    }
}

const fn operation(value: DeveloperStockOperation) -> &'static str {
    match value {
        DeveloperStockOperation::Create => "create",
        DeveloperStockOperation::Revise => "revise",
        DeveloperStockOperation::Review => "review",
        DeveloperStockOperation::AdoptAndEvaluate => "adopt_and_evaluate",
    }
}

fn invalid() -> ShopSettingsStoreError {
    ShopSettingsStoreError::InvalidStoredRecord
}
fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, ShopSettingsStoreError> {
    value.get(key).and_then(Value::as_str).ok_or_else(invalid)
}
fn number(value: &Value, key: &str) -> Result<u32, ShopSettingsStoreError> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(invalid)
}

fn validate_payload(p: &Value) -> Result<(), ShopSettingsStoreError> {
    let object = p.as_object().ok_or_else(invalid)?;
    if object.len() != KEYS.len() || KEYS.iter().any(|key| !object.contains_key(*key)) {
        return Err(invalid());
    }
    ActorId::new(text(p, "actor_id")?).map_err(|_| invalid())?;
    ProjectId::new(text(p, "project_id")?).map_err(|_| invalid())?;
    RecordId::new(text(p, "model_record_id")?).map_err(|_| invalid())?;
    RecordVersionId::new(text(p, "model_revision_id")?).map_err(|_| invalid())?;
    DataClassificationId::new(text(p, "classification_id")?).map_err(|_| invalid())?;
    RecordStateId::new(text(p, "record_state_id")?).map_err(|_| invalid())?;
    RecordedAt::new(text(p, "recorded_at")?).map_err(|_| invalid())?;
    AuditCorrelationId::new(text(p, "correlation_id")?).map_err(|_| invalid())?;
    ShopProfileId::new(text(p, "profile_id")?).map_err(|_| invalid())?;
    ShopSettingsRevision::new(number(p, "settings_revision")?).map_err(|_| invalid())?;
    RecordId::new(text(p, "candidate_id")?).map_err(|_| invalid())?;
    SecurityPolicyId::new(text(p, "policy_id")?).map_err(|_| invalid())?;
    SecurityPolicyVersion::new(
        p.get("policy_version")
            .and_then(Value::as_u64)
            .ok_or_else(invalid)?,
    )
    .map_err(|_| invalid())?;
    AuthorizationReasonCode::new(text(p, "reason_code")?).map_err(|_| invalid())?;
    let version = WORKFLOW_VERSION;
    if text(p, "event_schema")? != EVENT_SCHEMA
        || text(p, "workflow_rule_id")? != DEVELOPER_STOCK_WORKFLOW_RULE_ID
        || p["workflow_rule_version"] != json!([version.0, version.1, version.2])
        || !matches!(text(p, "outcome")?, "allowed" | "denied")
    {
        return Err(invalid());
    }
    for key in ["source_hash", "analysis_output_hash"] {
        let hash = text(p, key)?;
        if hash.len() != 64
            || !hash
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(invalid());
        }
    }
    let result = number(p, "resulting_revision")?;
    let expected = if p["expected_revision"].is_null() {
        None
    } else {
        Some(number(p, "expected_revision")?)
    };
    let ceiling = u32::try_from(MAX_DEVELOPER_STOCK_CANDIDATE_REVISIONS).map_err(|_| invalid())?;
    if result == 0 || result > ceiling || expected.is_some_and(|n| n == 0 || n > ceiling) {
        return Err(invalid());
    }
    let transition_valid = match text(p, "operation")? {
        "create" => expected.is_none() && result == 1,
        "revise" => expected.and_then(|n| n.checked_add(1)) == Some(result),
        "review" | "adopt_and_evaluate" => expected == Some(result),
        _ => false,
    };
    if !transition_valid {
        return Err(invalid());
    }
    Ok(())
}

pub(crate) fn verify_events(connection: &Connection) -> Result<(), ShopSettingsStoreError> {
    for (name, expected) in [
        (
            "developer_stock_authorization_events_no_update",
            UPDATE_TRIGGER,
        ),
        (
            "developer_stock_authorization_events_no_delete",
            DELETE_TRIGGER,
        ),
    ] {
        let actual: String = connection
            .query_row(
                "SELECT sql FROM sqlite_schema WHERE type = 'trigger' AND name = ?1",
                [name],
                |row| row.get(0),
            )
            .map_err(|_| invalid())?;
        if actual.split_whitespace().collect::<Vec<_>>()
            != expected.split_whitespace().collect::<Vec<_>>()
        {
            return Err(invalid());
        }
    }
    let mut statement = connection.prepare("SELECT e.correlation_id, e.profile_id, e.settings_revision, e.operation, e.outcome, e.event_schema, e.event_sha256, e.event_json, s.revision FROM developer_stock_authorization_events e LEFT JOIN shop_settings_revisions s ON s.profile_id = e.profile_id AND s.revision = e.settings_revision").map_err(|_| invalid())?;
    let mut rows = statement.query([]).map_err(|_| invalid())?;
    while let Some(row) = rows.next().map_err(|_| invalid())? {
        let encoded: String = row.get(7).map_err(|_| invalid())?;
        if encoded.len() > MAX_EVENT_BYTES {
            return Err(invalid());
        }
        let payload: Value = serde_json::from_str(&encoded).map_err(|_| invalid())?;
        validate_payload(&payload)?;
        for (index, key) in [
            (0, "correlation_id"),
            (1, "profile_id"),
            (3, "operation"),
            (4, "outcome"),
            (5, "event_schema"),
        ] {
            if row.get::<_, String>(index).map_err(|_| invalid())? != text(&payload, key)? {
                return Err(invalid());
            }
        }
        if row.get::<_, u32>(2).map_err(|_| invalid())? != number(&payload, "settings_revision")?
            || row
                .get::<_, Option<u32>>(8)
                .map_err(|_| invalid())?
                .is_none()
            || row.get::<_, String>(6).map_err(|_| invalid())? != sha256(encoded.as_bytes())
            || serde_json::to_string(&payload).map_err(|_| invalid())? != encoded
        {
            return Err(invalid());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MIGRATIONS, SqliteShopSettingsRepository};
    use partprobe_application::ShopSettingsDraftRepository;
    use partprobe_domain::{CurrencyCode, ShopSettingsDraft};
    use partprobe_test_support::TestDirectory;

    // Synthetic persisted decision facts only; not configured-worker or native workflow evidence.
    fn payload(correlation: &str, op: &str, outcome: &str) -> Value {
        let (expected, result) = match op {
            "create" => (None, 1),
            "revise" => (Some(1), 2),
            _ => (Some(2), 2),
        };
        json!({
            "actor_id": "synthetic-estimator",
            "analysis_output_hash": "2".repeat(64),
            "candidate_id": "synthetic-stock",
            "classification_id": "public-synthetic",
            "correlation_id": correlation,
            "event_schema": EVENT_SCHEMA,
            "expected_revision": expected,
            "model_record_id": "synthetic-model",
            "model_revision_id": "synthetic-model-v1",
            "operation": op,
            "outcome": outcome,
            "policy_id": "synthetic-stock-policy",
            "policy_version": 1,
            "profile_id": "synthetic-shop",
            "project_id": "synthetic-project",
            "reason_code": "TEST_ONLY",
            "record_state_id": "draft",
            "recorded_at": "2026-09-15T12:00:00Z",
            "resulting_revision": result,
            "settings_revision": 1,
            "source_hash": "1".repeat(64),
            "workflow_rule_id": DEVELOPER_STOCK_WORKFLOW_RULE_ID,
            "workflow_rule_version": [1, 0, 0],
        })
    }

    fn settings() -> ShopSettingsDraft {
        ShopSettingsDraft::new(
            ShopProfileId::new("synthetic-shop").unwrap(),
            ShopSettingsRevision::new(1).unwrap(),
            CurrencyCode::new("USD").unwrap(),
            None,
            None,
            ActorId::new("synthetic-estimator").unwrap(),
            RecordedAt::new("2026-09-15T12:00:00Z").unwrap(),
            "public synthetic storage test",
        )
        .unwrap()
    }

    fn save_settings(path: &Path) {
        let mut repository = SqliteShopSettingsRepository::open(path).unwrap();
        repository.save(&settings(), None).unwrap();
    }

    fn count(audit: &SqliteDeveloperStockAuthorizationAudit) -> u32 {
        audit
            .connection
            .query_row(
                "SELECT count(*) FROM developer_stock_authorization_events",
                [],
                |row| row.get(0),
            )
            .unwrap()
    }

    #[test]
    fn all_operation_decisions_reopen_and_survive_consistent_backup() {
        let directory = TestDirectory::create("stock-audit-backup").unwrap();
        let database = directory.path().join("settings.sqlite3");
        let backup = directory.path().join("backup.sqlite3");
        save_settings(&database);
        let audit = SqliteDeveloperStockAuthorizationAudit::open(&database).unwrap();
        for op in ["create", "revise", "review", "adopt_and_evaluate"] {
            for outcome in ["allowed", "denied"] {
                audit
                    .append_payload(payload(&format!("{op}-{outcome}"), op, outcome))
                    .unwrap();
            }
        }
        assert_eq!(count(&audit), 8);
        let encoded: String = audit.connection.query_row("SELECT event_json FROM developer_stock_authorization_events WHERE correlation_id = 'create-allowed'", [], |row| row.get(0)).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&encoded).unwrap(),
            payload("create-allowed", "create", "allowed")
        );
        for excluded in [
            "dimensions",
            "quantity",
            "price",
            "rates",
            "path",
            "CAD",
            "change_reason",
        ] {
            assert!(!encoded.contains(excluded));
        }
        drop(audit);
        let repository = SqliteShopSettingsRepository::open(&database).unwrap();
        repository.backup_to(&backup).unwrap();
        drop(repository);
        let restored = SqliteDeveloperStockAuthorizationAudit::open(&backup).unwrap();
        assert_eq!(count(&restored), 8);
        drop(restored);
        directory.cleanup().unwrap();
    }

    #[test]
    fn exact_and_changed_correlation_reuse_fail_across_connections_and_reopen() {
        let directory = TestDirectory::create("stock-audit-reuse").unwrap();
        let database = directory.path().join("settings.sqlite3");
        save_settings(&database);
        let first = SqliteDeveloperStockAuthorizationAudit::open(&database).unwrap();
        let second = SqliteDeveloperStockAuthorizationAudit::open(&database).unwrap();
        let original = payload("same-attempt", "adopt_and_evaluate", "allowed");
        first.append_payload(original.clone()).unwrap();
        assert_eq!(
            second.append_payload(original.clone()),
            Err(AuditAppendError)
        );
        let mut changed = original.clone();
        changed["actor_id"] = json!("another-estimator");
        assert_eq!(second.append_payload(changed), Err(AuditAppendError));
        assert_eq!(count(&first), 1);
        drop(first);
        drop(second);
        let reopened = SqliteDeveloperStockAuthorizationAudit::open(&database).unwrap();
        assert_eq!(reopened.append_payload(original), Err(AuditAppendError));
        assert_eq!(count(&reopened), 1);
        drop(reopened);
        directory.cleanup().unwrap();
    }

    #[test]
    fn settings_reference_is_required_and_failure_does_not_reserve_correlation() {
        let directory = TestDirectory::create("stock-audit-reference").unwrap();
        let database = directory.path().join("settings.sqlite3");
        let audit = SqliteDeveloperStockAuthorizationAudit::open(&database).unwrap();
        assert_eq!(count(&audit), 0);
        let original = payload("no-settings", "create", "denied");
        assert_eq!(
            audit.append_payload(original.clone()),
            Err(AuditAppendError)
        );
        assert_eq!(count(&audit), 0);
        save_settings(&database);
        audit.append_payload(original).unwrap();
        let mut wrong = payload("wrong-settings", "create", "allowed");
        wrong["settings_revision"] = json!(2);
        assert_eq!(audit.append_payload(wrong), Err(AuditAppendError));
        assert_eq!(count(&audit), 1);
        drop(audit);
        directory.cleanup().unwrap();
    }

    #[test]
    fn event_update_delete_and_disabled_triggers_fail_closed() {
        let directory = TestDirectory::create("stock-audit-triggers").unwrap();
        let database = directory.path().join("settings.sqlite3");
        save_settings(&database);
        let audit = SqliteDeveloperStockAuthorizationAudit::open(&database).unwrap();
        audit
            .append_payload(payload("first", "create", "allowed"))
            .unwrap();
        assert!(
            audit
                .connection
                .execute(
                    "UPDATE developer_stock_authorization_events SET outcome = 'denied'",
                    []
                )
                .is_err()
        );
        assert!(
            audit
                .connection
                .execute("DELETE FROM developer_stock_authorization_events", [])
                .is_err()
        );
        assert_eq!(count(&audit), 1);
        audit
            .connection
            .execute_batch("DROP TRIGGER developer_stock_authorization_events_no_delete")
            .unwrap();
        assert_eq!(
            audit.append_payload(payload("second", "create", "allowed")),
            Err(AuditAppendError)
        );
        assert!(matches!(
            SqliteDeveloperStockAuthorizationAudit::open(&database),
            Err(ShopSettingsStoreError::InvalidStoredRecord)
        ));
        assert!(matches!(
            SqliteShopSettingsRepository::open(&database),
            Err(ShopSettingsStoreError::InvalidStoredRecord)
        ));
        drop(audit);
        directory.cleanup().unwrap();
    }

    fn corrupt_json(audit: &SqliteDeveloperStockAuthorizationAudit, value: &Value) {
        audit
            .connection
            .execute_batch("DROP TRIGGER developer_stock_authorization_events_no_update")
            .unwrap();
        let encoded = serde_json::to_string(value).unwrap();
        audit.connection.execute("UPDATE developer_stock_authorization_events SET event_json = ?1, event_sha256 = ?2", params![encoded, sha256(encoded.as_bytes())]).unwrap();
        audit.connection.execute_batch(UPDATE_TRIGGER).unwrap();
    }

    #[test]
    fn semantic_payload_corruption_with_matching_hash_is_rejected_on_open_and_append() {
        for (key, replacement) in [
            ("actor_id", json!("")),
            ("source_hash", json!("G".repeat(64))),
            ("workflow_rule_version", json!([2, 0, 0])),
            ("expected_revision", json!(1)),
            ("resulting_revision", json!(33)),
            ("policy_version", json!(0)),
        ] {
            let directory = TestDirectory::create("stock-audit-corruption").unwrap();
            let database = directory.path().join("settings.sqlite3");
            save_settings(&database);
            let audit = SqliteDeveloperStockAuthorizationAudit::open(&database).unwrap();
            let mut original = payload("first", "create", "allowed");
            audit.append_payload(original.clone()).unwrap();
            original[key] = replacement;
            corrupt_json(&audit, &original);
            assert!(
                matches!(
                    SqliteDeveloperStockAuthorizationAudit::open(&database),
                    Err(ShopSettingsStoreError::InvalidStoredRecord)
                ),
                "{key}"
            );
            assert!(
                matches!(
                    SqliteShopSettingsRepository::open(&database),
                    Err(ShopSettingsStoreError::InvalidStoredRecord)
                ),
                "{key}"
            );
            assert_eq!(
                audit.append_payload(payload("second", "create", "allowed")),
                Err(AuditAppendError)
            );
            assert_eq!(count(&audit), 1);
            drop(audit);
            directory.cleanup().unwrap();
        }
    }

    #[test]
    fn unknown_fields_normalized_projection_and_noncanonical_bytes_are_rejected() {
        let directory = TestDirectory::create("stock-audit-projection").unwrap();
        let database = directory.path().join("settings.sqlite3");
        save_settings(&database);
        let audit = SqliteDeveloperStockAuthorizationAudit::open(&database).unwrap();
        let mut original = payload("first", "create", "allowed");
        original["private_dimensions"] = json!([1, 2, 3]);
        assert_eq!(audit.append_payload(original), Err(AuditAppendError));
        assert_eq!(count(&audit), 0);
        let original = payload("first", "create", "allowed");
        audit.append_payload(original.clone()).unwrap();
        let mut projection = original.clone();
        projection["outcome"] = json!("denied");
        corrupt_json(&audit, &projection);
        assert_eq!(
            verify_events(&audit.connection),
            Err(ShopSettingsStoreError::InvalidStoredRecord)
        );
        corrupt_json(&audit, &original);
        assert!(verify_events(&audit.connection).is_ok());
        audit
            .connection
            .execute_batch("DROP TRIGGER developer_stock_authorization_events_no_update")
            .unwrap();
        let pretty = serde_json::to_string_pretty(&original).unwrap();
        audit.connection.execute("UPDATE developer_stock_authorization_events SET event_json = ?1, event_sha256 = ?2", params![pretty, sha256(pretty.as_bytes())]).unwrap();
        audit.connection.execute_batch(UPDATE_TRIGGER).unwrap();
        assert_eq!(
            verify_events(&audit.connection),
            Err(ShopSettingsStoreError::InvalidStoredRecord)
        );
        drop(audit);
        directory.cleanup().unwrap();
    }

    #[test]
    fn payload_bounds_and_revision_transition_rules_reject_without_append() {
        let directory = TestDirectory::create("stock-audit-bounds").unwrap();
        let database = directory.path().join("settings.sqlite3");
        save_settings(&database);
        let audit = SqliteDeveloperStockAuthorizationAudit::open(&database).unwrap();
        for (key, value) in [
            ("project_id", json!("x".repeat(MAX_EVENT_BYTES + 1))),
            ("operation", json!("publish_quote")),
            ("outcome", json!("approved")),
            ("resulting_revision", json!(-1)),
            ("expected_revision", json!(0)),
            ("expected_revision", json!(2)),
            ("policy_version", json!(1.5)),
            ("event_schema", json!("unknown-v2")),
        ] {
            let mut original = payload("rejected", "revise", "allowed");
            original[key] = value;
            assert_eq!(
                audit.append_payload(original),
                Err(AuditAppendError),
                "{key}"
            );
        }
        let mut leaf = payload("max-review", "review", "allowed");
        leaf["expected_revision"] = json!(32);
        leaf["resulting_revision"] = json!(32);
        audit.append_payload(leaf).unwrap();
        assert_eq!(count(&audit), 1);
        drop(audit);
        directory.cleanup().unwrap();
    }

    #[test]
    fn schemas_one_through_four_migrate_without_seeds_and_preserve_prior_settings() {
        for prior_version in 1..=4 {
            let directory = TestDirectory::create("stock-audit-migration").unwrap();
            let database = directory.path().join("settings.sqlite3");
            let connection = Connection::open(&database).unwrap();
            connection.execute_batch("CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY CHECK (version > 0), name TEXT NOT NULL UNIQUE, checksum_sha256 TEXT NOT NULL CHECK (length(checksum_sha256) = 64)) STRICT").unwrap();
            for &(version, name, sql) in &MIGRATIONS[..prior_version] {
                connection.execute_batch(sql).unwrap();
                connection.execute("INSERT INTO schema_migrations (version, name, checksum_sha256) VALUES (?1, ?2, ?3)", params![version, name, sha256(sql.as_bytes())]).unwrap();
            }
            let original = settings();
            let encoded = serde_json::to_string(&original).unwrap();
            connection.execute("INSERT INTO shop_settings_revisions (profile_id, revision, currency, changed_by, changed_at, change_reason, payload_schema, payload_sha256, payload_json) VALUES (?1, 1, 'USD', ?2, ?3, ?4, 'shop-settings-draft-v1', ?5, ?6)", params![original.profile_id().as_str(), original.changed_by().as_str(), original.changed_at().as_str(), original.change_reason(), sha256(encoded.as_bytes()), encoded]).unwrap();
            connection.execute("INSERT INTO settings_change_events (profile_id, revision, changed_by, changed_at, reason) VALUES (?1, 1, ?2, ?3, ?4)", params![original.profile_id().as_str(), original.changed_by().as_str(), original.changed_at().as_str(), original.change_reason()]).unwrap();
            connection
                .pragma_update(None, "user_version", u32::try_from(prior_version).unwrap())
                .unwrap();
            drop(connection);
            let repository = SqliteShopSettingsRepository::open(&database).unwrap();
            assert_eq!(
                repository
                    .revision(original.profile_id(), original.revision())
                    .unwrap(),
                Some(original)
            );
            let audit = SqliteDeveloperStockAuthorizationAudit::open(&database).unwrap();
            assert_eq!(count(&audit), 0);
            for table in [
                "rate_card_snapshots",
                "pricing_policy_snapshots",
                "shop_resource_record_snapshots",
                "shop_resource_catalog_snapshots",
                "shop_resource_catalog_authorization_events",
            ] {
                let rows: u32 = audit
                    .connection
                    .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                        row.get(0)
                    })
                    .unwrap();
                assert_eq!(rows, 0);
            }
            let version: u32 = audit
                .connection
                .pragma_query_value(None, "user_version", |row| row.get(0))
                .unwrap();
            assert_eq!(version, crate::CURRENT_SCHEMA_VERSION);
            drop(audit);
            drop(repository);
            directory.cleanup().unwrap();
        }
    }
}
