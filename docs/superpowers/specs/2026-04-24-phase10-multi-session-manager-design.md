# Phase 10 — Multi-Session Manager + Project Settings
**Created:** 2026-04-24
**Author:** silver bullet (autonomous planning)
**Branch:** `phase/10-multi-session-manager`

## 1. Context

### 1.1 Prior Phases
- Phase 9 (Tauri Desktop Shell): `kay-tauri` with `IpcAgentEvent` IPC, `Channel<AgentEvent>` streaming, React 19 UI with tool-call timeline and token/cost meter, 4h memory canary. 3 commands: `start_session`, `stop_session`, `get_session_status`.
- Phase 9.5 (TUI Frontend): `kay-tui` with ratatui multi-pane layout, subprocess consuming `kay-cli` JSONL stream.
- Phase 6 (Session Store): SQLite-backed session persistence, resume/fork, transcript export.

### 1.2 Gap to Fill
Phase 9 has only 3 commands (start/stop/status). Phase 10 adds:
- Session control from GUI/TUI: list, pause, resume, fork, kill
- Project-level settings: env vars, model allowlist, sandbox policy
- OS keychain binding for OpenRouter API keys
- Model tier picker (Recommended/Experimental/All)
- Command approval dialog (interactive sandbox confirmation)
- Settings panel

