//! Hooks: shell commands the user runs at fixed points of the agent loop, configured in
//! `<project>/.agentiloop/hooks.json` and `~/.agentiloop/hooks.json` (both are loaded):
//!
//! ```json
//! {
//!   "hooks": {
//!     "PreToolUse":  [{ "matcher": "bash,write_file", "command": "sh .agentiloop/check.sh" }],
//!     "PostToolUse": [{ "matcher": "edit_file", "command": "cargo fmt" }],
//!     "Stop":        [{ "command": "say done" }]
//!   }
//! }
//! ```
//!
//! `matcher` uses the `--allow-tool` syntax (names, `mcp_*` prefixes, `*`); missing = every tool.
//! Each command runs through `sh -c` (`cmd /C` on Windows) in the project folder with one JSON
//! object on stdin (`event`, `cwd`, `tool`, `input`, and for PostToolUse `output` + `is_error`;
//! for Stop `stop_reason`). `AGENTILOOP_HOOK_EVENT` and `AGENTILOOP_TOOL` are set too.
//!
//! Exit code 2 means "talk to the model": a PreToolUse hook blocks the call and its stderr becomes
//! the tool error; a PostToolUse hook's stderr is appended to the tool result. Any other exit code
//! is ignored (logged), and a hook that fails to start or exceeds `TIMEOUT` is logged and skipped.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use serde::Deserialize;
use serde_json::{json, Value};
use tokio::io::AsyncWriteExt;

use crate::permission::ToolPatterns;

/// A hook that runs longer than this is killed and ignored.
pub const TIMEOUT: Duration = Duration::from_secs(60);
/// Tool output longer than this is cut before it is handed to a PostToolUse hook.
const OUTPUT_LIMIT: usize = 20_000;
/// Hook stderr/stdout longer than this is cut before it reaches the model.
const FEEDBACK_LIMIT: usize = 4_000;
/// Exit code that makes a hook's stderr visible to the model.
pub const TALK_EXIT_CODE: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    Stop,
}

