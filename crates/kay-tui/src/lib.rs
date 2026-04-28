// lib.rs — kay-tui library surface.
// See: docs/superpowers/specs/2026-04-24-phase9.5-tui-frontend-design.md
//
// All modules are re-exported at crate root so forge_main can depend on
// kay-tui as a library without pulling in the binary entry point.
//
// Wire format: events.rs TuiEvent must stay in sync with kay-tauri's
// IpcAgentEvent (see Phase 9.5 spec §3 "Wire Format Sync").

pub mod events;
pub mod jsonl;
pub mod session_manager;
pub mod state;
pub mod subprocess;
pub mod ui;
pub mod widgets;
