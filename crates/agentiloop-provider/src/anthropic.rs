use agentiloop_core::{ContentBlock, Message, Provider, ProviderRequest, ProviderResponse, Role, StopReason};
use anyhow::Context;
use async_trait::async_trait;
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

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn complete(&self, req: ProviderRequest) -> anyhow::Result<ProviderResponse> {
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
        let text = resp.text().await.context("reading Anthropic response body")?;

        if !status.is_success() {
            if let Ok(err) = serde_json::from_str::<WireError>(&text) {
                anyhow::bail!("Anthropic {} ({}): {}", status, err.error.kind, err.error.message);
            }
            anyhow::bail!("Anthropic {}: {}", status, text);
        }

        let wire: WireResponse = serde_json::from_str(&text).context("decoding Anthropic response")?;

        let stop_reason = match wire.stop_reason.as_deref() {
            Some("end_turn") => StopReason::EndTurn,
            Some("tool_use") => StopReason::ToolUse,
            Some("max_tokens") => StopReason::MaxTokens,
            Some("stop_sequence") => StopReason::StopSequence,
            _ => StopReason::Other,
        };

        Ok(ProviderResponse {
            message: Message { role: Role::Assistant, content: wire.content },
            stop_reason,
            input_tokens: wire.usage.input_tokens,
            output_tokens: wire.usage.output_tokens,
        })
    }
}
