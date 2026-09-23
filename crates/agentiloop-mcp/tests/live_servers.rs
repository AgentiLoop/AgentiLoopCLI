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

/// Live Streamable HTTP round-trip against AgentMCP's DemoHttp server
/// (`~/bin/mcp-server-demo-http <port>`, Python) when installed; skipped otherwise.
#[tokio::test]
async fn demo_http_round_trip() {
    let Some(bin) = dirs::home_dir().map(|h| h.join("bin/mcp-server-demo-http")).filter(|p| p.is_file()) else {
        eprintln!("mcp-server-demo-http not installed; skipping");
        return;
    };
    let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let _child = tokio::process::Command::new(bin)
        .arg(port.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let cfg = ServerConfig { url: Some(format!("http://localhost:{port}/mcp")), ..Default::default() };
    // Give the Python server a moment to bind.
    let mut server = None;
    for _ in 0..50 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if let Ok(s) = McpServer::connect("DemoHttp", &cfg, &std::env::temp_dir()).await {
            server = Some(s);
            break;
        }
    }
    let server = server.expect("demo-http server did not come up");
    assert_eq!(server.transport_kind, "http");
    assert!(server.tools.iter().any(|t| t.name == "calculate"));
    let (out, is_error) = server.call_tool("calculate", json!({ "operation": "add", "a": 2, "b": 3 })).await.unwrap();
    println!("calculate → {out}");
    assert!(!is_error && out.contains('5'));
    let (out, _) = server.call_tool("reverse_string", json!({ "text": "Agent!" })).await.unwrap();
    assert_eq!(out, "!tnegA");
    server.close().await;
}
