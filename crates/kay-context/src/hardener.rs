use serde_json::Value;

/// Delegates ForgeCode schema hardening to `kay_tools::schema::harden_tool_schema`
/// (DL-14 — no duplicate hardening logic in this crate).
///
/// `SchemaHardener::harden()` applies:
/// - `required` before `properties` key ordering (ForgeCode strict-mode)
/// - `additionalProperties: false` (strict mode)
/// - Optional truncation reminder (via `TruncationHints`)
///
/// Using `TruncationHints::default()` means `output_truncation_note` is
/// `None`, so no truncation text is appended — the operation is purely
/// structural and idempotent.
pub struct SchemaHardener;

impl SchemaHardener {
    pub fn new() -> Self {
        Self
    }

    /// Apply ForgeCode schema hardening to a single schema value.
    /// Delegates to `kay_tools::schema::harden_tool_schema` with default
    /// hints (no truncation note). Idempotent.
    pub fn harden(&self, schema: &mut Value) {
        kay_tools::schema::harden_tool_schema(
            schema,
            &kay_tools::schema::TruncationHints::default(),
        );
    }

    /// Apply hardening to all schemas in a slice.
    pub fn harden_all(&self, schemas: &mut [Value]) {
        for s in schemas.iter_mut() {
            self.harden(s);
        }
    }
}

impl Default for SchemaHardener {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn harden_all_applies_to_each_schema() {
        let mut s = SchemaHardener::new();
        let mut schemas = vec![
            serde_json::json!({"type": "object", "properties": {"x": {}}, "additionalProperties": false}),
            serde_json::json!({"type": "object", "properties": {"y": {}}}),
        ];
        s.harden_all(&mut schemas);
        // After hardening, each should have additionalProperties: false
        assert_eq!(schemas[0]["additionalProperties"], serde_json::json!(false));
        // schema 1 didn't have it — harden should add it
        assert_eq!(schemas[1]["additionalProperties"], serde_json::json!(false));
    }

    #[test]
    fn harden_is_idempotent() {
        let mut s = SchemaHardener::new();
        let mut schema = serde_json::json!({
            "type": "object",
            "required": ["x"],
            "properties": {"x": {}},
            "additionalProperties": false
        });
        s.harden(&mut schema);
        let after_first = schema.clone();
        s.harden(&mut schema);
        assert_eq!(schema, after_first, "harden must be idempotent");
    }

    #[test]
    fn schema_hardener_default_equals_new() {
        let a = SchemaHardener::new();
        let b = SchemaHardener::default();
        // Both are unit structs — just assert default() doesn't panic.
        let _ = (a, b);
    }
}
