//! I-04: Phase 3 events cross a tokio mpsc boundary intact.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_tools::events::{AgentEvent, ToolOutputChunk};
use kay_tools::seams::verifier::VerificationOutcome;
use tokio::sync::mpsc;

#[tokio::test]
async fn phase3_events_flow_through_registry_dispatch() {
    let (tx, mut rx) = mpsc::unbounded_channel::<AgentEvent>();

    tx.send(AgentEvent::ToolOutput {
        call_id: "c".into(),
        chunk: ToolOutputChunk::Stdout("line".into()),
    })
    .unwrap();
    tx.send(AgentEvent::TaskComplete {
        call_id: "c".into(),
        verified: false,
        outcome: VerificationOutcome::Pending { reason: "p".into() },
    })
    .unwrap();
    tx.send(AgentEvent::ImageRead {
        path: "/tmp/x.png".into(),
        bytes: vec![0x89],
    })
    .unwrap();
    drop(tx);

    let e1 = rx.recv().await.expect("first event");
    assert!(matches!(e1, AgentEvent::ToolOutput { .. }), "got {e1:?}");
    let e2 = rx.recv().await.expect("second event");
    assert!(matches!(e2, AgentEvent::TaskComplete { .. }), "got {e2:?}");
    let e3 = rx.recv().await.expect("third event");
    assert!(matches!(e3, AgentEvent::ImageRead { .. }), "got {e3:?}");
    assert!(rx.recv().await.is_none(), "channel should be closed");
}
