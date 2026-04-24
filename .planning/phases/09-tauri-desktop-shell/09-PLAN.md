# Phase 9 Plan — Tauri Desktop Shell

**Goal:** Ship `crates/kay-tauri/` as a production-quality Tauri 2.x desktop shell that streams `AgentEvent` to a React 19 frontend via `ipc::Channel<IpcAgentEvent>`.

**Branch:** `phase/09-tauri-desktop-shell`  
**Requirements:** TAURI-01..06, UI-01  
**Spec:** `docs/superpowers/specs/2026-04-23-phase9-tauri-desktop-shell-design.md` (Rev 4)

---

## Success Criteria

- `cargo check -p kay-tauri` compiles clean on macOS + Linux
- `cargo test -p kay-tauri --test gen_bindings` passes and generates `ui/src/bindings.ts`
- `scripts/check-bindings.sh` exits 0 (no drift)
- `pnpm build` in `crates/kay-tauri/ui/` exits 0 (TypeScript compiles + Vite bundles)
- All 18 `AgentEvent` variants mapped in `IpcAgentEvent::from()`
- `From<AgentEvent>` unit tests pass (Error→message, ImageRead→base64, Retry→Debug string)
- `flush_task` tests: 16ms flush, 64-event cap, final drain on sender-drop
- `VerificationCard` renders critic_role + verdict visually
- Memory canary test compiles (run separately with `--ignored`)

---

## Wave Structure (TDD: RED commit → GREEN commit per wave)

### Wave 1 — Cargo Deps + build.rs
RED: Add workspace deps + kay-tauri Cargo.toml skeleton referencing unresolved items  
GREEN: Full Cargo.toml + build.rs that calls `tauri_build::build()`  
Files: `Cargo.toml` (workspace), `crates/kay-tauri/Cargo.toml`, `crates/kay-tauri/build.rs`

### Wave 2 — IpcAgentEvent Mirror Type
RED: `ipc_event.rs` skeleton with type stubs, failing unit tests in `lib.rs`  
GREEN: Full `IpcAgentEvent` + all `From<>` impls + `bytes_to_data_url`  
Files: `crates/kay-tauri/src/ipc_event.rs`, `crates/kay-tauri/src/lib.rs`

### Wave 3 — Rust Backend (state, flush, commands, main)
RED: `state.rs`, `flush.rs`, `commands.rs`, `main.rs` stubs with `todo!()`  
GREEN: Full implementation with `AppState`, `flush_task`, `start_session`, `stop_session`, `get_session_status`  
Files: `crates/kay-tauri/src/state.rs`, `crates/kay-tauri/src/flush.rs`, `crates/kay-tauri/src/commands.rs`, `crates/kay-tauri/src/main.rs`, `crates/kay-tauri/src/lib.rs`

### Wave 4 — Frontend Scaffold
GREEN: `package.json`, `tsconfig.json`, `vite.config.ts`, `index.html`, `tauri.conf.json`  
Files: `crates/kay-tauri/ui/`

### Wave 5 — React Components
GREEN: All 12 components with dark theme, real-time streaming, `never` exhaustiveness check  
Files: `crates/kay-tauri/ui/src/`

### Wave 6 — Tests
GREEN: `tests/gen_bindings.rs`, `tests/memory_canary.rs`, `ipc_event` unit tests  
Files: `crates/kay-tauri/tests/`

### Wave 7 — CI + Ship
GREEN: `.github/workflows/canary.yml`, `scripts/check-bindings.sh`, commit + PR  

---

## Key Technical Decisions

- `IpcAgentEvent` in `crates/kay-tauri/src/ipc_event.rs` — never modify `AgentEvent`
- Binding generation in `tests/gen_bindings.rs` (NOT `build.rs`) — build scripts can't access main crate items
- `CancellationToken` for `stop_session` — not sender-drop
- 16ms flush task; 64-event size cap; final drain on channel close
- RC pins: `tauri-specta = "=2.0.0-rc.21"`, `specta = "=2.0.0-rc.20"`
- Offline provider for Phase 9 agent loop (Phase 10 adds OpenRouter key management)
- Dark theme via CSS custom properties; no external UI library

---

## Non-Negotiables (from spec §11)

1. No `externalBin` sidecar
2. `AgentEvent` additive-only — never modified
3. `IpcAgentEvent` owns all IPC concerns
4. Binding generation NOT in `main.rs`  
5. `stop_session` uses `CancellationToken`
6. DCO on every commit
7. Branch `phase/09-tauri-desktop-shell` only
8. 100% TDD — RED before GREEN per wave
