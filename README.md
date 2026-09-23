# AgentiLoop CLI

A cross-platform (macOS / Linux / Windows) agentic coding loop in the spirit of Claude Code, written in Rust. Created by AgentiLoop Agent!

## Layout

```
crates/
  agentiloop-core/      message model, Tool trait + registry, Provider trait, permission gate, the agent loop
  agentiloop-provider/  model backends: Anthropic Messages API, OpenAI-compatible Chat Completions, oMLX
  agentiloop-tools/     built-in tools: read_file, write_file, edit_file, list_dir, bash
  agentiloop-mcp/       MCP client (stdio, Streamable HTTP, legacy HTTP+SSE), ported from Agent!'s AgentMCP
  agentiloop-cli/       `agentiloop` binary: REPL + one-shot mode, interactive permission prompts
```

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

## TUI

`agentiloop --tui` (or `AGENTILOOP_TUI=1`) opens a full-screen ratatui interface: scrolling transcript, prompt box, status bar. Permission prompts appear as a modal (`y` / `n` / `a`lways). Keys: Enter send, ↑/↓ prompt history, PgUp/PgDn scroll, Ctrl-U clear line, Ctrl-C quit. All slash commands work; `/model` with no argument lists models — pick with `/model <n|id>`.

Env: `AGENTILOOP_PROVIDER`, `AGENTILOOP_MODEL`, `AGENTILOOP_YES`, `AGENTILOOP_TUI`, `AGENTILOOP_HOME`, `AGENTILOOP_COMPACT_AT`, `ANTHROPIC_BASE_URL`, `OPENAI_BASE_URL`, `OMLX_BASE_URL`, `OMLX_PORT`, `OMLX_API_KEY`, `RUST_LOG=debug` for token usage.

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

`~/.agentiloop/settings.json` (override dir with `AGENTILOOP_HOME`) remembers the last `/model` pick per provider:

```json
{ "models": { "anthropic": "claude-opus-5", "openai": "qwen3:4b", "omlx": "Qwen3-Coder-Next-8bit" } }
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
