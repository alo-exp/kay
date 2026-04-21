# Phase 6 Context — Locked Decisions for Planner

> **Date:** 2026-04-22
> **Phase:** 6 — Session Store + Transcript
> **Mode:** autonomous (§10e) — inline `gsd-discuss-phase` execution; all open questions resolved deterministically from upstream artifacts + codebase facts
> **Inputs scanned:** 06-BRAINSTORM.md (OQ-1..OQ-5), 06-QUALITY-GATES.md (QG-W1..W4), forge_config/src/reader.rs (config root pattern)

---

## Purpose

Lock decisions so downstream agents (`gsd-analyze-dependencies`, `gsd-plan-phase`) can act without asking the user again. Resolves 5 OQs from BRAINSTORM + 4 WARNs from QUALITY-GATES.

---

## Prior context reused (skip re-asking)

| Source | Locked facts |
|--------|-------------|
| PROJECT.md Key Decisions | DCO; signed tags; single merged binary; strict OpenRouter allowlist; JSON schema hardening |
| STATE.md | next_phase=6; Phase 5 complete on main (95412f0) |
| ROADMAP.md Phase 6 | SESS-01..05 + CLI-02; 5 SC; depends on Phase 5 |
| 06-BRAINSTORM.md | E-1..E-12: kay-session crate, event-tap, JSONL+SQLite schema v1, 7 TDD waves |
| 06-TEST-STRATEGY.md | 11 suites T-1..T-11; ≥56 tests; trybuild canaries; 3-OS CI |
| 06-QUALITY-GATES.md | 9/9 PASS design-time; 4 WARNs → DL-3..DL-6 below |
| forge_config/src/reader.rs | `FORGE_CONFIG` env var → `~/.forge` default pattern (parity reference for `KAY_HOME`) |

---

## Codebase scouting (this step)

**Config root pattern confirmed:**
- ForgeCode: `FORGE_CONFIG` env var → `~/forge` (legacy) → `~/.forge` (default) via `dirs::home_dir()`
- Kay Phase 6: `KAY_HOME` env var → `~/.kay` (default)
- `kay-session` reads config root via `SessionStore::open(root)` where `root` = `ConfigReader::base_path().join("sessions")`

**Existing JSONL-adjacent infrastructure:**
- `kay-tools/src/events_wire.rs` — `AgentEventWire` serde struct already implements `Display` (emits single JSONL line). Phase 6 JSONL writer reuses this directly — no new serialization layer.
- `forge_config/src/config.rs:194` — `session: Option<ModelConfig>` already exists in ForgeConfig. Phase 6 adds a `[session]` TOML section for `snapshot_max_bytes` configurable alongside existing session config.

**No existing session persistence** anywhere in workspace — Phase 6 is greenfield for `kay-session`.

---

## Locked decisions

### DL-1 — Replay semantics: stored events only (no re-inference)

**Question (OQ-1):** `kay session replay` — reconstruct model context + re-run inference, or replay stored events?

**Decision:** Replay stored events only. The JSONL transcript is the source of truth; `replay` emits events to stdout in sequence (same wire format as a live session). Re-inference ("fork + replay with fresh model calls") is a Phase 10 feature.

**Implementation:** `Session::replay(dest: &mut dyn Write)` reads transcript JSONL line-by-line and writes each to `dest`. No OpenRouter calls. No model context reconstruction.

**Rationale:** Phase 6's mandate is persistence and reproducibility, not re-execution. Re-inference changes event content (model is non-deterministic) and belongs with session forking in Phase 10.

---

### DL-2 — Rewind depth: most recent snapshot only (N-step deferred)

**Question (OQ-2):** `kay rewind` — most recent snapshot only, or N-step rewind?

**Decision:** Most recent snapshot only in Phase 6. `kay rewind [--session <id>] [--turn N]` restores the snapshot captured at turn N (default: highest turn with a snapshot). N-step navigation (rewind to turn 5, then turn 3) requires UX design better suited to Phase 10's session management panel.

**Implementation:** `kay rewind` queries `snapshots` SQLite table for `MAX(turn)` snapshots in the session, restores each file's content. With `--turn N`, restores snapshots from turn N specifically.

**Rationale:** Turn N is acceptable (user can specify which turn to rewind to) — what's deferred is interactive N-step stepping. This satisfies SESS-02 ("single-step rewind") without scope creep.

---

### DL-3 — Config root: `~/.kay/` with `KAY_HOME` env var override

**Question (OQ-3):** `~/.kay/` vs `$XDG_DATA_HOME/kay`?

**Decision:** `~/.kay/` as default, with `KAY_HOME` env var override — same pattern as ForgeCode's `FORGE_CONFIG` → `~/.forge`.

**Implementation:**
```rust
pub fn kay_home() -> PathBuf {
    if let Ok(path) = std::env::var("KAY_HOME") {
        return PathBuf::from(path);
    }
    dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".kay")
}
```

Session store lives at `~/.kay/sessions/`. Snapshots at `~/.kay/sessions/<id>/snapshots/`.

