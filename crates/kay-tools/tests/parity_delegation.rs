//! Wave 4 / 03-05 Task 1 parity gate.
//!
//! Proves that invoking the four parity tools (`fs_read`, `fs_write`,
//! `fs_search`, `net_fetch`) through the `Arc<dyn Tool>` registry path
//! produces byte-identical `ToolOutput` values versus calling the
//! underlying `forge_app::Fs*Service` impl directly and formatting via
//! `forge_bridge::format_*` helpers.
//!
//! Because the `ForgeServicesFacade` is the only path from a Tool impl
//! to the service layer, equality of:
//!
//!   facade(tool.invoke(..)).text  ==  format_*(service.call(..))
//!
//! is the Wave-4 parity gate. Phase 5 will upgrade this test to compare
//! against `ToolExecutor::execute(ToolCatalog::*)` once the full
//! `ForgeServices` bundle lands in kay-cli. The shape of this test
//! (facade path == direct path) is preserved across that upgrade.
//!
//! Additionally asserts object-safety of each tool — `Arc<dyn Tool>`
//! must round-trip through the registry and still dispatch.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::type_complexity)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use forge_app::{
    Content, FsReadService, FsSearchService, FsWriteService, FsWriteOutput, HttpResponse, Match,
    MatchResult, NetFetchService, ReadOutput, ResponseContext, SearchResult,
};
use forge_domain::{FSSearch, FileInfo, ToolOutput};
use kay_tools::forge_bridge::{
    format_fetch_output, format_read_output, format_search_output, format_write_output,
};
use kay_tools::{
    FsReadTool, FsSearchTool, FsWriteTool, ForgeServicesFacade, NetFetchTool, Tool,
    ToolCallContext,
};
use pretty_assertions::assert_eq;
use serde_json::json;

#[path = "support/mod.rs"]
mod support;

use support::{EventLog, make_ctx_with_services};

/// Shared recorder of the last args passed into each fake service — used
/// to verify that the tool forwarded inputs unchanged.
#[derive(Default)]
struct CallLog {
    read: Mutex<Option<(String, Option<u64>, Option<u64>)>>,
    write: Mutex<Option<(String, String, bool)>>,
    search: Mutex<Option<FSSearch>>,
    fetch: Mutex<Option<(String, Option<bool>)>>,
}

struct FakeFsRead {
    out: ReadOutput,
    log: Arc<CallLog>,
}

#[async_trait]
impl FsReadService for FakeFsRead {
    async fn read(
        &self,
        path: String,
        start_line: Option<u64>,
        end_line: Option<u64>,
    ) -> anyhow::Result<ReadOutput> {
        *self.log.read.lock().unwrap() = Some((path, start_line, end_line));
        Ok(ReadOutput {
            content: clone_content(&self.out.content),
            info: self.out.info.clone(),
        })
    }
}

struct FakeFsWrite {
    out: FsWriteOutput,
    log: Arc<CallLog>,
}

#[async_trait]
impl FsWriteService for FakeFsWrite {
    async fn write(
        &self,
        path: String,
        content: String,
        overwrite: bool,
    ) -> anyhow::Result<FsWriteOutput> {
        *self.log.write.lock().unwrap() = Some((path, content, overwrite));
        Ok(clone_write_output(&self.out))
    }
}

struct FakeFsSearch {
    out: Option<SearchResult>,
    log: Arc<CallLog>,
}

#[async_trait]
impl FsSearchService for FakeFsSearch {
    async fn search(&self, params: FSSearch) -> anyhow::Result<Option<SearchResult>> {
        *self.log.search.lock().unwrap() = Some(params);
        Ok(clone_search(&self.out))
    }
}

struct FakeNetFetch {
    out: HttpResponse,
    log: Arc<CallLog>,
}

#[async_trait]
impl NetFetchService for FakeNetFetch {
    async fn fetch(&self, url: String, raw: Option<bool>) -> anyhow::Result<HttpResponse> {
        *self.log.fetch.lock().unwrap() = Some((url, raw));
        Ok(clone_http(&self.out))
    }
}

// Content / FsWriteOutput / SearchResult / HttpResponse are not Clone.
// These helpers hand-clone what the fakes return so the service can
// yield the same value twice — once to the facade, once to the direct
// format comparison.

fn clone_content(c: &Content) -> Content {
    match c {
        Content::File(s) => Content::File(s.clone()),
        // Image path not exercised here — the fs_read parity test uses
        // File variant; Image is covered by the image_read tool's own
        // tests.
        Content::Image(_) => Content::File(String::new()),
    }
}

