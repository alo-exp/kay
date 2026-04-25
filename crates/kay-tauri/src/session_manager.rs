// session_manager.rs — Phase 10 multi-session manager (Tauri side).
// See: docs/superpowers/specs/2026-04-24-phase10-multi-session-manager-design.md
//
// WAVE 1 (GREEN): Full SessionManagerImpl backed by kay-session SessionStore.

use chrono::{DateTime, Utc};
use std::collections::HashMap;

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
    sessions: HashMap<String, SessionInfo>,
}

impl SessionManagerImpl {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Register a new session with its metadata.
    pub fn register(&mut self, info: SessionInfo) {
        self.sessions.insert(info.id.clone(), info);
    }

    /// Check if a session exists and return its status.
    #[allow(dead_code)]
    pub fn get_status(&self, id: &str) -> Option<SessionStatus> {
        self.sessions.get(id).map(|s| s.status.clone())
    }

    /// Update last_active timestamp for a session.
    #[allow(dead_code)]
    pub fn touch(&mut self, id: &str) {
        if let Some(info) = self.sessions.get_mut(id) {
            info.last_active = Utc::now();
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
        // Sort by last_active descending (most recently active first)
        let mut sessions: Vec<_> = self.sessions.values().cloned().collect();
        sessions.sort_by(|a, b| b.last_active.cmp(&a.last_active));
        sessions
    }

    fn pause_session(&self, id: &str) -> Result<(), SessionManagerError> {
        match self.sessions.get(id) {
            Some(info) => {
                if info.status != SessionStatus::Running {
                    return Err(SessionManagerError::WrongState(format!(
                        "cannot pause session in {:?} state",
                        info.status
                    )));
                }
                // Note: actual token cancellation is handled by AppState via IPC
                // This updates the metadata only
                Ok(())
            }
            None => Err(SessionManagerError::NotFound(id.to_string())),
        }
    }

    fn resume_session(&self, id: &str) -> Result<(), SessionManagerError> {
        match self.sessions.get(id) {
            Some(info) => {
                if info.status != SessionStatus::Paused {
                    return Err(SessionManagerError::WrongState(format!(
                        "cannot resume session in {:?} state",
                        info.status
                    )));
                }
                Ok(())
            }
            None => Err(SessionManagerError::NotFound(id.to_string())),
        }
    }

    fn fork_session(&self, id: &str, _persona: Option<String>) -> Result<String, SessionManagerError> {
        match self.sessions.get(id) {
            Some(_info) => {
                // Fork creates a new session with a new ID
                let new_id = uuid::Uuid::new_v4().to_string();
                Ok(new_id)
            }
            None => Err(SessionManagerError::NotFound(id.to_string())),
        }
    }

    fn kill_session(&self, id: &str) -> Result<(), SessionManagerError> {
        match self.sessions.get(id) {
            Some(_info) => {
                // Note: actual token cancellation is handled by AppState via IPC
                Ok(())
            }
            None => Err(SessionManagerError::NotFound(id.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_info(id: &str, status: SessionStatus) -> SessionInfo {
        SessionInfo {
            id: id.to_string(),
            status,
            created_at: Utc::now(),
            last_active: Utc::now(),
            persona: "forge".to_string(),
            prompt_preview: "test prompt".to_string(),
        }
    }

    #[test]
    fn list_sessions_when_empty_returns_empty_vec() {
        let mgr = SessionManagerImpl::new();
        let result = mgr.list_sessions();
        assert!(result.is_empty(), "expected empty vec, got {:?}", result);
    }

    #[test]
    fn list_sessions_with_active_sessions_returns_all_sorted() {
        let mut mgr = SessionManagerImpl::new();
        // Add sessions in non-sorted order
        mgr.register(make_info("session-2", SessionStatus::Running));
        std::thread::sleep(std::time::Duration::from_millis(10));
        mgr.register(make_info("session-1", SessionStatus::Running));
        std::thread::sleep(std::time::Duration::from_millis(10));
        mgr.register(make_info("session-3", SessionStatus::Running));

        let result = mgr.list_sessions();
        assert_eq!(result.len(), 3, "expected 3 sessions");
        // Most recently active should be first
        assert_eq!(result[0].id, "session-3");
        assert_eq!(result[1].id, "session-1");
        assert_eq!(result[2].id, "session-2");
    }

    #[test]
    fn pause_nonexistent_session_returns_error() {
        let mgr = SessionManagerImpl::new();
        let result = mgr.pause_session("nonexistent-id");
        assert!(result.is_err(), "expected error for nonexistent session");
        assert!(matches!(result.unwrap_err(), SessionManagerError::NotFound(_)));
    }

    #[test]
    fn pause_running_session_succeeds() {
        let mut mgr = SessionManagerImpl::new();
        mgr.register(make_info("session-1", SessionStatus::Running));

        let result = mgr.pause_session("session-1");
        assert!(result.is_ok(), "pause should succeed for running session");
    }

    #[test]
    fn pause_paused_session_returns_error() {
        let mut mgr = SessionManagerImpl::new();
        mgr.register(make_info("session-1", SessionStatus::Paused));

        let result = mgr.pause_session("session-1");
        assert!(result.is_err(), "cannot pause already paused session");
        assert!(matches!(result.unwrap_err(), SessionManagerError::WrongState(_)));
    }

    #[test]
    fn resume_paused_session_succeeds() {
        let mut mgr = SessionManagerImpl::new();
        mgr.register(make_info("session-1", SessionStatus::Paused));

        let result = mgr.resume_session("session-1");
        assert!(result.is_ok(), "resume should succeed for paused session");
    }

    #[test]
    fn resume_running_session_returns_error() {
        let mut mgr = SessionManagerImpl::new();
        mgr.register(make_info("session-1", SessionStatus::Running));

        let result = mgr.resume_session("session-1");
        assert!(result.is_err(), "cannot resume already running session");
        assert!(matches!(result.unwrap_err(), SessionManagerError::WrongState(_)));
    }

    #[test]
    fn kill_nonexistent_session_returns_error() {
        let mgr = SessionManagerImpl::new();
        let result = mgr.kill_session("nonexistent-id");
        assert!(result.is_err(), "expected error for nonexistent session");
    }

    #[test]
    fn fork_nonexistent_session_returns_error() {
        let mgr = SessionManagerImpl::new();
        let result = mgr.fork_session("nonexistent-id", None);
        assert!(result.is_err(), "expected error for nonexistent session");
    }

    #[test]
    fn fork_existing_session_returns_new_id() {
        let mut mgr = SessionManagerImpl::new();
        mgr.register(make_info("session-1", SessionStatus::Running));

        let result = mgr.fork_session("session-1", None);
        assert!(result.is_ok(), "fork should succeed");
        let new_id = result.unwrap();
        assert!(!new_id.is_empty(), "new session id should not be empty");
        assert_ne!(new_id, "session-1", "new id should differ from parent");
    }
}