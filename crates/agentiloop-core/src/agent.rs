use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::hooks::Hooks;
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
    /// Prompt tokens the provider served from its cache (Anthropic prompt caching).
    pub cache_read_tokens: u64,
    /// Prompt tokens the provider wrote to its cache.
    pub cache_write_tokens: u64,
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
    /// User hooks (`.agentiloop/hooks.json`) run around tool calls and at the end of a run.
    pub hooks: Hooks,
    /// How many times a provider call that failed transiently (rate limit, overloaded,
    /// 5xx, dropped connection) is retried before the error is reported. 0 = never.
    pub max_retries: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-5".into(),
            system_prompt: DEFAULT_SYSTEM_PROMPT.into(),
            max_tokens: None,
            max_turns: 50,
            compact_at_tokens: None,
            hooks: Hooks::default(),
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }
}

/// Retries of a transiently failing provider call (`AgentConfig::max_retries`).
pub const DEFAULT_MAX_RETRIES: usize = 3;

/// Wait before retry `attempt` (1-based): 1 s, 2 s, 4 s, … capped at 30 s.
pub fn retry_delay(attempt: usize) -> Duration {
    Duration::from_secs(1u64 << attempt.saturating_sub(1).min(5)).min(Duration::from_secs(30))
}

/// HTTP statuses worth retrying: timeouts, rate limits, server errors and Anthropic's 529 overloaded.
const TRANSIENT_STATUSES: &[u16] = &[408, 409, 425, 429, 500, 502, 503, 504, 529];

/// Whether a provider error is the kind that goes away by itself: a transient HTTP
/// status in the message (providers format them as `<name> <status> …`), an
/// overloaded / rate-limit message, or a dropped connection or timeout. A refused
/// connection (nothing listening) and a quota that resets in hours are not retried.
pub fn is_transient(err: &anyhow::Error) -> bool {
    let text = format!("{err:#}").to_lowercase();
    if text.contains("connection refused") || text.contains("usage limit") {
        return false;
    }
    // The first number in the message is the HTTP status (`Anthropic 529 …`, `codex 429: …`);
    // later numbers (token counts in a 400's detail) are not.
    let status = text
        .split(|c: char| c.is_whitespace() || c == ':' || c == '(' || c == ')')
        .find_map(|w| w.parse::<u16>().ok().filter(|n| (100..=599).contains(n)))
        .is_some_and(|n| TRANSIENT_STATUSES.contains(&n));
    status
        || ["overloaded", "rate limit", "rate_limit", "timed out", "timeout", "connection reset", "connection closed",
            "broken pipe", "error sending request", "incomplete message", "unexpected eof", "error reading", "stream error"]
            .iter()
            .any(|needle| text.contains(needle))
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

/// Added to the system prompt while plan mode is on (`Agent::set_plan_mode`).
pub const PLAN_MODE_PROMPT: &str = "PLAN MODE IS ON. You may only inspect the project: read files, list folders, search. \
Every tool that changes something (writing or editing files, shell commands, sub-agents) is switched off and will refuse. \
Do not try to work around that. Instead, work out what needs to be done and reply with a concrete, numbered plan: \
which files you would change and how, which commands you would run, and what could go wrong. \
Finish by telling the user to turn plan mode off (/plan) when they want you to carry it out.";

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
    /// A provider call failed transiently and is about to be repeated after `delay_ms`
    /// (`attempt` of `max_retries`).
    Retry { attempt: usize, max_retries: usize, delay_ms: u64, error: String },
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
    /// Sent as `ProviderRequest::cache_key` on every turn; set to the session id by the CLI.
    cache_key: Option<String>,
    /// Plan mode: tools that change something are refused and the model is asked for a plan.
    plan_mode: bool,
}

impl Agent {
    pub fn new(
        provider: Arc<dyn Provider>,
        tools: ToolRegistry,
        policy: SharedPolicy,
        config: AgentConfig,
        ctx: ToolContext,
    ) -> Self {
        Self { provider, tools, policy, config, ctx, history: Vec::new(), last_input_tokens: 0, usage: Usage::default(), pending_text: String::new(), limits: None, cache_key: None, plan_mode: false }
    }

    /// Plan mode on/off: while on, every mutating tool and `task` refuse to run and
    /// `PLAN_MODE_PROMPT` is added to the system prompt, so the model inspects and plans.
    pub fn set_plan_mode(&mut self, on: bool) {
        self.plan_mode = on;
    }

    pub fn plan_mode(&self) -> bool {
        self.plan_mode
    }

    /// Key the provider's prompt cache on this conversation (see `ProviderRequest::cache_key`).
    pub fn set_cache_key(&mut self, key: impl Into<String>) {
        self.cache_key = Some(key.into());
    }

    pub fn model(&self) -> &str {
        &self.config.model
    }

