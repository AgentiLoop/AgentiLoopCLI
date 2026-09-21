mod permission;

use std::io::{self, BufRead, Write};
use std::sync::Arc;

use agentiloop_core::{Agent, AgentConfig, AgentEvent, ToolContext};
use agentiloop_provider::AnthropicProvider;
use anyhow::Result;
use clap::Parser;

/// AgentiLoop — a cross-platform agentic coding loop for your terminal.
#[derive(Parser, Debug)]
#[command(name = "agentiloop", version, about)]
struct Cli {
    /// Model id to use.
    #[arg(short, long, env = "AGENTILOOP_MODEL", default_value = "claude-sonnet-4-5")]
    model: String,

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
    let config = AgentConfig { model: cli.model, max_turns: cli.max_turns, ..Default::default() };

    let mut agent = Agent::new(provider, tools, policy, config, ToolContext { cwd: cwd.clone() });

    if !cli.prompt.is_empty() {
        return agent.run(&cli.prompt.join(" "), render).await;
    }

    eprintln!("AgentiLoop — cwd: {}  (type /exit to quit)", cwd.display());
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
        if let Err(e) = agent.run(line, render).await {
            eprintln!("error: {e:#}");
        }
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
