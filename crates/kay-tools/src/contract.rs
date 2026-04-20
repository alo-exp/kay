//! Object-safe async Tool trait (TOOL-01 / D-01 / B1).

use forge_domain::{ToolName, ToolOutput};
use serde_json::Value;

use crate::error::ToolError;
use crate::runtime::context::ToolCallContext;

#[async_trait::async_trait]
pub trait Tool: Send + Sync + 'static {
    fn name(&self) -> &ToolName;
    fn description(&self) -> &str;

    /// Return an OWNED JSON Schema value (A1 resolution: `serde_json::Value`
    /// — not a borrowed `&Schema` nor a third-party schema type). Callers
    /// may need to mutate (harden) the schema without borrowing from the
    /// tool impl.
    fn input_schema(&self) -> serde_json::Value;

    /// Invoke the tool. `call_id` threads the provider-emitted tool call
    /// id (from `AgentEvent::ToolCallStart`) through streaming events
    /// (`AgentEvent::ToolOutput { call_id, .. }`).
    async fn invoke(
        &self,
        args: Value,
        ctx: &ToolCallContext,
        call_id: &str,
    ) -> Result<ToolOutput, ToolError>;
}
