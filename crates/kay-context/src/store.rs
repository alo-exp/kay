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
            _ => {
                tracing::warn!(
                    kind = s,
                    "unknown SymbolKind — falling back to FileBoundary"
                );
                Self::FileBoundary
            }
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

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM schema_version", [], |r| r.get(0))?;
        if count == 0 {
            conn.execute("INSERT INTO schema_version VALUES (1)", [])?;
        } else {
            let v: i64 = conn.query_row("SELECT version FROM schema_version", [], |r| r.get(0))?;
            if v != 1 {
                return Err(ContextError::SchemaVersionMismatch { found: v as u32, expected: 1 });
            }
        }
        Ok(())
    }

    pub fn insert_symbol(&self, sym: &Symbol) -> Result<(), ContextError> {
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
                    // unwrap_or_default() yields Duration::ZERO (epoch 0) when the
                    // system clock is set before the Unix epoch — a real possibility
                    // in containers or CI environments with broken/slewed clocks.
                    // Epoch 0 is an acceptable sentinel: it causes the file to appear
                    // "indexed at 1970-01-01", which is harmless today. If indexed_at
                    // is ever used for cache-invalidation logic, revisit this.
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
                    // unwrap_or_default() yields Duration::ZERO (epoch 0) when the
                    // system clock is set before the Unix epoch — a real possibility
                    // in containers or CI environments with broken/slewed clocks.
                    // Epoch 0 is an acceptable sentinel: it causes the file to appear
                    // "indexed at 1970-01-01", which is harmless today. If indexed_at
                    // is ever used for cache-invalidation logic, revisit this.
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

    /// Create a `symbols_vec` table for storing dense vector embeddings.
    ///
    /// Stored as JSON-serialised `f32` arrays in a TEXT column.  The `_embedder`
    /// argument is accepted so callers can pass a typed embedder reference;
    /// only `dimensions` is used at runtime.  This keeps the API symmetric with
    /// the full sqlite-vec vec0 virtual-table path that may be enabled in a
    /// future milestone.
    pub fn enable_vector_search<E: crate::embedder::EmbeddingProvider>(
        &self,
        _embedder: &E,
        _dimensions: usize,
    ) -> Result<(), ContextError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS symbols_vec (
                symbol_id INTEGER NOT NULL,
                embedding TEXT NOT NULL,
                PRIMARY KEY (symbol_id)
            );",
        )?;
        Ok(())
    }

    /// Insert or replace the dense-vector embedding for a symbol.
    ///
    /// `vector` is serialised to a JSON array and stored in `symbols_vec`.
    pub fn upsert_vector(&self, symbol_id: i64, vector: &[f32]) -> Result<(), ContextError> {
        let json = serde_json::to_string(vector)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO symbols_vec (symbol_id, embedding) VALUES (?1, ?2)",
            params![symbol_id, json],
        )?;
        Ok(())
    }

    /// Approximate-nearest-neighbour search by Euclidean distance.
    ///
    /// Performs a full table-scan of `symbols_vec`, computes L2 distance to
    /// `query_vec` in Rust, and returns the `limit` closest symbols ordered by
    /// distance ascending.
    ///
    /// For FakeEmbedder / test use, all vectors are zero-vectors so every row
    /// ties at distance 0.0 and the result is non-empty as long as rows exist.
    pub fn ann_search(&self, query_vec: &[f32], limit: usize) -> Result<Vec<Symbol>, ContextError> {
        // Collect all (symbol_id, embedding) rows.
        let mut stmt = self.conn.prepare(
            "SELECT sv.symbol_id, s.name, s.kind, s.file_path, s.start_line, s.end_line, s.sig, sv.embedding
             FROM symbols_vec sv
             JOIN symbols s ON s.id = sv.symbol_id",
        )?;

        let mut candidates: Vec<(f64, Symbol)> = stmt
            .query_map([], |r| {
                let symbol_id: i64 = r.get(0)?;
                let name: String = r.get(1)?;
                let kind_str: String = r.get(2)?;
                let file_path: String = r.get(3)?;
                let start_line: u32 = r.get(4)?;
                let end_line: u32 = r.get(5)?;
                let sig: String = r.get(6)?;
                let embedding_json: String = r.get(7)?;
                Ok((
                    symbol_id,
                    name,
                    kind_str,
                    file_path,
                    start_line,
                    end_line,
                    sig,
                    embedding_json,
                ))
            })?
            .filter_map(|r| r.ok())
            .filter_map(
                |(sid, name, kind_str, file_path, start_line, end_line, sig, embedding_json)| {
                    let sym = Symbol {
                        id: sid,
                        name,
                        kind: SymbolKind::from_kind_str(&kind_str),
                        file_path,
                        start_line,
                        end_line,
                        sig,
                    };
                    // Deserialise embedding; skip corrupt rows rather than silently
                    // treating a parse failure as a zero-vector (which would make
                    // corrupt rows appear at maximum distance instead of being surfaced).
                    let embedding: Vec<f32> = match serde_json::from_str(&embedding_json) {
                        Ok(v) => v,
                        Err(_e) => {
                            tracing::warn!(
                                symbol_id = sym.id,
                                "embedding parse failed; skipping row"
                            );
                            return None; // filter_map drops this row
                        }
                    };
                    let dist = l2_distance(query_vec, &embedding);
                    Some((dist, sym))
                },
            )
            .collect();

        candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        candidates.truncate(limit);
        Ok(candidates.into_iter().map(|(_, sym)| sym).collect())
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
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(ContextError::from)
    }
}

