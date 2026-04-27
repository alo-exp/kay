//! Tool Call Execution Infrastructure for Kay
//!
//! ## Architecture Note
//!
//! This module defines the ToolExecutor and ToolInput types for tool execution.
//! Current implementation: Agent loop uses `kay_tools::runtime::dispatcher::dispatch()`
//! for actual tool execution via the Services layer.
//!
//! This module is a design document / reference implementation for future
//! integration where ToolExecutor would be the primary dispatch mechanism.
//!
//! ## Current Tool Execution Path
//!
//! 1. Agent loop receives `AgentEvent::ToolCallComplete`
//! 2. `dispatch()` from `runtime::dispatcher` is called
//! 3. Dispatcher looks up tool in registry and executes via Service layer
//! 4. Result is returned as `ToolOutput` and sent back to provider
//!
//! ## Design (Future Integration)
//!
//! - ToolExecutor would wrap the dispatcher and provide a unified interface
//! - Results formatted and returned as ToolOutput
//! - Read-before-edit enforcement for patch/overwrite operations
//! - Path normalization for relative paths

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::fs::{FsPatchInput, FsReadInput, FsSearchInput, FsWriteInput};
use crate::shell::ShellInput;
use crate::fetch::NetFetchInput;

/// Tool input types - these correspond to the tools the agent can call
#[derive(Debug, Clone)]
pub enum ToolInput {
    /// Read file contents
    Read(FsReadInput),
    /// Write file contents
    Write(FsWriteInput),
    /// Patch file (find/replace)
    Patch(FsPatchInput),
    /// Search files by content
    Search(FsSearchInput),
    /// Execute shell command
    Shell(ShellInput),
    /// Fetch URL content
    Fetch(NetFetchInput),
    /// Remove file
    Remove(RemoveInput),
    /// Undo last change
    Undo(UndoInput),
    /// Multi-patch (atomic multi-edit)
    MultiPatch(MultiPatchInput),
    /// Todo write
    TodoWrite(TodoWriteInput),
    /// Todo read
    TodoRead(TodoReadInput),
}

/// Input for file removal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveInput {
    pub path: String,
}

/// Input for undo operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoInput {
    pub path: String,
    pub description: Option<String>,
}

/// Input for multi-patch (atomic multi-edit)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiPatchInput {
    pub file_path: String,
    pub edits: Vec<Edit>,
}

/// A single edit operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edit {
    pub old_string: String,
    pub new_string: String,
}

/// Input for todo operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoWriteInput {
    pub todos: Vec<TodoItem>,
}

/// A todo item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub content: String,
    pub status: TodoStatus,
}

/// Todo status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    InProgress,
    Completed,
    Cancelled,
    Pending,
}

/// Output from tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolOutput {
    /// The tool that was executed
    pub tool: String,
    /// Whether execution succeeded
    pub success: bool,
    /// Output content (may be truncated)
    pub content: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Metrics about the execution
    pub metrics: ToolMetrics,
}

/// Execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolMetrics {
    /// Lines read/written/modified
    pub lines_affected: u32,
    /// Files accessed
    pub files_accessed: Vec<String>,
    /// Execution time in ms
    pub execution_time_ms: u64,
    /// Whether output was truncated
    pub truncated: bool,
}

/// ToolExecutor handles tool execution
pub struct ToolExecutor<S> {
    services: Arc<S>,
}

