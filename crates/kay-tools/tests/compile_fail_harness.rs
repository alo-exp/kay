//! trybuild harness for Phase 3 (T-01, T-02) + Phase 5 Wave 6c
//! (ServicesHandle, default_tool_set) compile-fail fixtures.
//!
//! # Current status: ignored (pre-existing `forge_tool_macros` blocker)
//!
//! trybuild spawns an isolated cargo build for each fixture, which
//! re-resolves `forge_domain`'s `ToolDescription` derive macro.  That
//! derive uses `std::fs::read_to_string("crates/forge_domain/src/tools/descriptions/*.md")`
//! at macro-expansion time — paths that only resolve from the workspace
//! root.  Under trybuild, CWD is `target/tests/trybuild/<pkg>/`, so the
//! macro panics → `forge_domain` fails to build → unrelated compile
//! errors drown out the actual fixture signal.
//!
//! Fixing this requires upstream changes in `forge_tool_macros`
//! (switch to `CARGO_MANIFEST_DIR`-relative paths).  That change is
//! scoped out for Phase 5: Phase 1 EVAL-01 captured the TB 2.0 baseline
//! against the current macro, so any change has to re-prove parity.
//! Deferred to a future harness-focused phase that will either (a)
//! mock-out `forge_domain` in the fixture via a minimal re-export shim
//! in kay-tools, or (b) propose the `forge_tool_macros` fix upstream
//! behind a parity re-run.
//!
//! # Equivalent lock retained (integration-test tier)
//!
//! Every fixture below has a live runtime-compilation canary in an
//! `#[test]`-annotated file so the contract IS enforced at PR time
//! even though trybuild itself is ignored:
//!
//!   * `tool_not_object_safe.fail.rs` (T-01)       ↔
//!     `registry_integration.rs::arc_dyn_tool_is_object_safe` and
//!     `contract_object_safety_canaries.rs::tool_trait_is_object_safe_via_arc_dyn_coercion`
//!
//!   * `input_schema_wrong_return.fail.rs` (T-02)  ↔ in-tree
//!     `impl Tool for FakeTool` in `tests/registry_integration.rs`
//!     requires `fn input_schema(&self) -> Value`; weakening the
//!     trait to `-> &Value` breaks that impl's type-check
//!
//!   * `services_handle_not_object_safe.fail.rs` (Wave 6c)  ↔
//!     `contract_object_safety_canaries.rs::services_handle_is_object_safe_via_arc_dyn_coercion`
//!
//!   * `default_tool_set_signature.fail.rs`        (Wave 6c)  ↔
//!     `contract_object_safety_canaries.rs::default_tool_set_returns_owned_registry`
//!
//! The `.fail.rs` fixtures remain as reviewer-readable documentation
//! of the locked contracts and are ready to take over the negative-
//! test job the moment the trybuild blocker lifts.
//!
//! # When to un-ignore
//!
//! The moment a future phase (a) moves `forge_tool_macros` to
//! `CARGO_MANIFEST_DIR`-relative paths AND re-proves TB 2.0 parity,
//! OR (b) adds a forge_domain shim that trybuild can build, remove
//! the `#[ignore]` attribute below, run `TRYBUILD=overwrite cargo
//! test -p kay-tools --test compile_fail_harness`, and commit the
//! newly-generated `.stderr` snapshots under
//! `tests/compile_fail/*.stderr` alongside this file.

#[test]
#[ignore = "trybuild vs forge_tool_macros path resolution — see module docs; locks retained in contract_object_safety_canaries.rs + registry_integration.rs"]
fn compile_fail_fixtures() {
    let t = trybuild::TestCases::new();
    // Phase 3
    t.compile_fail("tests/compile_fail/tool_not_object_safe.fail.rs");
    t.compile_fail("tests/compile_fail/input_schema_wrong_return.fail.rs");
    // Phase 5 Wave 6c
    t.compile_fail("tests/compile_fail/services_handle_not_object_safe.fail.rs");
    t.compile_fail("tests/compile_fail/default_tool_set_signature.fail.rs");
}
