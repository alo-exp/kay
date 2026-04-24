//! Tauri application state — tracks active sessions for cancellation.

use dashmap::DashMap;
use tokio_util::sync::CancellationToken;

/// Shared application state managed by Tauri.
///
/// `sessions` maps `session_id → CancellationToken`. `stop_session` calls
/// `token.cancel()` which propagates to the agent loop via the control channel.
pub struct AppState {
    pub sessions: DashMap<String, CancellationToken>,
}

impl Default for AppState {
    fn default() -> Self {
        Self { sessions: DashMap::new() }
    }
}
