// session_manager.rs — Phase 10 TUI session control (kay-tui).
// See: docs/superpowers/specs/2026-04-24-phase10-multi-session-manager-design.md
//
// WAVE 6 (GREEN): TuiSessionManager for keyboard-driven session control.
// Keyboard shortcuts: n/p/r/f/x/s/?.

use serde::{Deserialize, Serialize};

/// Represents the current state of a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Session is running.
    Running,
    /// Session is paused.
    Paused,
    /// Session has completed.
    Completed,
    /// Session was killed.
    Killed,
}

/// Information about a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Unique session ID.
    pub id: String,
    /// Current state of the session.
    pub state: SessionState,
    /// Optional persona/name for the session.
    pub persona: Option<String>,
}

/// Keyboard action for session control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionAction {
    /// Spawn a new session.
    Spawn,
    /// Pause the current session.
    Pause,
    /// Resume a paused session.
    Resume,
    /// Fork (clone) the current session.
    Fork,
    /// Kill the current session.
    Kill,
    /// Toggle settings panel.
    ToggleSettings,
    /// Show help overlay.
    ShowHelp,
}

/// Maps keyboard input to session actions.
#[derive(Debug, Default)]
pub struct KeyboardMapper;

impl KeyboardMapper {
    /// Creates a new KeyboardMapper.
    pub fn new() -> Self {
        Self
    }

    /// Maps a key event to a session action.
    ///
    /// Returns `Some(action)` if the key maps to a session action,
    /// or `None` if the key is not handled by this mapper.
    ///
    /// Keyboard shortcuts:
    /// - `n`: Spawn new session
    /// - `p`: Pause current session
    /// - `r`: Resume paused session
    /// - `f`: Fork current session
    /// - `x`: Kill current session
    /// - `s`: Toggle settings panel
    /// - `?`: Show help overlay
    pub fn map_key(&self, code: crossterm::event::KeyCode) -> Option<SessionAction> {
        use crossterm::event::KeyCode;

        match code {
            KeyCode::Char('n') => Some(SessionAction::Spawn),
            KeyCode::Char('p') => Some(SessionAction::Pause),
            KeyCode::Char('r') => Some(SessionAction::Resume),
            KeyCode::Char('f') => Some(SessionAction::Fork),
            KeyCode::Char('x') => Some(SessionAction::Kill),
            KeyCode::Char('s') => Some(SessionAction::ToggleSettings),
            KeyCode::Char('?') => Some(SessionAction::ShowHelp),
            _ => None,
        }
    }

    /// Returns the keyboard shortcut for an action, for display in help.
    pub fn shortcut_for(action: SessionAction) -> char {
        match action {
            SessionAction::Spawn => 'n',
            SessionAction::Pause => 'p',
            SessionAction::Resume => 'r',
            SessionAction::Fork => 'f',
            SessionAction::Kill => 'x',
            SessionAction::ToggleSettings => 's',
            SessionAction::ShowHelp => '?',
        }
    }

    /// Returns the description for an action.
    pub fn description_for(action: SessionAction) -> &'static str {
        match action {
            SessionAction::Spawn => "Spawn new session",
            SessionAction::Pause => "Pause current session",
            SessionAction::Resume => "Resume paused session",
            SessionAction::Fork => "Fork (clone) current session",
            SessionAction::Kill => "Kill current session",
            SessionAction::ToggleSettings => "Toggle settings panel",
            SessionAction::ShowHelp => "Show keyboard shortcuts",
        }
    }
}

/// TuiSessionManager manages session state for the TUI.
#[derive(Debug, Default)]
pub struct TuiSessionManager {
    /// Current active session, if any.
    current_session: Option<SessionInfo>,
    /// All known sessions.
    sessions: Vec<SessionInfo>,
}

impl TuiSessionManager {
    /// Creates a new TuiSessionManager.
    pub fn new() -> Self {
        Self { current_session: None, sessions: Vec::new() }
    }

    /// Returns the current session, if any.
    pub fn current_session(&self) -> Option<&SessionInfo> {
        self.current_session.as_ref()
    }

    /// Returns all sessions.
    pub fn sessions(&self) -> &[SessionInfo] {
        &self.sessions
    }

    /// Spawns a new session.
    pub fn spawn(&mut self, persona: Option<String>) -> SessionInfo {
        let id = uuid::Uuid::new_v4().to_string();
        let info = SessionInfo { id: id.clone(), state: SessionState::Running, persona };
        self.sessions.push(info.clone());
        self.current_session = Some(info.clone());
        info
    }

    /// Pauses the current session.
    pub fn pause(&mut self) -> Option<SessionInfo> {
        if let Some(ref mut session) = self.current_session {
            if session.state == SessionState::Running {
                session.state = SessionState::Paused;
                return Some(session.clone());
            }
        }
        None
    }

    /// Resumes a paused session.
    pub fn resume(&mut self) -> Option<SessionInfo> {
        if let Some(ref mut session) = self.current_session {
            if session.state == SessionState::Paused {
                session.state = SessionState::Running;
                return Some(session.clone());
            }
        }
        None
    }