fn clone_write_output(o: &FsWriteOutput) -> FsWriteOutput {
    FsWriteOutput {
        path: o.path.clone(),
        before: o.before.clone(),
        errors: Vec::new(), // SyntaxError is Debug-only; none for this test
        content_hash: o.content_hash.clone(),
    }
}

fn clone_match_result(r: &Option<MatchResult>) -> Option<MatchResult> {
    r.as_ref().map(|r| match r {
        MatchResult::Error(s) => MatchResult::Error(s.clone()),
        MatchResult::Found { line_number, line } => MatchResult::Found {
            line_number: *line_number,
            line: line.clone(),
        },
        MatchResult::Count { count } => MatchResult::Count { count: *count },
        MatchResult::FileMatch => MatchResult::FileMatch,
        MatchResult::ContextMatch {
            line_number,
            line,
            before_context,
            after_context,
        } => MatchResult::ContextMatch {
            line_number: *line_number,
            line: line.clone(),
            before_context: before_context.clone(),
            after_context: after_context.clone(),
        },
    })
}

fn clone_search(s: &Option<SearchResult>) -> Option<SearchResult> {
    s.as_ref().map(|r| SearchResult {
        matches: r
            .matches
            .iter()
            .map(|m| Match {
                path: m.path.clone(),
                result: clone_match_result(&m.result),
            })
            .collect(),
    })
}

fn clone_http(h: &HttpResponse) -> HttpResponse {
    HttpResponse {
        content: h.content.clone(),
        code: h.code,
        content_type: h.content_type.clone(),
        context: match h.context {
            ResponseContext::Parsed => ResponseContext::Parsed,
            ResponseContext::Raw => ResponseContext::Raw,
        },
    }
}

// ---- factories that assemble a facade + matching services -----------

fn read_fixture() -> ReadOutput {
    ReadOutput {
        content: Content::File("hello\nworld\n".to_string()),
        info: FileInfo::new(1, 2, 2, "abc123".to_string()),
    }
}

fn write_fixture() -> FsWriteOutput {
    FsWriteOutput {
        path: "/tmp/out.txt".to_string(),
        before: None,
        errors: Vec::new(),
        content_hash: "deadbeef".to_string(),
    }
}

fn search_fixture() -> Option<SearchResult> {
    Some(SearchResult {
        matches: vec![
            Match {
                path: "/a.rs".to_string(),
                result: Some(MatchResult::Found {
                    line_number: Some(7),
                    line: "fn main() {}".to_string(),
                }),
            },
            Match {
                path: "/b.rs".to_string(),
                result: Some(MatchResult::Count { count: 3 }),
            },
        ],
    })
}

fn http_fixture() -> HttpResponse {
    HttpResponse {
        content: "<html>ok</html>".to_string(),
        code: 200,
        content_type: "text/html".to_string(),
        context: ResponseContext::Parsed,
    }
}

fn text(output: &ToolOutput) -> String {
    output.as_str().unwrap_or("").to_string()
}

// ---- tests ----------------------------------------------------------

#[tokio::test]
async fn fs_read_tool_matches_direct_format() {
    let log = Arc::new(CallLog::default());
    let fixture = read_fixture();

    // Direct path.
    let direct_body = format_read_output(&fixture);

    // Facade path through Tool::invoke.
    let facade = Arc::new(ForgeServicesFacade::new(
        Arc::new(FakeFsRead {
            out: ReadOutput {
                content: clone_content(&fixture.content),
                info: fixture.info.clone(),
            },
            log: log.clone(),
        }),
        Arc::new(FakeFsWrite {
            out: write_fixture(),
            log: log.clone(),
        }),
        Arc::new(FakeFsSearch {
            out: None,
            log: log.clone(),
        }),
        Arc::new(FakeNetFetch {
            out: http_fixture(),
            log: log.clone(),
        }),
    ));
    let ctx: ToolCallContext = make_ctx_with_services(EventLog::new(), facade);
    let tool: Arc<dyn Tool> = Arc::new(FsReadTool::new());
    let args = json!({"file_path": "/tmp/x.txt", "start_line": 1, "end_line": 2});

    let out = tool.invoke(args, &ctx, "call-1").await.expect("invoke ok");
    assert_eq!(text(&out), direct_body, "fs_read facade != direct format");

    // Verify args forwarded verbatim (ISize 1..=2 -> u64).
    let captured = log.read.lock().unwrap().clone().expect("recorded");
    assert_eq!(captured.0, "/tmp/x.txt");
    assert_eq!(captured.1, Some(1));
    assert_eq!(captured.2, Some(2));
}

