# Phase 6 Test Strategy — Session Store + Transcript

> **Date:** 2026-04-22
> **Phase:** 6 — Session Store + Transcript
> **Crate:** `crates/kay-session/` (new)
> **TDD discipline:** RED first — every wave starts with failing tests before any production code
> **CI matrix:** macOS-14 (arm64), ubuntu-latest, windows-latest

---

## Test Pyramid

```
       /  CLI E2E (T-8)  \            assert_cmd subprocess tests
      /  Integration (T-4..T-7) \     cross-module flows (resume, snapshot, export)
     /     Unit (T-1..T-3)       \    SessionStore, JSONL writer, SQLite CRUD
    /   Property (T-2p, T-7p)     \  proptest JSONL round-trip + export stability
   /  trybuild canaries (T-9)      \ compile-fail API contract enforcement
```

---

## Coverage Targets

| Scope | Target | Rationale |
|-------|--------|-----------|
| `kay-session/src/` line coverage | ≥ 90% | Business-critical persistence; no dead branches |
| `kay-session/src/store.rs` branch coverage | ≥ 85% | Error paths (cap exceeded, schema mismatch, corrupt JSONL) |
| `kay-session/src/snapshot.rs` branch coverage | ≥ 85% | LRU eviction + byte cap enforcement |
| QG-C4 gate (`event_filter.rs`) | 100% line + 100% branch | **Must stay green** — no Phase 6 changes allowed to weaken it |

---

## Dev-Dep Requirements for `kay-session`

New crates needed (not yet in workspace):
- `rusqlite = { version = "0.32", features = ["bundled"] }` — bundled SQLite avoids system lib dependency on CI
- `zstd` — NOT needed (export format simplified to directory, not tarball)

Reuse from workspace:
- `uuid = { workspace = true }` (already workspace dep with `v4 + serde`)
- `chrono = { workspace = true }` (already workspace dep with `serde`)
- `tempfile = { workspace = true }` (already workspace dev-dep)
- `insta = { workspace = true }` (already workspace dev-dep)
- `proptest = "1"` (local dev-dep; not yet workspace)
- `trybuild = "1.0"` (local dev-dep; not yet workspace)
- `assert_cmd = { workspace = true }` (already workspace dev-dep — for T-8)
- `predicates = { workspace = true }` (already workspace dev-dep — for T-8)

---

## Test Suites

### T-1 — SessionStore Initialization (Unit)

**File:** `tests/store_init.rs`

| Test | What it verifies |
|------|-----------------|
| `open_creates_db` | `SessionStore::open(dir)` creates `sessions.db`, `schema_version` table has value 1, `sessions` table exists |
| `open_is_idempotent` | Calling `open` twice on same dir returns `Ok` both times; second open finds existing schema |
| `open_sets_wal_mode` | `PRAGMA journal_mode` returns `"wal"` after open |
| `open_schema_version_mismatch` | If `schema_version` = 99, `open` returns `Err(SchemaVersionMismatch { found: 99, expected: 1 })` |
| `sessions_table_columns` | `PRAGMA table_info(sessions)` returns all expected columns: id, title, persona, model, status, parent_id, start_time, end_time, turn_count, cost_usd, jsonl_path |

**RED commit:** all 5 tests compile but panic/fail. **GREEN commit:** `SessionStore::open` implemented.

---

### T-2 — JSONL Transcript Write (Unit + Property)

**File:** `tests/transcript.rs`

| Test | What it verifies |
|------|-----------------|
| `append_writes_valid_json_newline` | Each `append_event` writes one `{...}\n` line; file grows by exactly that line |
| `append_round_trip` | Write `AgentEventWire::TextDelta`, read back via `serde_json`, fields match |
| `last_line_truncation` | Simulate crash: open file, write partial line (no `\n`), close fd; reopen via `SessionStore::resume_session`, verify partial line is stripped |
| `last_line_empty_file_ok` | Empty transcript (zero writes, crashed before first append) — resume returns `Ok` with 0 turns |
| `line_count_matches_turn_count` | After N `append_event` calls, `count_lines(path)` = N |

**File:** `tests/transcript_property.rs`

| Test | What it verifies |
|------|-----------------|
| `jsonl_round_trip_proptest` | Arbitrary `Vec<AgentEventWire>` written then read back; every field matches (proptest 10k iters) |
| `no_newline_in_json_body` | No generated JSONL line (except the trailing `\n`) contains a bare newline character (validates single-line-per-event invariant) |

**RED commit:** tests compile, all fail. **GREEN commit:** `Session::append_event` + last-line recovery implemented.

---

### T-3 — SQLite Sessions Index (Unit)

**File:** `tests/index.rs`

