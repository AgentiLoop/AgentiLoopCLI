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
/// OpenAI's per-model doc pages (`<id>.md`), the only place OpenAI publishes
/// context windows and output caps — `/models` doesn't carry them.
const MODEL_DOCS_URL: &str = "https://developers.openai.com/api/docs/models";

pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model_docs_url: String,
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
            model_docs_url: MODEL_DOCS_URL.into(),
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

    /// Error for a non-2xx reply, named after this backend. 401/403 say
    /// outright that the API key is missing or wrong and where to set it.
    fn http_error(&self, status: reqwest::StatusCode, text: &str) -> anyhow::Error {
        let detail = match serde_json::from_str::<WireError>(text) {
            Ok(err) => format!("({}): {}", err.error.kind.unwrap_or_default(), err.error.message),
            Err(_) => format!(": {text}"),
        };
        let mut msg = format!("{} {status} {detail}", self.name);
        if matches!(status.as_u16(), 401 | 403) {
            let hint = match self.name {
                "omlx" => "set OMLX_API_KEY or auth.api_key in ~/.omlx/settings.json",
                _ => "set OPENAI_API_KEY",
            };
            msg.push_str(&format!(" — API key missing or invalid; {hint}"));
        }
        anyhow::anyhow!(msg)
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
            return Err(self.http_error(status, &text));
        }
        Ok(resp)
    }

    /// Ollama (local or ollama.com) mounts its native API beside `/v1`:
    /// `POST /api/show` reports the model's context window under
    /// `model_info["<family>.context_length"]`, capped by a `num_ctx`
    /// Modelfile parameter when set. Any other server 404s → None.
    async fn ollama_context_length(&self, id: &str) -> Option<u64> {
        let root = self.base_url.strip_suffix("/v1")?;
        let resp = self
            .auth(self.client.post(format!("{root}/api/show")))
            .json(&serde_json::json!({ "model": id }))
            .send()
            .await
            .ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let show: WireOllamaShow = resp.json().await.ok()?;
        let model_max = show
            .model_info
            .iter()
            .find(|(k, _)| k.ends_with(".context_length"))
            .and_then(|(_, v)| v.as_u64());
        let num_ctx = show.parameters.lines().find_map(|line| {
            let mut words = line.split_whitespace();
            (words.next()? == "num_ctx").then(|| words.next()?.parse::<u64>().ok())?
        });
        match (model_max, num_ctx) {
            (Some(m), Some(n)) => Some(m.min(n)),
            (m, n) => m.or(n),
        }
    }

    /// OpenAI's published `(context window, max output)` for one of its own
    /// models, fetched from the model's doc page. Dated snapshots
    /// (`o1-mini-2024-09-12`) have no page of their own, so the date suffix
    /// is dropped and the family page is used instead.
    async fn openai_published_limits(&self, id: &str) -> Option<(u64, u32)> {
        if !looks_like_openai_model(id) {
            return None;
        }
        let family = id
            .len()
            .checked_sub(11)
            .map(|at| id.split_at(at))
            .filter(|(_, tail)| tail.starts_with('-') && is_snapshot_date(&tail[1..]))
            .map(|(family, _)| family);
        for page in std::iter::once(id).chain(family) {
            let resp = self.client.get(format!("{}/{page}.md", self.model_docs_url)).send().await.ok()?;
            if !resp.status().is_success() {
                continue;
            }
            return parse_model_doc(&resp.text().await.ok()?);
        }
        None
    }
}

/// `YYYY-MM-DD` as used in OpenAI snapshot ids.
fn is_snapshot_date(s: &str) -> bool {
    s.len() == 10 && s.chars().enumerate().all(|(i, c)| if i == 4 || i == 7 { c == '-' } else { c.is_ascii_digit() })
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
    /// OpenRouter: context window.
    #[serde(default)]
    context_length: Option<u64>,
    /// vLLM and oMLX: context window.
    #[serde(default)]
    max_model_len: Option<u64>,
    /// OpenRouter: output cap lives under `top_provider`.
    #[serde(default)]
    top_provider: Option<WireTopProvider>,
}

#[derive(Deserialize)]
struct WireTopProvider {
    #[serde(default)]
    max_completion_tokens: Option<u32>,
}

