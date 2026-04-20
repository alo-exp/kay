//! Integration test: ToolRegistry + `Arc<dyn Tool>` object-safety (TOOL-01, TOOL-06).
//!
//! Validates that Phase 3's `Tool` trait is truly object-safe and the
//! registry can round-trip `Arc<dyn Tool>` values. If someone adds a
//! generic method to `Tool`, the `Arc::new(FakeTool { .. }) as Arc<dyn Tool>`
//! coercion below stops compiling — locking object-safety at the
//! integration-test tier in addition to the trybuild T-01 fixture.
//! See .planning/phases/03-tool-registry-kira-core-tools/03-RESEARCH.md §2.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use async_trait::async_trait;
use forge_domain::{ToolName, ToolOutput};
use kay_tools::{Tool, ToolCallContext, ToolError, ToolRegistry};
use serde_json::{Value, json};

struct FakeTool {
    name: ToolName,
    description: String,
    schema: Value,
}

#[async_trait]
impl Tool for FakeTool {
    fn name(&self) -> &ToolName {
        &self.name
    }
    fn description(&self) -> &str {
        &self.description
    }
    fn input_schema(&self) -> Value {
        self.schema.clone()
    }
    async fn invoke(
        &self,
        _args: Value,
        _ctx: &ToolCallContext,
        _call_id: &str,
    ) -> Result<ToolOutput, ToolError> {
        Ok(ToolOutput::text(format!("invoked {}", self.name)))
    }
}

fn make_tool(name: &str) -> Arc<dyn Tool> {
    let schema = json!({
        "type": "object",
        "properties": {"path": {"type": "string"}},
        "required": ["path"],
        "additionalProperties": false,
    });
    Arc::new(FakeTool {
        name: ToolName::new(name),
        description: format!("fake tool: {name}"),
        schema,
    })
}

#[test]
fn arc_dyn_tool_is_object_safe() {
    // If `Tool` were not object-safe, this coercion would fail to compile.
    // We exercise dynamic dispatch through trait methods that do not
    // require a ToolCallContext (name, description, input_schema) — this
    // proves object-safety WITHOUT needing to construct the full runtime
    // context (which requires Sandbox/Verifier/Services arcs).
    let t: Arc<dyn Tool> = make_tool("proof");
    assert_eq!(t.name().as_str(), "proof");
    assert!(t.description().contains("fake tool"));
    let schema = t.input_schema();
    assert!(
        schema.is_object(),
        "input_schema must return an object Value"
    );
    assert_eq!(schema["type"], "object");
}

#[test]
fn registry_roundtrips_three_tools() {
    let mut r = ToolRegistry::new();
    r.register(make_tool("fs_read"));
    r.register(make_tool("fs_write"));
    r.register(make_tool("execute_commands"));
    assert_eq!(r.len(), 3);

    assert!(r.get(&ToolName::new("fs_read")).is_some());
    assert!(r.get(&ToolName::new("fs_write")).is_some());
    assert!(r.get(&ToolName::new("execute_commands")).is_some());
    assert!(r.get(&ToolName::new("missing")).is_none());
}

#[test]
fn tool_definitions_emit_all_tools() {
    let mut r = ToolRegistry::new();
    r.register(make_tool("alpha"));
    r.register(make_tool("beta"));
    r.register(make_tool("gamma"));
    let defs = r.tool_definitions();
    assert_eq!(defs.len(), 3);
    let mut names: Vec<_> = defs.iter().map(|d| d.name.to_string()).collect();
    names.sort();
    assert_eq!(
        names,
        vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()]
    );
    for d in &defs {
        assert!(
            d.description.contains("fake tool"),
            "desc: {}",
            d.description
        );
    }
}

#[test]
fn same_name_register_overwrites() {
    // D-11: registry is built once; register() uses HashMap::insert.
    let mut r = ToolRegistry::new();
    r.register(make_tool("same"));
    r.register(make_tool("same"));
    assert_eq!(
        r.len(),
        1,
        "HashMap overwrite semantics: second register replaces"
    );
}
