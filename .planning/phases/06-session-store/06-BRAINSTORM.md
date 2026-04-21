# Phase 6 Brainstorm — Session Store + Transcript

> **Date:** 2026-04-22
> **Phase:** 6 — Session Store + Transcript
> **Mode:** autonomous (§10e) — inline execution; superpowers:brainstorming + product-management:product-brainstorming combined
> **REQs in scope:** SESS-01, SESS-02, SESS-03, SESS-04, SESS-05, CLI-02

---

## Product-Lens (product-management:product-brainstorming)

### Problem Frame

**What:** Sessions are ephemeral after Phase 5. `kay` exits → everything is lost.
**Why now:** Phase 5 agent loop is stable; Phase 9 Tauri GUI and Phase 9.5 TUI can't usefully list or resume sessions without a persistence layer. This is the blocker.
**Constraints:** Must not break QG-C4 (event_filter CI gate); DCO on every commit; no new `externalBin` sidecars (Tauri notarization constraint).

### User Segments & JTBD

| Segment | Situation | Job | Expected Outcome |
|---------|-----------|-----|-----------------|
| Power CLI user | 2hr session gets interrupted (crash/disconnect) | Resume exactly where I left off | `kay resume <id>` restores full transcript + cursor position without re-prompting |
| Benchmark runner | Submitting TB 2.0 run | Export a self-contained, independently replayable bundle | `kay session export <id>` produces a JSONL + manifest that a reviewer can `import` and replay |
| Future frontend (Phase 9/9.5) | GUI session list panel | List sessions < 100ms without reading full JSONL | SQLite index with title/timestamp/turn-count/status metadata per session |
| Debugging user | Tool call corrupted a file | Recover the pre-edit state without git | `kay rewind` restores pre-edit snapshot from `~/.kay/snapshots/` |
| Team member receiving bug report | Reproducing another user's bug | Replay the session that triggered the bug | `kay session replay <bundle>` reproduces the exact event sequence |

### Jobs-to-be-Done (primary)

> "When my coding session is interrupted, I want to resume exactly where I left off so I can maintain flow without re-explaining context to the agent."

> "When submitting a benchmark run, I want to export a self-contained session bundle so the auditor can independently verify my results."

> "When a tool call corrupted my file, I want single-step rewind to the pre-edit state so I can recover without reaching for git."

### Strategic Insights

1. **JSONL export IS the TB 2.0 submission format.** Reproducibility is load-bearing infrastructure, not a convenience feature. The export schema must be stable before Phase 12.

2. **`parent_session_id` in v1 schema is Phase 10 infrastructure.** Multi-agent orchestration (Phase 10) needs to fork sessions. Wrong schema now = painful SQLite migration later. Reserve the column in v1 even if unused.

3. **Phase 7 will also use SQLite** (symbol/vector storage). Design Phase 6's DB path and WAL config so Phase 7 can open a *separate* SQLite file (or a different schema in the same file) without write-contention. Don't hardcode the DB path in a way that blocks Phase 7.

4. **Snapshot storage is a footgun without a budget.** Large files × many turns × multiple sessions = GB-scale storage with no visibility. Phase 6 must set a per-session byte cap (configurable; default 100 MiB) before Phase 9 makes snapshots visible in the GUI.

5. **The resume UX is the whole value prop.** If `kay resume <id>` feels clunky or partial, users won't trust it and will just restart from scratch. The transcript + cursor position must be indistinguishable from a live session.

### HMW Questions Resolved

| HMW | Resolution |
|-----|-----------|
| Make resume transparent? | Restore full cursor position (turn index + event offset) + last control state; the resumed session emits events as if the loop never stopped |
| Keep exports small? | Export includes transcript JSONL + manifest JSON, NOT snapshots (snapshots are local recovery artifacts, not portable) |
| Prevent JSONL corruption on crash? | Append-only writes; on read, truncate after last complete `\n`-terminated line (last-line recovery) |
| Avoid Phase 7 SQLite conflict? | Separate DB files: `~/.kay/sessions.db` (Phase 6) vs `~/.kay/context.db` (Phase 7). Both use WAL mode. |
| Support multi-session without contention? | WAL mode + per-session JSONL file (no shared write path) |

### Risks

| ID | Risk | Severity | Mitigation |
|----|------|----------|-----------|
| R-01 | Schema lock-in after Phase 9 starts reading sessions.db | HIGH | Finalize schema with explicit `schema_version` table in Phase 6; write a migration stub |
| R-02 | Snapshot storage bloat in long sessions | MEDIUM | Per-session byte cap (default 100 MiB configurable); LRU eviction of oldest snapshots within cap |
| R-03 | SQLite write contention in multi-session scenarios | MEDIUM | WAL mode from v1; each session owns its JSONL file exclusively |
| R-04 | Clock skew makes benchmark timestamps non-reproducible | LOW | Use monotonic turn sequence numbers as primary ordering key; wall-clock timestamp is secondary metadata |
| R-05 | JSONL last-line partial write on crash corrupts resume | HIGH | Truncate-to-last-newline on open; atomic append via `write_all` on each event (not buffered) |
| R-06 | Fork creates orphaned child sessions if parent deleted | LOW | `ON DELETE CASCADE` on `parent_session_id` FK, or document the orphan-retention policy explicitly |

