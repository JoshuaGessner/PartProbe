CREATE TABLE shop_resource_catalog_snapshots (
    catalog_id TEXT NOT NULL,
    version INTEGER NOT NULL CHECK (version > 0),
    currency TEXT NOT NULL CHECK (length(currency) = 3),
    payload_schema TEXT NOT NULL CHECK (payload_schema = 'shop-resource-catalog-v1'),
    payload_sha256 TEXT NOT NULL CHECK (length(payload_sha256) = 64),
    payload_json TEXT NOT NULL,
    PRIMARY KEY (catalog_id, version)
) STRICT;

CREATE TABLE shop_resource_selection_snapshots (
    selection_id TEXT NOT NULL,
    version INTEGER NOT NULL CHECK (version > 0),
    selection_state TEXT NOT NULL CHECK (selection_state IN ('reviewed', 'active_for_proposals', 'retired')),
    decided_by TEXT NOT NULL,
    decided_at TEXT NOT NULL,
    reason TEXT NOT NULL,
    payload_schema TEXT NOT NULL CHECK (payload_schema = 'resource-selection-v1'),
    payload_sha256 TEXT NOT NULL CHECK (length(payload_sha256) = 64),
    payload_json TEXT NOT NULL,
    PRIMARY KEY (selection_id, version)
) STRICT;

CREATE TABLE shop_settings_catalog_refs (
    profile_id TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK (revision > 0),
    catalog_id TEXT NOT NULL,
    catalog_version INTEGER NOT NULL CHECK (catalog_version > 0),
    PRIMARY KEY (profile_id, revision),
    FOREIGN KEY (profile_id, revision)
        REFERENCES shop_settings_revisions(profile_id, revision),
    FOREIGN KEY (catalog_id, catalog_version)
        REFERENCES shop_resource_catalog_snapshots(catalog_id, version)
) STRICT;

CREATE TRIGGER shop_resource_catalog_snapshots_no_update
BEFORE UPDATE ON shop_resource_catalog_snapshots BEGIN SELECT RAISE(ABORT, 'immutable shop resource catalog'); END;
CREATE TRIGGER shop_resource_catalog_snapshots_no_delete
BEFORE DELETE ON shop_resource_catalog_snapshots BEGIN SELECT RAISE(ABORT, 'immutable shop resource catalog'); END;
CREATE TRIGGER shop_resource_selection_snapshots_no_update
BEFORE UPDATE ON shop_resource_selection_snapshots BEGIN SELECT RAISE(ABORT, 'immutable resource selection'); END;
CREATE TRIGGER shop_resource_selection_snapshots_no_delete
BEFORE DELETE ON shop_resource_selection_snapshots BEGIN SELECT RAISE(ABORT, 'immutable resource selection'); END;
CREATE TRIGGER shop_settings_catalog_refs_no_update
BEFORE UPDATE ON shop_settings_catalog_refs BEGIN SELECT RAISE(ABORT, 'immutable shop catalog reference'); END;
CREATE TRIGGER shop_settings_catalog_refs_no_delete
BEFORE DELETE ON shop_settings_catalog_refs BEGIN SELECT RAISE(ABORT, 'immutable shop catalog reference'); END;
