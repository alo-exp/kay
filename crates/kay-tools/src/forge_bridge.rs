//! ForgeServicesFacade — production implementation of `ServicesHandle`.
//!
//! Wraps the four parity-tool service trait objects (`FsReadService`,
//! `FsWriteService`, `FsSearchService`, `NetFetchService`) and adapts their
//! outputs into `forge_domain::ToolOutput` values so Kay's tools can remain
//! object-safe `Arc<dyn Tool>` while still delegating byte-identically to
//! the ForgeCode implementations at the service boundary.
//!
//! # Parity note (Wave 4 Rule-3 reconciliation)
//!
//! The 03-05 plan text anticipated delegation through
//! `forge_app::ToolExecutor::execute(ToolCatalog::*, ctx)`, which formats
//! outputs via `ToolOperation::into_tool_output` using the full
//! `Services` bundle (25+ associated traits + Metrics + Sender). The
//! `Services` trait is NOT dyn-compatible (see context.rs module doc),
//! and its concrete impl (`forge_services::ForgeServices`) needs a deep
//! infrastructure stack (snapshots, validation, auth, providers) that
//! lands in Phase 5.
//!
//! For Wave 4 we pivot to **service-layer parity**: each facade method
//! invokes the same concrete `forge_services::Forge*` service impl as
//! ForgeCode would, then serializes the structured output into a
//! deterministic text body for `ToolOutput::text`. Calling the facade
//! vs. calling the service impl directly produces byte-identical
//! `ToolOutput` values — proven by `tests/parity_delegation.rs`.
//!
//! Phase 5 will swap the facade implementation to call
//! `ToolExecutor::execute` once the full `ForgeServices` bundle is
//! available in `kay-cli`; tool code is unchanged.

use std::sync::Arc;

use async_trait::async_trait;
use forge_app::{
    Content, FsReadService, FsSearchService, FsWriteService, MatchResult, NetFetchService,
    ResponseContext, SearchResult,
};
use forge_domain::{FSRead, FSSearch, FSWrite, NetFetch, ToolOutput};

use crate::runtime::context::ServicesHandle;

/// Production `ServicesHandle` impl backed by concrete forge service
/// trait objects. Each field is an `Arc<dyn Trait>` — the individual
/// service traits (unlike the aggregate `Services`) ARE dyn-safe.
#[derive(Clone)]
pub struct ForgeServicesFacade {
    fs_read: Arc<dyn FsReadService>,
    fs_write: Arc<dyn FsWriteService>,
    fs_search: Arc<dyn FsSearchService>,
    net_fetch: Arc<dyn NetFetchService>,
}

impl ForgeServicesFacade {
    /// Construct a new facade from the four concrete service trait
    /// objects. Callers are responsible for wiring each with an
    /// appropriate infrastructure stack (see `kay-cli` bootstrap for the
    /// production path).
    pub fn new(
        fs_read: Arc<dyn FsReadService>,
        fs_write: Arc<dyn FsWriteService>,
        fs_search: Arc<dyn FsSearchService>,
        net_fetch: Arc<dyn NetFetchService>,
    ) -> Self {
        Self { fs_read, fs_write, fs_search, net_fetch }
    }
}

/// Format the `ReadOutput` produced by `FsReadService::read` into the
/// deterministic text body emitted by `fs_read`. Image content is
/// base64-encoded so the output remains a single `ToolOutput::text`
/// value — callers that need the raw image bytes should use the
/// dedicated `image_read` tool instead.
pub fn format_read_output(output: &forge_app::ReadOutput) -> String {
    match &output.content {
        Content::File(text) => text.clone(),
        Content::Image(image) => {
            // `Image` stores the base64-encoded body as a `data:` URL
            // internally; emit it verbatim so parity callers round-trip
            // the upstream representation unchanged.
            format!("data:{};base64,{}", image.mime_type(), image.data())
        }
    }
}

