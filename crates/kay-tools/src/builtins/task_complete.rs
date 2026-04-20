//! task_complete — terminal-turn signal. Calls `ctx.verifier.verify(summary)`
//! (NoOpVerifier in Phase 3 → always Pending), emits
//! `AgentEvent::TaskComplete { call_id, verified=false, outcome }`, and
//! returns a short confirmation `ToolOutput`.
//!
//! T-3-06 invariant: Phase 3's NoOpVerifier must never produce a `Pass`
//! outcome. This tool therefore always emits `verified: false` — we
//! surface the verifier's outcome directly, but the boolean stays
//! explicit for downstream callers that key off it.

use async_trait::async_trait;
use forge_domain::{ToolName, ToolOutput};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::contract::Tool;
use crate::error::ToolError;
use crate::events::AgentEvent;
use crate::runtime::context::ToolCallContext;
use crate::schema::{TruncationHints, harden_tool_schema};
use crate::seams::verifier::VerificationOutcome;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct TaskCompleteArgs {
    /// Short summary of what the task accomplished. Passed to the
    /// verifier for evaluation.
    pub summary: String,
}

pub struct TaskCompleteTool {
    name: ToolName,
    description: String,
    input_schema: Value,
}

impl Default for TaskCompleteTool {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskCompleteTool {
    pub fn new() -> Self {
        let name = ToolName::new("task_complete");
        let description = "Signal task completion with a summary. The summary is evaluated \
            by the configured verifier."
            .to_string();
        let mut schema = serde_json::to_value(schemars::schema_for!(TaskCompleteArgs))
            .unwrap_or_else(|_| serde_json::json!({ "type": "object" }));
        harden_tool_schema(
            &mut schema,
            &TruncationHints {
                output_truncation_note: None,
            },
        );
        Self { name, description, input_schema: schema }
    }
}

/// Derive a short user-facing body from the verification outcome.
/// Kept as a free function so tests can assert it directly.
pub(crate) fn outcome_body(outcome: &VerificationOutcome) -> String {
    match outcome {
        VerificationOutcome::Pending { reason } => format!("verification pending: {reason}"),
        VerificationOutcome::Pass { note } => format!("verified: {note}"),
        VerificationOutcome::Fail { reason } => format!("verification failed: {reason}"),
    }
}

#[async_trait]
impl Tool for TaskCompleteTool {
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
        call_id: &str,
    ) -> Result<ToolOutput, ToolError> {
        let args = if args.is_null() { serde_json::json!({}) } else { args };
        let input: TaskCompleteArgs =
            serde_json::from_value(args).map_err(|e| ToolError::InvalidArgs {
                tool: self.name.clone(),
                reason: e.to_string(),
            })?;

        let outcome = ctx.verifier.verify(&input.summary).await;
        let verified = matches!(outcome, VerificationOutcome::Pass { .. });
        let body = outcome_body(&outcome);

        (ctx.stream_sink)(AgentEvent::TaskComplete {
            call_id: call_id.to_string(),
            verified,
            outcome,
        });

        Ok(ToolOutput::text(body))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn construct_produces_hardened_schema() {
        let t = TaskCompleteTool::new();
        let schema = t.input_schema();
        let obj = schema.as_object().expect("object");
        assert_eq!(obj.get("additionalProperties"), Some(&serde_json::json!(false)));
        assert!(obj.get("required").is_some());
    }

    #[test]
    fn name_is_task_complete() {
        let t = TaskCompleteTool::new();
        assert_eq!(t.name().as_str(), "task_complete");
    }

    #[test]
    fn outcome_body_formats_each_variant() {
        assert!(
            outcome_body(&VerificationOutcome::Pending {
                reason: "x".into(),
            })
            .contains("pending")
        );
        assert!(
            outcome_body(&VerificationOutcome::Pass { note: "ok".into() }).contains("verified")
        );
        assert!(
            outcome_body(&VerificationOutcome::Fail {
                reason: "bad".into(),
            })
            .contains("failed")
        );
    }
}
