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

fn home() -> Option<PathBuf> {
    match std::env::var_os("AGENTILOOP_HOME") {
        Some(h) => Some(PathBuf::from(h)),
        None => Some(dirs::home_dir()?.join(".agentiloop")),
    }
}

pub fn path() -> Option<PathBuf> {
    Some(home()?.join("settings.json"))
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
