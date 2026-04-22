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
//! `ServicesHandle` trait that Wave 4 (03-05) refines into a facade over
//! the four parity-tool service operations (fs_read, fs_write, fs_search,
//! net_fetch). The field type `services: Arc<dyn ServicesHandle>` is
//! unchanged (B7 honored); only the trait gains methods.
//!
//! Logged as Rule-1 deviation in 03-01-SUMMARY.md and Rule-3 reconciliation
//! in 03-05-SUMMARY.md.

use std::sync::Arc;

use async_trait::async_trait;
use forge_domain::{FSRead, FSSearch, FSWrite, NetFetch, ToolOutput};
use tokio_util::sync::CancellationToken;

use crate::events::AgentEvent;
use crate::quota::ImageQuota as ImageBudget;
use crate::seams::sandbox::Sandbox;
use crate::seams::verifier::TaskVerifier;

/// Dyn-safe handle to the ForgeCode service surface used by the four
/// parity tools (`fs_read`, `fs_write`, `fs_search`, `net_fetch`).
///
/// Wave 4 (03-05) added the four async methods below. Production
/// implementations wrap the concrete `forge_services::Forge*` service
/// impls; parity tools delegate directly through this trait so the
/// registry can remain object-safe (`Arc<dyn Tool>`).
#[async_trait]
pub trait ServicesHandle: Send + Sync + 'static {
    /// Delegate an `fs_read` call to the underlying FsReadService impl and
    /// return a normalized `ToolOutput`. Byte-identical to upstream
    /// `ForgeFsRead::read` at the service layer.
    async fn fs_read(&self, input: FSRead) -> anyhow::Result<ToolOutput>;

    /// Delegate an `fs_write` call. Byte-identical to upstream
    /// `ForgeFsWrite::write` at the service layer.
    async fn fs_write(&self, input: FSWrite) -> anyhow::Result<ToolOutput>;

    /// Delegate an `fs_search` call. Byte-identical to upstream
    /// `ForgeFsSearch::search` at the service layer.
    async fn fs_search(&self, input: FSSearch) -> anyhow::Result<ToolOutput>;

    /// Delegate a `net_fetch` call. Byte-identical to upstream
    /// `ForgeFetch::fetch` at the service layer.
    async fn net_fetch(&self, input: NetFetch) -> anyhow::Result<ToolOutput>;
}

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
    /// Agent-loop nesting depth. 0 for top-level turns; incremented by
    /// one for each recursive sub-turn spawned via `sage_query`. The
    /// `sage_query` tool reads this field at invocation time and
    /// rejects with [`crate::error::ToolError::NestingDepthExceeded`]
    /// when the parent depth is ≥ 2 (i.e. the sub-turn would be at
    /// depth 3 — beyond the Phase 5 LOOP-03 runaway-sub-query ceiling).
    ///
    /// A `u8` is sufficient (max legal value is 2; anything above is
    /// rejected before it can be built) and keeps the context cache-
    /// line-friendly. `u16`+ would waste space without ever being
    /// used.
    ///
    /// Threading the counter through the context — rather than
    /// making it a hidden global — means a future wave can display
    /// it to the user ("🪆 sub-query @ depth 2/2") and the test
    /// seam (`RecordingAgent` in `tests/sage_query.rs`) can snapshot
    /// it directly without reaching into crate internals.
    pub nesting_depth: u8,
}

impl ToolCallContext {
    /// Construct a `ToolCallContext`. External crates (integration tests,
    /// Wave 4 `default_tool_set` callers) cannot use struct-literal syntax
    /// because of `#[non_exhaustive]`; this constructor is the canonical
    /// entry point. Plan 03-04 Wave 3 introduced this as a Rule-3 scaffold
    /// augmentation.
    ///
    /// Phase 5 Wave 5 T5.2 added the 7th parameter `nesting_depth: u8`.
    /// Top-level callers (CLI, test harnesses) pass `0`; `sage_query`'s
    /// internal inner-ctx builder passes `parent.nesting_depth + 1`.
    /// Making the param required — rather than defaulting to 0 via a
    /// builder pattern — is deliberate: forgetting to thread the depth
    /// on a new sub-tool would silently defeat the LOOP-03 guard, so
    /// the compiler surfaces the omission at every call site.
    pub fn new(
        services: Arc<dyn ServicesHandle>,
        stream_sink: Arc<dyn Fn(AgentEvent) + Send + Sync>,
        image_budget: Arc<ImageBudget>,
        cancel_token: CancellationToken,
        sandbox: Arc<dyn Sandbox>,
        verifier: Arc<dyn TaskVerifier>,
        nesting_depth: u8,
    ) -> Self {
        Self {
            services,
            stream_sink,
            image_budget,
            cancel_token,
            sandbox,
            verifier,
            nesting_depth,
        }
    }

    /// Minimal context for unit tests. Uses no-op impls for all seams
    /// and `nesting_depth = 0` (top-level turn).
    #[cfg(test)]
    pub fn for_test() -> Self {
        use crate::seams::sandbox::NoOpSandbox;
        use crate::seams::verifier::NoOpVerifier;

        struct NullServices;
        #[async_trait::async_trait]
        impl ServicesHandle for NullServices {
            async fn fs_read(&self, _: FSRead) -> anyhow::Result<ToolOutput> {
                Ok(ToolOutput::text(""))
            }
            async fn fs_write(&self, _: FSWrite) -> anyhow::Result<ToolOutput> {
                Ok(ToolOutput::text(""))
            }
            async fn fs_search(&self, _: FSSearch) -> anyhow::Result<ToolOutput> {
                Ok(ToolOutput::text(""))
            }
            async fn net_fetch(&self, _: NetFetch) -> anyhow::Result<ToolOutput> {
                Ok(ToolOutput::text(""))
            }
        }

        Self::new(
            Arc::new(NullServices),
            Arc::new(|_| {}),
            Arc::new(ImageBudget::new(u32::MAX, u32::MAX)),
            CancellationToken::new(),
            Arc::new(NoOpSandbox),
            Arc::new(NoOpVerifier),
            0,
        )
    }
}

#[cfg(test)]
mod phase8_ctx_tests {
    #[test]
    fn fresh_task_context_is_empty() {
        todo!("RED: task_context field not yet added — W-4 GREEN will implement")
    }

    #[test]
    fn append_task_context_accumulates() {
        todo!("RED: task_context field not yet added — W-4 GREEN will implement")
    }

    #[test]
    fn snapshot_is_independent() {
        todo!("RED: task_context field not yet added — W-4 GREEN will implement")
    }
}
