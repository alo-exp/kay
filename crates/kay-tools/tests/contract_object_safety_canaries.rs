//! Wave 6c integration-tier object-safety canaries for the three
//! contracts the plan calls out: `Tool`, `ServicesHandle`,
//! `default_tool_set`.
//!
//! # Why integration-tier and not trybuild?
//!
//! Phase 3 carried a pre-existing `forge_tool_macros` CWD blocker:
//! the `ToolDescription` derive reads description markdown at
//! macro-expansion time via a path relative to the current working
//! directory. Under `trybuild`, CWD is
//! `target/tests/trybuild/<pkg>/`, the paths don't resolve, the
//! macro panics, and `forge_domain` fails to build under every
//! fixture — drowning out the actual compile-error signal we want
//! to lock.
//!
//! Fixing that requires `CARGO_MANIFEST_DIR`-relative paths in
//! `forge_tool_macros`, which is scoped out for Phase 5 (parity gate
//! caution — Phase 1 EVAL-01 baseline was captured against the
//! current macro; any change to it has to re-prove parity). See
//! `tests/compile_fail_harness.rs` module docs.
//!
//! Until that blocker lifts, the integration-tier canaries below
//! give us the equivalent REGRESSION protection for Wave 6c's three
//! object-safety contracts:
//!
//!   * `Tool` (already locked by
//!     `registry_integration.rs::arc_dyn_tool_is_object_safe` — this
//!     file adds a second canary anchored in a different import
//!     surface so a refactor can't silently disarm both at once);
//!   * `ServicesHandle` (NEW — the only prior anchor was the
//!     `Arc<dyn ServicesHandle>` parameter on `make_ctx_with_services`
//!     in `support/mod.rs`, which is buried in shared-helper land;
//!     this file gives it a name at the top level);
//!   * `default_tool_set` return type (NEW — locks the function
//!     signature shape `fn(_, _, _) -> ToolRegistry` so a refactor
//!     that changes the return to `&ToolRegistry` or a newtype
//!     wrapper breaks this file before it breaks downstream callers
//!     in `kay-core::loop` and `kay-cli::run`).
//!
//! If a future phase unblocks trybuild (or moves these fixtures into
//! a shim sub-crate), the `.fail.rs` fixtures in `tests/compile_fail/`
//! take over with negative stderr-snapshot verification. Until then
//! the positive `Arc<dyn _>` coercions here are the primary gate.
//!
//! Reference:
//!   - `.planning/REQUIREMENTS.md` (B1 Tool object-safety, D-02
//!     services handle)
//!   - `.planning/phases/05-agent-loop/05-PLAN.md` Wave 6c (T6c.1/2)
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use forge_domain::{FSRead, FSSearch, FSWrite, NetFetch, ToolName, ToolOutput};
use kay_tools::{
    ImageQuota, NoOpInnerAgent, ServicesHandle, Tool, ToolCallContext, ToolError, ToolRegistry,
    default_tool_set,
};
use serde_json::{Value, json};

// -----------------------------------------------------------------
// `Tool` object-safety canary (duplicate anchor)
// -----------------------------------------------------------------

struct MinTool {
    name: ToolName,
}

#[async_trait]
impl Tool for MinTool {
    fn name(&self) -> &ToolName {
        &self.name
    }
    fn description(&self) -> &str {
        "min"
    }
    fn input_schema(&self) -> Value {
        json!({"type": "object", "properties": {}, "required": []})
    }
    async fn invoke(
        &self,
        _a: Value,
        _c: &ToolCallContext,
        _id: &str,
    ) -> Result<ToolOutput, ToolError> {
        Ok(ToolOutput::text(""))
    }
}

#[test]
fn tool_trait_is_object_safe_via_arc_dyn_coercion() {
    // If someone adds a generic method to `Tool` this file stops
    // compiling (E0038) BEFORE any runtime assertion — the coercion
    // from `Arc<MinTool>` to `Arc<dyn Tool>` requires object-safety.
    let t: Arc<dyn Tool> = Arc::new(MinTool { name: ToolName::new("min") });
    assert_eq!(t.name().as_str(), "min");
    assert_eq!(t.description(), "min");
    let schema = t.input_schema();
    assert_eq!(schema["type"], "object");
}

// -----------------------------------------------------------------
// `ServicesHandle` object-safety canary
// -----------------------------------------------------------------

struct MinServices;

#[async_trait]
impl ServicesHandle for MinServices {
    async fn fs_read(&self, _input: FSRead) -> anyhow::Result<ToolOutput> {
        Ok(ToolOutput::text(""))
    }
    async fn fs_write(&self, _input: FSWrite) -> anyhow::Result<ToolOutput> {
        Ok(ToolOutput::text(""))
    }
    async fn fs_search(&self, _input: FSSearch) -> anyhow::Result<ToolOutput> {
        Ok(ToolOutput::text(""))
    }
    async fn net_fetch(&self, _input: NetFetch) -> anyhow::Result<ToolOutput> {
        Ok(ToolOutput::text(""))
    }
}

#[test]
fn services_handle_is_object_safe_via_arc_dyn_coercion() {
    // `ToolCallContext::new` takes `Arc<dyn ServicesHandle>`. If the
    // trait gains a generic method, the coercion below fails (E0038)
    // and this file stops compiling before any downstream caller
    // (forge_bridge, sage_query test NullServices, or the agent loop
    // in Phase 7) is affected — surfaces the regression at the
    // earliest possible point.
    //
    // The type coercion IS the assertion — the runtime use below
    // just prevents `let s = …` from being optimized out into a
    // dead store. No method is called; that would pull in the
    // `FSRead`/`FSWrite`/`FSSearch`/`NetFetch` input shapes which
    // upstream forge_domain may legitimately rev. Object-safety is
    // a property of the TRAIT, not of its arguments.
    let s: Arc<dyn ServicesHandle> = Arc::new(MinServices);
    assert!(Arc::strong_count(&s) >= 1);
}

// -----------------------------------------------------------------
// `default_tool_set` signature canary
// -----------------------------------------------------------------

#[test]
fn default_tool_set_returns_owned_registry() {
    // Locks `default_tool_set` returns `ToolRegistry` BY VALUE (not
    // `&ToolRegistry`, not `Box<ToolRegistry>`, not some wrapper). A
    // by-value return is what `kay-core::loop` (Phase 5 Wave 7) and
    // `kay-cli::run` (Phase 5 Wave 7+) expect when they build their
    // dispatcher from the default set. If a future refactor tries to
    // return a borrowed reference, THIS file's `let reg: ToolRegistry
    // = …` assignment stops compiling.
    let quota = Arc::new(ImageQuota::new(2, 20));
    let reg: ToolRegistry =
        default_tool_set(PathBuf::from("/tmp"), quota, Arc::new(NoOpInnerAgent));
    // Owned registry — we can mutate it, move it, drop it.
    assert!(
        reg.len() >= 8,
        "default_tool_set must register at least the 8 canonical \
         tools (execute_commands, task_complete, image_read, fs_read, \
         fs_write, fs_search, net_fetch, sage_query); got {}",
        reg.len()
    );
    // Ownership check: move the registry into a new binding. If the
    // return were a reference, this move wouldn't type-check.
    let moved = reg;
    drop(moved);
}