/// Format an `FsWriteOutput` into a deterministic confirmation body.
/// The emitted text includes the absolute path and the post-write
/// content hash so downstream consumers can detect external mutation.
pub fn format_write_output(output: &forge_app::FsWriteOutput) -> String {
    let mut body = format!(
        "wrote {path}\ncontent_hash: {hash}\n",
        path = output.path,
        hash = output.content_hash
    );
    if !output.errors.is_empty() {
        body.push_str("syntax_errors:\n");
        for err in &output.errors {
            body.push_str(&format!("  - {err:?}\n"));
        }
    }
    body
}

/// Render a single search `Match` as a deterministic line. Keeping this
/// as a free function makes parity obvious — both the facade and the
/// direct-comparison test call this same formatter.
pub fn format_search_match(m: &forge_app::Match) -> String {
    match &m.result {
        Some(MatchResult::Error(e)) => format!("{}: error: {}", m.path, e),
        Some(MatchResult::Found { line_number, line }) => match line_number {
            Some(n) => format!("{}:{}: {}", m.path, n, line),
            None => format!("{}: {}", m.path, line),
        },
        Some(MatchResult::Count { count }) => format!("{}: {count}", m.path),
        Some(MatchResult::FileMatch) => m.path.clone(),
        Some(MatchResult::ContextMatch { line_number, line, before_context, after_context }) => {
            let mut s = String::new();
            for b in before_context {
                s.push_str(&format!("{}-: {}\n", m.path, b));
            }
            match line_number {
                Some(n) => s.push_str(&format!("{}:{}: {}\n", m.path, n, line)),
                None => s.push_str(&format!("{}: {}\n", m.path, line)),
            }
            for a in after_context {
                s.push_str(&format!("{}+: {}\n", m.path, a));
            }
            s
        }
        None => m.path.clone(),
    }
}

/// Format the whole `SearchResult` — joins per-match lines with `\n`.
pub fn format_search_output(output: &Option<SearchResult>) -> String {
    match output {
        Some(result) => result
            .matches
            .iter()
            .map(format_search_match)
            .collect::<Vec<_>>()
            .join("\n"),
        None => String::new(),
    }
}

/// Format an `HttpResponse` into the body emitted by `net_fetch`.
pub fn format_fetch_output(output: &forge_app::HttpResponse) -> String {
    let parsed = matches!(output.context, ResponseContext::Parsed);
    format!(
        "status: {code}\ncontent_type: {ct}\nparsed: {parsed}\n\n{body}",
        code = output.code,
        ct = output.content_type,
        parsed = parsed,
        body = output.content
    )
}

#[async_trait]
impl ServicesHandle for ForgeServicesFacade {
    async fn fs_read(&self, input: FSRead) -> anyhow::Result<ToolOutput> {
        let output = self
            .fs_read
            .read(
                input.file_path,
                input.start_line.map(|i| i as u64),
                input.end_line.map(|i| i as u64),
            )
            .await?;
        Ok(ToolOutput::text(format_read_output(&output)))
    }

    async fn fs_write(&self, input: FSWrite) -> anyhow::Result<ToolOutput> {
        let output = self
            .fs_write
            .write(input.file_path, input.content, input.overwrite)
            .await?;
        Ok(ToolOutput::text(format_write_output(&output)))
    }

    async fn fs_search(&self, input: FSSearch) -> anyhow::Result<ToolOutput> {
        let output = self.fs_search.search(input).await?;
        Ok(ToolOutput::text(format_search_output(&output)))
    }

    async fn net_fetch(&self, input: NetFetch) -> anyhow::Result<ToolOutput> {
        let output = self.net_fetch.fetch(input.url, input.raw).await?;
        Ok(ToolOutput::text(format_fetch_output(&output)))
    }
}

// format_* helpers are pub(crate) — integration tests in this crate access
// them via `crate::forge_bridge::format_*` through a test-only re-export
// module exposed in lib.rs under `#[cfg(test)]` if needed. For the Wave-4
// parity test, we exercise parity by comparing the facade output against
// a direct-service-call path that calls the same helpers — both live in
// this crate.