### Scope Boundaries

**IN (Phase 6):**
- JSONL transcript files + SQLite index (sessions.db)
- `kay session list / resume / fork / export / import / replay`
- Pre-edit file snapshots under `~/.kay/snapshots/`
- `kay rewind` single-step restore
- CLI-02: session import/export/replay

**OUT (Phase 6):**
- Tauri/TUI session list panel (Phase 9/9.5)
- Multi-session spawn/pause/resume from GUI (Phase 10)
- Vector-search over session transcripts (Phase 7)
- Long-session memory-leak canary (Phase 9 TAURI-05)
- Auto-compaction of old sessions (future quality-of-life)

---

## Engineering-Lens (superpowers:brainstorming)

### Architecture Decisions

#### E-1 — Crate placement: `kay-session` as a new standalone crate

**Options considered:**
- A. Add session store to `kay-core` — couples persistence to the loop crate; makes Phase 7 harder to separate
- B. Add to `kay-cli` — session store becomes CLI-only; Tauri GUI can't use it in Phase 9
- C. New `crates/kay-session/` crate — clean interface seam; importable by kay-cli, future kay-tauri, and Phase 7

**Decision: C.** `crates/kay-session/` is a new workspace crate. Dependencies: `kay-tools::events::AgentEvent` (read the event type), `rusqlite` (SQLite), `serde_json` (JSONL serialization), `uuid` (session IDs), `chrono` (timestamps). `kay-core` does NOT depend on `kay-session`; the loop emits events and the CLI pipes them through the session writer.

#### E-2 — JSONL write architecture: event-tap pattern

**Pattern:** After Phase 5's `run_turn` emits events through `tokio::mpsc`, the CLI's drain loop (in `run.rs`) fans out each event to: (a) stdout JSONL writer (existing Phase 5 path) AND (b) session transcript writer (new Phase 6 path).

This is a passive tap — `run_turn` and `kay-core` are unmodified. The session writer receives `AgentEventWire` structs (the serialized form already used for stdout) and appends them to the JSONL file. Zero changes to LOOP-* REQs.

#### E-3 — JSONL file path: `~/.kay/sessions/<session-id>/transcript.jsonl`

Per-session directory: `~/.kay/sessions/<uuid>/`. Contains:
- `transcript.jsonl` — append-only event stream
- `meta.json` — session metadata (id, title, persona, model, start_time, end_time, turn_count, parent_session_id)

Snapshots live separately: `~/.kay/snapshots/<session-id>/<turn>/<original-path>` (relative path preserved).

#### E-4 — SQLite schema (sessions.db)

```sql
CREATE TABLE schema_version (version INTEGER NOT NULL);
INSERT INTO schema_version VALUES (1);

CREATE TABLE sessions (
    id            TEXT PRIMARY KEY,          -- UUID v4
    title         TEXT NOT NULL DEFAULT '',  -- first user prompt, truncated to 120 chars
    persona       TEXT NOT NULL,             -- "forge" | "muse" | "sage"
    model         TEXT NOT NULL,
    status        TEXT NOT NULL DEFAULT 'active',  -- active | complete | aborted
    parent_id     TEXT REFERENCES sessions(id) ON DELETE SET NULL,
    start_time    TEXT NOT NULL,             -- ISO 8601 UTC
    end_time      TEXT,
    turn_count    INTEGER NOT NULL DEFAULT 0,
    cost_usd      REAL NOT NULL DEFAULT 0.0,
    jsonl_path    TEXT NOT NULL              -- absolute path to transcript.jsonl
);

CREATE INDEX idx_sessions_start ON sessions(start_time DESC);
CREATE INDEX idx_sessions_parent ON sessions(parent_id);
```

`schema_version` table enables future migrations. `parent_id` satisfies SESS-04. `jsonl_path` lets Phase 9 GUI open the file directly without knowing the storage layout.

#### E-5 — Resume semantics: cursor = line count

"Cursor position" is the number of complete JSONL lines written before the session was interrupted. On `kay resume <id>`, the CLI:
1. Opens the transcript JSONL, counts lines (fast: seek to EOF, scan back for last `\n`)
2. Writes a `SessionResumed { session_id, resume_turn: N }` synthetic event to stdout (so frontends can update their state)
3. Continues appending to the same JSONL file from line N+1

The model gets the full conversation history reconstructed from JSONL on resume (Phase 7 context window management applies here; Phase 6 just restores the raw transcript).

#### E-6 — Pre-edit snapshot: triggered by `fs_write` and `execute_commands`

