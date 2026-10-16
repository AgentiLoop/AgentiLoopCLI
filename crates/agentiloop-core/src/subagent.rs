//! `task`: run a sub-agent on a self-contained job. It gets a fresh conversation, the same
//! provider, model, system prompt, permission policy and tools (except `task` itself, so the
//! depth is one), does the work, and only its final answer comes back as the tool result — the
//! parent's context stays clean of the exploration.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::agent::{Agent, AgentConfig, AgentEvent};
use crate::permission::SharedPolicy;
use crate::provider::Provider;
use crate::tool::{Tool, ToolContext, ToolError, ToolRegistry, ToolResult};

pub const TASK_TOOL_NAME: &str = "task";

pub struct TaskTool {
    provider: Arc<dyn Provider>,
    tools: ToolRegistry,
    policy: SharedPolicy,
    config: AgentConfig,
}

#[derive(Deserialize)]
struct TaskArgs {
    prompt: String,
    #[serde(default)]
    description: String,
}

/// What a sub-agent run produced, for the tool result and for tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOutcome {
    pub answer: String,
    pub turns: u64,
    pub tool_calls: usize,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl TaskTool {
    /// `tools` is the parent's registry; `task` is left out of the child's copy.
    pub fn new(provider: Arc<dyn Provider>, tools: &ToolRegistry, policy: SharedPolicy, config: AgentConfig) -> Self {
        let mut child_tools = ToolRegistry::new();
        for t in tools.iter().filter(|t| t.name() != TASK_TOOL_NAME) {
            child_tools.register_arc(t.clone());
        }
        let config = AgentConfig { hooks: config.hooks.without_stop(), ..config };
        Self { provider, tools: child_tools, policy, config }
    }

    pub async fn run(&self, cwd: PathBuf, prompt: &str) -> anyhow::Result<TaskOutcome> {
        let mut agent = Agent::new(self.provider.clone(), self.tools.clone(), self.policy.clone(), self.config.clone(), ToolContext { cwd });
        let mut answer = String::new();
        let mut tool_calls = 0;
        agent
            .run(prompt, |ev| match ev {
                AgentEvent::AssistantText(t) => answer = t,
                AgentEvent::ToolCall { name, .. } => {
                    tool_calls += 1;
                    tracing::debug!(tool = %name, "sub-agent tool call");
                }
                _ => {}
            })
            .await?;
        let u = agent.usage();
        Ok(TaskOutcome { answer, turns: u.requests, tool_calls, input_tokens: u.input_tokens, output_tokens: u.output_tokens })
    }
}

/// The tool result text: the answer, then one line of accounting.
pub fn format_outcome(o: &TaskOutcome) -> String {
    let answer = if o.answer.trim().is_empty() { "(the sub-agent gave no final answer)" } else { o.answer.trim() };
    format!("{answer}\n\n[sub-agent: {} turn(s), {} tool call(s), {} input + {} output tokens]", o.turns, o.tool_calls, o.input_tokens, o.output_tokens)
}

#[async_trait]
impl Tool for TaskTool {
    fn name(&self) -> &str {
        TASK_TOOL_NAME
    }

    fn description(&self) -> &str {
        "Run a sub-agent on a self-contained task and get back only its final answer. It starts with an empty \
         conversation and has the same tools as you (except task), so use it for exploration or research whose \
         intermediate steps would clutter this conversation: 'find every place X is used and summarize', 'check \
         whether the tests cover Y'. The prompt must contain everything it needs — it cannot see this conversation. \
         Say what to return. It cannot start further sub-agents."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "prompt": {"type": "string", "description": "The complete, self-contained task, including what to return"},
                "description": {"type": "string", "description": "A few words naming the task, shown to the user"}
            },
            "required": ["prompt"]
        })
    }

    async fn call(&self, ctx: &ToolContext, input: Value) -> ToolResult {
        let a: TaskArgs = serde_json::from_value(input).map_err(|e| ToolError::InvalidInput(e.to_string()))?;
        if a.prompt.trim().is_empty() {
            return Err(ToolError::InvalidInput("prompt is empty".into()));
        }
        tracing::info!(description = %a.description, "sub-agent starting");
        let outcome = self.run(ctx.cwd.clone(), &a.prompt).await.map_err(|e| ToolError::Failed(format!("sub-agent failed: {e:#}")))?;
        Ok(format_outcome(&outcome))
    }
}
