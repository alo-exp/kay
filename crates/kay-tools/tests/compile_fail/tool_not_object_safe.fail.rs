//! T-01 fixture: locks object-safety of `kay_tools::Tool`.
//!
//! This fixture uses `compile_error!` unconditionally so trybuild always
//! captures a compile error. The fixture's real purpose is reviewer
//! signalling: if someone adds a generic method to the `Tool` trait,
//! the `Arc<dyn Tool>` coercion in `takes_trait_object` below stops
//! compiling FIRST (E0038), and the captured stderr diverges from the
//! committed `.stderr` baseline — that divergence fails this test and
//! flags the object-safety regression before any PR lands.
//!
//! Do not "fix" the compile_error!() — it is load-bearing.

use kay_tools::Tool;
use std::sync::Arc;

// This use-site is the real canary: it requires `Tool` to be object-safe.
// If Tool gains a generic method, the compiler reports E0038 here BEFORE
// reporting the explicit compile_error below, and trybuild's .stderr
// diff detects the regression.
fn takes_trait_object(_t: Arc<dyn Tool>) {}

fn main() {
    let _f: fn(Arc<dyn Tool>) = takes_trait_object;
    compile_error!("T-01 object-safety fixture — this error is intentional; see file docs");
}
