//! Codex: your ChatGPT subscription via the OAuth tokens `codex login` writes to
//! `~/.codex/auth.json` (or `$CODEX_HOME/auth.json`), against
//! `chatgpt.com/backend-api/codex` — the Responses API the official Codex CLI
//! uses. Streaming only; tokens refresh automatically and are written back so
//! the Codex CLI stays signed in too.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use agentiloop_core::{ContentBlock, Message, ModelInfo, Provider, ProviderRequest, ProviderResponse, Role, StopReason};
use anyhow::Context;
use async_trait::async_trait;
use futures::StreamExt;
use serde_json::{json, Value};

const BASE_URL: &str = "https://chatgpt.com/backend-api/codex";
const TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
/// OpenAI's public (PKCE) client id for the Codex CLI.
const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
/// The backend wants a Codex CLI version on every call (`/models` 400s without it).
const CLIENT_VERSION: &str = "0.154.0";
/// Identity line the Codex CLI opens its instructions with.
const IDENTITY: &str =
    "You are Codex, based on GPT-5. You are running as a coding agent in the Codex CLI on a user's computer.";

pub struct CodexProvider {
    client: reqwest::Client,
    auth_path: PathBuf,
    base_url: String,
    token_url: String,
    effort: String,
}

struct Tokens {
    access: String,
    refresh: String,
    account: String,
}

impl CodexProvider {
    pub fn new(auth_path: PathBuf, base_url: impl Into<String>, token_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            auth_path,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            token_url: token_url.into(),
            effort: "medium".into(),
        }
    }

    /// `$CODEX_HOME/auth.json` (default `~/.codex/auth.json`); reasoning effort
    /// from `CODEX_REASONING_EFFORT` (low | medium | high | xhigh, default medium).
    pub fn from_env() -> anyhow::Result<Self> {
        let home = match std::env::var_os("CODEX_HOME").filter(|v| !v.is_empty()) {
            Some(h) => PathBuf::from(h),
            None => dirs::home_dir().context("no home directory")?.join(".codex"),
        };
        let mut p = Self::new(home.join("auth.json"), BASE_URL, TOKEN_URL);
        if let Ok(e) = std::env::var("CODEX_REASONING_EFFORT") {
            if !e.trim().is_empty() {
                p.effort = e.trim().to_ascii_lowercase();
            }
        }
        p.load()?;
        Ok(p)
    }

    fn load(&self) -> anyhow::Result<Tokens> {
        let missing = || format!("{} not found or not signed in — run `codex login` first", self.auth_path.display());
        let text = std::fs::read_to_string(&self.auth_path).with_context(missing)?;
        let root: Value = serde_json::from_str(&text).with_context(missing)?;
        let t = &root["tokens"];
        let s = |k: &str| t[k].as_str().map(str::to_string);
        match (s("access_token"), s("refresh_token"), s("account_id")) {
            (Some(access), Some(refresh), Some(account)) => Ok(Tokens { access, refresh, account }),
            _ => anyhow::bail!(missing()),
        }
    }

    /// Current tokens, refreshed first when the access token expires within 5 minutes.
    async fn tokens(&self) -> anyhow::Result<Tokens> {
        let t = self.load()?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        match jwt_exp(&t.access) {
            Some(exp) if exp < now + 300 => self.refresh(t).await,
            _ => Ok(t),
        }
    }

    /// Trade the refresh token for new tokens and write them back to auth.json,
    /// keeping every other key the Codex CLI stores there.
    async fn refresh(&self, t: Tokens) -> anyhow::Result<Tokens> {
        let resp = self
            .client
            .post(&self.token_url)
            .json(&json!({
                "grant_type": "refresh_token",
                "refresh_token": t.refresh,
                "client_id": CLIENT_ID,
                "scope": "openid profile email offline_access",
            }))
            .send()
            .await
            .context("refreshing Codex token")?;
        if !resp.status().is_success() {
            anyhow::bail!("codex token refresh failed ({}) — run `codex login` again", resp.status());
        }
        let v: Value = resp.json().await.context("decoding Codex token refresh")?;
        let access = v["access_token"].as_str().context("token refresh returned no access_token")?.to_string();
        let refresh = v["refresh_token"].as_str().map(str::to_string).unwrap_or(t.refresh);
        let account = jwt_account(&access).unwrap_or(t.account);

        let mut root: Value = std::fs::read_to_string(&self.auth_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| json!({}));
        let tokens = &mut root["tokens"];
        tokens["access_token"] = json!(access);
        tokens["refresh_token"] = json!(refresh);
        tokens["account_id"] = json!(account);
        if let Some(id) = v["id_token"].as_str() {
            tokens["id_token"] = json!(id);
        }
        std::fs::write(&self.auth_path, serde_json::to_vec_pretty(&root)?)
            .with_context(|| format!("writing {}", self.auth_path.display()))?;
        Ok(Tokens { access, refresh, account })
    }

    fn authed(&self, req: reqwest::RequestBuilder, t: &Tokens) -> reqwest::RequestBuilder {
        req.header("authorization", format!("Bearer {}", t.access))
            .header("chatgpt-account-id", &t.account)
            .header("openai-beta", "responses=v1")
            .header("user-agent", format!("codex_cli_rs/{CLIENT_VERSION}"))
    }

    /// Send with the current token; on 401 refresh once and retry.
    async fn send(&self, path: &str, body: Option<&Value>) -> anyhow::Result<reqwest::Response> {
        let url = format!("{}{path}?client_version={CLIENT_VERSION}", self.base_url);
        let mut t = self.tokens().await?;
        for attempt in 0..2 {
            let req = match body {
                Some(b) => self.client.post(&url).header("accept", "text/event-stream").json(b),
                None => self.client.get(&url),
            };
            let resp = self.authed(req, &t).send().await.context("request to Codex failed")?;
            let status = resp.status();
            if status.is_success() {
                return Ok(resp);
            }
            if status.as_u16() == 401 && attempt == 0 {
                t = self.refresh(t).await?;
                continue;
            }
            let text = resp.text().await.unwrap_or_default();
            return Err(http_error(status.as_u16(), &text));
        }
        unreachable!()
    }

    fn request_body(&self, req: &ProviderRequest) -> Value {
        let mut tools: Vec<Value> = req
            .tools
            .iter()
            .map(|t| json!({"type": "function", "name": t.name, "description": t.description, "parameters": t.input_schema}))
            .collect();
        // Hosted search: runs on OpenAI's side, billed to the subscription.
        tools.push(json!({"type": "web_search"}));
        json!({
            "model": req.model,
            "instructions": format!("{IDENTITY}\n\n{}", req.system),
            "input": to_input(&req.messages),
            "tools": tools,
            "store": false,
            "stream": true,
            "reasoning": {"effort": self.effort},
        })
    }
}

