use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::message::{ContentBlock, Message, Role, StopReason};
use crate::permission::{Permission, SharedPolicy};
use crate::provider::{Provider, ProviderRequest, ToolSpec};
use crate::tool::{ToolContext, ToolError, ToolRegistry};

/// Tokens spent by an agent since it was created (summaries for compaction included).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Usage {
    pub requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub model: String,
    pub system_prompt: String,
    /// Output cap per response. None = the model's own `max_tokens` from the
    /// provider catalog (falls back to `FALLBACK_MAX_TOKENS` if unknown).
    pub max_tokens: Option<u32>,
    /// Hard cap on provider round-trips per `run` to avoid runaway loops.
    pub max_turns: usize,
    /// When the last request's `input_tokens` reaches this, the history is
    /// summarized before the next request. None = `COMPACT_FRACTION` of the
    /// model's context window (or `FALLBACK_COMPACT_AT` if unknown). Some(0) disables.
    pub compact_at_tokens: Option<u64>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-5".into(),
            system_prompt: DEFAULT_SYSTEM_PROMPT.into(),
            max_tokens: None,
            max_turns: 50,
            compact_at_tokens: None,
        }
    }
}

/// Used when the provider catalog doesn't report the model's output limit.
pub const FALLBACK_MAX_TOKENS: u32 = 32_768;
/// Used when the provider catalog doesn't report the model's context window.
pub const FALLBACK_COMPACT_AT: u64 = 150_000;
/// Compact once input reaches this share of the model's context window.
const COMPACT_FRACTION: f64 = 0.8;

/// Output and context limits in effect for the agent's current model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelLimits {
    pub max_tokens: u32,
    /// Context window in tokens, when known.
    pub context_window: Option<u64>,
    /// Effective compaction threshold (0 = disabled).
    pub compact_at_tokens: u64,
}


pub const DEFAULT_SYSTEM_PROMPT: &str = "You are AgentiLoop, an autonomous terminal coding agent. \
Ignore any earlier name, identity, or vendor line — it is transport boilerplate, not who you are. \
If asked who you are, what you are, what model or company powers you, or what you are based on, answer only: \
\"I am AgentiLoop, powered by AgentiLoop.ai.\" Never mention Codex, GPT, OpenAI, Claude, Claude Code, Anthropic, \
or any other underlying model, vendor, or CLI, and never speculate about them. \
Use the provided tools to inspect and modify the project in the current working directory. \
Be concise. Prefer acting over asking. When the task is complete, reply with a short summary. \
Your replies are rendered as Markdown directly in the terminal: write Markdown (headings, lists, code fences for code) \
and it will be displayed styled. Never wrap an entire reply in a ```markdown fence, and never shell out to tools like \
glow, bat, or cat to \"render\" Markdown — just write it.";

const COMPACT_SYSTEM_PROMPT: &str = "You compress conversation transcripts for an autonomous coding agent so it can continue \
with less context. Write a dense summary that preserves: the user's goals and constraints, decisions made, files and \
symbols touched (with paths), what has been verified to work, what failed and why, and any pending next steps. \
Do not add commentary. Output only the summary.";

/// Tool output longer than this is trimmed in the compaction transcript.
const TRANSCRIPT_RESULT_LIMIT: usize = 2_000;

/// Events emitted during a run so the front-end can render progress.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// A streamed chunk of assistant text, emitted as it arrives.
    AssistantTextDelta(String),
    /// The complete assistant text for the turn (after all deltas).
    AssistantText(String),
    ToolCall { id: String, name: String, input: serde_json::Value },
    ToolResult { id: String, name: String, output: String, is_error: bool },
    /// One provider round-trip finished. `elapsed_ms` is the whole call;
    /// `first_token_ms` is when the first text delta arrived (None when the
    /// turn streamed no text, e.g. tool-call-only turns).
    TurnComplete { input_tokens: u64, output_tokens: u64, elapsed_ms: u64, first_token_ms: Option<u64> },
    /// History was summarized; `before_tokens` is the input size that triggered it.
    Compacted { before_tokens: u64, messages_dropped: usize },
    Done { stop_reason: StopReason },
}

pub struct Agent {
    provider: Arc<dyn Provider>,
    tools: ToolRegistry,
    policy: SharedPolicy,
    config: AgentConfig,
    ctx: ToolContext,
    pub history: Vec<Message>,
    /// `input_tokens` reported by the most recent provider response.
    last_input_tokens: u64,
    usage: Usage,
    /// Text received from the current stream but not yet committed to history.
    pending_text: String,
    /// Limits resolved from the provider catalog for `config.model`; cleared on `set_model`.
    limits: Option<ModelLimits>,
}

impl Agent {
    pub fn new(
        provider: Arc<dyn Provider>,
        tools: ToolRegistry,
        policy: SharedPolicy,
        config: AgentConfig,
        ctx: ToolContext,
    ) -> Self {
        Self { provider, tools, policy, config, ctx, history: Vec::new(), last_input_tokens: 0, usage: Usage::default(), pending_text: String::new(), limits: None }
    }

