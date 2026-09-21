# AgentiLoop CLI

A cross-platform (macOS / Linux / Windows) agentic coding loop in the spirit of Claude Code, written in Rust.

## Layout

```
crates/
  agentiloop-core/      message model, Tool trait + registry, Provider trait, permission gate, the agent loop
  agentiloop-provider/  model backends: Anthropic Messages API, OpenAI-compatible Chat Completions
  agentiloop-tools/     built-in tools: read_file, write_file, edit_file, list_dir, bash
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
```

Provider is auto-detected from which credentials are set (`ANTHROPIC_API_KEY` wins); force one with `--provider` / `AGENTILOOP_PROVIDER`.

Env: `AGENTILOOP_PROVIDER`, `AGENTILOOP_MODEL`, `AGENTILOOP_YES`, `AGENTILOOP_HOME`, `ANTHROPIC_BASE_URL`, `OPENAI_BASE_URL`, `RUST_LOG=debug` for token usage.

## Settings

`~/.agentiloop/settings.json` (override dir with `AGENTILOOP_HOME`) remembers the last `/model` pick per provider:

```json
{ "models": { "anthropic": "claude-opus-5", "openai": "qwen3:4b" } }
```

Precedence: `--model` / `AGENTILOOP_MODEL` → settings.json → provider default (`claude-sonnet-5` / `gpt-4o-mini`).

## Roadmap

- [x] Streaming (SSE) responses
- [x] OpenAI-compatible provider
- [ ] Context compaction when nearing the window limit
- [ ] Session persistence / resume
- [ ] TUI (ratatui)
- [ ] MCP client

## License

[PolyForm Noncommercial 1.0.0](LICENSE). You may use, modify, and share this software for noncommercial and personal purposes. Commercial use — including building or distributing commercial versions — is reserved exclusively to AgentiLoop. Contact AgentiLoop for a commercial license.
