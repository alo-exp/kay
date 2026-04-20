//! TOOL-05 / TOOL-06 property coverage — harden_tool_schema invariants (P-01).
//!
//! Validates the five structural invariants delegated to
//! `forge_app::utils::enforce_strict_schema` across 1024 randomly-shaped
//! object schemas per 03-TEST-STRATEGY §2.5.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_tools::{TruncationHints, harden_tool_schema};
use proptest::prelude::*;
use serde_json::{Map, Value, json};

/// Alphabet used for property-name generation — small to force
/// collisions that exercise the sort-stable required-array contract.
const PROP_ALPHABET: &[&str] = &["a", "b", "c", "d", "e"];
const LEAF_TYPES: &[&str] = &["string", "integer", "boolean"];

fn assert_strict_invariants(schema: &Value) {
    let obj = schema
        .as_object()
        .unwrap_or_else(|| panic!("schema root not an object: {schema}"));

    // (a) type must be "object"
    assert_eq!(
        obj.get("type"),
        Some(&json!("object")),
        "invariant (a) violated: type != \"object\": {schema}"
    );

    // (b) properties must be present and be an object
    let props = obj
        .get("properties")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("invariant (b) violated: properties missing: {schema}"));

    // (c) required must be a sorted array of every property key
    let required = obj
        .get("required")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("invariant (c) violated: required missing: {schema}"));

    let mut expected_keys: Vec<String> = props.keys().cloned().collect();
    expected_keys.sort();
    let required_keys: Vec<String> = required
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    assert_eq!(
        required_keys, expected_keys,
        "invariant (c) violated: required != sorted property keys: {schema}"
    );

    // (d) additionalProperties must be false
    assert_eq!(
        obj.get("additionalProperties"),
        Some(&json!(false)),
        "invariant (d) violated: additionalProperties != false: {schema}"
    );

    // (e) propertyNames must have been stripped
    assert!(
        !obj.contains_key("propertyNames"),
        "invariant (e) violated: propertyNames still present: {schema}"
    );
}

#[test]
fn verbatim_delegation_on_empty_object() {
    // Rule-1 deviation from plan 03-03 Task 3 Step 1 spec: the plan's guard
    // schema `json!({})` does NOT hit `is_object_schema` inside
    // `forge_app::utils::enforce_strict_schema` — that predicate requires at
    // least one of `type`, `properties`, or `additionalProperties` to be
    // present. A bare `{}` is therefore left untouched, which would spuriously
    // fail invariants (a)-(d). Using `{"type":"object"}` mirrors a realistic
    // empty-parameter tool schema and still exercises the strict-mode path
    // that adds `properties:{}`, `required:[]`, and `additionalProperties:false`.
    let mut schema = json!({ "type": "object" });
    harden_tool_schema(&mut schema, &TruncationHints::default());
    assert_strict_invariants(&schema);

    let obj = schema.as_object().unwrap();
    assert!(
        obj.get("properties")
            .unwrap()
            .as_object()
            .unwrap()
            .is_empty()
    );
    assert!(obj.get("required").unwrap().as_array().unwrap().is_empty());
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1024,
        .. ProptestConfig::default()
    })]

    /// P-01: Invariants (a)-(e) hold across 1024 randomly shaped object schemas.
    #[test]
    fn harden_always_produces_valid_strict_schema(
        prop_count in 0usize..=5,
        seed in any::<u64>(),
    ) {
        let mut properties = Map::new();
        for i in 0..prop_count {
            let name_idx = ((seed >> (i * 8)) & 0xff) as usize % PROP_ALPHABET.len();
            let type_idx =
                ((seed >> (i * 8 + 4)) & 0x0f) as usize % LEAF_TYPES.len();
            properties.insert(
                PROP_ALPHABET[name_idx].to_string(),
                json!({ "type": LEAF_TYPES[type_idx] }),
            );
        }

        let mut schema = json!({
            "properties": Value::Object(properties),
        });

        harden_tool_schema(&mut schema, &TruncationHints::default());
        assert_strict_invariants(&schema);
    }
}
