//! net_fetch — parity delegation to `ServicesHandle::net_fetch`.
//!
//! Sandbox enforcement stays on the Kay side — `ForgeFetch` has no
//! knowledge of `kay_tools::Sandbox`, so the tool parses the URL and
//! calls `ctx.sandbox.check_net(&url)` BEFORE delegating.

use async_trait::async_trait;
use forge_domain::{NetFetch, ToolName, ToolOutput};
use serde_json::Value;
use url::Url;

use crate::contract::Tool;
use crate::error::ToolError;
use crate::runtime::context::ToolCallContext;
use crate::schema::{TruncationHints, harden_tool_schema};

pub struct NetFetchTool {
    name: ToolName,
    description: String,
    input_schema: Value,
}

impl Default for NetFetchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl NetFetchTool {
    pub fn new() -> Self {
        let name = ToolName::new("net_fetch");
        let description =
            "Fetch a URL. file:// is blocked; large responses are truncated.".to_string();
        let mut schema = serde_json::to_value(schemars::schema_for!(NetFetch))
            .unwrap_or_else(|_| serde_json::json!({ "type": "object" }));
        harden_tool_schema(
            &mut schema,
            &TruncationHints {
                output_truncation_note: Some(
                    "Large responses are truncated; file:// is blocked.".to_string(),
                ),
            },
        );
        Self { name, description, input_schema: schema }
    }
}

#[async_trait]
impl Tool for NetFetchTool {
    fn name(&self) -> &ToolName {
        &self.name
    }
    fn description(&self) -> &str {
        &self.description
    }
    fn input_schema(&self) -> Value {
        self.input_schema.clone()
    }

    async fn invoke(
        &self,
        args: Value,
        ctx: &ToolCallContext,
        _call_id: &str,
    ) -> Result<ToolOutput, ToolError> {
        let args = if args.is_null() {
            serde_json::json!({})
        } else {
            args
        };
        let input: NetFetch = serde_json::from_value(args).map_err(|e| ToolError::InvalidArgs {
            tool: self.name.clone(),
            reason: e.to_string(),
        })?;

        // Sandbox check on the parsed URL before handing off to the
        // facade. ForgeFetch itself enforces robots.txt and binary
        // detection; schemes-based blocking is Kay's responsibility.
        let url = Url::parse(&input.url).map_err(|e| ToolError::InvalidArgs {
            tool: self.name.clone(),
            reason: format!("invalid URL {}: {e}", input.url),
        })?;
        ctx.sandbox
            .check_net(&url)
            .await
            .map_err(|denial| ToolError::SandboxDenied {
                tool: self.name.clone(),
                reason: denial.reason,
            })?;

        ctx.services
            .net_fetch(input)
            .await
            .map_err(|e| ToolError::ExecutionFailed { tool: self.name.clone(), source: e })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn construct_produces_hardened_schema() {
        let t = NetFetchTool::new();
        let schema = t.input_schema();
        let obj = schema.as_object().expect("object");
        assert_eq!(
            obj.get("additionalProperties"),
            Some(&serde_json::json!(false))
        );
        assert!(obj.get("required").is_some());
    }

    #[test]
    fn name_is_net_fetch() {
        let t = NetFetchTool::new();
        assert_eq!(t.name().as_str(), "net_fetch");
    }
}
