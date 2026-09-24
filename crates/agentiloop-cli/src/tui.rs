//! Full-screen terminal UI (`--tui`): a scrolling transcript, an input line,
//! and a status bar. The agent loop runs on the tokio runtime and talks to the
//! UI thread over channels; permission prompts appear as a modal.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use agentiloop_core::permission::{Permission, PermissionPolicy};
use agentiloop_core::AgentEvent;
use async_trait::async_trait;
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use tokio::sync::{mpsc, oneshot};

use crate::diff::{self, FileDiff, Tag};

/// Messages from the agent task to the UI.
pub enum UiMsg {
    Event(AgentEvent),
    /// A plain informational line (slash command output).
    Line(String),
    Error(String),
    Permission(PermissionRequest),
    /// Replace the status-bar text (model or session changed).
    Status(String),
    /// The agent finished the current prompt / command.
    Idle,
    /// A user prompt from a resumed session being replayed.
    User(String),
    /// Wipe the transcript (a different session was loaded, or /clear).
    Clear,
    /// A file was written or edited: show its diff and update the files pane.
    Diff(diff::Change),
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
    /// Esc: skip this call, keep the agent running.
    Cancel,
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
            Ok(Answer::Cancel) => Permission::Cancel,
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
    /// Pre-styled lines (syntax-highlighted code); when set, `text` is ignored.
    code: Option<Vec<Vec<Span<'static>>>>,
    /// Whole-row style per `code` line (diff tints); empty = none.
    styles: Vec<Style>,
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
    /// When the current prompt started, for the spinner and elapsed time.
    busy_since: Instant,
    /// What the agent is doing right now, shown next to the spinner.
    activity: String,
    quit: bool,
    modal: Option<PermissionRequest>,
    status: String,
    /// Speed of the most recent model call, shown in the status bar.
    speed: Option<String>,
    history: Vec<String>,
    hist_idx: Option<usize>,
    /// Where `history` is persisted (shared with the REPL's rustyline file).
    history_file: Option<PathBuf>,
    /// Whether the last entry is assistant text still being streamed.
    streaming: bool,
    /// `path` argument of in-flight `read_file` calls, by tool-call id.
    pending_paths: HashMap<String, String>,
    /// Screen cells occupied by links in the last frame, for click handling.
    link_hits: RefCell<Vec<LinkHit>>,
    /// Files changed this session (total diff each), in first-touched order.
    files: Vec<FileDiff>,
    /// Which file's diff the pane shows (the most recently changed one).
    files_sel: usize,
    /// Pre-rendered diff of `files[files_sel]` for the pane.
    pane_rows: (Vec<Vec<Span<'static>>>, Vec<Style>),
    /// Ctrl-F toggles the files pane.
    show_files: bool,
}

struct LinkHit {
    x0: u16,
    x1: u16,
    y: u16,
    url: String,
}

/// Result of a key press or mouse event that the event loop must act on.
pub enum Action {
    Submit(String),
    Quit,
    /// A link was clicked; open it in the system browser.
    OpenUrl(String),
}

impl App {
    pub fn new(status: impl Into<String>) -> Self {
        Self {
            entries: Vec::new(),
            input: String::new(),
            cursor: 0,
            scroll: 0,
            busy: false,
            busy_since: Instant::now(),
            activity: String::new(),
            quit: false,
            modal: None,
            status: status.into(),
            speed: None,
            history: Vec::new(),
            hist_idx: None,
            history_file: None,
            streaming: false,
            pending_paths: HashMap::new(),
            link_hits: RefCell::new(Vec::new()),
            files: Vec::new(),
            files_sel: 0,
            pane_rows: (Vec::new(), Vec::new()),
            show_files: true,
        }
    }

    /// Load prompt history from `path` and keep appending to it, so ↑ recalls
    /// prompts from earlier launches (TUI and REPL share the file).
    pub fn with_history_file(mut self, path: Option<PathBuf>) -> Self {
        if let Some(p) = &path {
            self.history = load_history(p);
        }
        self.history_file = path;
        self
    }

    pub fn quit(&self) -> bool {
        self.quit
    }

    fn push(&mut self, kind: Kind, text: impl Into<String>) {
        self.streaming = false;
        self.entries.push(Entry { kind, text: text.into(), code: None, styles: Vec::new() });
    }