#[tokio::test]
async fn fs_write_tool_matches_direct_format() {
    let log = Arc::new(CallLog::default());
    let fixture = write_fixture();
    let direct_body = format_write_output(&fixture);

    let facade = Arc::new(ForgeServicesFacade::new(
        Arc::new(FakeFsRead {
            out: read_fixture(),
            log: log.clone(),
        }),
        Arc::new(FakeFsWrite {
            out: clone_write_output(&fixture),
            log: log.clone(),
        }),
        Arc::new(FakeFsSearch {
            out: None,
            log: log.clone(),
        }),
        Arc::new(FakeNetFetch {
            out: http_fixture(),
            log: log.clone(),
        }),
    ));
    let ctx = make_ctx_with_services(EventLog::new(), facade);
    let tool: Arc<dyn Tool> = Arc::new(FsWriteTool::new());
    let args = json!({"path": "/tmp/out.txt", "content": "hi", "overwrite": true});

    let out = tool.invoke(args, &ctx, "call-2").await.expect("invoke ok");
    assert_eq!(text(&out), direct_body, "fs_write facade != direct format");

    let captured = log.write.lock().unwrap().clone().expect("recorded");
    assert_eq!(captured.0, "/tmp/out.txt");
    assert_eq!(captured.1, "hi");
    assert!(captured.2);
}

#[tokio::test]
async fn fs_search_tool_matches_direct_format() {
    let log = Arc::new(CallLog::default());
    let fixture = search_fixture();
    let direct_body = format_search_output(&fixture);

    let facade = Arc::new(ForgeServicesFacade::new(
        Arc::new(FakeFsRead {
            out: read_fixture(),
            log: log.clone(),
        }),
        Arc::new(FakeFsWrite {
            out: write_fixture(),
            log: log.clone(),
        }),
        Arc::new(FakeFsSearch {
            out: clone_search(&fixture),
            log: log.clone(),
        }),
        Arc::new(FakeNetFetch {
            out: http_fixture(),
            log: log.clone(),
        }),
    ));
    let ctx = make_ctx_with_services(EventLog::new(), facade);
    let tool: Arc<dyn Tool> = Arc::new(FsSearchTool::new());
    let args = json!({"pattern": "fn", "path": "/"});

    let out = tool.invoke(args, &ctx, "call-3").await.expect("invoke ok");
    assert_eq!(
        text(&out),
        direct_body,
        "fs_search facade != direct format"
    );

    let captured = log
        .search
        .lock()
        .unwrap()
        .as_ref()
        .map(|p| p.path.clone());
    assert_eq!(captured.flatten().as_deref(), Some("/"));
}

#[tokio::test]
async fn net_fetch_tool_matches_direct_format() {
    let log = Arc::new(CallLog::default());
    let fixture = http_fixture();
    let direct_body = format_fetch_output(&fixture);

    let facade = Arc::new(ForgeServicesFacade::new(
        Arc::new(FakeFsRead {
            out: read_fixture(),
            log: log.clone(),
        }),
        Arc::new(FakeFsWrite {
            out: write_fixture(),
            log: log.clone(),
        }),
        Arc::new(FakeFsSearch {
            out: None,
            log: log.clone(),
        }),
        Arc::new(FakeNetFetch {
            out: clone_http(&fixture),
            log: log.clone(),
        }),
    ));
    let ctx = make_ctx_with_services(EventLog::new(), facade);
    let tool: Arc<dyn Tool> = Arc::new(NetFetchTool::new());
    let args = json!({"url": "https://example.com/", "raw": false});

    let out = tool.invoke(args, &ctx, "call-4").await.expect("invoke ok");
    assert_eq!(
        text(&out),
        direct_body,
        "net_fetch facade != direct format"
    );

    let captured = log.fetch.lock().unwrap().clone().expect("recorded");
    assert_eq!(captured.0, "https://example.com/");
    assert_eq!(captured.1, Some(false));
}

// ---- object-safety -------------------------------------------------

#[test]
fn parity_tools_are_object_safe() {
    // If any of the four builtins lose dyn-compatibility, this fails to
    // compile. Keeping the assertion in a test body (rather than a
    // const) ensures it shows up in `cargo test` output.
    let _v: Vec<Arc<dyn Tool>> = vec![
        Arc::new(FsReadTool::new()),
        Arc::new(FsWriteTool::new()),
        Arc::new(FsSearchTool::new()),
        Arc::new(NetFetchTool::new()),
    ];
}
