# Phase 6: Session Store + Transcript — Pattern Map

**Mapped:** 2026-04-22
**Files analyzed:** 12 new/modified files across `crates/kay-session/` and `crates/kay-cli/`
**Analogs found:** 7 / 7 (all patterns have a codebase analog; rusqlite initialization is the only greenfield element)

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `crates/kay-session/Cargo.toml` | config | — | `crates/kay-tools/Cargo.toml` | exact |
| `crates/kay-session/src/lib.rs` | config | — | `crates/kay-tools/src/lib.rs` | exact |
| `crates/kay-session/src/error.rs` | utility | — | `crates/kay-tools/src/error.rs` | exact |
| `crates/kay-session/src/store.rs` | service | CRUD | `crates/forge_repo/src/database/pool.rs` + `conversation_repo.rs` | role-match (SQLite) |
| `crates/kay-session/src/transcript.rs` | service | file-I/O | `crates/kay-tools/src/events_wire.rs` | exact (JSONL writer) |
| `crates/kay-session/src/snapshot.rs` | service | file-I/O | `crates/forge_snaps/src/service.rs` | role-match |
| `crates/kay-session/src/config.rs` | config | — | `crates/forge_config/src/reader.rs` | role-match (config root pattern) |
| `crates/kay-session/tests/store_init.rs` | test | CRUD | `crates/forge_repo/src/conversation/conversation_repo.rs` (test block) | role-match |
| `crates/kay-session/tests/compile_fail_harness.rs` | test | — | `crates/kay-tools/tests/compile_fail_harness.rs` | exact |
| `crates/kay-session/tests/*.rs` (all other test files) | test | — | `crates/kay-tools/tests/image_read_r2.rs` | role-match (TempDir pattern) |
| `crates/kay-cli/src/main.rs` | controller | request-response | `crates/kay-cli/src/main.rs` (self — add new arms) | exact |
| `crates/kay-cli/src/run.rs` | service | event-driven | `crates/kay-cli/src/run.rs` (self — add event-tap fan-out) | exact |

---

## Pattern Assignments

### `crates/kay-session/Cargo.toml` (config)

**Analog:** `crates/kay-tools/Cargo.toml`

**Crate scaffold pattern** (lines 1–13):
```toml
[package]
name = "kay-session"
publish = false
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Session store + transcript persistence (Phase 6)"
```

**Workspace dep inheritance pattern** (from `kay-tools/Cargo.toml` lines 22–36):
```toml
[dependencies]
kay-tools = { path = "../kay-tools" }       # AgentEventWire (intra-workspace path dep)

uuid        = { workspace = true }           # Session IDs as UUID v4
chrono      = { workspace = true }           # start_time / end_time (serde-enabled)
dirs        = { workspace = true }           # dirs::home_dir() for kay_home()
serde       = { workspace = true }           # SessionMeta, ExportManifest derives
serde_json  = { workspace = true }           # JSONL serialization + manifest.json
thiserror   = { workspace = true }           # SessionError typed enum
sha2        = { workspace = true }           # SHA-256 snapshot integrity
tracing     = { workspace = true }           # structured logging
```

**rusqlite as local dep (NOT workspace)** — invariant from 06-DEPENDENCIES.md §2.1:
```toml
# Local dep: only kay-session uses rusqlite; don't promote to workspace yet.
rusqlite = { version = "0.32", features = ["bundled"] }
```

**Dev-deps** (from `kay-tools/Cargo.toml` lines 46–54, adapted):
```toml
[dev-dependencies]
tempfile   = { workspace = true }   # All tests use TempDir — I-4 invariant
insta      = { workspace = true }   # Snapshot assertions for manifest JSON shape
assert_cmd = { workspace = true }   # CLI E2E tests (T-8)
predicates = { workspace = true }   # assert_cmd predicate combinators
proptest   = "1"                    # Local dev-dep (I-6: do NOT promote to workspace)
trybuild   = "1.0"                  # Local dev-dep (I-6: do NOT promote to workspace)
```

