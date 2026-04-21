// T-9 API contract canaries — lock kay-session public API shape.
// These tests verify that certain operations are COMPILE ERRORS.
// Run with: cargo test -p kay-session -- compile_fail
//
// kay-session has no forge_tool_macros dep, so path-resolution blocker does NOT apply.
#[test]
fn compile_fail_fixtures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/session_not_cloneable.fail.rs");
    t.compile_fail("tests/compile_fail/append_after_close.fail.rs");
    t.compile_fail("tests/compile_fail/store_open_requires_path.fail.rs");
}
