CREATE TABLE rate_card_snapshots (
    card_id TEXT NOT NULL,
    version INTEGER NOT NULL CHECK (version > 0),
    currency TEXT NOT NULL CHECK (length(currency) = 3),
    payload_schema TEXT NOT NULL CHECK (payload_schema = 'rate-card-v1'),
    payload_sha256 TEXT NOT NULL CHECK (length(payload_sha256) = 64),
    payload_json TEXT NOT NULL,
    PRIMARY KEY (card_id, version)
) STRICT;

CREATE TABLE pricing_policy_snapshots (
    policy_id TEXT NOT NULL,
    version INTEGER NOT NULL CHECK (version > 0),
    currency TEXT NOT NULL CHECK (length(currency) = 3),
    payload_schema TEXT NOT NULL CHECK (payload_schema = 'pricing-policy-v1'),
    payload_sha256 TEXT NOT NULL CHECK (length(payload_sha256) = 64),
    payload_json TEXT NOT NULL,
    PRIMARY KEY (policy_id, version)
) STRICT;

CREATE TABLE shop_settings_revisions (
    profile_id TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK (revision > 0),
    currency TEXT NOT NULL CHECK (length(currency) = 3),
    rate_card_id TEXT,
    rate_card_version INTEGER,
    pricing_policy_id TEXT,
    pricing_policy_version INTEGER,
    changed_by TEXT NOT NULL,
    changed_at TEXT NOT NULL,
    change_reason TEXT NOT NULL,
    payload_schema TEXT NOT NULL CHECK (payload_schema = 'shop-settings-draft-v1'),
    payload_sha256 TEXT NOT NULL CHECK (length(payload_sha256) = 64),
    payload_json TEXT NOT NULL,
    PRIMARY KEY (profile_id, revision),
    CHECK ((rate_card_id IS NULL) = (rate_card_version IS NULL)),
    CHECK ((pricing_policy_id IS NULL) = (pricing_policy_version IS NULL)),
    FOREIGN KEY (rate_card_id, rate_card_version)
        REFERENCES rate_card_snapshots(card_id, version),
    FOREIGN KEY (pricing_policy_id, pricing_policy_version)
        REFERENCES pricing_policy_snapshots(policy_id, version)
) STRICT;

CREATE TABLE shop_settings_current (
    profile_id TEXT PRIMARY KEY,
    revision INTEGER NOT NULL CHECK (revision > 0),
    FOREIGN KEY (profile_id, revision)
        REFERENCES shop_settings_revisions(profile_id, revision)
) STRICT;

CREATE TABLE settings_change_events (
    event_id INTEGER PRIMARY KEY,
    profile_id TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK (revision > 0),
    changed_by TEXT NOT NULL,
    changed_at TEXT NOT NULL,
    reason TEXT NOT NULL,
    UNIQUE (profile_id, revision),
    FOREIGN KEY (profile_id, revision)
        REFERENCES shop_settings_revisions(profile_id, revision)
) STRICT;

CREATE TRIGGER rate_card_snapshots_no_update
BEFORE UPDATE ON rate_card_snapshots BEGIN SELECT RAISE(ABORT, 'immutable rate card'); END;
CREATE TRIGGER rate_card_snapshots_no_delete
BEFORE DELETE ON rate_card_snapshots BEGIN SELECT RAISE(ABORT, 'immutable rate card'); END;
CREATE TRIGGER pricing_policy_snapshots_no_update
BEFORE UPDATE ON pricing_policy_snapshots BEGIN SELECT RAISE(ABORT, 'immutable pricing policy'); END;
CREATE TRIGGER pricing_policy_snapshots_no_delete
BEFORE DELETE ON pricing_policy_snapshots BEGIN SELECT RAISE(ABORT, 'immutable pricing policy'); END;
CREATE TRIGGER shop_settings_revisions_no_update
BEFORE UPDATE ON shop_settings_revisions BEGIN SELECT RAISE(ABORT, 'immutable settings revision'); END;
CREATE TRIGGER shop_settings_revisions_no_delete
BEFORE DELETE ON shop_settings_revisions BEGIN SELECT RAISE(ABORT, 'immutable settings revision'); END;
CREATE TRIGGER settings_change_events_no_update
BEFORE UPDATE ON settings_change_events BEGIN SELECT RAISE(ABORT, 'immutable settings event'); END;
CREATE TRIGGER settings_change_events_no_delete
BEFORE DELETE ON settings_change_events BEGIN SELECT RAISE(ABORT, 'immutable settings event'); END;
CREATE TRIGGER schema_migrations_no_update
BEFORE UPDATE ON schema_migrations BEGIN SELECT RAISE(ABORT, 'immutable migration history'); END;
CREATE TRIGGER schema_migrations_no_delete
BEFORE DELETE ON schema_migrations BEGIN SELECT RAISE(ABORT, 'immutable migration history'); END;