    pub fn model(&self) -> &str {
        &self.config.model
    }

    pub fn set_model(&mut self, model: impl Into<String>) {
        self.config.model = model.into();
        self.limits = None;
    }

    pub fn last_input_tokens(&self) -> u64 {
        self.last_input_tokens
    }

    /// Tokens spent since this agent was created; `/clear` does not reset it.
    pub fn usage(&self) -> Usage {
        self.usage
    }

    fn add_usage(&mut self, input: u64, output: u64) {
        self.usage.requests += 1;
        self.usage.input_tokens += input;
        self.usage.output_tokens += output;
    }

    /// Limits in effect for the current model, once a run has resolved them.
    pub fn limits(&self) -> Option<ModelLimits> {
        self.limits
    }

    /// Look up the model's output cap and context window from the provider
    /// catalog (once per model) and combine them with any config overrides.
    /// A catalog failure is not fatal: the fallbacks apply.
    pub async fn resolve_limits(&mut self) -> ModelLimits {
        if let Some(l) = self.limits {
            return l;
        }
        let info = match self.provider.model_info(&self.config.model).await {
            Ok(info) => info,
            Err(e) => {
                tracing::warn!("could not look up limits for {}: {e:#}", self.config.model);
                None
            }
        };
        let context_window = info.as_ref().and_then(|m| m.max_input_tokens);
        let max_tokens = self
            .config
            .max_tokens
            .or(info.as_ref().and_then(|m| m.max_tokens))
            .unwrap_or(FALLBACK_MAX_TOKENS);
        let compact_at_tokens = self.config.compact_at_tokens.unwrap_or_else(|| {
            context_window.map(|w| (w as f64 * COMPACT_FRACTION) as u64).unwrap_or(FALLBACK_COMPACT_AT)
        });
        let limits = ModelLimits { max_tokens, context_window, compact_at_tokens };
        self.limits = Some(limits);
        limits
    }

    /// Drop all conversation context and tool history.
    pub fn clear(&mut self) {
        self.history.clear();
        self.pending_text.clear();
        self.last_input_tokens = 0;
    }

    /// Finish an interrupted run after its future has been dropped. Keep completed
    /// work and pair every outstanding tool call so the next request is valid.
    pub fn interrupt(&mut self) {
        if !self.pending_text.is_empty() {
            self.history.push(Message {
                role: Role::Assistant,
                content: vec![ContentBlock::Text { text: std::mem::take(&mut self.pending_text) }],
            });
        }
        if let Some(i) = self.history.iter().rposition(|m| m.role == Role::Assistant) {
            let missing: Vec<_> = self.history[i].tool_uses().filter_map(|(id, _, _)| {
                let answered = self.history[i + 1..].iter().flat_map(|m| &m.content).any(|b| {
                    matches!(b, ContentBlock::ToolResult { tool_use_id, .. } if tool_use_id == id)
                });
                (!answered).then(|| ContentBlock::ToolResult {
                    tool_use_id: id.to_string(),
                    content: "Interrupted by user; execution may be incomplete. Do not assume changes were undone.".into(),
                    is_error: true,
                })
            }).collect();
            if !missing.is_empty() {
                if self.history.len() == i + 1 {
                    self.history.push(Message::tool_results(missing));
                } else {
                    self.history[i + 1].content.extend(missing);
                }
            }
        }
    }

    fn should_compact(&self) -> bool {
        let at = self.limits.map(|l| l.compact_at_tokens).unwrap_or(0);
        at > 0 && self.last_input_tokens >= at
    }


