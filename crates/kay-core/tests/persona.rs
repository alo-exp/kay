//! Phase 5 Wave 3 T3.1 RED — `kay_core::persona` YAML schema validation.
//!
//! LOOP-03. Five unit tests pinning the Persona YAML contract per
//! `05-BRAINSTORM.md` §Engineering-Lens: four required fields
//! (`name`, `system_prompt`, `tool_filter`, `model`) and `serde`-level
//! `deny_unknown_fields` strictness. Plus post-parse validation:
//!
//! - `Persona::validate_against_registry(&ToolRegistry)` — every
//!   `tool_filter` entry must resolve to a tool registered in the
//!   Phase 3 `ToolRegistry`. This is the choke-point that closes the
//!   "YAML persona injection" risk row from BRAINSTORM — a malicious
//!   or mistaken persona YAML cannot grant a non-existent or
//!   unregistered tool.
//! - `Persona::validate_model(&[&str])` — `model` must appear in
//!   Kay's launch allowlist. The allowlist itself is owned by
//!   `kay-provider-openrouter` (see `tests/fixtures/config/allowlist.json`
//!   — three entries: `anthropic/claude-sonnet-4.6`,
//!   `anthropic/claude-opus-4.6`, `openai/gpt-5.4`). Persona takes
//!   the allowlist as a `&[&str]` slice rather than a hard-coded const
//!   so the test harness can inject fixtures and so the persona module
//!   stays free of a reverse-layering dep on the provider crate.
//!
//! ## The five tests
//!
//! 1. **`persona_loads_valid_forge_yaml`** — full valid YAML with all
//!    four fields parses into a `Persona` with the expected values.
//!    Happy-path smoke.
//!
//! 2. **`persona_rejects_unknown_field`** — a YAML that adds a bogus
//!    extra field (`secret_backdoor`) fails at deserialization time
//!    because the struct carries `#[serde(deny_unknown_fields)]`.
//!    Without this gate, a persona file could silently carry payload
//!    that a future refactor accidentally starts reading from — the
//!    BRAINSTORM explicitly calls deny_unknown_fields out as mitigation.
//!
//! 3. **`persona_rejects_missing_required_field`** — omitting any of
//!    the four required fields (here `system_prompt`) yields a
//!    "missing field" error. `Option<T>` is explicitly NOT used for
//!    the four core fields — personas without a `system_prompt` would
//!    emit an empty system message and confuse the model.
//!
//! 4. **`persona_rejects_bad_tool_filter_entry`** — a YAML listing a
//!    tool name (`imaginary_tool_that_does_not_exist`) not present in
//!    the ToolRegistry fails `validate_against_registry`. Schema parse
//!    still succeeds (tool_filter is a `Vec<String>` at serde time);
//!    the registry check happens post-parse.
//!
//! 5. **`persona_model_field_validates_against_allowlist`** — a YAML
//!    whose `model` is not in the provided allowlist fails
//!    `validate_model`. Positive path: swapping to an allowlisted
//!    model passes. Covers both error and success branches.
//!
//! ## Expected RED state (T3.1)
//!
//! `kay_core::persona` does not yet exist. Compilation fails with E0432
//! "unresolved import `kay_core::persona`". T3.2 GREEN creates
//! `crates/kay-core/src/persona.rs` and adds the module declaration to
//! `lib.rs`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::sync::Arc;

use kay_core::persona::{Persona, PersonaError};
use kay_tools::{ImageQuota, ToolRegistry, default_tool_set};

// Launch allowlist fixture — mirrors
// `crates/kay-provider-openrouter/tests/fixtures/config/allowlist.json`
// `allowed_models` array verbatim (read at test setup time above
// `Persona::validate_model`). Kept inline here rather than pulling in
// the provider crate so the persona unit tests stay in a tight
// dependency envelope.
const LAUNCH_ALLOWLIST: &[&str] = &[
    "anthropic/claude-sonnet-4.6",
    "anthropic/claude-opus-4.6",
    "openai/gpt-5.4",
];

/// Build a realistic ToolRegistry with the full Phase 3 default set
/// (7 tools: execute_commands, task_complete, image_read, fs_read,
/// fs_write, fs_search, net_fetch). Using the real default_tool_set
/// rather than an empty registry makes the bad-tool-filter test prove
/// both that (a) known names would pass and (b) unknown names still
/// fail — the test assertion is asymmetric by name, not by emptiness.
fn registry_with_default_tools() -> ToolRegistry {
    let project_root = PathBuf::from("/tmp/persona-test");
    let quota = Arc::new(ImageQuota::new(2, 20));
    default_tool_set(project_root, quota)
}

// -----------------------------------------------------------------------------
// T3.1.a — happy-path parse
// -----------------------------------------------------------------------------

