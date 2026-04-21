//! kay-tools — Tool trait, registry, and KIRA core tools (Phase 3).
//!
//! See .planning/phases/03-tool-registry-kira-core-tools/03-CONTEXT.md
//! for decisions D-01..D-12. Built-in tools delegate to
//! `forge_app::ToolExecutor::execute` to preserve byte-identical parity
//! with ForgeCode (D-10 / EVAL-01 parity gate).

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod builtins;
pub mod contract;
mod default_set;
pub mod error;
pub mod events;
pub mod events_wire;
pub mod forge_bridge;
pub mod markers;
pub mod quota;
pub mod registry;
pub mod runtime;
pub mod schema;
pub mod seams;

pub use builtins::{
    ExecuteCommandsTool, FsReadTool, FsSearchTool, FsWriteTool, ImageReadTool, NetFetchTool,
    TaskCompleteTool,
};
pub use contract::Tool;
pub use default_set::default_tool_set;
pub use error::{CapScope, ToolError};
pub use events::{AgentEvent, ToolOutputChunk};
pub use forge_bridge::ForgeServicesFacade;
pub use quota::ImageQuota;
pub use registry::ToolRegistry;
pub use runtime::context::{ServicesHandle, ToolCallContext};
pub use schema::{TruncationHints, harden_tool_schema};
pub use seams::sandbox::{NoOpSandbox, Sandbox, SandboxDenial};
pub use seams::verifier::{NoOpVerifier, TaskVerifier, VerificationOutcome};
