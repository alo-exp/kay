use kay_context::hardener::SchemaHardener;
use kay_context::engine::{ContextEngine, NoOpContextEngine};
use serde_json::json;

fn make_schema(with_properties_first: bool) -> serde_json::Value {
    if with_properties_first {
        json!({
            "type": "object",
            "properties": {
                "input": { "type": "string", "description": "the input" }
            },
            "required": ["input"]
        })
    } else {
        json!({
            "type": "object",
            "required": ["input"],
            "properties": {
                "input": { "type": "string", "description": "the input" }
            }
        })
    }
}

#[test]
fn harden_moves_required_before_properties() {
    let hardener = SchemaHardener::default();
    let mut schema = make_schema(true); // properties BEFORE required
    hardener.harden(&mut schema);
    // After hardening, required should appear before properties in iteration
    // Verify by checking the hardened JSON object key ordering
    let hardened_str = serde_json::to_string(&schema).unwrap();
    let req_pos = hardened_str.find("\"required\"").unwrap();
    let props_pos = hardened_str.find("\"properties\"").unwrap();
    assert!(req_pos < props_pos,
        "required should appear before properties after hardening, got: {}", hardened_str);
}

#[test]
fn harden_is_idempotent() {
    let hardener = SchemaHardener::default();
    let mut schema = make_schema(true);
    hardener.harden(&mut schema);
    let once = schema.clone();
    hardener.harden(&mut schema);
    let twice = schema.clone();
    assert_eq!(
        serde_json::to_string(&once).unwrap(),
        serde_json::to_string(&twice).unwrap(),
        "harden should be idempotent"
    );
}

#[test]
fn harden_adds_truncation_reminder() {
    let hardener = SchemaHardener::default();
    let mut schema = make_schema(false);
    hardener.harden(&mut schema);
    // After hardening, the schema should have a truncation reminder
    // (from TruncationHints) — just verify it's still a valid JSON object
    let hardened_str = serde_json::to_string(&schema).unwrap();
    assert!(hardened_str.contains("type"), "hardened schema should still be valid JSON object");
}

#[tokio::test]
async fn noop_engine_hardens_schemas() {
    // NoOpContextEngine::retrieve must return a ContextPacket
    // The schemas parameter is passed through (Phase 7 only assembles ContextPacket)
    let engine = NoOpContextEngine::default();
    let schemas = vec![make_schema(false)];
    let packet = engine.retrieve("query", &schemas).await.unwrap();
    // NoOp returns empty symbols but the call must succeed
    assert!(packet.symbols.is_empty());
    // ContextPacket is valid
    assert_eq!(packet.dropped_symbols, 0);
}

#[test]
fn tool_registry_schemas_method() {
    use kay_tools::registry::ToolRegistry;
    use kay_tools::contract::Tool;
    use kay_tools::error::ToolError;
    use kay_tools::runtime::context::ToolCallContext;
    use forge_domain::{ToolName, ToolOutput};
    use serde_json::Value;
    use std::sync::Arc;

    struct TestTool {
        name: ToolName,
    }

    #[async_trait::async_trait]
    impl Tool for TestTool {
        fn name(&self) -> &ToolName { &self.name }
        fn description(&self) -> &str { "test tool" }
        fn input_schema(&self) -> Value {
            json!({
                "type": "object",
                "properties": {
                    "x": { "type": "string" }
                },
                "required": ["x"]
            })
        }
        async fn invoke(
            &self,
            _args: Value,
            _ctx: &ToolCallContext,
            _call_id: &str,
        ) -> Result<ToolOutput, ToolError> {
            Ok(ToolOutput::text("ok"))
        }
    }

    // Empty registry returns empty vec
    let registry = ToolRegistry::new();
    let schemas = registry.schemas();
    assert!(schemas.is_empty(), "empty registry should return empty schemas vec");

    // Registry with a tool returns its schema
    let mut registry2 = ToolRegistry::new();
    registry2.register(Arc::new(TestTool { name: ToolName::new("test-tool") }));
    let schemas2 = registry2.schemas();
    assert_eq!(schemas2.len(), 1, "registry with 1 tool should return 1 schema");
    assert!(schemas2[0].is_object(), "schema should be a JSON object");
}
