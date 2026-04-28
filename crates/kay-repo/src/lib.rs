//! kay-repo: Repository analysis and Git operations for Kay CLI
//!
//! This crate provides Forge-equivalent functionality to forge_repo:
//! - Git operations (status, diff, log, branch)
//! - Repository analysis (workspace detection, file types)
//! - File walking with .gitignore integration

pub mod git;
pub mod repo;

pub use git::{GitStatus, GitDiff, GitLog, GitBranch};
pub use repo::{Workspace, FileInfo, FileType, CrateInfo};

/// Placeholder for Repository (not yet implemented)
pub struct Repository;