**Gotcha:** `proptest` and `trybuild` already appear as local deps in `kay-tools/Cargo.toml` (lines 49, 50) with the same version pins. Copy those exact pins. Do NOT add them to the root `Cargo.toml` workspace deps.

---

### `crates/kay-session/src/lib.rs` (module declaration)

**Analog:** `crates/kay-tools/src/lib.rs`

**Module declaration + deny pattern** (lines 1–40 of `kay-tools/src/lib.rs`):
```rust
//! kay-session — Session store, JSONL transcript, snapshot, and CLI wiring (Phase 6).
//!
//! See .planning/phases/06-session-store/06-CONTEXT.md for decisions DL-1..DL-9.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod config;
pub mod error;
pub mod snapshot;
pub mod store;
pub mod transcript;

pub use config::kay_home;
pub use error::SessionError;
pub use store::{Session, SessionStatus, SessionStore, SessionSummary};
pub use transcript::ExportManifest;
```

**Gotcha:** `#![deny(clippy::unwrap_used, clippy::expect_used)]` is applied at the crate level in every Kay crate — it must appear in `lib.rs` as the first attribute. Test modules in `tests/` use `#[allow(clippy::unwrap_used, clippy::expect_used)]` per the pattern in `kay-tools/tests/image_read_r2.rs` line 28.

---

### `crates/kay-session/src/error.rs` (error enum, thiserror)

**Analog:** `crates/kay-tools/src/error.rs`

**Full pattern** (lines 1–68 of `kay-tools/src/error.rs`):
```rust
use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("session not found: {id}")]
    SessionNotFound { id: String },

    #[error("schema version mismatch: found {found}, expected {expected}")]
    SchemaVersionMismatch { found: u32, expected: u32 },

    #[error("path traversal rejected: {path:?} is outside session cwd {session_cwd:?}")]
    PathTraversalRejected { path: std::path::PathBuf, session_cwd: std::path::PathBuf },

    #[error("transcript deleted during session {session_id}: {path:?}")]
    TranscriptDeleted { session_id: String, path: std::path::PathBuf },

    #[error("snapshot byte cap exceeded ({cap} bytes)")]
    SnapshotCapExceeded { cap: u64 },

    #[error("no snapshots available for session {session_id}")]
    NoSnapshotsAvailable { session_id: String },

    #[error("confirmation required — use --force to restore in non-interactive mode")]
    ConfirmationRequired,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
```

**Key conventions from `kay-tools/src/error.rs`:**
- `#[non_exhaustive]` on the enum (line 8) — mandatory for public API stability
- Named fields (not positional) in every variant — matches the pattern at lines 11–64
- `#[error(transparent)]` + `#[from]` for stdlib I/O errors (line 67) — same as `ToolError::Io`
- Test block at bottom (lines 78–132) with `display_includes_` naming convention for assertions

---

### `crates/kay-session/src/config.rs` (`kay_home()` function)

**Analog:** `crates/forge_config/src/reader.rs`

**Config root pattern** (lines 67–84 of `forge_config/src/reader.rs`):
```rust
fn resolve_base_path() -> PathBuf {
    if let Ok(path) = std::env::var("FORGE_CONFIG") {
        return PathBuf::from(path);
    }

    let base = dirs::home_dir().unwrap_or(PathBuf::from("."));
    let path = base.join("forge");

    // Prefer ~/forge (legacy) when it exists so existing users are not
    // disrupted; fall back to ~/.forge as the default.
    if path.exists() {
        tracing::info!("Using legacy path");
        return path;
    }

    tracing::info!("Using new path");
    base.join(".forge")
}
```

**Kay adaptation** (DL-3 from 06-CONTEXT.md — no legacy path needed):
```rust
/// Returns the Kay home directory.
///
/// Resolution order:
/// 1. `KAY_HOME` environment variable, if set.
/// 2. `~/.kay` as the default.
pub fn kay_home() -> std::path::PathBuf {
    if let Ok(path) = std::env::var("KAY_HOME") {
        return std::path::PathBuf::from(path);
    }
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".kay")
}
```

