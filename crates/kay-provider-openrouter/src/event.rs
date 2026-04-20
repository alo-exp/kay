//! Compatibility re-export (Phase 3 Wave 2, plan 03-03, E1).
//!
//! The source of truth for `AgentEvent` and `ToolOutputChunk` is now
//! `kay_tools::events` — this module re-exports them so existing call-sites
//! that spell the path as `kay_provider_openrouter::event::AgentEvent`
//! keep compiling with NO behavioral change. The relocation was forced by
//! the need for `kay-tools` to emit tool-execution events without a
//! `kay-tools <-> kay-provider-openrouter` dependency cycle.
//!
//! DAG direction post-03-03:
//!   kay-provider-openrouter -> kay-tools -> kay-provider-errors
//!   kay-provider-openrouter -> kay-provider-errors

pub use kay_tools::events::AgentEvent;
#[allow(unused_imports)]
pub use kay_tools::events::ToolOutputChunk;
