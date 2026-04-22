use crate::error::ContextError;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::{Path, PathBuf};

pub struct SymbolStore {
    pub conn: Connection,
    pub db_path: PathBuf,
}

impl std::fmt::Debug for SymbolStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymbolStore")
            .field("db_path", &self.db_path)
            .finish_non_exhaustive()
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Method,
    Trait,
    Struct,
    Enum,
    Module,
    Class,
    FileBoundary,
}

impl SymbolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Method => "method",
            Self::Trait => "trait",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Module => "module",
            Self::Class => "class",
            Self::FileBoundary => "file_boundary",
        }
    }

    pub fn from_kind_str(s: &str) -> Self {
        match s {
            "function" => Self::Function,
            "method" => Self::Method,
            "trait" => Self::Trait,
            "struct" => Self::Struct,
            "enum" => Self::Enum,
            "module" => Self::Module,
            "class" => Self::Class,
            _ => Self::FileBoundary,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: i64,
    pub name: String,
    pub kind: SymbolKind,
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub sig: String,
}

impl SymbolStore {
    pub fn open(root: &Path) -> Result<Self, ContextError> {
        std::fs::create_dir_all(root)?;
        let db_path = root.join("context.db");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 30000;
             PRAGMA foreign_keys = ON;",
        )?;
        Self::init_schema(&conn)?;
        Ok(Self { conn, db_path })
    }

    fn init_schema(conn: &Connection) -> Result<(), ContextError> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);

            CREATE TABLE IF NOT EXISTS symbols (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                name       TEXT NOT NULL,
                kind       TEXT NOT NULL,
                file_path  TEXT NOT NULL,
                start_line INTEGER NOT NULL DEFAULT 0,
                end_line   INTEGER NOT NULL DEFAULT 0,
                sig        TEXT NOT NULL DEFAULT ''
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS symbols_fts USING fts5(
                name, sig, content=symbols, content_rowid=id,
                tokenize='unicode61 remove_diacritics 1'
            );

            CREATE TABLE IF NOT EXISTS index_state (
                file_path    TEXT PRIMARY KEY,
                content_hash TEXT NOT NULL,
                indexed_at   INTEGER NOT NULL DEFAULT 0
            );

            CREATE TRIGGER IF NOT EXISTS symbols_ai
            AFTER INSERT ON symbols BEGIN
                INSERT INTO symbols_fts(rowid, name, sig)
                VALUES (new.id, new.name, new.sig);
            END;

            CREATE TRIGGER IF NOT EXISTS symbols_ad
            AFTER DELETE ON symbols BEGIN
                INSERT INTO symbols_fts(symbols_fts, rowid, name, sig)
                VALUES ('delete', old.id, old.name, old.sig);
            END;
            ",
        )?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM schema_version",
            [],
            |r| r.get(0),
        )?;
        if count == 0 {
            conn.execute("INSERT INTO schema_version VALUES (1)", [])?;
        } else {
            let v: i64 = conn.query_row(
                "SELECT version FROM schema_version",
                [],
                |r| r.get(0),
            )?;
            if v != 1 {
                return Err(ContextError::SchemaVersionMismatch {
                    found: v as u32,
                    expected: 1,
                });
            }
        }
        Ok(())
    }

    pub fn upsert_symbol(&self, sym: &Symbol) -> Result<(), ContextError> {
        self.conn.execute(
            "INSERT INTO symbols (name, kind, file_path, start_line, end_line, sig)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                sym.name,
                sym.kind.as_str(),
                sym.file_path,
                sym.start_line,
                sym.end_line,
                sym.sig
            ],
        )?;
        Ok(())
    }

    pub fn delete_file(&self, file_path: &str) -> Result<(), ContextError> {
        // Delete from symbols — triggers fire to clean FTS
        self.conn.execute(
            "DELETE FROM symbols WHERE file_path = ?1",
            params![file_path],
        )?;
        // Remove from index_state
        self.conn.execute(
            "DELETE FROM index_state WHERE file_path = ?1",
            params![file_path],
        )?;
        Ok(())
    }

    /// Returns `true` if the file should be SKIPPED (hash unchanged).
    /// Returns `false` if the file needs re-indexing (new or hash changed).
    /// When hash changes, automatically deletes old symbols for this file.
    pub fn check_and_set_index_state(
        &self,
        file_path: &str,
        content_hash: &str,
    ) -> Result<bool, ContextError> {
        let existing: Option<String> = self
            .conn
            .query_row(
                "SELECT content_hash FROM index_state WHERE file_path = ?1",
                params![file_path],
                |r| r.get(0),
            )
            .optional()?;

        match existing {
            Some(ref h) if h == content_hash => Ok(true), // skip — hash unchanged
            Some(_) => {
                // Hash changed — delete old symbols and update state
                self.delete_file(file_path)?;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                self.conn.execute(
                    "INSERT OR REPLACE INTO index_state (file_path, content_hash, indexed_at)
                     VALUES (?1, ?2, ?3)",
                    params![file_path, content_hash, now as i64],
                )?;
                Ok(false) // must index
            }
            None => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                self.conn.execute(
                    "INSERT INTO index_state (file_path, content_hash, indexed_at)
                     VALUES (?1, ?2, ?3)",
                    params![file_path, content_hash, now as i64],
                )?;
                Ok(false) // must index
            }
        }
    }

    pub fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<Symbol>, ContextError> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.name, s.kind, s.file_path, s.start_line, s.end_line, s.sig
             FROM symbols_fts f
             JOIN symbols s ON s.id = f.rowid
             WHERE symbols_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![query, limit as i64], |r| {
            Ok(Symbol {
                id: r.get(0)?,
                name: r.get(1)?,
                kind: SymbolKind::from_kind_str(&r.get::<_, String>(2)?),
                file_path: r.get(3)?,
                start_line: r.get(4)?,
                end_line: r.get(5)?,
                sig: r.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(ContextError::from)
    }
}