**Gotcha:** ForgeCode uses `LazyLock` to cache the base path (line 34 of `reader.rs`). Do NOT use `LazyLock` for `kay_home()` — Phase 6 tests set `KAY_HOME` env var per-test to point to a `TempDir`, so caching would break test isolation (I-4 invariant). The function must be re-evaluated on every call.

**Test pattern from `forge_config/src/reader.rs` lines 165–203** (env-var serialization guard):
```rust
// In tests: use an env-var mutex to prevent races between tests that
// set/unset KAY_HOME. Same pattern as forge_config::tests::ENV_MUTEX.
static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct EnvGuard {
    key: &'static str,
    _lock: std::sync::MutexGuard<'static, ()>,
}
// Drop impl removes the env var. Set before each env-sensitive test.
```

---

### `crates/kay-session/src/store.rs` (SQLite initialization, CRUD)

**Analog:** `crates/forge_repo/src/database/pool.rs` (WAL + schema init pattern)

**Important difference:** `forge_repo` uses `diesel` + `r2d2` connection pool. `kay-session` uses `rusqlite` directly (no ORM, no pool). The WAL PRAGMA pattern is what to copy; the ORM layer is not.

**SQLite open + WAL + schema init pattern** (adapted from `pool.rs` lines 108–128):
```rust
use rusqlite::{Connection, params};

pub struct SessionStore {
    conn: Connection,
    sessions_dir: std::path::PathBuf,
}

impl SessionStore {
    pub fn open(root: &std::path::Path) -> Result<Self, SessionError> {
        std::fs::create_dir_all(root)?;
        let db_path = root.join("sessions.db");
        let conn = Connection::open(&db_path)?;

        // WAL mode + concurrency pragmas — same settings as forge_repo's
        // SqliteCustomizer::on_acquire (pool.rs lines 113–127).
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA busy_timeout = 30000;
            PRAGMA foreign_keys = ON;
        ")?;

        Self::init_schema(&conn)?;
        Ok(Self { conn, sessions_dir: root.to_path_buf() })
    }

    fn init_schema(conn: &Connection) -> Result<(), SessionError> {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS sessions (
                id          TEXT PRIMARY KEY,
                title       TEXT NOT NULL DEFAULT '',
                persona     TEXT NOT NULL,
                model       TEXT NOT NULL,
                status      TEXT NOT NULL DEFAULT 'active',
                parent_id   TEXT REFERENCES sessions(id) ON DELETE SET NULL,
                start_time  TEXT NOT NULL,
                end_time    TEXT,
                turn_count  INTEGER NOT NULL DEFAULT 0,
                cost_usd    REAL NOT NULL DEFAULT 0.0,
                jsonl_path  TEXT NOT NULL,
                cwd         TEXT NOT NULL DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS idx_sessions_start ON sessions(start_time DESC);
            CREATE INDEX IF NOT EXISTS idx_sessions_parent ON sessions(parent_id);
        ")?;

        // Idempotent: insert schema_version only if absent
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM schema_version", [], |r| r.get(0)
        )?;
        if count == 0 {
            conn.execute("INSERT INTO schema_version VALUES (1)", [])?;
        } else {
            let version: i64 = conn.query_row(
                "SELECT version FROM schema_version", [], |r| r.get(0)
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
}
```

**Test isolation pattern from `forge_repo/src/conversation/conversation_repo.rs` lines 162–164:**
```rust
// forge_repo uses DatabasePool::in_memory() for test isolation.
// kay-session has no pool, so use TempDir instead:
fn open_store() -> Result<SessionStore, SessionError> {
    let dir = tempfile::TempDir::new().unwrap();
    SessionStore::open(dir.path())
    // Note: TempDir must be kept alive for the test duration —
    // store it in the test struct or bind to `let _dir = ...`
}
```

**Gotcha:** `forge_repo` runs DB operations in `tokio::task::spawn_blocking` (conversation_repo.rs lines 21–42) because rusqlite is synchronous. `kay-session` is a synchronous library (no `async fn` in the public API) — callers in `kay-cli` wrap in `spawn_blocking` at the CLI layer, not inside `kay-session`. This keeps `kay-session` free of tokio as a hard dep.