/// `exp` claim of a JWT (seconds since epoch).
fn jwt_exp(jwt: &str) -> Option<u64> {
    jwt_claims(jwt)?["exp"].as_u64()
}

fn jwt_account(jwt: &str) -> Option<String> {
    jwt_claims(jwt)?["https://api.openai.com/auth"]["chatgpt_account_id"].as_str().map(str::to_string)
}

fn jwt_claims(jwt: &str) -> Option<Value> {
    let payload = jwt.split('.').nth(1)?;
    serde_json::from_slice(&base64url(payload)?).ok()
}

fn base64url(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0);
    for c in s.bytes().take_while(|&c| c != b'=') {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'-' | b'+' => 62,
            b'_' | b'/' => 63,
            _ => return None,
        };
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Some(out)
}

fn http_error(status: u16, text: &str) -> anyhow::Error {
    let v: Value = serde_json::from_str(text).unwrap_or(Value::Null);
    let err = if v["error"].is_object() { &v["error"] } else { &v };
    if err["type"] == "usage_limit_reached" {
        let hours = err["resets_in_seconds"].as_u64().unwrap_or(0) as f64 / 3600.0;
        return anyhow::anyhow!("codex {status}: ChatGPT plan usage limit reached; resets in ~{hours:.1}h");
    }
    let msg = err["message"].as_str().or(err["detail"].as_str()).map(str::to_string).unwrap_or_else(|| text.to_string());
    let hint = if status == 401 { " — run `codex login` again" } else { "" };
    anyhow::anyhow!("codex {status}: {msg}{hint}")
}