impl HookEvent {
    pub fn name(self) -> &'static str {
        match self {
            HookEvent::PreToolUse => "PreToolUse",
            HookEvent::PostToolUse => "PostToolUse",
            HookEvent::Stop => "Stop",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "PreToolUse" => Some(HookEvent::PreToolUse),
            "PostToolUse" => Some(HookEvent::PostToolUse),
            "Stop" => Some(HookEvent::Stop),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hook {
    pub event: HookEvent,
    /// Which tools it applies to (ignored for Stop). Empty = all.
    pub matcher: ToolPatterns,
    pub command: String,
    /// The file it came from.
    pub source: PathBuf,
}

impl Hook {
    fn applies(&self, event: HookEvent, tool: &str) -> bool {
        self.event == event && (event == HookEvent::Stop || self.matcher.is_empty() || self.matcher.matches(tool))
    }
}

/// What one hook said back, for callers that render it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Feedback {
    pub command: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Hooks(Vec<Hook>);

#[derive(Deserialize)]
struct HooksFile {
    #[serde(default)]
    hooks: std::collections::BTreeMap<String, Vec<HookEntry>>,
}

#[derive(Deserialize)]
struct HookEntry {
    #[serde(default)]
    matcher: String,
    command: String,
}

/// The file names searched, user first so project hooks run last.
pub fn default_paths(cwd: &Path, home: Option<&Path>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(h) = home {
        out.push(h.join("hooks.json"));
    }
    out.push(cwd.join(".agentiloop").join("hooks.json"));
    out
}

impl Hooks {
    /// Parse the given files (missing ones are skipped). Problems are returned as messages, one
    /// per file or entry, and never stop the rest from loading.
    pub fn load(paths: &[PathBuf]) -> (Hooks, Vec<String>) {
        let mut hooks = Vec::new();
        let mut errors = Vec::new();
        for path in paths {
            let text = match std::fs::read_to_string(path) {
                Ok(t) => t,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => {
                    errors.push(format!("{}: {e}", path.display()));
                    continue;
                }
            };
            let file: HooksFile = match serde_json::from_str(&text) {
                Ok(f) => f,
                Err(e) => {
                    errors.push(format!("{}: {e}", path.display()));
                    continue;
                }
            };
            for (name, entries) in file.hooks {
                let Some(event) = HookEvent::parse(&name) else {
                    errors.push(format!("{}: unknown hook event `{name}` (PreToolUse, PostToolUse, Stop)", path.display()));
                    continue;
                };
                for e in entries {
                    if e.command.trim().is_empty() {
                        errors.push(format!("{}: {name} hook with an empty command", path.display()));
                        continue;
                    }
                    hooks.push(Hook { event, matcher: ToolPatterns::new(&[e.matcher]), command: e.command, source: path.clone() });
                }
            }
        }
        (Hooks(hooks), errors)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Hook> {
        self.0.iter()
    }

    /// Lines for `/hooks`.
    pub fn listing(&self) -> String {
        if self.0.is_empty() {
            return "no hooks configured (add .agentiloop/hooks.json or ~/.agentiloop/hooks.json)".into();
        }
        self.0
            .iter()
            .map(|h| {
                let m = if h.event == HookEvent::Stop { String::new() } else if h.matcher.is_empty() { "*".into() } else { h.matcher.to_string() };
                format!("{:<12} {:<20} {}  ({})", h.event.name(), m, h.command, h.source.display())
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Run the PreToolUse hooks for `tool`. `Err(reason)` when one blocked the call.
    pub async fn run_pre(&self, cwd: &Path, tool: &str, input: &Value) -> Result<(), Feedback> {
        for h in self.0.iter().filter(|h| h.applies(HookEvent::PreToolUse, tool)) {
            let payload = json!({ "event": "PreToolUse", "cwd": cwd, "tool": tool, "input": input });
            if let Some(message) = run(h, cwd, tool, &payload).await {
                return Err(Feedback { command: h.command.clone(), message });
            }
        }
        Ok(())
    }

    /// Run the PostToolUse hooks for `tool`; the messages of those that exited 2.
    pub async fn run_post(&self, cwd: &Path, tool: &str, input: &Value, output: &str, is_error: bool) -> Vec<Feedback> {
        let mut out = Vec::new();
        for h in self.0.iter().filter(|h| h.applies(HookEvent::PostToolUse, tool)) {
            let payload = json!({
                "event": "PostToolUse", "cwd": cwd, "tool": tool, "input": input,
                "output": cut(output, OUTPUT_LIMIT), "is_error": is_error,
            });
            if let Some(message) = run(h, cwd, tool, &payload).await {
                out.push(Feedback { command: h.command.clone(), message });
            }
        }
        out
    }

    /// Run the Stop hooks; what they say is ignored.
    pub async fn run_stop(&self, cwd: &Path, stop_reason: &str) {
        for h in self.0.iter().filter(|h| h.event == HookEvent::Stop) {
            let payload = json!({ "event": "Stop", "cwd": cwd, "stop_reason": stop_reason });
            run(h, cwd, "", &payload).await;
        }
    }
}

fn cut(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let at = s.floor_char_boundary(max);
    format!("{}…[{} more bytes]", &s[..at], s.len() - at)
}

/// Run one hook. `Some(message)` when it exited with `TALK_EXIT_CODE` (stderr, else stdout,
/// else a default line); `None` otherwise. Failures to start or finish are logged.
async fn run(h: &Hook, cwd: &Path, tool: &str, payload: &Value) -> Option<String> {
    let mut cmd = if cfg!(windows) {
        let mut c = tokio::process::Command::new("cmd");
        c.args(["/C", &h.command]);
        c
    } else {
        let mut c = tokio::process::Command::new("sh");
        c.args(["-c", &h.command]);
        c
    };
    cmd.current_dir(cwd)
        .env("AGENTILOOP_HOOK_EVENT", h.event.name())
        .env("AGENTILOOP_TOOL", tool)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("{} hook `{}` could not start: {e}", h.event.name(), h.command);
            return None;
        }
    };
    if let Some(mut stdin) = child.stdin.take() {
        let body = serde_json::to_vec(payload).unwrap_or_default();
        // A hook that never reads stdin closes the pipe early; that is fine.
        let _ = stdin.write_all(&body).await;
        drop(stdin);
    }
    let out = match tokio::time::timeout(TIMEOUT, child.wait_with_output()).await {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => {
            tracing::warn!("{} hook `{}` failed: {e}", h.event.name(), h.command);
            return None;
        }
        Err(_) => {
            tracing::warn!("{} hook `{}` timed out after {}s", h.event.name(), h.command, TIMEOUT.as_secs());
            return None;
        }
    };
    let code = out.status.code();
    if code == Some(TALK_EXIT_CODE) {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let msg = if !stderr.is_empty() { stderr } else if !stdout.is_empty() { stdout } else { format!("hook `{}` exited 2", h.command) };
        return Some(cut(&msg, FEEDBACK_LIMIT));
    }
    if !out.status.success() {
        tracing::warn!("{} hook `{}` exited with {:?} (ignored): {}", h.event.name(), h.command, code, String::from_utf8_lossy(&out.stderr).trim());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("agentiloop-hooks-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".agentiloop")).unwrap();
        dir
    }

    #[test]
    fn loads_user_then_project_and_reports_bad_entries() {
        let dir = tmp("load");
        let home = dir.join("home");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(home.join("hooks.json"), r#"{"hooks":{"Stop":[{"command":"echo done"}]}}"#).unwrap();
        std::fs::write(
            dir.join(".agentiloop/hooks.json"),
            r#"{"hooks":{"PreToolUse":[{"matcher":"bash, mcp_*","command":"check"},{"command":""}],"Weird":[{"command":"x"}]}}"#,
        )
        .unwrap();
        let (hooks, errors) = Hooks::load(&default_paths(&dir, Some(&home)));
        assert_eq!(hooks.len(), 2);
        assert_eq!(hooks.0[0].event, HookEvent::Stop);
        assert_eq!(hooks.0[1].event, HookEvent::PreToolUse);
        assert!(hooks.0[1].applies(HookEvent::PreToolUse, "bash") && hooks.0[1].applies(HookEvent::PreToolUse, "mcp_x_y"));
        assert!(!hooks.0[1].applies(HookEvent::PreToolUse, "edit_file") && !hooks.0[1].applies(HookEvent::PostToolUse, "bash"));
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert!(errors[0].contains("empty command") && errors[1].contains("unknown hook event `Weird`"));
        assert!(hooks.listing().contains("PreToolUse   bash,mcp_*"), "{}", hooks.listing());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_files_and_broken_json() {
        let dir = tmp("missing");
        let (hooks, errors) = Hooks::load(&default_paths(&dir, None));
        assert!(hooks.is_empty() && errors.is_empty());
        assert!(hooks.listing().starts_with("no hooks configured"));
        std::fs::write(dir.join(".agentiloop/hooks.json"), "{not json").unwrap();
        let (hooks, errors) = Hooks::load(&default_paths(&dir, None));
        assert!(hooks.is_empty() && errors.len() == 1 && errors[0].contains("hooks.json"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn pre_hook_blocks_on_exit_2_and_reads_the_payload() {
        let dir = tmp("pre");
        std::fs::write(
            dir.join(".agentiloop/hooks.json"),
            r#"{"hooks":{"PreToolUse":[
                {"matcher":"bash","command":"grep -q rm && { echo \"no rm, tool=$AGENTILOOP_TOOL\" >&2; exit 2; }; exit 0"},
                {"matcher":"edit_file","command":"exit 1"}
            ]}}"#,
        )
        .unwrap();
        let (hooks, errors) = Hooks::load(&default_paths(&dir, None));
        assert!(errors.is_empty(), "{errors:?}");
        let err = hooks.run_pre(&dir, "bash", &json!({"command": "rm -rf x"})).await.unwrap_err();
        assert_eq!(err.message, "no rm, tool=bash");
        assert!(hooks.run_pre(&dir, "bash", &json!({"command": "ls"})).await.is_ok());
        // Exit 1 and unmatched tools never block.
        assert!(hooks.run_pre(&dir, "edit_file", &json!({})).await.is_ok());
        assert!(hooks.run_pre(&dir, "write_file", &json!({})).await.is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn post_hook_feedback_and_stop_hook_side_effect() {
        let dir = tmp("post");
        std::fs::write(
            dir.join(".agentiloop/hooks.json"),
            r#"{"hooks":{
                "PostToolUse":[{"command":"grep -q '\"is_error\":true' && exit 0; echo remember-to-test; exit 2"}],
                "Stop":[{"command":"cat > stop.json"}]
            }}"#,
        )
        .unwrap();
        let (hooks, _) = Hooks::load(&default_paths(&dir, None));
        let fb = hooks.run_post(&dir, "edit_file", &json!({}), "ok", false).await;
        assert_eq!(fb.len(), 1);
        assert_eq!(fb[0].message, "remember-to-test");
        assert!(hooks.run_post(&dir, "edit_file", &json!({}), "boom", true).await.is_empty());
        hooks.run_stop(&dir, "end_turn").await;
        let stop: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("stop.json")).unwrap()).unwrap();
        assert_eq!(stop["event"], "Stop");
        assert_eq!(stop["stop_reason"], "end_turn");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cut_keeps_char_boundaries() {
        assert_eq!(cut("héllo", 100), "héllo");
        assert!(cut("héllo", 2).starts_with("h…"));
    }
}
