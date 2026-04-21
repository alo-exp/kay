# Phase 6 Quality Gates — Session Store + Transcript

> **Date:** 2026-04-22
> **Phase:** 6 — Session Store + Transcript
> **Mode:** design-time (pre-plan; no PLAN.md yet)
> **Overall: ✅ PASS — proceed to discuss-phase + planning**

---

## Quality Gates Report

| Dimension     | Result | Key evidence |
|---------------|--------|-------------|
| Modularity    | ✅ PASS | 5 focused modules (store, transcript, snapshot, export, session); each has one-sentence responsibility; no circular imports; each wave touches ≤3 source files |
| Reusability   | ✅ PASS | `kay-session` is a new crate with no duplication; consumer-perspective API (E-9); 2+ planned consumers (kay-cli now, kay-tauri Phase 9) |
| Scalability   | ✅ PASS | SQLite WAL mode + start_time + parent_id indexes; `list_sessions(limit)` paginated; snapshot byte cap + LRU eviction; horizontal scaling N/A (local CLI tool) |
| Security      | ✅ PASS | 2 WARNs captured below; rusqlite parameterized queries; PathBuf everywhere; cargo-audit CI gate |
| Reliability   | ✅ PASS | WAL mode + last-line crash recovery (E-5); all I/O returns Result; idempotent open; 1 edge case noted below |
| Usability     | ✅ PASS | Intuitive CLI names; error types have actionable messages; `kay rewind` destructive action needs `--force` flag |
| Testability   | ✅ PASS | `SessionStore::open(root: &Path)` injectable; all tests use `tempfile::TempDir`; pure core functions; no global state |
| Extensibility | ✅ PASS | `schema_version` table enables migrations; `parent_session_id` reserved for Phase 10; `SessConfig` configurable policy |
| AI/LLM Safety | ✅ PASS | QG-C4 gate unmodified (event_filter.rs 100% CI); replay emits to stdout only; 1 WARN on session title as untrusted data |

---

## Failures Requiring Redesign

*None — all 9 dimensions pass.*

---

## WARNs (non-blocking; must appear in PLAN.md enforcement contracts)

### QG-W1 — Path traversal in `record_snapshot` + `kay rewind`

**Risk:** `original_path` stored in sessions.db could be `../../.ssh/authorized_keys` if a malicious actor crafts an import bundle. `kay rewind` writing to that path writes outside the working directory.

**Required mitigation in PLAN.md:**
- `record_snapshot` must canonicalize `original_path` and verify it starts with the project working directory (or the user's home directory minus sensitive dirs)
- `import_session` must validate all snapshot paths in the manifest before creating the session
- `kay rewind` must refuse to write to paths outside the session's recorded working directory

**Test gate:** T-5 must include a path-traversal attempt test; reject must return `Err(PathTraversalRejected)`.

### QG-W2 — Session title as untrusted data

**Risk:** `title` field in sessions.db / manifest.json is derived from the first user prompt (untrusted). If Phase 7 (context engine) interpolates session title into a model prompt for context retrieval, it creates a stored prompt injection vector.

**Required mitigation:**
- Session title is stored verbatim as user-supplied data
- Any consumer that injects session title into a model prompt (Phase 7+) MUST treat it as `[DATA]` not `[INSTRUCTION]` with clear delimiter in the prompt message structure
- Phase 6 PLAN.md must document this constraint in the `meta.json` / SQLite schema comments

### QG-W3 — `kay rewind` destructive action needs `--force` flag

**Risk:** `kay rewind` overwrites the current file with the snapshot content. A misfire destroys the user's current work.

**Required mitigation in PLAN.md:**
- Default: `kay rewind` prompts "Overwrite <path> with snapshot from turn N? [y/N]"
- With `--force` flag: skip confirmation (for scripting)
- With `--dry-run` flag: show what would be restored without overwriting

### QG-W4 — `append_event` must handle externally deleted JSONL

**Risk:** If the user manually deletes `~/.kay/sessions/<id>/transcript.jsonl` while a session is running, the next `append_event` should return a clear error, not panic.

**Required mitigation:** `append_event` opens the file with `OpenOptions::append(true)` on each call (or caches the handle and catches `EBADF`/`ENOENT` → `Err(TranscriptDeleted)`).

---

## Carry-Forward Enforcement Contracts

These constraints apply to the PLAN.md and all implementation waves:

| ID | Contract | Enforcement mechanism |
|----|----------|-----------------------|
| QG-C4 | `event_filter.rs` 100% line + branch — SandboxViolation MUST NOT be re-injected into model context | CI `coverage-event-filter` job (no Phase 6 changes to this file) |
| QG-C5 | Session JSONL stores events as DATA; replay emits to stdout ONLY — never feeds back into `run_turn` model context | Code review gate + T-10 smoke test |
| QG-C6 | `original_path` in snapshots must be validated against working directory boundary before write | T-5 path-traversal test; `PathTraversalRejected` error type |
| QG-C7 | Session title stored verbatim; consumers constructing model prompts must delimit it as `[DATA]` | PLAN.md comment; Phase 7 dependency documented |
| QG-C8 | `kay rewind` always requires confirmation or `--force` flag; never silently overwrites | CLI integration test `rewind_requires_force_without_flag` |

---

## N/A Items (for future phases)

| Item | Dimension | Deferred to |
|------|-----------|------------|
| Horizontal scaling / stateless design | Scalability | N/A — local CLI tool with intentional local state |
| HTTP caching / connection pooling | Scalability | N/A — embedded SQLite, not a web service |
| Screen reader / color contrast | Usability | Phase 9 (Tauri GUI) / Phase 9.5 (TUI) |
| Multi-agent trust boundary isolation | AI/LLM Safety | Phase 10 (multi-session manager) |

---

## Backlog Items Captured

Per Step 5 of silver-quality-gates: these advisory items are deferred to future phases, not Phase 6.

1. **Snapshot auto-compaction** — automatic cleanup of oldest sessions when `~/.kay/sessions/` exceeds a configurable total cap. Deferred to a maintenance release after Phase 9.
2. **Session title sanitization UI** — Phase 9 Tauri GUI should show session titles with HTML escaping in the session list panel. Deferred to Phase 9 PLAN.md.
3. **Export bundle signature verification** — sign exported bundles with the ED25519 release key so import can verify provenance. Deferred to Phase 11 (release pipeline).
