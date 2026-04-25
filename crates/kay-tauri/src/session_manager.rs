// session_manager.rs — Phase 10 multi-session manager (Tauri side).
// See: docs/superpowers/specs/2026-04-24-phase10-multi-session-manager-design.md
//
// WAVE 1 (RED): SessionManager trait + state. Stub that returns todo!().
// WAVE 1 (GREEN): Full implementation backed by kay-session SessionStore.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Information about a session for listing/display.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub persona: String,
    pub prompt_preview: String,
}

/// Session lifecycle status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionStatus {
    Running,
    Paused,
    Completed,
    Failed,
    Killed,
}

/// Error variants for session operations.
#[derive(Debug, thiserror::Error)]
pub enum SessionManagerError {
    #[error("session not found: {0}")]
    NotFound(String),
    #[error("session is in wrong state: {0}")]
    WrongState(String),
    #[error("store error: {0}")]
    StoreError(String),
}

/// Core trait for session lifecycle operations.
/// Implemented in-memory for Tauri, backed by SessionStore for persistence.
pub trait SessionManager: Send + Sync {
    fn list_sessions(&self) -> Vec<SessionInfo>;
    fn pause_session(&self, id: &str) -> Result<(), SessionManagerError>;
    fn resume_session(&self, id: &str) -> Result<(), SessionManagerError>;
    fn fork_session(&self, id: &str, persona: Option<String>) -> Result<String, SessionManagerError>;
    fn kill_session(&self, id: &str) -> Result<(), SessionManagerError>;
}

/// In-memory session manager backed by Phase 6 SessionStore.
/// Use this for Tauri — it holds live session state in a RwLock.
pub struct SessionManagerImpl {
    /// Active sessions: session_id -> metadata.
    /// For Phase 9 compatibility, we track running sessions here.
    /// Phase 10 Wave 4 will add persistence via SessionStore.
    sessions: RwLock<HashMap<String, SessionInfo>>,
}

impl SessionManagerImpl {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for SessionManagerImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager for SessionManagerImpl {
    fn list_sessions(&self) -> Vec<SessionInfo> {
        todo!("WAVE 1 (RED): list_sessions returns todo!()")
    }

    fn pause_session(&self, id: &str) -> Result<(), SessionManagerError> {
        todo!("WAVE 1 (RED): pause_session returns todo!()")
    }

    fn resume_session(&self, id: &str) -> Result<(), SessionManagerError> {
        todo!("WAVE 1 (RED): resume_session returns todo!()")
    }

    fn fork_session(&self, id: &str, persona: Option<String>) -> Result<String, SessionManagerError> {
        todo!("WAVE 1 (RED): fork_session returns todo!()")
    }

    fn kill_session(&self, id: &str) -> Result<(), SessionManagerError> {
        todo!("WAVE 1 (RED): kill_session returns todo!()")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_sessions_when_empty_returns_empty_vec() {
        let mgr = SessionManagerImpl::new();
        let result = mgr.list_sessions();
        assert!(result.is_empty(), "expected empty vec, got {:?}", result);
    }

    #[test]
    fn list_sessions_with_active_sessions_returns_all() {
        let mgr = SessionManagerImpl::new();
        // RED test: will fail because list_sessions returns todo!()
        let result = mgr.list_sessions();
        assert_eq!(result.len(), 0, "expected 0 sessions initially");
    }

    #[test]
    fn pause_nonexistent_session_returns_error() {
        let mgr = SessionManagerImpl::new();
        let result = mgr.pause_session("nonexistent-id");
        // RED test: will fail because pause_session returns todo!()
        assert!(result.is_err(), "expected error for nonexistent session");
    }
}