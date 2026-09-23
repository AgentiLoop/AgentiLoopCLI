//! `mcpServers` config files — the same JSON shape Agent!, Claude Code and
//! Claude Desktop use:
//!
//! ```json
//! { "mcpServers": {
//!     "HelloWorld": { "command": "mcp-server-hello", "args": [], "env": {} },
//!     "DemoHttp":   { "transport": "http", "url": "http://localhost:8085/mcp", "headers": {} }
//! } }
//! ```

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ServerConfig {
    /// `stdio`, `http` / `streamable-http`, or `sse` (legacy). Inferred when absent.
    #[serde(alias = "type")]
    pub transport: Option<String>,
    // stdio
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    // HTTP
    pub url: Option<String>,
    pub headers: HashMap<String, String>,
    /// Non-empty forces the legacy HTTP+SSE transport (as in AgentMCP).
    pub sse_endpoint: Option<String>,
    /// Path appended to `url` for Streamable HTTP POSTs.
    pub http_endpoint: Option<String>,
    // Agent! metadata / Claude-style flag
    pub enabled: Option<bool>,
    pub auto_start: Option<bool>,
    pub disabled: bool,
}

impl ServerConfig {
    pub fn is_http(&self) -> bool {
        self.url.as_deref().is_some_and(|u| !u.is_empty())
    }

    pub fn should_start(&self) -> bool {
        self.enabled.unwrap_or(true) && self.auto_start.unwrap_or(true) && !self.disabled
    }
}

#[derive(Deserialize)]
struct ConfigFile {
    #[serde(rename = "mcpServers", default)]
    mcp_servers: BTreeMap<String, ServerConfig>,
}

/// Standard config locations, lowest precedence first: the user file, then
/// the project's `.mcp.json` (same name overrides).
pub fn default_paths(user_file: Option<PathBuf>, cwd: &Path) -> Vec<PathBuf> {
    user_file.into_iter().chain([cwd.join(".mcp.json")]).collect()
}

/// Merge every existing file. Unparsable files are reported, not fatal.
pub fn load(paths: &[PathBuf]) -> (BTreeMap<String, ServerConfig>, Vec<String>) {
    let mut servers = BTreeMap::new();
    let mut errors = Vec::new();
    for path in paths {
        let Ok(text) = std::fs::read_to_string(path) else { continue };
        match serde_json::from_str::<ConfigFile>(&text) {
            Ok(file) => servers.extend(file.mcp_servers),
            Err(e) => errors.push(format!("{}: {e}", path.display())),
        }
    }
    (servers, errors)
}

/// Expand `${VAR}` / `${VAR:-default}` from the environment, so secrets can
/// stay out of the config file.
pub fn expand_env(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find('}') else {
            out.push_str(&rest[start..]);
            return out;
        };
        let expr = &after[..end];
        let (var, default) = expr.split_once(":-").unwrap_or((expr, ""));
        out.push_str(&std::env::var(var).unwrap_or_else(|_| default.to_string()));
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_agent_and_claude_shapes() {
        let file: ConfigFile = serde_json::from_str(
            r#"{"mcpServers":{
                "Hello":{"transport":"stdio","command":"/bin/hello","args":["-v"],"env":{}},
                "Demo":{"type":"http","url":"http://localhost:8085/mcp","headers":{}},
                "Off":{"command":"x","disabled":true},
                "Off2":{"command":"x","enabled":false}
            }}"#,
        )
        .unwrap();
        let s = &file.mcp_servers;
        assert!(!s["Hello"].is_http() && s["Hello"].args == ["-v"]);
        assert!(s["Demo"].is_http() && s["Demo"].transport.as_deref() == Some("http"));
        assert!(!s["Off"].should_start() && !s["Off2"].should_start() && s["Hello"].should_start());
    }

    #[test]
    fn expands_env_vars() {
        std::env::set_var("AGENTILOOP_MCP_TEST", "tok");
        assert_eq!(expand_env("Bearer ${AGENTILOOP_MCP_TEST}"), "Bearer tok");
        assert_eq!(expand_env("${AGENTILOOP_MCP_NOPE:-x}/y"), "x/y");
        assert_eq!(expand_env("plain ${oops"), "plain ${oops");
    }
}
