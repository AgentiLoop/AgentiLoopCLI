//! Full-screen terminal UI (`--tui`): a scrolling transcript, an input line,
//! and a status bar. The agent loop runs on the tokio runtime and talks to the
//! UI thread over channels; permission prompts appear as a modal.

use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Duration;

use agentiloop_core::permission::{Permission, PermissionPolicy};
use agentiloop_core::AgentEvent;
use async_trait::async_trait;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use tokio::sync::{mpsc, oneshot};

/// Messages from the agent task to the UI.
pub enum UiMsg {
    Event(AgentEvent),
    /// A plain informational line (slash command output).
    Line(String),
    Error(String),
    Permission(PermissionRequest),
    /// The agent finished the current prompt / command.
    Idle,
}

/// What the UI sends to the agent task.
pub enum Input {
    Submit(String),
}

pub struct PermissionRequest {
    pub tool: String,
    pub input: String,
    pub reply: oneshot::Sender<Answer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Allow,
    Always,
    Deny,
}

/// Permission policy that asks the UI thread and awaits its answer.
pub struct ChannelPolicy {
    tx: mpsc::UnboundedSender<UiMsg>,
    always: Mutex<HashSet<String>>,
}

impl ChannelPolicy {
    pub fn new(tx: mpsc::UnboundedSender<UiMsg>) -> Self {
        Self { tx, always: Mutex::new(HashSet::new()) }
    }
}