    /// Replace the history with a provider-written summary of it. No-op when empty.
    pub async fn compact(&mut self) -> anyhow::Result<Option<AgentEvent>> {
        if self.history.is_empty() {
            return Ok(None);
        }
        let limits = self.resolve_limits().await;
        let transcript = transcript(&self.history);
        let mut req = ProviderRequest {
            model: self.config.model.clone(),
            system: COMPACT_SYSTEM_PROMPT.into(),
            messages: vec![Message::user_text(format!(
                "Summarize the following transcript.\n\n<transcript>\n{transcript}\n</transcript>"
            ))],
            tools: Vec::new(),
            max_tokens: 4096,
        };
        Self::budget_request(&mut req, limits)?;
        let resp = self.provider.complete(req).await?;
        self.add_usage(resp.input_tokens, resp.output_tokens);
        let summary = resp.message.text();
        anyhow::ensure!(!summary.trim().is_empty(), "compaction produced an empty summary");

        let dropped = self.history.len();
        let before = self.last_input_tokens;
        self.history = vec![
            Message::user_text(format!("[Context was compacted. Summary of the conversation so far:]\n{summary}")),
            Message {
                role: Role::Assistant,
                content: vec![ContentBlock::Text { text: "Understood. I will continue from that summary.".into() }],
            },
        ];
        self.last_input_tokens = 0;
        Ok(Some(AgentEvent::Compacted { before_tokens: before, messages_dropped: dropped }))
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

    /// Reserve input space before applying the model's output ceiling. UTF-8
    /// bytes plus framing headroom are a conservative estimate, not a tokenizer.
    fn budget_request(req: &mut ProviderRequest, limits: ModelLimits) -> anyhow::Result<()> {
        req.max_tokens = req.max_tokens.min(limits.max_tokens);
        if let Some(window) = limits.context_window {
            let input = serde_json::to_vec(&(&req.system, &req.messages, &req.tools))?.len() as u64
                + 256 + 16 * (req.messages.len() + req.tools.len()) as u64;
            let available = window.saturating_sub(input);
            anyhow::ensure!(available > 0, "request input exceeds the estimated context budget ({input} of {window} tokens); compact or clear the conversation, reduce the prompt or tools, or use a larger-context model");
            req.max_tokens = u64::from(req.max_tokens).min(available) as u32;
        }
        Ok(())
    }

    /// The core agentic loop: send → if tool_use, execute tools, append results, repeat.
    pub async fn run<F>(&mut self, user_input: &str, mut on_event: F) -> anyhow::Result<()>
    where
        F: FnMut(AgentEvent) + Send,
    {
        self.pending_text.clear();
        let limits = self.resolve_limits().await;
        if self.should_compact() {
            if let Some(ev) = self.compact().await? {
                on_event(ev);
            }
        }
        self.history.push(Message::user_text(user_input));

        for _ in 0..self.config.max_turns {
            let mut req = ProviderRequest {
                model: self.config.model.clone(),
                system: self.config.system_prompt.clone(),
                messages: self.history.clone(),
                tools: self.tool_specs(),
                max_tokens: limits.max_tokens,
            };
            Self::budget_request(&mut req, limits)?;


            let started = Instant::now();
            let mut first_token: Option<Duration> = None;
            let resp = self
                .provider
                .complete_stream(req, &mut |delta| {
                    self.pending_text.push_str(delta);
                    first_token.get_or_insert_with(|| started.elapsed());
                    on_event(AgentEvent::AssistantTextDelta(delta.to_string()))
                })
                .await?;
            self.pending_text.clear();
            let elapsed = started.elapsed();
            self.last_input_tokens = resp.input_tokens;
            self.add_usage(resp.input_tokens, resp.output_tokens);
            on_event(AgentEvent::TurnComplete {
                input_tokens: resp.input_tokens,
                output_tokens: resp.output_tokens,
                elapsed_ms: elapsed.as_millis() as u64,
                first_token_ms: first_token.map(|d| d.as_millis() as u64),
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

            self.history.push(Message::tool_results(Vec::with_capacity(calls.len())));
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
                self.history.last_mut().unwrap().content.push(ContentBlock::ToolResult { tool_use_id: id, content: output, is_error });
            }

            if self.should_compact() {
                if let Some(ev) = self.compact().await? {
                    on_event(ev);
                }
                self.history.push(Message::user_text("Continue the task from the summary above."));
            }
        }

        anyhow::bail!("max_turns ({}) reached", self.config.max_turns)
    }

    async fn execute(&self, name: &str, input: serde_json::Value) -> Result<String, ToolError> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| ToolError::InvalidInput(format!("unknown tool `{name}`")))?;

        match self.policy.check(name, tool.is_mutating(), &input).await {
            Permission::Allow => {}
            Permission::Deny => return Err(ToolError::Denied(format!("user declined `{name}`"))),
            Permission::Cancel => return Err(ToolError::Cancelled(name.to_string())),
        }

        tool.call(&self.ctx, input).await
    }
}

/// Plain-text rendering of the history for the compaction prompt.
pub fn transcript(history: &[Message]) -> String {
    let mut out = String::new();
    for m in history {
        let role = match m.role {
            Role::User => "USER",
            Role::Assistant => "ASSISTANT",
        };
        for block in &m.content {
            match block {
                ContentBlock::Text { text } => out.push_str(&format!("{role}: {text}\n")),
                ContentBlock::ToolUse { name, input, .. } => out.push_str(&format!("{role} → tool {name} {input}\n")),
                ContentBlock::ToolResult { content, is_error, .. } => {
                    let tag = if *is_error { "tool error" } else { "tool result" };
                    let body = if content.len() > TRANSCRIPT_RESULT_LIMIT {
                        let cut = content.floor_char_boundary(TRANSCRIPT_RESULT_LIMIT);
                        format!("{}…[{} more bytes]", &content[..cut], content.len() - cut)
                    } else {
                        content.clone()
                    };
                    out.push_str(&format!("{tag}: {body}\n"));
                }
            }
        }
    }
    out
}