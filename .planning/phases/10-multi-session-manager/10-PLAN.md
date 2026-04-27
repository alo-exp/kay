# Phase 10 Plan — Multi-Session Manager + Project Settings

**Plan file:** 10-PLAN.md
**Spec:** `docs/superpowers/specs/2026-04-24-phase10-multi-session-manager-design.md`
**Branch:** `phase/10-multi-session-manager` (cut from `main` after Phase 9 + 9.5 merge)
**TDD:** RED before GREEN per wave. All commits signed with `Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>`.

## Pre-Checks

- [x] Phase 9 PR #18 merged to main (dependency chain B)
- [x] Phase 9.5 PR #19 merged to main
- [x] `cargo check --workspace` passes on `main` at plan start
- [x] `kay-tauri` commands: 3 exist (start_session, stop_session, get_session_status)

## WAVES

### WAVE 1: SessionManager trait + state (RED → GREEN)

**RED:** `crates/kay-tauri/src/session_manager.rs` stub — `SessionManager` trait with `list_sessions`, `pause_session`, `resume_session`, `fork_session`, `kill_session` methods that return `todo!()`. `SessionInfo` + `SessionStatus` types defined. Tests: `session_manager_tests.rs` — 3 tests that fail because trait methods panic.

**GREEN:** Full `SessionManagerImpl` backed by Phase 6 `SessionStore`. Uses `tokio::sync::RwLock` for session map. Implements all 5 methods. Tests: `session_manager_tests.rs` — 3 tests pass.

### WAVE 2: 8 new Tauri commands (RED → GREEN)

**RED:** `crates/kay-tauri/src/commands.rs` — add 8 new command stubs that return `todo!()`. Specta types don't exist yet. Tests fail because stubs panic.

**GREEN:** Full implementations of all 8 commands. Extend specta export. Tests: command unit tests with mock `SessionManager`.

**New commands:**
1. `list_sessions() -> Vec<SessionInfo>`
2. `pause_session(session_id: String) -> Result<()>`
3. `resume_session(session_id: String) -> Result<()>`
4. `fork_session(session_id: String, persona: Option<String>) -> Result<String>`
5. `kill_session(session_id: String) -> Result<()>`
6. `get_session_events(session_id: String, from_turn: Option<u32>) -> Vec<IpcAgentEvent>`
7. `save_project_settings(settings: ProjectSettings) -> Result<()>`
8. `load_project_settings(project_path: String) -> Result<Option<ProjectSettings>>`
9. `bind_api_key(provider: String, key: String) -> Result<()>` (added: keyring command)
10. `get_api_key_fingerprint(provider: String) -> Result<Option<String>>` (added: keyring command)

### WAVE 3: IpcAgentEvent additions (RED → GREEN)

**RED:** `crates/kay-tauri/src/ipc_event.rs` — add 6 new variants to `IpcAgentEvent`. `From<AgentEvent>` impls missing. `ipc_event_tests.rs` fails.

**GREEN:** All 6 variants implemented with `From<AgentEvent>`. Round-trip tests for all 6 pass.

**New variants:**
1. `SessionSpawned { session_id, persona, created_at }`
2. `SessionPaused { session_id, paused_at }`
3. `SessionResumed { session_id, resumed_at }`
4. `SessionForked { parent_id, child_id }`
5. `ApprovalRequested { tool_name, command, sandbox_status }`
6. `ApprovalDecision { tool_name, approved, decided_at }`

### WAVE 4: ProjectSettings + keyring (RED → GREEN)

**RED:** `crates/kay-tauri/src/project_settings.rs`, `crates/kay-tui/src/keyring.rs` stub files. Tests fail.

**GREEN:** Full `ProjectSettings` struct, `OsKeyring` trait + platform implementations. `ModelTier` enum. Tests: serialization, keyring store/retrieve (mocked).

### WAVE 5: Command approval (Tauri) (RED → GREEN)

**RED:** `crates/kay-tauri/src/command_approval.rs` stub. React `CommandApprovalDialog.tsx` doesn't exist.

**GREEN:** `ApprovalRequest` event flows through IPC. React dialog renders with Enter/Escape keyboard shortcuts. `ApprovalStore` persists decisions within session.

### WAVE 6: TUI session control (RED → GREEN)

**RED:** `crates/kay-tui/src/session_manager.rs` stub with `todo!()`. `app.rs` keyboard handlers don't exist.

**GREEN:** `TuiSessionManager` backed by IPC calls to kay-tauri (or direct session store access). Keyboard shortcuts wired: n/p/r/f/x/s/?.

### WAVE 7: Settings panel (Tauri + TUI) (RED → GREEN)

**RED:** `crates/kay-tauri/ui/src/components/SettingsPanel.tsx` doesn't exist. TUI settings overlay doesn't exist.

**GREEN:** Full settings panel with tabs: Session, Model, Verifier, Sandbox. Persist to `~/.kay/projects/<path>/settings.json`. TUI overlay with ratatui popup.

### WAVE 8: Integration tests + CI (RED → GREEN)

**RED:** `crates/kay-tauri/tests/session_manager_integration.rs` stub. `scripts/check-bindings.sh` doesn't verify all 11 commands.

**GREEN:** Full integration test: spawn/pause/resume/fork/kill cycle. check-bindings.sh updated.

## VERIFICATION CRITERIA

After all waves complete, verify:
1. `cargo test -p kay-tauri` ≥ 25 tests pass
2. `cargo test -p kay-tui` ≥ 10 tests pass
3. `bash scripts/check-bindings.sh` passes (all 11 commands bound)
4. `cargo check -p kay-tauri --lib --bins` clean (no warnings)
5. `cargo fmt -p kay-tauri -p kay-tui` clean

## SUCCESS CRITERIA (from spec §4)

1. Session list sorted by last_active descending — GUI and TUI
2. Spawn/pause/resume/fork/kill from GUI without touching CLI
3. OpenRouter key bound to OS keychain; retrievable after restart
4. Model picker tiered list; Recommended shows Exacto allowlist only
5. Project settings save/load; persist across restarts
6. Command approval dialog on first tool call when enabled; subsequent skip
7. TUI keyboard shortcuts (n/p/r/f/x/s/?) functional
8. All 11 commands have tauri-specta TypeScript bindings
9. `cargo test -p kay-tauri` ≥ 25 unit tests pass
10. `cargo test -p kay-tui` ≥ 10 unit tests pass