### 1.3 Constraints
- Must extend Phase 9's `IpcAgentEvent` enum additively (non_exhaustive)
- kay-tui must also get session control commands (keyboard shortcuts for spawn/pause/resume/fork/quit)
- Settings panel must work across both GUI (Tauri) and TUI (ratatui)
- Keyring: macOS Keychain, Linux libsecret, Windows Credential Manager
- Phase 9 PR (#18) must be merged to main before Phase 10 starts (dependency chain B)

## 2. Design

### 2.1 Architecture

```
kay-tauri (Rust/Tauri):
  src/
    session_manager.rs   — SessionManager: sessions map, list/pause/resume/fork/kill
    project_settings.rs  — ProjectSettings: env, model allowlist, sandbox policy
    keyring.rs           — OsKeyring trait: macOS Keychain / Linux libsecret / Windows CredMgr
    model_picker.rs      — ModelAllowlist with tier metadata
    command_approval.rs  — ApprovalRequest + UI dialog
    settings_panel.rs    — SettingsPanel Tauri state
    commands.rs          — 8 new commands (see §2.3)
    ui/
      src/components/SessionList.tsx
      src/components/SettingsPanel.tsx
      src/components/ModelPicker.tsx
      src/components/CommandApprovalDialog.tsx

kay-tui (Rust/ratatui):
  src/
    session_manager.rs   — TuiSessionManager: spawn/pause/resume/fork/kill over IPC
    app.rs               — Session list pane + keyboard shortcuts

kay-core (shared):
  src/session_manager.rs — SessionManager trait + default impl (used by both GUI + TUI)
```

### 2.2 Session Manager

`SessionManager` trait (shared between Tauri and TUI):

```rust
pub trait SessionManager: Send + Sync {
    fn list_sessions(&self) -> Vec<SessionInfo>;
    fn pause_session(&self, id: &str) -> Result<()>;
    fn resume_session(&self, id: &str) -> Result<()>;
    fn fork_session(&self, id: &str) -> Result<String>; // returns new session id
    fn kill_session(&self, id: &str) -> Result<()>;
}
```

`SessionInfo` struct:
- `id: String`
- `status: SessionStatus` (Running, Paused, Completed, Failed)
- `created_at: DateTime<Utc>`
- `last_active: DateTime<Utc>`
- `persona: String`
- `prompt_preview: String` (first 80 chars)

`SessionStatus` enum: `Running | Paused | Completed | Failed | Killed`

### 2.3 New Tauri Commands (8 commands)

| Command | Signature | Description |
|---------|-----------|-------------|
| `list_sessions` | `() -> Vec<SessionInfo>` | List all sessions (from Session Store) |
| `pause_session` | `(session_id: String) -> Result<()>` | Pause a running session |
| `resume_session` | `(session_id: String) -> Result<()>` | Resume a paused session |
| `fork_session` | `(session_id: String, persona: Option<String>) -> Result<String>` | Fork session, returns new id |
| `kill_session` | `(session_id: String) -> Result<()>` | SIGTERM + cleanup |
| `get_session_events` | `(session_id: String, from_turn: Option<u32>) -> Vec<IpcAgentEvent>` | Retrieve transcript |
| `save_project_settings` | `(settings: ProjectSettings) -> Result<()>` | Persist to `~/.kay/projects/<path>/settings.json` |
| `load_project_settings` | `(project_path: String) -> Result<Option<ProjectSettings>>` | Load from project dir |
| `bind_api_key` | `(provider: String, key: String) -> Result<()>` | Store in OS keychain |
| `get_api_key_fingerprint` | `(provider: String) -> Result<Option<String>>` | Check if key bound |

### 2.4 Project Settings Schema

```rust
pub struct ProjectSettings {
    pub project_path: String,          // absolute path to project root
    pub openrouter_key_alias: Option<String>, // reference to keychain entry
    pub model_allowlist_tier: ModelTier,   // Recommended | Experimental | All
    pub verifier_policy: VerifierPolicy,    // enabled, max_retries, cost_ceiling
    pub sandbox_policy: SandboxPolicy,      // allowed_paths, denied_paths, net_whitelist
    pub command_approval: CommandApproval, // Off | OnFirstUse | Always
}
```

### 2.5 Model Allowlist Tiers

```rust
pub enum ModelTier {
    Recommended, // Exacto allowlist (verified safe)
    Experimental, // smoke-tested models
    All,          // behind "Compatibility unknown" warning
}
```

Known models per tier (from Phase 5):
- **Recommended**: `anthropic/claude-4-sonnet`, `openai/gpt-4o`, `google/gemini-2.5-flash`
- **Experimental**: `anthropic/claude-4-opus`, `openai/o3`, `meta/llama-4`
- **All**: any model not explicitly allowlisted (warning)

### 2.6 OS Keychain Integration

```rust
pub trait OsKeyring {
    async fn store(&self, service: &str, account: &str, secret: &str) -> Result<()>;
    async fn retrieve(&self, service: &str, account: &str) -> Result<Option<String>>;
    async fn delete(&self, service: &str, account: &str) -> Result<()>;
}
```

- **macOS**: `security` CLI (`security add-generic-password`, `security find-generic-password`)
- **Linux**: `libsecret` via `secret-tool` CLI
- **Windows**: `cmdkey` CLI (`cmdkey /generic:... /user:... /pass:...`)

Service name: `kay` for the Kay app. Account: `<provider>/<model>`.

### 2.7 Command Approval Dialog

When `command_approval = OnFirstUse | Always`:
1. Tool call intercepted before execution
2. `ApprovalRequest` event emitted: `(tool_name, command, sandbox_status)`
3. UI shows dialog: tool name + command preview + sandbox violation risk
4. User approves (Enter) or denies (Escape)
5. Decision stored in session for `OnFirstUse` (don't ask again for this tool in this session)

### 2.8 TUI Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `n` | New session (spawn) |
| `p` | Pause current |
| `r` | Resume current |
| `f` | Fork current |
| `x` | Kill current |
| `s` | Toggle settings panel |
| `?` | Show help overlay |
| `q` | Quit |

### 2.9 IpcAgentEvent Additions

Phase 9 defined 17 variants. Phase 10 adds 4 more (additive, non_exhaustive):

```rust
// Add to IpcAgentEvent in kay-tauri/src/ipc_event.rs
SessionSpawned { session_id: String, persona: String, created_at: DateTime<Utc> },
SessionPaused { session_id: String, paused_at: DateTime<Utc> },
SessionResumed { session_id: String, resumed_at: DateTime<Utc> },
SessionForked { parent_id: String, child_id: String },
ApprovalRequested { tool_name: String, command: String, sandbox_status: SandboxStatus },
ApprovalDecision { tool_name: String, approved: bool, decided_at: DateTime<Utc> },
```

## 3. Implementation Plan

### Wave 1: SessionManager trait + state (Tauri)
- Add `SessionManager` trait to `kay-tauri/src/session_manager.rs`
- Implement in-memory `SessionManagerImpl` backed by Phase 6 `SessionStore`
- Add `SessionInfo` + `SessionStatus` types
- 3 unit tests (list_sessions empty, list_sessions with sessions, pause_nonexistent)

### Wave 2: 8 new Tauri commands
- Extend `commands.rs` with 8 new commands
- Extend `bindings.ts` (specta auto-regenerates)
- Each command validates session_id exists before operating
- Unit tests: each command with mock state

### Wave 3: IpcAgentEvent additions
- Add 6 new event variants to `ipc_event.rs`
- Add `From<AgentEvent>` impls for all new variants
- Add round-trip tests for all 6 new variants

### Wave 4: ProjectSettings + keyring
- `project_settings.rs` with `ProjectSettings` struct
- `keyring.rs` with `OsKeyring` trait + 3 platform impls
- `model_picker.rs` with `ModelTier` enum
- Unit tests: settings serialization, keyring store/retrieve

### Wave 5: Command approval dialog (Tauri)
- `command_approval.rs` with `ApprovalRequest` and `ApprovalStore`
- React dialog component in `kay-tauri/ui/src/components/CommandApprovalDialog.tsx`
- Keyboard shortcuts (Enter = approve, Escape = deny)
- `ApprovalDecision` event emitted after user decision

### Wave 6: TUI session control
- Extend `kay-tui/src/session_manager.rs` with TuiSessionManager
- Wire keyboard shortcuts (n/p/r/f/x) in `app.rs`
- Settings panel overlay in TUI (using ratatui popup/paragraph)

### Wave 7: Settings panel (Tauri + TUI)
- Tauri: `SettingsPanel.tsx` component with tabs
- TUI: ratatui popup/paragraph settings overlay
- Persist to `~/.kay/projects/<path>/settings.json`

### Wave 8: Integration + CI
- `tests/session_manager_integration.rs` (spawn/pause/resume/fork/kill cycle)
- `scripts/check-bindings.sh` updated to check all 11 commands

## 4. Success Criteria

1. A user can list all sessions from GUI and TUI, sorted by last_active descending.
2. A user can spawn/pause/resume/fork/kill sessions entirely from GUI without touching CLI.
3. The user binds an OpenRouter key to OS keychain (never plaintext); key is retrievable after app restart.
4. The model picker shows tiered list; Recommended tier only shows Exacto allowlist models.
5. A user can save/load project settings; settings persist across Kay restarts.
6. Command approval dialog shows on first tool call when enabled; subsequent calls from same tool skip dialog.
7. TUI keyboard shortcuts (n/p/r/f/x/s/?) all functional.
8. All 11 commands have tauri-specta TypeScript bindings.
9. cargo test -p kay-tauri passes with 25+ unit tests.
10. cargo test -p kay-tui passes with 10+ unit tests.

## 5. Dependencies
- Phase 6 (Session Store): ✅ (list/fork/resume/kill backed by SessionStore)
- Phase 9 (Tauri Shell): ✅ (extends commands.rs, ipc_event.rs, React UI)
- Phase 9.5 (TUI): ✅ (adds session control to ratatui app)

## 6. Non-Functional

- **Memory**: SessionManager holds ~200 bytes per active session (UUID + status + timestamps). 1000 sessions ≈ 200KB — negligible.
- **Concurrency**: SessionManager uses `tokio::sync::RwLock` for session map (reads common, writes rare).
- **Keyring errors**: Failed keyring ops surface as user-facing error in settings panel. Never silently fail.
- **Settings file**: Use serde_json for `settings.json`. Validate on load; discard invalid with warning.