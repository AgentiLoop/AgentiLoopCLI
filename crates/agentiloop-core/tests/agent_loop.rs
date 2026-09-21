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

// ---- compaction -------------------------------------------------------------

fn agent_with(provider: Arc<ScriptedProvider>, config: AgentConfig) -> Agent {
    let mut tools = ToolRegistry::new();
    tools.register(Echo);
    Agent::new(provider, tools, Arc::new(agentiloop_core::permission::AllowAll), config, ToolContext {
        cwd: std::env::temp_dir(),
    })
}

fn big(t: &str, input_tokens: u64) -> ProviderResponse {
    ProviderResponse { input_tokens, ..text(t) }
}

#[test]
fn compact_replaces_history_with_summary_via_provider() {
    let p = ScriptedProvider::new(vec![text("first"), text("SUMMARY")]);
    let mut a = agent_with(p.clone(), AgentConfig { compact_at_tokens: 0, ..Default::default() });
    collect(&mut a, "do a thing").0.unwrap();
    assert_eq!(a.history.len(), 2);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let ev = rt.block_on(a.compact()).unwrap().expect("event");
    assert!(matches!(ev, AgentEvent::Compacted { messages_dropped: 2, .. }));

    // summary user message + assistant ack
    assert_eq!(a.history.len(), 2);
    assert_eq!(a.history[0].role, Role::User);
    assert!(a.history[0].text().contains("SUMMARY"), "{}", a.history[0].text());
    assert_eq!(a.history[1].role, Role::Assistant);
    assert_eq!(a.last_input_tokens(), 0);

    // the summarization request carried no tools and a transcript of the old history
    let reqs = p.requests.lock().unwrap();
    let sum_req = &reqs[1];
    assert!(sum_req.tools.is_empty());
    assert_eq!(sum_req.messages.len(), 1);
    let body = sum_req.messages[0].text();
    assert!(body.contains("USER: do a thing"), "{body}");
    assert!(body.contains("ASSISTANT: first"), "{body}");
}

#[test]
fn compact_on_empty_history_is_noop() {
    let p = ScriptedProvider::new(vec![]);
    let mut a = agent_with(p.clone(), AgentConfig::default());
    let rt = tokio::runtime::Runtime::new().unwrap();
    assert!(rt.block_on(a.compact()).unwrap().is_none());
    assert!(p.requests.lock().unwrap().is_empty());
}

#[test]
fn compact_fails_on_empty_summary_and_keeps_history() {
    let p = ScriptedProvider::new(vec![text("first"), text("   ")]);
    let mut a = agent_with(p, AgentConfig { compact_at_tokens: 0, ..Default::default() });
    collect(&mut a, "x").0.unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let err = rt.block_on(a.compact()).unwrap_err().to_string();
    assert!(err.contains("empty summary"), "{err}");
    assert_eq!(a.history.len(), 2, "history must be untouched on failure");
}

#[test]
fn auto_compacts_before_next_run_when_threshold_reached() {
    // run 1 reports 1000 input tokens (>= threshold 500) → run 2 compacts first.
    let p = ScriptedProvider::new(vec![big("first", 1000), text("SUMMARY"), text("second")]);
    let mut a = agent_with(p.clone(), AgentConfig { compact_at_tokens: 500, ..Default::default() });
    collect(&mut a, "one").0.unwrap();
    assert_eq!(a.last_input_tokens(), 1000);

    let (res, events) = collect(&mut a, "two");
    res.unwrap();
    assert!(matches!(events[0], AgentEvent::Compacted { before_tokens: 1000, messages_dropped: 2 }));

    // summary, ack, "two", "second"
    assert_eq!(a.history.len(), 4);
    assert!(a.history[0].text().contains("SUMMARY"));
    assert_eq!(a.history[2].text(), "two");
    assert_eq!(a.history[3].text(), "second");

    // the request for "two" was built from the compacted history
    let reqs = p.requests.lock().unwrap();
    assert_eq!(reqs.len(), 3);
    assert_eq!(reqs[2].messages.len(), 3);
    assert!(reqs[2].messages[0].text().contains("SUMMARY"));
}

#[test]
fn auto_compacts_mid_loop_after_tool_results() {
    let mut call = tool_call("t1", "echo", json!({"msg": "a"}));
    call.input_tokens = 900;
    let p = ScriptedProvider::new(vec![call, text("SUMMARY"), text("finished")]);
    let mut a = agent_with(p.clone(), AgentConfig { compact_at_tokens: 500, ..Default::default() });
    let (res, events) = collect(&mut a, "go");
    res.unwrap();

    let idx = events.iter().position(|e| matches!(e, AgentEvent::Compacted { before_tokens: 900, .. })).expect("compacted");
    // compaction happened after the tool result and before the final answer
    assert!(events[..idx].iter().any(|e| matches!(e, AgentEvent::ToolResult { .. })));
    assert!(events[idx..].iter().any(|e| matches!(e, AgentEvent::AssistantText(t) if t == "finished")));

    // summary, ack, continue-prompt, final
    assert_eq!(a.history.len(), 4);
    assert!(a.history[2].text().contains("Continue the task"));
    assert_eq!(a.history[3].text(), "finished");
    let reqs = p.requests.lock().unwrap();
    assert_eq!(reqs.len(), 3);
    assert!(reqs[2].messages.iter().all(|m| m.tool_uses().count() == 0), "old tool_use blocks must be gone");
}

#[test]
fn compaction_disabled_when_threshold_is_zero() {
    let p = ScriptedProvider::new(vec![big("first", 1_000_000), text("second")]);
    let mut a = agent_with(p.clone(), AgentConfig { compact_at_tokens: 0, ..Default::default() });
    collect(&mut a, "one").0.unwrap();
    let (_, events) = collect(&mut a, "two");
    assert!(!events.iter().any(|e| matches!(e, AgentEvent::Compacted { .. })));
    assert_eq!(p.requests.lock().unwrap().len(), 2);
}

#[test]
fn transcript_trims_long_tool_results() {
    let long = "x".repeat(5_000);
    let history = vec![
        Message::user_text("hi"),
        Message {
            role: Role::Assistant,
            content: vec![ContentBlock::ToolUse { id: "t".into(), name: "bash".into(), input: json!({"command": "ls"}) }],
        },
        Message::tool_results(vec![ContentBlock::ToolResult { tool_use_id: "t".into(), content: long, is_error: true }]),
    ];
    let t = agentiloop_core::agent::transcript(&history);
    assert!(t.contains("USER: hi"));
    assert!(t.contains("ASSISTANT → tool bash {\"command\":\"ls\"}"));
    assert!(t.contains("tool error: "));
    assert!(t.contains("…[3000 more bytes]"), "{t}");
    assert!(t.len() < 2_500, "{}", t.len());
}