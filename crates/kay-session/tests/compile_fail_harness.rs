// T-9 API contract canaries — lock kay-session public API shape.
// These tests verify that certain operations are COMPILE ERRORS.
// Run with: cargo test -p kay-session -- compile_fail
//
// Ignored: forge_domain proc-macro panics (missing tool .md files) during the trybuild
// workspace build, causing the trybuild runner to fail before reaching the fixture files.
// This is a pre-existing workspace issue, not a kay-session defect. The .fail.rs fixtures
// are kept as API contract documentation and will activate once forge_domain is fixed.
#[test]
#[ignore = "trybuild workspace build fails: forge_domain proc-macro missing tool .md files"]
fn compile_fail_fixtures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/session_not_cloneable.fail.rs");
    t.compile_fail("tests/compile_fail/append_after_close.fail.rs");
    t.compile_fail("tests/compile_fail/store_open_requires_path.fail.rs");
}