| Test | What it verifies |
|------|-----------------|
| `create_session_inserts_row` | `create_session(...)` inserts a row; `list_sessions(1)` returns it |
| `list_sessions_ordered_by_start_time_desc` | Insert sessions with different `start_time`; verify list order is newest-first |
| `list_sessions_limit_respected` | Insert 10 sessions, `list_sessions(3)` returns exactly 3 |
| `close_session_sets_status_and_end_time` | After `session.close(SessionStatus::Complete)`, row has `status="complete"` and non-null `end_time` |
| `resume_by_id_returns_correct_path` | `resume_session(uuid)` returns `Session` whose `jsonl_path` matches the stored path |
| `resume_unknown_id_returns_err` | `resume_session(Uuid::new_v4())` returns `Err(SessionNotFound)` |
| `parent_id_fk_set_null_on_delete` | Fork child references parent; delete parent row; child row has `parent_id = NULL` (FK `ON DELETE SET NULL`) |
| `session_list_summary_fields` | `list_sessions` returns `SessionSummary` with all display fields: id, title, status, start_time, turn_count, cost_usd |

**RED commit:** schema + CRUD stubs. **GREEN commit:** full SQLite implementation.

---

### T-4 — Session Resume Cursor (Integration)

**File:** `tests/resume.rs`

| Test | What it verifies |
|------|-----------------|
| `resume_restores_turn_count` | Write 7 events to transcript, close session; resume returns `Session` with `current_turn = 7` |
| `resume_appends_after_existing_events` | Resume session, append 3 more events; transcript has 10 lines total |
| `resume_partial_write_recovery` | Truncate transcript at byte 50 (mid-line), resume; turn_count = number of complete lines before truncation point |
| `resume_emits_session_resumed_synthetic_event` | On resume, first event appended to transcript is `SessionResumed { session_id, resume_turn: N }` |
| `resume_updates_status_to_active` | After resume, SQLite row `status = "active"` |

**RED commit:** resume stubs. **GREEN commit:** resume with cursor counting + synthetic event.

---

### T-5 — Pre-edit Snapshots (Unit + Integration)

**File:** `tests/snapshot.rs`

| Test | What it verifies |
|------|-----------------|
| `record_snapshot_writes_file` | `session.record_snapshot(path, bytes)` creates file at `~/.kay/snapshots/<id>/<turn>/<rel-path>` |
| `snapshot_path_preserves_subdirectory` | Snapshot of `src/main.rs` → path includes `src/main.rs` suffix (PathBuf join, not string concat) |
| `snapshot_byte_cap_no_eviction_below_cap` | Write 10 MiB of snapshots into 100 MiB cap session; all snapshots present |
| `snapshot_byte_cap_triggers_lru_eviction` | Write snapshots totalling 150 MiB into a 100 MiB cap; oldest-turn snapshots evicted; newest retained |
| `snapshot_hash_matches_original` | SHA-256 of stored snapshot = SHA-256 of original bytes passed to `record_snapshot` |
| `rewind_restores_most_recent_snapshot` | Write snapshot for turn 1, snapshot for turn 2 (different content); `rewind` restores turn 2 content to original path |
| `rewind_no_snapshot_returns_err` | `rewind` on session with no snapshots returns `Err(NoSnapshotsAvailable)` |
| `snapshot_cap_default_is_100mib` | Default `SessConfig::default()` has `snapshot_max_bytes = 104_857_600` |

**RED commit:** snapshot struct stubs. **GREEN commit:** snapshot write + cap + LRU.

---

### T-6 — Fork Semantics (Unit)

**File:** `tests/fork.rs`

| Test | What it verifies |
|------|-----------------|
| `fork_sets_parent_id` | `fork_session(parent_id)` creates child with `parent_id` column = parent's UUID |
| `fork_creates_independent_jsonl` | Child session has a distinct `jsonl_path` from parent |
| `fork_inherits_persona_and_model` | Child session has same `persona` and `model` as parent (fork starts from same config) |
| `fork_child_status_is_active` | Forked child has `status = "active"` regardless of parent status |
| `fork_parent_deletion_sets_null` | Delete parent session; child row has `parent_id = NULL` |
| `fork_preserves_turn_count_independence` | Appending to child does not increment parent's `turn_count` |

**RED commit:** fork stub. **GREEN commit:** fork implementation.

---

### T-7 — Export + Import + Replay (Integration + Property)

**File:** `tests/export.rs`

| Test | What it verifies |
|------|-----------------|
| `export_creates_transcript_and_manifest` | Output dir contains `transcript.jsonl` + `manifest.json` |
| `manifest_has_required_fields` | `manifest.json` deserializes to `ExportManifest` with: session_id, kay_version, schema_version=1, turn_count, model, persona |
| `export_does_not_include_snapshots` | Output dir contains no files outside `transcript.jsonl` + `manifest.json` |
| `import_creates_new_session` | `import_session(dir)` creates new session in sessions.db with new UUID |
| `import_new_uuid_not_original` | Imported session ID ≠ original session ID |
| `import_transcript_matches_original` | Imported JSONL line count = exported line count; byte-for-byte equality |
| `replay_emits_events_to_stdout` | `Session::replay(dest)` emits one JSONL line per stored event (verified via output capture) |
| `replay_preserves_event_order` | Events replayed in same sequence as stored (monotonic turn index) |

