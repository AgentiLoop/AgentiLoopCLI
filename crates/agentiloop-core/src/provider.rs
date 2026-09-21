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

/// A model backend. Implementations live in `agentiloop-provider`.
#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    async fn complete(&self, req: ProviderRequest) -> anyhow::Result<ProviderResponse>;
}
