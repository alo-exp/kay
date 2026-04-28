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
    /// Syntax: {{#if KEY}}...{{/if KEY}}
    ///
    /// For each variable in `self.variables`:
    /// - If value is truthy (non-empty): replace `{{#if KEY}}...{{/if KEY}}`
    ///   with `...` (the inner content only).
    /// - If value is falsy (empty/None): replace the entire block with `""`.
    ///
    /// Conditionals are processed iteratively — each match is replaced in
    /// isolation, allowing multiple independent blocks to coexist without
    /// index-shifting issues. Blocks are processed in insertion order.
    pub fn render_with_conditionals(&self) -> String {
        let mut result = self.content.clone();

        for (key, value) in &self.variables {
            // {{#if KEY}} in template → format string "{{{{#if {}}}}}" (each {{ produces one {)
            let start_tag = format!("{{{{#if {}}}}}", key);
            let end_tag = format!("{{{{/if {}}}}}", key);

            let start_pos = result.find(&start_tag);
            if let Some(start) = start_pos {
                let inner_start = start + start_tag.len();
                let remaining = &result[inner_start..];
                if let Some(end) = remaining.find(&end_tag) {
                    let actual_end = inner_start + end;
                    let inner_content = &result[inner_start..actual_end];

                    if value.is_empty() {
                        // Remove entire block including tags
                        result = result[..start].to_string()
                            + &result[actual_end + end_tag.len()..];
                    } else {
                        // Keep only inner content (strip tags)
                        result = result[..start].to_string()
                            + inner_content
                            + &result[actual_end + end_tag.len()..];
                    }
                }
            }
        }

        // Now substitute remaining {{VAR}} placeholders.
        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
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
        let _template = Template::new("{{greeting}}, {{name}}!");
        let mut vars = HashMap::new();
        vars.insert("greeting".to_string(), "Hello".to_string());
        vars.insert("name".to_string(), "Alice".to_string());
        
        assert_eq!(render_template("{{greeting}}, {{name}}!", &vars), "Hello, Alice!");
    }

    #[test]
    fn test_conditional() {
        let mut template = Template::new("Hello{{#if name}}, {{name}}{{/if name}}!");
        template.set("name", "");
        let result = template.render_with_conditionals();
        assert_eq!(result, "Hello!");
        
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