impl<S> ToolExecutor<S>
where
    S: AsRef<crate::FsService>
        + AsRef<crate::ShellService>
        + AsRef<crate::SearchService>
        + AsRef<crate::FetchService>,
{
    /// Create a new ToolExecutor
    pub fn new(services: Arc<S>) -> Self {
        Self { services }
    }

    /// Execute a tool input and return the result
    pub async fn execute(&self, input: ToolInput) -> anyhow::Result<ToolOutput> {
        let start = std::time::Instant::now();
        
        let result = match input {
            ToolInput::Read(input) => self.execute_read(input).await,
            ToolInput::Write(input) => self.execute_write(input).await,
            ToolInput::Patch(input) => self.execute_patch(input).await,
            ToolInput::Search(input) => self.execute_search(input).await,
            ToolInput::Shell(input) => self.execute_shell(input).await,
            ToolInput::Fetch(input) => self.execute_fetch(input).await,
            ToolInput::Remove(input) => self.execute_remove(input).await,
            ToolInput::Undo(input) => self.execute_undo(input).await,
            ToolInput::MultiPatch(input) => self.execute_multi_patch(input).await,
            ToolInput::TodoWrite(input) => self.execute_todo_write(input).await,
            ToolInput::TodoRead(_input) => self.execute_todo_read().await,
        };

        let elapsed = start.elapsed().as_millis() as u64;
        
        match result {
            Ok(output) => Ok(ToolOutput {
                tool: output.0,
                success: true,
                content: Some(output.1),
                error: None,
                metrics: ToolMetrics {
                    lines_affected: 0,
                    files_accessed: vec![],
                    execution_time_ms: elapsed,
                    truncated: false,
                },
            }),
            Err(e) => Ok(ToolOutput {
                tool: "unknown".to_string(),
                success: false,
                content: None,
                error: Some(e.to_string()),
                metrics: ToolMetrics {
                    lines_affected: 0,
                    files_accessed: vec![],
                    execution_time_ms: elapsed,
                    truncated: false,
                },
            }),
        }
    }

    async fn execute_read(&self, input: FsReadInput) -> anyhow::Result<(String, String)> {
        let fs = self.services.as_ref().as_ref();
        let content = fs.read(&input.file_path, input.start_line, input.end_line).await?;
        Ok(("read".to_string(), content))
    }

    async fn execute_write(&self, input: FsWriteInput) -> anyhow::Result<(String, String)> {
        let fs = self.services.as_ref().as_ref();
        fs.write(&input.file_path, &input.content, input.overwrite).await?;
        Ok(("write".to_string(), format!("Written to {}", input.file_path)))
    }

    async fn execute_patch(&self, input: FsPatchInput) -> anyhow::Result<(String, String)> {
        let fs = self.services.as_ref().as_ref();
        fs.patch(
            &input.file_path,
            &input.old_string,
            &input.new_string,
            input.replace_all,
        )
        .await?;
        Ok(("patch".to_string(), format!("Patched {}", input.file_path)))
    }

    async fn execute_search(&self, input: FsSearchInput) -> anyhow::Result<(String, String)> {
        let search = self.services.as_ref().as_ref();
        let results = search.search(input).await?;
        Ok(("search".to_string(), results))
    }

    async fn execute_shell(&self, input: ShellInput) -> anyhow::Result<(String, String)> {
        let shell = self.services.as_ref().as_ref();
        let output = shell.execute(
            &input.command,
            input.cwd.map(|p| PathBuf::from(p)),
            input.keep_ansi,
            input.description,
        ).await?;
        Ok(("shell".to_string(), output))
    }

    async fn execute_fetch(&self, input: NetFetchInput) -> anyhow::Result<(String, String)> {
        let fetch = self.services.as_ref().as_ref();
        let output = fetch.fetch(&input.url, input.raw).await?;
        Ok(("fetch".to_string(), output))
    }

    async fn execute_remove(&self, input: RemoveInput) -> anyhow::Result<(String, String)> {
        let fs = self.services.as_ref().as_ref();
        fs.remove(&input.path).await?;
        Ok(("remove".to_string(), format!("Removed {}", input.path)))
    }

    async fn execute_undo(&self, input: UndoInput) -> anyhow::Result<(String, String)> {
        // Undo is implemented in fs service
        let fs = self.services.as_ref().as_ref();
        fs.undo(&input.path).await?;
        Ok(("undo".to_string(), format!("Undone changes to {}", input.path)))
    }

    async fn execute_multi_patch(&self, input: MultiPatchInput) -> anyhow::Result<(String, String)> {
        let fs = self.services.as_ref().as_ref();
        let edits: Vec<_> = input.edits.into_iter().map(|e| (e.old_string, e.new_string)).collect();
        fs.multi_patch(&input.file_path, edits).await?;
        Ok(("multi_patch".to_string(), format!("Multi-patched {}", input.file_path)))
    }

    async fn execute_todo_write(&self, input: TodoWriteInput) -> anyhow::Result<(String, String)> {
        // Todo operations would use a todo service
        Ok(("todo_write".to_string(), format!("Updated {} todos", input.todos.len())))
    }

    async fn execute_todo_read(&self) -> anyhow::Result<(String, String)> {
        Ok(("todo_read".to_string(), "Todo list retrieved".to_string()))
    }
}

// ============================================================================
// Services trait bounds
// ============================================================================

use crate::fs::FsService;
use crate::shell::ShellService;
use crate::search::SearchService;
use crate::fetch::FetchService;

/// Trait bounds for the services required by ToolExecutor
pub trait ToolServices: Send + Sync {
    type Fs: FsService;
    type Shell: ShellService;
    type Search: SearchService;
    type Fetch: FetchService;
    
    fn fs(&self) -> &Self::Fs;
    fn shell(&self) -> &Self::Shell;
    fn search(&self) -> &Self::Search;
    fn fetch(&self) -> &Self::Fetch;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_input_clone() {
        let input = ToolInput::Read(FsReadInput {
            file_path: "test.rs".to_string(),
            start_line: None,
            end_line: None,
        });
        let cloned = input.clone();
        match cloned {
            ToolInput::Read(_) => {},
            _ => panic!("Expected Read variant"),
        }
    }

    #[test]
    fn test_tool_output_serialization() {
        let output = ToolOutput {
            tool: "read".to_string(),
            success: true,
            content: Some("test content".to_string()),
            error: None,
            metrics: ToolMetrics {
                lines_affected: 10,
                files_accessed: vec!["test.rs".to_string()],
                execution_time_ms: 100,
                truncated: false,
            },
        };
        
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"success\":true"));
    }
}