    pub fn apply(&mut self, msg: UiMsg) {
        match msg {
            UiMsg::Event(ev) => self.apply_event(ev),
            UiMsg::Line(s) => self.push(Kind::Info, s),
            UiMsg::Error(s) => self.push(Kind::Error, s),
            UiMsg::Permission(req) => {
                self.activity = "Waiting for approval".into();
                self.modal = Some(req)
            }
            UiMsg::Status(s) => self.status = s,
            UiMsg::Idle => self.busy = false,
            UiMsg::User(s) => self.push(Kind::User, s),
            UiMsg::Clear => {
                self.entries.clear();
                self.streaming = false;
                self.pending_paths.clear();
                self.scroll = 0;
                self.files.clear();
                self.refresh_pane();
            }
            UiMsg::Diff(c) => {
                let (code, styles) = diff_rows(&c.edit, diff::INLINE_MAX, true);
                self.streaming = false;
                self.entries.push(Entry { kind: Kind::Tool, text: String::new(), code: Some(code), styles });
                let total = c.total;
                match self.files.iter().position(|f| f.path == total.path) {
                    // Edited back to the original: no longer a change.
                    Some(i) if total.is_empty() => drop(self.files.remove(i)),
                    Some(i) => self.files[i] = total.clone(),
                    None if !total.is_empty() => self.files.push(total.clone()),
                    None => {}
                }
                self.files_sel =
                    self.files.iter().position(|f| f.path == total.path).unwrap_or(self.files.len().saturating_sub(1));
                self.refresh_pane();
            }
        }
    }

    fn refresh_pane(&mut self) {
        self.pane_rows = self.files.get(self.files_sel).map(|f| diff_rows(f, PANE_MAX, false)).unwrap_or_default();
    }

    fn apply_event(&mut self, ev: AgentEvent) {
        self.activity = match &ev {
            AgentEvent::AssistantTextDelta(_) => "Writing".into(),
            AgentEvent::ToolCall { name, .. } => format!("Running {name}"),
            AgentEvent::Compacted { .. } => "Compacting".into(),
            _ => "Thinking".into(),
        };
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
            AgentEvent::ToolCall { id, name, input } => {
                if name == "read_file" {
                    if let Some(p) = input.get("path").and_then(|v| v.as_str()) {
                        self.pending_paths.insert(id, p.to_string());
                    }
                }
                self.push(Kind::Tool, format!("\u{1f527} {name} {}", crate::compact(&input)));
            }
            AgentEvent::ToolResult { id, output, is_error, .. } => {
                let path = self.pending_paths.remove(&id);
                let head: Vec<&str> = output.lines().take(8).collect();
                let (kind, mark) = if is_error { (Kind::ToolError, "✖") } else { (Kind::Tool, "✓") };
                // Numbered read_file output gets syntax-highlighted by extension.
                if !is_error {
                    if let Some(mut code) = path.and_then(|p| crate::highlight::highlight_numbered(&head, &p)) {
                        let mark_style = Style::default().fg(Color::Yellow);
                        for (i, line) in code.iter_mut().enumerate() {
                            line.insert(0, Span::styled(if i == 0 { format!("{mark} ") } else { "  ".into() }, mark_style));
                        }
                        self.streaming = false;
                        self.entries.push(Entry { kind, text: String::new(), code: Some(code), styles: Vec::new() });
                        return;
                    }
                }
                // Continuation lines are indented past the mark so multi-line
                // output (e.g. numbered file contents) stays column-aligned.
                let preview = head.join("\n  ");
                self.push(kind, format!("{mark} {preview}"));
            }
            AgentEvent::TurnComplete { input_tokens, output_tokens, elapsed_ms, first_token_ms } => {
                tracing::debug!(input_tokens, output_tokens, elapsed_ms, ?first_token_ms, "turn");
                self.speed = Some(crate::speed_line(output_tokens, elapsed_ms, first_token_ms));
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
                KeyCode::Char('n') | KeyCode::Char('N') => Answer::Deny,
                KeyCode::Esc => Answer::Cancel,
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
            KeyCode::Char('f') if ctrl => self.show_files = !self.show_files,
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
                    if let Some(p) = &self.history_file {
                        if let Err(e) = save_history(p, &self.history) {
                            tracing::warn!("could not save history: {e}");
                        }
                    }
                }
                if line == "/exit" || line == "/quit" {
                    self.quit = true;
                    return Some(Action::Quit);
                }
                if !line.starts_with('/') {
                    self.push(Kind::User, line.clone());
                }
                self.busy = true;
                self.busy_since = Instant::now();
                self.activity = if line.starts_with('/') { "Working" } else { "Thinking" }.into();
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

    /// Wheel scrolls the transcript; a left click on a link opens it.
    pub fn handle_mouse(&mut self, m: MouseEvent) -> Option<Action> {
        match m.kind {
            MouseEventKind::ScrollUp => self.scroll = self.scroll.saturating_add(3),
            MouseEventKind::ScrollDown => self.scroll = self.scroll.saturating_sub(3),
            MouseEventKind::Down(MouseButton::Left) if self.modal.is_none() => {
                let hits = self.link_hits.borrow();
                let hit = hits.iter().find(|h| h.y == m.row && m.column >= h.x0 && m.column < h.x1)?;
                return Some(Action::OpenUrl(hit.url.clone()));
            }
            _ => {}
        }
        None
    }

    pub fn draw(&self, frame: &mut Frame) {
        let [transcript, input, status] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(3), Constraint::Length(1)]).areas(frame.area());

