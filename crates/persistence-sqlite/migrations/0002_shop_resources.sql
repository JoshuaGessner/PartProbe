CREATE TABLE shop_resource_library_snapshots (
    library_id TEXT NOT NULL,
    version INTEGER NOT NULL CHECK (version > 0),
    currency TEXT NOT NULL CHECK (length(currency) = 3),
    payload_schema TEXT NOT NULL CHECK (payload_schema = 'shop-resource-library-v1'),
    payload_sha256 TEXT NOT NULL CHECK (length(payload_sha256) = 64),
    payload_json TEXT NOT NULL,
    PRIMARY KEY (library_id, version)
) STRICT;

CREATE TABLE shop_resource_record_snapshots (
    record_kind TEXT NOT NULL CHECK (record_kind IN ('material', 'material_offer', 'stock_allowance', 'machine', 'runtime')),
    record_id TEXT NOT NULL,
    version INTEGER NOT NULL CHECK (version > 0),
    payload_schema TEXT NOT NULL,
    payload_sha256 TEXT NOT NULL CHECK (length(payload_sha256) = 64),
    payload_json TEXT NOT NULL,
    PRIMARY KEY (record_kind, record_id, version)
) STRICT;

CREATE TABLE shop_settings_resource_refs (
    profile_id TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK (revision > 0),
    library_id TEXT NOT NULL,
    library_version INTEGER NOT NULL CHECK (library_version > 0),
    PRIMARY KEY (profile_id, revision),
    FOREIGN KEY (profile_id, revision)
        REFERENCES shop_settings_revisions(profile_id, revision),
    FOREIGN KEY (library_id, library_version)
        REFERENCES shop_resource_library_snapshots(library_id, version)
) STRICT;

CREATE TRIGGER shop_resource_library_snapshots_no_update
BEFORE UPDATE ON shop_resource_library_snapshots BEGIN SELECT RAISE(ABORT, 'immutable shop resource library'); END;
CREATE TRIGGER shop_resource_library_snapshots_no_delete
BEFORE DELETE ON shop_resource_library_snapshots BEGIN SELECT RAISE(ABORT, 'immutable shop resource library'); END;
CREATE TRIGGER shop_resource_record_snapshots_no_update
BEFORE UPDATE ON shop_resource_record_snapshots BEGIN SELECT RAISE(ABORT, 'immutable shop resource record'); END;
CREATE TRIGGER shop_resource_record_snapshots_no_delete
BEFORE DELETE ON shop_resource_record_snapshots BEGIN SELECT RAISE(ABORT, 'immutable shop resource record'); END;
CREATE TRIGGER shop_settings_resource_refs_no_update
BEFORE UPDATE ON shop_settings_resource_refs BEGIN SELECT RAISE(ABORT, 'immutable shop resource reference'); END;
CREATE TRIGGER shop_settings_resource_refs_no_delete
BEFORE DELETE ON shop_settings_resource_refs BEGIN SELECT RAISE(ABORT, 'immutable shop resource reference'); END;
