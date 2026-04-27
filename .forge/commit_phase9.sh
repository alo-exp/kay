#!/bin/sh
set -e

# Stage all Phase 9 implementation files
git add \
  Cargo.lock \
  Cargo.toml \
  crates/kay-tauri/Cargo.toml \
  crates/kay-tauri/src/commands.rs \
  crates/kay-tauri/src/lib.rs \
  crates/kay-tauri/src/main.rs \
  crates/kay-tauri/tests/gen_bindings.rs \
  crates/kay-tauri/tests/memory_canary.rs \
  crates/kay-tauri/ui/src/bindings.ts \
  crates/kay-tauri/src/agent_loop.rs \
  crates/kay-tauri/tauri.conf.json \
  docs/sessions/2026-04-24-phase9-tauri-shell.md

git commit -m "feat(kay-tauri): Phase 9 — Tauri 2.x desktop shell with specta bindings

## Core Changes

specta Builder pattern:
- Refactor main.rs: remove tauri::generate_handler![] (requires
  tauri-macros/compression sidecar). Use tauri_specta::Builder
  with specta::collect_commands!() instead — no sidecar needed.
- commands.rs: keep only 3 specta-annotated commands
  (start_session, stop_session, get_session_status). Agent loop
  moved to agent_loop.rs module.
- agent_loop.rs: NEW — owned agent run() loop with session state
  and event channel, imported by commands.rs.

Module structure:
- src/lib.rs: pub mod agent_loop
- src/main.rs: App::new().setup(|app, _| { builder.build(app) })
- src/commands.rs: 3 #[tauri::command] + #[specta::specta] functions
- src/agent_loop.rs: Agent loop run() and State::default()

## Tests

- gen_bindings.rs: export_tauri_bindings — asserts bindings.ts
  exists and Typescript export matches specta types.
- memory_canary.rs: rss_measurement_works + short_ipc_canary (10s
  CI check; 4h threshold in canary.yml workflow).

## Dependencies

- Cargo.toml: specta/tauri-specta workspace features aligned
  (rc.24), tauri-specta re-exported via lib.
- kay-tauri/Cargo.toml: specta-typescript as lib dep (not dev-dep)
  since it's imported in lib.rs for specta::export::<Typescript>().
- kay-tauri/tauri.conf.json: devtools enabled for test harness,
  kay-tauri identifier.

## CI

- scripts/check-bindings.sh: drift gate for TypeScript bindings.
- .github/workflows/canary.yml: 4h memory canary (weekly, macOS latest).
- bindings.ts regenerated from specta after builder refactor.

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
echo "Commit created"