/// Euclidean (L2) distance between two `f32` slices.
///
/// If the slices differ in length, the shorter one is implicitly zero-padded —
/// extra dimensions in the longer slice contribute their full squared value.
fn l2_distance(a: &[f32], b: &[f32]) -> f64 {
    let max_len = a.len().max(b.len());
    let mut sum = 0f64;
    for i in 0..max_len {
        let ai = a.get(i).copied().unwrap_or(0.0) as f64;
        let bi = b.get(i).copied().unwrap_or(0.0) as f64;
        let diff = ai - bi;
        sum += diff * diff;
    }
    sum.sqrt()
}

// M12-Task 6: Inline unit tests for kay-context store module.
// Complements the integration tests in tests/store.rs with quick
// synchronous assertions on SymbolKind, l2_distance, and Symbol construction.

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn symbol_kind_as_str_roundtrips() {
        for kind in [
            SymbolKind::Function,
            SymbolKind::Method,
            SymbolKind::Trait,
            SymbolKind::Struct,
            SymbolKind::Enum,
            SymbolKind::Module,
            SymbolKind::Class,
            SymbolKind::FileBoundary,
        ] {
            let s = kind.as_str();
            let restored = SymbolKind::from_kind_str(s);
            assert_eq!(restored, kind, "from_kind_str must roundtrip via as_str");
        }
    }

    #[test]
    fn symbol_kind_from_kind_str_unknown_falls_back_to_file_boundary() {
        assert!(matches!(
            SymbolKind::from_kind_str("totally_unknown_kind_xyz"),
            SymbolKind::FileBoundary
        ));
    }

    #[test]
    fn l2_distance_zero_for_identical_vectors() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        assert!((l2_distance(&a, &b)).abs() < 1e-9);
    }

    #[test]
    fn l2_distance_positive_for_different_vectors() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        assert!((l2_distance(&a, &b) - 5.0).abs() < 1e-9, "L2 distance of (3,4) must be 5.0");
    }

    #[test]
    fn l2_distance_zero_pads_shorter_vector() {
        let a = vec![1.0];
        let b = vec![1.0, 1.0];
        // distance = sqrt((1-1)^2 + (0-1)^2) = 1.0
        assert!((l2_distance(&a, &b) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn symbol_debug_format_does_not_panic() {
        let sym = Symbol {
            id: 1,
            name: "foo".to_string(),
            kind: SymbolKind::Function,
            file_path: "test.rs".to_string(),
            start_line: 1,
            end_line: 5,
            sig: "fn foo()".to_string(),
        };
        let debug = format!("{:?}", sym);
        assert!(debug.contains("foo"));
    }
}