When `fs_write` or `execute_commands` modifies a file, before the write:
1. Copy current file contents to `~/.kay/snapshots/<session-id>/<turn>/<relative-path>`
2. Record the snapshot in a `snapshots` table in sessions.db (session_id, turn, original_path, snapshot_path, file_hash)

`kay rewind` restores the most recent snapshot for a given session.

Snapshot byte cap: configurable `[session] snapshot_max_bytes = 104857600` (100 MiB). When exceeded, oldest snapshots are evicted (LRU by turn number).

#### E-7 — Export format: JSONL + manifest

`kay session export <id>` produces `<id>.tar.zst` containing:
- `transcript.jsonl` — the full event stream
- `manifest.json` — `{ session_id, kay_version, export_time, turn_count, model, persona, schema_version: 1 }`

Snapshots are NOT included (local recovery artifacts, not portable). Import strips the tarball and writes to `~/.kay/sessions/<new-id>/`.

#### E-8 — WAL mode + exclusive per-session JSONL

`PRAGMA journal_mode=WAL;` set on sessions.db open. Multiple CLI processes can read; one writer per session JSONL file (locked by a per-file `fcntl` advisory lock or by the OS write-append semantics). The index (sessions.db) is shared across sessions.

#### E-9 — Crate public API surface (minimal)

```rust
pub struct SessionStore { /* db + session dir */ }
impl SessionStore {
    pub fn open(root: &Path) -> Result<Self>;
    pub fn create_session(/*...*/) -> Result<Session>;
    pub fn resume_session(id: &Uuid) -> Result<Session>;
    pub fn fork_session(id: &Uuid) -> Result<Session>;
    pub fn list_sessions(limit: usize) -> Result<Vec<SessionSummary>>;
    pub fn export_session(id: &Uuid, dest: &Path) -> Result<()>;
    pub fn import_session(bundle: &Path) -> Result<Session>;
}

pub struct Session { /* id, transcript writer, snapshot writer */ }
impl Session {
    pub fn append_event(&mut self, wire: &AgentEventWire) -> Result<()>;
    pub fn record_snapshot(&mut self, original: &Path, content: &[u8]) -> Result<()>;
    pub fn close(self, status: SessionStatus) -> Result<()>;
}
```

`trybuild` compile-fail canaries for the public API contract (same pattern as Phase 5 kay-tools canaries).

#### E-10 — CLI subcommands added to `kay-cli`

```
kay session list [--limit N] [--format table|json]
kay session resume <id>
kay session fork <id>
kay session export <id> [--output PATH]
kay session import <bundle>
kay session replay <bundle> [--speed 1x|2x|max]
kay rewind [--session <id>] [--turn N]
```

`RunArgs` gets `--resume <id>` flag: `kay run --prompt "..." --resume <id>` resumes an existing session.

#### E-11 — TDD wave structure (7 waves)

| Wave | Scope | RED tests first |
|------|-------|----------------|
| W-1 | SessionStore::open + schema creation + schema_version | schema migrations, idempotent open |
| W-2 | Session::append_event + JSONL write + last-line recovery | crash-truncation, concurrent append |
| W-3 | SQLite index CRUD + list + resume lookup | parent_id FK, index queries |
| W-4 | Snapshot write + byte cap + LRU eviction | cap enforcement, path preservation |
| W-5 | fork_session + parent_id propagation | orphan semantics, FK cascade |
| W-6 | export + import + replay | round-trip, manifest validation, replay event order |
| W-7 | CLI integration (clap subcommands + E2E) | `kay session list` golden output, `kay rewind` restore |

#### E-12 — Carry-forward items from Phase 5 addressed in Phase 6

| INFO-ID | Item | Phase 6 action |
|---------|------|---------------|
| INFO-02 | No model-call circuit breaker (NoOpVerifier stub) | Real verifier wired in Phase 6 run_turn integration — `kay resume` must rehydrate verifier state |
| INFO-01 | pause-buffer unbounded | Session store bounds indirectly: paused events are buffered in memory only; replay from JSONL provides the durable version |
| INFO-03 | select! at 5-arm ceiling | No new arms added in Phase 6 — session write is synchronous in the drain loop, not a new select! arm |

### Open Questions for Discuss-Phase

| Q-ID | Question | Likely answer |
|------|----------|--------------|
| OQ-1 | Should `kay session replay` reconstruct the model context and re-run inference, or just replay the stored events? | Replay events only (stored events = source of truth); re-inference belongs to a future "fork + re-run" feature |
| OQ-2 | Does `kay rewind` restore only the most recent snapshot or allow N-step rewind? | Most recent only in Phase 6; N-step is Phase 10 |
| OQ-3 | Is `~/.kay/` the right config root, or should we respect `$XDG_DATA_HOME`? | Check forge_config for existing ForgeCode config root convention before deciding |
| OQ-4 | Should snapshot storage be per-session or per-project? | Per-session for v1 simplicity; per-project grouping in Phase 10 |
| OQ-5 | Does `kay session export` need to include the system prompt / persona YAML? | Yes — include persona name + model ID in manifest; full persona YAML is optional flag |
