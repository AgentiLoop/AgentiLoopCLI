//! HTTP transports: Streamable HTTP (MCP 2025-03-26, single POST endpoint)
//! and the legacy HTTP+SSE transport (MCP 2024-11-05, GET `/sse` stream +
//! POST to the URL announced by the server's `endpoint` event).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, ACCEPT};
use reqwest::{Client, Response, StatusCode, Url};
use serde_json::Value;
use tokio::sync::oneshot;

use crate::transport::{await_response, dispatch, response_id, rpc_notification, rpc_request, Pending, SseParser, Transport};

const SESSION_HEADER: &str = "mcp-session-id";

pub(crate) fn header_map(headers: &HashMap<String, String>) -> Result<HeaderMap> {
    let mut map = HeaderMap::new();
    for (k, v) in headers {
        map.insert(
            HeaderName::from_bytes(k.as_bytes()).with_context(|| format!("bad header name {k}"))?,
            HeaderValue::from_str(v).with_context(|| format!("bad value for header {k}"))?,
        );
    }
    Ok(map)
}

async fn error_body(resp: Response) -> String {
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    format!("HTTP {}: {}", status.as_u16(), body.chars().take(512).collect::<String>())
}

// MARK: - Streamable HTTP

pub struct HttpTransport {
    client: Client,
    url: Url,
    headers: HeaderMap,
    session_id: StdMutex<Option<String>>,
    next_id: AtomicI64,
    alive: AtomicBool,
}

impl HttpTransport {
    pub fn new(url: Url, headers: &HashMap<String, String>) -> Result<Self> {
        let client = Client::builder().connect_timeout(Duration::from_secs(30)).timeout(Duration::from_secs(600)).build()?;
        Ok(Self {
            client,
            url,
            headers: header_map(headers)?,
            session_id: StdMutex::new(None),
            next_id: AtomicI64::new(0),
            alive: AtomicBool::new(true),
        })
    }

    async fn post(&self, body: &Value) -> Result<Response> {
        let mut rb = self
            .client
            .post(self.url.clone())
            .headers(self.headers.clone())
            .header(ACCEPT, "application/json, text/event-stream")
            .json(body);
        if let Some(sid) = self.session_id.lock().unwrap().clone() {
            rb = rb.header(SESSION_HEADER, sid);
        }
        let resp = rb.send().await?;
        if let Some(sid) = resp.headers().get(SESSION_HEADER).and_then(|v| v.to_str().ok()) {
            *self.session_id.lock().unwrap() = Some(sid.to_string());
        }
        if resp.status() == StatusCode::NOT_FOUND {
            *self.session_id.lock().unwrap() = None;
            anyhow::bail!("MCP session expired (404)");
        }
        if !resp.status().is_success() {
            anyhow::bail!(error_body(resp).await);
        }
        Ok(resp)
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        anyhow::ensure!(self.is_alive(), "HTTP connection is closed");
        let id = self.next_id.fetch_add(1, Ordering::SeqCst) + 1;
        let resp = self.post(&rpc_request(id, method, params)).await?;
        let is_sse = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|c| c.contains("text/event-stream"));
        if !is_sse {
            return resp.json::<Value>().await.context("invalid JSON-RPC response");
        }
        // The server may stream progress notifications before the final response.
        let mut stream = resp.bytes_stream();
        let mut parser = SseParser::default();
        let mut last = None;
        while let Some(chunk) = stream.next().await {
            for ev in parser.push(&chunk?) {
                if let Some(json) = ev.message_json() {
                    if response_id(&json) == Some(id) {
                        return Ok(json);
                    }
                    last = Some(json);
                }
            }
        }
        if let Some(json) = parser.flush().and_then(|ev| ev.message_json()) {
            return Ok(json);
        }
        last.context("event stream closed without a response")
    }

    async fn notify(&self, method: &str, params: Option<Value>) -> Result<()> {
        if self.is_alive() {
            self.post(&rpc_notification(method, params)).await?;
        }
        Ok(())
    }

    fn is_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }

    async fn close(&self) {
        self.alive.store(false, Ordering::SeqCst);
        // Per spec, DELETE ends the server-side session.
        let sid = self.session_id.lock().unwrap().take();
        if let Some(sid) = sid {
            let _ = self
                .client
                .delete(self.url.clone())
                .headers(self.headers.clone())
                .header(SESSION_HEADER, sid)
                .timeout(Duration::from_secs(2))
                .send()
                .await;
        }
    }
}

