//! `default_tool_set` factory (D-11 / TOOL-01).
//!
//! Builds the canonical 7-tool registry used by `kay-cli`. The factory
//! is the single audit point for Phase 3's tool surface — every
//! registered tool name MUST match the ForgeCode convention called out
//! in 03-QUALITY-GATES.md:
//!
//!   execute_commands, task_complete, image_read,
//!   fs_read, fs_write, fs_search, net_fetch
//!
//! # Rule-3 reconciliation vs. 03-05-PLAN (#6)
//!
//! The plan text signature was
//! `default_tool_set(services, sandbox, verifier, project_root, quota)`
//! — five arguments routed into specific tools. That signature
//! oversupplies the current tools:
//! - `services` belongs on `ToolCallContext`, not on individual tools
//!   (they reach it via `ctx.services` at invoke time).
//! - `sandbox` / `verifier` likewise live on the context.
//! - Only `project_root` (→ `ExecuteCommandsTool`) and `quota`
//!   (→ `ImageReadTool`) are genuine per-tool construction inputs.
//!
//! So the factory takes exactly those two args; the rest are context
//! inputs the caller threads separately when building each
//! `ToolCallContext`. This keeps the factory a one-call constructor
//! and matches how `kay-cli` will wire the registry vs. the per-turn
//! context.

use std::path::PathBuf;
use std::sync::Arc;

use crate::builtins::{
    ExecuteCommandsTool, FsReadTool, FsSearchTool, FsWriteTool, ImageReadTool, NetFetchTool,
    TaskCompleteTool,
};
use crate::contract::Tool;
use crate::quota::ImageQuota;
use crate::registry::ToolRegistry;

/// Build the 7-tool registry.
///
/// - `project_root` — working directory root the shell tool resolves
///   relative `cwd` against.
/// - `quota` — shared `ImageQuota` handle backing `image_read`'s caps.
///
/// Returns a fully populated `ToolRegistry` ready to hand to Phase 5's
/// agent loop.
pub fn default_tool_set(project_root: PathBuf, quota: Arc<ImageQuota>) -> ToolRegistry {
    let mut reg = ToolRegistry::new();
    reg.register(Arc::new(ExecuteCommandsTool::new(project_root)) as Arc<dyn Tool>);
    reg.register(Arc::new(TaskCompleteTool::new()) as Arc<dyn Tool>);
    reg.register(Arc::new(ImageReadTool::new(quota)) as Arc<dyn Tool>);
    reg.register(Arc::new(FsReadTool::new()) as Arc<dyn Tool>);
    reg.register(Arc::new(FsWriteTool::new()) as Arc<dyn Tool>);
    reg.register(Arc::new(FsSearchTool::new()) as Arc<dyn Tool>);
    reg.register(Arc::new(NetFetchTool::new()) as Arc<dyn Tool>);
    reg
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use forge_domain::ToolName;

    /// U-19: registered tool names must match the ForgeCode reference
    /// set byte-for-byte (03-QUALITY-GATES.md).
    #[test]
    fn default_set_names_match_reference() {
        let q = Arc::new(ImageQuota::new(2, 20));
        let reg = default_tool_set(PathBuf::from("/tmp"), q);

        let expected = [
            "execute_commands",
            "task_complete",
            "image_read",
            "fs_read",
            "fs_write",
            "fs_search",
            "net_fetch",
        ];
        for name in expected {
            assert!(
                reg.get(&ToolName::new(name)).is_some(),
                "registry missing tool: {name}"
            );
        }
        // `tool_definitions()` must emit one entry per tool — proves
        // every built-in's schema round-trips into a
        // `schemars::Schema`.
        assert_eq!(reg.tool_definitions().len(), 7);
    }
}
