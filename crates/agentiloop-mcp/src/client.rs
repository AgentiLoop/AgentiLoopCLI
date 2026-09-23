//! MCP client: connect + initialize handshake, capability discovery, tool
//! calls and resource reads. Port of AgentMCP's `MCPClient`.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Url;
use serde_json::{json, Map, Value};

use crate::config::{expand_env, ServerConfig};
use crate::http::{HttpTransport, LegacySseTransport};
use crate::stdio::StdioTransport;
use crate::transport::Transport;

const PROTOCOL_VERSION: &str = "2024-11-05";
const INIT_TIMEOUT: Duration = Duration::from_secs(90);
const MAX_TEXT: usize = 1024 * 1024;
const MAX_IMAGE: usize = 10 * 1024 * 1024;
const MAX_BLOCKS: usize = 100;

/// Env vars a config may never inject into a server process.
const BLOCKED_ENV: &[&str] = &["LD_PRELOAD", "LD_LIBRARY_PATH"];

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    /// `annotations.readOnlyHint` — read-only tools skip the permission prompt.
    pub read_only: bool,
}

#[derive(Debug, Clone)]
pub struct ResourceInfo {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

pub struct McpServer {
    pub name: String,
    /// `serverInfo.name` / `version` reported by the server.
    pub server_info: String,
    pub transport_kind: &'static str,
    pub tools: Vec<ToolInfo>,
    pub resources: Vec<ResourceInfo>,
    conn: Arc<dyn Transport>,
}

impl McpServer {
    /// Launch / connect, then run `initialize` → `notifications/initialized`
    /// → `tools/list` / `resources/list`, all within 90 s.
    pub async fn connect(name: &str, cfg: &ServerConfig, cwd: &Path) -> Result<Self> {
        let (conn, kind): (Arc<dyn Transport>, &'static str) = if cfg.is_http() {
            let url = http_url(cfg)?;
            let headers: HashMap<String, String> = cfg.headers.iter().map(|(k, v)| (k.clone(), expand_env(v))).collect();
            let legacy = matches!(cfg.transport.as_deref(), Some("sse"))
                || url.path().trim_end_matches('/').ends_with("/sse")
                || cfg.sse_endpoint.as_deref().is_some_and(|s| !s.is_empty());
            if legacy {
                (Arc::new(LegacySseTransport::connect(url, &headers).await?), "sse")
            } else {
                (Arc::new(HttpTransport::new(url, &headers)?), "http")
            }
        } else {
            anyhow::ensure!(!cfg.command.is_empty(), "no `command` or `url` configured");
            let program = resolve_command(&expand_env(&cfg.command));
            let args: Vec<String> = cfg.args.iter().map(|a| expand_env(a)).collect();
            (Arc::new(StdioTransport::spawn(&program, &args, &server_env(&cfg.env), cwd)?), "stdio")
        };

        match tokio::time::timeout(INIT_TIMEOUT, handshake(name, conn.clone())).await {
            Ok(Ok((server_info, tools, resources))) => {
                Ok(Self { name: name.to_string(), server_info, transport_kind: kind, tools, resources, conn })
            }
            Ok(Err(e)) => {
                conn.close().await;
                Err(e)
            }
            Err(_) => {
                conn.close().await;
                anyhow::bail!("initialization timed out after 90 seconds")
            }
        }
    }

    pub fn is_alive(&self) -> bool {
        self.conn.is_alive()
    }

    pub async fn close(&self) {
        self.conn.close().await;
    }

    /// `tools/call`. Returns the flattened text output and the `isError` flag;
    /// a JSON-RPC error is reported as a tool error, not a transport failure.
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<(String, bool)> {
        anyhow::ensure!(self.conn.is_alive(), "MCP server `{}` is no longer running", self.name);
        let resp = self.conn.request("tools/call", Some(json!({ "name": name, "arguments": arguments }))).await?;
        if let Some(err) = resp.get("error") {
            let msg = err.get("message").and_then(Value::as_str).unwrap_or("Unknown error");
            return Ok((msg.replace('\n', " ").chars().take(512).collect(), true));
        }
        let result = resp.get("result").context("invalid tools/call response")?;
        let is_error = result.get("isError").and_then(Value::as_bool).unwrap_or(false);
        Ok((format_content(result), is_error))
    }

