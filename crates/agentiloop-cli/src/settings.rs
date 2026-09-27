//! Persistent user settings at `~/.agentiloop/settings.json` (JSON, like
//! Claude Code's `~/.claude/settings.json`). Override the location with
//! `AGENTILOOP_HOME`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Last model selected via `/model` (legacy, Anthropic-only); superseded by `models`.
    pub model: Option<String>,
    /// Last model selected via `/model`, per provider name.
    pub models: BTreeMap<String, String>,
    /// Options from the last interactive launch, reused when not given on the command line.
    pub last: LastLaunch,
    /// What the first-run wizard wrote outside `~/.agentiloop`, so `--reset` can undo exactly that.
    pub setup: Setup,
}

/// Record of the first-run wizard (`agentiloop --setup`).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Setup {
    /// RFC 3339 timestamp; `None` until the wizard has completed.
    pub completed_at: Option<String>,
    /// Shell profile that holds the `# >>> agentiloop >>>` block, if one was written.
    pub profile: Option<PathBuf>,
    /// macOS Keychain items (service names) the wizard created.
    pub keychain: Vec<String>,
    /// Windows user environment variables (`setx`) the wizard created.
    pub user_env: Vec<String>,
}

/// Remembered launch options. `--yes` and `--no-mcp` are deliberately never remembered.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LastLaunch {
    pub provider: Option<String>,
    pub tui: bool,
    pub max_turns: Option<usize>,
    pub compact_at: Option<u64>,
}

impl Settings {
    pub fn model_for(&self, provider: &str) -> Option<&str> {
        self.models
            .get(provider)
            .map(String::as_str)
            .or_else(|| (provider == "anthropic").then_some(self.model.as_deref()).flatten())
    }

    pub fn set_model(&mut self, provider: &str, model: &str) {
        self.models.insert(provider.to_string(), model.to_string());
        if provider == "anthropic" {
            self.model = Some(model.to_string());
        }
    }
}

pub fn home() -> Option<PathBuf> {
    match std::env::var_os("AGENTILOOP_HOME") {
        Some(h) => Some(PathBuf::from(h)),
        None => Some(dirs::home_dir()?.join(".agentiloop")),
    }
}

pub fn path() -> Option<PathBuf> {
    Some(home()?.join("settings.json"))
}

/// Credentials written by the first-run wizard, one `KEY=value` per line (mode 0600).
pub fn env_path() -> Option<PathBuf> {
    Some(home()?.join("env"))
}

/// Loads `~/.agentiloop/env` into the process environment. Variables that are
/// already set (and non-empty) win, so a shell `export` always overrides the file.
pub fn load_env_file() {
    let Some(p) = env_path() else { return };
    let Ok(text) = std::fs::read_to_string(&p) else { return };
    for (key, value) in parse_env_file(&text) {
        if std::env::var_os(&key).map_or(true, |v| v.is_empty()) {
            std::env::set_var(&key, &value);
        }
    }
}

fn parse_env_file(text: &str) -> Vec<(String, String)> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.strip_prefix("export ").unwrap_or(l).split_once('='))
        .map(|(k, v)| (k.trim().to_string(), v.trim().trim_matches('"').to_string()))
        .filter(|(k, _)| !k.is_empty())
        .collect()
}

/// Writes `~/.agentiloop/env` (replacing it) with the given variables.
pub fn save_env_file(vars: &[(String, String)]) -> anyhow::Result<PathBuf> {
    let Some(p) = env_path() else { anyhow::bail!("no home directory") };
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut out = String::from("# Written by `agentiloop --setup`. Delete with `agentiloop --reset`.\n");
    for (k, v) in vars {
        out.push_str(&format!("{k}={v}\n"));
    }
    std::fs::write(&p, out)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(p)
}

/// REPL prompt history (one entry per line), used for up/down arrow recall.
pub fn history_path() -> Option<PathBuf> {
    Some(home()?.join("history.txt"))
}

/// Saved conversations, one JSON file per session.
pub fn sessions_dir() -> Option<PathBuf> {
    Some(home()?.join("sessions"))
}

/// User-level MCP servers (`mcpServers` JSON); the project's `.mcp.json` is merged on top.
pub fn mcp_config_path() -> Option<PathBuf> {
    Some(home()?.join("mcp.json"))
}

pub fn load() -> Settings {
    let Some(p) = path() else { return Settings::default() };
    match std::fs::read_to_string(&p) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_else(|e| {
            tracing::warn!("ignoring malformed {}: {e}", p.display());
            Settings::default()
        }),
        Err(_) => Settings::default(),
    }
}

pub fn save(settings: &Settings) -> anyhow::Result<()> {
    let Some(p) = path() else { anyhow::bail!("no home directory") };
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&p, serde_json::to_string_pretty(settings)? + "\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_file_parses_comments_export_and_quotes() {
        let vars = parse_env_file("# c\n\nexport A=\"x y\"\nB = 2\n=nope\nC\n");
        assert_eq!(vars, vec![("A".into(), "x y".into()), ("B".into(), "2".into())]);
    }

    #[test]
    fn setup_record_round_trips_and_defaults() {
        let s: Settings = serde_json::from_str(r#"{"model":"m"}"#).unwrap();
        assert!(s.setup.completed_at.is_none() && s.setup.keychain.is_empty());
        let mut s = Settings::default();
        s.setup = Setup { completed_at: Some("t".into()), profile: Some("/p".into()), keychain: vec!["K".into()], user_env: vec![] };
        let back: Settings = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(back.setup.profile.as_deref(), Some(std::path::Path::new("/p")));
        assert_eq!(back.setup.keychain, vec!["K"]);
    }
}
