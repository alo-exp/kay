// kay-tauri — Kay Tauri 2.x desktop shell (Phase 9).
//! kay-tauri — Kay Tauri 2.x desktop shell (Phase 9).
//!
//! Public API surface consumed by integration tests (`tests/gen_bindings.rs`,
//! `tests/memory_canary.rs`) and by `main.rs`.

pub mod agent_loop;
pub mod command_approval;
pub mod commands;
pub mod flush;
pub mod ipc_event;
pub mod keyring;
pub mod project_settings;
pub mod session_manager;
pub mod state;
