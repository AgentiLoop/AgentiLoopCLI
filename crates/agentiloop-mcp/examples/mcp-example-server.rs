//! Minimal example MCP server used by the transport tests. One binary, three transports:
//!
//! ```text
//! mcp-example-server --stdio          newline-delimited JSON-RPC on stdin/stdout
//! mcp-example-server --http <port>    Streamable HTTP on http://127.0.0.1:<port>/mcp
//! mcp-example-server --sse  <port>    legacy HTTP+SSE: GET /sse, POST /messages?session_id=…
//! ```
//!
//! Port 0 picks a free port; HTTP modes print `listening on <port>` as the first stdout line.
//! Tools: `echo` (text), `add` (read-only), `fail` (isError), `slow` (sleeps `ms`), listed
//! over two `tools/list` pages. Resource: `example://greeting`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let port = || args.get(1).and_then(|p| p.parse::<u16>().ok()).unwrap_or(0);
    match args.first().map(String::as_str) {
        Some("--stdio") => stdio().await,
        Some("--http") => serve(port(), Mode::Http).await,
        Some("--sse") => serve(port(), Mode::Sse).await,
        _ => {
            eprintln!("usage: mcp-example-server --stdio | --http <port> | --sse <port>");
            std::process::exit(2)
        }
    }
}

// MARK: - MCP logic (shared by all transports)

/// Handle one JSON-RPC message; `None` for notifications.
async fn handle(msg: &Value, transport: &str) -> Option<Value> {
    let id = msg.get("id")?.clone();
    let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
    let params = msg.get("params").cloned().unwrap_or(json!({}));
    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {}, "resources": {} },
            "serverInfo": { "name": format!("example-{transport}"), "version": "1.0.0" }
        })),
        "tools/list" => Ok(match params.get("cursor").and_then(Value::as_str) {
            None => json!({ "tools": [
                { "name": "echo", "description": "Echo back a message",
                  "inputSchema": { "type": "object", "properties": { "message": { "type": "string" } }, "required": ["message"] } },
                { "name": "add", "description": "Add two numbers",
                  "inputSchema": { "type": "object", "properties": { "a": { "type": "number" }, "b": { "type": "number" } } },
                  "annotations": { "readOnlyHint": true } },
                { "name": "bad name!", "description": "invalid name, must be dropped by the client" }
            ], "nextCursor": "page2" }),
            Some(_) => json!({ "tools": [
                { "name": "fail", "description": "Always reports a tool error" },
                { "name": "slow", "description": "Sleep then answer",
                  "inputSchema": { "type": "object", "properties": { "ms": { "type": "integer" } } } }
            ] }),
        }),
        "tools/call" => call_tool(&params).await,
        "resources/list" => Ok(json!({ "resources": [
            { "uri": "example://greeting", "name": "Greeting", "mimeType": "text/plain" }
        ] })),
        "resources/read" => match params.get("uri").and_then(Value::as_str) {
            Some("example://greeting") => Ok(json!({ "contents": [
                { "uri": "example://greeting", "mimeType": "text/plain", "text": format!("Hello from {transport}!") }
            ] })),
            other => Err((-32002, format!("resource not found: {}", other.unwrap_or("")))),
        },
        "ping" => Ok(json!({})),
        _ => Err((-32601, format!("method not found: {method}"))),
    };
    Some(match result {
        Ok(r) => json!({ "jsonrpc": "2.0", "id": id, "result": r }),
        Err((code, message)) => json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } }),
    })
}

async fn call_tool(params: &Value) -> Result<Value, (i64, String)> {
    let args = params.get("arguments").cloned().unwrap_or(json!({}));
    let text = |t: String| json!({ "content": [{ "type": "text", "text": t }] });
    match params.get("name").and_then(Value::as_str).unwrap_or("") {
        "echo" => Ok(text(args.get("message").and_then(Value::as_str).unwrap_or("").to_string())),
        "add" => {
            let n = |k: &str| args.get(k).and_then(Value::as_f64).unwrap_or(0.0);
            Ok(text(format!("{}", n("a") + n("b"))))
        }
        "fail" => Ok(json!({ "content": [{ "type": "text", "text": "this tool always fails" }], "isError": true })),
        "slow" => {
            let ms = args.get("ms").and_then(Value::as_u64).unwrap_or(100);
            tokio::time::sleep(Duration::from_millis(ms)).await;
            Ok(text(format!("slept {ms}ms")))
        }
        other => Err((-32602, format!("unknown tool: {other}"))),
    }
}

