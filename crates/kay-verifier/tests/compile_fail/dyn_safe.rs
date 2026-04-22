// Trybuild compile-pass test (T1-02e): TaskVerifier must be dyn-compatible.
// This file MUST compile. If TaskVerifier ever gains a Sized bound or a
// non-dyn-safe method, this file will fail to compile — which is the
// intended detection mechanism.
use kay_tools::seams::verifier::TaskVerifier;

fn _uses_as_dyn_object(_v: &dyn TaskVerifier) {}

fn main() {}