**File:** `tests/export_property.rs`

| Test | What it verifies |
|------|-----------------|
| `export_import_export_stable` | export → import → export: second export's `transcript.jsonl` byte-equals first export's (proptest arbitrary session content) |

**RED commit:** export stubs. **GREEN commit:** export + import + replay.

---

### T-8 — CLI E2E (assert_cmd subprocess)

**File:** `crates/kay-cli/tests/session_e2e.rs`

| Test | What it verifies |
|------|-----------------|
| `session_list_empty` | `kay session list` on fresh store → human-readable "No sessions found" |
| `session_list_table_format` | After creating a session, `kay session list` outputs table with header row |
| `session_list_json_format` | `kay session list --format json` outputs valid JSON array |
| `session_export_creates_files` | `kay session export <id> --output /tmp/X` → creates `transcript.jsonl` + `manifest.json` |
| `session_import_round_trip` | Export then `kay session import <dir>` → session appears in list |
| `session_replay_emits_jsonl` | `kay session replay <dir>` → stdout has N JSONL lines matching original |
| `rewind_restores_file` | Create snapshot via tool call stub, run `kay rewind --session <id>`, verify file restored |
| `rewind_no_snapshot_exit_1` | `kay rewind` with no snapshots → exit code 1 + stderr message |
| `session_fork_creates_child` | `kay session fork <id>` → new session in list with parent_id in `--format json` |
| `resume_flag_on_run` | `kay run --prompt TEST:done --resume <id>` → exit 0, transcript extended |

**RED commit:** command dispatch stubs (commands exist but return "unimplemented"). **GREEN commit:** full CLI wiring.

---

### T-9 — trybuild Compile-Fail Canaries

**File:** `crates/kay-session/tests/compile_fail_harness.rs`

| Canary | What it enforces |
|--------|-----------------|
| `session_not_cloneable.fail.rs` | `Session` does not derive `Clone` (file handle is exclusive) |
| `append_after_close.fail.rs` | `Session::close(self)` consumes self; subsequent `append_event` is a compile error |
| `store_open_requires_path.fail.rs` | `SessionStore::open` requires a `&Path`, not a raw `&str` (type safety) |

---

### T-10 — QG-C4 Regression Guard

**Approach:** No changes to `crates/kay-core/src/event_filter.rs`. The `coverage-event-filter` CI job continues to enforce 100% line + branch.

**Smoke test in `tests/export.rs`:**
- Export a session containing a `SandboxViolation` event
- Verify the event is present in `transcript.jsonl` (stored correctly)
- Verify the event is NOT present in any `ModelInput` payload (storage ≠ re-injection; the session store is write-side only)

---

### T-11 — 3-OS CI Matrix Considerations

| Platform concern | Mitigation |
|-----------------|-----------|
| Windows path separators | Use `PathBuf::join` throughout; never string-concatenate paths |
| Windows advisory file lock | Use `fs2::FileExt::try_lock_exclusive` (cross-platform); document behavior difference |
| macOS `O_EXLOCK` vs Linux `fcntl` | Abstract behind `advisory_lock` helper in `kay-session`; test concurrent-append on all three OSes |
| SQLite on Windows | `rusqlite` bundled feature avoids system SQLite dependency |
| Temp dir paths with spaces | All tests use `tempfile::TempDir`; no hardcoded `/tmp` paths |

---

## Wave-to-Test-Suite Mapping

| Wave | Tests run GREEN at wave exit |
|------|------------------------------|
| W-1: SessionStore::open + schema | T-1 (all 5) |
| W-2: JSONL append + recovery | T-2 unit (5) + T-2 property (2) |
| W-3: SQLite CRUD + list + resume | T-3 (8) + T-4 (5) |
| W-4: Snapshot write + cap + LRU | T-5 (8) |
| W-5: Fork + parent_id | T-6 (6) |
| W-6: Export + import + replay | T-7 integration (8) + T-7 property (1) + T-9 canaries (3) |
| W-7: CLI subcommands | T-8 (10) + T-10 smoke |

**Total test count target:** ≥ 56 tests across 11 suites (unit + integration + property + E2E + canaries).

---

## TDD Commit Cadence per Wave

```
Wave N:
  commit 1 (RED):   test file + stubs; cargo test → N failures, 0 compilation errors
  commit 2 (GREEN): production code; cargo test → all pass
  commit 3 (REFACTOR, optional): clippy clean, dead code removed, docs updated
  All commits: DCO-signed (Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>)
```
