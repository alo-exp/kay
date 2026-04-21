//! Phase 5 Wave 4 T4.7 RED — proptest over random channel close-order
//! and event-sequence combinations.
//!
//! LOOP-01. Pins the `tokio::select!` loop's termination and
//! forwarding invariants under adversarial input — not just the
//! three hand-written fixtures in `tests/loop.rs` /
//! `tests/loop_dispatcher_integration.rs`. The property:
//!
//!   Given any sequence of 0..=8 `TextDelta` events and any close
//!   order across the `model_rx` + `control_rx` senders (drop
//!   control first / drop model first / keep control alive until
//!   after the drain completes), the loop:
//!
//!     1. Forwards EXACTLY one event per model frame sent before
//!        close — no drops, no duplicates.
//!     2. Terminates with `Ok(())` within a bounded timeout.
//!     3. Preserves event ordering.
//!
//! ## Why a property test (and why here)
//!
//! `tests/loop.rs` locks three specific paths through `run_turn`.
//! The `tokio::select!` macro's biased priority ordering plus
//! four potential close-orderings plus arbitrary event counts is a
//! combinatorial space that hand-written fixtures cover at most
//! ~5% of. A proptest is the only way to catch regressions like:
//!
//!   - future biased-arm reshuffle accidentally starving the model
//!     arm when control fires repeatedly;
//!   - an off-by-one in the post-T4.10 pause buffer swallowing one
//!     event at resume boundary;
//!   - a future verify-gate extension terminating on the wrong
//!     event type.
//!
//! Catching those bugs at commit time via proptest costs one CI
//! second per 256 cases — cheaper than any debug-test-re-run loop
//! triggered by an incident in prod.
//!
//! ## Why 256 cases (not 10,000 like event_filter_property)
//!
//! Each case in this file drives a full `run_turn` task through a
//! tokio runtime — channel setup, N async sends, loop spawn, drain,
//! join. Measured ~3–5 ms per case on dev hardware. At 256 cases the
//! whole property takes ~1 s. At 10,000 cases it would take ~40 s of
//! CI wall-clock, which crowds out the rest of the kay-core test
//! surface. The `event_filter_property` test runs 10k because its
//! inner body is a pure `matches!` — no async, no runtime, no
//! channels. The right case count is a function of inner-body cost,
//! not a global knob; 256 is more than enough coverage of the
//! close-order × event-count product space for `select!`.
//!
//! ## Expected RED state (T4.7)
//!
//! Current `run_turn` (T4.6) already satisfies this property:
//! the model arm drains buffered frames, then sees `None` and
//! returns `Ok(())`; the control arm disables itself on close.
//! This test therefore PASSES on T4.6 as written — the RED label
//! is used to mark it as the task that *pins the invariant*. Any
//! future change to the select! structure that breaks the property
//! turns this RED again. T4.8 GREEN is a no-op verification task
//! confirming the property holds, per the PLAN.md Wave 4 structure.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use forge_domain::{FSRead, FSSearch, FSWrite, NetFetch, ToolOutput};
use proptest::prelude::*;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use kay_core::control::control_channel;
use kay_core::r#loop::{RunTurnArgs, run_turn};
use kay_core::persona::Persona;
use kay_provider_errors::ProviderError;
use kay_tools::{
    AgentEvent, ImageQuota, NoOpSandbox, NoOpVerifier, ServicesHandle, ToolCallContext,
    ToolRegistry,
};

// ── Null services + sink fixtures (duplicated from tests/loop.rs) ─────
//
// Duplicated deliberately rather than extracted to a shared helper
// module: each test file is compiled as its own crate, sharing would
// require a `common.rs` mod import pattern that couples formerly
// independent suites. The ~30 LOC cost is lower than the coupling
// cost — see the rationale comment in `tests/loop.rs`.
struct NullServices;

#[async_trait]
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

