use std::path::{Path, PathBuf};
use rusqlite::Connection;
use crate::error::SessionError;

/// Session store backed by SQLite (sessions.db) and a sessions directory.
///
/// Schema version: 1 (see .planning/phases/06-session-store/06-CONTEXT.md)
/// WAL mode is set on every open; foreign keys enforced via PRAGMA.
pub struct SessionStore {
    pub(crate) conn: Connection,
    pub(crate) sessions_dir: PathBuf,
}

impl std::fmt::Debug for SessionStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionStore")
            .field("sessions_dir", &self.sessions_dir)
            .finish_non_exhaustive()
    }
}

impl SessionStore {
    /// Open (or create) the session store at `root`.
    ///
    /// Creates `root/sessions.db` if absent. Initializes schema v1 on first
    /// open; subsequent opens validate schema_version == 1.
    pub fn open(root: &Path) -> Result<Self, SessionError> {
        std::fs::create_dir_all(root)?;
        let db_path = root.join("sessions.db");
        let conn = Connection::open(&db_path)?;

        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA busy_timeout = 30000;
            PRAGMA foreign_keys = ON;
        ")?;

        Self::init_schema(&conn)?;

        Ok(Self {
            conn,
            sessions_dir: root.to_path_buf(),
        })
    }

    fn init_schema(conn: &Connection) -> Result<(), SessionError> {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id          TEXT PRIMARY KEY NOT NULL,
                title       TEXT NOT NULL DEFAULT '', -- USER-SUPPLIED DATA: delimit as [USER_DATA:...] before model injection (DL-7, QG-C7)
                persona     TEXT NOT NULL DEFAULT '',
                model       TEXT NOT NULL DEFAULT '',
                status      TEXT NOT NULL DEFAULT 'active',
                parent_id   TEXT REFERENCES sessions(id) ON DELETE SET NULL,
                start_time  TEXT NOT NULL,
                end_time    TEXT,
                turn_count  INTEGER NOT NULL DEFAULT 0,
                cost_usd    REAL NOT NULL DEFAULT 0.0,
                jsonl_path  TEXT NOT NULL,
                cwd         TEXT NOT NULL DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS sessions_start_time ON sessions(start_time DESC);
            CREATE INDEX IF NOT EXISTS sessions_parent_id ON sessions(parent_id);

            CREATE TABLE IF NOT EXISTS snapshots (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id    TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                turn          INTEGER NOT NULL,
                original_path TEXT NOT NULL,
                snapshot_path TEXT NOT NULL,
                byte_size     INTEGER NOT NULL,
                sha256        TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS snapshots_session_turn ON snapshots(session_id, turn DESC);
        ")?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM schema_version",
            [],
            |row| row.get(0),
        )?;

        if count == 0 {
            conn.execute("INSERT INTO schema_version VALUES (1)", [])?;
        } else {
            let version: i64 = conn.query_row(
                "SELECT version FROM schema_version",
                [],
                |row| row.get(0),
            )?;
            if version != 1 {
                return Err(SessionError::SchemaVersionMismatch {
                    found: version as u32,
                    expected: 1,
                });
            }
        }

        Ok(())
    }

    /// Return the directory that holds per-session subdirectories.
    pub fn sessions_dir(&self) -> &Path {
        &self.sessions_dir
    }
}
