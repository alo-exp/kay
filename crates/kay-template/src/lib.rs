//! Template Engine for Kay
//!
//! Provides simple template rendering with variable substitution.
//! Based on Forge's template crate design principles.

use std::collections::HashMap;

/// Template engine for rendering templates with variables
pub struct Template {
    /// Template content
    content: String,
    /// Variable substitutions
    variables: HashMap<String, String>,
}

impl Template {
    /// Create a new template from string content
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            variables: HashMap::new(),
        }
    }

    /// Set a variable for substitution
    pub fn set(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    /// Set multiple variables at once
    pub fn set_all(&mut self, vars: HashMap<String, String>) {
        self.variables.extend(vars);
    }

    /// Render the template with current variables
    pub fn render(&self) -> String {
        let mut result = self.content.clone();
        
        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        result
    }

    /// Render with conditional blocks
    /// Syntax: {{#if KEY}}...{{{/if KEY}}
    pub fn render_with_conditionals(&self) -> String {
        let mut result = self.content.clone();
        
        // Simple conditional: {{#if KEY}}content{{/if KEY}}
        // If KEY exists and is non-empty, include content
        // Otherwise remove the conditional block
        for (key, value) in &self.variables {
            let start_tag = format!("{{{{#if {}}}}}", key);
            let end_tag = format!("{{{{/if {}}}}}", key);
            
            if value.is_empty() {
                // Remove conditional block
                if let Some(start_pos) = result.find(&start_tag) {
                    if let Some(end_pos) = result.find(&end_tag) {
                        let end_after = end_pos + end_tag.len();
                        result = format!(
                            "{}{}",
                            &result[..start_pos],
                            &result[end_after..]
                        );
                    }
                }
            }
        }
        
        // Now render variables
        self.render()
    }
}

/// Simple template rendering function
pub fn render_template(content: &str, vars: &HashMap<String, String>) -> String {
    let mut template = Template::new(content);
    template.set_all(vars.clone());
    template.render()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_substitution() {
        let template = Template::new("Hello, {{name}}!");
        let mut t = template;
        t.set("name", "World");
        assert_eq!(t.render(), "Hello, World!");
    }

    #[test]
    fn test_multiple_variables() {
        let template = Template::new("{{greeting}}, {{name}}!");
        let mut vars = HashMap::new();
        vars.insert("greeting".to_string(), "Hello".to_string());
        vars.insert("name".to_string(), "Alice".to_string());
        
        assert_eq!(render_template("{{greeting}}, {{name}}!", &vars), "Hello, Alice!");
    }

    #[test]
    fn test_conditional() {
        let mut template = Template::new("Hello{{#if name}}, {{name}}{{/if name}}!");
        template.set("name", "");
        assert_eq!(template.render_with_conditionals(), "Hello!");
        
        let mut template2 = Template::new("Hello{{#if name}}, {{name}}{{/if name}}!");
        template2.set("name", "World");
        assert_eq!(template2.render_with_conditionals(), "Hello, World!");
    }

    #[test]
    fn test_missing_variable() {
        let template = Template::new("Hello, {{name}}!");
        let result = template.render();
        // Missing variable stays as placeholder
        assert_eq!(result, "Hello, {{name}}!");
    }
}