    pub fn set_model(&mut self, model: impl Into<String>) {
        self.config.model = model.into();
        self.limits = None;
    }

    pub fn hooks(&self) -> &Hooks {
        &self.config.hooks
    }

    pub fn last_input_tokens(&self) -> u64 {
        self.last_input_tokens
    }

    /// Tokens spent since this agent was created; `/clear` does not reset it.
    pub fn usage(&self) -> Usage {
        self.usage
    }

    fn add_usage(&mut self, resp: &crate::provider::ProviderResponse) {
        self.usage.requests += 1;
        self.usage.input_tokens += resp.input_tokens;
        self.usage.output_tokens += resp.output_tokens;
        self.usage.cache_read_tokens += resp.cache_read_tokens;
        self.usage.cache_write_tokens += resp.cache_write_tokens;
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

    /// Record a shell command the user ran themselves (`!cmd` in the REPL / TUI) and
    /// its output as a user turn, so the model sees it on the next request.
    pub fn add_shell_output(&mut self, command: &str, output: &str) {
        self.history.push(Message::user_text(shell_note(command, output)));
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
            cache_key: None,
        };
        Self::budget_request(&mut req, limits)?;
        let resp = complete_with_retry(&*self.provider, self.config.max_retries, req, &mut |_| {}).await?;
        self.add_usage(&resp);
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
            let system = if self.plan_mode { format!("{}\n\n{PLAN_MODE_PROMPT}", self.config.system_prompt) } else { self.config.system_prompt.clone() };
            let mut req = ProviderRequest {
                model: self.config.model.clone(),
                system,
                messages: self.history.clone(),
                tools: self.tool_specs(),
                max_tokens: limits.max_tokens,
                cache_key: self.cache_key.clone(),
            };
            Self::budget_request(&mut req, limits)?;


            let started = Instant::now();
            let mut first_token: Option<Duration> = None;
            let resp = stream_with_retry(&*self.provider, self.config.max_retries, req, &mut |ev| {
                if let AgentEvent::AssistantTextDelta(delta) = &ev {
                    self.pending_text.push_str(delta);
                    first_token.get_or_insert_with(|| started.elapsed());
                }
                on_event(ev)
            })
            .await?;
            self.pending_text.clear();
            let elapsed = started.elapsed();
            self.last_input_tokens = resp.input_tokens;
            self.add_usage(&resp);
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
                let reason = serde_json::to_value(resp.stop_reason).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default();
                self.config.hooks.run_stop(&self.ctx.cwd, &reason).await;
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

        if self.plan_mode && (tool.is_mutating() || name == crate::subagent::TASK_TOOL_NAME) {
            return Err(ToolError::Denied(format!("plan mode is on: `{name}` is switched off; describe what you would do instead")));
        }
        match self.policy.check(name, tool.is_mutating(), &input).await {
            Permission::Allow => {}
            Permission::Deny => return Err(ToolError::Denied(format!("user declined `{name}`"))),
            Permission::Cancel => return Err(ToolError::Cancelled(name.to_string())),
        }

        let hooks = &self.config.hooks;
        if let Err(fb) = hooks.run_pre(&self.ctx.cwd, name, &input).await {
            return Err(ToolError::Denied(format!("blocked by a PreToolUse hook (`{}`): {}", fb.command, fb.message)));
        }
        let result = tool.call(&self.ctx, input.clone()).await;
        let (mut text, is_error) = match &result {
            Ok(out) => (out.clone(), false),
            Err(e) => (e.to_string(), true),
        };
        let feedback = hooks.run_post(&self.ctx.cwd, name, &input, &text, is_error).await;
        if feedback.is_empty() {
            return result;
        }
        for fb in feedback {
            text.push_str(&format!("\n\n[PostToolUse hook `{}`]: {}", fb.command, fb.message));
        }
        if is_error { Err(ToolError::Failed(text)) } else { Ok(text) }
    }
}

/// `provider.complete_stream`, repeated up to `max_retries` times while it fails
/// transiently before any text arrived (a retry after streamed text would show it
/// twice, so such a failure is reported). Deltas and `Retry` events go to `on_event`.
async fn stream_with_retry(
    provider: &dyn Provider,
    max_retries: usize,
    req: ProviderRequest,
    on_event: &mut (dyn FnMut(AgentEvent) + Send),
) -> anyhow::Result<crate::provider::ProviderResponse> {
    let mut attempt = 0;
    loop {
        let mut streamed = false;
        let res = provider
            .complete_stream(req.clone(), &mut |delta| {
                streamed = true;
                on_event(AgentEvent::AssistantTextDelta(delta.to_string()))
            })
            .await;
        match res {
            Err(e) if !streamed && attempt < max_retries && is_transient(&e) => {
                attempt += 1;
                let delay = retry_delay(attempt);
                tracing::warn!("provider call failed ({e:#}); retry {attempt}/{} in {delay:?}", max_retries);
                on_event(AgentEvent::Retry { attempt, max_retries: max_retries, delay_ms: delay.as_millis() as u64, error: format!("{e:#}") });
                tokio::time::sleep(delay).await;
            }
            other => return other,
        }
    }
}

