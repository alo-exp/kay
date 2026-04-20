//! Integration test: malformed tool_call arguments never terminate the
//! stream (PROV-05, plan 02-09).
//!
//! The `tool_call_malformed.jsonl` cassette sends `{cmd: "ls," }` as the
//! tool-call arguments fragment — unquoted key + trailing comma inside
//! the string value's closing. `forge_json_repair` typically handles both
//! flaws, so the expected event is `ToolCallComplete` with repaired JSON.
//! If a future repair-library change causes pass 2 to fail, the stream
//! should emit `ToolCallMalformed` (Ok variant) instead — what is NOT
//! acceptable is either a panic or an `AgentEvent::Error` / terminating
//! `ProviderError`.
//!
//! Assertions:
//!   - No `Err(ProviderError)` surfaces from the stream.
//!   - No `AgentEvent::Error` variant is observed.
//!   - Exactly one terminal event (Complete OR Malformed) keyed to
//!     `call_malformed`.
//!   - The trailing `Usage` frame still arrives — proof that the malformed
//!     tool_call did NOT interrupt the rest of the stream.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use futures::StreamExt;
use kay_provider_openrouter::{
    AgentEvent, Allowlist, ChatRequest, Message, OpenRouterProvider, Provider,
};

use crate::common::mock_server::MockServer;

fn launch_allowlist() -> Allowlist {
    let path = format!(
        "{}/tests/fixtures/config/allowlist.json",
        env!("CARGO_MANIFEST_DIR")
    );
    Allowlist::from_path(&path).expect("load fixture")
}

#[tokio::test]
async fn malformed_tool_call_emits_repaired_or_malformed_never_panics() {
    let mut srv = MockServer::new().await;
    let events = MockServer::load_sse_cassette("tool_call_malformed");
    let _m = srv.mock_openrouter_chat_stream(events, 200).await;

    let provider = OpenRouterProvider::builder()
        .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
        .api_key("test-key-tool-call-malformed")
        .allowlist(launch_allowlist())
        .build()
        .expect("build provider");

    let req = ChatRequest {
        model: "anthropic/claude-sonnet-4.6".into(),
        messages: vec![Message {
            role: "user".into(),
            content: "run ls".into(),
            tool_call_id: None,
        }],
        tools: vec![],
        temperature: None,
        max_tokens: None,
    };

    // Collect the whole stream. If the translator ever panics, this test
    // crashes (and the crash IS the failure). If it ever emits a
    // ProviderError that terminates the stream, we assert-fail with a
    // detailed message.
    let mut stream = provider.chat(req).await.expect("chat returns stream");
    let mut collected: Vec<AgentEvent> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    while let Some(ev) = stream.next().await {
        match ev {
            Ok(e) => collected.push(e),
            Err(e) => errors.push(format!("{e:?}")),
        }
    }

    // Key assertion: the malformed cassette MUST NOT surface as a
    // ProviderError in the Err path. Either it's Repaired (emitted as
    // ToolCallComplete) or it's Malformed (emitted as ToolCallMalformed).
    // Both are Ok(...) variants.
    assert!(
        errors.is_empty(),
        "malformed cassette must not produce ProviderError terminal; got: {errors:?}"
    );

    // No AgentEvent::Error variant either.
    let agent_errors: Vec<&AgentEvent> = collected
        .iter()
        .filter(|e| matches!(e, AgentEvent::Error { .. }))
        .collect();
    assert!(
        agent_errors.is_empty(),
        "no AgentEvent::Error expected, got: {agent_errors:?}"
    );

    // Exactly one of: ToolCallComplete{call_malformed} OR
    // ToolCallMalformed{call_malformed}. Accept either outcome; both are
    // valid non-panic non-terminal dispositions.
    let terminated: Vec<&AgentEvent> = collected
        .iter()
        .filter(|e| match e {
            AgentEvent::ToolCallComplete { id, .. } => id == "call_malformed",
            AgentEvent::ToolCallMalformed { id, .. } => id == "call_malformed",
            _ => false,
        })
        .collect();
    assert_eq!(
        terminated.len(),
        1,
        "expected exactly one terminal event for call_malformed, got: {collected:?}"
    );

    // Usage STILL arrives after a malformed tool_call — proves the stream
    // was not interrupted. If the malformed handling incorrectly yielded
    // an Err or returned early, usage would be missing.
    let usages: Vec<&AgentEvent> = collected
        .iter()
        .filter(|e| matches!(e, AgentEvent::Usage { .. }))
        .collect();
    assert_eq!(
        usages.len(),
        1,
        "expected exactly one Usage event, got: {collected:?}"
    );
}

