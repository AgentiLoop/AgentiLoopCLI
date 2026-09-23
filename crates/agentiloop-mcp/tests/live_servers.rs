//! Live round-trip against AgentMCP's HelloWorld stdio server when installed
//! (`~/bin/mcp-server-hello`); skipped otherwise.

use agentiloop_mcp::{McpServer, ServerConfig};
use serde_json::json;

#[tokio::test]
async fn hello_world_stdio_round_trip() {
    let Some(bin) = dirs::home_dir().map(|h| h.join("bin/mcp-server-hello")).filter(|p| p.is_file()) else {
        eprintln!("mcp-server-hello not installed; skipping");
        return;
    };
    let cfg = ServerConfig { command: bin.display().to_string(), ..Default::default() };
    let server = McpServer::connect("HelloWorld", &cfg, &std::env::temp_dir()).await.unwrap();
    assert!(server.tools.iter().any(|t| t.name == "hello"));
    let (out, is_error) = server.call_tool("hello", json!({ "name": "Todd" })).await.unwrap();
    println!("hello → {out}");
    assert!(!is_error && out.contains("Todd"));
    server.close().await;
}
