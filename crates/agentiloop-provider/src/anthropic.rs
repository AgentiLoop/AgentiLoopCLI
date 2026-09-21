use agentiloop_core::{ContentBlock, Message, Provider, ProviderRequest, ProviderResponse, Role, StopReason};
use anyhow::Context;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const API_VERSION: &str = "2023-06-01";

pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            base_url: std::env::var("ANTHROPIC_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.into()),
        }
    }

    pub fn from_env() -> anyhow::Result<Self> {
        let key = std::env::var("ANTHROPIC_API_KEY").context("ANTHROPIC_API_KEY is not set")?;
        Ok(Self::new(key))
    }
}

#[derive(Serialize)]
struct WireRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    messages: &'a [Message],
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<WireTool<'a>>,
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
        let body = WireRequest {
            model: &req.model,
            max_tokens: req.max_tokens,
            system: &req.system,
            messages: &req.messages,
            tools: req
                .tools
                .iter()
                .map(|t| WireTool { name: &t.name, description: &t.description, input_schema: &t.input_schema })
                .collect(),
        };

        let resp = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
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