/// Conversation → Responses `input` items. Tool calls and results become
/// `function_call` / `function_call_output` items (no server-side state: `store:false`).
pub(crate) fn to_input(history: &[Message]) -> Vec<Value> {
    let mut out = Vec::new();
    for m in history {
        let (role, kind) = match m.role {
            Role::User => ("user", "input_text"),
            Role::Assistant => ("assistant", "output_text"),
        };
        for b in &m.content {
            out.push(match b {
                ContentBlock::Text { text } => json!({"type": "message", "role": role, "content": [{"type": kind, "text": text}]}),
                ContentBlock::ToolUse { id, name, input } => {
                    json!({"type": "function_call", "call_id": id, "name": name, "arguments": input.to_string()})
                }
                ContentBlock::ToolResult { tool_use_id, content, .. } => {
                    json!({"type": "function_call_output", "call_id": tool_use_id, "output": content})
                }
            });
        }
    }
    out
}

#[async_trait]
impl Provider for CodexProvider {
    fn name(&self) -> &str {
        "codex"
    }

    /// Empty: the model set depends on the ChatGPT plan, so ask `/models`.
    fn default_model(&self) -> &str {
        ""
    }

    /// `/models`: only `visibility: "list"` entries are user-selectable; the
    /// server's `priority` puts the newest first.
    async fn list_models(&self) -> anyhow::Result<Vec<ModelInfo>> {
        let v: Value = self.send("/models", None).await?.json().await.context("decoding Codex model list")?;
        let items = v["models"].as_array().context("Codex /models reply has no `models` array")?;
        let mut listed: Vec<&Value> = items.iter().filter(|m| m["visibility"] == "list").collect();
        listed.sort_by_key(|m| m["priority"].as_i64().unwrap_or(i64::MAX));
        Ok(listed
            .into_iter()
            .filter_map(|m| {
                let id = m["slug"].as_str()?.to_string();
                Some(ModelInfo {
                    display_name: m["display_name"].as_str().unwrap_or(&id).to_string(),
                    id,
                    created_at: String::new(),
                    max_input_tokens: m["context_window"].as_u64(),
                    max_tokens: None,
                })
            })
            .collect())
    }

    async fn complete(&self, req: ProviderRequest) -> anyhow::Result<ProviderResponse> {
        self.complete_stream(req, &mut |_| {}).await
    }

    async fn complete_stream(
        &self,
        req: ProviderRequest,
        on_text: &mut (dyn for<'a> FnMut(&'a str) + Send),
    ) -> anyhow::Result<ProviderResponse> {
        let resp = self.send("/responses", Some(&self.request_body(&req))).await?;
        read_stream(resp, on_text, "codex").await
    }
}