**Rationale:** XDG data dirs is the Linux standard but ForgeCode doesn't use XDG. Parity with ForgeCode's config pattern avoids a config location conflict when Kay migrates existing ForgeCode users. `KAY_HOME` enables CI test isolation (point to `tempdir`).

**NOTE for planner:** Add `dirs = "5"` to `kay-session/Cargo.toml` (already in workspace if not — check).

---

### DL-4 — Snapshot storage: per-session, under `~/.kay/sessions/<id>/snapshots/`

**Question (OQ-4):** Per-session or per-project?

**Decision:** Per-session. Snapshots live at `~/.kay/sessions/<session-id>/snapshots/<turn>/<rel-path>`. The session directory is the natural container since snapshots are only meaningful within their session's turn context.

**Implementation:** `record_snapshot` takes `(original_path: &Path, bytes: &[u8])` and writes to:
`session_dir/snapshots/<turn_number>/<original_path relative to CWD at session creation>`.

Per-project grouping (grouping all sessions for a project together) is deferred to Phase 10.

---

### DL-5 — Export format: transcript JSONL + manifest JSON (no compression in Phase 6)

**Question (OQ-5):** Full persona YAML in export, or just name + model ID?

**Decision:** Manifest includes: `{ session_id, kay_version, schema_version: 1, turn_count, model, persona_name, start_time, export_time }`. Full persona YAML: optional with `--include-persona` flag. Default export is a directory (not tarball) with two files.

**Implementation:**
- `kay session export <id> [--output DIR] [--include-persona]`
- Default output: `<id>/` directory with `transcript.jsonl` + `manifest.json`
- With `--include-persona`: also writes `persona.yaml` (copy of bundled YAML)
- No tar/zstd compression in Phase 6 (avoids new crate deps; CLI users can compress themselves)

**Rationale:** No compression keeps the dep count low. TB 2.0 submission only needs transcript + manifest (the reviewer runs `kay session import` to replay). Personas are already in the binary; bundling them is optional convenience.

---

### DL-6 — Path traversal mitigation: CWD boundary enforcement

**From QG-W1:** `record_snapshot` / `kay rewind` path traversal.

**Decision:** `SessionStore` records `cwd: PathBuf` at session creation (stored in `sessions` SQLite table and `meta.json`). `record_snapshot` validates that `canonical(original_path).starts_with(&session.cwd)` before writing. If not, returns `Err(PathTraversalRejected { path, session_cwd })`.

**Implementation additions to SQLite schema:**
```sql
ALTER TABLE sessions ADD COLUMN cwd TEXT NOT NULL DEFAULT '';
```
(included in the v1 schema from the start — not an ALTER, just added to `CREATE TABLE`)

`import_session` validates all snapshot paths from manifest against the exported `cwd` before creating the session.

---

### DL-7 — Session title as untrusted data

**From QG-W2:** Session title stored verbatim from user prompt.

**Decision:** `title` in SQLite and `manifest.json` is stored verbatim (truncated to 120 chars, no sanitization — it's the user's own input). Phase 6 PLAN.md documents: **ANY consumer that interpolates `session.title` into a model system prompt MUST delimit it as `[USER_DATA: ...]` to prevent stored prompt injection.** This constraint is documented as a comment in the schema definition and in `meta.json` serialization.

Phase 7 (context engine) is the first planned consumer of session metadata in model context — it must respect this constraint.

---

### DL-8 — `kay rewind` confirmation pattern

**From QG-W3:** Rewind is destructive.

**Decision:**
- Default (interactive TTY): prompt `"Restore <path> from turn N snapshot? [y/N]: "` before each file
- `--force` flag: skip all confirmations (safe for scripting)
- `--dry-run` flag: print what would be restored, no writes
- Non-interactive (no TTY, no `--force`): return `Err(ConfirmationRequired)` with message `"Use --force to restore in non-interactive mode"`

---

### DL-9 — Transcript deletion detection

**From QG-W4:** JSONL file externally deleted during session.

**Decision:** `Session` struct holds an open `File` handle (append mode) from `create_session` / `resume_session`. On `append_event`, I/O errors from the write (including `ENOENT` / `EBADF`) are mapped to `Err(TranscriptDeleted { session_id, path })`. On this error, the caller (`kay-cli` drain loop) marks the session `status = "lost"` in SQLite and exits with code 1.

---

## Wave plan summary (for planner)

| Wave | Core deliverable | Key decisions used |
|------|-----------------|-------------------|
| W-1 | `SessionStore::open`, SQLite schema v1, `kay_home()` | DL-3, DL-6 (cwd column) |
| W-2 | `Session::append_event`, JSONL write, last-line recovery | DL-9 (TranscriptDeleted) |
| W-3 | SQLite CRUD, `list_sessions`, `resume_session` | DL-6 (path validation) |
| W-4 | `record_snapshot`, byte cap, LRU eviction | DL-4 (per-session), DL-6 (boundary) |
| W-5 | `fork_session`, parent_id FK | SESS-04 |
| W-6 | `export_session`, `import_session`, `replay` | DL-1 (events only), DL-5 (dir format), DL-7 (title) |
| W-7 | `kay session *` CLI + `kay rewind` | DL-2 (turn N), DL-8 (--force/--dry-run) |