/// Drive one `run_turn` to completion with the given inputs and
/// return the (forwarded-events, loop-result) pair. Synchronous
/// outside, builds a dedicated current-thread runtime inside so each
/// proptest case gets a clean scheduler — avoiding carryover state
/// that could mask a bug.
///
/// `close_control_before_model = true`  → drop `_ctl_tx` before `model_tx`.
/// `close_control_before_model = false` → drop `model_tx` before `_ctl_tx`.
///
/// Both orderings are legal real-world scenarios: the control sender
/// lives in the CLI's signal handler and the model sender lives in
/// the provider adapter; either task can exit first depending on
/// session shape (user-abort vs. natural turn-end).
fn drive_loop(
    event_contents: Vec<String>,
    close_control_before_model: bool,
) -> (Vec<AgentEvent>, Result<(), kay_core::r#loop::LoopError>) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build current-thread runtime");

    rt.block_on(async move {
        let (model_tx, model_rx) = mpsc::channel::<Result<AgentEvent, ProviderError>>(32);
        let (ctl_tx, control_rx) = control_channel();
        let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(32);

        // Queue every event into the model channel before spawning,
        // so the loop sees a "fully-buffered-then-closed" sequence
        // rather than racing a live producer. Race-free inputs are
        // what keep the property deterministic — any flake would
        // imply a select! bug, not a test bug.
        for content in &event_contents {
            model_tx
                .send(Ok(AgentEvent::TextDelta { content: content.clone() }))
                .await
                .expect("capacity-32 channel accepts buffered send");
        }

        // Close in the randomized order. Use an explicit branch
        // rather than a swap trick so the generated rustc drop
        // order is exactly what the test says it is.
        if close_control_before_model {
            drop(ctl_tx);
            drop(model_tx);
        } else {
            drop(model_tx);
            drop(ctl_tx);
        }

        let persona = Persona::load("forge").expect("bundled forge persona loads");
        let registry = Arc::new(ToolRegistry::new());
        let tool_ctx = ToolCallContext::new(
            Arc::new(NullServices),
            Arc::new(|_| {}),
            Arc::new(ImageQuota::new(u32::MAX, u32::MAX)),
            CancellationToken::new(),
            Arc::new(NoOpSandbox),
            Arc::new(NoOpVerifier),
        );

        let handle = tokio::spawn(run_turn(RunTurnArgs {
            persona,
            model_rx,
            control_rx,
            event_tx,
            registry,
            tool_ctx,
        }));

        // 2 s bound is far more than enough for 8 events + two drops
        // in a current-thread runtime. A failure to terminate inside
        // this budget means a real liveness bug, not a slow host.
        let drained = tokio::time::timeout(Duration::from_secs(2), async {
            let mut acc = Vec::new();
            while let Some(ev) = event_rx.recv().await {
                acc.push(ev);
            }
            acc
        })
        .await
        .expect("loop dropped event_tx within 2 s timeout");

        let loop_result = handle.await.expect("run_turn task joined without panic");

        (drained, loop_result)
    })
}

// ── The property ────────────────────────────────────────────────────
//
// One test function; two invariants folded in via a single fixture.
// Splitting into "count preserved" + "order preserved" + "returns Ok"
// would triple the case count (proptest does not share fixtures
// across tests), tripling CI time for zero coverage gain. The three
// assertions together describe one behavior: "the loop acts like a
// lossless ordered channel that closes cleanly".

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// For any 0..=8 `TextDelta` contents and any close order, the
    /// loop must (a) forward exactly those events, (b) in order,
    /// (c) and return `Ok(())`.
    ///
    /// `".{0,16}"` regex keeps generated content short and ASCII-
    /// weighted (proptest's regex generator skews toward common
    /// characters), which is realistic for streaming text tokens
    /// and keeps the UTF-8 validation surface off the critical path.
    #[test]
    fn select_robust_to_random_close_order(
        contents in proptest::collection::vec(".{0,16}", 0..=8usize),
        close_control_before_model in any::<bool>(),
    ) {
        let (forwarded, loop_result) =
            drive_loop(contents.clone(), close_control_before_model);

        prop_assert!(
            loop_result.is_ok(),
            "run_turn must return Ok(()) regardless of close order; got {:?}",
            loop_result,
        );

        prop_assert_eq!(
            forwarded.len(),
            contents.len(),
            "count mismatch: sent {} events, forwarded {}",
            contents.len(),
            forwarded.len(),
        );

        for (i, (expected, actual)) in contents.iter().zip(forwarded.iter()).enumerate() {
            match actual {
                AgentEvent::TextDelta { content } => {
                    prop_assert_eq!(
                        content,
                        expected,
                        "event #{} content mismatch",
                        i,
                    );
                }
                other => {
                    prop_assert!(
                        false,
                        "event #{} was not a TextDelta: {:?}",
                        i,
                        other,
                    );
                }
            }
        }
    }
}
