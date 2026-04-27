#!/bin/sh
set -e
cd /Users/shafqat/Documents/Projects/opencode/vs-others

# Stage all Phase 9.5 files
git add \
  Cargo.toml \
  Cargo.lock \
  crates/kay-tui/Cargo.toml \
  crates/kay-tui/src/lib.rs \
  crates/kay-tui/src/main.rs \
  crates/kay-tui/src/events.rs \
  crates/kay-tui/src/jsonl.rs \
  crates/kay-tui/src/state.rs \
  crates/kay-tui/src/subprocess.rs \
  crates/kay-tui/src/ui.rs \
  .planning/phases/09.5-tui-frontend/

# Commit
git commit -m "GREEN(phase9.5): kay-tui ratatui TUI frontend — full-screen terminal UI

TDD Waves 1-7 implemented and verified:

- events.rs: TuiEvent enum (18 variants, 9 serde round-trip tests)
  - Mirror of kay-tauri IpcAgentEvent wire format (spec §3)
  - Removed Unknown variant: serde(tag) doesn't support catch-all

- jsonl.rs: JsonlParser + LineBuffer (11 tests)
  - LineBuffer: 1 MB cap, drops oldest when full (PERF-01)
  - JsonlParser: feeds bytes → yields TuiEvent events
  - Malformed JSON: logged at WARN, skipped (ERR-01)
  - Unknown event types: logged at WARN, skipped (ERR-02)

- subprocess.rs: KaySubprocess spawn (1 test)
  - Spawns kay-cli with --output-format jsonl
  - Streams stdout via mpsc channel
  - kill_on_drop(true) ensures cleanup
  - KAY_CLI_PATH env var for custom binary path

- state.rs: AppState + SessionState + EventLog (4 tests)
  - EventLog: circular buffer cap 10_000 events (PERF-01)
  - CostAccumulator: prompt/completion tokens + USD cost
  - ActiveTool tracking from ToolCallStart/Complete events

- ui.rs: ratatui components + event loop (0 unit, integration)
  - App struct with ratatui List navigation
  - Header: elapsed time, cost meter, active tool
  - Event log: scrollable list with icon + summary text
  - Footer: status bar with keyboard hints
  - Ctrl+C → SIGINT, q → quit

- lib.rs: pub mod re-exports for forge_main library surface
- main.rs: replaced exit(69) placeholder with ui::run(app)

Workspace deps added:
- ratatui 0.26 + crossterm 0.27 (terminal I/O)
- tokio [process, io-util, sync, rt-multi-thread]
- serde + serde_json
- anyhow + tracing

Tests: 28 passed (9 events + 11 jsonl + 4 state + 1 subprocess + 3 other)
Security: ACCEPTABLE (SEC-01 path injection LOW, rest NONE)
Quality Gates: PASSED all 9 dimensions
Verification: PASSED all 10 criteria

Signed-off-by: Shafqat Ullah <shafqat@sourcevo.com>"
echo "Commit created"
