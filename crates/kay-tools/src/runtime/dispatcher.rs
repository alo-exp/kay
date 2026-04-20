//! Dispatcher — registry lookup + invoke (R-5 / Phase 5 entry point).

use std::sync::Arc;

use forge_domain::{ToolName, ToolOutput};
use serde_json::Value;

use crate::error::ToolError;
use crate::registry::ToolRegistry;
use crate::runtime::context::ToolCallContext;

/// Look up `tool_name` in `registry` and invoke it with `args`.
///
/// Returns `ToolError::NotFound` for unknown names. Phase 5 wraps this
/// function with the agent loop; it must NOT be called with sandbox checks
/// already done — each tool's `invoke()` receives `ctx.sandbox` and performs
/// its own pre-flight check.
pub async fn dispatch(
    registry: &ToolRegistry,
    tool_name: &ToolName,
    args: Value,
    ctx: &ToolCallContext,
    call_id: &str,
) -> Result<ToolOutput, ToolError> {
    let tool = registry
        .get(tool_name)
        .ok_or_else(|| ToolError::NotFound { tool: tool_name.clone() })?;
    tool.invoke(args, ctx, call_id).await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use forge_domain::{ToolName, ToolOutput};
    use serde_json::json;

    use super::*;
    use crate::contract::Tool;
    use crate::registry::ToolRegistry;

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

    fn make_ctx() -> ToolCallContext {
        ToolCallContext::for_test()
    }

    #[tokio::test]
    async fn test_dispatch_routes_to_registered_tool() {
        let mut registry = ToolRegistry::new();
        let name = ToolName::new("echo");
        registry.register(Arc::new(EchoTool { name: name.clone() }));
        let result = dispatch(&registry, &name, json!({"x": 1}), &make_ctx(), "cid-1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dispatch_unknown_tool_returns_not_found() {
        let registry = ToolRegistry::new();
        let name = ToolName::new("nonexistent");
        let result = dispatch(&registry, &name, json!({}), &make_ctx(), "cid-2").await;
        assert!(matches!(result, Err(ToolError::NotFound { .. })));
    }
}