    /// `resources/read` — text of the first content item.
    pub async fn read_resource(&self, uri: &str) -> Result<String> {
        let resp = self.conn.request("resources/read", Some(json!({ "uri": uri }))).await?;
        let first = resp
            .pointer("/result/contents/0")
            .with_context(|| format!("resource not found: {uri}"))?;
        Ok(match first.get("text").and_then(Value::as_str) {
            Some(t) => t.to_string(),
            None => format!("[binary resource {uri}, {}]", first.get("mimeType").and_then(Value::as_str).unwrap_or("unknown type")),
        })
    }
}

async fn handshake(name: &str, conn: Arc<dyn Transport>) -> Result<(String, Vec<ToolInfo>, Vec<ResourceInfo>)> {
    let resp = conn
        .request(
            "initialize",
            Some(json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": { "name": "AgentiLoop", "version": env!("CARGO_PKG_VERSION") }
            })),
        )
        .await?;
    if let Some(msg) = resp.pointer("/error/message").and_then(Value::as_str) {
        anyhow::bail!("initialize failed: {msg}");
    }
    let result = resp.get("result").filter(|r| r.is_object()).context("invalid initialize response")?;
    let info = result.get("serverInfo");
    let server_info = match info.and_then(|i| i.get("name")).and_then(Value::as_str) {
        Some(n) => match info.and_then(|i| i.get("version")).and_then(Value::as_str) {
            Some(v) => format!("{n} {v}"),
            None => n.to_string(),
        },
        None => name.to_string(),
    };
    conn.notify("notifications/initialized", None).await?;

    let caps = result.get("capabilities");
    let has = |k: &str| caps.and_then(|c| c.get(k)).is_some_and(|v| !v.is_null());
    // Discovery failures leave the list empty rather than failing the server (as in AgentMCP).
    let tools = if has("tools") { list_tools(&*conn).await.unwrap_or_default() } else { Vec::new() };
    let resources = if has("resources") { list_resources(&*conn).await.unwrap_or_default() } else { Vec::new() };
    Ok((server_info, tools, resources))
}

/// Paged `list` call; `key` is the result array field.
async fn list_all(conn: &dyn Transport, method: &str, key: &str) -> Result<Vec<Value>> {
    let mut out = Vec::new();
    let mut cursor: Option<String> = None;
    for _ in 0..20 {
        let params = cursor.as_ref().map(|c| json!({ "cursor": c }));
        let resp = conn.request(method, params).await?;
        let result = resp.get("result").with_context(|| format!("invalid {method} response"))?;
        out.extend(result.get(key).and_then(Value::as_array).cloned().unwrap_or_default());
        cursor = result.get("nextCursor").and_then(Value::as_str).map(str::to_string);
        if cursor.is_none() {
            break;
        }
    }
    Ok(out)
}

async fn list_tools(conn: &dyn Transport) -> Result<Vec<ToolInfo>> {
    Ok(list_all(conn, "tools/list", "tools").await?.iter().filter_map(parse_tool).collect())
}

fn parse_tool(tool: &Value) -> Option<ToolInfo> {
    let name = tool.get("name")?.as_str()?;
    let valid = !name.is_empty()
        && name.len() <= 128
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if !valid {
        return None;
    }
    let mut schema = match tool.get("inputSchema") {
        Some(Value::Object(m)) => m.clone(),
        _ => Map::new(),
    };
    if !schema.get("properties").is_some_and(Value::is_object) {
        schema.insert("properties".into(), json!({}));
    }
    schema.entry("type").or_insert_with(|| json!("object"));
    let mut input_schema = Value::Object(schema);
    if serde_json::to_string(&input_schema).map_or(true, |s| s.len() > 100_000) {
        input_schema = json!({ "type": "object", "properties": {} });
    }
    Some(ToolInfo {
        name: name.to_string(),
        description: tool.get("description").and_then(Value::as_str).unwrap_or("").chars().take(2048).collect(),
        input_schema,
        read_only: tool.pointer("/annotations/readOnlyHint").and_then(Value::as_bool).unwrap_or(false),
    })
}

async fn list_resources(conn: &dyn Transport) -> Result<Vec<ResourceInfo>> {
    let s = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).map(str::to_string);
    Ok(list_all(conn, "resources/list", "resources")
        .await?
        .iter()
        .map(|r| ResourceInfo {
            uri: s(r, "uri").unwrap_or_default(),
            name: s(r, "name").unwrap_or_default(),
            description: s(r, "description"),
            mime_type: s(r, "mimeType"),
        })
        .collect())
}

/// Flatten a `tools/call` result's content blocks into text for the model.
pub(crate) fn format_content(result: &Value) -> String {
    let blocks = result.get("content").and_then(Value::as_array).cloned().unwrap_or_default();
    let str_of = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let mut parts: Vec<String> = Vec::new();
    for item in blocks.iter().take(MAX_BLOCKS) {
        let kind = item.get("type").and_then(Value::as_str).unwrap_or("text");
        let part = match kind {
            "text" => str_of(item, "text"),
            "image" | "audio" => {
                let data = str_of(item, "data");
                if data.len() > MAX_IMAGE {
                    format!("[{kind} too large: {} bytes]", data.len())
                } else {
                    format!("[{kind}: {}, {} bytes base64]", str_of(item, "mimeType"), data.len())
                }
            }
            "resource" => {
                let r = item.get("resource").cloned().unwrap_or_default();
                match r.get("text").and_then(Value::as_str) {
                    Some(t) => t.to_string(),
                    None => format!("[resource: {}]", str_of(&r, "uri").chars().take(2048).collect::<String>()),
                }
            }
            "resource_link" => format!("[resource link: {}]", str_of(item, "uri")),
            other => item.get("text").and_then(Value::as_str).map_or_else(|| format!("[{other}]"), str::to_string),
        };
        parts.push(part.chars().take(MAX_TEXT).collect());
    }
    if parts.is_empty() {
        if let Some(sc) = result.get("structuredContent") {
            return sc.to_string();
        }
    }
    parts.join("\n")
}

