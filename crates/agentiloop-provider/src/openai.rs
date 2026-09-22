//! OpenAI-compatible Chat Completions backend. Works with OpenAI itself and
//! anything that speaks the same wire format (Ollama `/v1`, LM Studio, Groq,
//! OpenRouter, Together, DeepSeek, vLLM, …) — point `OPENAI_BASE_URL` at it.

use agentiloop_core::{ContentBlock, Message, ModelInfo, Provider, ProviderRequest, ProviderResponse, Role, StopReason};
use anyhow::Context;
use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    name: &'static str,
    default_model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into().trim().to_string(),
            base_url: base_url.trim_end_matches('/').to_string(),
            name: "openai",
            default_model: "gpt-4o-mini".into(),
        }
    }

    /// Rebrand this backend for a server that speaks the OpenAI wire format
    /// under its own name (e.g. `omlx`), so settings.json and the status line
    /// key on that name. An empty `default_model` means "ask `/models`".
    pub fn with_identity(mut self, name: &'static str, default_model: impl Into<String>) -> Self {
        self.name = name;
        self.default_model = default_model.into();
        self
    }

    /// Reads `OPENAI_API_KEY` and `OPENAI_BASE_URL` (default `https://api.openai.com/v1`).
    /// Local servers such as Ollama ignore the key, so it may be omitted when a
    /// custom base URL is set.
    pub fn from_env() -> anyhow::Result<Self> {
        let base_url = std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.into());
        let key = match std::env::var("OPENAI_API_KEY") {
            Ok(k) => k,
            Err(_) if base_url != DEFAULT_BASE_URL => "none".into(),
            Err(_) => anyhow::bail!("OPENAI_API_KEY is not set (or set OPENAI_BASE_URL for a local server)"),
        };
        Ok(Self::new(key, base_url))
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.header("authorization", format!("Bearer {}", self.api_key))
    }

    async fn send_chat(&self, req: &ProviderRequest, stream: bool) -> anyhow::Result<reqwest::Response> {
        let body = WireRequest {
            model: &req.model,
            max_tokens: req.max_tokens,
            messages: to_wire_messages(&req.system, &req.messages),
            tools: req
                .tools
                .iter()
                .map(|t| WireTool {
                    kind: "function",
                    function: WireFunction { name: &t.name, description: &t.description, parameters: &t.input_schema },
                })
                .collect(),
            stream,
            stream_options: stream.then_some(WireStreamOptions { include_usage: true }),
        };

        let resp = self
            .auth(self.client.post(format!("{}/chat/completions", self.base_url)))
            .json(&body)
            .send()
            .await
            .with_context(|| format!("request to {} failed", self.base_url))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.context("reading error body")?;
            if let Ok(err) = serde_json::from_str::<WireError>(&text) {
                anyhow::bail!("OpenAI {} ({}): {}", status, err.error.kind.unwrap_or_default(), err.error.message);
            }
            anyhow::bail!("OpenAI {}: {}", status, text);
        }
        Ok(resp)
    }
}

// ---- request wire types -----------------------------------------------------

#[derive(Serialize)]
struct WireRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<WireMessage<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<WireTool<'a>>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<WireStreamOptions>,
}

#[derive(Serialize)]
struct WireStreamOptions {
    include_usage: bool,
}

#[derive(Serialize)]
struct WireTool<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    function: WireFunction<'a>,
}

#[derive(Serialize)]
struct WireFunction<'a> {
    name: &'a str,
    description: &'a str,
    parameters: &'a Value,
}