**Gotcha:** `PRAGMA foreign_keys = ON` must be set on every connection open — SQLite disables FK enforcement by default. `forge_repo` uses diesel's connection-customizer (`pool.rs` line 112) to set pragmas; with raw rusqlite, set them in `Connection::open` via `execute_batch`.

---

### `crates/kay-session/src/transcript.rs` (JSONL write, Display impl)

**Analog:** `crates/kay-tools/src/events_wire.rs`

**Display impl — the single JSONL write point** (lines 202–207 of `events_wire.rs`):
```rust
/// JSONL line form: one compact JSON object terminated by a single `\n`.
impl<'a> std::fmt::Display for AgentEventWire<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        writeln!(f, "{json}")
    }
}
```

**How `kay-cli/src/run.rs` writes JSONL to stdout** (lines 361–380 of `run.rs`):
```rust
// Event-tap fan-out point for Phase 6:
// Each event goes to (a) stdout and (b) session transcript writer.
write!(stdout, "{}", AgentEventWire::from(&ev))?;  // existing
// ADD: session.append_event(&AgentEventWire::from(&ev))?;
stdout.flush().ok();
```

**Session append_event implementation** (copy the Display pattern):
```rust
// In transcript.rs — synchronous, append-mode File handle
pub struct TranscriptWriter {
    file: std::fs::File,
    path: std::path::PathBuf,
    session_id: uuid::Uuid,
}

impl TranscriptWriter {
    pub fn append_event(&mut self, wire: &AgentEventWire<'_>) -> Result<(), SessionError> {
        use std::io::Write;
        // write! uses Display impl — emits one compact JSON + \n (zero copy)
        write!(self.file, "{wire}").map_err(|e| {
            SessionError::TranscriptDeleted {
                session_id: self.session_id.to_string(),
                path: self.path.clone(),
            }
        })
    }
}
```

**Last-line crash recovery pattern** (from 06-CONTEXT.md DL-9):
```rust
/// Truncate transcript to last complete \n-terminated line.
/// Call this on resume before appending new events.
fn truncate_to_last_newline(path: &std::path::Path) -> Result<u64, std::io::Error> {
    let content = std::fs::read(path)?;
    // Scan backward for the last \n
    let truncate_at = content
        .iter()
        .rposition(|&b| b == b'\n')
        .map(|i| i + 1)        // keep the newline itself
        .unwrap_or(0);         // empty file: truncate to 0
    let file = std::fs::OpenOptions::new().write(true).open(path)?;
    file.set_len(truncate_at as u64)?;
    // Count complete lines (= turn count on resume)
    Ok(content[..truncate_at].iter().filter(|&&b| b == b'\n').count() as u64)
}
```

**Gotcha:** The `Display` impl on `AgentEventWire` uses `writeln!` (not `write!` + manual `\n`) — this is the schema invariant locked by `tests/events_wire_snapshots.rs::snap_jsonl_line_format`. The transcript writer MUST call `write!(file, "{wire}")`, NOT `writeln!(file, "{}", json)`, to avoid double-newline. The `\n` is already embedded in the `Display` output.

**Gotcha:** The file handle must be opened in **append mode** (`OpenOptions::new().append(true).create(true).open(path)?`) so concurrent writes from the same process extend the file rather than overwrite it. `append_event` calls `write_all` (via `fmt::Display`) which is atomic for small payloads on POSIX but NOT guaranteed on Windows. Use advisory locking (`fs2::FileExt`) for the Windows CI path.

---

### `crates/kay-session/src/snapshot.rs` (file snapshot write + LRU eviction)

**Analog:** `crates/forge_snaps/src/service.rs`

