use std::sync::Arc;

use crate::message::{ContentBlock, Message, StopReason};
use crate::permission::{Permission, SharedPolicy};
use crate::provider::{Provider, ProviderRequest, ToolSpec};
use crate::tool::{ToolContext, ToolError, ToolRegistry};

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub model: String,
    pub system_prompt: String,
    pub max_tokens: u32,
    /// Hard cap on provider round-trips per `run` to avoid runaway loops.
    pub max_turns: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-5".into(),
            system_prompt: DEFAULT_SYSTEM_PROMPT.into(),
            max_tokens: 8192,
            max_turns: 50,
        }
    }
}

pub const DEFAULT_SYSTEM_PROMPT: &str = "You are AgentiLoop, an autonomous coding agent running in the user's terminal. \
Use the provided tools to inspect and modify the project in the current working directory. \
Be concise. Prefer acting over asking. When the task is complete, reply with a short summary.";

/// Events emitted during a run so the front-end can render progress.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    AssistantText(String),
    ToolCall { id: String, name: String, input: serde_json::Value },
    ToolResult { id: String, name: String, output: String, is_error: bool },
    TurnComplete { input_tokens: u64, output_tokens: u64 },
    Done { stop_reason: StopReason },
}

pub struct Agent {
    provider: Arc<dyn Provider>,
    tools: ToolRegistry,
    policy: SharedPolicy,
    config: AgentConfig,
    ctx: ToolContext,
    pub history: Vec<Message>,
}

impl Agent {
    pub fn new(
        provider: Arc<dyn Provider>,
        tools: ToolRegistry,
        policy: SharedPolicy,
        config: AgentConfig,
        ctx: ToolContext,
    ) -> Self {
        Self { provider, tools, policy, config, ctx, history: Vec::new() }
    }

    pub fn model(&self) -> &str {
        &self.config.model
    }

    pub fn set_model(&mut self, model: impl Into<String>) {
        self.config.model = model.into();
    }

    /// Drop all conversation context and tool history.
    pub fn clear(&mut self) {
        self.history.clear();
    }

    fn tool_specs(&self) -> Vec<ToolSpec> {
        self.tools
            .iter()
            .map(|t| ToolSpec {
                name: t.name().to_string(),
                description: t.description().to_string(),
                input_schema: t.input_schema(),
            })
            .collect()
    }

    /// The core agentic loop: send → if tool_use, execute tools, append results, repeat.
    pub async fn run<F>(&mut self, user_input: &str, mut on_event: F) -> anyhow::Result<()>
    where
        F: FnMut(AgentEvent),
    {
        self.history.push(Message::user_text(user_input));

        for _ in 0..self.config.max_turns {
            let req = ProviderRequest {
                model: self.config.model.clone(),
                system: self.config.system_prompt.clone(),
                messages: self.history.clone(),
                tools: self.tool_specs(),
                max_tokens: self.config.max_tokens,
            };

            let resp = self.provider.complete(req).await?;
            on_event(AgentEvent::TurnComplete {
                input_tokens: resp.input_tokens,
                output_tokens: resp.output_tokens,
            });

            let text = resp.message.text();
            if !text.is_empty() {
                on_event(AgentEvent::AssistantText(text));
            }

            let calls: Vec<(String, String, serde_json::Value)> = resp
                .message
                .tool_uses()
                .map(|(id, name, input)| (id.to_string(), name.to_string(), input.clone()))
                .collect();

            self.history.push(resp.message.clone());

            if resp.stop_reason != StopReason::ToolUse || calls.is_empty() {
                on_event(AgentEvent::Done { stop_reason: resp.stop_reason });
                return Ok(());
            }

            let mut results = Vec::with_capacity(calls.len());
            for (id, name, input) in calls {
                on_event(AgentEvent::ToolCall { id: id.clone(), name: name.clone(), input: input.clone() });
                let (output, is_error) = match self.execute(&name, input).await {
                    Ok(out) => (out, false),
                    Err(e) => (e.to_string(), true),
                };
                on_event(AgentEvent::ToolResult {
                    id: id.clone(),
                    name,
                    output: output.clone(),
                    is_error,
                });
                results.push(ContentBlock::ToolResult { tool_use_id: id, content: output, is_error });
            }
            self.history.push(Message::tool_results(results));
        }

        anyhow::bail!("max_turns ({}) reached", self.config.max_turns)
    }

    async fn execute(&self, name: &str, input: serde_json::Value) -> Result<String, ToolError> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| ToolError::InvalidInput(format!("unknown tool `{name}`")))?;

        if self.policy.check(name, tool.is_mutating(), &input).await == Permission::Deny {
            return Err(ToolError::Denied(format!("user declined `{name}`")));
        }

        tool.call(&self.ctx, input).await
    }
}
