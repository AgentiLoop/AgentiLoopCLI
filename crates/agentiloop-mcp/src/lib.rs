//! agentiloop-mcp: Model Context Protocol client (stdio, Streamable HTTP and
//! legacy HTTP+SSE), ported from Agent!'s AgentMCP. Every tool a connected
//! server exposes is registered as `mcp_<server>_<tool>`.

mod client;
pub mod config;
mod http;
mod stdio;
mod transport;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use agentiloop_core::{Tool, ToolContext, ToolError, ToolRegistry, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};

pub use client::{McpServer, ResourceInfo, ToolInfo};
pub use config::ServerConfig;

/// Tool names must match `^[a-zA-Z0-9_-]{1,64}$` for the model APIs.
fn tool_name(server: &str, tool: &str) -> String {
    format!("mcp_{server}_{tool}")
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .take(64)
        .collect()
}

/// One MCP server tool exposed through the agent's tool registry.
pub struct McpTool {
    server: Arc<McpServer>,
    info: ToolInfo,
    name: String,
    description: String,
}

#[async_trait]
impl Tool for McpTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> Value {
        self.info.input_schema.clone()
    }

    /// External tools can do anything, so they're gated unless marked read-only.
    fn is_mutating(&self) -> bool {
        !self.info.read_only
    }

    async fn call(&self, _ctx: &ToolContext, input: Value) -> ToolResult {
        let args = if input.is_object() { input } else { json!({}) };
        if serde_json::to_vec(&args).map_or(0, |v| v.len()) > 1024 * 1024 {
            return Err(ToolError::InvalidInput("arguments exceed 1 MB limit".into()));
        }
        match self.server.call_tool(&self.info.name, args).await {
            Ok((text, false)) => Ok(text),
            Ok((text, true)) => Err(ToolError::Failed(text)),
            Err(e) => Err(ToolError::Failed(format!("MCP error: {e:#}"))),
        }
    }
}

/// `mcp_read_resource`: read a resource from any connected server.
pub struct ReadResource {
    servers: Vec<Arc<McpServer>>,
    description: String,
}

#[async_trait]
impl Tool for ReadResource {
    fn name(&self) -> &str {
        "mcp_read_resource"
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "server": { "type": "string", "description": "MCP server name" },
                "uri": { "type": "string", "description": "Resource URI" }
            },
            "required": ["server", "uri"]
        })
    }

    async fn call(&self, _ctx: &ToolContext, input: Value) -> ToolResult {
        let get = |k: &str| input.get(k).and_then(Value::as_str).ok_or_else(|| ToolError::InvalidInput(format!("`{k}` is required")));
        let (server, uri) = (get("server")?, get("uri")?);
        let s = self
            .servers
            .iter()
            .find(|s| s.name == server)
            .ok_or_else(|| ToolError::InvalidInput(format!("no connected MCP server named `{server}`")))?;
        s.read_resource(uri).await.map_err(ToolError::from)
    }
}

/// All configured servers: the ones that connected, and why the others didn't.
#[derive(Default)]
pub struct McpManager {
    pub servers: Vec<Arc<McpServer>>,
    /// (server name, error). Config-file parse errors use the file path as the name.
    pub errors: Vec<(String, String)>,
}

impl McpManager {
    /// Load `mcpServers` from `paths` and connect every enabled server concurrently.
    pub async fn start(paths: &[PathBuf], cwd: &Path) -> Self {
        let (configs, parse_errors) = config::load(paths);
        let mut mgr = McpManager { errors: parse_errors.into_iter().map(|e| ("config".into(), e)).collect(), ..Default::default() };
        let attempts = configs
            .iter()
            .filter(|(_, c)| c.should_start())
            .map(|(name, cfg)| async move { (name.clone(), McpServer::connect(name, cfg, cwd).await) });
        for (name, res) in futures::future::join_all(attempts).await {
            match res {
                Ok(s) => mgr.servers.push(Arc::new(s)),
                Err(e) => mgr.errors.push((name, format!("{e:#}"))),
            }
        }
        mgr
    }

    pub fn is_empty(&self) -> bool {
        self.servers.is_empty() && self.errors.is_empty()
    }

    pub fn tool_count(&self) -> usize {
        self.servers.iter().map(|s| s.tools.len()).sum()
    }

    /// Add every discovered tool (plus `mcp_read_resource` when any server has resources).
    pub fn register_tools(&self, registry: &mut ToolRegistry) {
        for server in &self.servers {
            for info in &server.tools {
                let description = if info.description.is_empty() {
                    format!("`{}` tool from MCP server `{}`", info.name, server.name)
                } else {
                    format!("{} (MCP server `{}`)", info.description, server.name)
                };
                registry.register(McpTool {
                    server: server.clone(),
                    name: tool_name(&server.name, &info.name),
                    info: info.clone(),
                    description,
                });
            }
        }
        let resources: Vec<String> = self
            .servers
            .iter()
            .flat_map(|s| s.resources.iter().map(move |r| format!("- {} {} ({})", s.name, r.uri, r.name)))
            .take(50)
            .collect();
        if !resources.is_empty() {
            registry.register(ReadResource {
                servers: self.servers.clone(),
                description: format!("Read a resource from a connected MCP server. Available:\n{}", resources.join("\n")),
            });
        }
    }

    /// Human-readable status for `/mcp`.
    pub fn status_lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        for s in &self.servers {
            let state = if s.is_alive() { "connected" } else { "disconnected" };
            out.push(format!(
                "● {} [{}] {} — {} ({} tools, {} resources)",
                s.name,
                s.transport_kind,
                s.server_info,
                state,
                s.tools.len(),
                s.resources.len()
            ));
            for t in &s.tools {
                let first = t.description.lines().next().unwrap_or("");
                out.push(format!("    {}  {}", tool_name(&s.name, &t.name), first.chars().take(80).collect::<String>()));
            }
        }
        for (name, err) in &self.errors {
            out.push(format!("✗ {name}: {err}"));
        }
        out
    }

    pub async fn shutdown(&self) {
        futures::future::join_all(self.servers.iter().map(|s| s.close())).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_names_are_api_safe() {
        assert_eq!(tool_name("Hello World", "say.hi"), "mcp_Hello_World_say_hi");
        assert_eq!(tool_name(&"x".repeat(80), "t").len(), 64);
    }
}
