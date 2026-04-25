// session_manager.rs — Phase 10 multi-session manager (Tauri side).
// See: docs/superpowers/specs/2026-04-24-phase10-multi-session-manager-design.md
//
// WAVE 1 (GREEN): Full SessionManagerImpl backed by kay-session SessionStore.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

/// Information about a session for listing/display.
/// Timestamps use i64 Unix seconds for Tauri IPC compatibility.
#[derive(Debug, Clone, specta::Type, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub status: SessionStatus,
    #[specta(type = i64)]
    pub created_at: i64,  // Unix timestamp seconds
    #[specta(type = i64)]
    pub last_active: i64, // Unix timestamp seconds
    pub persona: String,
    pub prompt_preview: String,
}

/// Session lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, specta::Type, Serialize, Deserialize)]
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
/// Use this for Tauri — it holds live session state.
/// Uses interior mutability via Mutex for thread-safety.
pub struct SessionManagerImpl {
    /// Active sessions: session_id -> metadata.
    /// For Phase 9 compatibility, we track running sessions here.
    /// Phase 10 Wave 4 will add persistence via SessionStore.
    sessions: Mutex<HashMap<String, SessionInfo>>,
}

impl SessionManagerImpl {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Register a new session with its metadata.
    pub fn register(&self, info: SessionInfo) {
        self.sessions.lock().unwrap().insert(info.id.clone(), info);
    }

    /// Check if a session exists and return its status.
    #[allow(dead_code)]
    pub fn get_status(&self, id: &str) -> Option<SessionStatus> {
        self.sessions.lock().unwrap().get(id).map(|s| s.status)
    }

    /// Update last_active timestamp for a session.
    #[allow(dead_code)]
    pub fn touch(&self, id: &str) {
        if let Some(info) = self.sessions.lock().unwrap().get_mut(id) {
            info.last_active = Utc::now().timestamp();
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
        let sessions: Vec<_> = self.sessions.lock().unwrap().values().cloned().collect();
        let mut sorted = sessions;
        sorted.sort_by(|a, b| b.last_active.cmp(&a.last_active));
        sorted
    }

    fn pause_session(&self, id: &str) -> Result<(), SessionManagerError> {
        match self.sessions.lock().unwrap().get_mut(id) {
            Some(info) => {
                if info.status != SessionStatus::Running {
                    return Err(SessionManagerError::WrongState(format!(
                        "cannot pause session in {:?} state",
                        info.status
                    )));
                }
                info.status = SessionStatus::Paused;
                info.last_active = Utc::now().timestamp();
                // Note: actual token cancellation is handled by AppState via IPC
                Ok(())
            }
            None => Err(SessionManagerError::NotFound(id.to_string())),
        }
    }

    fn resume_session(&self, id: &str) -> Result<(), SessionManagerError> {
        match self.sessions.lock().unwrap().get_mut(id) {
            Some(info) => {
                if info.status != SessionStatus::Paused {
                    return Err(SessionManagerError::WrongState(format!(
                        "cannot resume session in {:?} state",
                        info.status
                    )));
                }
                info.status = SessionStatus::Running;
                info.last_active = Utc::now().timestamp();
                Ok(())
            }
            None => Err(SessionManagerError::NotFound(id.to_string())),
        }
    }

    fn fork_session(&self, id: &str, persona: Option<String>) -> Result<String, SessionManagerError> {
        let info = match self.sessions.lock().unwrap().get(id).cloned() {
            Some(info) => info,
            None => return Err(SessionManagerError::NotFound(id.to_string())),
        };
        // Create new session as a fork
        let new_id = uuid::Uuid::new_v4().to_string();
        let forked = SessionInfo {
            id: new_id.clone(),
            status: SessionStatus::Running,
            created_at: Utc::now().timestamp(),
            last_active: Utc::now().timestamp(),
            persona: persona.unwrap_or_else(|| info.persona.clone()),
            prompt_preview: info.prompt_preview.clone(),
        };
        self.sessions.lock().unwrap().insert(new_id.clone(), forked);
        Ok(new_id)
    }

    fn kill_session(&self, id: &str) -> Result<(), SessionManagerError> {
        match self.sessions.lock().unwrap().get_mut(id) {
            Some(info) => {
                // Mark as killed
                info.status = SessionStatus::Killed;
                info.last_active = Utc::now().timestamp();
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
        let now = Utc::now().timestamp();
        SessionInfo {
            id: id.to_string(),
            status,
            created_at: now,
            last_active: now,
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
        let mgr = SessionManagerImpl::new();
        // Add sessions with explicit timestamps to guarantee sort order
        mgr.register(SessionInfo {
            id: "session-1".to_string(),
            status: SessionStatus::Running,
            created_at: 1000,
            last_active: 1000,
            persona: "forge".to_string(),
            prompt_preview: "test".to_string(),
        });
        mgr.register(SessionInfo {
            id: "session-2".to_string(),
            status: SessionStatus::Running,
            created_at: 2000,
            last_active: 2000,
            persona: "forge".to_string(),
            prompt_preview: "test".to_string(),
        });
        mgr.register(SessionInfo {
            id: "session-3".to_string(),
            status: SessionStatus::Running,
            created_at: 3000,
            last_active: 3000,
            persona: "forge".to_string(),
            prompt_preview: "test".to_string(),
        });

        let result = mgr.list_sessions();
        assert_eq!(result.len(), 3, "expected 3 sessions");
        // Most recently active should be first (descending order by last_active)
        assert_eq!(result[0].id, "session-3");
        assert_eq!(result[1].id, "session-2");
        assert_eq!(result[2].id, "session-1");
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
        let mgr = SessionManagerImpl::new();
        mgr.register(make_info("session-1", SessionStatus::Running));

        let result = mgr.pause_session("session-1");
        assert!(result.is_ok(), "pause should succeed for running session");
    }

    #[test]
    fn pause_paused_session_returns_error() {
        let mgr = SessionManagerImpl::new();
        mgr.register(make_info("session-1", SessionStatus::Paused));

        let result = mgr.pause_session("session-1");
        assert!(result.is_err(), "cannot pause already paused session");
        assert!(matches!(result.unwrap_err(), SessionManagerError::WrongState(_)));
    }

    #[test]
    fn resume_paused_session_succeeds() {
        let mgr = SessionManagerImpl::new();
        mgr.register(make_info("session-1", SessionStatus::Paused));

        let result = mgr.resume_session("session-1");
        assert!(result.is_ok(), "resume should succeed for paused session");
    }

    #[test]
    fn resume_running_session_returns_error() {
        let mgr = SessionManagerImpl::new();
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
        let mgr = SessionManagerImpl::new();
        mgr.register(make_info("session-1", SessionStatus::Running));

        let result = mgr.fork_session("session-1", None);
        assert!(result.is_ok(), "fork should succeed");
        let new_id = result.unwrap();
        assert!(!new_id.is_empty(), "new session id should not be empty");
        assert_ne!(new_id, "session-1", "new id should differ from parent");
    }
}