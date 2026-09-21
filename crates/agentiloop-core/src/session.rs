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