    /// Forks the current session.
    pub fn fork(&mut self) -> Option<SessionInfo> {
        if let Some(ref session) = self.current_session {
            let id = uuid::Uuid::new_v4().to_string();
            let forked = SessionInfo {
                id,
                state: SessionState::Running,
                persona: session.persona.clone(),
            };
            self.sessions.push(forked.clone());
            self.current_session = Some(forked.clone());
            return Some(forked);
        }
        None
    }

    /// Kills the current session.
    pub fn kill(&mut self) -> Option<SessionInfo> {
        if let Some(ref mut session) = self.current_session {
            session.state = SessionState::Killed;
            let killed = session.clone();
            self.current_session = None;
            return Some(killed);
        }
        None
    }

    /// Returns whether a session action is currently valid.
    pub fn can_action(&self, action: SessionAction) -> bool {
        match action {
            SessionAction::Spawn => true,
            SessionAction::Pause => self
                .current_session
                .as_ref()
                .map(|s| s.state == SessionState::Running)
                .unwrap_or(false),
            SessionAction::Resume => self
                .current_session
                .as_ref()
                .map(|s| s.state == SessionState::Paused)
                .unwrap_or(false),
            SessionAction::Fork | SessionAction::Kill => self
                .current_session
                .as_ref()
                .is_some_and(|s| s.state != SessionState::Killed),
            SessionAction::ToggleSettings | SessionAction::ShowHelp => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_mapper_spawn_shortcut() {
        use crossterm::event::KeyCode;
        let mapper = KeyboardMapper::new();
        assert_eq!(
            mapper.map_key(KeyCode::Char('n')),
            Some(SessionAction::Spawn)
        );
    }

    #[test]
    fn keyboard_mapper_all_shortcuts() {
        use crossterm::event::KeyCode;
        let mapper = KeyboardMapper::new();

        assert_eq!(
            mapper.map_key(KeyCode::Char('n')),
            Some(SessionAction::Spawn)
        );
        assert_eq!(
            mapper.map_key(KeyCode::Char('p')),
            Some(SessionAction::Pause)
        );
        assert_eq!(
            mapper.map_key(KeyCode::Char('r')),
            Some(SessionAction::Resume)
        );
        assert_eq!(
            mapper.map_key(KeyCode::Char('f')),
            Some(SessionAction::Fork)
        );
        assert_eq!(
            mapper.map_key(KeyCode::Char('x')),
            Some(SessionAction::Kill)
        );
        assert_eq!(
            mapper.map_key(KeyCode::Char('s')),
            Some(SessionAction::ToggleSettings)
        );
        assert_eq!(
            mapper.map_key(KeyCode::Char('?')),
            Some(SessionAction::ShowHelp)
        );

        // Unhandled keys
        assert_eq!(mapper.map_key(KeyCode::Char('q')), None);
        assert_eq!(mapper.map_key(KeyCode::Esc), None);
    }

    #[test]
    fn session_manager_spawn() {
        let mut mgr = TuiSessionManager::new();
        let session = mgr.spawn(Some("coder".to_string()));

        assert!(mgr.current_session().is_some());
        assert_eq!(session.state, SessionState::Running);
        assert_eq!(session.persona, Some("coder".to_string()));
        assert_eq!(mgr.sessions().len(), 1);
    }

    #[test]
    fn session_manager_pause_resume() {
        let mut mgr = TuiSessionManager::new();
        mgr.spawn(None);

        // Pause
        assert!(mgr.pause().is_some());
        assert_eq!(mgr.current_session().unwrap().state, SessionState::Paused);

        // Can't pause again
        assert!(mgr.pause().is_none());

        // Resume
        assert!(mgr.resume().is_some());
        assert_eq!(mgr.current_session().unwrap().state, SessionState::Running);
    }

    #[test]
    fn session_manager_fork() {
        let mut mgr = TuiSessionManager::new();
        let original = mgr.spawn(Some("original".to_string()));
        let forked = mgr.fork().unwrap();

        assert_ne!(original.id, forked.id);
        assert_eq!(forked.persona, original.persona);
        assert_eq!(mgr.sessions().len(), 2);
    }

    #[test]
    fn session_manager_kill() {
        let mut mgr = TuiSessionManager::new();
        mgr.spawn(None);

        let killed = mgr.kill().unwrap();
        assert_eq!(killed.state, SessionState::Killed);

        // Can't fork/kill again
        assert!(mgr.fork().is_none());
        assert!(mgr.kill().is_none());
    }

    #[test]
    fn can_action() {
        let mut mgr = TuiSessionManager::new();

        // No session yet
        assert!(mgr.can_action(SessionAction::Spawn));
        assert!(!mgr.can_action(SessionAction::Pause));
        assert!(!mgr.can_action(SessionAction::Resume));

        // Spawn session
        mgr.spawn(None);
        assert!(mgr.can_action(SessionAction::Pause));
        assert!(!mgr.can_action(SessionAction::Resume));

        // Pause
        mgr.pause();
        assert!(mgr.can_action(SessionAction::Resume));
        assert!(!mgr.can_action(SessionAction::Pause));
    }
}
