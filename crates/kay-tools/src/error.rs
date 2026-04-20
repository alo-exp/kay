//! ToolError taxonomy (D-09). Separate from kay-provider-openrouter::ProviderError.

use std::time::Duration;

use forge_domain::ToolName;
use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("invalid args for tool {tool:?}: {reason}")]
    InvalidArgs { tool: ToolName, reason: String },

    #[error("tool {tool:?} timed out after {elapsed:?}")]
    Timeout { tool: ToolName, elapsed: Duration },

    #[error("tool {tool:?} execution failed: {source}")]
    ExecutionFailed {
        tool: ToolName,
        #[source]
        source: anyhow::Error,
    },

    #[error("image cap exceeded ({scope:?}, limit={limit})")]
    ImageCapExceeded { scope: CapScope, limit: u32 },

    #[error("sandbox denied {tool:?}: {reason}")]
    SandboxDenied { tool: ToolName, reason: String },

    #[error("tool not found: {tool:?}")]
    NotFound { tool: ToolName },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapScope {
    PerTurn,
    PerSession,
}
