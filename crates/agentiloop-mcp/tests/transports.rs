//! End-to-end tests of all three MCP transports (stdio, Streamable HTTP,
//! legacy HTTP+SSE) against the bundled `mcp-example-server`.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use agentiloop_core::{ToolContext, ToolError, ToolRegistry};
use agentiloop_mcp::{McpManager, McpServer, ServerConfig};
use serde_json::json;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

const EXAMPLE: &str = env!("CARGO_BIN_EXE_mcp-example-server");

/// Every test gets a hard deadline so a transport bug fails instead of hanging CI.
async fn deadline<F: std::future::Future>(f: F) -> F::Output {
    tokio::time::timeout(Duration::from_secs(30), f).await.expect("test timed out")
}

/// Start the example server in an HTTP mode; returns the child (killed on drop) and its port.
async fn start_http(mode: &str) -> (Child, u16) {
    let mut child = Command::new(EXAMPLE)
        .args([mode, "0"])
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("spawn example server");
    let mut line = String::new();
    BufReader::new(child.stdout.take().unwrap()).read_line(&mut line).await.unwrap();
    let port = line.trim().strip_prefix("listening on ").and_then(|p| p.parse().ok()).expect("port line");
    (child, port)
}

fn stdio_cfg() -> ServerConfig {
    ServerConfig { command: EXAMPLE.into(), args: vec!["--stdio".into()], ..Default::default() }
}

fn url_cfg(url: String) -> ServerConfig {
    ServerConfig { url: Some(url), ..Default::default() }
}

fn cwd() -> PathBuf {
    std::env::temp_dir()
}

/// The same checks for every transport.
async fn exercise(server: &McpServer, transport: &str) {
    assert_eq!(server.transport_kind, transport);
    assert_eq!(server.server_info, format!("example-{transport} 1.0.0"));

    // Two tools/list pages were merged and the invalid name dropped.
    let names: Vec<&str> = server.tools.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["echo", "add", "fail", "slow"]);
    let tool = |n: &str| server.tools.iter().find(|t| t.name == n).unwrap();
    assert!(tool("add").read_only && !tool("echo").read_only);
    assert_eq!(tool("fail").input_schema, json!({ "type": "object", "properties": {} }));

    assert_eq!(server.resources.len(), 1);
    assert_eq!(server.resources[0].uri, "example://greeting");

    assert_eq!(server.call_tool("echo", json!({ "message": "hi ✓" })).await.unwrap(), ("hi ✓".to_string(), false));
    assert_eq!(server.call_tool("add", json!({ "a": 2, "b": 3 })).await.unwrap().0, "5");

    // isError result, and a JSON-RPC error, both surface as tool errors.
    assert_eq!(server.call_tool("fail", json!({})).await.unwrap(), ("this tool always fails".to_string(), true));
    let (msg, is_error) = server.call_tool("nope", json!({})).await.unwrap();
    assert!(is_error && msg.contains("unknown tool: nope"), "{msg}");

    assert_eq!(server.read_resource("example://greeting").await.unwrap(), format!("Hello from {transport}!"));
    assert!(server.read_resource("example://missing").await.is_err());

    // Concurrent requests: a slow call must not block or steal the others' responses.
    let slow = server.call_tool("slow", json!({ "ms": 300 }));
    let echoes = futures::future::join_all((0..10).map(|i| server.call_tool("echo", json!({ "message": format!("m{i}") }))));
    let (slow, echoes) = tokio::join!(slow, echoes);
    assert_eq!(slow.unwrap().0, "slept 300ms");
    for (i, r) in echoes.into_iter().enumerate() {
        assert_eq!(r.unwrap().0, format!("m{i}"));
    }

    assert!(server.is_alive());
    server.close().await;
    assert!(!server.is_alive());
    assert!(server.call_tool("echo", json!({ "message": "x" })).await.is_err());
}

