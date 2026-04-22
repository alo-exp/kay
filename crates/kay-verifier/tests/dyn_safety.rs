//! T1-02e: TaskVerifier must be dyn-compatible (object-safe).
//! Uses trybuild to assert the trait compiles as a dyn object.

#[test]
fn task_verifier_is_dyn_compatible() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile_fail/dyn_safe.rs");
}
