mod highlight;
mod markdown;
mod permission;
mod settings;
mod tui;

use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use agentiloop_core::{Agent, AgentConfig, AgentEvent, ModelInfo, Provider, Session, ToolContext};
use anyhow::{Context, Result};
use clap::Parser;
use rustyline::error::ReadlineError;

/// AgentiLoop — a cross-platform agentic coding loop for your terminal.
#[derive(Parser, Debug)]
#[command(name = "agentiloop", version, about)]
struct Cli {
    /// Model backend: `anthropic`, `openai` (OpenAI-compatible: OpenAI, Ollama,
    /// LM Studio, Groq, OpenRouter, … via OPENAI_BASE_URL), or `omlx` (local
    /// oMLX server, http://localhost:8000/v1). Auto-detected from which
    /// credentials are set when omitted.
    #[arg(short, long, env = "AGENTILOOP_PROVIDER")]
    provider: Option<String>,

    /// Model id to use. Defaults to the last model picked with /model for this
    /// provider (~/.agentiloop/settings.json), then the provider's default.
    #[arg(short, long, env = "AGENTILOOP_MODEL")]
    model: Option<String>,

    /// Skip all permission prompts (dangerous; intended for CI).
    #[arg(long, env = "AGENTILOOP_YES")]
    yes: bool,

    /// Max provider round-trips per prompt.
    #[arg(long, default_value_t = 50)]
    max_turns: usize,

    /// Summarize the conversation once a request reaches this many input tokens (0 = never).
    #[arg(long, default_value_t = 150_000, env = "AGENTILOOP_COMPACT_AT")]
    compact_at: u64,

    /// Working directory the agent operates in (defaults to cwd).
    #[arg(short = 'C', long)]
    cwd: Option<PathBuf>,

    /// Resume a saved session by id (see /sessions).
    #[arg(short = 'r', long, conflicts_with = "continue_last")]
    resume: Option<String>,

    /// Resume the most recent session for this working directory.
    #[arg(short = 'c', long = "continue")]
    continue_last: bool,

    /// Full-screen terminal UI (ratatui) instead of the line REPL.
    #[arg(long, env = "AGENTILOOP_TUI", conflicts_with = "prompt")]
    tui: bool,

    /// One-shot prompt. If omitted, starts an interactive REPL.
    prompt: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(io::stderr)
        .init();

    let cli = Cli::parse();

    let cwd = match cli.cwd {
        Some(p) => p.canonicalize()?,
        None => std::env::current_dir()?,
    };

    let provider = agentiloop_provider::from_env(cli.provider.as_deref())?;
    let tools = agentiloop_tools::default_registry();
    // The TUI answers permission prompts through its own channel-backed policy.
    let (ui_tx, ui_rx) = tokio::sync::mpsc::unbounded_channel::<tui::UiMsg>();
    let policy: agentiloop_core::permission::SharedPolicy = if cli.tui && !cli.yes {
        std::sync::Arc::new(tui::ChannelPolicy::new(ui_tx.clone()))
    } else {
        permission::policy(cli.yes)
    };
    let mut saved = settings::load();
    let sessions_dir = settings::sessions_dir();

    // Resume, if asked: the session's model wins unless --model was given.
    let resumed = match (&cli.resume, cli.continue_last, &sessions_dir) {
        (Some(id), _, Some(dir)) => Some(Session::load(dir, id)?),
        (None, true, Some(dir)) => Session::latest_for(dir, &cwd)?,
        _ => None,
    };
    if cli.continue_last && resumed.is_none() {
        eprintln!("no previous session for {}; starting fresh", cwd.display());
    }

    let model = match cli
        .model
        .or_else(|| resumed.as_ref().map(|s| s.model.clone()))
        .or_else(|| saved.model_for(provider.name()).map(str::to_string))
    {
        Some(m) => m,
        // Local servers (oMLX) have no fixed catalog: take whatever is served first.
        None if provider.default_model().is_empty() => provider
            .list_models()
            .await
            .with_context(|| format!("{} is not reachable; is the server running?", provider.name()))?
            .into_iter()
            .next()
            .map(|m| m.id)
            .with_context(|| format!("{} serves no models; load one or pass --model", provider.name()))?,
        None => provider.default_model().to_string(),
    };
    let config = AgentConfig { model, max_turns: cli.max_turns, compact_at_tokens: cli.compact_at, ..Default::default() };