#[test]
fn persona_loads_valid_forge_yaml() {
    let yaml = r#"
name: forge
system_prompt: "You are Forge, a code-writing agent focused on implementation."
tool_filter:
  - fs_read
  - fs_write
  - execute_commands
  - task_complete
model: anthropic/claude-sonnet-4.6
"#;

    let persona = Persona::from_yaml_str(yaml).expect("valid YAML should parse");
    assert_eq!(persona.name, "forge");
    assert_eq!(persona.model, "anthropic/claude-sonnet-4.6");
    assert_eq!(persona.tool_filter.len(), 4);
    assert_eq!(persona.tool_filter[0], "fs_read");
    assert!(persona.system_prompt.contains("Forge"));
}

// -----------------------------------------------------------------------------
// T3.1.b — deny_unknown_fields strictness
// -----------------------------------------------------------------------------

#[test]
fn persona_rejects_unknown_field() {
    // `secret_backdoor` is not one of the four core fields — serde
    // should fail with deny_unknown_fields.
    let yaml = r#"
name: forge
system_prompt: "prompt"
tool_filter: []
model: anthropic/claude-sonnet-4.6
secret_backdoor: "evil"
"#;

    let err = Persona::from_yaml_str(yaml).expect_err("unknown field should error");
    let msg = format!("{err}").to_lowercase();
    assert!(
        msg.contains("secret_backdoor") || msg.contains("unknown field"),
        "expected error to name the unknown field or 'unknown field'; got: {err}"
    );
}

// -----------------------------------------------------------------------------
// T3.1.c — missing required field
// -----------------------------------------------------------------------------

#[test]
fn persona_rejects_missing_required_field() {
    // `system_prompt` is required — dropping it must error.
    let yaml = r#"
name: forge
tool_filter: []
model: anthropic/claude-sonnet-4.6
"#;

    let err = Persona::from_yaml_str(yaml).expect_err("missing required field should error");
    let msg = format!("{err}").to_lowercase();
    assert!(
        msg.contains("system_prompt") || msg.contains("missing"),
        "expected error to name the missing field; got: {err}"
    );
}

// -----------------------------------------------------------------------------
// T3.1.d — tool_filter entries validated against ToolRegistry
// -----------------------------------------------------------------------------

#[test]
fn persona_rejects_bad_tool_filter_entry() {
    // `fs_read` is registered; `imaginary_tool_that_does_not_exist` is
    // not. Schema parses cleanly (tool_filter is just Vec<String> at
    // serde time), then validate_against_registry rejects the bogus
    // entry.
    let yaml = r#"
name: forge
system_prompt: "prompt"
tool_filter:
  - fs_read
  - imaginary_tool_that_does_not_exist
model: anthropic/claude-sonnet-4.6
"#;

    let persona = Persona::from_yaml_str(yaml).expect("YAML parses — registry check is post-parse");
    let registry = registry_with_default_tools();

    let err = persona
        .validate_against_registry(&registry)
        .expect_err("unknown tool name in tool_filter must fail validation");
    let msg = format!("{err}");
    assert!(
        msg.contains("imaginary_tool_that_does_not_exist"),
        "expected error to name the unregistered tool; got: {err}"
    );

    // The specific error variant must be UnknownTool (not a generic
    // Yaml/Io/etc.) — future callers will pattern-match on this.
    assert!(
        matches!(err, PersonaError::UnknownTool(_)),
        "expected PersonaError::UnknownTool; got {err:?}"
    );
}

// -----------------------------------------------------------------------------
// T3.1.e — model allowlist (both failure and success branches)
// -----------------------------------------------------------------------------

#[test]
fn persona_model_field_validates_against_allowlist() {
    // Failure branch: model not in the launch allowlist.
    let yaml_bad = r#"
name: forge
system_prompt: "prompt"
tool_filter: []
model: openai/gpt-not-on-allowlist
"#;
    let persona_bad = Persona::from_yaml_str(yaml_bad).expect("YAML parses");
    let err = persona_bad
        .validate_model(LAUNCH_ALLOWLIST)
        .expect_err("non-allowlisted model must error");
    let msg = format!("{err}");
    assert!(
        msg.contains("openai/gpt-not-on-allowlist"),
        "expected error to name the rejected model; got: {err}"
    );
    assert!(
        matches!(err, PersonaError::ModelNotAllowed(_)),
        "expected PersonaError::ModelNotAllowed; got {err:?}"
    );

    // Success branch: every allowlisted model passes, proving the
    // comparison is by exact string match and not accidentally
    // normalized/lowercased/trimmed.
    for good in LAUNCH_ALLOWLIST {
        let yaml_ok = format!(
            r#"
name: forge
system_prompt: "prompt"
tool_filter: []
model: {good}
"#
        );
        let persona_ok = Persona::from_yaml_str(&yaml_ok).expect("YAML parses");
        persona_ok
            .validate_model(LAUNCH_ALLOWLIST)
            .unwrap_or_else(|e| panic!("allowlisted model {good} should pass; got {e}"));
    }
}
