//! Agent event stream type (A2 resolution).
//!
//! `AgentEvent` originates in kay-tools. kay-provider-openrouter re-exports
//! it via `pub use kay_tools::events::AgentEvent;` to preserve existing
//! call-sites without introducing a circular dependency.
//! DAG direction: kay-provider-openrouter → kay-tools → forge_app.
//!
//! NOTE: `VerificationOutcome` is defined in `crate::seams::verifier`
//! alongside the `TaskVerifier` trait (E1 four-module layout — all
//! trait-based seams live in the `seams` module).

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    // Wave 1 (03-02) fills the full variant set per 03-RESEARCH §7.
    // Placeholder marker variant kept so the enum compiles until Wave 1.
    #[doc(hidden)]
    __NonExhaustive,
}
