//! OpenRouter-flavored mockito wrapper.
//!
//! Mirrors the shape of `kay_core::forge_repo::provider::mock_server::MockServer`
//! (see .planning/phases/02-provider-hal-tolerant-json-parser/02-PATTERNS.md
//! §tests/mock_server.rs for the analog) but speaks the OpenRouter endpoint
//! `/api/v1/chat/completions`. Kept independent of kay-core so it compiles
//! during the structural-rename work in plans 02-02 through 02-05.

#![allow(dead_code)] // populated by downstream plans; not every helper used in this plan

use mockito::{Mock, Server, ServerGuard};

pub struct MockServer {
    server: ServerGuard,
}

impl MockServer {
    pub async fn new() -> Self {
        let server = Server::new_async().await;
        Self { server }
    }

    pub fn url(&self) -> String {
        self.server.url()
    }

    /// OpenRouter-flavored SSE mock: /api/v1/chat/completions,
    /// text/event-stream, final chunk may carry usage.cost per §Pitfall 4.
    pub async fn mock_openrouter_chat_stream(
        &mut self,
        events: Vec<String>,
        status: usize,
    ) -> Mock {
        let sse_body = events.join("\n\n");
        self.server
            .mock("POST", "/api/v1/chat/completions")
            .with_status(status)
            .with_header("content-type", "text/event-stream")
            .with_header("cache-control", "no-cache")
            .with_body(sse_body)
            .create_async()
            .await
    }

    /// 429 with Retry-After header (integer seconds per D-09 and §Pitfall 6).
    pub async fn mock_rate_limit(&mut self, retry_after_secs: u64) -> Mock {
        self.server
            .mock("POST", "/api/v1/chat/completions")
            .with_status(429)
            .with_header("retry-after", &retry_after_secs.to_string())
            .with_body(r#"{"error":{"code":"rate_limit","message":"rate limited"}}"#)
            .create_async()
            .await
    }

    /// 503 server error; no Retry-After; backon default applies.
    pub async fn mock_server_error_503(&mut self) -> Mock {
        self.server
            .mock("POST", "/api/v1/chat/completions")
            .with_status(503)
            .with_body(r#"{"error":{"code":"server_error","message":"upstream unavailable"}}"#)
            .create_async()
            .await
    }

    /// Generic status + body mock used by `tests/error_taxonomy.rs` to
    /// cover the full PROV-08 taxonomy (401 / 402 / 404 / 502 / etc.).
    pub async fn mock_status_body(&mut self, status: usize, body: &str) -> Mock {
        self.server
            .mock("POST", "/api/v1/chat/completions")
            .with_status(status)
            .with_body(body)
            .create_async()
            .await
    }

    /// Load an SSE cassette from tests/fixtures/sse/*.jsonl as a Vec<String>
    /// suitable for passing to mock_openrouter_chat_stream.
    ///
    /// Fixture format: one JSON event per line; blank lines between. The
    /// loader strips blanks and returns each JSON object wrapped with its
    /// `data: ` SSE prefix, ready for mockito body assembly.
    pub fn load_sse_cassette(name: &str) -> Vec<String> {
        let path = format!(
            "{}/tests/fixtures/sse/{}.jsonl",
            env!("CARGO_MANIFEST_DIR"),
            name
        );
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read cassette {path}: {e}"));
        raw.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| format!("data: {l}"))
            .collect()
    }
}
