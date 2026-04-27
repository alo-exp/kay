//! Dispatcher — Registry lookup + invoke (Forge-equivalent ToolExecutor)
//!
//! This is the PRIMARY dispatch mechanism for tool execution in Kay.
//! It matches Forge's ToolExecutor functionality:
//! - Read-before-edit enforcement
//! - Path normalization
//! - Output truncation
//! - Metrics tracking

use std::collections::HashMap;
use std::path::PathBuf;

use forge_domain::{ToolName, ToolOutput};
use serde_json::Value;

use crate::error::ToolError;
use crate::registry::ToolRegistry;
use crate::runtime::context::ToolCallContext;

// Maximum output size before truncation (Forge uses 50_000)
const MAX_OUTPUT_LENGTH: usize = 50_000;

/// Dispatcher state for tracking read files and enforcing read-before-edit
pub struct DispatcherState {
    /// Tracks files that have been read (for read-before-edit enforcement)
    read_files: HashMap<String, String>,
    /// Base directory for path normalization
    base_dir: PathBuf,
}

impl Default for DispatcherState {
    fn default() -> Self {
        Self {
            read_files: HashMap::new(),
            base_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }
}

impl DispatcherState {
    /// Create a new DispatcherState
    pub fn new() -> Self {
        Self::default()
    }

    /// Enforce read-before-edit for patch/overwrite operations
    /// Equivalent to Forge's require_prior_read()
    pub fn require_prior_read(&self, path: &str, operation: &str) -> Result<(), ToolError> {
        let normalized = Self::normalize_path(path, &self.base_dir);
        let path_str = normalized.to_string_lossy();

        if !self.read_files.contains_key(&*path_str) {
            return Err(ToolError::ReadBeforeEdit {
                tool: ToolName::new(operation),
                path: path.to_string(),
            });
        }
        Ok(())
    }

    /// Track that a file was read
    pub fn mark_read(&mut self, path: &str, content: &str) {
        let normalized = Self::normalize_path(path, &self.base_dir);
        self.read_files.insert(normalized.to_string_lossy().to_string(), content.to_string());
    }

    /// Normalize a path to absolute form
    fn normalize_path(path: &str, base_dir: &PathBuf) -> PathBuf {
        let p = std::path::Path::new(path);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            base_dir.join(p)
        }
    }

    /// Truncate output if too large
    pub fn truncate_output(output: &str) -> (String, bool) {
        if output.len() > MAX_OUTPUT_LENGTH {
            (
                format!(
                    "{}\n\n[Output truncated - {} bytes exceeds limit of {}]",
                    &output[..MAX_OUTPUT_LENGTH],
                    output.len(),
                    MAX_OUTPUT_LENGTH
                ),
                true,
            )
        } else {
            (output.to_string(), false)
        }
    }
}

/// Look up tool_name in registry and invoke it with args.
/// Returns ToolError::NotFound for unknown names.
pub async fn dispatch(
    registry: &ToolRegistry,
    tool_name: &ToolName,
    args: Value,
    ctx: &ToolCallContext,
    call_id: &str,
    _state: Option<&mut DispatcherState>,
) -> Result<ToolOutput, ToolError> {
    let tool = registry
        .get(tool_name)
        .ok_or_else(|| ToolError::NotFound { tool: tool_name.clone() })?;

    tool.invoke(args, ctx, call_id).await
}

/// Alternative dispatch that returns tool name and timing info
pub async fn dispatch_with_metrics(
    registry: &ToolRegistry,
    tool_name: &ToolName,
    args: Value,
    ctx: &ToolCallContext,
    call_id: &str,
) -> Result<(String, u64, bool), ToolError> {
    let tool = registry
        .get(tool_name)
        .ok_or_else(|| ToolError::NotFound { tool: tool_name.clone() })?;

    let start = std::time::Instant::now();
    let result = tool.invoke(args, ctx, call_id).await;
    let elapsed = start.elapsed().as_millis() as u64;

    match result {
        Ok(output) => {
            let content = output.as_str().unwrap_or_default();
            let truncated = content.len() > MAX_OUTPUT_LENGTH;
            Ok((tool_name.to_string(), elapsed, truncated))
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use forge_domain::{ToolName, ToolOutput};
    use serde_json::json;

    use super::*;
    use crate::contract::Tool;
    use crate::registry::ToolRegistry;

    fn make_ctx() -> ToolCallContext {
        ToolCallContext::for_test()
    }

    struct EchoTool {
        name: ToolName,
    }

    #[async_trait::async_trait]
    impl Tool for EchoTool {
        fn name(&self) -> &ToolName {
            &self.name
        }
        fn description(&self) -> &str {
            "echo"
        }
        fn input_schema(&self) -> serde_json::Value {
            json!({})
        }
        async fn invoke(
            &self,
            args: Value,
            _ctx: &ToolCallContext,
            _call_id: &str,
        ) -> Result<ToolOutput, ToolError> {
            Ok(ToolOutput::text(args.to_string()))
        }
    }

    #[tokio::test]
    async fn test_dispatch_routes_to_registered_tool() {
        let mut registry = ToolRegistry::new();
        let name = ToolName::new("echo");
        registry.register(Arc::new(EchoTool { name: name.clone() }));
        let result = dispatch(&registry, &name, json!({"x": 1}), &make_ctx(), "cid-1", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dispatch_unknown_tool_returns_not_found() {
        let registry = ToolRegistry::new();
        let name = ToolName::new("nonexistent");
        let result = dispatch(&registry, &name, json!({}), &make_ctx(), "cid-2", None).await;
        assert!(matches!(result, Err(ToolError::NotFound { .. })));
    }

    #[tokio::test]
    async fn test_dispatcher_state_require_prior_read() {
        let state = DispatcherState::new();
        let result = state.require_prior_read("test.rs", "patch");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_truncate_output_short() {
        let (content, truncated) = DispatcherState::truncate_output("hello");
        assert_eq!(content, "hello");
        assert!(!truncated);
    }

    #[tokio::test]
    async fn test_truncate_output_long() {
        let long = "x".repeat(60_000);
        let (content, truncated) = DispatcherState::truncate_output(&long);
        assert!(truncated);
        assert!(content.contains("Output truncated"));
        assert!(content.len() <= MAX_OUTPUT_LENGTH + 100);
    }
}