/// REVIEW LO-04: the `tool_call_malformed` cassette above happens to carry
/// JSON that `forge_json_repair` successfully repairs today, so it never
/// exercises the `ParseOutcome::Malformed` branch in the translator — any
/// future refactor that accidentally reverts the plan 02-09 upgrade (e.g.
/// surfacing `ProviderError::ToolCallMalformed` as a terminating error
/// instead of an `Ok(AgentEvent::ToolCallMalformed)` data frame) would
/// still pass that test because the Repaired path alone satisfies its
/// assertions.
///
/// This test uses a cassette (`tool_call_catastrophic.jsonl`) whose
/// arguments are `{{}}}` — documented in
/// `tool_parser::unit::catastrophic_input_malformed` as a pass-2
/// unrepairable input. It asserts EXACTLY `ToolCallMalformed` is emitted
/// AND the stream continues to deliver `Usage`. Locks in the never-
/// terminate-on-malformed-tool-call invariant end-to-end.
#[tokio::test]
async fn catastrophic_tool_call_emits_malformed_and_usage_continues() {
    let mut srv = MockServer::new().await;
    let events = MockServer::load_sse_cassette("tool_call_catastrophic");
    let _m = srv.mock_openrouter_chat_stream(events, 200).await;

    let provider = OpenRouterProvider::builder()
        .endpoint(format!("{}/api/v1/chat/completions", srv.url()))
        .api_key("test-key-tool-call-catastrophic")
        .allowlist(launch_allowlist())
        .build()
        .expect("build provider");

    let req = ChatRequest {
        model: "anthropic/claude-sonnet-4.6".into(),
        messages: vec![Message {
            role: "user".into(),
            content: "run ls".into(),
            tool_call_id: None,
        }],
        tools: vec![],
        temperature: None,
        max_tokens: None,
    };

    let mut stream = provider.chat(req).await.expect("chat returns stream");
    let mut collected: Vec<AgentEvent> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    while let Some(ev) = stream.next().await {
        match ev {
            Ok(e) => collected.push(e),
            Err(e) => errors.push(format!("{e:?}")),
        }
    }

    // Same non-termination invariant as the sibling test.
    assert!(
        errors.is_empty(),
        "catastrophic cassette must not produce ProviderError terminal; got: {errors:?}"
    );

    // Key LO-04 assertion: the Malformed branch IS executed. We expect
    // exactly one AgentEvent::ToolCallMalformed keyed to
    // `call_catastrophic`, and exactly zero AgentEvent::ToolCallComplete
    // for that id.
    let malformed: Vec<&AgentEvent> = collected
        .iter()
        .filter(|e| matches!(
            e,
            AgentEvent::ToolCallMalformed { id, .. } if id == "call_catastrophic"
        ))
        .collect();
    assert_eq!(
        malformed.len(),
        1,
        "expected exactly one ToolCallMalformed for call_catastrophic — \
         cassette should reliably hit ParseOutcome::Malformed. Got: {collected:?}"
    );
    let completes: Vec<&AgentEvent> = collected
        .iter()
        .filter(|e| matches!(
            e,
            AgentEvent::ToolCallComplete { id, .. } if id == "call_catastrophic"
        ))
        .collect();
    assert_eq!(
        completes.len(),
        0,
        "catastrophic cassette must NOT produce ToolCallComplete (the \
         repair path must have failed). Got: {collected:?}"
    );

    // Raw payload is preserved on Malformed for post-hoc diagnosis.
    match malformed[0] {
        AgentEvent::ToolCallMalformed { raw, error, .. } => {
            assert!(
                !error.is_empty(),
                "ToolCallMalformed.error must carry a diagnostic"
            );
            assert_eq!(
                raw, "{{}}}",
                "ToolCallMalformed.raw must preserve the original bytes"
            );
        }
        other => panic!("expected ToolCallMalformed, got {other:?}"),
    }

    // Stream continuation proof: Usage still arrives AFTER the malformed
    // tool_call. If the translator accidentally reverted to terminating
    // `ProviderError::ToolCallMalformed`, Usage would be missing.
    let usages: Vec<&AgentEvent> = collected
        .iter()
        .filter(|e| matches!(e, AgentEvent::Usage { .. }))
        .collect();
    assert_eq!(
        usages.len(),
        1,
        "expected exactly one Usage event; stream must continue past \
         Malformed. Got: {collected:?}"
    );
}
