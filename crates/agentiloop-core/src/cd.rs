//! `change_dir`: the model switches the session's working directory (the agent applies it
//! after a successful call); `/cd` in the CLI uses the same `resolve_dir`.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::tool::{Tool, ToolContext, ToolError, ToolResult};

pub const CHANGE_DIR_TOOL_NAME: &str = "change_dir";

/// `path` as an existing directory: `~` is the home folder, a relative path is taken from `cwd`.
pub fn resolve_dir(cwd: &Path, path: &str) -> Result<PathBuf, String> {
    let path = path.trim();
    if path.is_empty() {
        return Err("no directory given".into());
    }
    let expanded = match (path.strip_prefix('~'), dirs::home_dir()) {
        (Some(""), Some(h)) => h,
        (Some(rest), Some(h)) if rest.starts_with(['/', '\\']) => h.join(&rest[1..]),
        _ => PathBuf::from(path),
    };
    let full = if expanded.is_absolute() { expanded } else { cwd.join(expanded) };
    match full.canonicalize() {
        Ok(p) if p.is_dir() => Ok(p),
        Ok(p) => Err(format!("{} is not a directory", p.display())),
        Err(e) => Err(format!("{}: {e}", full.display())),
    }
}

pub struct ChangeDir;

#[async_trait]
impl Tool for ChangeDir {
    fn name(&self) -> &str {
        CHANGE_DIR_TOOL_NAME
    }
    fn description(&self) -> &str {
        "Change the working directory for the rest of the session: later tool calls, relative paths and shell commands run there. Use it when the user asks to switch to another project or folder."
    }
    fn input_schema(&self) -> Value {
        json!({"type":"object","properties":{"path":{"type":"string","description":"Directory (absolute, relative to cwd, or starting with ~)"}},"required":["path"]})
    }
    async fn call(&self, ctx: &ToolContext, input: Value) -> ToolResult {
        let path = input.get("path").and_then(Value::as_str).ok_or_else(|| ToolError::InvalidInput("missing `path`".into()))?;
        let dir = resolve_dir(&ctx.cwd, path).map_err(ToolError::Failed)?;
        Ok(format!("working directory is now {}", dir.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_relative_home_and_rejects_files() {
        let tmp = std::env::temp_dir().canonicalize().unwrap();
        let sub = tmp.join("agentiloop-cd-test");
        std::fs::create_dir_all(&sub).unwrap();
        assert_eq!(resolve_dir(&tmp, "agentiloop-cd-test").unwrap(), sub);
        assert_eq!(resolve_dir(&sub, "..").unwrap(), tmp);
        let home = dirs::home_dir().unwrap().canonicalize().unwrap();
        assert_eq!(resolve_dir(&sub, "~").unwrap(), home);
        let file = sub.join("f.txt");
        std::fs::write(&file, "x").unwrap();
        assert!(resolve_dir(&sub, "f.txt").unwrap_err().contains("not a directory"));
        assert!(resolve_dir(&sub, "missing").is_err());
    }
}
