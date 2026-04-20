//! ToolRegistry — immutable `HashMap<ToolName, Arc<dyn Tool>>` (D-01, D-11).

use std::collections::HashMap;
use std::sync::Arc;

use forge_domain::{ToolDefinition, ToolName};

use crate::contract::Tool;

pub struct ToolRegistry {
    tools: HashMap<ToolName, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    pub fn register(&mut self, _tool: Arc<dyn Tool>) {
        todo!("Wave 1 (03-02): insert by tool.name().clone()")
    }

    pub fn get(&self, _name: &ToolName) -> Option<&Arc<dyn Tool>> {
        todo!("Wave 1 (03-02): HashMap::get passthrough")
    }

    pub fn tool_definitions(&self) -> Vec<ToolDefinition> {
        todo!("Wave 1 (03-02): iterate self.tools -> ToolDefinition {{ name, description, input_schema }}")
    }

    pub fn len(&self) -> usize { self.tools.len() }
    pub fn is_empty(&self) -> bool { self.tools.is_empty() }
}

impl Default for ToolRegistry {
    fn default() -> Self { Self::new() }
}
