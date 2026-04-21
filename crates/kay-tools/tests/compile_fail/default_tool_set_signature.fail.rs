//! Wave 6c fixture: locks `default_tool_set` return type.
//!
//! Contract: `default_tool_set` returns `ToolRegistry` BY VALUE
//! (not `&ToolRegistry`, not `Box<ToolRegistry>`, not some wrapper).
//! Callers in `kay-core::loop` and `kay-cli::run` (Phase 5 Wave 7+)
//! depend on owned-return semantics so they can mutate and move the
//! registry into their dispatch tables.
//!
//! Like the Phase 3 `input_schema_wrong_return.fail.rs` fixture, this
//! one documents the contract for reviewers. The live regression
//! check is the integration-tier canary
//! `tests/contract_object_safety_canaries.rs::default_tool_set_returns_owned_registry`
//! (trybuild harness is still `#[ignore]`-d for the Phase 3
//! `forge_tool_macros` CWD blocker — see `tests/compile_fail_harness.rs`).
//!
//! If/when the trybuild harness is unblocked, this fixture resumes
//! the negative-test job: a type annotation that asserts the return
//! is a BORROWED reference fails to compile; the captured `.stderr`
//! (mismatched-type error) is locked against a committed baseline.

use std::path::PathBuf;
use std::sync::Arc;

use kay_tools::{ImageQuota, NoOpInnerAgent, ToolRegistry, default_tool_set};

fn main() {
    let quota = Arc::new(ImageQuota::new(2, 20));
    // INTENTIONAL VIOLATION: annotate the return as a borrowed
    // `&ToolRegistry`. The real signature is `-> ToolRegistry` (by
    // value), so this assignment must fail to compile with an
    // `expected &ToolRegistry, found ToolRegistry` mismatch.
    //
    // If someone weakens the signature to return a reference, this
    // line would START compiling and the `.stderr` snapshot would
    // diverge — the regression is caught at trybuild time.
    let _reg: &ToolRegistry = &default_tool_set(
        PathBuf::from("/tmp"),
        quota,
        Arc::new(NoOpInnerAgent),
    );
    // Guard: make the type assertion above actually unreachable at
    // runtime in case the compiler ever ignores the borrow-vs-move
    // diagnostic here (it won't, but belt + suspenders).
    compile_error!(
        "Wave 6c default_tool_set signature fixture — this error is \
         intentional; see file docs"
    );
}
