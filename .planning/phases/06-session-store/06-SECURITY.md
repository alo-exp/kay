---
phase: 6
date: 2026-04-22
status: passed
---

# Phase 6 Security Review — Session Store + Transcript

## Threat Model Coverage

| Threat ID | Category | Component | Status | Mitigation Evidence |
|-----------|----------|-----------|--------|---------------------|
| TM-01 | Tampering | Path traversal via session import | MITIGATED | `snapshot.rs:37-48`: `original_path.canonicalize()` + `starts_with(cwd_canonical)` guard before any file write |
| TM-02 | Tampering | Session title injection via list output | MITIGATED | `session.rs:list`: title rendered as plain `{}` string (not HTML); JSON output via `serde_json::json!` escapes correctly |
| TM-03 | Denial of Service | Rewind destructive overwrite | MITIGATED | `session.rs:rewind_cmd`: `--dry-run` uses `list_rewind_paths()` (no restore); non-TTY without `--force` → `ConfirmationRequired` error |
| TM-04 | Tampering | SandboxViolation re-injection via event-tap | MITIGATED | Event-tap is write-only fan-out; `event_filter.rs` unchanged (QG-C4 verified via `git diff`); no path back to `run_turn` |
| TM-05 | Tampering | Transcript deletion during active session | MITIGATED | `run.rs:431-438`: append_event I/O error → `mark_session_lost` in SQLite + `return Err` — session not silently continued (DL-9) |
| TM-06 | Information Disclosure | JSONL transcript stored in plaintext | ACCEPTED | Stored in `~/.kay/sessions/<id>/transcript.jsonl` under user home directory; OS filesystem permissions provide access control. Encryption deferred to Phase 10. |
| TM-07 | Denial of Service | Snapshot byte cap bypass | MITIGATED | `snapshot.rs:77-108`: `SUM(byte_size)` check + LRU eviction loop after every `record_snapshot`; 100 MiB default cap in `SessConfig` |
| TM-08 | Spoofing | Import of tampered manifest | MITIGATED | `import_session` in `export.rs`: manifest deserialized via serde with typed struct; `schema_version` field checked; UUID is re-generated on import (not trusted from manifest) |

## Key Security Properties

1. **Path traversal guard (DL-6, QG-C6)**: `record_snapshot` canonicalizes both the target path and the session cwd, then verifies `starts_with`. Any symlink-traversal or `../` attempt is rejected with `PathTraversalRejected` before the first byte is written.

2. **QG-C4 intact**: Zero changes to `crates/kay-core/src/event_filter.rs`. The event-tap fan-out in `run.rs` is a passive write-only consumer; it does not feed events back to the model context. Verified: `git diff HEAD -- crates/kay-core/src/event_filter.rs` produces no output.

3. **DL-8 confirmation gate**: `kay rewind` in non-interactive (non-TTY) environments requires `--force`. Without it, `ConfirmationRequired` error causes exit 1. `--dry-run` path uses `list_rewind_paths()` (query-only, no file writes). Verified by `rewind_dry_run_no_write` E2E test.

4. **DL-9 transcript deletion**: On any I/O error from `append_event`, the drain loop calls `mark_session_lost` to update SQLite status, then returns an error that causes exit 1. Session is not silently continued with a broken transcript.

5. **Export UUID re-generation**: `import_session` generates a fresh `Uuid::new_v4()` regardless of the manifest's `session_id`. The imported session cannot impersonate an existing session by UUID collision.

## No New Attack Surface

Phase 6 adds file I/O (JSONL writes, SQLite) and CLI subcommands, but:
- No network I/O in kay-session
- No shell execution
- No `unsafe` blocks introduced
- All file paths validated through library functions with canonicalization guards