#[async_trait]
impl PermissionPolicy for ChannelPolicy {
    async fn check(&self, tool: &str, is_mutating: bool, input: &serde_json::Value) -> Permission {
        if !is_mutating || self.always.lock().unwrap().contains(tool) {
            return Permission::Allow;
        }
        let (reply, rx) = oneshot::channel();
        let req = PermissionRequest {
            tool: tool.to_string(),
            input: serde_json::to_string_pretty(input).unwrap_or_default(),
            reply,
        };
        if self.tx.send(UiMsg::Permission(req)).is_err() {
            return Permission::Deny;
        }
        match rx.await {
            Ok(Answer::Allow) => Permission::Allow,
            Ok(Answer::Always) => {
                self.always.lock().unwrap().insert(tool.to_string());
                Permission::Allow
            }
            _ => Permission::Deny,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    User,
    Assistant,
    Tool,
    ToolError,
    Info,
    Error,
}

struct Entry {
    kind: Kind,
    text: String,
}

/// UI state; backend-agnostic so it can be driven by tests.
pub struct App {
    entries: Vec<Entry>,
    input: String,
    /// Byte offset of the cursor within `input`.
    cursor: usize,
    /// Lines scrolled up from the bottom (0 = follow output).
    scroll: usize,
    busy: bool,
    quit: bool,
    modal: Option<PermissionRequest>,
    status: String,
    history: Vec<String>,
    hist_idx: Option<usize>,
    /// Whether the last entry is assistant text still being streamed.
    streaming: bool,
}

/// Result of a key press that the event loop must act on.
pub enum Action {
    Submit(String),
    Quit,
}

impl App {
    pub fn new(status: impl Into<String>) -> Self {
        Self {
            entries: Vec::new(),
            input: String::new(),
            cursor: 0,
            scroll: 0,
            busy: false,
            quit: false,
            modal: None,
            status: status.into(),
            history: Vec::new(),
            hist_idx: None,
            streaming: false,
        }
    }

    pub fn quit(&self) -> bool {
        self.quit
    }

    fn push(&mut self, kind: Kind, text: impl Into<String>) {
        self.streaming = false;
        self.entries.push(Entry { kind, text: text.into() });
    }

    pub fn apply(&mut self, msg: UiMsg) {
        match msg {
            UiMsg::Event(ev) => self.apply_event(ev),
            UiMsg::Line(s) => self.push(Kind::Info, s),
            UiMsg::Error(s) => self.push(Kind::Error, s),
            UiMsg::Permission(req) => self.modal = Some(req),
            UiMsg::Idle => self.busy = false,
        }
    }

    fn apply_event(&mut self, ev: AgentEvent) {
        match ev {
            AgentEvent::AssistantTextDelta(t) => {
                if !self.streaming {
                    self.push(Kind::Assistant, "");
                    self.streaming = true;
                }
                self.entries.last_mut().unwrap().text.push_str(&t);
            }
            // Deltas already built the text; just close the streaming entry.
            AgentEvent::AssistantText(t) => {
                if !self.streaming {
                    self.push(Kind::Assistant, t);
                }
                self.streaming = false;
            }
            AgentEvent::ToolCall { name, input, .. } => {
                self.push(Kind::Tool, format!("\u{1f527} {name} {}", crate::compact(&input)));
            }
            AgentEvent::ToolResult { output, is_error, .. } => {
                let preview: String = output.lines().take(8).collect::<Vec<_>>().join("\n");
                let (kind, mark) = if is_error { (Kind::ToolError, "✖") } else { (Kind::Tool, "✓") };
                self.push(kind, format!("{mark} {preview}"));
            }
            AgentEvent::TurnComplete { input_tokens, output_tokens } => {
                tracing::debug!(input_tokens, output_tokens, "turn");
            }
            AgentEvent::Compacted { before_tokens, messages_dropped } => {
                self.push(Kind::Info, crate::compacted_line(before_tokens, messages_dropped));
            }
            AgentEvent::Done { .. } => {}
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        if key.kind != KeyEventKind::Press {
            return None;
        }
        if let Some(req) = self.modal.take() {
            let answer = match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => Answer::Allow,
                KeyCode::Char('a') | KeyCode::Char('A') => Answer::Always,
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Answer::Deny,
                _ => {
                    self.modal = Some(req);
                    return None;
                }
            };
            let _ = req.reply.send(answer);
            return None;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('c') | KeyCode::Char('d') if ctrl => {
                self.quit = true;
                return Some(Action::Quit);
            }
            KeyCode::Char('u') if ctrl => {
                self.input.clear();
                self.cursor = 0;
            }
            KeyCode::Enter => {
                let line = self.input.trim().to_string();
                if line.is_empty() || self.busy {
                    return None;
                }
                self.input.clear();
                self.cursor = 0;
                self.hist_idx = None;
                self.scroll = 0;
                if self.history.last() != Some(&line) {
                    self.history.push(line.clone());
                }
                if line == "/exit" || line == "/quit" {
                    self.quit = true;
                    return Some(Action::Quit);
                }
                if !line.starts_with('/') {
                    self.push(Kind::User, line.clone());
                }
                self.busy = true;
                return Some(Action::Submit(line));
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor, c);
                self.cursor += c.len_utf8();
            }
            KeyCode::Backspace => {
                if let Some(prev) = self.prev_boundary() {
                    self.input.replace_range(prev..self.cursor, "");
                    self.cursor = prev;
                }
            }
            KeyCode::Delete => {
                if let Some(next) = self.next_boundary() {
                    self.input.replace_range(self.cursor..next, "");
                }
            }
            KeyCode::Left => self.cursor = self.prev_boundary().unwrap_or(self.cursor),
            KeyCode::Right => self.cursor = self.next_boundary().unwrap_or(self.cursor),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.input.len(),
            KeyCode::Up => self.recall(-1),
            KeyCode::Down => self.recall(1),
            KeyCode::PageUp => self.scroll = self.scroll.saturating_add(10),
            KeyCode::PageDown => self.scroll = self.scroll.saturating_sub(10),
            _ => {}
        }
        None
    }

    fn prev_boundary(&self) -> Option<usize> {
        self.input[..self.cursor].char_indices().next_back().map(|(i, _)| i)
    }

    fn next_boundary(&self) -> Option<usize> {
        self.input[self.cursor..].chars().next().map(|c| self.cursor + c.len_utf8())
    }

    /// Up/down through earlier prompts, like the REPL.
    fn recall(&mut self, dir: i32) {
        if self.history.is_empty() {
            return;
        }
        let next = match (self.hist_idx, dir) {
            (None, -1) => Some(self.history.len() - 1),
            (None, _) => None,
            (Some(0), -1) => Some(0),
            (Some(i), -1) => Some(i - 1),
            (Some(i), _) if i + 1 >= self.history.len() => None,
            (Some(i), _) => Some(i + 1),
        };
        self.hist_idx = next;
        self.input = next.map(|i| self.history[i].clone()).unwrap_or_default();
        self.cursor = self.input.len();
    }

    pub fn draw(&self, frame: &mut Frame) {
        let [transcript, input, status] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(3), Constraint::Length(1)]).areas(frame.area());

