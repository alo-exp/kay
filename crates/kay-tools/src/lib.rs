//! kay-tools — Tool trait, registry, and KIRA core tools (Phase 3).
//!
//! See .planning/phases/03-tool-registry-kira-core-tools/03-CONTEXT.md
//! for decisions D-01..D-12. Built-in tools delegate to
//! `forge_app::ToolExecutor::execute` to preserve byte-identical parity
//! with ForgeCode (D-10 / EVAL-01 parity gate).

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod contract;
pub mod registry;
pub mod error;
pub mod events;
pub mod schema;
pub mod runtime;
pub mod seams;
pub mod quota;
pub mod markers;
pub mod builtins;
mod default_set;

pub use contract::Tool;
pub use registry::ToolRegistry;
pub use error::{ToolError, CapScope};
pub use events::{AgentEvent, ToolOutputChunk};
pub use schema::{TruncationHints, harden_tool_schema};
pub use runtime::context::ToolCallContext;
pub use seams::sandbox::{Sandbox, NoOpSandbox, SandboxDenial};
pub use seams::verifier::{TaskVerifier, NoOpVerifier, VerificationOutcome};
pub use default_set::default_tool_set;
pub use builtins::ExecuteCommandsTool;