/// Ollama's native `POST /api/show` reply (only the limit-bearing fields).
#[derive(Deserialize)]
struct WireOllamaShow {
    /// Keyed `"<family>.context_length"`, e.g. `"qwen3.context_length"`.
    #[serde(default)]
    model_info: std::collections::HashMap<String, Value>,
    /// Modelfile parameters, one `name value` per line (`num_ctx 8192`).
    #[serde(default)]
    parameters: String,
}

/// Pulls `(context window, max output)` out of an OpenAI model doc page
/// (`developers.openai.com/api/docs/models/<id>.md`). The "Model details"
/// list carries lines like `- 400,000 context window`,
/// `- Maximum input tokens: 272,000` and `- 128,000 max output tokens`;
/// the explicit input cap wins over the context window when both appear.
fn parse_model_doc(md: &str) -> Option<(u64, u32)> {
    fn number(md: &str, label: &str) -> Option<u64> {
        md.lines()
            .filter_map(|l| l.trim_start().strip_prefix("- "))
            .find(|l| l.contains(label))?
            .split(|c: char| !c.is_ascii_digit() && c != ',')
            .find(|w| w.chars().any(|c| c.is_ascii_digit()))?
            .replace(',', "")
            .parse()
            .ok()
    }
    let input = number(md, "Maximum input tokens").or_else(|| number(md, "context window"))?;
    let output = number(md, "max output tokens")?;
    Some((input, u32::try_from(output).ok()?))
}

