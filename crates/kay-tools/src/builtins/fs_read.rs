//! fs_read — parity delegation to `ServicesHandle::fs_read`.
//!
//! Delegation is one-line: deserialize the `forge_domain::FSRead` input
//! from the raw JSON args, hand it to `ctx.services.fs_read(...)`, and
//! return the resulting `ToolOutput` unchanged. All read logic —
//! sandbox checks, path normalization, size limits, binary detection —
//! lives in the underlying `ForgeFsRead` service via the facade.

use async_trait::async_trait;
use forge_domain::{FSRead, ToolName, ToolOutput};
use serde_json::Value;

use crate::contract::Tool;
use crate::error::ToolError;
use crate::runtime::context::ToolCallContext;
use crate::schema::{TruncationHints, harden_tool_schema};

pub struct FsReadTool {
    name: ToolName,
    description: String,
    input_schema: Value,
}

impl Default for FsReadTool {
    fn default() -> Self {
        Self::new()
    }
}

impl FsReadTool {
    pub fn new() -> Self {
        let name = ToolName::new("fs_read");
        let description = "Read a file from disk. For large files, use start_line/end_line \
            range reads rather than full reads."
            .to_string();
        let mut schema = serde_json::to_value(schemars::schema_for!(FSRead))
            .unwrap_or_else(|_| serde_json::json!({ "type": "object" }));
        harden_tool_schema(
            &mut schema,
            &TruncationHints {
                output_truncation_note: Some(
                    "For large files, use start_line/end_line range reads.".to_string(),
                ),
            },
        );
        Self { name, description, input_schema: schema }
    }
}

#[async_trait]
impl Tool for FsReadTool {
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
        let args = if args.is_null() { serde_json::json!({}) } else { args };
        let input: FSRead =
            serde_json::from_value(args).map_err(|e| ToolError::InvalidArgs {
                tool: self.name.clone(),
                reason: e.to_string(),
            })?;

        ctx.services
            .fs_read(input)
            .await
            .map_err(|e| ToolError::ExecutionFailed {
                tool: self.name.clone(),
                source: e,
            })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn construct_produces_hardened_schema() {
        let t = FsReadTool::new();
        let schema = t.input_schema();
        let obj = schema.as_object().expect("object");
        assert_eq!(obj.get("additionalProperties"), Some(&serde_json::json!(false)));
        assert!(
            obj.get("required").is_some(),
            "required array must be present after hardening"
        );
    }

    #[test]
    fn name_is_fs_read() {
        let t = FsReadTool::new();
        assert_eq!(t.name().as_str(), "fs_read");
    }
}
