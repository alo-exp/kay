//! Schema hardening wrapper (D-02 / TOOL-05).

use serde_json::Value;

#[derive(Default)]
pub struct TruncationHints {
    pub output_truncation_note: Option<String>,
}

/// Run `forge_app::utils::enforce_strict_schema` (strict=true), then append
/// truncation-reminder text to the top-level description.
pub fn harden_tool_schema(_schema: &mut Value, _hints: &TruncationHints) {
    todo!("Wave 2 (03-03): call forge_app::utils::enforce_strict_schema(schema, true) then patch description")
}