/// Does this id look like one of OpenAI's own models (`gpt-*`, `chatgpt-*`, `o1`…`o4`)?
fn looks_like_openai_model(id: &str) -> bool {
    id.starts_with("gpt-")
        || id.starts_with("chatgpt-")
        || (id.starts_with('o') && id[1..].starts_with(|c: char| c.is_ascii_digit()))
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
            return Err(self.http_error(status, &text));
        }
        let mut list: WireModelList = serde_json::from_str(&text).context("decoding /models")?;
        list.data.sort_by(|a, b| b.created.cmp(&a.created).then_with(|| a.id.cmp(&b.id)));
        Ok(list
            .data
            .into_iter()
            .map(|m| ModelInfo {
                id: m.id.clone(),
                display_name: m.id,
                created_at: String::new(),
                max_input_tokens: m.context_length.or(m.max_model_len),
                max_tokens: m.top_provider.and_then(|t| t.max_completion_tokens),
            })
            .collect())
    }

    /// Scans `/models` (OpenRouter, vLLM and oMLX report limits there), then
    /// asks Ollama's `/api/show` for the context window when the catalog
    /// didn't carry one, and finally fills in OpenAI's published limits.
    async fn model_info(&self, id: &str) -> anyhow::Result<Option<ModelInfo>> {
        let Some(mut info) = self.list_models().await?.into_iter().find(|m| m.id == id) else {
            return Ok(None);
        };
        if info.max_input_tokens.is_none() {
            info.max_input_tokens = self.ollama_context_length(id).await;
        }
        if info.max_input_tokens.is_none() || info.max_tokens.is_none() {
            if let Some((input, output)) = self.openai_published_limits(id).await {
                info.max_input_tokens.get_or_insert(input);
                info.max_tokens.get_or_insert(output);
            }
        }
        Ok(Some(info))
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

    /// Serve canned JSON bodies keyed by request path (404 otherwise) for as
    /// many connections as the test makes.
    async fn serve_routes(routes: &'static [(&'static str, &'static str)]) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            loop {
                let (mut sock, _) = listener.accept().await.unwrap();
                let mut req = vec![0u8; 8192];
                let n = sock.read(&mut req).await.unwrap_or(0);
                let head = String::from_utf8_lossy(&req[..n]);
                let path = head.split_whitespace().nth(1).unwrap_or("");
                let (status, body) = match routes.iter().find(|(p, _)| *p == path) {
                    Some((_, body)) => ("200 OK", *body),
                    None => ("404 Not Found", "{}"),
                };
                let reply = format!(
                    "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                sock.write_all(reply.as_bytes()).await.unwrap();
                sock.shutdown().await.unwrap();
            }
        });
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn list_models_reads_limits_reported_by_catalog() {
        // OpenRouter shape (context_length + top_provider) and vLLM/oMLX shape (max_model_len).
        let base = serve_routes(&[(
            "/models",
            r#"{"data":[{"id":"a","context_length":131072,"top_provider":{"max_completion_tokens":8192}},{"id":"b","max_model_len":32768}]}"#,
        )])
        .await;
        let models = OpenAIProvider::new("k", base).list_models().await.unwrap();
        assert_eq!((models[0].max_input_tokens, models[0].max_tokens), (Some(131072), Some(8192)));
        assert_eq!((models[1].max_input_tokens, models[1].max_tokens), (Some(32768), None));
    }

    #[tokio::test]
    async fn model_info_asks_ollama_for_context_window_capped_by_num_ctx() {
        let base = serve_routes(&[
            ("/v1/models", r#"{"data":[{"id":"qwen3:4b"}]}"#),
            ("/api/show", r#"{"model_info":{"qwen3.context_length":262144},"parameters":"stop \"x\"\nnum_ctx 8192"}"#),
        ])
        .await;
        let p = OpenAIProvider::new("k", format!("{base}/v1"));
        let m = p.model_info("qwen3:4b").await.unwrap().unwrap();
        assert_eq!((m.max_input_tokens, m.max_tokens), (Some(8192), None));
        assert!(p.model_info("missing").await.unwrap().is_none());
    }

    // Trimmed from developers.openai.com/api/docs/models/{gpt-4o-mini,gpt-5}.md.
    const DOC_4O_MINI: &str = "# GPT-4o mini\n\n## Model details\n\n- Default snapshot: `gpt-4o-mini-2024-07-18`\n- 128,000 context window\n- 16,384 max output tokens\n- Oct 01, 2023 knowledge cutoff\n";
    const DOC_GPT5: &str = "# GPT-5\n\n- 400,000 context window\n- Maximum input tokens: 272,000\n- 128,000 max output tokens\n";

    #[tokio::test]
    async fn model_info_pulls_openai_published_limits_from_model_docs() {
        // No `/v1` suffix, so no Ollama probe; `/models` carries no limits.
        let base = serve_routes(&[
            ("/models", r#"{"data":[{"id":"gpt-4o-mini"},{"id":"gpt-4o-mini-2024-07-18"},{"id":"gpt-5"},{"id":"llama3"}]}"#),
            ("/docs/gpt-4o-mini.md", DOC_4O_MINI),
            ("/docs/gpt-5.md", DOC_GPT5),
        ])
        .await;
        let mut p = OpenAIProvider::new("k", base.clone());
        p.model_docs_url = format!("{base}/docs");
        let m = p.model_info("gpt-4o-mini").await.unwrap().unwrap();
        assert_eq!((m.max_input_tokens, m.max_tokens), (Some(128_000), Some(16_384)));
        // Dated snapshot has no page of its own → family page.
        let m = p.model_info("gpt-4o-mini-2024-07-18").await.unwrap().unwrap();
        assert_eq!((m.max_input_tokens, m.max_tokens), (Some(128_000), Some(16_384)));
        // Explicit input cap beats the context window.
        let m = p.model_info("gpt-5").await.unwrap().unwrap();
        assert_eq!((m.max_input_tokens, m.max_tokens), (Some(272_000), Some(128_000)));
        let m = p.model_info("llama3").await.unwrap().unwrap();
        assert_eq!((m.max_input_tokens, m.max_tokens), (None, None));
    }

    #[test]
    fn model_doc_parsing_ignores_prose_and_needs_both_limits() {
        // gpt-4.1-nano's blurb mentions "1M token context window" in prose; only the details list counts.
        let md = "GPT-4.1 nano: 1M token context window, and low latency.\n\n- 1,047,576 context window\n- 32,768 max output tokens\n";
        assert_eq!(parse_model_doc(md), Some((1_047_576, 32_768)));
        assert_eq!(parse_model_doc("- 128,000 context window\n"), None);
        assert_eq!(parse_model_doc("## Model details\n"), None);
    }

    #[test]
    fn only_openai_looking_ids_are_looked_up() {
        for id in ["gpt-4o", "gpt-5.2", "chatgpt-4o-latest", "o1-mini", "o3", "o4-mini-2025-04-16"] {
            assert!(looks_like_openai_model(id), "{id}");
        }
        for id in ["llama3", "qwen3:4b", "olmo-2", "claude-3", "o"] {
            assert!(!looks_like_openai_model(id), "{id}");
        }
        assert!(is_snapshot_date("2024-09-12"));
        assert!(!is_snapshot_date("preview"));
    }
}
