//! T1-02e: TaskVerifier must be dyn-compatible (object-safe).
//! Inline compile-time check: if TaskVerifier ever gains a Sized bound or
//! a non-dyn-safe method, this file will fail to compile.

use kay_tools::seams::verifier::TaskVerifier;

// This function must compile — it's the dyn-safety proof.
// If TaskVerifier becomes non-object-safe, the compiler rejects `&dyn TaskVerifier`.
fn _assert_dyn_compatible(_v: &dyn TaskVerifier) {}

#[test]
fn task_verifier_is_dyn_compatible() {
    // Compilation of this test file IS the test.
    // If _assert_dyn_compatible above compiles, the trait is dyn-safe.
}
