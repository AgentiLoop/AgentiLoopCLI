//! Agent loop tests driven by a scripted mock provider — no network.

use std::sync::{Arc, Mutex};

use agentiloop_core::{
    Agent, AgentConfig, AgentEvent, ContentBlock, Message, ModelInfo, Permission, PermissionPolicy, Provider,
    ProviderRequest, ProviderResponse, Role, StopReason, Tool, ToolContext, ToolRegistry, ToolResult,
};
use async_trait::async_trait;
use serde_json::{json, Value};

/// Replays a fixed list of responses, recording every request it received.
struct ScriptedProvider {
    responses: Mutex<Vec<ProviderResponse>>,
    requests: Mutex<Vec<ProviderRequest>>,
}

impl ScriptedProvider {
    fn new(responses: Vec<ProviderResponse>) -> Arc<Self> {
        Arc::new(Self { responses: Mutex::new(responses), requests: Mutex::new(Vec::new()) })
    }
}

#[async_trait]
impl Provider for ScriptedProvider {
    fn name(&self) -> &str {
        "scripted"
    }
    fn default_model(&self) -> &str {
        "mock"
    }
    async fn list_models(&self) -> anyhow::Result<Vec<ModelInfo>> {
        Ok(vec![])
    }
    async fn complete(&self, req: ProviderRequest) -> anyhow::Result<ProviderResponse> {
        self.requests.lock().unwrap().push(req);
        let mut r = self.responses.lock().unwrap();
        anyhow::ensure!(!r.is_empty(), "scripted provider ran out of responses");
        Ok(r.remove(0))
    }
}

fn text(t: &str) -> ProviderResponse {
    ProviderResponse {
        message: Message { role: Role::Assistant, content: vec![ContentBlock::Text { text: t.into() }] },
        stop_reason: StopReason::EndTurn,
        input_tokens: 10,
        output_tokens: 5,
    }
}

fn tool_call(id: &str, name: &str, input: Value) -> ProviderResponse {
    ProviderResponse {
        message: Message {
            role: Role::Assistant,
            content: vec![ContentBlock::ToolUse { id: id.into(), name: name.into(), input }],
        },
        stop_reason: StopReason::ToolUse,
        input_tokens: 10,
        output_tokens: 5,
    }
}

/// Echoes its `msg` argument back; mutating so it exercises the permission gate.
struct Echo;

#[async_trait]
impl Tool for Echo {
    fn name(&self) -> &str {
        "echo"
    }
    fn description(&self) -> &str {
        "echo"
    }
    fn input_schema(&self) -> Value {
        json!({"type":"object","properties":{"msg":{"type":"string"}}})
    }
    fn is_mutating(&self) -> bool {
        true
    }
    async fn call(&self, _ctx: &ToolContext, input: Value) -> ToolResult {
        Ok(format!("echo:{}", input["msg"].as_str().unwrap_or("")))
    }
}

struct DenyMutating;

#[async_trait]
impl PermissionPolicy for DenyMutating {
    async fn check(&self, _tool: &str, is_mutating: bool, _input: &Value) -> Permission {
        if is_mutating { Permission::Deny } else { Permission::Allow }
    }
}

fn agent(provider: Arc<ScriptedProvider>, policy: Arc<dyn PermissionPolicy>, max_turns: usize) -> Agent {
    let mut tools = ToolRegistry::new();
    tools.register(Echo);
    let config = AgentConfig { model: "mock".into(), max_turns, ..Default::default() };
    Agent::new(provider, tools, policy, config, ToolContext { cwd: std::env::temp_dir() })
}

fn collect(agent: &mut Agent, prompt: &str) -> (anyhow::Result<()>, Vec<AgentEvent>) {
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink = events.clone();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let res = rt.block_on(agent.run(prompt, move |e| sink.lock().unwrap().push(e)));
    let events = events.lock().unwrap().clone();
    (res, events)
}

