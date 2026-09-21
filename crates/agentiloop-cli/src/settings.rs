//! Persistent user settings at `~/.agentiloop/settings.json` (JSON, like
//! Claude Code's `~/.claude/settings.json`). Override the location with
//! `AGENTILOOP_HOME`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Last model selected via `/model`; used when `--model` / `AGENTILOOP_MODEL` are absent.
    pub model: Option<String>,
}

pub fn path() -> Option<PathBuf> {
    let home = match std::env::var_os("AGENTILOOP_HOME") {
        Some(h) => PathBuf::from(h),
        None => dirs::home_dir()?.join(".agentiloop"),
    };
    Some(home.join("settings.json"))
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
