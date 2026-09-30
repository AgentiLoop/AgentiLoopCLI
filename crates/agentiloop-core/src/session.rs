//! Conversation persistence: one JSON file per session so a run can be resumed.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::message::{ContentBlock, Message, Role};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    /// Unix seconds.
    pub created: u64,
    pub updated: u64,
    pub cwd: PathBuf,
    pub provider: String,
    pub model: String,
    pub history: Vec<Message>,
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Tool output longer than this is cut in an export.
const EXPORT_RESULT_CHARS: usize = 2000;

/// A fenced code block whose fence is longer than any backtick run inside `body`.
fn fenced(lang: &str, body: &str) -> String {
    let (mut longest, mut run) = (0, 0);
    for c in body.chars() {
        run = if c == '`' { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    let fence = "`".repeat((longest + 1).max(3));
    format!("{fence}{lang}\n{body}\n{fence}")
}

impl Session {
    pub fn new(cwd: impl Into<PathBuf>, provider: impl Into<String>, model: impl Into<String>) -> Self {
        let created = now();
        Self {
            id: format!("{created}-{}", std::process::id()),
            created,
            updated: created,
            cwd: cwd.into(),
            provider: provider.into(),
            model: model.into(),
            history: Vec::new(),
        }
    }

    /// First user prompt, trimmed to one line of at most 60 chars.
    pub fn title(&self) -> String {
        let first = self
            .history
            .iter()
            .filter(|m| m.role == Role::User)
            .flat_map(|m| m.content.iter())
            .find_map(|b| match b {
                ContentBlock::Text { text } => Some(text.lines().next().unwrap_or("").trim()),
                _ => None,
            })
            .unwrap_or("");
        let mut t: String = first.chars().take(60).collect();
        if first.chars().count() > 60 {
            t.push('…');
        }
        t
    }

    /// The conversation as Markdown: a header, then `## You` / `## AgentiLoop` sections with tool calls
    /// and (cut) results in code blocks.
    pub fn to_markdown(&self) -> String {
        let title = self.title();
        let mut out = format!(
            "# {}\n\n- Session: {}\n- Provider: {} · Model: {}\n- Directory: {}\n",
            if title.is_empty() { "AgentiLoop session" } else { &title },
            self.id,
            self.provider,
            self.model,
            self.cwd.display()
        );
        let mut section: Option<Role> = None;
        for m in &self.history {
            let has_text = m.content.iter().any(|b| matches!(b, ContentBlock::Text { .. }));
            if has_text && section != Some(m.role) {
                out.push_str(if m.role == Role::User { "\n## You\n" } else { "\n## AgentiLoop\n" });
                section = Some(m.role);
            }
            for b in &m.content {
                match b {
                    ContentBlock::Text { text } => out.push_str(&format!("\n{}\n", text.trim_end())),
                    ContentBlock::ToolUse { name, input, .. } => {
                        let input = serde_json::to_string(input).unwrap_or_default();
                        out.push_str(&format!("\n**Tool: `{name}`**\n\n{}\n", fenced("json", &input)));
                    }
                    ContentBlock::ToolResult { content, is_error, .. } => {
                        let mut body: String = content.chars().take(EXPORT_RESULT_CHARS).collect();
                        if content.chars().count() > EXPORT_RESULT_CHARS {
                            body.push_str("\n…[truncated]");
                        }
                        let label = if *is_error { "Error" } else { "Result" };
                        out.push_str(&format!("\n**{label}**\n\n{}\n", fenced("", &body)));
                    }
                }
            }
        }
        out
    }

    pub fn path(dir: &Path, id: &str) -> PathBuf {
        dir.join(format!("{id}.json"))
    }

    /// Write `<dir>/<id>.json` (via a temp file + rename so a crash never
    /// leaves a half-written session) and bump `updated`.
    pub fn save(&mut self, dir: &Path) -> anyhow::Result<PathBuf> {
        self.updated = now();
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        let path = Self::path(dir, &self.id);
        let tmp = dir.join(format!(".{}.json.tmp", self.id));
        std::fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(&tmp, &path).with_context(|| format!("writing {}", path.display()))?;
        Ok(path)
    }

    pub fn load(dir: &Path, id: &str) -> anyhow::Result<Self> {
        let path = Self::path(dir, id);
        let text = std::fs::read_to_string(&path).with_context(|| format!("no session {id} in {}", dir.display()))?;
        serde_json::from_str(&text).with_context(|| format!("decoding {}", path.display()))
    }

    /// All sessions in `dir`, most recently updated first. Malformed files are skipped.
    pub fn list(dir: &Path) -> anyhow::Result<Vec<Self>> {
        let mut out = Vec::new();
        let rd = match std::fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
            Err(e) => return Err(e.into()),
        };
        for entry in rd {
            let path = entry?.path();
            let is_json = path.extension().is_some_and(|e| e == "json");
            let hidden = path.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with('.'));
            if !is_json || hidden {
                continue;
            }
            match std::fs::read_to_string(&path).map_err(anyhow::Error::from).and_then(|t| Ok(serde_json::from_str(&t)?)) {
                Ok(s) => out.push(s),
                Err(e) => tracing::warn!("skipping {}: {e:#}", path.display()),
            }
        }
        out.sort_by(|a, b| b.updated.cmp(&a.updated).then_with(|| b.id.cmp(&a.id)));
        Ok(out)
    }

    /// Most recently updated session whose `cwd` matches.
    pub fn latest_for(dir: &Path, cwd: &Path) -> anyhow::Result<Option<Self>> {
        Ok(Self::list(dir)?.into_iter().find(|s| s.cwd == cwd))
    }
}

