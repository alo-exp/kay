//! Wave 6c fixture: locks object-safety of `kay_tools::ServicesHandle`.
//!
//! Like the Phase 3 `tool_not_object_safe.fail.rs` fixture, this one
//! exists primarily as REVIEWER-READABLE DOCUMENTATION of the locked
//! contract. The live regression check is the integration-tier canary
//! `tests/contract_object_safety_canaries.rs::services_handle_is_object_safe_via_arc_dyn_coercion`
//! (trybuild harness is still `#[ignore]`-d for the Phase 3
//! `forge_tool_macros` CWD blocker — see `tests/compile_fail_harness.rs`).
//!
//! If/when the trybuild harness is unblocked, this fixture resumes the
//! negative-test job: trybuild attempts to build it with an intentional
//! object-safety violation; the captured `.stderr` (E0038 at the
//! `Arc<dyn ServicesHandle>` coercion) is locked against a committed
//! baseline. Any change that makes the trait NOT object-safe (e.g.
//! adding `async fn foo<T>(&self, x: T)`) diverges from the snapshot
//! and fails CI.
//!
//! Do not "fix" the compile_error!() — it is load-bearing.

use std::sync::Arc;

use kay_tools::ServicesHandle;

// This use-site is the real canary: it requires `ServicesHandle` to
// be object-safe. If the trait gains a generic method, the compiler
// reports E0038 here BEFORE the explicit compile_error below.
fn takes_trait_object(_s: Arc<dyn ServicesHandle>) {}

fn main() {
    let _f: fn(Arc<dyn ServicesHandle>) = takes_trait_object;
    compile_error!(
        "Wave 6c ServicesHandle object-safety fixture — this error is \
         intentional; see file docs"
    );
}
