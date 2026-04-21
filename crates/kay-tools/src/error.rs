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

    /// `sage_query` was invoked at a nesting depth that would push the
    /// sub-turn beyond the configured ceiling (Phase 5 LOOP-03 — max 2
    /// levels of agent recursion). Belt + suspenders: sage's YAML
    /// `tool_filter` already excludes `sage_query`, but if that
    /// exclusion ever drifts, this runtime guard catches the recursion
    /// before the sub-turn spawns.
    ///
    /// `depth` is the PARENT context's nesting_depth at the time of
    /// the rejected invoke. `limit` is the inclusive maximum (parent
    /// depth must be `< limit`). The sage_query tool rejects when
    /// `depth >= limit` — i.e. the inner sub-turn would be at depth
    /// `limit + 1`. Default limit is 2 (configured inside the
    /// `sage_query` builtin).
    #[error("sage_query nesting depth {depth} exceeds limit {limit}")]
    NestingDepthExceeded { depth: u8, limit: u8 },

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