/// `provider.complete` with the same retry policy as `stream_with_retry`.
async fn complete_with_retry(
    provider: &dyn Provider,
    max_retries: usize,
    req: ProviderRequest,
    on_event: &mut (dyn FnMut(AgentEvent) + Send),
) -> anyhow::Result<crate::provider::ProviderResponse> {
    let mut attempt = 0;
    loop {
        match provider.complete(req.clone()).await {
            Err(e) if attempt < max_retries && is_transient(&e) => {
                attempt += 1;
                let delay = retry_delay(attempt);
                tracing::warn!("provider call failed ({e:#}); retry {attempt}/{} in {delay:?}", max_retries);
                on_event(AgentEvent::Retry { attempt, max_retries: max_retries, delay_ms: delay.as_millis() as u64, error: format!("{e:#}") });
                tokio::time::sleep(delay).await;
            }
            other => return other,
        }
    }
}


#[cfg(test)]
mod retry_tests {
    use super::*;

    #[test]
    fn transient_errors_are_recognised_by_status_or_message() {
        let t = |s: &str| is_transient(&anyhow::anyhow!("{s}"));
        assert!(t("Anthropic 529 Service Overloaded (overloaded_error): Overloaded"));
        assert!(t("Anthropic 429 Too Many Requests (rate_limit_error): slow down"));
        assert!(t("openai 503 Service Unavailable : upstream"));
        assert!(t("codex 500: internal"));
        assert!(t("Anthropic stream error (overloaded_error): Overloaded"));
        assert!(t("request to Anthropic failed: error sending request for url (https://x)"));
        assert!(t("reading Anthropic stream: connection reset by peer"));
        assert!(!t("Anthropic 400 Bad Request (invalid_request_error): max_tokens 500 exceeds the limit"));
        assert!(!t("Anthropic 401 Unauthorized (authentication_error): invalid x-api-key"));
        assert!(!t("codex 429: ChatGPT plan usage limit reached; resets in ~5.0h"));
        assert!(!t("request to oMLX failed: connection refused"));
        assert!(!t("compaction produced an empty summary"));
    }

    #[test]
    fn delay_doubles_from_one_second_and_is_capped() {
        assert_eq!(retry_delay(1), Duration::from_secs(1));
        assert_eq!(retry_delay(2), Duration::from_secs(2));
        assert_eq!(retry_delay(3), Duration::from_secs(4));
        assert_eq!(retry_delay(9), Duration::from_secs(30));
    }
}

/// The command behind a `!cmd` prompt line (REPL / TUI): the text after the `!`, or None
/// when the line isn't one (`!` alone, or a line that merely contains a `!`).
pub fn shell_passthrough(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix('!')?.trim();
    (!rest.is_empty()).then_some(rest)
}

/// Longest shell output kept in the conversation for a `!cmd` line (bytes).
pub const SHELL_NOTE_LIMIT: usize = 30_000;

/// The user turn recorded for a `!cmd` line: the command and its (cut) output.
pub fn shell_note(command: &str, output: &str) -> String {
    let output = output.trim_end();
    let body = if output.is_empty() {
        "(no output)".to_string()
    } else if output.len() > SHELL_NOTE_LIMIT {
        let cut = output.floor_char_boundary(SHELL_NOTE_LIMIT);
        format!("{}\n…[{} more bytes]", &output[..cut], output.len() - cut)
    } else {
        output.to_string()
    };
    format!("I ran this shell command myself in the project folder:\n$ {command}\n\n{body}")
}

#[cfg(test)]
mod shell_tests {
    use super::*;

    #[test]
    fn bang_lines_are_shell_commands() {
        assert_eq!(shell_passthrough("!git status"), Some("git status"));
        assert_eq!(shell_passthrough("  ! ls -la  "), Some("ls -la"));
        assert_eq!(shell_passthrough("!"), None);
        assert_eq!(shell_passthrough("!   "), None);
        assert_eq!(shell_passthrough("fix it!"), None);
        assert_eq!(shell_passthrough("/help"), None);
    }

    #[test]
    fn shell_note_has_the_command_and_the_cut_output() {
        assert_eq!(shell_note("ls", "a\nb\n"), "I ran this shell command myself in the project folder:\n$ ls\n\na\nb");
        assert!(shell_note("true", "  \n").ends_with("(no output)"));
        let big = "x".repeat(SHELL_NOTE_LIMIT + 10);
        assert!(shell_note("cat big", &big).ends_with("…[10 more bytes]"));
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