//! Line diffs for the files the agent writes or edits, like Claude Code:
//! `-`/`+` lines with line numbers on red/green tints in the transcript, and
//! a running "files changed" list for the TUI's side pane.
//!
//! [`Tracker`] snapshots a file when `write_file`/`edit_file` is called and
//! diffs it once the tool succeeds. It must see events synchronously, before
//! the tool runs, so it lives on the agent side of the TUI channel.

use std::collections::HashMap;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use agentiloop_core::AgentEvent;
use similar::{ChangeTag, TextDiff};

/// Unchanged lines shown around each change.
const CONTEXT: usize = 3;
/// Diff lines shown inline per edit; the rest is summarized.
pub const INLINE_MAX: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Context,
    Removed,
    Added,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub tag: Tag,
    /// Old line number for removed lines, new line number otherwise.
    pub no: usize,
    pub text: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileDiff {
    /// Relative to the working directory when the file is inside it.
    pub path: String,
    pub added: usize,
    pub removed: usize,
    /// Changed regions with context; drawn with a `⋮` between them.
    pub hunks: Vec<Vec<DiffLine>>,
}

impl FileDiff {
    pub fn is_empty(&self) -> bool {
        self.added == 0 && self.removed == 0
    }

    /// e.g. "Added 3 lines, removed 1 line".
    pub fn summary(&self) -> String {
        let n = |k: usize, w: &str| format!("{k} {w}{}", if k == 1 { "" } else { "s" });
        match (self.added, self.removed) {
            (0, 0) => "No changes".into(),
            (a, 0) => format!("Added {}", n(a, "line")),
            (0, r) => format!("Removed {}", n(r, "line")),
            (a, r) => format!("Added {}, removed {}", n(a, "line"), n(r, "line")),
        }
    }

    /// File extension, for syntax highlighting.
    pub fn ext(&self) -> &str {
        Path::new(&self.path).extension().and_then(|e| e.to_str()).unwrap_or("")
    }
}

/// Diff `before` (None = new file) against `after`.
pub fn diff(path: &str, before: Option<&str>, after: &str) -> FileDiff {
    let td = TextDiff::from_lines(before.unwrap_or(""), after);
    let mut d = FileDiff { path: path.into(), ..Default::default() };
    for group in td.grouped_ops(CONTEXT) {
        let mut hunk = Vec::new();
        for op in &group {
            for ch in td.iter_changes(op) {
                let (tag, idx) = match ch.tag() {
                    ChangeTag::Equal => (Tag::Context, ch.new_index()),
                    ChangeTag::Delete => {
                        d.removed += 1;
                        (Tag::Removed, ch.old_index())
                    }
                    ChangeTag::Insert => {
                        d.added += 1;
                        (Tag::Added, ch.new_index())
                    }
                };
                let text = ch.value().trim_end_matches(['\n', '\r']).replace('\t', "    ");
                hunk.push(DiffLine { tag, no: idx.map_or(0, |i| i + 1), text });
            }
        }
        d.hunks.push(hunk);
    }
    d
}

/// One successful write/edit: this edit's diff, and the file's total change
/// since the agent first touched it this session.
#[derive(Debug, Clone)]
pub struct Change {
    pub edit: FileDiff,
    pub total: FileDiff,
}

pub struct Tracker {
    cwd: PathBuf,
    /// Tool-call id → (file, content before the call; None = didn't exist).
    pending: HashMap<String, (PathBuf, Option<String>)>,
    /// Content of each touched file before its first change.
    originals: HashMap<PathBuf, Option<String>>,
}

impl Tracker {
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self { cwd: cwd.into(), pending: HashMap::new(), originals: HashMap::new() }
    }

    /// Feed every agent event through here, in order, before the UI sees it.
    pub fn observe(&mut self, ev: &AgentEvent) -> Option<Change> {
        match ev {
            AgentEvent::ToolCall { id, name, input } if name == "write_file" || name == "edit_file" => {
                let p = input.get("path")?.as_str()?;
                let abs = if Path::new(p).is_absolute() { PathBuf::from(p) } else { self.cwd.join(p) };
                let before = match std::fs::read_to_string(&abs) {
                    Ok(s) => Some(s),
                    Err(_) if !abs.exists() => None,
                    Err(_) => return None, // not UTF-8 text: nothing to show
                };
                self.pending.insert(id.clone(), (abs, before));
                None
            }
            AgentEvent::ToolResult { id, is_error, .. } => {
                let (abs, before) = self.pending.remove(id)?;
                if *is_error {
                    return None;
                }
                let after = std::fs::read_to_string(&abs).ok()?;
                let original = self.originals.entry(abs.clone()).or_insert_with(|| before.clone());
                let shown = abs.strip_prefix(&self.cwd).unwrap_or(&abs).display().to_string();
                Some(Change {
                    edit: diff(&shown, before.as_deref(), &after),
                    total: diff(&shown, original.as_deref(), &after),
                })
            }
            _ => None,
        }
    }
}

// Tints shared with the TUI.
pub const REMOVED_BG: (u8, u8, u8) = (0x4B, 0x12, 0x12);
pub const ADDED_BG: (u8, u8, u8) = (0x12, 0x3D, 0x16);
pub const REMOVED_FG: (u8, u8, u8) = (0xFF, 0x7B, 0x72);
pub const ADDED_FG: (u8, u8, u8) = (0x7E, 0xE7, 0x87);