// MARK: - stdio

async fn stdio() -> std::io::Result<()> {
    eprintln!("example server: stdio ready"); // exercises the client's stderr drain
    let out = Arc::new(tokio::sync::Mutex::new(tokio::io::stdout()));
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    while let Some(line) = lines.next_line().await? {
        let Ok(msg) = serde_json::from_str::<Value>(&line) else { continue };
        let out = out.clone();
        // Answer concurrently so replies can arrive out of order.
        tokio::spawn(async move {
            if let Some(resp) = handle(&msg, "stdio").await {
                let mut w = out.lock().await;
                let _ = w.write_all(format!("{resp}\n").as_bytes()).await;
                let _ = w.flush().await;
            }
        });
    }
    Ok(())
}

// MARK: - HTTP (hand-rolled HTTP/1.1, enough for the tests)

#[derive(Clone, Copy)]
enum Mode {
    Http,
    Sse,
}

#[derive(Default)]
struct State {
    next: AtomicU64,
    /// Streamable HTTP sessions that were initialized and not DELETEd.
    sessions: Mutex<Vec<String>>,
    /// Legacy SSE: session id → sender feeding that client's GET stream.
    streams: Mutex<HashMap<String, mpsc::UnboundedSender<String>>>,
}

async fn serve(port: u16, mode: Mode) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port)).await?;
    println!("listening on {}", listener.local_addr()?.port());
    use std::io::Write;
    std::io::stdout().flush()?;
    let state = Arc::new(State::default());
    loop {
        let (sock, _) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            let _ = connection(sock, mode, state).await;
        });
    }
}