/// Parse a Responses API SSE stream (Codex backend or api.openai.com
/// `/v1/responses`) into one assistant turn. `who` prefixes stream errors.
pub(crate) async fn read_stream(
    resp: reqwest::Response,
    on_text: &mut (dyn for<'a> FnMut(&'a str) + Send),
    who: &str,
) -> anyhow::Result<ProviderResponse> {
    let mut body = resp.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();
    let mut content = Vec::new();
    let (mut stop, mut input_tokens, mut output_tokens) = (StopReason::EndTurn, 0, 0);

    while let Some(chunk) = body.next().await {
        buf.extend_from_slice(&chunk.with_context(|| format!("reading {who} stream"))?);
        while let Some(nl) = buf.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buf.drain(..=nl).collect();
            let line = String::from_utf8_lossy(&line);
            let Some(data) = line.trim_end().strip_prefix("data:") else { continue };
            let Ok(ev) = serde_json::from_str::<Value>(data.trim_start()) else { continue };
            match ev["type"].as_str().unwrap_or_default() {
                "response.output_text.delta" => {
                    if let Some(d) = ev["delta"].as_str() {
                        on_text(d);
                    }
                }
                // Completed items carry the full text / call arguments.
                "response.output_item.done" => {
                    let item = &ev["item"];
                    match item["type"].as_str() {
                        Some("message") => {
                            let text: String =
                                item["content"].as_array().into_iter().flatten().filter_map(|c| c["text"].as_str()).collect();
                            if !text.is_empty() {
                                content.push(ContentBlock::Text { text });
                            }
                        }
                        Some("function_call") => {
                            let name = item["name"].as_str().unwrap_or_default().to_string();
                            let args = item["arguments"].as_str().filter(|a| !a.trim().is_empty()).unwrap_or("{}");
                            let input = serde_json::from_str(args)
                                .with_context(|| format!("model sent invalid JSON arguments for `{name}`: {args}"))?;
                            let id = item["call_id"].as_str().or(item["id"].as_str()).unwrap_or_default().to_string();
                            content.push(ContentBlock::ToolUse { id, name, input });
                        }
                        _ => {}
                    }
                }
                "response.completed" | "response.incomplete" => {
                    let r = &ev["response"];
                    input_tokens = r["usage"]["input_tokens"].as_u64().unwrap_or(0);
                    output_tokens = r["usage"]["output_tokens"].as_u64().unwrap_or(0);
                    if ev["type"] == "response.incomplete" {
                        stop = match r["incomplete_details"]["reason"].as_str() {
                            Some("max_output_tokens") => StopReason::MaxTokens,
                            _ => StopReason::Other,
                        };
                    }
                }
                "response.failed" => {
                    let msg = ev["response"]["error"]["message"].as_str().unwrap_or("response.failed");
                    anyhow::bail!("{who}: {msg}");
                }
                "error" => {
                    let msg = ev["message"].as_str().or(ev["error"]["message"].as_str()).unwrap_or("stream error");
                    anyhow::bail!("{who}: {msg}");
                }
                _ => {}
            }
        }
    }

    if content.iter().any(|b| matches!(b, ContentBlock::ToolUse { .. })) {
        stop = StopReason::ToolUse;
    }
    Ok(ProviderResponse {
        message: Message { role: Role::Assistant, content },
        stop_reason: stop,
        input_tokens,
        output_tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentiloop_core::ToolSpec;
    use std::sync::{Arc, Mutex};

    fn b64(s: &str) -> String {
        const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let bytes = s.as_bytes();
        let mut out = String::new();
        for c in bytes.chunks(3) {
            let n = (u32::from(c[0]) << 16) | (u32::from(*c.get(1).unwrap_or(&0)) << 8) | u32::from(*c.get(2).unwrap_or(&0));
            for i in 0..=c.len() {
                out.push(A[(n >> (18 - 6 * i) & 63) as usize] as char);
            }
        }
        out
    }

    fn jwt(exp: u64, account: &str) -> String {
        let claims = json!({"exp": exp, "https://api.openai.com/auth": {"chatgpt_account_id": account}});
        format!("h.{}.s", b64(&claims.to_string()))
    }

    fn auth_file(access: &str) -> PathBuf {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("agentiloop-codex-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("auth.json");
        let root = json!({"auth_mode": "chatgpt", "OPENAI_API_KEY": null,
            "tokens": {"access_token": access, "refresh_token": "r0", "account_id": "acct", "id_token": "i0"}});
        std::fs::write(&path, root.to_string()).unwrap();
        path
    }

    /// Serves canned replies in order, one per connection, recording each
    /// request's first line, headers and body.
    async fn serve(replies: Vec<(&'static str, String)>) -> (String, Arc<Mutex<Vec<String>>>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = seen.clone();
        tokio::spawn(async move {
            for (status, body) in replies {
                let (mut sock, _) = listener.accept().await.unwrap();
                let mut req = Vec::new();
                let mut chunk = vec![0u8; 65536];
                loop {
                    let n = sock.read(&mut chunk).await.unwrap();
                    req.extend_from_slice(&chunk[..n]);
                    let text = String::from_utf8_lossy(&req);
                    if let Some(end) = text.find("\r\n\r\n") {
                        let len = text[..end]
                            .lines()
                            .find_map(|l| l.to_ascii_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse::<usize>().unwrap()))
                            .unwrap_or(0);
                        if req.len() >= end + 4 + len || n == 0 {
                            break;
                        }
                    }
                }
                log.lock().unwrap().push(String::from_utf8_lossy(&req).into_owned());
                let head = format!("HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n", body.len());
                sock.write_all(head.as_bytes()).await.unwrap();
                sock.write_all(body.as_bytes()).await.unwrap();
                sock.shutdown().await.unwrap();
            }
        });
        (format!("http://{addr}"), seen)
    }

    fn req() -> ProviderRequest {
        ProviderRequest {
            model: "gpt-5.5".into(),
            system: "sys".into(),
            messages: vec![Message::user_text("hi")],
            tools: vec![ToolSpec { name: "list_dir".into(), description: "d".into(), input_schema: json!({"type": "object"}) }],
            max_tokens: 16,
        }
    }

    const FAR: u64 = 4_000_000_000;

    // Event shapes captured from chatgpt.com/backend-api/codex/responses.
    fn sse(events: &[Value]) -> String {
        events.iter().map(|e| format!("event: {}\ndata: {e}\n\n", e["type"].as_str().unwrap())).collect()
    }

    #[test]
    fn jwt_claims_decode() {
        let t = jwt(1234, "acct_9");
        assert_eq!(jwt_exp(&t), Some(1234));
        assert_eq!(jwt_account(&t).as_deref(), Some("acct_9"));
        assert_eq!(jwt_exp("opaque"), None);
    }

    #[test]
    fn history_becomes_responses_items() {
        let history = vec![
            Message::user_text("list"),
            Message {
                role: Role::Assistant,
                content: vec![
                    ContentBlock::Text { text: "ok".into() },
                    ContentBlock::ToolUse { id: "call_1".into(), name: "list_dir".into(), input: json!({"path": "."}) },
                ],
            },
            Message::tool_results(vec![ContentBlock::ToolResult { tool_use_id: "call_1".into(), content: "a.rs".into(), is_error: false }]),
        ];
        assert_eq!(
            Value::Array(to_input(&history)),
            json!([
                {"type": "message", "role": "user", "content": [{"type": "input_text", "text": "list"}]},
                {"type": "message", "role": "assistant", "content": [{"type": "output_text", "text": "ok"}]},
                {"type": "function_call", "call_id": "call_1", "name": "list_dir", "arguments": "{\"path\":\".\"}"},
                {"type": "function_call_output", "call_id": "call_1", "output": "a.rs"}
            ])
        );
    }

    #[test]
    fn request_body_shape() {
        let p = CodexProvider::new(PathBuf::from("/nonexistent"), BASE_URL, TOKEN_URL);
        let b = p.request_body(&req());
        assert!(b["instructions"].as_str().unwrap().starts_with(IDENTITY));
        assert!(b["instructions"].as_str().unwrap().ends_with("sys"));
        assert_eq!(b["stream"], true);
        assert_eq!(b["store"], false);
        assert_eq!(b["reasoning"]["effort"], "medium");
        // The backend 400s on max_output_tokens.
        assert!(b.get("max_output_tokens").is_none());
        assert_eq!(b["tools"][0]["name"], "list_dir");
        assert_eq!(b["tools"][1], json!({"type": "web_search"}));
    }

    #[tokio::test]
    async fn stream_assembles_text_tool_call_and_usage() {
        let body = sse(&[
            json!({"type": "response.created"}),
            json!({"type": "response.output_text.delta", "item_id": "m1", "delta": "Let me "}),
            json!({"type": "response.output_text.delta", "item_id": "m1", "delta": "look."}),
            json!({"type": "response.output_item.done", "item": {"type": "message", "id": "m1", "content": [{"type": "output_text", "text": "Let me look."}]}}),
            json!({"type": "response.function_call_arguments.delta", "item_id": "fc1", "delta": "{\"pa"}),
            json!({"type": "response.output_item.done", "item": {"type": "function_call", "id": "fc1", "call_id": "call_9", "name": "list_dir", "arguments": "{\"path\":\"/tmp\"}"}}),
            json!({"type": "response.output_item.done", "item": {"type": "web_search_call", "id": "ws1"}}),
            json!({"type": "response.completed", "response": {"usage": {"input_tokens": 43, "output_tokens": 5}}}),
        ]);
        let (base, seen) = serve(vec![("200 OK", body)]).await;
        let p = CodexProvider::new(auth_file(&jwt(FAR, "acct")), base, "http://unused");
        let mut deltas = Vec::new();
        let resp = p.complete_stream(req(), &mut |t| deltas.push(t.to_string())).await.unwrap();
        assert_eq!(deltas, vec!["Let me ", "look."]);
        assert_eq!(resp.message.text(), "Let me look.");
        assert_eq!(resp.stop_reason, StopReason::ToolUse);
        assert_eq!((resp.input_tokens, resp.output_tokens), (43, 5));
        let calls: Vec<_> = resp.message.tool_uses().collect();
        assert_eq!(calls, vec![("call_9", "list_dir", &json!({"path": "/tmp"}))]);

        let r = seen.lock().unwrap()[0].to_ascii_lowercase();
        assert!(r.starts_with("post /responses?client_version="), "{r}");
        assert!(r.contains("chatgpt-account-id: acct"));
        assert!(r.contains("openai-beta: responses=v1"));
    }

    #[tokio::test]
    async fn failed_event_is_an_error() {
        let body = sse(&[json!({"type": "response.failed", "response": {"error": {"message": "You have no credits remaining"}}})]);
        let (base, _) = serve(vec![("200 OK", body)]).await;
        let p = CodexProvider::new(auth_file(&jwt(FAR, "acct")), base, "http://unused");
        let err = p.complete(req()).await.unwrap_err().to_string();
        assert!(err.contains("no credits remaining"), "{err}");
    }

    #[tokio::test]
    async fn list_models_filters_hidden_and_sorts_by_priority() {
        let models = json!({"models": [
            {"slug": "gpt-5.5", "display_name": "GPT-5.5", "visibility": "list", "priority": 12, "context_window": 272000},
            {"slug": "gpt-reserve", "visibility": "hide", "priority": 3},
            {"slug": "gpt-6-astra", "display_name": "GPT-6-Astra", "visibility": "list", "priority": 1, "context_window": 272000}
        ]});
        let (base, seen) = serve(vec![("200 OK", models.to_string())]).await;
        let p = CodexProvider::new(auth_file(&jwt(FAR, "acct")), base, "http://unused");
        let m = p.list_models().await.unwrap();
        let ids: Vec<_> = m.iter().map(|m| (m.id.as_str(), m.display_name.as_str(), m.max_input_tokens)).collect();
        assert_eq!(ids, vec![("gpt-6-astra", "GPT-6-Astra", Some(272000)), ("gpt-5.5", "GPT-5.5", Some(272000))]);
        assert!(seen.lock().unwrap()[0].starts_with("GET /models?client_version="));
    }

    #[tokio::test]
    async fn expiring_token_is_refreshed_and_written_back() {
        let fresh = jwt(FAR, "acct_new");
        let (token_url, seen) =
            serve(vec![("200 OK", json!({"access_token": fresh, "refresh_token": "r1", "id_token": "i1"}).to_string())]).await;
        let (base, api_seen) = serve(vec![("200 OK", json!({"models": []}).to_string())]).await;
        let path = auth_file(&jwt(1, "acct"));
        let p = CodexProvider::new(path.clone(), base, token_url);
        p.list_models().await.unwrap();

        let refresh_req = &seen.lock().unwrap()[0];
        assert!(refresh_req.contains("\"grant_type\":\"refresh_token\"") && refresh_req.contains("\"refresh_token\":\"r0\""));
        assert!(api_seen.lock().unwrap()[0].contains(&format!("Bearer {fresh}")));
        let saved: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(saved["tokens"]["access_token"], json!(fresh));
        assert_eq!(saved["tokens"]["refresh_token"], "r1");
        assert_eq!(saved["tokens"]["account_id"], "acct_new");
        assert_eq!(saved["auth_mode"], "chatgpt", "other keys survive");
    }

    #[tokio::test]
    async fn unauthorized_refreshes_once_and_retries() {
        let fresh = jwt(FAR, "acct");
        let (token_url, _) = serve(vec![("200 OK", json!({"access_token": fresh}).to_string())]).await;
        let (base, seen) = serve(vec![
            ("401 Unauthorized", json!({"detail": "revoked"}).to_string()),
            ("200 OK", json!({"models": []}).to_string()),
        ])
        .await;
        let p = CodexProvider::new(auth_file(&jwt(FAR, "acct")), base, token_url);
        p.list_models().await.unwrap();
        assert!(seen.lock().unwrap()[1].contains(&format!("Bearer {fresh}")));
    }

    #[tokio::test]
    async fn errors_are_readable() {
        let (base, _) = serve(vec![(
            "429 Too Many Requests",
            json!({"error": {"type": "usage_limit_reached", "resets_in_seconds": 7200}}).to_string(),
        )])
        .await;
        let p = CodexProvider::new(auth_file(&jwt(FAR, "acct")), base, "http://unused");
        let err = p.list_models().await.unwrap_err().to_string();
        assert!(err.contains("usage limit reached; resets in ~2.0h"), "{err}");

        let p = CodexProvider::new(PathBuf::from("/nonexistent/auth.json"), BASE_URL, TOKEN_URL);
        let err = p.list_models().await.unwrap_err().to_string();
        assert!(err.contains("run `codex login`"), "{err}");
    }
}
