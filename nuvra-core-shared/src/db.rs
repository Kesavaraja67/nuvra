// nuvra-core-shared/src/db.rs
// SQLite schema and typed access layer — D-STK-05, D-DAT-01/02/03.
// R-13: No unsafe.
#![forbid(unsafe_code)]

/// Current DB schema version.
pub const DB_SCHEMA_VERSION: i32 = 1;

/// SQL to create all v1 tables.
/// Sensitive columns store AES-256-GCM ciphertext blobs (D-DAT-01).
pub const CREATE_TABLES_SQL: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS schema_meta (
    version INTEGER NOT NULL
);

-- Clipboard items (D-DAT-02): max 200, 7 days, 1 MB text, 8 MB image, 100 MB total.
-- content_encrypted: AES-256-GCM ciphertext. Nonce prepended (12 bytes).
CREATE TABLE IF NOT EXISTS clipboard_items (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at       INTEGER NOT NULL,  -- Unix timestamp seconds
    content_type     TEXT    NOT NULL,  -- 'text' | 'image' | 'link'
    content_encrypted BLOB   NOT NULL,
    size_bytes       INTEGER NOT NULL,
    pinned           INTEGER NOT NULL DEFAULT 0,
    source_app       TEXT,
    deleted          INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_clipboard_created  ON clipboard_items(created_at);
CREATE INDEX IF NOT EXISTS idx_clipboard_pinned   ON clipboard_items(pinned);

-- Shelf item references (D-DAT-04): max 50.
CREATE TABLE IF NOT EXISTS shelf_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    added_at    INTEGER NOT NULL,
    path_hash   BLOB    NOT NULL UNIQUE,  -- BLAKE3 hash of path (R-20: no path stored)
    path_encrypted BLOB NOT NULL,         -- AES-256-GCM of the actual path
    display_name_encrypted BLOB NOT NULL, -- AES-256-GCM of filename only
    size_bytes  INTEGER,
    missing     INTEGER NOT NULL DEFAULT 0
);
"#;
