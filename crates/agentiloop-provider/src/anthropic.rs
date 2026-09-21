use agentiloop_core::{ContentBlock, Message, Provider, ProviderRequest, ProviderResponse, Role, StopReason};
use anyhow::Context;
use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const API_VERSION: &str = "2023-06-01";
const OAUTH_PREFIX: &str = "sk-ant-oat01-";
const OAUTH_BETA: &str = "oauth-2025-04-20,prompt-caching-2024-07-31";
/// OAuth tokens (from `claude setup-token`) are gated at the API to requests
/// whose first system block is exactly this string.
const CLAUDE_CODE_IDENTITY: &str = "You are Claude Code, Anthropic's official CLI for Claude.";

pub struct AnthropicProvider {
    client: reqwest::Client,
    credential: String,
    base_url: String,
}

impl AnthropicProvider {
    /// Accepts either a standard API key (`sk-ant-api…`) or a Claude Code
    /// OAuth token (`sk-ant-oat01-…`); the auth scheme is chosen automatically.
    pub fn new(credential: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            credential: sanitize(&credential.into()),
            base_url: std::env::var("ANTHROPIC_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.into()),
        }
    }

    /// Reads `ANTHROPIC_API_KEY` (API key or OAuth token), falling back to `ANTHROPIC_OAUTH_TOKEN`.
    pub fn from_env() -> anyhow::Result<Self> {
        let key = std::env::var("ANTHROPIC_API_KEY")
            .or_else(|_| std::env::var("ANTHROPIC_OAUTH_TOKEN"))
            .context("ANTHROPIC_API_KEY is not set (API key or sk-ant-oat01- OAuth token)")?;
        Ok(Self::new(key))
    }

    pub fn is_oauth(&self) -> bool {
        self.credential.starts_with(OAUTH_PREFIX)
    }

    /// Fetch the live model catalog from `GET /v1/models`, newest first
    /// (the API already returns them sorted by `created_at` descending).
    pub async fn list_models(&self) -> anyhow::Result<Vec<ModelInfo>> {
        let mut http = self
            .client
            .get(format!("{}/v1/models?limit=100", self.base_url))
            .header("anthropic-version", API_VERSION);
        http = if self.is_oauth() {
            http.header("authorization", format!("Bearer {}", self.credential))
                .header("anthropic-beta", "oauth-2025-04-20")
        } else {
            http.header("x-api-key", &self.credential)
        };
        let resp = http.send().await.context("request to /v1/models failed")?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            anyhow::bail!("Anthropic {}: {}", status, text);
        }
        let list: WireModelList = serde_json::from_str(&text).context("decoding /v1/models")?;
        Ok(list.data)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Deserialize)]
struct WireModelList {
    data: Vec<ModelInfo>,
}

/// Strip whitespace/control chars a terminal paste may have wrapped into the token.
fn sanitize(raw: &str) -> String {
    raw.chars().filter(|c| !c.is_whitespace() && !c.is_control()).collect()
}

#[derive(Serialize)]
struct WireRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    system: Vec<WireSystem<'a>>,
    messages: &'a [Message],
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<WireTool<'a>>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
}

#[derive(Serialize)]
struct WireSystem<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    text: &'a str,
}

#[derive(Serialize)]
struct WireTool<'a> {
    name: &'a str,
    description: &'a str,
    input_schema: &'a Value,
}

#[derive(Deserialize)]
struct WireResponse {
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
    #[serde(default)]
    usage: WireUsage,
}

#[derive(Deserialize, Default)]
struct WireUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
}

#[derive(Deserialize)]
struct WireError {
    error: WireErrorBody,
}

#[derive(Deserialize)]
struct WireErrorBody {
    #[serde(rename = "type")]
    kind: String,
    message: String,
}

/// One SSE `data:` payload from a streaming `/v1/messages` response.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WireEvent {
    MessageStart { message: WireStreamMessage },
    ContentBlockStart { content_block: WireBlockStart },
    ContentBlockDelta { index: usize, delta: WireDelta },
    ContentBlockStop {},
    MessageDelta { delta: WireMessageDelta, #[serde(default)] usage: WireUsage },
    MessageStop {},
    Ping {},
    Error { error: WireErrorBody },
}

