# Phase 10: Multi-Session Manager + Project Settings

## What changed

Phase 10 implements **multi-session management** for Kay, enabling users to run, pause, resume, fork, and kill concurrent agent sessions — with OS keychain integration for API key storage.

### kay-tauri (Desktop Shell)

| File | Change |
|------|--------|
| `src/session_manager.rs` | `SessionManager` trait + `SessionManagerImpl` (HashMap + Mutex) — list/pause/resume/fork/kill sessions |
| `src/commands.rs` | 8 new Tauri IPC commands: `list_sessions`, `pause_session`, `resume_session`, `fork_session`, `kill_session`, `get_session_events`, `save_project_settings`, `load_project_settings` |
| `src/keyring.rs` | OS keychain integration (macOS Keychain, Linux libsecret, Windows Credential Manager) |
| `src/command_approval.rs` | `ApprovalStore` — command approval logic (Off/FirstUse/Always) |
| `src/project_settings.rs` | `ProjectSettings` struct with all configuration fields |
| `ui/src/components/SettingsPanel.tsx` | React settings UI with 4 tabs (Session/Model/Verifier/Sandbox) |
| `tests/session_manager_integration.rs` | 6 integration tests |

### kay-tui (Terminal UI)

| File | Change |
|------|--------|
| `src/session_manager.rs` | `TuiSessionManager` + `KeyboardMapper` — keyboard-driven session control |
| `src/ui.rs` | Settings overlay (press `s` to toggle) |

### Tests

- **kay-tauri**: 47 tests passing
- **kay-tui**: 18 tests passing
- **bindings**: ✅ in sync

## Verification

```bash
cargo test -p kay-tauri -p kay-tui 2>&1 | grep "passed.*failed"
```

## Next steps

- **Phase 11**: Cross-platform hardening + release pipeline (signed/notarized bundles + `cargo install kay`)
- **Phase 12**: EVAL-01a baseline run (>=80% on TB 2.0)
