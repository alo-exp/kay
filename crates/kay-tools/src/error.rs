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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use forge_domain::ToolName;

    use super::*;

    #[test]
    fn invalidargs_display_includes_reason() {
        let e = ToolError::InvalidArgs {
            tool: ToolName::new("fs_read"),
            reason: "missing field `file_path`".into(),
        };
        let s = format!("{e}");
        assert!(s.contains("fs_read"), "display: {s}");
        assert!(s.contains("missing field"), "display: {s}");
    }

    #[test]
    fn timeout_display_includes_elapsed() {
        let e = ToolError::Timeout {
            tool: ToolName::new("execute_commands"),
            elapsed: Duration::from_secs(300),
        };
        let s = format!("{e}");
        assert!(s.contains("execute_commands"), "display: {s}");
        assert!(s.contains("300"), "display: {s}");
    }

    #[test]
    fn image_cap_exceeded_display_names_scope() {
        let e = ToolError::ImageCapExceeded { scope: CapScope::PerTurn, limit: 2 };
        let s = format!("{e}");
        assert!(s.contains("PerTurn"), "display: {s}");
        assert!(s.contains('2'), "display: {s}");
    }

    #[test]
    fn sandbox_denied_display_includes_reason() {
        let e = ToolError::SandboxDenied {
            tool: ToolName::new("net_fetch"),
            reason: "file:// scheme blocked".into(),
        };
        let s = format!("{e}");
        assert!(s.contains("net_fetch"), "display: {s}");
        assert!(s.contains("file://"), "display: {s}");
    }

    #[test]
    fn not_found_display_includes_tool_name() {
        let e = ToolError::NotFound { tool: ToolName::new("absent_tool") };
        let s = format!("{e}");
        assert!(s.contains("absent_tool"), "display: {s}");
    }
}
