//! ToolRegistry — immutable `HashMap<ToolName, Arc<dyn Tool>>` (D-01, D-11).

use std::collections::HashMap;
use std::sync::Arc;

use forge_domain::{ToolDefinition, ToolName};
use schemars::Schema;

use crate::contract::Tool;

pub struct ToolRegistry {
    tools: HashMap<ToolName, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    /// Insert a tool. Later inserts with the same name overwrite; this
    /// is an internal API (D-11: no runtime registration surface for v1).
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.name().clone();
        self.tools.insert(name, tool);
    }

    pub fn get(&self, name: &ToolName) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    /// Emit `ToolDefinition { name, description, input_schema }` for each
    /// registered tool. Per A1, `Tool::input_schema()` returns an OWNED
    /// `serde_json::Value`; it is wrapped into `schemars::Schema` for the
    /// `ToolDefinition` struct (which stores `schemars::Schema` as the
    /// field type). Invalid (non-object) schema values are skipped rather
    /// than panicking — a tool with a malformed schema is a bug but must
    /// not take down the registry-building path. Iteration order is not
    /// stable (HashMap) — providers do not rely on order.
    pub fn tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .filter_map(|tool| {
                let value = tool.input_schema();
                // schemars::Schema::try_from requires an object Value.
                let schema = Schema::try_from(value).ok()?;
                Some(ToolDefinition {
                    name: tool.name().clone(),
                    description: tool.description().to_string(),
                    input_schema: schema,
                })
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use forge_domain::{ToolName, ToolOutput};
    use serde_json::{Value, json};

    use super::*;
    use crate::contract::Tool;
    use crate::error::ToolError;
    use crate::runtime::context::ToolCallContext;

    /// Dummy tool used purely for registry-shape tests. `invoke` is never
    /// called in registry unit tests — its presence only proves that
    /// `Tool` is object-safe (an `Arc<dyn Tool>` can be constructed).
    struct DummyTool {
        name: ToolName,
        description: String,
        schema: Value,
    }

    #[async_trait]
    impl Tool for DummyTool {
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
            Ok(ToolOutput::text("ok"))
        }
    }

    fn dummy(name: &str) -> Arc<dyn Tool> {
        let schema = json!({
            "type": "object",
            "properties": {},
            "required": [],
            "additionalProperties": false,
        });
        Arc::new(DummyTool {
            name: ToolName::new(name),
            description: format!("dummy tool {name}"),
            schema,
        })
    }

    #[test]
    fn register_and_get_roundtrips() {
        let mut r = ToolRegistry::new();
        r.register(dummy("alpha"));
        r.register(dummy("beta"));
        assert_eq!(r.len(), 2);
        assert!(r.get(&ToolName::new("alpha")).is_some());
        assert!(r.get(&ToolName::new("beta")).is_some());
        assert!(r.get(&ToolName::new("missing")).is_none());
    }

    #[test]
    fn tool_definitions_emits_one_per_tool() {
        let mut r = ToolRegistry::new();
        r.register(dummy("alpha"));
        r.register(dummy("beta"));
        let defs = r.tool_definitions();
        assert_eq!(defs.len(), 2);
        let mut names: Vec<_> = defs.iter().map(|d| d.name.to_string()).collect();
        names.sort();
        assert_eq!(names, vec!["alpha".to_string(), "beta".to_string()]);
        for d in &defs {
            assert!(
                d.description.contains("dummy tool"),
                "description should include source text: {}",
                d.description
            );
        }
    }

    #[test]
    fn default_is_empty() {
        let r = ToolRegistry::default();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn register_with_same_name_overwrites() {
        // D-11 documentation: register() uses HashMap::insert semantics.
        let mut r = ToolRegistry::new();
        r.register(dummy("same"));
        r.register(dummy("same"));
        assert_eq!(r.len(), 1, "second register with same name overwrites");
    }

    #[test]
    fn schemas_returns_one_per_tool() {
        let mut r = ToolRegistry::new();
        r.register(dummy("alpha"));
        r.register(dummy("beta"));
        let schemas = r.schemas();
        assert_eq!(schemas.len(), 2);
        for s in &schemas {
            assert!(
                s.get("type").is_some() || s.get("properties").is_some(),
                "schema must be a JSON object: {s}"
            );
        }
    }
}