    let mut agent = Agent::new(provider.clone(), tools, policy, config, ToolContext { cwd: cwd.clone() });
    let mut session = match resumed {
        Some(s) => {
            eprintln!("resumed session {} ({} messages): {}", s.id, s.history.len(), s.title());
            agent.history = s.history.clone();
            s
        }
        None => Session::new(cwd.clone(), provider.name(), agent.model()),
    };

    if !cli.prompt.is_empty() {
        let res = agent.run(&cli.prompt.join(" "), render).await;
        persist(&mut session, &agent, sessions_dir.as_deref());
        return res;
    }

    if cli.tui {
        let status = format!(
            " AgentiLoop  {}  {}  {}  session {} ",
            cwd.display(),
            provider.name(),
            agent.model(),
            session.id
        );
        let (in_tx, mut in_rx) = tokio::sync::mpsc::unbounded_channel::<tui::Input>();
        let app = tui::App::new(status).with_history_file(settings::history_path());
        let ui = tokio::task::spawn_blocking(move || tui::run(app, ui_rx, in_tx));
        // Agent side: one prompt or slash command at a time, until the UI hangs up.
        while let Some(tui::Input::Submit(line)) = in_rx.recv().await {
            if line.starts_with('/') {
                let tx = ui_tx.clone();
                let mut say = |s: String| {
                    let _ = tx.send(tui::UiMsg::Line(s));
                };
                if let Err(e) =
                    slash_command(&line, &mut agent, &*provider, &mut saved, &mut session, sessions_dir.as_deref(), false, &mut say)
                        .await
                {
                    let _ = ui_tx.send(tui::UiMsg::Error(format!("{e:#}")));
                }
            } else {
                let tx = ui_tx.clone();
                if let Err(e) = agent
                    .run(&line, move |ev| {
                        let _ = tx.send(tui::UiMsg::Event(ev));
                    })
                    .await
                {
                    let _ = ui_tx.send(tui::UiMsg::Error(format!("{e:#}")));
                }
                persist(&mut session, &agent, sessions_dir.as_deref());
            }
            let _ = ui_tx.send(tui::UiMsg::Idle);
        }
        drop(ui_tx);
        ui.await??;
        return Ok(());
    }