        self.draw_transcript(frame, transcript);

        let title = if self.busy { " working… " } else { " prompt " };
        let block = Block::default().borders(Borders::ALL).title(title);
        let inner = block.inner(input);
        frame.render_widget(Paragraph::new(self.input.as_str()).block(block), input);
        if self.modal.is_none() {
            let col = self.input[..self.cursor].chars().count() as u16;
            frame.set_cursor_position((inner.x + col.min(inner.width.saturating_sub(1)), inner.y));
        }

        let help = "  Enter send · ↑↓ history · PgUp/PgDn scroll · Ctrl-C quit";
        let bar = Line::from(vec![Span::raw(self.status.clone()), Span::styled(help, Style::default().fg(Color::DarkGray))]);
        frame.render_widget(Paragraph::new(bar).style(Style::default().add_modifier(Modifier::REVERSED)), status);

        if let Some(req) = &self.modal {
            self.draw_modal(frame, req);
        }
    }

    fn draw_transcript(&self, frame: &mut Frame, area: Rect) {
        let width = area.width.max(1) as usize;
        let mut lines: Vec<Line> = Vec::new();
        for e in &self.entries {
            let (prefix, style) = match e.kind {
                Kind::User => ("> ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Kind::Assistant => ("", Style::default()),
                Kind::Tool => ("  ", Style::default().fg(Color::Yellow)),
                Kind::ToolError => ("  ", Style::default().fg(Color::Red)),
                Kind::Info => ("· ", Style::default().fg(Color::DarkGray)),
                Kind::Error => ("! ", Style::default().fg(Color::Red)),
            };
            let indent = " ".repeat(prefix.len());
            let opts = textwrap::Options::new(width.saturating_sub(prefix.len()).max(1));
            let mut first = true;
            for raw in e.text.split('\n') {
                // textwrap yields one empty piece for an empty line, so blank
                // lines in the text come through as exactly one blank row.
                for piece in textwrap::wrap(raw, &opts) {
                    let p = if first { prefix } else { indent.as_str() };
                    first = false;
                    lines.push(Line::from(Span::styled(format!("{p}{piece}"), style)));
                }
            }
            lines.push(Line::default());
        }
        let height = area.height as usize;
        let end = lines.len().saturating_sub(self.scroll.min(lines.len().saturating_sub(height)));
        let start = end.saturating_sub(height);
        frame.render_widget(Paragraph::new(lines[start..end].to_vec()), area);
    }

    fn draw_modal(&self, frame: &mut Frame, req: &PermissionRequest) {
        let area = frame.area();
        let w = area.width.saturating_sub(4).min(80).max(20);
        let body: Vec<&str> = req.input.lines().take(12).collect();
        let h = (body.len() as u16 + 4).min(area.height.saturating_sub(2)).max(5);
        let rect = Rect::new(area.x + (area.width.saturating_sub(w)) / 2, area.y + (area.height.saturating_sub(h)) / 2, w, h);
        let mut lines: Vec<Line> = body.iter().map(|l| Line::from(*l)).collect();
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            format!("[y]es  [n]o  [a]lways for `{}`", req.tool),
            Style::default().add_modifier(Modifier::BOLD),
        )));
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(format!(" \u{26a0} {} wants to run ", req.tool));
        frame.render_widget(Clear, rect);
        frame.render_widget(Paragraph::new(lines).block(block), rect);
    }
}

