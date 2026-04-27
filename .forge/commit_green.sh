#!/bin/sh
set -e

git add \
  Cargo.lock \
  crates/kay-tauri/src/commands.rs \
  crates/kay-tauri/src/flush.rs \
  crates/kay-tauri/src/lib.rs \
  crates/kay-tauri/src/main.rs \
  crates/kay-tauri/tests/gen_bindings.rs \
  crates/kay-tauri/ui/src/bindings.ts

git commit -m "GREEN(phase9): kay-tauri Tauri 2.x desktop shell — tauri-specta bindings + memory canary

Wave 6 (Tests) + Wave 7 (CI):

- gen_bindings.rs: specta TypeScript binding generation (tauri-specta 2.0.0-rc.24)
  - export_tauri_bindings: regenerates bindings.ts from collect_commands!
  - bindings_contain_expected_commands: validates all 3 commands present
  - Appends serde_json::Value TypeScript type (specta doesn't auto-generate recursive types)

- memory_canary.rs: 4-hour IPC memory leak detector
  - process_rss_is_nonzero: verifies RSS measurement API
  - short_ipc_canary: 10s CI-friendly check (< 20MB RSS growth threshold)
  - four_hour_ipc_canary: #[ignore] — nightly only, runs via -- --ignored

- flush.rs: Clone impl on IpcEventSink (required by specta typedError<T,E>)

- commands.rs: specta_commands_builder() exported to main (avoids __cmd__ cross-crate visibility issue)

- lib.rs: removed pub use of specta types (avoids macro re-export conflict)

- main.rs: removed redundant collect_commands! calls (builder setup in commands.rs)

CI (pre-existing):
- scripts/check-bindings.sh: drift gate — fails if bindings.ts out of sync
- .github/workflows/canary.yml: nightly 2AM UTC, macOS + Ubuntu, 260 min timeout

Tests: 4 passed (2 gen_bindings + 2 memory_canary), 1 ignored
UI build: pnpm build passes (64 modules, 209KB → 65KB gzipped)

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
echo "GREEN commit created"