**Snapshot write pattern** (lines 22–35 of `forge_snaps/src/service.rs`):
```rust
pub async fn create_snapshot(&self, path: PathBuf) -> Result<Snapshot> {
    // Create intermediary directories if they don't exist
    let snapshot_path = snapshot.snapshot_path(Some(self.snapshots_directory.clone()));
    if let Some(parent) = PathBuf::from(&snapshot_path).parent() {
        ForgeFS::create_dir_all(parent).await?;
    }
    let content = ForgeFS::read(&snapshot.path).await?;
    ForgeFS::write(path, content).await?;
    Ok(snapshot)
}
```

**Kay adaptation** (synchronous, per-session path layout from DL-4):
```rust
pub fn record_snapshot(
    &mut self,
    original: &std::path::Path,
    content: &[u8],
    session_cwd: &std::path::Path,
) -> Result<(), SessionError> {
    // DL-6: validate original_path is within session cwd
    let canonical = original.canonicalize()?;
    if !canonical.starts_with(session_cwd) {
        return Err(SessionError::PathTraversalRejected {
            path: canonical,
            session_cwd: session_cwd.to_path_buf(),
        });
    }
    // Compute relative path for storage (preserves directory structure)
    let rel = canonical.strip_prefix(session_cwd)
        .expect("starts_with checked above");
    let dest = self.snapshot_dir
        .join(self.current_turn.to_string())
        .join(rel);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&dest, content)?;
    // Track byte usage for cap enforcement
    self.bytes_used += content.len() as u64;
    self.check_cap()?;
    Ok(())
}
```

**Test pattern from `forge_snaps/src/service.rs` lines 85–135** (TestContext struct):
```rust
// forge_snaps uses a TestContext struct that holds TempDir alive.
// kay-session tests should do the same — TempDir drop = directory deletion.
struct TestStore {
    _dir: tempfile::TempDir,   // keep alive — drop order matters
    store: SessionStore,
}
impl TestStore {
    fn new() -> Self {
        let dir = tempfile::TempDir::new().unwrap();
        let store = SessionStore::open(dir.path()).unwrap();
        Self { _dir: dir, store }
    }
}
```

**Gotcha:** `forge_snaps` is async (uses `ForgeFS::read`/`write`). `kay-session`'s `record_snapshot` is **synchronous** (std::fs) because it runs inside the drain loop which is synchronous at the session-write layer. If the caller is async (tokio), wrap in `tokio::task::spawn_blocking`.

---

### `crates/kay-cli/src/main.rs` (session subcommand dispatch)

**Analog:** `crates/kay-cli/src/main.rs` (existing file — add `Session` arm)

**Existing subcommand dispatch pattern** (lines 56–79 of `main.rs`):
```rust
#[derive(clap::Subcommand)]
enum Command {
    Run(run::RunArgs),
    Eval { #[command(subcommand)] target: eval::EvalTarget },
    Tools { #[command(subcommand)] action: ToolsAction },
    // ADD in W-7:
    Session { #[command(subcommand)] action: SessionAction },
    Rewind(RewindArgs),
}

#[derive(clap::Subcommand)]
enum SessionAction {
    List(session::ListArgs),
    Resume(session::ResumeArgs),
    Fork(session::ForkArgs),
    Export(session::ExportArgs),
    Import(session::ImportArgs),
    Replay(session::ReplayArgs),
}
```

**Main dispatch arm addition** (lines 100–133 of `main.rs` — copy the pattern):
```rust
Some(Command::Session { action }) => match session::dispatch(action) {
    Ok(()) => ExitCode::Success,
    Err(e) => {
        eprintln!("Error: {e:?}");
        classify_error(&e)
    }
},
Some(Command::Rewind(args)) => match session::rewind(args) {
    Ok(()) => ExitCode::Success,
    Err(e) => {
        eprintln!("Error: {e:?}");
        classify_error(&e)
    }
},
```

**Gotcha:** All arms follow the identical `match fn() { Ok → Success, Err → classify_error }` shape. Every new subcommand must return `anyhow::Result<()>` (not `Result<ExitCode, _>`) unless it needs a non-Success exit code on `Ok`. The `kay session *` commands all return exit 0 on success.

---

