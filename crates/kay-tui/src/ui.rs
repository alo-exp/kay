// ui.rs — ratatui component library.
// See: docs/superpowers/specs/2026-04-24-phase9.5-tui-frontend-design.md §7
//
// WAVE 5: ratatui components (App struct + render fns).
// WAVE 6: App event loop with layout + input handling.
//
// Layout: [Header] [Event Log (scrollable)] [Tool Timeline] [Input]

use std::time::Duration;
use std::time::Instant;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::events::TuiEvent;
use crate::state::SessionState;

/// ratatui component state.
#[derive(Debug)]
pub struct App {
    /// Current session state.
    session: SessionState,
    /// Index into session.event_log.events for the selected item.
    selected_index: usize,
    /// Scroll offset in the event log list.
    scroll_offset: usize,
    /// For list navigation state.
    list_state: ListState,
    /// Elapsed time display.
    started_at: Instant,
    /// Whether the app is running.
    running: bool,
    /// Subprocess handle (if spawned).
    subprocess: Option<crate::subprocess::KaySubprocess>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            session: SessionState::new(),
            selected_index: 0,
            scroll_offset: 0,
            list_state: ListState::default(),
            started_at: Instant::now(),
            running: true,
            subprocess: None,
        }
    }

    /// Process a received TuiEvent, updating state.
    pub fn handle_event(&mut self, event: TuiEvent) {
        self.session.push_event(&event);

        // Update selection when new events arrive
        let n = self.session.event_log.len();
        if self.selected_index >= n {
            self.selected_index = n.saturating_sub(1);
        }
    }

    /// Handle terminal key input.
    pub fn handle_input(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let n = self.session.event_log.len();

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    if self.selected_index < self.scroll_offset {
                        self.scroll_offset = self.selected_index;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < n.saturating_sub(1) {
                    self.selected_index += 1;
                    let vis = self.scroll_offset + 20; // approx visible height
                    if self.selected_index >= vis {
                        self.scroll_offset = self.selected_index.saturating_sub(19);
                    }
                }
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.running = false;
            }
            _ => {}
        }
    }

    /// True if the app is still running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Stop the app loop.
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Draw the TUI.
    pub fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();

        if area.width < 80 || area.height < 10 {
            // Too small to render — skip
            return;
        }

        // Main layout: [Header(3)] [Event Log (*)] [Input(3)]
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(1),    // Event log (scrollable)
                Constraint::Length(3), // Status/input
            ])
            .split(area);

        self.render_header(frame, chunks[0]);
        self.render_event_log(frame, chunks[1]);
        self.render_footer(frame, chunks[2]);
    }

    fn render_header(&self, frame: &mut Frame<'_>, area: Rect) {
        let elapsed = self.started_at.elapsed();
        let elapsed_str = format!(
            "{:02}:{:02}:{:02}",
            elapsed.as_secs() / 3600,
            (elapsed.as_secs() % 3600) / 60,
            elapsed.as_secs() % 60
        );

        let cost = &self.session.cost;
        let cost_str = format!(
            "${:.4} | p:{}/c:{}",
            cost.cost_usd, cost.prompt_tokens, cost.completion_tokens
        );

        let tool_str = self
            .session
            .active_tool
            .as_ref()
            .map(|t| format!("⏳ {}", t.name))
            .unwrap_or_else(|| "🟢 idle".to_string());

        let line = Line::from(vec![
            Span::raw(" kay-tui "),
            Span::styled("[", Style::new().fg(Color::DarkGray)),
            Span::raw(elapsed_str),
            Span::styled("] ", Style::new().fg(Color::DarkGray)),
            Span::styled(&cost_str, Style::new().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::styled(&tool_str, Style::new().fg(Color::Cyan)),
            Span::raw(" | events: "),
            Span::raw(self.session.event_log.len().to_string()),
            Span::styled(" | ↑↓ navigate | q quit", Style::new().fg(Color::DarkGray)),
        ]);

        let p = Paragraph::new(line)
            .style(Style::new().bg(Color::Rgb(30, 30, 35)))
            .block(Block::default().title(" Kay ").borders(Borders::BOTTOM))
            .wrap(Wrap { trim: false });

        frame.render_widget(p, area);
    }

    fn render_event_log(&self, frame: &mut Frame<'_>, area: Rect) {
        let items: Vec<ListItem> = self
            .session
            .event_log
            .events()
            .iter()
            .enumerate()
            .map(|(i, event)| {
                let (icon, text, style) = event_summary(event, i == self.selected_index);
                ListItem::new(Line::from(vec![
                    Span::styled(icon, style),
                    Span::raw(" "),
                    Span::raw(text),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title(" Events ").borders(Borders::ALL))
            .style(Style::new())
            .highlight_style(
                Style::new()
                    .bg(Color::Rgb(50, 50, 60))
                    .add_modifier(Modifier::BOLD),
            );

        let mut state = self.list_state.clone();
        state.select(Some(self.selected_index));
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_footer(&self, frame: &mut Frame<'_>, area: Rect) {
        let status = if self.session.active_tool.is_some() {
            Line::from(Span::styled(
                " Press Ctrl+C to interrupt | q to quit ",
                Style::new().fg(Color::Yellow),
            ))
        } else {
            Line::from(Span::styled(
                " Ready — q quits ",
                Style::new().fg(Color::Green),
            ))
        };

        let p = Paragraph::new(status)
            .style(Style::new().bg(Color::Rgb(20, 20, 25)))
            .block(Block::default().borders(Borders::TOP))
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(p, area);
    }
}

/// Returns (icon, summary_text, style) for an event.
fn event_summary(event: &TuiEvent, _selected: bool) -> (&'static str, String, Style) {
    use crate::events::TuiEvent::*;
    match event {
        TextDelta { content } => {
            let preview = if content.len() > 60 {
                format!("{}...", &content[..60])
            } else {
                content.clone()
            };
            ("✦", preview, Style::new().fg(Color::White))
        }
        ToolCallStart { name, .. } => ("⚙", format!("→ {name}"), Style::new().fg(Color::Blue)),
        ToolCallDelta { id, arguments_delta } => {
            let preview = if arguments_delta.len() > 40 {
                format!("{}...", &arguments_delta[..40])
            } else {
                arguments_delta.clone()
            };
            (
                "…",
                format!("[{id}] {preview}"),
                Style::new().fg(Color::DarkGray),
            )
        }
        ToolCallComplete { id, name, .. } => (
            "✓",
            format!("[{id}] {name} done"),
            Style::new().fg(Color::Green),
        ),
        ToolCallMalformed { id, error, .. } => (
            "✗",
            format!("[{id}] malformed: {error}"),
            Style::new().fg(Color::Red),
        ),
        Usage { prompt_tokens, completion_tokens, cost_usd } => (
            "$",
            format!(
                "tokens p:{}/c:{} cost:${:.4}",
                prompt_tokens, completion_tokens, cost_usd
            ),
            Style::new().fg(Color::Yellow),
        ),
        Retry { attempt, delay_ms, reason } => (
            "↺",
            format!("attempt {attempt} in {delay_ms}ms: {reason}"),
            Style::new().fg(Color::Magenta),
        ),
        Error { message } => (
            "!",
            format!("error: {message}"),
            Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        ToolOutput { call_id, chunk } => {
            use crate::events::TuiToolOutputChunk::*;
            let preview = match chunk {
                Stdout(s) if s.len() > 40 => format!("stdout: {}...", &s[..40]),
                Stderr(s) if s.len() > 40 => format!("stderr: {}...", &s[..40]),
                Stdout(s) => format!("stdout: {s}"),
                Stderr(s) => format!("stderr: {s}"),
                Closed { exit_code, marker_detected } => {
                    format!("closed code:{exit_code:?} marker:{marker_detected}")
                }
            };
            (
                "○",
                format!("[{call_id}] {preview}"),
                Style::new().fg(Color::Cyan),
            )
        }
        TaskComplete { call_id, verified, outcome } => {
            use crate::events::TuiVerificationOutcome::*;
            let (icon, note) = match outcome {
                Pending { reason } => ("?", format!("pending: {reason}")),
                Pass { note } => ("✓", format!("verified: {note}")),
                Fail { reason } => ("✗", format!("failed: {reason}")),
            };
            let style = if *verified { Color::Green } else { Color::Red };
            (icon, format!("[{call_id}] {note}"), Style::new().fg(style))
        }
        ImageRead { path, .. } => ("🖼", format!("read: {path}"), Style::new().fg(Color::Blue)),
        SandboxViolation { tool_name, resource, policy_rule, .. } => (
            "🚫",
            format!("sandbox violation: {tool_name} on {resource} ({policy_rule})"),
            Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Paused => ("⏸", "session paused".into(), Style::new().fg(Color::Yellow)),
        Aborted { reason } => (
            "■",
            format!("aborted: {reason}"),
            Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        ContextTruncated { dropped_symbols, budget_tokens } => (
            "↕",
            format!("truncated {dropped_symbols} symbols (budget: {budget_tokens})"),
            Style::new().fg(Color::DarkGray),
        ),
        IndexProgress { indexed, total } => (
            "📚",
            format!("indexed {indexed}/{total}"),
            Style::new().fg(Color::Cyan),
        ),
        Verification { critic_role, verdict, cost_usd, .. } => (
            "🔍",
            format!("{critic_role}: {verdict} (${:.4})", cost_usd),
            Style::new().fg(Color::Magenta),
        ),
        VerifierDisabled { reason, cost_usd } => (
            "⊘",
            format!("verifier disabled: {reason} (saved ${:.4})", cost_usd),
            Style::new().fg(Color::DarkGray),
        ),
    }
}

/// Run the TUI event loop. Returns exit code.
pub fn run(mut app: App) -> anyhow::Result<i32> {
    use crossterm::event::{Event, KeyCode, KeyModifiers};

    let mut terminal = ratatui::init();

    while app.is_running() {
        // Render
        terminal.draw(|frame| {
            app.render(frame);
        })?;

        // Poll for events (with timeout so we can update the header)
        if crossterm::event::poll(Duration::from_millis(100))? {
            match crossterm::event::read()? {
                Event::Key(key) => {
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        // SIGINT
                        app.stop();
                    } else {
                        app.handle_input(key);
                    }
                }
                Event::Resize(..) => {
                    // Trigger re-render on resize
                }
                _ => {}
            }
        }
    }

    ratatui::restore();
    Ok(0)
}