/// Blocking UI loop: drains agent messages, redraws, and forwards key presses.
/// Runs until the user quits or the agent side hangs up.
pub fn run(
    mut app: App,
    mut rx: mpsc::UnboundedReceiver<UiMsg>,
    tx: mpsc::UnboundedSender<Input>,
) -> std::io::Result<()> {
    let mut terminal = ratatui::try_init()?;
    let result = (|| {
        loop {
            loop {
                match rx.try_recv() {
                    Ok(msg) => app.apply(msg),
                    Err(mpsc::error::TryRecvError::Empty) => break,
                    Err(mpsc::error::TryRecvError::Disconnected) => return Ok(()),
                }
            }
            terminal.draw(|f| app.draw(f))?;
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    match app.handle_key(key) {
                        Some(Action::Quit) => return Ok(()),
                        Some(Action::Submit(line)) => {
                            if tx.send(Input::Submit(line)).is_err() {
                                return Ok(());
                            }
                        }
                        None => {}
                    }
                }
            }
            if app.quit() {
                return Ok(());
            }
        }
    })();
    ratatui::restore();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn type_str(app: &mut App, s: &str) {
        for c in s.chars() {
            app.handle_key(key(KeyCode::Char(c)));
        }
    }

    fn screen(app: &App, w: u16, h: u16) -> String {
        let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
        t.draw(|f| app.draw(f)).unwrap();
        let buf = t.backend().buffer();
        let mut out = String::new();
        for y in 0..h {
            for x in 0..w {
                out.push_str(buf.cell((x, y)).unwrap().symbol());
            }
            out.push('\n');
        }
        out
    }

    #[test]
    fn enter_submits_and_records_user_line() {
        let mut app = App::new("s");
        type_str(&mut app, "hello there");
        let Some(Action::Submit(line)) = app.handle_key(key(KeyCode::Enter)) else { panic!("expected submit") };
        assert_eq!(line, "hello there");
        assert!(app.busy);
        assert!(app.input.is_empty());
        let s = screen(&app, 40, 8);
        assert!(s.contains("> hello there"), "{s}");
        assert!(s.contains("working"), "{s}");
    }

    #[test]
    fn enter_ignored_while_busy_and_after_idle() {
        let mut app = App::new("s");
        type_str(&mut app, "a");
        app.handle_key(key(KeyCode::Enter));
        type_str(&mut app, "b");
        assert!(app.handle_key(key(KeyCode::Enter)).is_none());
        app.apply(UiMsg::Idle);
        assert!(matches!(app.handle_key(key(KeyCode::Enter)), Some(Action::Submit(l)) if l == "b"));
    }

    #[test]
    fn slash_exit_quits_and_ctrl_c_quits() {
        let mut app = App::new("s");
        type_str(&mut app, "/exit");
        assert!(matches!(app.handle_key(key(KeyCode::Enter)), Some(Action::Quit)));
        let mut app = App::new("s");
        assert!(matches!(app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)), Some(Action::Quit)));
    }

    #[test]
    fn streaming_deltas_accumulate_into_one_entry() {
        let mut app = App::new("s");
        app.apply(UiMsg::Event(AgentEvent::AssistantTextDelta("Hel".into())));
        app.apply(UiMsg::Event(AgentEvent::AssistantTextDelta("lo".into())));
        app.apply(UiMsg::Event(AgentEvent::AssistantText("Hello".into())));
        assert_eq!(app.entries.len(), 1);
        assert_eq!(app.entries[0].text, "Hello");
        app.apply(UiMsg::Event(AgentEvent::AssistantTextDelta("again".into())));
        assert_eq!(app.entries.len(), 2);
    }

    #[test]
    fn tool_events_render_with_marks() {
        let mut app = App::new("s");
        app.apply(UiMsg::Event(AgentEvent::ToolCall { id: "1".into(), name: "bash".into(), input: serde_json::json!({"cmd":"ls"}) }));
        app.apply(UiMsg::Event(AgentEvent::ToolResult { id: "1".into(), name: "bash".into(), output: "boom".into(), is_error: true }));
        let s = screen(&app, 50, 8);
        assert!(s.contains("\u{1f527}") && s.contains("bash {\"cmd\":\"ls\"}"), "{s}");
        assert!(s.contains("✖ boom"), "{s}");
    }

    #[test]
    fn long_lines_wrap_and_bottom_follows() {
        let mut app = App::new("s");
        for i in 0..30 {
            app.apply(UiMsg::Line(format!("line {i} {}", "x".repeat(60))));
        }
        let s = screen(&app, 30, 10);
        assert!(s.contains("line 29"), "{s}");
        assert!(!s.contains("line 0 "), "{s}");
        app.handle_key(key(KeyCode::PageUp));
        let s2 = screen(&app, 30, 10);
        assert!(!s2.contains("line 29"), "{s2}");
    }

    #[test]
    fn blank_line_in_text_renders_as_one_blank_row() {
        let mut app = App::new("s");
        app.apply(UiMsg::Event(AgentEvent::AssistantText("para one\n\npara two".into())));
        let s = screen(&app, 20, 8);
        let rows: Vec<&str> = s.lines().map(str::trim_end).collect();
        let one = rows.iter().position(|r| *r == "para one").unwrap();
        let two = rows.iter().position(|r| *r == "para two").unwrap();
        assert_eq!(two - one, 2, "expected exactly one blank row between paragraphs:\n{s}");
    }

    #[test]
    fn permission_modal_answers_and_closes() {
        let mut app = App::new("s");
        let (reply, rx) = oneshot::channel();
        app.apply(UiMsg::Permission(PermissionRequest { tool: "bash".into(), input: "{\"cmd\": \"rm x\"}".into(), reply }));
        let s = screen(&app, 60, 12);
        assert!(s.contains("bash wants to run"), "{s}");
        assert!(s.contains("[y]es"), "{s}");
        // Unrelated keys keep the modal up and don't reach the input.
        app.handle_key(key(KeyCode::Char('z')));
        assert!(app.modal.is_some());
        assert!(app.input.is_empty());
        app.handle_key(key(KeyCode::Char('a')));
        assert!(app.modal.is_none());
        assert_eq!(rx.blocking_recv().unwrap(), Answer::Always);
    }

    #[tokio::test]
    async fn channel_policy_round_trip_and_always() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let policy = std::sync::Arc::new(ChannelPolicy::new(tx));
        let p2 = policy.clone();
        let check = tokio::spawn(async move { p2.check("bash", true, &serde_json::json!({})).await });
        let Some(UiMsg::Permission(req)) = rx.recv().await else { panic!("expected permission request") };
        assert_eq!(req.tool, "bash");
        req.reply.send(Answer::Always).unwrap();
        assert_eq!(check.await.unwrap(), Permission::Allow);
        // Remembered: no second prompt.
        assert_eq!(policy.check("bash", true, &serde_json::json!({})).await, Permission::Allow);
        assert!(rx.try_recv().is_err());
        // Non-mutating never asks.
        assert_eq!(policy.check("read_file", false, &serde_json::json!({})).await, Permission::Allow);
        // Dropped reply → deny.
        let p3 = policy.clone();
        let check = tokio::spawn(async move { p3.check("write_file", true, &serde_json::json!({})).await });
        let Some(UiMsg::Permission(req)) = rx.recv().await else { panic!() };
        drop(req);
        assert_eq!(check.await.unwrap(), Permission::Deny);
    }

    #[test]
    fn history_recall_up_down() {
        let mut app = App::new("s");
        type_str(&mut app, "one");
        app.handle_key(key(KeyCode::Enter));
        app.apply(UiMsg::Idle);
        type_str(&mut app, "two");
        app.handle_key(key(KeyCode::Enter));
        app.apply(UiMsg::Idle);
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.input, "two");
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.input, "one");
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.input, "two");
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.input, "");
    }
}