#[tokio::test]
async fn stdio_transport() {
    deadline(async {
        let server = McpServer::connect("Stdio", &stdio_cfg(), &cwd()).await.unwrap();
        exercise(&server, "stdio").await;
    })
    .await
}

#[tokio::test]
async fn streamable_http_transport() {
    deadline(async {
        let (_child, port) = start_http("--http").await;
        let server = McpServer::connect("Http", &url_cfg(format!("http://127.0.0.1:{port}/mcp")), &cwd()).await.unwrap();
        exercise(&server, "http").await;
    })
    .await
}

#[tokio::test]
async fn streamable_http_endpoint_path_is_appended() {
    deadline(async {
        let (_child, port) = start_http("--http").await;
        let cfg = ServerConfig { http_endpoint: Some("/mcp".into()), ..url_cfg(format!("http://localhost:{port}/")) };
        let server = McpServer::connect("Http", &cfg, &cwd()).await.unwrap();
        assert_eq!(server.call_tool("echo", json!({ "message": "ok" })).await.unwrap().0, "ok");
        server.close().await;
    })
    .await
}

#[tokio::test]
async fn legacy_sse_transport_autodetected_from_url() {
    deadline(async {
        let (_child, port) = start_http("--sse").await;
        let server = McpServer::connect("Sse", &url_cfg(format!("http://127.0.0.1:{port}/sse")), &cwd()).await.unwrap();
        exercise(&server, "sse").await;
    })
    .await
}

#[tokio::test]
async fn legacy_sse_transport_forced_by_config() {
    deadline(async {
        let (_child, port) = start_http("--sse").await;
        // URL doesn't end in /sse: `"transport": "sse"` or a non-empty `sseEndpoint` forces legacy.
        let url = format!("http://127.0.0.1:{port}/events");
        let by_transport = ServerConfig { transport: Some("sse".into()), ..url_cfg(url.clone()) };
        let by_endpoint = ServerConfig { sse_endpoint: Some("/events".into()), ..url_cfg(url.clone()) };
        for cfg in [by_transport, by_endpoint] {
            let server = McpServer::connect("Sse", &cfg, &cwd()).await.unwrap();
            assert_eq!(server.transport_kind, "sse");
            assert_eq!(server.call_tool("add", json!({ "a": 1.5, "b": 1 })).await.unwrap().0, "2.5");
            server.close().await;
        }
        // Without either, the same URL is treated as Streamable HTTP (and the SSE server rejects the POST).
        assert!(McpServer::connect("Sse", &url_cfg(url), &cwd()).await.is_err());
    })
    .await
}

#[tokio::test]
async fn connection_failures_are_reported() {
    deadline(async {
        let missing = ServerConfig { command: "/definitely/not/a/server".into(), ..Default::default() };
        assert!(McpServer::connect("x", &missing, &cwd()).await.is_err());

        // Wrong path on a live Streamable HTTP server → 404 during initialize.
        let (_child, port) = start_http("--http").await;
        let err = McpServer::connect("x", &url_cfg(format!("http://127.0.0.1:{port}/wrong")), &cwd()).await.err().unwrap();
        assert!(format!("{err:#}").contains("404"), "{err:#}");

        // Nothing listening on the SSE port.
        let (child, port) = start_http("--sse").await;
        drop(child);
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(McpServer::connect("x", &url_cfg(format!("http://127.0.0.1:{port}/sse")), &cwd()).await.is_err());

        // Remote plain HTTP is refused before any network I/O.
        let err = McpServer::connect("x", &url_cfg("http://example.com/mcp".into()), &cwd()).await.err().unwrap();
        assert!(format!("{err:#}").contains("localhost"), "{err:#}");
    })
    .await
}

#[tokio::test]
async fn stdio_server_exit_is_detected() {
    deadline(async {
        let server = McpServer::connect("Stdio", &stdio_cfg(), &cwd()).await.unwrap();
        assert!(server.is_alive());
        server.close().await; // kills the process
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(!server.is_alive());
        let err = server.call_tool("echo", json!({})).await.err().unwrap();
        assert!(format!("{err:#}").contains("no longer running"), "{err:#}");
    })
    .await
}