#[derive(Deserialize)]
struct WireStreamMessage {
    #[serde(default)]
    usage: WireUsage,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WireBlockStart {
    Text {},
    ToolUse { id: String, name: String },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WireDelta {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct WireMessageDelta {
    stop_reason: Option<String>,
}

/// A content block being assembled from stream deltas.
enum Partial {
    Text(String),
    ToolUse { id: String, name: String, json: String },
    Skip,
}

fn parse_stop_reason(raw: Option<&str>) -> StopReason {
    match raw {
        Some("end_turn") => StopReason::EndTurn,
        Some("tool_use") => StopReason::ToolUse,
        Some("max_tokens") => StopReason::MaxTokens,
        Some("stop_sequence") => StopReason::StopSequence,
        _ => StopReason::Other,
    }
}

impl AnthropicProvider {
    async fn send_messages(&self, req: &ProviderRequest, stream: bool) -> anyhow::Result<reqwest::Response> {
        let mut system = Vec::with_capacity(2);
        if self.is_oauth() {
            system.push(WireSystem { kind: "text", text: CLAUDE_CODE_IDENTITY });
        }
        system.push(WireSystem { kind: "text", text: &req.system });

        let body = WireRequest {
            model: &req.model,
            max_tokens: req.max_tokens,
            system,
            messages: &req.messages,
            tools: req
                .tools
                .iter()
                .map(|t| WireTool { name: &t.name, description: &t.description, input_schema: &t.input_schema })
                .collect(),
            stream,
        };

        let mut http = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("anthropic-version", API_VERSION);
        http = if self.is_oauth() {
            http.header("authorization", format!("Bearer {}", self.credential))
                .header("anthropic-beta", OAUTH_BETA)
        } else {
            http.header("x-api-key", &self.credential)
        };

        let resp = http
            .json(&body)
            .send()
            .await
            .context("request to Anthropic failed")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.context("reading Anthropic error body")?;
            if let Ok(err) = serde_json::from_str::<WireError>(&text) {
                anyhow::bail!("Anthropic {} ({}): {}", status, err.error.kind, err.error.message);
            }
            anyhow::bail!("Anthropic {}: {}", status, text);
        }
        Ok(resp)
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn complete(&self, req: ProviderRequest) -> anyhow::Result<ProviderResponse> {
        let resp = self.send_messages(&req, false).await?;
        let text = resp.text().await.context("reading Anthropic response body")?;
        let wire: WireResponse = serde_json::from_str(&text).context("decoding Anthropic response")?;

        Ok(ProviderResponse {
            message: Message { role: Role::Assistant, content: wire.content },
            stop_reason: parse_stop_reason(wire.stop_reason.as_deref()),
            input_tokens: wire.usage.input_tokens,
            output_tokens: wire.usage.output_tokens,
        })
    }

    async fn complete_stream(
        &self,
        req: ProviderRequest,
        on_text: &mut (dyn for<'a> FnMut(&'a str) + Send),
    ) -> anyhow::Result<ProviderResponse> {
        let resp = self.send_messages(&req, true).await?;
        let mut body = resp.bytes_stream();

        let mut buf: Vec<u8> = Vec::new();
        let mut blocks: Vec<Partial> = Vec::new();
        let mut stop_reason = None;
        let mut input_tokens = 0;
        let mut output_tokens = 0;

        while let Some(chunk) = body.next().await {
            buf.extend_from_slice(&chunk.context("reading Anthropic stream")?);
            // SSE frames are newline-delimited; the JSON payload sits on `data:` lines.
            while let Some(nl) = buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buf.drain(..=nl).collect();
                let line = String::from_utf8_lossy(&line);
                let Some(data) = line.trim_end().strip_prefix("data:") else { continue };
                let event: WireEvent = serde_json::from_str(data.trim_start()).context("decoding stream event")?;
                match event {
                    WireEvent::MessageStart { message } => input_tokens = message.usage.input_tokens,
                    WireEvent::ContentBlockStart { content_block } => blocks.push(match content_block {
                        WireBlockStart::Text {} => Partial::Text(String::new()),
                        WireBlockStart::ToolUse { id, name } => Partial::ToolUse { id, name, json: String::new() },
                        WireBlockStart::Other => Partial::Skip,
                    }),
                    WireEvent::ContentBlockDelta { index, delta } => match (blocks.get_mut(index), delta) {
                        (Some(Partial::Text(t)), WireDelta::TextDelta { text }) => {
                            on_text(&text);
                            t.push_str(&text);
                        }
                        (Some(Partial::ToolUse { json, .. }), WireDelta::InputJsonDelta { partial_json }) => {
                            json.push_str(&partial_json);
                        }
                        _ => {}
                    },
                    WireEvent::MessageDelta { delta, usage } => {
                        stop_reason = delta.stop_reason;
                        output_tokens = usage.output_tokens;
                    }
                    WireEvent::Error { error } => anyhow::bail!("Anthropic stream error ({}): {}", error.kind, error.message),
                    WireEvent::ContentBlockStop {} | WireEvent::MessageStop {} | WireEvent::Ping {} => {}
                }
            }
        }

        let content = blocks
            .into_iter()
            .filter_map(|b| match b {
                Partial::Text(text) => Some(Ok(ContentBlock::Text { text })),
                Partial::ToolUse { id, name, json } => {
                    let input = if json.trim().is_empty() {
                        Value::Object(Default::default())
                    } else {
                        match serde_json::from_str(&json).with_context(|| format!("decoding tool input for `{name}`")) {
                            Ok(v) => v,
                            Err(e) => return Some(Err(e)),
                        }
                    };
                    Some(Ok(ContentBlock::ToolUse { id, name, input }))
                }
                Partial::Skip => None,
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(ProviderResponse {
            message: Message { role: Role::Assistant, content },
            stop_reason: parse_stop_reason(stop_reason.as_deref()),
            input_tokens,
            output_tokens,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    const SSE: &str = "\
event: message_start
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"m\",\"usage\":{\"input_tokens\":25,\"output_tokens\":1}}}

event: content_block_start
data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}

event: ping
data: {\"type\":\"ping\"}

event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hel\"}}

event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"lo\"}}

event: content_block_stop
data: {\"type\":\"content_block_stop\",\"index\":0}

event: content_block_start
data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"read_file\",\"input\":{}}}

event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"path\\\": \"}}

event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"\\\"a.rs\\\"}\"}}

event: content_block_stop
data: {\"type\":\"content_block_stop\",\"index\":1}

event: message_delta
data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\",\"stop_sequence\":null},\"usage\":{\"output_tokens\":15}}

event: message_stop
data: {\"type\":\"message_stop\"}

";

    /// Serve one canned SSE response on a local port, in chunks that split
    /// lines mid-way to exercise the buffering.
    async fn serve_once() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut req = vec![0u8; 8192];
            let _ = sock.read(&mut req).await;
            let head = "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\nconnection: close\r\n\r\n";
            sock.write_all(head.as_bytes()).await.unwrap();
            for chunk in SSE.as_bytes().chunks(37) {
                sock.write_all(chunk).await.unwrap();
                sock.flush().await.unwrap();
            }
            sock.shutdown().await.unwrap();
        });
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn streams_text_deltas_and_assembles_tool_use() {
        let base_url = serve_once().await;
        let provider = AnthropicProvider { client: reqwest::Client::new(), credential: "sk-ant-api-test".into(), base_url };
        let req = ProviderRequest {
            model: "m".into(),
            system: "s".into(),
            messages: vec![Message::user_text("hi")],
            tools: vec![],
            max_tokens: 16,
        };

        let mut deltas = Vec::new();
        let resp = provider.complete_stream(req, &mut |t| deltas.push(t.to_string())).await.unwrap();

        assert_eq!(deltas, vec!["Hel", "lo"]);
        assert_eq!(resp.message.text(), "Hello");
        assert_eq!(resp.stop_reason, StopReason::ToolUse);
        assert_eq!(resp.input_tokens, 25);
        assert_eq!(resp.output_tokens, 15);
        let tools: Vec<_> = resp.message.tool_uses().collect();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].0, "toolu_1");
        assert_eq!(tools[0].1, "read_file");
        assert_eq!(tools[0].2, &serde_json::json!({"path": "a.rs"}));
    }
}
