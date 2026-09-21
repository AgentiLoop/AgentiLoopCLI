use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::message::{Message, StopReason};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone)]
pub struct ProviderRequest {
    pub model: String,
    pub system: String,
    pub messages: Vec<Message>,
    pub tools: Vec<ToolSpec>,
    pub max_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub message: Message,
    pub stop_reason: StopReason,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// One entry from a provider's model catalog.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub created_at: String,
}

/// A model backend. Implementations live in `agentiloop-provider`.
#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    /// Model id used when the user hasn't picked one.
    fn default_model(&self) -> &str;
    async fn complete(&self, req: ProviderRequest) -> anyhow::Result<ProviderResponse>;

    /// Live model catalog, newest first where the backend supports ordering.
    async fn list_models(&self) -> anyhow::Result<Vec<ModelInfo>>;

    /// Streaming variant: text deltas are delivered through `on_text` as they
    /// arrive; the assembled response is returned once the stream ends.
    /// Default falls back to `complete` and emits the full text as one delta.
    async fn complete_stream(
        &self,
        req: ProviderRequest,
        on_text: &mut (dyn for<'a> FnMut(&'a str) + Send),
    ) -> anyhow::Result<ProviderResponse> {
        let resp = self.complete(req).await?;
        let text = resp.message.text();
        if !text.is_empty() {
            on_text(&text);
        }
        Ok(resp)
    }
}
