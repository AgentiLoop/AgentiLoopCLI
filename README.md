# AgentiLoop CLI

A cross-platform (macOS / Linux / Windows) agentic coding loop in the spirit of Claude Code, written in Rust. Created with AgentiLoop Agent! This is our baby. No binaries are available at this time. Compile from source. Rust / Cargo is required.

## AgentiLoop Coding already in Action
<img width="2048" height="1152" alt="image" src="https://github.com/user-attachments/assets/d910bbd2-b47d-4c4a-89af-ac0753ccf279" />


## Layout

```
crates/
  agentiloop-core/      message model, Tool trait + registry, Provider trait, permission gate, the agent loop
  agentiloop-provider/  model backends: Anthropic Messages API, OpenAI-compatible Chat Completions, oMLX
  agentiloop-tools/     built-in tools: read_file, write_file, edit_file, list_dir, bash
  agentiloop-mcp/       MCP client (stdio, Streamable HTTP, legacy HTTP+SSE), ported from Agent!'s AgentMCP
  agentiloop-cli/       `agentiloop` binary: REPL, TUI and one-shot mode, interactive permission prompts
```

### Dependencies

The workspace uses 18 external crates directly, and every crate lists only the ones it actually uses:

| Crate | Used for | Used by |
|---|---|---|
| `tokio`, `futures`, `async-trait` | async runtime, streams, async traits | all |
| `serde`, `serde_json` | JSON messages, sessions, settings, MCP JSON-RPC | all |
| `reqwest` (rustls) | HTTP to model providers and MCP servers | provider, mcp |
| `anyhow`, `thiserror` | error handling | cli, core, provider, mcp / core |
| `tracing`, `tracing-subscriber` | logging (`RUST_LOG`) | cli, core, mcp / cli |
| `dirs` | home directory (`~/.agentiloop`, `~/.omlx`) | cli, provider, mcp |
| `clap` | command-line options | cli |
| `rustyline` | line REPL editing and history | cli |
| `ratatui`, `unicode-width`, `textwrap` | TUI rendering and wrapping | cli |
| `pulldown-cmark`, `syntect` | Markdown and syntax highlighting (pure-Rust regex, no C build) | cli |

## Compile

