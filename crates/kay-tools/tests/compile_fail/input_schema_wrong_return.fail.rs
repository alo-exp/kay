//! T-02 fixture: locks `fn input_schema(&self) -> serde_json::Value` (A1).
//!
//! If someone weakens the trait to accept a borrowed `&Value` return type
//! (or some other non-owned shape), this fixture — which attempts to
//! `impl Tool` returning `&Value` — would START compiling, and the
//! captured `.stderr` diverges from the committed baseline. The test
//! fails; the A1 regression is surfaced at CI.

use async_trait::async_trait;
use forge_domain::{ToolName, ToolOutput};
use kay_tools::{Tool, ToolCallContext, ToolError};
use serde_json::Value;

struct Bad {
    name: ToolName,
    desc: String,
    schema: Value,
}

#[async_trait]
impl Tool for Bad {
    fn name(&self) -> &ToolName {
        &self.name
    }
    fn description(&self) -> &str {
        &self.desc
    }
    // INTENTIONAL VIOLATION: returns &Value instead of owned Value.
    // Trait requires `fn input_schema(&self) -> Value` — this must fail.
    fn input_schema(&self) -> &Value {
        &self.schema
    }
    async fn invoke(
        &self,
        _a: Value,
        _c: &ToolCallContext,
        _id: &str,
    ) -> Result<ToolOutput, ToolError> {
        unreachable!()
    }
}

fn main() {}