### `crates/kay-cli/src/run.rs` (event-tap fan-out for W-7)

**Analog:** `crates/kay-cli/src/run.rs` lines 361–380 (drain loop)

**Current drain loop** (lines 361–380 of `run.rs`):
```rust
let mut stdout = std::io::stdout().lock();
let mut sandbox_violation_seen = false;
let mut aborted_seen = false;
while let Some(ev) = event_rx.recv().await {
    if matches!(ev, AgentEvent::SandboxViolation { .. }) {
        sandbox_violation_seen = true;
    }
    if matches!(ev, AgentEvent::Aborted { .. }) {
        aborted_seen = true;
    }
    write!(stdout, "{}", AgentEventWire::from(&ev))?;
    stdout.flush().ok();
}
```

**Event-tap addition for Phase 6** (W-7 modification — add after `write!`):
```rust
// Phase 6 event-tap: passive fan-out to session transcript writer.
// Zero changes to run_turn / kay-core — the drain loop is the only
// subscriber (E-2 architecture from 06-BRAINSTORM.md).
if let Some(ref mut session) = active_session {
    // spawn_blocking not needed: session.append_event is sync
    // and the drain loop runs on a current-thread runtime.
    // I/O errors map to SessionError::TranscriptDeleted → mark lost.
    if let Err(e) = session.append_event(&AgentEventWire::from(&ev)) {
        tracing::error!(error = %e, "transcript write failed — session marked lost");
        session.mark_lost(); // sets status = "lost" in SQLite, DL-9
        active_session = None;
    }
}
```

**`RunArgs` extension for `--resume`** (add to the existing `RunArgs` struct, lines 118–157 of `run.rs`):
```rust
/// Resume an existing session by ID. Loads transcript cursor from
/// sessions.db and appends new events after the existing lines.
#[arg(long, value_name = "SESSION_ID")]
pub resume: Option<uuid::Uuid>,
```

---

### `crates/kay-session/tests/compile_fail_harness.rs` (trybuild canaries)

**Analog:** `crates/kay-tools/tests/compile_fail_harness.rs`

**Trybuild pattern** (lines 58–68 of `compile_fail_harness.rs`):
```rust
#[test]
// NOTE: If the same forge_tool_macros path-resolution blocker hits
// kay-session, add #[ignore = "trybuild path resolution blocker"]
// with the same note as kay-tools. Kay-session has no forge_tool_macros
// dep, so this blocker should NOT apply — run without ignore first.
fn compile_fail_fixtures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/session_not_cloneable.fail.rs");
    t.compile_fail("tests/compile_fail/append_after_close.fail.rs");
    t.compile_fail("tests/compile_fail/store_open_requires_path.fail.rs");
}
```

**Fixture file convention** (from `kay-tools/tests/compile_fail/` directory structure):
- Fixture files: `tests/compile_fail/*.fail.rs`
- Expected stderr: `tests/compile_fail/*.stderr` (generated by `TRYBUILD=overwrite cargo test`)
- The `.fail.rs` extension is a naming convention, not a cargo feature — trybuild finds them by path passed to `t.compile_fail()`

---

### All test files: `crates/kay-session/tests/*.rs` (TempDir isolation)

**Analog:** `crates/kay-tools/tests/image_read_r2.rs` lines 46–49 + `crates/forge_snaps/src/service.rs` lines 86–135

**TempDir isolation pattern** (from `image_read_r2.rs` line 34 + test helper):
```rust
#![allow(clippy::unwrap_used, clippy::expect_used)]

use tempfile::TempDir;

fn open_store() -> (TempDir, SessionStore) {
    let dir = TempDir::new().unwrap();
    let store = SessionStore::open(dir.path()).unwrap();
    (dir, store)   // caller MUST bind dir to keep TempDir alive
}

#[test]
fn open_creates_db() {
    let (_dir, store) = open_store();
    // ... assertions
}
```

**Critical gotcha on TempDir lifetime:** `TempDir` deletes the directory on drop. If the test only binds `store` (dropping `dir` immediately), the temp directory is deleted before `store` is used. Always bind with `let (_dir, store) = open_store()` or wrap in a `TestStore` struct (see forge_snaps pattern above) that keeps `_dir` alive for the struct's lifetime.

