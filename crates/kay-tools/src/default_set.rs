//! `default_tool_set` factory (D-11 / TOOL-01 + Phase 5 LOOP-03).
//!
//! Builds the canonical 8-tool registry used by `kay-cli`. The factory
//! is the single audit point for Kay's tool surface — every registered
//! tool name MUST match the ForgeCode convention called out in
//! 03-QUALITY-GATES.md plus Kay's own Phase 5 addition:
//!
//!   execute_commands, task_complete, image_read,
//!   fs_read, fs_write, fs_search, net_fetch,
//!   sage_query  (Phase 5 Wave 5: forge/muse → sage delegation)
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
//! - Only `project_root` (→ `ExecuteCommandsTool`), `quota`
//!   (→ `ImageReadTool`), and `inner_agent` (→ `SageQueryTool`) are
//!   genuine per-tool construction inputs.
//!
//! So the factory takes exactly those three args; the rest are context
//! inputs the caller threads separately when building each
//! `ToolCallContext`. This keeps the factory a one-call constructor
//! and matches how `kay-cli` wires the registry vs. the per-turn
//! context.
//!
//! # Why `inner_agent` is a factory input, not a context input
//!
//! Unlike `Sandbox` / `TaskVerifier` (which EVERY tool sees via
//! `ctx.*`), the inner-agent handle is needed by exactly one tool:
//! `SageQueryTool`. Threading it through `ToolCallContext` would
//! force every other tool to acknowledge its existence in signatures
//! and keep a live `Arc` it never uses. Passing it to the factory
//! once, at registry-build time, confines the dependency to the one
//! tool that actually needs it.

use std::path::PathBuf;
use std::sync::Arc;

use crate::builtins::{
    ExecuteCommandsTool, FsReadTool, FsSearchTool, FsWriteTool, ImageReadTool, InnerAgent,
    NetFetchTool, SageQueryTool, TaskCompleteTool,
};
use crate::contract::Tool;
use crate::quota::ImageQuota;
use crate::registry::ToolRegistry;

/// Build the 8-tool registry.
///
/// - `project_root` — working directory root the shell tool resolves
///   relative `cwd` against.
/// - `quota` — shared `ImageQuota` handle backing `image_read`'s caps.
/// - `inner_agent` — handle the `sage_query` tool uses to spawn the
///   inner sage turn. Production callers wire an OpenRouter-backed
///   impl (Wave 7); early boot (`kay tools list`) and tests can pass
///   `Arc::new(NoOpInnerAgent)` — a loud-erroring placeholder that
///   fails any actual sage_query invocation with a pointed message
///   rather than silently succeeding.
///
/// Returns a fully populated `ToolRegistry` ready to hand to Phase 5's
/// agent loop.
pub fn default_tool_set(
    project_root: PathBuf,
    quota: Arc<ImageQuota>,
    inner_agent: Arc<dyn InnerAgent>,
) -> ToolRegistry {
    let mut reg = ToolRegistry::new();
    reg.register(Arc::new(ExecuteCommandsTool::new(project_root)) as Arc<dyn Tool>);
    reg.register(Arc::new(TaskCompleteTool::new()) as Arc<dyn Tool>);
    reg.register(Arc::new(ImageReadTool::new(quota)) as Arc<dyn Tool>);
    reg.register(Arc::new(FsReadTool::new()) as Arc<dyn Tool>);
    reg.register(Arc::new(FsWriteTool::new()) as Arc<dyn Tool>);
    reg.register(Arc::new(FsSearchTool::new()) as Arc<dyn Tool>);
    reg.register(Arc::new(NetFetchTool::new()) as Arc<dyn Tool>);
    reg.register(Arc::new(SageQueryTool::new(inner_agent)) as Arc<dyn Tool>);
    reg
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::builtins::NoOpInnerAgent;
    use forge_domain::ToolName;

    /// U-19: registered tool names must match the reference set
    /// byte-for-byte (03-QUALITY-GATES.md + Phase 5 LOOP-03 sage_query
    /// addition).
    #[test]
    fn default_set_names_match_reference() {
        let q = Arc::new(ImageQuota::new(2, 20));
        let reg = default_tool_set(
            PathBuf::from("/tmp"),
            q,
            Arc::new(NoOpInnerAgent),
        );

        let expected = [
            "execute_commands",
            "task_complete",
            "image_read",
            "fs_read",
            "fs_write",
            "fs_search",
            "net_fetch",
            "sage_query",
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
        assert_eq!(reg.tool_definitions().len(), 8);
    }
}