// MARK: - Legacy HTTP+SSE

pub struct LegacySseTransport {
    /// Separate client for POSTs so they never share a connection with the long-lived stream.
    post_client: Client,
    endpoint: Url,
    headers: HeaderMap,
    pending: Pending,
    next_id: AtomicI64,
    alive: Arc<AtomicBool>,
    reader: tokio::task::JoinHandle<()>,
}

impl LegacySseTransport {
    /// Open the GET stream and wait (≤30 s) for the server's `endpoint` event.
    pub async fn connect(url: Url, headers: &HashMap<String, String>) -> Result<Self> {
        let headers = header_map(headers)?;
        let stream_client = Client::builder().connect_timeout(Duration::from_secs(30)).build()?;
        let resp = stream_client
            .get(url.clone())
            .headers(headers.clone())
            .header(ACCEPT, "text/event-stream")
            .send()
            .await
            .with_context(|| format!("could not open SSE stream {url}"))?;
        if !resp.status().is_success() {
            anyhow::bail!(error_body(resp).await);
        }

        let pending: Pending = Default::default();
        let alive = Arc::new(AtomicBool::new(true));
        let (ep_tx, ep_rx) = oneshot::channel::<String>();
        let reader = {
            let (pending, alive) = (pending.clone(), alive.clone());
            tokio::spawn(async move {
                let mut ep_tx = Some(ep_tx);
                let mut parser = SseParser::default();
                let mut stream = resp.bytes_stream();
                while let Some(Ok(chunk)) = stream.next().await {
                    for ev in parser.push(&chunk) {
                        if ev.event == "endpoint" {
                            if let Some(tx) = ep_tx.take() {
                                let _ = tx.send(ev.data.trim().to_string());
                            }
                        } else if let Some(json) = ev.message_json() {
                            dispatch(&pending, json);
                        }
                    }
                }
                alive.store(false, Ordering::SeqCst);
                pending.lock().unwrap().clear();
            })
        };

        let ep = match tokio::time::timeout(Duration::from_secs(30), ep_rx).await {
            Ok(Ok(ep)) => ep,
            Ok(Err(_)) => {
                reader.abort();
                anyhow::bail!("SSE stream closed before the `endpoint` event")
            }
            Err(_) => {
                reader.abort();
                anyhow::bail!("SSE endpoint handshake timed out after 30s")
            }
        };
        let endpoint = url.join(&ep).with_context(|| format!("bad endpoint `{ep}`"))?;
        let post_client = Client::builder().connect_timeout(Duration::from_secs(30)).timeout(Duration::from_secs(180)).build()?;
        Ok(Self { post_client, endpoint, headers, pending, next_id: AtomicI64::new(0), alive, reader })
    }

    async fn post(&self, body: &Value) -> Result<Response> {
        let resp = self.post_client.post(self.endpoint.clone()).headers(self.headers.clone()).json(body).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!(error_body(resp).await);
        }
        Ok(resp)
    }
}

#[async_trait]
impl Transport for LegacySseTransport {
    async fn request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        anyhow::ensure!(self.is_alive(), "SSE stream is closed");
        let id = self.next_id.fetch_add(1, Ordering::SeqCst) + 1;
        let (tx, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(id, tx);
        match self.post(&rpc_request(id, method, params)).await {
            // Normally 202 + empty body, the reply arrives on the stream. Some
            // servers answer inline instead; accept that too.
            Ok(resp) => {
                if let Ok(json) = resp.json::<Value>().await {
                    if response_id(&json) == Some(id) {
                        self.pending.lock().unwrap().remove(&id);
                        return Ok(json);
                    }
                }
            }
            Err(e) => {
                self.pending.lock().unwrap().remove(&id);
                return Err(e);
            }
        }
        await_response(&self.pending, id, rx, method).await
    }

    async fn notify(&self, method: &str, params: Option<Value>) -> Result<()> {
        self.post(&rpc_notification(method, params)).await.map(|_| ())
    }

    fn is_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }

    async fn close(&self) {
        self.alive.store(false, Ordering::SeqCst);
        self.reader.abort();
        self.pending.lock().unwrap().clear();
    }
}
