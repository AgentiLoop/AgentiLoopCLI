//! Transport abstraction shared by stdio, Streamable HTTP and legacy HTTP+SSE,
//! plus the JSON-RPC / SSE helpers they have in common.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::sync::oneshot;

/// MCP tools (domain checks, builds, …) can be slow: 15 minutes per request.
pub(crate) const REQUEST_TIMEOUT: Duration = Duration::from_secs(900);

#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a JSON-RPC request and wait for the matching response object.
    async fn request(&self, method: &str, params: Option<Value>) -> anyhow::Result<Value>;
    /// Send a JSON-RPC notification (no response expected).
    async fn notify(&self, method: &str, params: Option<Value>) -> anyhow::Result<()>;
    fn is_alive(&self) -> bool;
    async fn close(&self);
}

/// In-flight requests keyed by JSON-RPC id; dropping a sender fails the waiter.
pub(crate) type Pending = Arc<Mutex<HashMap<i64, oneshot::Sender<Value>>>>;

pub(crate) fn rpc_request(id: i64, method: &str, params: Option<Value>) -> Value {
    let mut msg = json!({ "jsonrpc": "2.0", "id": id, "method": method });
    if let Some(p) = params {
        msg["params"] = p;
    }
    msg
}

pub(crate) fn rpc_notification(method: &str, params: Option<Value>) -> Value {
    let mut msg = json!({ "jsonrpc": "2.0", "method": method });
    if let Some(p) = params {
        msg["params"] = p;
    }
    msg
}

/// JSON-RPC id as an integer (servers may echo it back as a numeric string).
pub(crate) fn response_id(msg: &Value) -> Option<i64> {
    match msg.get("id")? {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

/// Hand a message to whichever request is waiting on its id.
pub(crate) fn dispatch(pending: &Pending, msg: Value) {
    if let Some(id) = response_id(&msg) {
        if let Some(tx) = pending.lock().unwrap().remove(&id) {
            let _ = tx.send(msg);
        }
    }
}

/// Wait for a registered response with the standard timeout; cleans up on failure.
pub(crate) async fn await_response(pending: &Pending, id: i64, rx: oneshot::Receiver<Value>, method: &str) -> anyhow::Result<Value> {
    match tokio::time::timeout(REQUEST_TIMEOUT, rx).await {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(_)) => anyhow::bail!("MCP server closed the connection"),
        Err(_) => {
            pending.lock().unwrap().remove(&id);
            anyhow::bail!("timeout waiting for {method}")
        }
    }
}

/// One complete Server-Sent Event.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct SseEvent {
    pub event: String,
    pub data: String,
}

impl SseEvent {
    /// JSON payload of a `message` (or untyped) event, per the MCP spec.
    pub fn message_json(&self) -> Option<Value> {
        if !(self.event.is_empty() || self.event == "message") {
            return None;
        }
        serde_json::from_str::<Value>(&self.data).ok().filter(Value::is_object)
    }
}

/// Incremental SSE parser: feed raw bytes, get events out.
#[derive(Default)]
pub(crate) struct SseParser {
    buf: Vec<u8>,
    event: String,
    data: String,
}

impl SseParser {
    pub fn push(&mut self, bytes: &[u8]) -> Vec<SseEvent> {
        self.buf.extend_from_slice(bytes);
        let mut out = Vec::new();
        while let Some(pos) = self.buf.iter().position(|b| *b == b'\n') {
            let raw: Vec<u8> = self.buf.drain(..=pos).collect();
            let line = String::from_utf8_lossy(&raw);
            if let Some(ev) = self.line(line.trim_end_matches(['\r', '\n'])) {
                out.push(ev);
            }
        }
        out
    }

    fn line(&mut self, line: &str) -> Option<SseEvent> {
        if line.is_empty() {
            return self.take();
        }
        if let Some(v) = line.strip_prefix("event:") {
            self.event = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("data:") {
            let v = v.strip_prefix(' ').unwrap_or(v);
            if !self.data.is_empty() {
                self.data.push('\n');
            }
            self.data.push_str(v);
        }
        // id:, retry: and `:` comments are ignored.
        None
    }

    /// Event still buffered when the stream closed without a trailing blank line.
    pub fn flush(&mut self) -> Option<SseEvent> {
        if !self.buf.is_empty() {
            let rest = std::mem::take(&mut self.buf);
            let line = String::from_utf8_lossy(&rest).trim_end().to_string();
            self.line(&line);
        }
        self.take()
    }

    fn take(&mut self) -> Option<SseEvent> {
        let ev = SseEvent { event: std::mem::take(&mut self.event), data: std::mem::take(&mut self.data) };
        (!ev.data.is_empty()).then_some(ev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_events_across_chunks() {
        let mut p = SseParser::default();
        assert!(p.push(b"event: endpoint\ndata: /messages?s").is_empty());
        let evs = p.push(b"=1\n\ndata: {\"jsonrpc\":\"2.0\",\"id\":3}\n\n");
        assert_eq!(evs[0], SseEvent { event: "endpoint".into(), data: "/messages?s=1".into() });
        assert!(evs[0].message_json().is_none());
        assert_eq!(response_id(&evs[1].message_json().unwrap()), Some(3));
    }

    #[test]
    fn flushes_trailing_event_and_string_ids() {
        let mut p = SseParser::default();
        p.push(b"data: {\"id\":\"7\"}");
        let ev = p.flush().unwrap();
        assert_eq!(response_id(&ev.message_json().unwrap()), Some(7));
    }
}
