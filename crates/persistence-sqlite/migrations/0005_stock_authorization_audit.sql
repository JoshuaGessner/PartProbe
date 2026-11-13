CREATE TABLE developer_stock_authorization_events (
    correlation_id TEXT PRIMARY KEY,
    profile_id TEXT NOT NULL,
    settings_revision INTEGER NOT NULL CHECK (settings_revision > 0),
    operation TEXT NOT NULL CHECK (operation IN ('create', 'revise', 'review', 'adopt_and_evaluate')),
    outcome TEXT NOT NULL CHECK (outcome IN ('allowed', 'denied')),
    event_schema TEXT NOT NULL CHECK (event_schema = 'developer-stock-authorization-event-v1'),
    event_sha256 TEXT NOT NULL CHECK (length(event_sha256) = 64),
    event_json TEXT NOT NULL CHECK (length(event_json) <= 8192),
    FOREIGN KEY (profile_id, settings_revision)
        REFERENCES shop_settings_revisions(profile_id, revision)
) STRICT;

CREATE INDEX developer_stock_authorization_events_profile_revision
ON developer_stock_authorization_events(profile_id, settings_revision);

CREATE TRIGGER developer_stock_authorization_events_no_update
BEFORE UPDATE ON developer_stock_authorization_events
BEGIN SELECT RAISE(ABORT, 'immutable stock authorization event'); END;

CREATE TRIGGER developer_stock_authorization_events_no_delete
BEFORE DELETE ON developer_stock_authorization_events
BEGIN SELECT RAISE(ABORT, 'immutable stock authorization event'); END;