        // Claude Code-style "files changed" pane on the right, when there's room.
        let transcript = if self.show_files && !self.files.is_empty() && transcript.width >= 100 {
            let pane_w = (transcript.width * 2 / 5).clamp(36, 90);
            let [left, right] = Layout::horizontal([Constraint::Min(40), Constraint::Length(pane_w)]).areas(transcript);
            self.draw_files(frame, right);
            left
        } else {
            transcript
        };
        self.draw_transcript(frame, transcript);

        let title = if self.busy { self.busy_title() } else { Line::from(" prompt ") };
        let block = Block::default().borders(Borders::ALL).title(title);
        let inner = block.inner(input);
        frame.render_widget(Paragraph::new(self.input.as_str()).block(block), input);
        if self.modal.is_none() {
            let col = self.input[..self.cursor].chars().count() as u16;
            frame.set_cursor_position((inner.x + col.min(inner.width.saturating_sub(1)), inner.y));
        }

        let help = "  Enter send · ↑↓ history · PgUp/PgDn scroll · click links · Ctrl-F files · Ctrl-C quit";
        let mut bar = vec![Span::raw(self.status.clone())];
        if let Some(s) = &self.speed {
            bar.push(Span::styled(format!("⏱ {s} "), Style::default().fg(Color::Green)));
        }
        bar.push(Span::styled(help, Style::default().fg(Color::DarkGray)));
        let bar = Line::from(bar);
        frame.render_widget(Paragraph::new(bar).style(Style::default().add_modifier(Modifier::REVERSED)), status);

        if let Some(req) = &self.modal {
            self.draw_modal(frame, req);
        }
    }

