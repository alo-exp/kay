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
            &TruncationHints { output_truncation_note: None },
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
        let args = if args.is_null() {
            serde_json::json!({})
        } else {
            args
        };
        let input: TaskCompleteArgs = serde_json::from_value(args).map_err(|e| {
            ToolError::InvalidArgs { tool: self.name.clone(), reason: e.to_string() }
        })?;

        let task_ctx_snapshot = ctx.snapshot_task_context();
        let outcome = ctx
            .verifier
            .verify(&input.summary, &task_ctx_snapshot)
            .await;
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
        assert_eq!(
            obj.get("additionalProperties"),
            Some(&serde_json::json!(false))
        );
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
            outcome_body(&VerificationOutcome::Pending { reason: "x".into() }).contains("pending")
        );
        assert!(
            outcome_body(&VerificationOutcome::Pass { note: "ok".into() }).contains("verified")
        );
        assert!(
            outcome_body(&VerificationOutcome::Fail { reason: "bad".into() }).contains("failed")
        );
    }

    #[tokio::test]
    async fn task_complete_passes_task_context_to_verifier() {
        use crate::quota::ImageQuota;
        use crate::runtime::context::{ServicesHandle, ToolCallContext};
        use crate::seams::sandbox::NoOpSandbox;
        use crate::seams::verifier::{TaskVerifier, VerificationOutcome};
        use async_trait::async_trait;
        use forge_domain::{FSRead, FSSearch, FSWrite, NetFetch, ToolOutput};
        use std::sync::{Arc, Mutex};
        use tokio_util::sync::CancellationToken;

        struct CapturingVerifier {
            captured_ctx: Arc<Mutex<String>>,
        }
        #[async_trait]
        impl TaskVerifier for CapturingVerifier {
            async fn verify(&self, _summary: &str, task_context: &str) -> VerificationOutcome {
                *self.captured_ctx.lock().unwrap() = task_context.to_string();
                VerificationOutcome::Pass { note: "captured".into() }
            }
        }

        struct NullSvc;
        #[async_trait]
        impl ServicesHandle for NullSvc {
            async fn fs_read(&self, _: FSRead) -> anyhow::Result<ToolOutput> {
                Ok(ToolOutput::text(""))
            }
            async fn fs_write(&self, _: FSWrite) -> anyhow::Result<ToolOutput> {
                Ok(ToolOutput::text(""))
            }
            async fn fs_search(&self, _: FSSearch) -> anyhow::Result<ToolOutput> {
                Ok(ToolOutput::text(""))
            }
            async fn net_fetch(&self, _: NetFetch) -> anyhow::Result<ToolOutput> {
                Ok(ToolOutput::text(""))
            }
        }

        let captured_ctx = Arc::new(Mutex::new(String::new()));
        let verifier = Arc::new(CapturingVerifier { captured_ctx: captured_ctx.clone() });
        let task_ctx = Arc::new(Mutex::new("tool: fs_read → ok\n".to_string()));

        let ctx = ToolCallContext::new(
            Arc::new(NullSvc),
            Arc::new(|_| {}),
            Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
            CancellationToken::new(),
            Arc::new(NoOpSandbox),
            verifier,
            0,
            task_ctx,
        );

        let tool = TaskCompleteTool::new();
        let args = serde_json::json!({ "summary": "done" });
        let _ = tool.invoke(args, &ctx, "call-1").await.expect("invoke");

        let got = captured_ctx.lock().unwrap().clone();
        assert!(
            got.contains("fs_read"),
            "verifier must receive task_context snapshot; got: {got:?}"
        );
    }
}
