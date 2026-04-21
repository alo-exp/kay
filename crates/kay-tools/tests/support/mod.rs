//! Shared test support for integration tests.
//! Included via `#[path = "support/mod.rs"] mod support;` from each test file.
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use forge_domain::{FSRead, FSSearch, FSWrite, NetFetch, ToolOutput};
use kay_tools::{
    AgentEvent, ImageQuota, NoOpSandbox, NoOpVerifier, ServicesHandle, ToolCallContext,
};
use tokio_util::sync::CancellationToken;

/// Minimal in-memory `ServicesHandle` implementation for tests that do not
/// exercise the parity tools (e.g. execute_commands and quota-boundary
/// tests). The four parity methods return an empty successful `ToolOutput`
/// so object construction succeeds; tests that actually need fs/net
/// behavior use `real_services` below.
pub struct TestServices;

#[async_trait]
impl ServicesHandle for TestServices {
    async fn fs_read(&self, _input: FSRead) -> anyhow::Result<ToolOutput> {
        Ok(ToolOutput::text(String::new()))
    }
    async fn fs_write(&self, _input: FSWrite) -> anyhow::Result<ToolOutput> {
        Ok(ToolOutput::text(String::new()))
    }
    async fn fs_search(&self, _input: FSSearch) -> anyhow::Result<ToolOutput> {
        Ok(ToolOutput::text(String::new()))
    }
    async fn net_fetch(&self, _input: NetFetch) -> anyhow::Result<ToolOutput> {
        Ok(ToolOutput::text(String::new()))
    }
}

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
    make_ctx_with_services(log, Arc::new(TestServices))
}

/// Variant of `make_ctx` that accepts a caller-supplied `ServicesHandle`
/// (used by parity tests to inject a real `ForgeServicesFacade`).
pub fn make_ctx_with_services(log: EventLog, services: Arc<dyn ServicesHandle>) -> ToolCallContext {
    let log_arc = log.0.clone();
    let sink: Arc<dyn Fn(AgentEvent) + Send + Sync> = Arc::new(move |ev: AgentEvent| {
        if let Ok(mut guard) = log_arc.lock() {
            guard.push(ev);
        }
    });
    // nesting_depth = 0: tests in kay-tools are all top-level invocations
    // (sage_query's depth-threading logic is tested independently in
    // crates/kay-tools/tests/sage_query.rs with explicit depths).
    ToolCallContext::new(
        services,
        sink,
        Arc::new(ImageQuota::new(2, 20)),
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(NoOpVerifier),
        0,
    )
}

/// Variant with a custom `ImageQuota` — used by the image_quota
/// integration test to drive boundary conditions deterministically.
pub fn make_ctx_with_quota(log: EventLog, quota: Arc<ImageQuota>) -> ToolCallContext {
    let log_arc = log.0.clone();
    let sink: Arc<dyn Fn(AgentEvent) + Send + Sync> = Arc::new(move |ev: AgentEvent| {
        if let Ok(mut guard) = log_arc.lock() {
            guard.push(ev);
        }
    });
    ToolCallContext::new(
        Arc::new(TestServices),
        sink,
        quota,
        CancellationToken::new(),
        Arc::new(NoOpSandbox),
        Arc::new(NoOpVerifier),
        0,
    )
}