#[cfg(test)]
mod export_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn markdown_export_has_sections_tools_and_safe_fences() {
        let mut s = Session::new("/work/app", "anthropic", "m1");
        s.id = "S1".into();
        s.history = vec![
            Message::user_text("Fix the bug\nplease"),
            Message {
                role: Role::Assistant,
                content: vec![
                    ContentBlock::Text { text: "Looking.".into() },
                    ContentBlock::ToolUse { id: "t1".into(), name: "read_file".into(), input: json!({"path": "a.rs"}) },
                ],
            },
            Message::tool_results(vec![ContentBlock::ToolResult {
                tool_use_id: "t1".into(),
                content: "code with ``` inside".into(),
                is_error: false,
            }]),
            Message {
                role: Role::Assistant,
                content: vec![ContentBlock::ToolUse { id: "t2".into(), name: "bash".into(), input: json!({"command": "x"}) }],
            },
            Message::tool_results(vec![ContentBlock::ToolResult { tool_use_id: "t2".into(), content: "y".repeat(2500), is_error: true }]),
            Message { role: Role::Assistant, content: vec![ContentBlock::Text { text: "Done.".into() }] },
        ];
        let md = s.to_markdown();
        let want_head = "# Fix the bug\n\n- Session: S1\n- Provider: anthropic · Model: m1\n- Directory: /work/app\n\n## You\n\nFix the bug\nplease\n\n## AgentiLoop\n\nLooking.\n\n**Tool: `read_file`**\n\n```json\n{\"path\":\"a.rs\"}\n```\n\n**Result**\n\n````\ncode with ``` inside\n````\n";
        assert!(md.starts_with(want_head), "{md}");
        assert_eq!(md.matches("## AgentiLoop").count(), 1, "section header is not repeated");
        assert!(md.contains("**Error**") && md.contains(&format!("{}\n…[truncated]", "y".repeat(2000))), "{md}");
        assert!(md.ends_with("\nDone.\n"), "{md}");
        assert!(Session::new("/x", "p", "m").to_markdown().starts_with("# AgentiLoop session\n"));
    }
}