**Allow-list for test modules:** Every test file that calls `.unwrap()` must have the `#[allow(...)]` attribute at the top — matching `image_read_r2.rs` line 28 exactly:
```rust
#[allow(clippy::unwrap_used, clippy::expect_used)]
```

---

## Shared Patterns

### WAL Mode Initialization
**Source:** `crates/forge_repo/src/database/pool.rs` lines 108–128 (`SqliteCustomizer::on_acquire`)
**Apply to:** `crates/kay-session/src/store.rs` `SessionStore::open`

The four PRAGMAs to copy verbatim (adapted to rusqlite `execute_batch`):
```rust
conn.execute_batch("
    PRAGMA journal_mode = WAL;
    PRAGMA synchronous = NORMAL;
    PRAGMA busy_timeout = 30000;
    PRAGMA foreign_keys = ON;
")?;
```

### thiserror Error Enum Convention
**Source:** `crates/kay-tools/src/error.rs` lines 8–68
**Apply to:** `crates/kay-session/src/error.rs`

Rules extracted from the analog:
1. `#[non_exhaustive]` on the enum (breaks `match` exhaustiveness for downstream, enabling future variants)
2. Named struct fields (not tuple fields) for every variant with multiple fields
3. `#[error(transparent)]` + `#[from]` for upstream error types (`std::io::Error`, `rusqlite::Error`, `serde_json::Error`)
4. `#[source]` attribute (not `#[from]`) when the upstream error should be the source but NOT auto-converted via `?`
5. Test block at bottom — `display_includes_` test naming convention

### Crate-Level Lint Deny
**Source:** `crates/kay-tools/src/lib.rs` line 8, `crates/kay-sandbox-policy/src/lib.rs` (implied)
**Apply to:** `crates/kay-session/src/lib.rs`

```rust
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

Every Kay crate has this. Test files counteract with `#[allow(...)]`.

### JSONL Wire Format
**Source:** `crates/kay-tools/src/events_wire.rs` lines 179–207
**Apply to:** `crates/kay-session/src/transcript.rs` (`TranscriptWriter::append_event`)

The `Display` impl on `AgentEventWire` produces exactly one `{...}\n` line. The transcript writer calls `write!(file, "{wire}")` — NOT `writeln!` — because `\n` is already embedded. This is the schema invariant locked by `events_wire_snapshots.rs`.

### Test Isolation (TempDir)
**Source:** `crates/forge_snaps/src/service.rs` lines 86–135, `crates/kay-tools/tests/image_read_r2.rs` line 34
**Apply to:** All files in `crates/kay-session/tests/`

I-4 invariant: no test may write to `~/.kay`. All session store roots must point to `TempDir::new().unwrap().path()`. The `KAY_HOME` env var provides the same isolation for any code that calls `kay_home()` inside a test.

---

## No Analog Found

There are no files in this phase with no codebase analog — every module maps to at least a role-match. However, the following areas are **greenfield for the Kay codebase** even though the pattern is copied from `forge_repo`:

| Element | Greenfield reason |
|---|---|
| `rusqlite` raw API usage | `forge_repo` uses diesel ORM; rusqlite API calls are new. Use rusqlite docs for `Connection::open`, `query_row`, `execute_batch`. |
| `JSONL last-line truncation` | The `truncate_to_last_newline` function has no existing analog — implement from first principles using `std::fs::File::set_len`. |
| `LRU eviction by turn` | `forge_snaps` has no byte-cap/eviction logic. Implement: sort snapshot subdirs by turn number (parse as `u64`), delete oldest until `bytes_used <= cap`. |

---

## Metadata

**Analog search scope:** `crates/kay-*/`, `crates/forge_repo/`, `crates/forge_config/`, `crates/forge_snaps/`
**Files scanned:** 18 source files + root `Cargo.toml`
**Pattern extraction date:** 2026-04-22
