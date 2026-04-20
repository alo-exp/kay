//! JSON-schema hardening for Kay tools (Phase 3, TOOL-05 / TOOL-06).
//!
//! ForgeCode's `enforce_strict_schema` is load-bearing for TB 2.0 scores
//! — we REUSE it unchanged (project CLAUDE.md Non-Negotiable #7, CONTEXT
//! D-02). This module is a thin wrapper that also appends a KIRA-style
//! truncation reminder to the top-level `description` when a hint is
//! provided.

use serde_json::Value;

/// Hints driving the non-schema wrap applied after strict-mode
/// normalization. Only `output_truncation_note` is populated today;
/// additional hints will be appended in future phases (e.g., rate-limit
/// notes in Phase 8).
#[derive(Debug, Default, Clone)]
pub struct TruncationHints {
    /// Reminder text appended to the tool's top-level `description` —
    /// typical value: `"Long outputs are truncated; narrow the command
    /// or use grep."`. When `None`, the description is untouched.
    pub output_truncation_note: Option<String>,
}

/// Harden a tool's JSON schema for strict-mode OpenAI / OpenRouter tool
/// calling (TOOL-05 / TOOL-06 / D-02).
///
/// Delegates structural normalization to
/// `forge_app::utils::enforce_strict_schema(schema, true)` unchanged,
/// then appends the optional `output_truncation_note` to the TOP-LEVEL
/// `description` string. Nested object descriptions are not touched.
///
/// # Idempotency
///
/// When `hints.output_truncation_note` is `None`, this function is
/// idempotent. With a non-None note, calling twice will append the note
/// twice — callers should NOT re-harden an already hardened schema.
pub fn harden_tool_schema(schema: &mut Value, hints: &TruncationHints) {
    forge_app::utils::enforce_strict_schema(schema, true);

    let Some(note) = hints.output_truncation_note.as_ref() else {
        return;
    };

    let Some(obj) = schema.as_object_mut() else {
        return;
    };

    let desc_key = "description";
    match obj.get_mut(desc_key) {
        Some(Value::String(existing)) => {
            existing.push_str("\n\n");
            existing.push_str(note);
        }
        _ => {
            obj.insert(desc_key.to_string(), Value::String(note.clone()));
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use serde_json::json;

    use super::*;

    fn sample_schema() -> Value {
        json!({
            "properties": {
                "name": { "type": "string" },
                "age":  { "type": "integer" }
            }
        })
    }

    #[test]
    fn required_sorted_after_harden() {
        // U-11
        let mut schema = json!({
            "properties": {
                "c": { "type": "string" },
                "a": { "type": "string" },
                "b": { "type": "string" }
            }
        });
        harden_tool_schema(&mut schema, &TruncationHints::default());
        assert_eq!(schema.get("required"), Some(&json!(["a", "b", "c"])));
    }

    #[test]
    fn additional_properties_false_after_harden() {
        // U-12
        let mut schema = sample_schema();
        harden_tool_schema(&mut schema, &TruncationHints::default());
        assert_eq!(schema.get("additionalProperties"), Some(&json!(false)));
    }

    #[test]
    fn all_of_flattened_after_harden() {
        // U-13
        let mut schema = json!({
            "allOf": [
                { "properties": { "a": { "type": "string" } } },
                { "properties": { "b": { "type": "integer" } } }
            ]
        });
        harden_tool_schema(&mut schema, &TruncationHints::default());
        assert!(
            schema.get("allOf").is_none(),
            "allOf must be flattened: {schema}"
        );
        let props = schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("properties after flatten");
        assert!(props.contains_key("a"), "a missing: {schema}");
        assert!(props.contains_key("b"), "b missing: {schema}");
    }

    #[test]
    fn truncation_reminder_present_after_harden() {
        // U-14
        let mut schema = json!({
            "description": "doc.",
            "properties": { "name": { "type": "string" } }
        });
        let hints = TruncationHints {
            output_truncation_note: Some("Long outputs are truncated.".to_string()),
        };
        harden_tool_schema(&mut schema, &hints);

        let desc = schema
            .get("description")
            .and_then(Value::as_str)
            .expect("description field");
        assert_eq!(desc, "doc.\n\nLong outputs are truncated.");
    }

    #[test]
    fn harden_delegates_to_enforce_strict_schema_verbatim() {
        let mut a = sample_schema();
        let mut b = sample_schema();
        forge_app::utils::enforce_strict_schema(&mut a, true);
        harden_tool_schema(&mut b, &TruncationHints::default());
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap(),
            "harden_tool_schema must delegate byte-for-byte when no hint is set"
        );
    }

    #[test]
    fn harden_creates_description_when_absent() {
        let mut schema = json!({
            "properties": { "name": { "type": "string" } }
        });
        let hints = TruncationHints { output_truncation_note: Some("Only note.".to_string()) };
        harden_tool_schema(&mut schema, &hints);
        assert_eq!(schema.get("description"), Some(&json!("Only note.")));
    }

    #[test]
    fn harden_noop_when_hint_is_none() {
        let mut schema = json!({
            "description": "Unchanged.",
            "properties": { "name": { "type": "string" } }
        });
        harden_tool_schema(&mut schema, &TruncationHints::default());
        assert_eq!(schema.get("description"), Some(&json!("Unchanged.")));
    }

    #[test]
    fn harden_does_not_touch_nested_descriptions() {
        let mut schema = json!({
            "description": "top",
            "properties": {
                "inner": {
                    "type": "object",
                    "description": "nested",
                    "properties": { "k": { "type": "string" } }
                }
            }
        });
        let hints = TruncationHints { output_truncation_note: Some("APPEND".to_string()) };
        harden_tool_schema(&mut schema, &hints);

        let top_desc = schema
            .get("description")
            .and_then(Value::as_str)
            .expect("top description");
        assert!(
            top_desc.contains("APPEND"),
            "top description missing append: {top_desc}"
        );

        let nested_desc = schema
            .pointer("/properties/inner/description")
            .and_then(Value::as_str)
            .expect("nested description");
        assert_eq!(
            nested_desc, "nested",
            "nested description must not be touched"
        );
    }

    #[test]
    fn harden_is_idempotent_when_hint_is_none() {
        let mut schema = sample_schema();
        harden_tool_schema(&mut schema, &TruncationHints::default());
        let first = serde_json::to_string(&schema).unwrap();
        harden_tool_schema(&mut schema, &TruncationHints::default());
        let second = serde_json::to_string(&schema).unwrap();
        assert_eq!(
            first, second,
            "harden_tool_schema with default hints must be idempotent"
        );
    }
}