fn write_config(dir: &Path, body: serde_json::Value) -> PathBuf {
    let p = dir.join("mcp.json");
    std::fs::write(&p, body.to_string()).unwrap();
    p
}

/// Config file → manager → agent tool registry → tool call, across all three transports at once.
#[tokio::test]
async fn manager_registers_tools_from_all_transports() {
    deadline(async {
        let (_h, hp) = start_http("--http").await;
        let (_s, sp) = start_http("--sse").await;
        let dir = std::env::temp_dir().join(format!("agl-mcp-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("AGL_MCP_TEST_PORT", sp.to_string());
        let cfg = write_config(
            &dir,
            json!({ "mcpServers": {
                "Local": { "command": EXAMPLE, "args": ["--stdio"] },
                "Web": { "type": "http", "url": format!("http://127.0.0.1:{hp}/mcp") },
                "Legacy": { "url": "http://127.0.0.1:${AGL_MCP_TEST_PORT}/sse" },
                "Off": { "command": EXAMPLE, "args": ["--stdio"], "disabled": true },
                "Broken": { "command": "/no/such/binary" }
            }}),
        );
        let mgr = McpManager::start(&[cfg], &dir).await;
        let mut names: Vec<&str> = mgr.servers.iter().map(|s| s.name.as_str()).collect();
        names.sort();
        assert_eq!(names, ["Legacy", "Local", "Web"]);
        assert_eq!(mgr.errors.len(), 1);
        assert_eq!(mgr.errors[0].0, "Broken");
        assert_eq!(mgr.tool_count(), 12);

        let mut reg = ToolRegistry::new();
        mgr.register_tools(&mut reg);
        assert_eq!(reg.len(), 13); // 12 tools + mcp_read_resource
        let ctx = ToolContext { cwd: dir.clone() };
        for server in ["Local", "Web", "Legacy"] {
            let echo = reg.get(&format!("mcp_{server}_echo")).unwrap();
            assert!(echo.is_mutating());
            assert_eq!(echo.call(&ctx, json!({ "message": server })).await.unwrap(), server);
            assert!(!reg.get(&format!("mcp_{server}_add")).unwrap().is_mutating());
            let fail = reg.get(&format!("mcp_{server}_fail")).unwrap().call(&ctx, json!({})).await;
            assert!(matches!(fail, Err(ToolError::Failed(m)) if m == "this tool always fails"));
        }
        let read = reg.get("mcp_read_resource").unwrap();
        assert_eq!(read.call(&ctx, json!({ "server": "Legacy", "uri": "example://greeting" })).await.unwrap(), "Hello from sse!");
        assert!(read.call(&ctx, json!({ "server": "Nope", "uri": "x" })).await.is_err());

        let status = mgr.status_lines().join("\n");
        assert!(status.contains("● Local [stdio] example-stdio 1.0.0 — connected (4 tools, 1 resources)"), "{status}");
        assert!(status.contains("[http]") && status.contains("[sse]") && status.contains("✗ Broken"), "{status}");

        mgr.shutdown().await;
        assert!(mgr.servers.iter().all(|s| !s.is_alive()));
        let _ = std::fs::remove_dir_all(&dir);
    })
    .await
}

#[tokio::test]
async fn bad_config_file_is_reported_not_fatal() {
    let dir = std::env::temp_dir().join(format!("agl-mcp-bad-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("mcp.json");
    std::fs::write(&p, "{ not json").unwrap();
    let mgr = McpManager::start(&[p, dir.join("missing.json")], &dir).await;
    assert!(mgr.servers.is_empty());
    assert_eq!(mgr.errors.len(), 1);
    assert_eq!(mgr.errors[0].0, "config");
    let _ = std::fs::remove_dir_all(&dir);
}
