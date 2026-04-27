//! Git operations - equivalent to forge_repo/src/git.rs

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::process::Command;

/// Git status of a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub modified: Vec<String>,
    pub untracked: Vec<String>,
    pub deleted: Vec<String>,
    pub staged: Vec<String>,
    pub ahead: u32,
    pub behind: u32,
}

impl GitStatus {
    pub async fn get<P: AsRef<Path>>(cwd: P) -> Result<Self> {
        let cwd = cwd.as_ref();
        let branch = Self::get_branch(cwd).await.unwrap_or_else(|_| "HEAD".to_string());

        let output = Command::new("git")
            .args(["status", "--porcelain", "-b"])
            .current_dir(cwd)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("git status failed"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut modified = vec![];
        let mut untracked = vec![];
        let mut deleted = vec![];
        let mut staged = vec![];
        let mut ahead = 0u32;
        let mut behind = 0u32;

        for line in stdout.lines() {
            if line.starts_with("## ") {
                if let Some(idx) = line.find("ahead ") {
                    let rest = &line[idx + 6..];
                    if let Some(comma) = rest.find(',') {
                        ahead = rest[..comma].trim().parse().unwrap_or(0);
                    }
                }
                if let Some(idx) = line.find("behind ") {
                    let rest = &line[idx + 7..];
                    behind = rest.trim().split_whitespace().next()
                        .and_then(|s| s.trim_end_matches(']').parse().ok()).unwrap_or(0);
                }
                continue;
            }
            if line.len() < 3 { continue; }
            let s = &line[0..1];
            let w = &line[1..2];
            let path = &line[3..];
            match s { "M" | "A" | "D" | "R" => staged.push(path.to_string()), _ => {} }
            match w { "M" => modified.push(path.to_string()), "D" => deleted.push(path.to_string()), "?" => untracked.push(path.to_string()), _ => {} }
        }

        Ok(Self { branch, modified, untracked, deleted, staged, ahead, behind })
    }

    pub async fn get_branch<P: AsRef<Path>>(cwd: P) -> Result<String> {
        let output = Command::new("git").args(["branch", "--show-current"]).current_dir(cwd.as_ref()).output().await?;
        if !output.status.success() { return Err(anyhow!("git branch failed")); }
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(if branch.is_empty() { "HEAD".to_string() } else { branch })
    }
}

/// Git diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiff {
    pub files: Vec<DiffFile>,
    pub unified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffFile {
    pub path: String,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffLineType { Context, Added, Removed }

impl GitDiff {
    pub async fn get<P: AsRef<Path>>(cwd: P, file: Option<&str>) -> Result<Self> {
        let cwd = cwd.as_ref();
        let mut args = vec!["diff", "--unified=3"];
        if let Some(f) = file { args.push(f); }
        let output = Command::new("git").args(&args).current_dir(cwd).output().await?;
        let unified = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(Self { files: vec![], unified })
    }
}

/// Git log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLog {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub email: String,
    pub date: String,
    pub subject: String,
}

impl GitLog {
    pub async fn get<P: AsRef<Path>>(cwd: P, count: usize) -> Result<Vec<Self>> {
        let output = Command::new("git")
            .args(["log", &format!("-{}", count), "--format=%H%n%h%n%an%n%ae%n%aI%n%s"])
            .current_dir(cwd.as_ref())
            .output()
            .await?;
        if !output.status.success() { return Err(anyhow!("git log failed")); }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut logs = vec![];
        for entry in stdout.trim().split("\n\n") {
            let parts: Vec<&str> = entry.split('\n').collect();
            if parts.len() >= 7 {
                logs.push(GitLog {
                    hash: parts[0].to_string(),
                    short_hash: parts[1].to_string(),
                    author: parts[2].to_string(),
                    email: parts[3].to_string(),
                    date: parts[5].to_string(),
                    subject: parts[6].to_string(),
                });
            }
        }
        Ok(logs)
    }
}

/// Git branch info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBranch {
    pub name: String,
    pub current: bool,
    pub upstream: Option<String>,
}

impl GitBranch {
    pub async fn current<P: AsRef<Path>>(cwd: P) -> Result<Self> {
        let name = GitStatus::get_branch(&cwd).await?;
        Ok(Self { name: name.clone(), current: true, upstream: None })
    }
}
