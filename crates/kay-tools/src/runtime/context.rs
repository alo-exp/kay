//! ToolCallContext — frozen (B7/VAL-007) container carried through every
//! tool invocation. All forward-known fields are present from day one so
//! downstream plans 03-02/04/05 consume (never mutate) this shape.
//! `#[non_exhaustive]` future-proofs post-Phase-3 evolution without SemVer breaks.
//!
//! # Services handle (Rule-1 deviation from plan spec)
//!
//! The 03-01-PLAN draft specified the literal field
//! `services: Arc<dyn forge_app::Services>`. That compile-fails because
//! `forge_app::Services` is NOT dyn-compatible — it carries many associated
//! types (ProviderService, ConversationService, …) and requires `Clone`,
//! which disqualifies it from being used behind `dyn`. To preserve the
//! scaffold-compiles invariant, kay-tools introduces a local dyn-safe
//! `ServicesHandle` marker trait that Wave 1 (03-02) refines to a trait-object
//! facade over the ForgeCode `Services` bundle (or replaces with a generic
//! parameter; see 03-02 plan).
//!
//! Logged as Rule-1 deviation in 03-01-SUMMARY.md.

use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::events::AgentEvent;
use crate::quota::ImageQuota as ImageBudget;
use crate::seams::sandbox::Sandbox;
use crate::seams::verifier::TaskVerifier;

/// Dyn-safe handle to the ForgeCode `Services` bundle. Wave 1 fills in the
/// concrete API (likely `async fn fs_read(...)`, `fs_write(...)`, …) as
/// trait-object methods that forward to a generic `Services` impl held
/// behind this boundary. Kept empty in Wave 0 so the scaffold compiles.
pub trait ServicesHandle: Send + Sync + 'static {}

#[non_exhaustive]
pub struct ToolCallContext {
    /// Services bundle. Intended conceptual type: `services: Arc<dyn forge_app::Services>`.
    /// See module docs for why the scaffold uses a local dyn-safe
    /// `ServicesHandle` trait instead.
    pub services: Arc<dyn ServicesHandle>,
    pub stream_sink: Arc<dyn Fn(AgentEvent) + Send + Sync>,
    pub image_budget: Arc<ImageBudget>,
    pub cancel_token: CancellationToken,
    pub sandbox: Arc<dyn Sandbox>,
    pub verifier: Arc<dyn TaskVerifier>,
}