fn http_url(cfg: &ServerConfig) -> Result<Url> {
    let raw = expand_env(cfg.url.as_deref().unwrap_or(""));
    let mut url = Url::parse(&raw).with_context(|| format!("invalid URL: {raw}"))?;
    match url.scheme() {
        "https" => {}
        "http" => {
            let host = url.host_str().unwrap_or("").trim_matches(['[', ']']).to_ascii_lowercase();
            anyhow::ensure!(
                matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1"),
                "plain HTTP is only allowed for localhost; use HTTPS for remote servers"
            );
        }
        other => anyhow::bail!("only HTTP/HTTPS URLs are supported, got: {other}"),
    }
    if let Some(ep) = cfg.http_endpoint.as_deref().filter(|e| !e.is_empty()) {
        url.path_segments_mut()
            .map_err(|_| anyhow::anyhow!("URL cannot take a path: {raw}"))?
            .pop_if_empty()
            .extend(ep.trim_matches('/').split('/'));
    }
    Ok(url)
}

/// Extra entries prepended to PATH so servers launched via npx/uvx/brew are found.
fn extra_path_dirs() -> Vec<String> {
    let home = dirs::home_dir().map(|h| h.display().to_string()).unwrap_or_default();
    if cfg!(windows) {
        return Vec::new();
    }
    [
        format!("{home}/.local/bin"),
        "/opt/homebrew/bin".into(),
        "/usr/local/bin".into(),
        format!("{home}/.cargo/bin"),
        format!("{home}/.nvm/current/bin"),
        "/usr/bin".into(),
        "/bin".into(),
    ]
    .into()
}

/// Resolve a bare command name (e.g. `uvx`) against common tool dirs, then PATH.
fn resolve_command(command: &str) -> String {
    if command.contains('/') || command.contains('\\') {
        return command.to_string();
    }
    let path = std::env::var_os("PATH").unwrap_or_default();
    extra_path_dirs()
        .into_iter()
        .map(std::path::PathBuf::from)
        .chain(std::env::split_paths(&path))
        .map(|d| d.join(command))
        .find(|p| p.is_file())
        .map_or_else(|| command.to_string(), |p| p.display().to_string())
}

/// Config env layered over the inherited environment, with PATH widened and
/// library-injection variables refused.
fn server_env(cfg_env: &HashMap<String, String>) -> HashMap<String, String> {
    let mut env: HashMap<String, String> = cfg_env
        .iter()
        .filter(|(k, _)| {
            let up = k.to_ascii_uppercase();
            !up.starts_with("DYLD_") && !BLOCKED_ENV.contains(&up.as_str())
        })
        .map(|(k, v)| (k.clone(), expand_env(v)))
        .collect();
    if !env.contains_key("PATH") {
        let mut dirs = extra_path_dirs();
        if let Ok(p) = std::env::var("PATH") {
            dirs.push(p);
        }
        if !dirs.is_empty() {
            env.insert("PATH".into(), dirs.join(if cfg!(windows) { ";" } else { ":" }));
        }
    }
    env
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_schema_is_normalized_and_names_validated() {
        let t = parse_tool(&json!({ "name": "echo", "inputSchema": { "properties": null } })).unwrap();
        assert_eq!(t.input_schema, json!({ "type": "object", "properties": {} }));
        assert!(parse_tool(&json!({ "name": "bad name" })).is_none());
        let ro = parse_tool(&json!({ "name": "get", "annotations": { "readOnlyHint": true } })).unwrap();
        assert!(ro.read_only);
    }

    #[test]
    fn content_blocks_flatten_to_text() {
        let r = json!({ "content": [
            { "type": "text", "text": "hi" },
            { "type": "image", "data": "AAAA", "mimeType": "image/png" },
            { "type": "resource", "resource": { "uri": "file:///x" } }
        ]});
        assert_eq!(format_content(&r), "hi\n[image: image/png, 4 bytes base64]\n[resource: file:///x]");
        assert_eq!(format_content(&json!({ "content": [], "structuredContent": { "a": 1 } })), "{\"a\":1}");
    }

    #[test]
    fn remote_plain_http_is_refused() {
        let cfg = |u: &str| ServerConfig { url: Some(u.into()), ..Default::default() };
        assert!(http_url(&cfg("http://example.com/mcp")).is_err());
        assert!(http_url(&cfg("http://localhost:8085/mcp")).is_ok());
        let mut c = cfg("https://x.dev/api/");
        c.http_endpoint = Some("/mcp".into());
        assert_eq!(http_url(&c).unwrap().as_str(), "https://x.dev/api/mcp");
    }
}
