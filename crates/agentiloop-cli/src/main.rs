mod permission;
mod settings;

use std::io::{self, BufRead, Write};
use std::sync::Arc;

use agentiloop_core::{Agent, AgentConfig, AgentEvent, ToolContext};
use agentiloop_provider::{AnthropicProvider, ModelInfo};
use anyhow::Result;
use clap::Parser;

const DEFAULT_MODEL: &str = "claude-sonnet-5";

/// AgentiLoop — a cross-platform agentic coding loop for your terminal.
#[derive(Parser, Debug)]
#[command(name = "agentiloop", version, about)]
struct Cli {
    /// Model id to use. Defaults to the last model picked with /model
    /// (~/.agentiloop/settings.json), then claude-sonnet-5.
    #[arg(short, long, env = "AGENTILOOP_MODEL")]
    model: Option<String>,

    /// Skip all permission prompts (dangerous; intended for CI).
    #[arg(long, env = "AGENTILOOP_YES")]
    yes: bool,

    /// Max provider round-trips per prompt.
    #[arg(long, default_value_t = 50)]
    max_turns: usize,

    /// Working directory the agent operates in (defaults to cwd).
    #[arg(short = 'C', long)]
    cwd: Option<std::path::PathBuf>,

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

    let provider = Arc::new(AnthropicProvider::from_env()?);
    let tools = agentiloop_tools::default_registry();
    let policy = permission::policy(cli.yes);
    let mut saved = settings::load();
    let model = cli.model.or_else(|| saved.model.clone()).unwrap_or_else(|| DEFAULT_MODEL.into());
    let config = AgentConfig { model, max_turns: cli.max_turns, ..Default::default() };

    let mut agent = Agent::new(provider.clone(), tools, policy, config, ToolContext { cwd: cwd.clone() });

    if !cli.prompt.is_empty() {
        return agent.run(&cli.prompt.join(" "), render).await;
    }

    eprintln!("AgentiLoop — cwd: {}  model: {}  (/help for commands)", cwd.display(), agent.model());
    let stdin = io::stdin();
    loop {
        eprint!("\n> ");
        io::stderr().flush()?;
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line == "/exit" || line == "/quit" {
            break;
        }
        if line.starts_with('/') {
            slash_command(line, &mut agent, &provider, &mut saved).await?;
            continue;
        }
        if let Err(e) = agent.run(line, render).await {
            eprintln!("error: {e:#}");
        }
    }
    Ok(())
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

/// Live model list from the API, falling back to the static catalog.
async fn fetch_models(provider: &AnthropicProvider) -> Vec<ModelInfo> {
    match provider.list_models().await {
        Ok(list) if !list.is_empty() => list,
        Ok(_) => fallback_models(),
        Err(e) => {
            tracing::warn!("model list fetch failed, using fallback: {e:#}");
            fallback_models()
        }
    }
}

fn fallback_models() -> Vec<ModelInfo> {
    FALLBACK_MODELS
        .iter()
        .map(|(id, name)| ModelInfo { id: id.to_string(), display_name: name.to_string(), created_at: String::new() })
        .collect()
}

async fn slash_command(
    line: &str,
    agent: &mut Agent,
    provider: &AnthropicProvider,
    saved: &mut settings::Settings,
) -> Result<()> {
    let (cmd, arg) = line.split_once(' ').map_or((line, ""), |(c, a)| (c, a.trim()));
    match cmd {
        "/clear" => {
            agent.clear();
            eprintln!("context and tool history cleared");
        }
        "/model" => {
            let models = fetch_models(provider).await;
            // `/model 2` picks entry #2 directly; `/model <id>` sets an id.
            let pick = if arg.is_empty() {
                for (i, m) in models.iter().enumerate() {
                    let mark = if m.id == agent.model() { "*" } else { " " };
                    let date = m.created_at.get(..10).unwrap_or("");
                    eprintln!("{mark} {:>2}. {:<22} {:<28} {date}", i + 1, m.display_name, m.id);
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
                    eprintln!("out of range [1-{}]", models.len());
                    return Ok(());
                }
                Err(_) => agent.set_model(pick),
            }
            saved.model = Some(agent.model().to_string());
            if let Err(e) = settings::save(saved) {
                eprintln!("warning: could not save settings: {e:#}");
            }
            eprintln!("model: {}", agent.model());
        }
        "/help" => {
            eprintln!("/model [n|id]  show picker, or pick #n / set id directly\n/clear         clear context and tool history\n/exit          quit");
        }
        _ => eprintln!("unknown command {cmd} (try /help)"),
    }
    Ok(())
}

fn render(ev: AgentEvent) {
    match ev {
        AgentEvent::AssistantText(t) => println!("{t}"),
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
        AgentEvent::Done { .. } => {}
    }
}

fn compact(v: &serde_json::Value) -> String {
    let s = v.to_string();
    if s.len() > 120 { format!("{}…", &s[..s.floor_char_boundary(120)]) } else { s }
}