    /// Animated prompt-box title while the agent works, e.g. ` ✻ Thinking...  12s `.
    /// The UI loop redraws every 50 ms, so the frame is derived from elapsed time.
    fn busy_title(&self) -> Line<'static> {
        const SPINNER: [&str; 10] = ["·", "✢", "✳", "✶", "✻", "✽", "✻", "✶", "✳", "✢"];
        const DOTS: [&str; 4] = ["   ", ".  ", ".. ", "..."];
        let ms = self.busy_since.elapsed().as_millis();
        let spin = SPINNER[(ms / 120) as usize % SPINNER.len()];
        let dots = DOTS[(ms / 350) as usize % DOTS.len()];
        let secs = ms / 1000;
        let elapsed = if secs >= 60 { format!("{}m {:02}s", secs / 60, secs % 60) } else { format!("{secs}s") };
        let accent = Style::default().fg(Color::Rgb(0xE0, 0x8A, 0x5B)).add_modifier(Modifier::BOLD);
        Line::from(vec![
            Span::styled(format!(" {spin} "), accent),
            Span::styled(format!("{}{dots} ", self.activity), accent),
            Span::styled(format!("{elapsed} "), Style::default().fg(Color::DarkGray)),
        ])
    }

    fn draw_transcript(&self, frame: &mut Frame, area: Rect) {
        let width = area.width.max(1) as usize;
        let mut lines: Vec<Line> = Vec::new();
        // (transcript line, start col, end col, url) for every link.
        let mut links: Vec<(usize, usize, usize, String)> = Vec::new();
        for e in &self.entries {
            if let Some(code) = &e.code {
                for (n, line) in code.iter().enumerate() {
                    let style = e.styles.get(n).copied().unwrap_or_default();
                    for (i, piece) in crate::highlight::hard_wrap(line.clone(), width).into_iter().enumerate() {
                        let mut spans = if i == 0 { Vec::new() } else { vec![Span::raw("  ")] };
                        spans.extend(piece);
                        lines.push(fill_row(spans, width, style));
                    }
                }
                lines.push(Line::default());
                continue;
            }
            if e.kind == Kind::Assistant {
                let (md, md_links) = crate::markdown::render_links(&e.text, width);
                links.extend(md_links.into_iter().map(|l| (lines.len() + l.line, l.start, l.end, l.url)));
                lines.extend(md);
                lines.push(Line::default());
                continue;
            }
            let (prefix, style) = match e.kind {
                Kind::User => ("> ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Kind::Assistant => ("", Style::default()),
                Kind::Tool => ("  ", Style::default().fg(Color::Yellow)),
                Kind::ToolError => ("  ", Style::default().fg(Color::Red)),
                // Light gray: readable on dark themes, still distinct from replies.
                Kind::Info => ("· ", Style::default().fg(Color::Rgb(0xC8, 0xC8, 0xCC))),
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
        *self.link_hits.borrow_mut() = links
            .into_iter()
            .filter(|(line, ..)| (start..end).contains(line))
            .map(|(line, s, e, url)| LinkHit {
                x0: area.x + s.min(width) as u16,
                x1: area.x + e.min(width) as u16,
                y: area.y + (line - start) as u16,
                url,
            })
            .collect();
        frame.render_widget(Paragraph::new(lines[start..end].to_vec()), area);
    }

    /// Right-hand pane: every changed file with its +/- counts, then the
    /// most recently changed file's diff.
    fn draw_files(&self, frame: &mut Frame, area: Rect) {
        let (added, removed) = self.files.iter().fold((0, 0), |(a, r), f| (a + f.added, r + f.removed));
        let n = self.files.len();
        let bold = Style::default().add_modifier(Modifier::BOLD);
        let title = Line::from(vec![
            Span::styled(format!(" {n} file{} changed ", if n == 1 { "" } else { "s" }), bold),
            Span::styled(format!("+{added} "), Style::default().fg(rgb(diff::ADDED_FG))),
            Span::styled(format!("-{removed} "), Style::default().fg(rgb(diff::REMOVED_FG))),
        ]);
        let block = Block::default().borders(Borders::LEFT).border_style(Style::default().fg(Color::DarkGray)).title(title);
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let w = inner.width.max(1) as usize;
        let mut lines: Vec<Line> = Vec::new();
        for (i, f) in self.files.iter().enumerate() {
            let counts = format!("+{} -{}", f.added, f.removed);
            let room = w.saturating_sub(counts.len() + 1);
            // Keep the end of long paths: the file name matters most.
            let len = f.path.chars().count();
            let path = if len > room { format!("…{}", f.path.chars().skip(len + 1 - room.max(1)).collect::<String>()) } else { f.path.clone() };
            let pad = w.saturating_sub(path.chars().count() + counts.len());
            let style = if i == self.files_sel { bold } else { Style::default() };
            lines.push(Line::from(vec![
                Span::styled(path, style),
                Span::raw(" ".repeat(pad)),
                Span::styled(format!("+{}", f.added), Style::default().fg(rgb(diff::ADDED_FG))),
                Span::raw(" "),
                Span::styled(format!("-{}", f.removed), Style::default().fg(rgb(diff::REMOVED_FG))),
            ]));
        }
        if let Some(f) = self.files.get(self.files_sel) {
            lines.push(Line::from(Span::styled("─".repeat(w), Style::default().fg(Color::DarkGray))));
            lines.push(Line::from(Span::styled(f.path.clone(), bold)));
            let (rows, styles) = &self.pane_rows;
            for (n, row) in rows.iter().enumerate() {
                if lines.len() >= inner.height as usize {
                    break;
                }
                let style = styles.get(n).copied().unwrap_or_default();
                for piece in crate::highlight::hard_wrap(row.clone(), w) {
                    lines.push(fill_row(piece, w, style));
                }
            }
        }
        lines.truncate(inner.height as usize);
        frame.render_widget(Paragraph::new(lines), inner);
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
            format!("[y]es  [n]o  [a]lways for `{}`  [esc] skip", req.tool),
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

/// Diff lines kept for the files pane (it only shows what fits anyway).
const PANE_MAX: usize = 400;

/// A row with `style`; tinted rows are padded to `width` so the background
/// spans the whole row (Paragraph only paints under the text).
fn fill_row(mut spans: Vec<Span<'static>>, width: usize, style: Style) -> Line<'static> {
    if style.bg.is_some() {
        let used: usize = spans.iter().map(Span::width).sum();
        spans.push(Span::raw(" ".repeat(width.saturating_sub(used))));
    }
    Line::from(spans).style(style)
}

fn rgb((r, g, b): (u8, u8, u8)) -> Color {
    Color::Rgb(r, g, b)
}

/// Styled rows for a diff, Claude Code style: an optional summary header, then
/// numbered `-`/`+` lines with syntax colors on red/green tinted rows (the
/// tint is the returned per-row style, so it fills the whole width).
fn diff_rows(d: &FileDiff, max: usize, header: bool) -> (Vec<Vec<Span<'static>>>, Vec<Style>) {
    let dim = Style::default().fg(Color::DarkGray);
    let mut rows: Vec<Vec<Span<'static>>> = Vec::new();
    let mut styles = Vec::new();
    if header {
        rows.push(vec![
            Span::styled("⎿ ", dim),
            Span::styled(d.path.clone(), Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!(": {}", d.summary())),
        ]);
        styles.push(Style::default());
    }
    let total: usize = d.hunks.iter().map(Vec::len).sum();
    let mut shown = 0;
    'hunks: for (h, hunk) in d.hunks.iter().enumerate() {
        if h > 0 {
            rows.push(vec![Span::styled(format!("{:>7}", "⋮"), dim)]);
            styles.push(Style::default());
        }
        for l in hunk {
            if shown == max {
                break 'hunks;
            }
            shown += 1;
            let (sign, fg, bg) = match l.tag {
                Tag::Context => (' ', None, None),
                Tag::Removed => ('-', Some(rgb(diff::REMOVED_FG)), Some(rgb(diff::REMOVED_BG))),
                Tag::Added => ('+', Some(rgb(diff::ADDED_FG)), Some(rgb(diff::ADDED_BG))),
            };
            let mut spans = vec![
                Span::styled(format!("{:>5} ", l.no), dim),
                Span::styled(format!("{sign} "), fg.map_or(dim, |c| Style::default().fg(c).add_modifier(Modifier::BOLD))),
            ];
            match crate::highlight::highlight(&l.text, d.ext()).and_then(|mut v| v.pop()) {
                Some(code) => spans.extend(code),
                None => spans.push(Span::styled(l.text.clone(), fg.map_or(Style::default(), |c| Style::default().fg(c)))),
            }
            rows.push(spans);
            styles.push(bg.map_or(Style::default(), |c| Style::default().bg(c)));
        }
    }
    if total > shown {
        rows.push(vec![Span::styled(format!("      … {} more lines", total - shown), dim)]);
        styles.push(Style::default());
    }
    (rows, styles)
}

/// Blocking UI loop: drains agent messages, redraws, and forwards key presses.
/// Runs until the user quits or the agent side hangs up.
pub fn run(
    mut app: App,
    mut rx: mpsc::UnboundedReceiver<UiMsg>,
    tx: mpsc::UnboundedSender<Input>,
) -> std::io::Result<()> {
    let mut terminal = ratatui::try_init()?;
    // Mouse reporting so link clicks reach us (wheel scrolling is handled too).
    execute!(std::io::stdout(), EnableMouseCapture)?;
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
                let action = match event::read()? {
                    Event::Key(key) => app.handle_key(key),
                    Event::Mouse(m) => app.handle_mouse(m),
                    _ => None,
                };
                match action {
                    Some(Action::Quit) => return Ok(()),
                    Some(Action::Submit(line)) => {
                        if tx.send(Input::Submit(line)).is_err() {
                            return Ok(());
                        }
                    }
                    Some(Action::OpenUrl(url)) => match open_url(&url) {
                        Ok(()) => app.apply(UiMsg::Line(format!("↗ {url}"))),
                        Err(e) => app.apply(UiMsg::Error(format!("could not open {url}: {e}"))),
                    },
                    None => {}
                }
            }
            if app.quit() {
                return Ok(());
            }
        }
    })();
    let _ = execute!(std::io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}

/// Entries kept in the history file.
const HISTORY_MAX: usize = 1000;

/// Read rustyline's `#V2` history format (`\\` and `\n` escaped); plain
/// one-entry-per-line files are accepted too. Missing file → empty.
fn load_history(path: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(path) else { return Vec::new() };
    let mut lines = text.lines().peekable();
    let v2 = lines.peek() == Some(&"#V2");
    if v2 {
        lines.next();
    }
    lines
        .filter(|l| !l.is_empty())
        .map(|l| if v2 { unescape(l) } else { l.to_string() })
        .collect()
}

/// Write the last `HISTORY_MAX` entries in rustyline's `#V2` format so the
/// line REPL can read the same file.
fn save_history(path: &Path, history: &[String]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut out = String::from("#V2\n");
    for h in &history[history.len().saturating_sub(HISTORY_MAX)..] {
        out.push_str(&h.replace('\\', "\\\\").replace('\n', "\\n"));
        out.push('\n');
    }
    std::fs::write(path, out)
}

fn unescape(l: &str) -> String {
    let mut s = String::with_capacity(l.len());
    let mut chars = l.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => s.push('\n'),
                Some(o) => s.push(o),
                None => s.push('\\'),
            }
        } else {
            s.push(c);
        }
    }
    s
}

