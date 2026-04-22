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