struct Request {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

async fn read_request(r: &mut BufReader<TcpStream>) -> std::io::Result<Option<Request>> {
    let mut line = String::new();
    if r.read_line(&mut line).await? == 0 {
        return Ok(None);
    }
    let mut parts = line.split_whitespace();
    let (method, path) = (parts.next().unwrap_or("").to_string(), parts.next().unwrap_or("").to_string());
    let mut headers = HashMap::new();
    loop {
        line.clear();
        r.read_line(&mut line).await?;
        let l = line.trim_end();
        if l.is_empty() {
            break;
        }
        if let Some((k, v)) = l.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }
    let len = headers.get("content-length").and_then(|v| v.parse().ok()).unwrap_or(0);
    let mut body = vec![0; len];
    r.read_exact(&mut body).await?;
    Ok(Some(Request { method, path, headers, body }))
}

async fn respond(w: &mut BufReader<TcpStream>, status: &str, extra: &[(&str, String)], body: &str) -> std::io::Result<()> {
    let mut head = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\n", body.len());
    for (k, v) in extra {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    head.push_str("\r\n");
    w.get_mut().write_all(head.as_bytes()).await?;
    w.get_mut().write_all(body.as_bytes()).await?;
    w.get_mut().flush().await
}

async fn connection(sock: TcpStream, mode: Mode, state: Arc<State>) -> std::io::Result<()> {
    let mut r = BufReader::new(sock);
    // Keep-alive: serve requests until the client closes (or we hand the socket to an SSE stream).
    while let Some(req) = read_request(&mut r).await? {
        let (path, query) = req.path.split_once('?').unwrap_or((&req.path, ""));
        match (mode, req.method.as_str(), path) {
            (Mode::Http, "POST", "/mcp") => streamable_post(&mut r, &req, &state).await?,
            (Mode::Http, "DELETE", "/mcp") => {
                let sid = req.headers.get("mcp-session-id").cloned().unwrap_or_default();
                state.sessions.lock().unwrap().retain(|s| *s != sid);
                respond(&mut r, "200 OK", &[], "").await?
            }
            // `/events` lets tests check that `"transport": "sse"` forces the legacy transport.
            (Mode::Sse, "GET", "/sse" | "/events") => return sse_stream(r, &state).await,
            (Mode::Sse, "POST", "/messages") => {
                let sid = query.strip_prefix("session_id=").unwrap_or("").to_string();
                let tx = state.streams.lock().unwrap().get(&sid).cloned();
                let Some(tx) = tx else {
                    respond(&mut r, "404 Not Found", &[], "unknown session").await?;
                    continue;
                };
                let Ok(msg) = serde_json::from_slice::<Value>(&req.body) else {
                    respond(&mut r, "400 Bad Request", &[], "bad json").await?;
                    continue;
                };
                respond(&mut r, "202 Accepted", &[], "").await?;
                // The reply travels back over the GET stream, not this response.
                tokio::spawn(async move {
                    if let Some(resp) = handle(&msg, "sse").await {
                        let _ = tx.send(format!("event: message\ndata: {resp}\n\n"));
                    }
                });
            }
            _ => respond(&mut r, "404 Not Found", &[], "not found").await?,
        }
    }
    Ok(())
}

async fn streamable_post(r: &mut BufReader<TcpStream>, req: &Request, state: &State) -> std::io::Result<()> {
    let Ok(msg) = serde_json::from_slice::<Value>(&req.body) else {
        return respond(r, "400 Bad Request", &[], "bad json").await;
    };
    let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
    let sid = req.headers.get("mcp-session-id").cloned();
    let mut extra = Vec::new();
    if method == "initialize" {
        let new = format!("sess-{}", state.next.fetch_add(1, Ordering::SeqCst));
        state.sessions.lock().unwrap().push(new.clone());
        extra.push(("Mcp-Session-Id", new));
    } else if !sid.is_some_and(|s| state.sessions.lock().unwrap().contains(&s)) {
        // Unknown / missing session → 404, which clients must treat as "re-initialize".
        return respond(r, "404 Not Found", &[], "session not found").await;
    }
    let Some(resp) = handle(&msg, "http").await else {
        return respond(r, "202 Accepted", &extra, "").await;
    };
    let accepts_sse = req.headers.get("accept").is_some_and(|a| a.contains("text/event-stream"));
    if method == "tools/call" && accepts_sse {
        // Stream a progress notification before the actual response, like real servers do.
        let progress = json!({ "jsonrpc": "2.0", "method": "notifications/progress", "params": { "progress": 1 } });
        let body = format!(": keepalive\n\nevent: message\ndata: {progress}\n\ndata: {resp}\n\n");
        extra.push(("Content-Type", "text/event-stream".into()));
        respond(r, "200 OK", &extra, &body).await
    } else {
        extra.push(("Content-Type", "application/json".into()));
        respond(r, "200 OK", &extra, &resp.to_string()).await
    }
}

async fn sse_stream(r: BufReader<TcpStream>, state: &State) -> std::io::Result<()> {
    let mut sock = r.into_inner();
    let sid = format!("s{}", state.next.fetch_add(1, Ordering::SeqCst));
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    state.streams.lock().unwrap().insert(sid.clone(), tx);
    // No Content-Length: the body runs until we close the socket.
    sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n").await?;
    sock.write_all(format!("event: endpoint\ndata: /messages?session_id={sid}\n\n").as_bytes()).await?;
    sock.flush().await?;
    let mut probe = [0u8; 1];
    loop {
        tokio::select! {
            ev = rx.recv() => {
                let Some(ev) = ev else { break };
                if sock.write_all(ev.as_bytes()).await.is_err() { break }
                let _ = sock.flush().await;
            }
            // Client hung up.
            n = sock.read(&mut probe) => if matches!(n, Ok(0) | Err(_)) { break },
        }
    }
    state.streams.lock().unwrap().remove(&sid);
    Ok(())
}
