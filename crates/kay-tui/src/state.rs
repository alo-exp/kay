// state.rs — Full TUI state machine (AppState).
// See: docs/superpowers/specs/2026-04-24-phase9.5-tui-frontend-design.md §6
//
// WAVE 4: AppState enum + SessionState struct + EventLog circular buffer.
// - Phase 9.5 does NOT persist state (Phase 6 handles session store)
// - EventLog: circular buffer capped at 10_000 events (PERF-01)

use std::collections::VecDeque;
use std::time::Instant;

use crate::events::TuiEvent;

/// Maximum events held in the EventLog (spec §6 PERF-01).
const MAX_EVENT_LOG: usize = 10_000;

/// Total cost accumulated across all Usage events.
#[derive(Debug, Clone, Default)]
pub struct CostAccumulator {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cost_usd: f64,
}

/// State of the current agent session.
#[derive(Debug, Clone)]
pub struct SessionState {
    pub started_at: Instant,
    /// All received events (circular buffer).
    pub event_log: EventLog,
    /// Accumulated token/cost metrics.
    pub cost: CostAccumulator,
    /// Currently active tool call (if any).
    pub active_tool: Option<ActiveTool>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
            event_log: EventLog::new(),
            cost: CostAccumulator::default(),
            active_tool: None,
        }
    }

    /// Process a received TuiEvent, updating session state.
    pub fn push_event(&mut self, event: &TuiEvent) {
        self.event_log.push(event.clone());

        match event {
            TuiEvent::Usage { prompt_tokens, completion_tokens, cost_usd } => {
                self.cost.prompt_tokens += prompt_tokens;
                self.cost.completion_tokens += completion_tokens;
                self.cost.cost_usd += cost_usd;
            }
            TuiEvent::ToolCallStart { id, name } => {
                self.active_tool = Some(ActiveTool {
                    id: id.clone(),
                    name: name.clone(),
                    started_at: Instant::now(),
                });
            }
            TuiEvent::ToolCallComplete { .. } => {
                self.active_tool = None;
            }
            TuiEvent::ToolCallMalformed { id, .. } => {
                if self.active_tool.as_ref().map(|t| &t.id) == Some(id) {
                    self.active_tool = None;
                }
            }
            _ => {}
        }
    }
}

/// Currently running tool call (for timeline display).
#[derive(Debug, Clone)]
pub struct ActiveTool {
    pub id: String,
    pub name: String,
    pub started_at: Instant,
}

/// Circular event log. Older events are dropped when capacity is reached.
#[derive(Debug, Clone)]
pub struct EventLog {
    /// Newest events at the back.
    events: VecDeque<TuiEvent>,
    max_events: usize,
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new()
    }
}

impl EventLog {
    pub fn new() -> Self {
        Self {
            events: VecDeque::with_capacity(MAX_EVENT_LOG),
            max_events: MAX_EVENT_LOG,
        }
    }

    /// Add an event, dropping the oldest if at capacity.
    pub fn push(&mut self, event: TuiEvent) {
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Return a reference to all events (newest last).
    pub fn events(&self) -> &VecDeque<TuiEvent> {
        &self.events
    }

    /// Number of events currently stored.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// True if the log is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Phase 9.5 does NOT persist state (Phase 6 handles session store).
#[derive(Debug, Default)]
pub struct AppState;

impl AppState {
    pub fn new() -> Self {
        Self
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn text_event(content: &str) -> TuiEvent {
        TuiEvent::TextDelta { content: content.to_string() }
    }

    fn usage_event(pt: u64, ct: u64, cost: f64) -> TuiEvent {
        TuiEvent::Usage { prompt_tokens: pt, completion_tokens: ct, cost_usd: cost }
    }

    fn tool_start(id: &str, name: &str) -> TuiEvent {
        TuiEvent::ToolCallStart { id: id.to_string(), name: name.to_string() }
    }

    fn tool_complete(id: &str) -> TuiEvent {
        TuiEvent::ToolCallComplete {
            id: id.to_string(),
            name: String::new(),
            arguments: serde_json::Value::Null,
        }
    }

    #[test]
    fn event_log_capacity() {
        let mut log = EventLog::new();
        for i in 0..12_000 {
            log.push(text_event(&format!("event {i}")));
        }
        assert_eq!(log.len(), 10_000);
        // Oldest events should be dropped
        let first = log.events().front().unwrap();
        assert!(matches!(first, TuiEvent::TextDelta { content } if content.contains("2000")));
    }

    #[test]
    fn session_accumulates_cost() {
        let mut session = SessionState::new();
        session.push_event(&usage_event(100, 50, 0.001));
        session.push_event(&usage_event(200, 100, 0.002));
        assert_eq!(session.cost.prompt_tokens, 300);
        assert_eq!(session.cost.completion_tokens, 150);
        assert!((session.cost.cost_usd - 0.003).abs() < 1e-9);
    }

    #[test]
    fn session_tracks_active_tool() {
        let mut session = SessionState::new();
        assert!(session.active_tool.is_none());
        session.push_event(&tool_start("t1", "edit_file"));
        assert!(session.active_tool.is_some());
        assert_eq!(session.active_tool.as_ref().unwrap().name, "edit_file");
        session.push_event(&tool_complete("t1"));
        assert!(session.active_tool.is_none());
    }

    #[test]
    fn event_log_clone_is_independent() {
        let mut log = EventLog::new();
        log.push(text_event("hello"));
        let cloned = log.clone();
        drop(log);
        assert_eq!(cloned.len(), 1);
    }
}