/// Open `url` with the platform's default handler.
fn open_url(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = std::process::Command::new("open");
        c.arg(url);
        c
    };
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "start", "", url]);
        c
    };
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let mut cmd = {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(url);
        c
    };
    cmd.stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn().map(drop)
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
        assert!(s.contains("Thinking") && s.contains("0s"), "{s}");
    }

    #[test]
    fn replayed_session_shows_and_clear_wipes() {
        let mut app = App::new("s");
        app.apply(UiMsg::User("earlier question".into()));
        app.apply(UiMsg::Event(AgentEvent::AssistantText("earlier answer".into())));
        let s = screen(&app, 40, 8);
        assert!(s.contains("> earlier question") && s.contains("earlier answer"), "{s}");
        app.apply(UiMsg::Clear);
        assert!(!screen(&app, 40, 8).contains("earlier"));
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
    fn read_file_preview_is_highlighted_and_aligned() {
        let mut app = App::new("s");
        app.apply(UiMsg::Event(AgentEvent::ToolCall { id: "7".into(), name: "read_file".into(), input: serde_json::json!({"path":"src/a.rs"}) }));
        app.apply(UiMsg::Event(AgentEvent::ToolResult { id: "7".into(), name: "read_file".into(), output: "    1│fn main() {}\n    2│let x = 1;".into(), is_error: false }));
        let code = app.entries.last().unwrap().code.as_ref().expect("highlighted entry");
        assert!(code[0].iter().any(|s| s.content == "fn" && matches!(s.style.fg, Some(Color::Rgb(..)))));
        assert!(app.pending_paths.is_empty());
        let s = screen(&app, 40, 8);
        assert!(s.contains("✓     1│fn main() {}"), "{s}");
        assert!(s.contains("      2│let x = 1;"), "{s}");
    }

    #[test]
    fn clicking_a_link_opens_it_and_wheel_scrolls() {
        let mut app = App::new("s");
        app.apply(UiMsg::Event(AgentEvent::AssistantText("see [docs](https://a.io/x) now".into())));
        let s = screen(&app, 40, 8);
        assert!(s.contains("see docs now"), "{s}");
        let click = |col, row| MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column: col, row, modifiers: KeyModifiers::NONE };
        // "docs" occupies columns 4..8 on the first transcript row.
        assert!(matches!(app.handle_mouse(click(5, 0)), Some(Action::OpenUrl(u)) if u == "https://a.io/x"));
        assert!(app.handle_mouse(click(1, 0)).is_none());
        assert!(app.handle_mouse(click(5, 1)).is_none());
        let wheel = MouseEvent { kind: MouseEventKind::ScrollUp, column: 0, row: 0, modifiers: KeyModifiers::NONE };
        assert!(app.handle_mouse(wheel).is_none());
        assert_eq!(app.scroll, 3);
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
    fn esc_in_permission_modal_cancels_just_that_call() {
        let mut app = App::new("s");
        let (reply, rx) = oneshot::channel();
        app.apply(UiMsg::Permission(PermissionRequest { tool: "bash".into(), input: "{}".into(), reply }));
        assert!(screen(&app, 70, 12).contains("[esc] skip"));
        assert!(app.handle_key(key(KeyCode::Esc)).is_none());
        assert!(app.modal.is_none());
        assert!(!app.quit());
        assert_eq!(rx.blocking_recv().unwrap(), Answer::Cancel);
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
    fn history_persists_across_launches() {
        let dir = std::env::temp_dir().join(format!("agentiloop-hist-{}", std::process::id()));
        let path = dir.join("history.txt");
        let _ = std::fs::remove_dir_all(&dir);
        let mut app = App::new("s").with_history_file(Some(path.clone()));
        type_str(&mut app, "first \\ prompt");
        app.handle_key(key(KeyCode::Enter));
        app.apply(UiMsg::Idle);
        type_str(&mut app, "second");
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "#V2\nfirst \\\\ prompt\nsecond\n");
        // A fresh launch recalls both, newest first.
        let mut app2 = App::new("s").with_history_file(Some(path.clone()));
        app2.handle_key(key(KeyCode::Up));
        assert_eq!(app2.input, "second");
        app2.handle_key(key(KeyCode::Up));
        assert_eq!(app2.input, "first \\ prompt");
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn status_update_replaces_model_in_bar() {
        let mut app = App::new(" anthropic  claude-fable-5-1 ");
        app.apply(UiMsg::Status(" anthropic  claude-opus-5-5 ".into()));
        let s = screen(&app, 120, 8);
        assert!(s.contains("claude-opus-5-5") && !s.contains("claude-fable-5-1"), "{s}");
    }

    #[test]
    fn status_bar_shows_tokens_per_second() {
        let mut app = App::new("S");
        app.apply(UiMsg::Event(AgentEvent::TurnComplete { input_tokens: 10, output_tokens: 100, elapsed_ms: 2500, first_token_ms: Some(500) }));
        let s = screen(&app, 120, 8);
        assert!(s.contains("50.0 tok/s · ttft 0.50s · 100 tok in 2.5s"), "{s}");
    }

    fn change(path: &str, before: &str, after: &str) -> UiMsg {
        let d = diff::diff(path, Some(before), after);
        UiMsg::Diff(diff::Change { edit: d.clone(), total: d })
    }

    #[test]
    fn edit_diff_shows_numbered_tinted_lines() {
        let mut app = App::new("s");
        app.apply(change("src/a.txt", "one\ntwo\nthree\n", "one\nTWO\nthree\n"));
        let mut t = Terminal::new(TestBackend::new(60, 10)).unwrap();
        t.draw(|f| app.draw(f)).unwrap();
        let buf = t.backend().buffer();
        let row = |y: u16| (0..60).map(|x| buf.cell((x, y)).unwrap().symbol()).collect::<String>();
        assert!(row(0).starts_with("⎿ src/a.txt: Added 1 line, removed 1 line"), "{}", row(0));
        assert!(row(2).starts_with("    2 - two"), "{}", row(2));
        assert!(row(3).starts_with("    2 + TWO"), "{}", row(3));
        // The tint runs to the right edge, not just under the text.
        assert_eq!(buf.cell((59, 2)).unwrap().bg, rgb(diff::REMOVED_BG));
        assert_eq!(buf.cell((59, 3)).unwrap().bg, rgb(diff::ADDED_BG));
        assert_eq!(buf.cell((59, 1)).unwrap().bg, Color::Reset);
    }

    #[test]
    fn files_pane_lists_changes_and_toggles() {
        let mut app = App::new("s");
        app.apply(change("a.rs", "x\n", "y\n"));
        app.apply(change("docs/b.md", "", "1\n2\n3\n"));
        let s = screen(&app, 120, 20);
        assert!(s.contains("2 files changed +4 -1"), "{s}");
        assert!(s.contains("a.rs") && s.contains("+1 -1"), "{s}");
        assert!(s.contains("docs/b.md") && s.contains("+3 -0"), "{s}");
        // Narrow terminals and Ctrl-F hide it.
        assert!(!screen(&app, 80, 20).contains("files changed"));
        app.handle_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL));
        assert!(!screen(&app, 120, 20).contains("files changed"));
        // Editing a file back to its original drops it from the list.
        app.handle_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL));
        app.apply(change("a.rs", "x\n", "x\n"));
        assert!(screen(&app, 120, 20).contains("1 file changed +3 -0"));
        app.apply(UiMsg::Clear);
        assert!(!screen(&app, 120, 20).contains("changed"));
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