#[derive(Serialize)]
#[serde(tag = "role", rename_all = "lowercase")]
enum WireMessage<'a> {
    System { content: &'a str },
    User { content: &'a str },
    Assistant {
        content: Option<String>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        tool_calls: Vec<WireToolCall>,
    },
    Tool { tool_call_id: &'a str, content: &'a str },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WireToolCall {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    function: WireToolCallFn,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WireToolCallFn {
    name: String,
    /// JSON-encoded arguments (a string on the wire, not an object).
    arguments: String,
}

/// Flatten our Anthropic-shaped history into OpenAI's role-per-message form:
/// tool results become individual `tool` messages, assistant tool uses become
/// `tool_calls` with stringified arguments.
fn to_wire_messages<'a>(system: &'a str, history: &'a [Message]) -> Vec<WireMessage<'a>> {
    let mut out = Vec::with_capacity(history.len() + 1);
    if !system.is_empty() {
        out.push(WireMessage::System { content: system });
    }
    for m in history {
        match m.role {
            Role::User => {
                for block in &m.content {
                    match block {
                        ContentBlock::Text { text } => out.push(WireMessage::User { content: text }),
                        ContentBlock::ToolResult { tool_use_id, content, .. } => {
                            out.push(WireMessage::Tool { tool_call_id: tool_use_id, content })
                        }
                        ContentBlock::ToolUse { .. } => {}
                    }
                }
            }
            Role::Assistant => {
                let text = m.text();
                let tool_calls = m
                    .tool_uses()
                    .map(|(id, name, input)| WireToolCall {
                        id: id.to_string(),
                        kind: "function".into(),
                        function: WireToolCallFn { name: name.to_string(), arguments: input.to_string() },
                    })
                    .collect();
                out.push(WireMessage::Assistant { content: (!text.is_empty()).then_some(text), tool_calls });
            }
        }
    }
    out
}

// ---- response wire types ----------------------------------------------------

#[derive(Deserialize)]
struct WireResponse {
    choices: Vec<WireChoice>,
    #[serde(default)]
    usage: Option<WireUsage>,
}

#[derive(Deserialize)]
struct WireChoice {
    message: WireRespMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct WireRespMessage {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<WireToolCall>,
}

#[derive(Deserialize, Default)]
struct WireUsage {
    #[serde(default)]
    prompt_tokens: u64,
    #[serde(default)]
    completion_tokens: u64,
}

#[derive(Deserialize)]
struct WireError {
    error: WireErrorBody,
}

#[derive(Deserialize)]
struct WireErrorBody {
    #[serde(rename = "type", default)]
    kind: Option<String>,
    message: String,
}

#[derive(Deserialize)]
struct WireModelList {
    data: Vec<WireModel>,
}

#[derive(Deserialize)]
struct WireModel {
    id: String,
    #[serde(default)]
    created: u64,
}

// streaming

#[derive(Deserialize)]
struct WireChunk {
    #[serde(default)]
    choices: Vec<WireChunkChoice>,
    #[serde(default)]
    usage: Option<WireUsage>,
}

#[derive(Deserialize)]
struct WireChunkChoice {
    delta: WireDelta,
    finish_reason: Option<String>,
}

#[derive(Deserialize, Default)]
struct WireDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<WireToolCallDelta>,
}

#[derive(Deserialize)]
struct WireToolCallDelta {
    index: usize,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    function: Option<WireToolCallFnDelta>,
}

#[derive(Deserialize, Default)]
struct WireToolCallFnDelta {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

fn parse_finish(raw: Option<&str>, has_tool_calls: bool) -> StopReason {
    match raw {
        Some("tool_calls") | Some("function_call") => StopReason::ToolUse,
        Some("stop") if has_tool_calls => StopReason::ToolUse, // some servers report "stop" alongside tool calls
        Some("stop") => StopReason::EndTurn,
        Some("length") => StopReason::MaxTokens,
        _ if has_tool_calls => StopReason::ToolUse,
        _ => StopReason::Other,
    }
}

fn parse_tool_args(name: &str, raw: &str) -> anyhow::Result<Value> {
    if raw.trim().is_empty() {
        return Ok(Value::Object(Default::default()));
    }
    serde_json::from_str(raw).with_context(|| format!("decoding tool arguments for `{name}`: {raw}"))
}

fn build_content(text: String, calls: Vec<WireToolCall>) -> anyhow::Result<Vec<ContentBlock>> {
    let mut content = Vec::with_capacity(1 + calls.len());
    if !text.is_empty() {
        content.push(ContentBlock::Text { text });
    }
    for c in calls {
        let input = parse_tool_args(&c.function.name, &c.function.arguments)?;
        content.push(ContentBlock::ToolUse { id: c.id, name: c.function.name, input });
    }
    Ok(content)
}

#[async_trait]
impl Provider for OpenAIProvider {
    fn name(&self) -> &str {
        self.name
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    /// `GET /models`; sorted newest first by `created` when present.
    async fn list_models(&self) -> anyhow::Result<Vec<ModelInfo>> {
        let resp = self
            .auth(self.client.get(format!("{}/models", self.base_url)))
            .send()
            .await
            .context("request to /models failed")?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            anyhow::bail!("OpenAI {}: {}", status, text);
        }
        let mut list: WireModelList = serde_json::from_str(&text).context("decoding /models")?;
        list.data.sort_by(|a, b| b.created.cmp(&a.created).then_with(|| a.id.cmp(&b.id)));
        Ok(list
            .data
            .into_iter()
            .map(|m| ModelInfo { id: m.id.clone(), display_name: m.id, created_at: String::new() })
            .collect())
    }

    async fn complete(&self, req: ProviderRequest) -> anyhow::Result<ProviderResponse> {
        let resp = self.send_chat(&req, false).await?;
        let text = resp.text().await.context("reading response body")?;
        let wire: WireResponse = serde_json::from_str(&text).context("decoding OpenAI response")?;
        let choice = wire.choices.into_iter().next().context("OpenAI response had no choices")?;
        let has_calls = !choice.message.tool_calls.is_empty();
        let usage = wire.usage.unwrap_or_default();

        Ok(ProviderResponse {
            message: Message {
                role: Role::Assistant,
                content: build_content(choice.message.content.unwrap_or_default(), choice.message.tool_calls)?,
            },
            stop_reason: parse_finish(choice.finish_reason.as_deref(), has_calls),
            input_tokens: usage.prompt_tokens,
            output_tokens: usage.completion_tokens,
        })
    }

    async fn complete_stream(
        &self,
        req: ProviderRequest,
        on_text: &mut (dyn for<'a> FnMut(&'a str) + Send),
    ) -> anyhow::Result<ProviderResponse> {
        let resp = self.send_chat(&req, true).await?;
        let mut body = resp.bytes_stream();

        let mut buf: Vec<u8> = Vec::new();
        let mut text = String::new();
        // Tool calls arrive as deltas keyed by `index`; arguments are concatenated.
        let mut calls: Vec<WireToolCall> = Vec::new();
        let mut finish: Option<String> = None;
        let mut usage = WireUsage::default();

        'outer: while let Some(chunk) = body.next().await {
            buf.extend_from_slice(&chunk.context("reading OpenAI stream")?);
            while let Some(nl) = buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buf.drain(..=nl).collect();
                let line = String::from_utf8_lossy(&line);
                let Some(data) = line.trim_end().strip_prefix("data:") else { continue };
                let data = data.trim_start();
                if data == "[DONE]" {
                    break 'outer;
                }
                let chunk: WireChunk = serde_json::from_str(data).with_context(|| format!("decoding stream chunk: {data}"))?;
                if let Some(u) = chunk.usage {
                    usage = u;
                }
                for choice in chunk.choices {
                    if let Some(t) = choice.delta.content.as_deref().filter(|t| !t.is_empty()) {
                        on_text(t);
                        text.push_str(t);
                    }
                    for tc in choice.delta.tool_calls {
                        if calls.len() <= tc.index {
                            calls.resize_with(tc.index + 1, || WireToolCall {
                                id: String::new(),
                                kind: "function".into(),
                                function: WireToolCallFn { name: String::new(), arguments: String::new() },
                            });
                        }
                        let slot = &mut calls[tc.index];
                        if let Some(id) = tc.id.filter(|s| !s.is_empty()) {
                            slot.id = id;
                        }
                        if let Some(f) = tc.function {
                            if let Some(n) = f.name.filter(|s| !s.is_empty()) {
                                slot.function.name = n;
                            }
                            if let Some(a) = f.arguments {
                                slot.function.arguments.push_str(&a);
                            }
                        }
                    }
                    if choice.finish_reason.is_some() {
                        finish = choice.finish_reason;
                    }
                }
            }
        }

        // Servers that omit ids would break tool_result pairing; synthesize one.
        for (i, c) in calls.iter_mut().enumerate() {
            if c.id.is_empty() {
                c.id = format!("call_{i}");
            }
        }
        let has_calls = !calls.is_empty();

        Ok(ProviderResponse {
            message: Message { role: Role::Assistant, content: build_content(text, calls)? },
            stop_reason: parse_finish(finish.as_deref(), has_calls),
            input_tokens: usage.prompt_tokens,
            output_tokens: usage.completion_tokens,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_flattens_to_openai_roles() {
        let history = vec![
            Message::user_text("hi"),
            Message {
                role: Role::Assistant,
                content: vec![
                    ContentBlock::Text { text: "checking".into() },
                    ContentBlock::ToolUse { id: "call_1".into(), name: "list_dir".into(), input: serde_json::json!({"path": "."}) },
                ],
            },
            Message::tool_results(vec![ContentBlock::ToolResult { tool_use_id: "call_1".into(), content: "a.rs".into(), is_error: false }]),
        ];
        let wire = serde_json::to_value(to_wire_messages("sys", &history)).unwrap();
        assert_eq!(
            wire,
            serde_json::json!([
                {"role": "system", "content": "sys"},
                {"role": "user", "content": "hi"},
                {"role": "assistant", "content": "checking", "tool_calls": [
                    {"id": "call_1", "type": "function", "function": {"name": "list_dir", "arguments": "{\"path\":\".\"}"}}
                ]},
                {"role": "tool", "tool_call_id": "call_1", "content": "a.rs"}
            ])
        );
    }

    /// Serve one canned HTTP body on a local port in small chunks (splits SSE
    /// lines mid-way to exercise the buffering), then close.
    async fn serve_once(body: &'static str) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut req = vec![0u8; 8192];
            let _ = sock.read(&mut req).await;
            let head = "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\nconnection: close\r\n\r\n";
            sock.write_all(head.as_bytes()).await.unwrap();
            for chunk in body.as_bytes().chunks(41) {
                sock.write_all(chunk).await.unwrap();
                sock.flush().await.unwrap();
            }
            sock.shutdown().await.unwrap();
        });
        format!("http://{addr}")
    }

    fn req() -> ProviderRequest {
        ProviderRequest {
            model: "m".into(),
            system: "s".into(),
            messages: vec![Message::user_text("hi")],
            tools: vec![],
            max_tokens: 16,
        }
    }

    // Shape captured from Ollama's /v1 endpoint: tool_call id+name in one chunk,
    // arguments fragmented across later chunks, usage in a trailing choices-less chunk.
    const SSE_TOOL: &str = "\
data: {\"id\":\"c1\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"Let me \"},\"finish_reason\":null}]}

data: {\"id\":\"c1\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"look.\"},\"finish_reason\":null}]}

data: {\"id\":\"c1\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"id\":\"call_abc\",\"index\":0,\"type\":\"function\",\"function\":{\"name\":\"list_dir\",\"arguments\":\"\"}}]},\"finish_reason\":null}]}

data: {\"id\":\"c1\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"{\\\"pa\"}}]},\"finish_reason\":null}]}

data: {\"id\":\"c1\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"th\\\":\\\"/tmp\\\"}\"}}]},\"finish_reason\":null}]}

data: {\"id\":\"c1\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"tool_calls\"}]}

data: {\"id\":\"c1\",\"object\":\"chat.completion.chunk\",\"choices\":[],\"usage\":{\"prompt_tokens\":139,\"completion_tokens\":21,\"total_tokens\":160}}

data: [DONE]

";

    #[tokio::test]
    async fn stream_assembles_text_and_fragmented_tool_call() {
        let base = serve_once(SSE_TOOL).await;
        let p = OpenAIProvider::new("k", base);
        let mut deltas = Vec::new();
        let resp = p.complete_stream(req(), &mut |t| deltas.push(t.to_string())).await.unwrap();

        assert_eq!(deltas, vec!["Let me ", "look."]);
        assert_eq!(resp.message.text(), "Let me look.");
        assert_eq!(resp.stop_reason, StopReason::ToolUse);
        assert_eq!((resp.input_tokens, resp.output_tokens), (139, 21));
        let calls: Vec<_> = resp.message.tool_uses().collect();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "call_abc");
        assert_eq!(calls[0].1, "list_dir");
        assert_eq!(calls[0].2, &serde_json::json!({"path": "/tmp"}));
    }

    const SSE_TEXT: &str = "\
data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}

data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}

data: [DONE]

";

    #[tokio::test]
    async fn stream_plain_text_ends_turn_without_usage() {
        let base = serve_once(SSE_TEXT).await;
        let p = OpenAIProvider::new("k", base);
        let mut deltas = Vec::new();
        let resp = p.complete_stream(req(), &mut |t| deltas.push(t.to_string())).await.unwrap();
        assert_eq!(deltas, vec!["hi"]);
        assert_eq!(resp.stop_reason, StopReason::EndTurn);
        assert_eq!((resp.input_tokens, resp.output_tokens), (0, 0));
        assert_eq!(resp.message.tool_uses().count(), 0);
    }

    const NON_STREAM: &str = "{\"choices\":[{\"index\":0,\"message\":{\"role\":\"assistant\",\"content\":null,\"tool_calls\":[{\"id\":\"call_1\",\"type\":\"function\",\"function\":{\"name\":\"bash\",\"arguments\":\"{\\\"command\\\":\\\"ls\\\"}\"}}]},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":7,\"completion_tokens\":3}}";

    #[tokio::test]
    async fn non_streaming_tool_call_with_stop_finish_reason_is_tool_use() {
        let base = serve_once(NON_STREAM).await;
        let p = OpenAIProvider::new("k", base);
        let resp = p.complete(req()).await.unwrap();
        // Some servers say "stop" even when tool_calls are present; we must still loop.
        assert_eq!(resp.stop_reason, StopReason::ToolUse);
        let calls: Vec<_> = resp.message.tool_uses().collect();
        assert_eq!(calls[0].1, "bash");
        assert_eq!(calls[0].2, &serde_json::json!({"command": "ls"}));
        assert_eq!((resp.input_tokens, resp.output_tokens), (7, 3));
    }

    #[test]
    fn finish_reason_mapping() {
        assert_eq!(parse_finish(Some("stop"), false), StopReason::EndTurn);
        assert_eq!(parse_finish(Some("stop"), true), StopReason::ToolUse);
        assert_eq!(parse_finish(Some("tool_calls"), true), StopReason::ToolUse);
        assert_eq!(parse_finish(Some("length"), false), StopReason::MaxTokens);
        assert_eq!(parse_finish(None, false), StopReason::Other);
        assert_eq!(parse_finish(None, true), StopReason::ToolUse);
    }

    #[test]
    fn empty_tool_arguments_become_empty_object() {
        assert_eq!(parse_tool_args("x", "").unwrap(), serde_json::json!({}));
        assert_eq!(parse_tool_args("x", "  ").unwrap(), serde_json::json!({}));
        assert!(parse_tool_args("x", "{not json").is_err());
    }
}