You need a stable Rust toolchain from [rustup](https://rustup.rs). It builds the same way on macOS, Linux and Windows.

```sh
git clone https://github.com/AgentiLoop/AgentiLoopCLI.git
cd AgentiLoopCLI
cargo build --release                          # → target/release/agentiloop
cargo install --path crates/agentiloop-cli     # optional: puts `agentiloop` in ~/.cargo/bin
```

From source you can run it as `cargo run -- <options>`. Everything after `--` goes to agentiloop. You can also run the built binary directly: `target/release/agentiloop <options>`, or `agentiloop <options>` after `cargo install`.

## Build & run

```sh
# Anthropic: either a standard API key (sk-ant-api…) or a Claude Code OAuth token
# (sk-ant-oat01-…, from `claude setup-token`) — auth scheme is auto-detected.
export ANTHROPIC_API_KEY=sk-ant-...
cargo run -- "list the files in this project and summarize the layout"   # one-shot
cargo run                                                                # REPL
cargo run -- --yes -C /path/to/repo "fix the failing test"               # no prompts

# OpenAI-compatible (OpenAI, Ollama, LM Studio, Groq, OpenRouter, Together, vLLM, …)
export OPENAI_API_KEY=sk-...                                             # OpenAI itself
cargo run -- -p openai -m gpt-4o-mini "summarize the layout"
export OPENAI_BASE_URL=http://localhost:11434/v1                         # local Ollama, no key needed
cargo run -- -m qwen3:4b

# oMLX (https://omlx.ai) — local MLX server on Apple Silicon, http://localhost:8000/v1
# No key or model needed: the first model oMLX serves is used; /model lists the rest.
cargo run -- -p omlx "summarize the layout"
export OMLX_BASE_URL=http://localhost:8000/v1  OMLX_API_KEY=...            # optional overrides (OMLX_PORT also honoured)
```

Provider is auto-detected from which credentials are set (`ANTHROPIC_API_KEY` wins, then `OPENAI_*`, then `OMLX_*`); force one with `--provider` / `AGENTILOOP_PROVIDER`.

## Command-line options

```
agentiloop [OPTIONS] [PROMPT]...
```

If you pass a prompt, agentiloop runs it once and exits. With no prompt it starts the interactive REPL, or the full-screen TUI when you add `--tui`.

| Option | Env var | Description |
|---|---|---|
| `-p, --provider <PROVIDER>` | `AGENTILOOP_PROVIDER` | `anthropic`, `openai` (any OpenAI-compatible server) or `omlx`. Auto-detected when omitted |
| `-m, --model <MODEL>` | `AGENTILOOP_MODEL` | Model id. Defaults to the last `/model` pick for this provider, then the provider default |
| `--tui` | `AGENTILOOP_TUI` | Full-screen terminal UI (ratatui) instead of the line REPL. Can't be combined with a one-shot prompt |
| `--no-tui` | | Use the line REPL even if the TUI was used last time |
| `--yes` | `AGENTILOOP_YES` | Skip all permission prompts (dangerous; meant for CI) |
| `--max-turns <N>` | | Max provider round-trips per prompt (default 50) |
| `--compact-at <TOKENS>` | `AGENTILOOP_COMPACT_AT` | Summarize the conversation once a request reaches this many input tokens (default 150000, 0 = never) |
| `-C, --cwd <DIR>` | | Working directory the agent operates in (defaults to the current directory) |
| `-c, --continue` | | Resume the most recent session for this working directory (already the default for interactive launches) |
| `--new` | | Start a new session instead of continuing the last one |
| `-r, --resume <ID>` | | Resume a saved session by id (see `/sessions`) |
| `--no-mcp` | `AGENTILOOP_NO_MCP` | Don't start MCP servers from `~/.agentiloop/mcp.json` / `./.mcp.json` |
| `-h, --help` / `-V, --version` | | Print help / version |

```sh
cargo run -- --tui                          # TUI, provider auto-detected
cargo run -- -p omlx --tui                  # TUI on a local oMLX server
cargo run -- -c                             # continue the last session here
cargo run -- -C ../other-repo --yes "run the tests and fix failures"
agentiloop -p openai -m gpt-4o-mini --no-mcp "explain src/main.rs"
```

### Remembered launch: set it once, then just run `agentiloop`

agentiloop remembers how you launched it last time. Pass your options once, and from then on a bare `agentiloop` (or `cargo run`) starts the same way and picks up your last conversation.

```sh
agentiloop -p anthropic --tui    # first time: choose provider + UI
agentiloop                       # every time after: Anthropic, same model, TUI, previous session
```

| Remembered | Saved when | Stored in |
|---|---|---|
| Provider (`-p`) | every interactive launch | `settings.json` → `last.provider` |
| TUI on/off (`--tui` / `--no-tui`) | every interactive launch | `last.tui` |
| `--max-turns`, `--compact-at` | every interactive launch | `last.max_turns`, `last.compact_at` |
| Model (`-m` or `/model` pick) | every launch and `/model` change, per provider | `models.<provider>` |
| Conversation | every turn | `sessions/<id>.json`, continued automatically in the same directory |

**Not remembered, on purpose:** `--yes` (skipping permission prompts has to be chosen each time), `--no-mcp`, `-C`, one-shot prompts, and API keys. Keep keys in your shell profile. For example, add this to `~/.zshrc` to load a key stored in the macOS Keychain:

```sh
export ANTHROPIC_API_KEY="$(security find-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w 2>/dev/null)"
```

**Changing or overriding.** Any flag or env var you pass takes priority for that run and becomes the new remembered value:

```sh
agentiloop -p omlx               # switch provider; oMLX's own last model is used
agentiloop -m claude-sonnet-5    # switch model (also remembered per provider via /model)
agentiloop --no-tui              # back to the line REPL from now on
agentiloop --new                 # fresh session (sessions stay listed under /sessions)
agentiloop -r <id>               # a specific older session
```

The previous session is continued only when it's the latest one in the current directory **and** it used the same provider. Switching provider starts a fresh session. One-shot runs (`agentiloop "prompt"`) save their session and model, but they don't change the remembered provider or UI.

**Resolution order**, highest first:
- provider: `-p` / `AGENTILOOP_PROVIDER` → `last.provider` → auto-detect from credentials
- model: `-m` / `AGENTILOOP_MODEL` → resumed session's model → `models.<provider>` → provider default
- UI and limits: flag / env → `last.*` → built-in defaults (REPL, 50 turns, 150000 tokens)

**Reset:** delete `~/.agentiloop/settings.json`, or just its `"last"` block, to forget launch options. Delete `~/.agentiloop/sessions/` to forget conversations. To keep a separate profile, for example for testing, point `AGENTILOOP_HOME` at another directory: `AGENTILOOP_HOME=/tmp/agl-test agentiloop -p omlx`.

Slash commands (REPL and TUI):

| Command | Description |
|---|---|
| `/model [n\|id]` | Show the model picker, or pick #n / set an id directly |
| `/mcp` | List MCP servers and their tools |
| `/compact` | Summarize the conversation to free context |
| `/sessions` | List saved sessions (newest first) |
| `/resume <id\|n>` | Load a saved session |
| `/clear` | Clear context and start a new session |
| `/help` | List commands |
| `/exit` | Quit |

## TUI

`agentiloop --tui` (or `AGENTILOOP_TUI=1`) opens a full-screen ratatui interface: scrolling transcript, prompt box, status bar. While the agent works, the prompt box title shows an animated indicator like ` ✻ Thinking...  12s `: a spinner, what the agent is doing (Thinking, Writing, Running `<tool>`, Waiting for approval, Compacting) and the elapsed time. Permission prompts appear as a modal (`y` / `n` / `a`lways). Keys: Enter send, ↑/↓ prompt history, PgUp/PgDn scroll, Ctrl-U clear line, Ctrl-C quit. All slash commands work; `/model` with no argument lists models — pick with `/model <n|id>`.

Env: `AGENTILOOP_PROVIDER`, `AGENTILOOP_MODEL`, `AGENTILOOP_YES`, `AGENTILOOP_TUI`, `AGENTILOOP_HOME`, `AGENTILOOP_COMPACT_AT`, `AGENTILOOP_NO_MCP`, `ANTHROPIC_API_KEY` (or `ANTHROPIC_OAUTH_TOKEN`), `ANTHROPIC_BASE_URL`, `OPENAI_API_KEY`, `OPENAI_BASE_URL`, `OMLX_BASE_URL`, `OMLX_PORT`, `OMLX_API_KEY`, `RUST_LOG=debug` for token usage.

## MCP servers

Servers listed under `mcpServers` in `~/.agentiloop/mcp.json` and the project's `./.mcp.json` start with the CLI. The format matches Agent!, Claude Code and Claude Desktop. If a server name appears in both files, the project entry wins. Each tool a server exposes becomes an agent tool named `mcp_<server>_<tool>`. If a server has resources, `mcp_read_resource` is added too.

```json
{ "mcpServers": {
    "HelloWorld": { "command": "mcp-server-hello", "args": [], "env": {} },
    "DemoHttp":   { "transport": "http", "url": "http://localhost:8085/mcp", "headers": { "Authorization": "Bearer ${DEMO_TOKEN}" } },
    "Search":     { "url": "https://example.com/api/sse" }
} }
```

- **stdio**: bare command names are looked up in `~/.local/bin`, Homebrew, `~/.cargo/bin` and then `PATH`. The server runs in the project folder, and `DYLD_*` / `LD_PRELOAD` entries in `env` are ignored.
- **http**: Streamable HTTP (POST JSON-RPC, JSON or SSE replies, `Mcp-Session-Id`). A URL ending in `/sse`, `"transport": "sse"` or a non-empty `sseEndpoint` switches to the legacy HTTP+SSE transport. Plain `http://` is only allowed for localhost.
- `${VAR}` / `${VAR:-default}` in `command`, `args`, `env`, `url` and `headers` is filled in from the environment.
- `"enabled": false` or `"disabled": true` skips a server. `--no-mcp` / `AGENTILOOP_NO_MCP=1` turns MCP off.
- MCP tools ask for permission like other mutating tools, unless the server marks them `readOnlyHint`. `/mcp` lists servers, tools and connection errors.

## Settings

`~/.agentiloop/settings.json` (override dir with `AGENTILOOP_HOME`) remembers the last model per provider and the last launch options (see [Remembered launch](#remembered-launch-set-it-once-then-just-run-agentiloop)):

```json
{ "models": { "anthropic": "claude-opus-5", "openai": "qwen3:4b", "omlx": "Qwen3-Coder-Next-8bit" },
  "last":   { "provider": "omlx", "tui": true, "max_turns": 50, "compact_at": 150000 } }
```

Precedence: `--model` / `AGENTILOOP_MODEL` → resumed session's model → settings.json → provider default (`claude-sonnet-5` / `gpt-4o-mini` / first model listed by oMLX).

## Sessions

Every turn is saved to `~/.agentiloop/sessions/<id>.json`. Resume with `-c` / `--continue` (latest session for this cwd) or `-r <id>` / `--resume <id>`; inside the REPL use `/sessions` and `/resume <id|n>`. `/clear` starts a new session.

## Context compaction

When a request reaches `--compact-at` input tokens (default 150 000, `AGENTILOOP_COMPACT_AT`, 0 disables) the history is summarized by the model and replaced with that summary — before the next prompt, or mid-task after tool results. `/compact` does it on demand.

## Tests

```sh
cargo test --workspace
```

No network needed: the agent loop runs against a scripted mock provider, the SSE parsers against a local canned server, and the tools against temp dirs.

The MCP tests run the bundled example server (`crates/agentiloop-mcp/examples/mcp-example-server.rs`) over all three transports: stdio, Streamable HTTP and legacy HTTP+SSE. To try MCP by hand, run it yourself:

```sh
cargo run -p agentiloop-mcp --example mcp-example-server -- --http 8791   # or --sse 8792 / --stdio
```

`tests/live_servers.rs` also calls AgentMCP's HelloWorld and DemoHttp servers when they're installed in `~/bin`, and skips them otherwise.

## Roadmap

- [x] Streaming (SSE) responses
- [x] OpenAI-compatible provider
- [x] Context compaction when nearing the window limit
- [x] Session persistence / resume
- [x] TUI (ratatui)
- [x] MCP client
- [ ] What's NeXT?

## License

[PolyForm Noncommercial 1.0.0](LICENSE). You may use, modify, and share this software for noncommercial and personal purposes. Commercial use — including building or distributing commercial versions — is reserved exclusively to AgentiLoop. Contact AgentiLoop for a commercial license.
