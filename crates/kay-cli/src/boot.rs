//! Startup wiring for kay-cli (Phase 3 Wave 4 / 03-05 Task 3).
//!
//! `install_tool_registry()` is the single function the Phase 5 agent
//! loop will call to obtain a pre-populated `ToolRegistry`. Today
//! (Phase 3) the only caller is the `tools list` subcommand, which
//! exercises the registry's `tool_definitions()` emission so we can
//! prove at runtime that all 7 tools wire up and produce valid
//! hardened schemas.

use std::path::PathBuf;
use std::sync::Arc;

use kay_tools::{ImageQuota, NoOpInnerAgent, ToolRegistry, default_tool_set};

/// Default per-turn cap for image reads (D-07). Phase 5 will make
/// this configurable via `ForgeConfig`; Phase 3 hardcodes the
/// reference values so a plain `kay tools list` works out-of-the-box.
const DEFAULT_IMAGE_PER_TURN: u32 = 2;

/// Default per-session cap for image reads (D-07).
const DEFAULT_IMAGE_PER_SESSION: u32 = 20;

/// Build the immutable 8-tool registry for this invocation.
///
/// - `project_root` defaults to `std::env::current_dir()` when the
///   caller does not override — the shell tool resolves relative
///   paths against it.
/// - `ImageQuota` defaults to `(2, 20)` per D-07.
/// - `sage_query`'s `InnerAgent` is a `NoOpInnerAgent` placeholder
///   until Wave 7 wires the real OpenRouter-backed agent. This makes
///   `kay tools list` correctly enumerate sage_query's hardened
///   schema even before end-to-end sage dispatch is live; any actual
///   invocation (via a future `kay run --prompt "…"` that has forge
///   issue a sage_query) will surface a clear error directing the
///   user to a wired build.
pub fn install_tool_registry(project_root: Option<PathBuf>) -> anyhow::Result<ToolRegistry> {
    let root = match project_root {
        Some(p) => p,
        None => std::env::current_dir()?,
    };
    let quota = Arc::new(ImageQuota::new(
        DEFAULT_IMAGE_PER_TURN,
        DEFAULT_IMAGE_PER_SESSION,
    ));
    Ok(default_tool_set(root, quota, Arc::new(NoOpInnerAgent)))
}
