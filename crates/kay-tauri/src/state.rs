//! Tauri application state — tracks active sessions for cancellation.

use dashmap::DashMap;
use tokio_util::sync::CancellationToken;

use crate::session_manager::SessionManagerImpl;

/// Shared application state managed by Tauri.
///
/// `sessions` maps `session_id → CancellationToken`. `stop_session` calls
/// `token.cancel()` which propagates to the agent loop via the control channel.
/// `session_manager` tracks session metadata for Phase 10 multi-session UI.
pub struct AppState {
    /// Active session tokens for cancellation.
    pub sessions: DashMap<String, CancellationToken>,
    /// Session metadata for listing/pause/resume/fork/kill.
    pub session_manager: SessionManagerImpl,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            sessions: DashMap::new(),
            session_manager: SessionManagerImpl::new(),
        }
    }
}
