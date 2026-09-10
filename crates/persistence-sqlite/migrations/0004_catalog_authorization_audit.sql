CREATE TABLE shop_resource_catalog_authorization_events (
    correlation_id TEXT PRIMARY KEY,
    profile_id TEXT NOT NULL,
    settings_revision INTEGER NOT NULL CHECK (settings_revision > 0),
    catalog_id TEXT NOT NULL,
    catalog_version INTEGER NOT NULL CHECK (catalog_version > 0),
    selection_id TEXT NOT NULL,
    selection_version INTEGER NOT NULL CHECK (selection_version > 0),
    actor_id TEXT NOT NULL,
    recorded_at TEXT NOT NULL,
    operation TEXT NOT NULL CHECK (operation = 'activate_for_proposals'),
    outcome TEXT NOT NULL CHECK (outcome IN ('allowed', 'denied')),
    policy_id TEXT NOT NULL,
    policy_version INTEGER NOT NULL CHECK (policy_version > 0),
    reason_code TEXT NOT NULL,
    event_schema TEXT NOT NULL CHECK (event_schema = 'shop-resource-catalog-authorization-event-v1'),
    event_sha256 TEXT NOT NULL CHECK (length(event_sha256) = 64),
    event_json TEXT NOT NULL,
    FOREIGN KEY (profile_id, settings_revision)
        REFERENCES shop_settings_revisions(profile_id, revision),
    FOREIGN KEY (catalog_id, catalog_version)
        REFERENCES shop_resource_catalog_snapshots(catalog_id, version),
    FOREIGN KEY (selection_id, selection_version)
        REFERENCES shop_resource_selection_snapshots(selection_id, version)
) STRICT;

CREATE INDEX shop_resource_catalog_authorization_events_profile_time
ON shop_resource_catalog_authorization_events(profile_id, recorded_at);

CREATE TRIGGER shop_resource_catalog_authorization_events_no_update
BEFORE UPDATE ON shop_resource_catalog_authorization_events
BEGIN SELECT RAISE(ABORT, 'immutable catalog authorization event'); END;

CREATE TRIGGER shop_resource_catalog_authorization_events_no_delete
BEFORE DELETE ON shop_resource_catalog_authorization_events
BEGIN SELECT RAISE(ABORT, 'immutable catalog authorization event'); END;
