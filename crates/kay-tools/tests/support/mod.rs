//! Shared test support for execute_commands integration tests.
//! Included via `#[path = "support/mod.rs"] mod support;` from each test file.
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use kay_tools::{
    AgentEvent, ImageQuota, NoOpSandbox, NoOpVerifier, ServicesHandle, ToolCallContext,
};
use tokio_util::sync::CancellationToken;

/// Empty services marker for integration tests — the real trait object
/// will be a forge_app::Services facade in Wave 4.
pub struct TestServices;
impl ServicesHandle for TestServices {}

/// Collected event log. Tests pop from `lock()` to assert event sequences.
#[derive(Clone, Default)]
pub struct EventLog(pub Arc<Mutex<Vec<AgentEvent>>>);

impl EventLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&self, ev: AgentEvent) {
        if let Ok(mut guard) = self.0.lock() {
            guard.push(ev);
        }
    }

    pub fn drain(&self) -> Vec<AgentEvent> {
        match self.0.lock() {
            Ok(mut g) => std::mem::take(&mut *g),
            Err(_) => Vec::new(),
        }
    }

    pub fn snapshot(&self) -> Vec<String> {
        match self.0.lock() {
            Ok(g) => g.iter().map(|e| format!("{e:?}")).collect(),
            Err(_) => Vec::new(),
        }
    }
}

/// Build a `ToolCallContext` wired to an `EventLog` so tests can assert
/// the sequence of emitted `AgentEvent`s.
pub fn make_ctx(log: EventLog) -> ToolCallContext {
    let log_arc = log.0.clone();
    let sink: Arc<dyn Fn(AgentEvent) + Send + Sync> = Arc::new(move |ev: AgentEvent| {
        if let Ok(mut guard) = log_arc.lock() {
            guard.push(ev);
        }
    });
    ToolCallContext::new(
        Arc::new(TestServices),
        sink,
        Arc::new(ImageQuota::new(2, 20)),
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(NoOpVerifier),
    )
}
