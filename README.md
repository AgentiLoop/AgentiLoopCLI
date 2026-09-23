# AgentiLoop CLI

An AI coding agent for your terminal, written in Rust, in the spirit of Claude Code. It runs on macOS, Linux and Windows. Created with AgentiLoop Agent! This is our baby.

There are no binaries yet, so you compile from source (Rust / Cargo required).

<img width="2048" height="1152" alt="AgentiLoop Coding in action" src="https://github.com/user-attachments/assets/d910bbd2-b47d-4c4a-89af-ac0753ccf279" />

## 1. Install

Get Rust from [rustup.rs](https://rustup.rs), then:

```sh
git clone https://github.com/AgentiLoop/AgentiLoopCLI.git
cd AgentiLoopCLI
cargo install --path crates/agentiloop-cli    # puts `agentiloop` on your PATH
```

If you'd rather not install it, use `cargo run -- <options>` from the repo instead of `agentiloop <options>`.

## 2. Pick a model provider

Set one of these (put it in `~/.zshrc` so it sticks):

| Provider | What to set |
|---|---|
| Anthropic (Claude) | `export ANTHROPIC_API_KEY=sk-ant-...` (an API key or a `claude setup-token` token both work) |
| OpenAI | `export OPENAI_API_KEY=sk-...` |
| Ollama / LM Studio / any OpenAI-compatible server | `export OPENAI_BASE_URL=http://localhost:11434/v1` |
| oMLX (local, Apple Silicon) | nothing, just run with `-p omlx` |

## 3. Run it

```sh
agentiloop --tui                        # full-screen UI (recommended)
agentiloop                              # simple line-by-line chat
agentiloop "explain this project"       # one question, then exit
```

**It remembers your setup.** Launch it once with your options, e.g. `agentiloop -p anthropic --tui`. After that, plain `agentiloop` starts with the same provider, model and UI, and continues your last conversation in that folder.

- `--new` starts a fresh conversation
- `--no-tui` switches back to the simple chat
- `-p` / `-m` switch provider / model

## Options

| Option | What it does |
|---|---|
| `-p <provider>` | `anthropic`, `openai` or `omlx` |
| `-m <model>` | pick a model |
| `--tui` / `--no-tui` | full-screen UI on / off |
| `--new` | start a new conversation |
| `-r <id>` | reopen an old conversation |
| `-C <dir>` | work in another folder |
| `--yes` | don't ask before running tools (careful! never remembered) |
| `--no-mcp` | don't start MCP servers |
| `--max-turns <n>` | max steps per request (default 50) |
| `--compact-at <n>` | summarize the chat at this many tokens (default 150000, 0 = never) |

`agentiloop --help` lists everything, including the matching env vars.

## Commands inside the chat

| Command | What it does |
|---|---|
| `/model` | list models, `/model <n>` picks one |
| `/sessions` | list saved conversations |
| `/resume <n>` | reopen one |
| `/clear` | start fresh |
| `/compact` | summarize the chat to save space |
| `/mcp` | list MCP servers and tools |
| `/exit` | quit |

TUI keys: **Enter** send · **↑/↓** history · **PgUp/PgDn** scroll · **y/n/a** answer permission prompts · **Ctrl-C** quit.

## MCP servers (optional)

Add tools from MCP servers by listing them in `~/.agentiloop/mcp.json`, or in `.mcp.json` inside a project. It's the same format Claude Code uses:

```json
{ "mcpServers": {
    "Local":  { "command": "my-mcp-server", "args": [] },
    "Remote": { "url": "https://example.com/mcp", "headers": { "Authorization": "Bearer ${TOKEN}" } }
} }
```

stdio, HTTP and older SSE servers all work. Type `/mcp` to see what's connected.

## Where things are saved

Everything lives in `~/.agentiloop/`:

- `settings.json`: remembered provider, model and UI (delete it to reset)
- `sessions/`: your conversations
- `mcp.json`: MCP servers
- `history.txt`: prompt history

## For developers

```sh
cargo build --release     # → target/release/agentiloop
cargo test --workspace    # no network needed
```

| Crate | What's in it |
|---|---|
| `agentiloop-core` | the agent loop, tools, permissions, sessions |
| `agentiloop-provider` | Anthropic, OpenAI-compatible, oMLX |
| `agentiloop-tools` | read_file, write_file, edit_file, list_dir, bash |
| `agentiloop-mcp` | MCP client (ported from Agent!'s AgentMCP) |
| `agentiloop-cli` | the `agentiloop` app: chat, TUI, options |

To test MCP by hand, run the example server: `cargo run -p agentiloop-mcp --example mcp-example-server -- --http 8791`

## Roadmap

- [x] Streaming responses
- [x] OpenAI-compatible providers
- [x] Auto-summarize long chats
- [x] Saved sessions
- [x] Full-screen TUI
- [x] MCP client
- [ ] What's NeXT?

## License

[PolyForm Noncommercial 1.0.0](LICENSE). Free for personal and noncommercial use. Commercial use is reserved to AgentiLoop, so contact us for a commercial license.
