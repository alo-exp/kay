//! fs_write — parity delegation to `ServicesHandle::fs_write`.

use async_trait::async_trait;
use forge_domain::{FSWrite, ToolName, ToolOutput};
use serde_json::Value;

use crate::contract::Tool;
use crate::error::ToolError;
use crate::runtime::context::ToolCallContext;
use crate::schema::{TruncationHints, harden_tool_schema};

pub struct FsWriteTool {
    name: ToolName,
    description: String,
    input_schema: Value,
}

impl Default for FsWriteTool {
    fn default() -> Self {
        Self::new()
    }
}

impl FsWriteTool {
    pub fn new() -> Self {
        let name = ToolName::new("fs_write");
        let description =
            "Create or overwrite a file on disk. Large writes may be rate-limited.".to_string();
        let mut schema = serde_json::to_value(schemars::schema_for!(FSWrite))
            .unwrap_or_else(|_| serde_json::json!({ "type": "object" }));
        harden_tool_schema(
            &mut schema,
            &TruncationHints {
                output_truncation_note: Some("Large writes may be rate-limited.".to_string()),
            },
        );
        Self { name, description, input_schema: schema }
    }
}

#[async_trait]
impl Tool for FsWriteTool {
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
        let input: FSWrite = serde_json::from_value(args).map_err(|e| ToolError::InvalidArgs {
            tool: self.name.clone(),
            reason: e.to_string(),
        })?;

        ctx.services
            .fs_write(input)
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
        let t = FsWriteTool::new();
        let schema = t.input_schema();
        let obj = schema.as_object().expect("object");
        assert_eq!(
            obj.get("additionalProperties"),
            Some(&serde_json::json!(false))
        );
        assert!(obj.get("required").is_some());
    }

    #[test]
    fn name_is_fs_write() {
        let t = FsWriteTool::new();
        assert_eq!(t.name().as_str(), "fs_write");
    }
}