#[test]
fn plain_reply_ends_turn_and_records_history() {
    let p = ScriptedProvider::new(vec![text("hello")]);
    let mut a = agent(p.clone(), Arc::new(agentiloop_core::permission::AllowAll), 5);
    let (res, events) = collect(&mut a, "hi");
    res.unwrap();

    // user prompt + assistant reply
    assert_eq!(a.history.len(), 2);
    assert_eq!(a.history[0].role, Role::User);
    assert_eq!(a.history[1].text(), "hello");

    // default complete_stream emits the whole text as one delta, then the full text
    assert!(matches!(&events[0], AgentEvent::AssistantTextDelta(t) if t == "hello"));
    assert!(events.iter().any(|e| matches!(e, AgentEvent::AssistantText(t) if t == "hello")));
    assert!(matches!(events.last(), Some(AgentEvent::Done { stop_reason: StopReason::EndTurn })));

    // the request carried the system prompt and the tool spec
    let reqs = p.requests.lock().unwrap();
    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].system, agentiloop_core::agent::DEFAULT_SYSTEM_PROMPT);
    assert_eq!(reqs[0].tools.len(), 1);
    assert_eq!(reqs[0].tools[0].name, "echo");
}

#[test]
fn tool_call_round_trip_feeds_result_back() {
    let p = ScriptedProvider::new(vec![tool_call("t1", "echo", json!({"msg": "ping"})), text("done")]);
    let mut a = agent(p.clone(), Arc::new(agentiloop_core::permission::AllowAll), 5);
    let (res, events) = collect(&mut a, "go");
    res.unwrap();

    assert!(events.iter().any(|e| matches!(e, AgentEvent::ToolCall { id, name, .. } if id == "t1" && name == "echo")));
    assert!(events
        .iter()
        .any(|e| matches!(e, AgentEvent::ToolResult { id, output, is_error: false, .. } if id == "t1" && output == "echo:ping")));

    // user, assistant(tool_use), user(tool_result), assistant(text)
    assert_eq!(a.history.len(), 4);
    match &a.history[2].content[0] {
        ContentBlock::ToolResult { tool_use_id, content, is_error } => {
            assert_eq!(tool_use_id, "t1");
            assert_eq!(content, "echo:ping");
            assert!(!is_error);
        }
        other => panic!("expected tool_result, got {other:?}"),
    }

    // second request saw the full history including the tool result
    let reqs = p.requests.lock().unwrap();
    assert_eq!(reqs.len(), 2);
    assert_eq!(reqs[1].messages.len(), 3);
}

#[test]
fn unknown_tool_is_reported_as_error_result_not_crash() {
    let p = ScriptedProvider::new(vec![tool_call("t1", "nope", json!({})), text("ok")]);
    let mut a = agent(p, Arc::new(agentiloop_core::permission::AllowAll), 5);
    let (res, events) = collect(&mut a, "go");
    res.unwrap();
    assert!(events
        .iter()
        .any(|e| matches!(e, AgentEvent::ToolResult { is_error: true, output, .. } if output.contains("unknown tool"))));
}

#[test]
fn denied_permission_becomes_error_result() {
    let p = ScriptedProvider::new(vec![tool_call("t1", "echo", json!({"msg": "x"})), text("ok")]);
    let mut a = agent(p, Arc::new(DenyMutating), 5);
    let (res, events) = collect(&mut a, "go");
    res.unwrap();
    assert!(events
        .iter()
        .any(|e| matches!(e, AgentEvent::ToolResult { is_error: true, output, .. } if output.contains("permission denied"))));
}

#[test]
fn max_turns_stops_runaway_loop() {
    let looping: Vec<_> = (0..10).map(|i| tool_call(&format!("t{i}"), "echo", json!({"msg": "again"}))).collect();
    let p = ScriptedProvider::new(looping);
    let mut a = agent(p.clone(), Arc::new(agentiloop_core::permission::AllowAll), 3);
    let (res, _) = collect(&mut a, "loop");
    let err = res.unwrap_err().to_string();
    assert!(err.contains("max_turns (3)"), "{err}");
    assert_eq!(p.requests.lock().unwrap().len(), 3);
}

#[test]
fn clear_drops_history() {
    let p = ScriptedProvider::new(vec![text("a"), text("b")]);
    let mut a = agent(p, Arc::new(agentiloop_core::permission::AllowAll), 5);
    collect(&mut a, "one").0.unwrap();
    assert_eq!(a.history.len(), 2);
    a.clear();
    assert!(a.history.is_empty());
    collect(&mut a, "two").0.unwrap();
    assert_eq!(a.history.len(), 2);
}