/// Gutter + sign for a line, e.g. `"   15 - "`.
pub fn gutter(l: &DiffLine) -> String {
    let sign = match l.tag {
        Tag::Context => ' ',
        Tag::Removed => '-',
        Tag::Added => '+',
    };
    format!("{:>5} {sign} ", l.no)
}

/// Whether stderr should get colors (a terminal, and NO_COLOR unset).
pub fn color_stderr() -> bool {
    std::io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

/// Plain-terminal rendering (REPL and one-shot): tinted rows with `color`,
/// plain `-`/`+` text otherwise. At most `max` diff lines.
pub fn ansi(d: &FileDiff, max: usize, color: bool) -> String {
    let bg = |(r, g, b): (u8, u8, u8)| format!("\x1b[48;2;{r};{g};{b}m");
    let fg = |(r, g, b): (u8, u8, u8)| format!("\x1b[38;2;{r};{g};{b}m");
    let mut out = format!("   ⎿ {}: {}\n", d.path, d.summary());
    let mut shown = 0;
    let total: usize = d.hunks.iter().map(Vec::len).sum();
    for (h, hunk) in d.hunks.iter().enumerate() {
        if h > 0 && shown < max {
            out.push_str(&format!("{:>7}\n", "⋮"));
        }
        for l in hunk {
            if shown == max {
                break;
            }
            shown += 1;
            let row = format!("{}{}", gutter(l), l.text);
            if !color {
                out.push_str(&row);
            } else {
                match l.tag {
                    // \x1b[K paints the background to the end of the row.
                    Tag::Removed => out.push_str(&format!("{}{}{row}\x1b[K\x1b[0m", bg(REMOVED_BG), fg(REMOVED_FG))),
                    Tag::Added => out.push_str(&format!("{}{}{row}\x1b[K\x1b[0m", bg(ADDED_BG), fg(ADDED_FG))),
                    Tag::Context => out.push_str(&format!("\x1b[2m{row}\x1b[0m")),
                }
            }
            out.push('\n');
        }
    }
    if total > shown {
        out.push_str(&format!("      … {} more lines\n", total - shown));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn diff_numbers_lines_and_counts() {
        let d = diff("a.rs", Some("one\ntwo\nthree\n"), "one\nTWO\nthree\nfour\n");
        assert_eq!((d.added, d.removed), (2, 1));
        let rows: Vec<String> = d.hunks[0].iter().map(|l| format!("{}{}", gutter(l), l.text)).collect();
        assert_eq!(rows, ["    1   one", "    2 - two", "    2 + TWO", "    3   three", "    4 + four"]);
        assert_eq!(d.summary(), "Added 2 lines, removed 1 line");
        assert_eq!(d.ext(), "rs");
    }

    #[test]
    fn far_apart_changes_make_separate_hunks() {
        let before: String = (1..=30).map(|i| format!("l{i}\n")).collect();
        let after = before.replace("l2\n", "x\n").replace("l28\n", "y\n");
        let d = diff("f", Some(&before), &after);
        assert_eq!(d.hunks.len(), 2);
        assert!(d.hunks[1].iter().any(|l| l.tag == Tag::Added && l.no == 28 && l.text == "y"));
    }

    #[test]
    fn new_file_is_all_added() {
        let d = diff("n.txt", None, "a\nb\n");
        assert_eq!((d.added, d.removed), (2, 0));
        assert_eq!(d.summary(), "Added 2 lines");
    }

    #[test]
    fn ansi_plain_output_is_capped() {
        let d = diff("n.txt", None, &"x\n".repeat(50));
        let s = ansi(&d, 5, false);
        assert!(s.starts_with("   ⎿ n.txt: Added 50 lines\n"), "{s}");
        assert!(s.contains("    5 + x") && !s.contains("    6 + x"), "{s}");
        assert!(s.contains("… 45 more lines"), "{s}");
    }

    #[test]
    fn tracker_diffs_edits_and_keeps_session_total() {
        let dir = std::env::temp_dir().join(format!("agentiloop-diff-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.txt");
        let mut t = Tracker::new(&dir);
        let call = |id: &str, name: &str| AgentEvent::ToolCall { id: id.into(), name: name.into(), input: json!({"path": "a.txt"}) };
        let ok = |id: &str| AgentEvent::ToolResult { id: id.into(), name: String::new(), output: String::new(), is_error: false };

        assert!(t.observe(&call("1", "write_file")).is_none());
        std::fs::write(&file, "a\nb\n").unwrap();
        let c = t.observe(&ok("1")).unwrap();
        assert_eq!((c.edit.path.as_str(), c.edit.added, c.edit.removed), ("a.txt", 2, 0));

        t.observe(&call("2", "edit_file"));
        std::fs::write(&file, "a\nB\n").unwrap();
        let c = t.observe(&ok("2")).unwrap();
        assert_eq!((c.edit.added, c.edit.removed), (1, 1));
        // Total is measured from before the first write (file didn't exist).
        assert_eq!((c.total.added, c.total.removed), (2, 0));

        // Failed calls and other tools produce nothing.
        t.observe(&call("3", "edit_file"));
        assert!(t.observe(&AgentEvent::ToolResult { id: "3".into(), name: String::new(), output: String::new(), is_error: true }).is_none());
        assert!(t.observe(&call("4", "read_file")).is_none());
        assert!(t.observe(&ok("4")).is_none());
        std::fs::remove_dir_all(dir).ok();
    }
}
