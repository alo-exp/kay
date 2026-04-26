//! Task Delegation for Kay
//!
//! Allows Kay to spawn sub-tasks (agents) for parallel execution.
//! Following the Forge pattern but implemented independently (cleanroom).

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// Task result from a delegated agent
#[derive(Debug, Clone)]
pub struct TaskResult {
    /// Task identifier
    pub task_id: String,
    /// Whether the task succeeded
    pub success: bool,
    /// Output from the task
    pub output: String,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Task specification
#[derive(Debug, Clone)]
pub struct TaskSpec {
    /// Task identifier
    pub task_id: String,
    /// Prompt for the task
    pub prompt: String,
    /// Working directory (defaults to current)
    pub cwd: Option<PathBuf>,
    /// Model to use (defaults to config default)
    pub model: Option<String>,
    /// Max turns for this task
    pub max_turns: Option<u32>,
    /// Environment variables to set
    pub env: std::collections::HashMap<String, String>,
}

/// Spawn a task and wait for its completion
pub async fn spawn_task(task: TaskSpec) -> anyhow::Result<TaskResult> {
    let start = std::time::Instant::now();
    
    // Build the kay command
    let mut cmd = Command::new("kay");
    cmd.arg("run").arg("--prompt").arg(&task.prompt);
    
    if let Some(ref cwd) = task.cwd {
        cmd.current_dir(cwd);
    }
    
    if let Some(ref model) = task.model {
        cmd.arg("--model").arg(model);
    }
    
    if let Some(max_turns) = task.max_turns {
        cmd.arg("--max-turns").arg(max_turns.to_string());
    }
    
    // Set environment variables
    for (key, value) in &task.env {
        cmd.env(key, value);
    }
    
    // Capture stdout and stderr
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.stdin(Stdio::null());
    let mut child = cmd.spawn()?;
    
    // Read stdout
    let stdout = child.stdout.take();
    let mut output = String::new();
    
    if let Some(stdout) = stdout {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            output.push_str(&line);
            output.push('\n');
        }
    }
    
    // Wait for child to complete
    let status = child.wait().await?;
    
    let elapsed = start.elapsed().as_millis() as u64;
    
    Ok(TaskResult {
        task_id: task.task_id,
        success: status.success(),
        output,
        error: if status.success() { None } else { Some(format!("Exit code: {}", status)) },
        execution_time_ms: elapsed,
    })
}

/// Spawn multiple tasks in parallel and collect results
pub async fn spawn_tasks_parallel(tasks: Vec<TaskSpec>) -> Vec<TaskResult> {
    use tokio::task::JoinSet;
    
    let mut join_set = JoinSet::new();
    
    for task in tasks {
        join_set.spawn(async move { spawn_task(task).await });
    }
    
    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(task_result)) => results.push(task_result),
            Ok(Err(e)) => {
                results.push(TaskResult {
                    task_id: "unknown".to_string(),
                    success: false,
                    output: String::new(),
                    error: Some(format!("Task failed: {}", e)),
                    execution_time_ms: 0,
                });
            }
            Err(e) => {
                results.push(TaskResult {
                    task_id: "unknown".to_string(),
                    success: false,
                    output: String::new(),
                    error: Some(format!("Task panicked: {}", e)),
                    execution_time_ms: 0,
                });
            }
        }
    }
    
    results
}

/// Task manager for tracking delegated tasks
pub struct TaskManager {
    tasks: Arc<std::sync::RwLock<std::collections::HashMap<String, TaskResult>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
    
    /// Submit a task and get its ID
    pub fn submit(&self, task: TaskSpec) -> String {
        let task_id = task.task_id.clone();
        let task_id_for_insert = task_id.clone();
        let tasks = self.tasks.clone();
        
        // Spawn the task in background
        tokio::spawn(async move {
            let result = spawn_task(task).await;
            if let Ok(result) = result {
                if let Ok(mut tasks) = tasks.write() {
                    tasks.insert(task_id_for_insert, result);
                }
            }
        });
        
        task_id
    }
    
    /// Get a task result by ID
    pub fn get_result(&self, task_id: &str) -> Option<TaskResult> {
        self.tasks.read().ok().and_then(|tasks| tasks.get(task_id).cloned())
    }
    
    /// List all task IDs
    pub fn list_tasks(&self) -> Vec<String> {
        self.tasks.read().ok()
            .map(|tasks| tasks.keys().cloned().collect())
            .unwrap_or_default()
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_spec_clone() {
        let spec = TaskSpec {
            task_id: "test-1".to_string(),
            prompt: "Hello".to_string(),
            cwd: None,
            model: None,
            max_turns: None,
            env: std::collections::HashMap::new(),
        };
        let cloned = spec.clone();
        assert_eq!(cloned.task_id, "test-1");
    }
}