    eprintln!(
        "AgentiLoop — cwd: {}  provider: {}  model: {}  session: {}  (/help for commands)",
        cwd.display(),
        provider.name(),
        agent.model(),
        session.id
    );
    // rustyline gives us line editing plus up/down arrow recall of earlier prompts.
    let mut rl = rustyline::DefaultEditor::new()?;
    let history = settings::history_path();
    if let Some(p) = &history {
        let _ = rl.load_history(p); // missing on first run
    }
    loop {
        let line = match rl.readline("\n> ") {
            Ok(l) => l,
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(e) => return Err(e.into()),
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let _ = rl.add_history_entry(line);
        if line == "/exit" || line == "/quit" {
            break;
        }
        if line.starts_with('/') {
            slash_command(line, &mut agent, &*provider, &mut saved, &mut session, sessions_dir.as_deref(), true, &mut |s| {
                eprintln!("{s}")
            })
            .await?;
            continue;
        }
        if let Err(e) = agent.run(line, render).await {
            eprintln!("error: {e:#}");
        }
        persist(&mut session, &agent, sessions_dir.as_deref());
    }
    if let Some(p) = &history {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = rl.save_history(p) {
            eprintln!("warning: could not save history: {e}");
        }
    }
    Ok(())
}

/// Snapshot the agent's history into the session file. Empty histories are not written.
fn persist(session: &mut Session, agent: &Agent, dir: Option<&Path>) {
    let Some(dir) = dir else { return };
    session.history = agent.history.clone();
    session.model = agent.model().to_string();
    if session.history.is_empty() {
        return;
    }
    if let Err(e) = session.save(dir) {
        eprintln!("warning: could not save session: {e:#}");
    }
}

/// Fallback catalog used when `/v1/models` can't be reached (newest first).
const FALLBACK_MODELS: &[(&str, &str)] = &[
    ("claude-fable-5-1", "Claude Fable 5.1"),
    ("claude-opus-5", "Claude Opus 5"),
    ("claude-sonnet-5", "Claude Sonnet 5"),
    ("claude-fable-5", "Claude Fable 5"),
    ("claude-opus-4-8", "Claude Opus 4.8"),
    ("claude-opus-4-7", "Claude Opus 4.7"),
    ("claude-sonnet-4-6", "Claude Sonnet 4.6"),
    ("claude-opus-4-6", "Claude Opus 4.6"),
    ("claude-opus-4-5-20251101", "Claude Opus 4.5"),
    ("claude-haiku-4-5-20251001", "Claude Haiku 4.5"),
    ("claude-sonnet-4-5-20250929", "Claude Sonnet 4.5"),
];

/// Live model list from the provider. Anthropic falls back to the static
/// catalog when `/v1/models` can't be reached; other providers report the error.
async fn fetch_models(provider: &dyn Provider) -> Vec<ModelInfo> {
    match provider.list_models().await {
        Ok(list) if !list.is_empty() => list,
        Ok(_) if provider.name() == "anthropic" => fallback_models(),
        Ok(_) => {
            eprintln!("provider returned no models");
            Vec::new()
        }
        Err(e) if provider.name() == "anthropic" => {
            tracing::warn!("model list fetch failed, using fallback: {e:#}");
            fallback_models()
        }
        Err(e) => {
            eprintln!("could not list models: {e:#}");
            Vec::new()
        }
    }
}

fn fallback_models() -> Vec<ModelInfo> {
    FALLBACK_MODELS
        .iter()
        .map(|(id, name)| ModelInfo { id: id.to_string(), display_name: name.to_string(), created_at: String::new() })
        .collect()
}

/// Runs a `/command`. Output lines go through `say` so the REPL (stderr) and
/// the TUI (transcript) share one implementation; `interactive` allows the
/// `/model` picker to read a choice from stdin.
async fn slash_command(
    line: &str,
    agent: &mut Agent,
    provider: &dyn Provider,
    saved: &mut settings::Settings,
    session: &mut Session,
    sessions_dir: Option<&Path>,
    interactive: bool,
    say: &mut dyn FnMut(String),
) -> Result<()> {
    let (cmd, arg) = line.split_once(' ').map_or((line, ""), |(c, a)| (c, a.trim()));
    match cmd {
        "/clear" => {
            agent.clear();
            *session = Session::new(session.cwd.clone(), provider.name(), agent.model());
            say(format!("context and tool history cleared; new session {}", session.id));
        }
        "/model" => {
            let models = fetch_models(provider).await;
            // `/model 2` picks entry #2 directly; `/model <id>` sets an id.
            let pick = if arg.is_empty() {
                for (i, m) in models.iter().enumerate() {
                    let mark = if m.id == agent.model() { "*" } else { " " };
                    let date = m.created_at.get(..10).unwrap_or("");
                    say(format!("{mark} {:>2}. {:<22} {:<28} {date}", i + 1, m.display_name, m.id));
                }
                if !interactive {
                    say(format!("current: {}  — pick with /model <n|id>", agent.model()));
                    return Ok(());
                }
                eprint!("select [1-{}] or type a model id (enter to keep {}): ", models.len(), agent.model());
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                line.trim().to_string()
            } else {
                arg.to_string()
            };
            if pick.is_empty() {
                return Ok(());
            }
            match pick.parse::<usize>() {
                Ok(n) if (1..=models.len()).contains(&n) => agent.set_model(models[n - 1].id.clone()),
                Ok(_) => {
                    say(format!("out of range [1-{}]", models.len()));
                    return Ok(());
                }
                Err(_) => agent.set_model(pick),
            }
            saved.set_model(provider.name(), agent.model());
            if let Err(e) = settings::save(saved) {
                say(format!("warning: could not save settings: {e:#}"));
            }
            say(format!("model: {}", agent.model()));
        }
        "/compact" => match agent.compact().await {
            Ok(Some(AgentEvent::Compacted { before_tokens, messages_dropped })) => {
                say(compacted_line(before_tokens, messages_dropped));
                persist(session, agent, sessions_dir);
            }
            Ok(_) => say("nothing to compact".into()),
            Err(e) => say(format!("compaction failed: {e:#}")),
        },
        "/sessions" => {
            let Some(dir) = sessions_dir else {
                say("no home directory; sessions are not saved".into());
                return Ok(());
            };
            let list = Session::list(dir)?;
            if list.is_empty() {
                say("no saved sessions".into());
            }
            for (i, s) in list.iter().take(20).enumerate() {
                let mark = if s.id == session.id { "*" } else { " " };
                say(format!("{mark} {:>2}. {:<22} {:>3} msgs  {:<24} {}", i + 1, s.id, s.history.len(), s.model, s.title()));
            }
        }
        "/resume" => {
            let Some(dir) = sessions_dir else {
                say("no home directory; sessions are not saved".into());
                return Ok(());
            };
            if arg.is_empty() {
                say("usage: /resume <id|n>  (see /sessions)".into());
                return Ok(());
            }
            // `/resume 2` picks entry #2 from the /sessions listing.
            let id = match arg.parse::<usize>() {
                Ok(n) => match Session::list(dir)?.into_iter().nth(n.saturating_sub(1)) {
                    Some(s) if n >= 1 => s.id,
                    _ => {
                        say(format!("no session #{n}"));
                        return Ok(());
                    }
                },
                Err(_) => arg.to_string(),
            };
            match Session::load(dir, &id) {
                Ok(s) => {
                    agent.clear();
                    agent.history = s.history.clone();
                    agent.set_model(s.model.clone());
                    say(format!("resumed session {} ({} messages, model {}): {}", s.id, s.history.len(), s.model, s.title()));
                    *session = s;
                }
                Err(e) => say(format!("{e:#}")),
            }
        }
        "/help" => say(HELP.into()),
        _ => say(format!("unknown command {cmd} (try /help)")),
    }
    Ok(())
}

const HELP: &str = "/model [n|id]   show picker, or pick #n / set id directly\n\
/compact        summarize the conversation to free context\n\
/sessions       list saved sessions (newest first)\n\
/resume <id|n>  load a saved session into this REPL\n\
/clear          clear context and start a new session\n\
/exit           quit";

pub(crate) fn compacted_line(before_tokens: u64, messages_dropped: usize) -> String {
    format!("\u{1f4e6} context compacted ({before_tokens} tokens, {messages_dropped} messages → summary)")
}

fn render(ev: AgentEvent) {
    match ev {
        AgentEvent::AssistantTextDelta(t) => {
            print!("{t}");
            let _ = io::stdout().flush();
        }
        // Deltas already printed the text; terminate the line.
        AgentEvent::AssistantText(_) => println!(),
        AgentEvent::ToolCall { name, input, .. } => {
            eprintln!("\u{1f527} {name} {}", compact(&input));
        }
        AgentEvent::ToolResult { output, is_error, .. } => {
            let mark = if is_error { "✖" } else { "✓" };
            let preview: String = output.lines().take(8).collect::<Vec<_>>().join("\n   ");
            eprintln!("   {mark} {preview}");
        }
        AgentEvent::TurnComplete { input_tokens, output_tokens } => {
            tracing::debug!(input_tokens, output_tokens, "turn");
        }
        AgentEvent::Compacted { before_tokens, messages_dropped } => {
            eprintln!("{}", compacted_line(before_tokens, messages_dropped));
        }
        AgentEvent::Done { .. } => {}
    }
}

pub(crate) fn compact(v: &serde_json::Value) -> String {
    let s = v.to_string();
    if s.len() > 120 { format!("{}…", &s[..s.floor_char_boundary(120)]) } else { s }
}
