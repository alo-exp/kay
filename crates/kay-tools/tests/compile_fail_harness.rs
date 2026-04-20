//! trybuild harness for Phase 3 compile_fail fixtures (T-01, T-02).
//!
//! # Current status (Wave 1): ignored
//!
//! trybuild spawns an isolated cargo build for each fixture, which
//! re-resolves `forge_domain`'s `ToolDescription` derive macro.  That
//! derive uses `std::fs::read_to_string("crates/forge_domain/src/tools/descriptions/*.md")`
//! at macro-expansion time — paths that only resolve from the workspace
//! root.  Under trybuild, CWD is `target/tests/trybuild/<pkg>/`, so the
//! macro panics → `forge_domain` fails to build → unrelated compile
//! errors drown out the actual T-01/T-02 signal.
//!
//! Fixing this requires upstream changes in `forge_tool_macros`
//! (use `CARGO_MANIFEST_DIR`-relative paths) which NN#1 forbids
//! touching `forge_*` crates.  **Deferred to a future harness plan**
//! that will either (a) mock-out `forge_domain` in the fixture via a
//! minimal re-export shim in kay-tools, or (b) propose the
//! `forge_tool_macros` fix upstream.
//!
//! # Equivalent lock retained
//!
//! Object-safety of `Tool` (T-01) IS locked at the integration-test
//! tier: see `tests/registry_integration.rs::arc_dyn_tool_is_object_safe`
//! — the `let t: Arc<dyn Tool> = make_tool(..)` line fails to compile if
//! anyone adds a generic method to the `Tool` trait.
//!
//! Input-schema owned-return (T-02, A1) is locked by the in-tree
//! `impl Tool for FakeTool` in `tests/registry_integration.rs` requiring
//! `fn input_schema(&self) -> Value` — anyone weakening the trait to
//! `-> &Value` breaks that impl's type-check.
//!
//! The `.fail.rs` fixtures in `tests/compile_fail/` remain as
//! reviewer-readable documentation of the locked contracts.

#[test]
#[ignore = "trybuild vs forge_tool_macros path resolution — see module docs; locks retained in registry_integration.rs"]
fn compile_fail_fixtures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/tool_not_object_safe.fail.rs");
    t.compile_fail("tests/compile_fail/input_schema_wrong_return.fail.rs");
}
