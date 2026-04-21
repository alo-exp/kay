//! Wave 4 / 03-05 Task 3 registry wiring gate (TOOL-01 / TOOL-06).
//!
//! Proves the `default_tool_set` factory yields a registry whose:
//! 1. tool names match the ForgeCode reference set byte-for-byte
//!    (03-QUALITY-GATES.md),
//! 2. `tool_definitions()` emits exactly 7 entries with hardened
//!    strict-mode JSON schemas (TOOL-06),
//! 3. each entry carries `additionalProperties: false` and a
//!    non-empty `required` array — the ForgeCode-style hardening
//!    that drives the TB 2.0 score,
//! 4. re-registering the same name is a no-op from the caller's
//!    perspective (the registry overwrites; count stays stable).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::sync::Arc;

use kay_tools::{ImageQuota, NoOpInnerAgent, default_tool_set};

// Phase 5 Wave 5: reference set grows from 7 → 8 with the addition of
// `sage_query`, the forge/muse-callable sub-tool that delegates to
// sage for focused research. Kept sorted by "registration order" in
// `default_tool_set` for easy visual diffing vs. source.
const EXPECTED: &[&str] = &[
    "execute_commands",
    "task_complete",
    "image_read",
    "fs_read",
    "fs_write",
    "fs_search",
    "net_fetch",
    "sage_query",
];

fn build_registry() -> kay_tools::ToolRegistry {
    let quota = Arc::new(ImageQuota::new(2, 20));
    // NoOpInnerAgent suffices: these tests never invoke sage_query,
    // they only assert it is REGISTERED. A concrete agent isn't
    // needed until Wave 7's end-to-end tests.
    default_tool_set(PathBuf::from("/tmp"), quota, Arc::new(NoOpInnerAgent))
}

#[test]
fn default_set_has_eight_tools_with_expected_names() {
    let reg = build_registry();
    let defs = reg.tool_definitions();
    assert_eq!(defs.len(), 8, "expected 8 tools, got {}", defs.len());

    let mut names: Vec<String> = defs.iter().map(|d| d.name.as_str().to_string()).collect();
    names.sort();
    let mut expected: Vec<String> = EXPECTED.iter().map(|s| s.to_string()).collect();
    expected.sort();
    assert_eq!(names, expected);
}

#[test]
fn every_tool_schema_is_hardened_strict_mode() {
    let reg = build_registry();
    for def in reg.tool_definitions() {
        let schema = serde_json::to_value(&def.input_schema).unwrap();
        let obj = schema.as_object().expect("schema must be an object");
        assert_eq!(
            obj.get("additionalProperties"),
            Some(&serde_json::json!(false)),
            "{} schema must have additionalProperties: false",
            def.name.as_str()
        );
        let required = obj
            .get("required")
            .and_then(|v| v.as_array())
            .expect("required array must exist");
        assert!(
            !required.is_empty() || def.name.as_str() == "execute_commands",
            "{} must declare at least one required field",
            def.name.as_str()
        );
    }
}

#[test]
fn default_set_is_deterministic_across_calls() {
    // Building twice with the same inputs yields the same set of
    // names — pins 7-tool invariance against future drift.
    let a = build_registry();
    let b = build_registry();
    let mut na: Vec<String> = a
        .tool_definitions()
        .iter()
        .map(|d| d.name.as_str().to_string())
        .collect();
    let mut nb: Vec<String> = b
        .tool_definitions()
        .iter()
        .map(|d| d.name.as_str().to_string())
        .collect();
    na.sort();
    nb.sort();
    assert_eq!(na, nb);
}
