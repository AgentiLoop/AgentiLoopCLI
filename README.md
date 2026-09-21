# AgentiLoop CLI

A cross-platform (macOS / Linux / Windows) agentic coding loop in the spirit of Claude Code, written in Rust.

## Layout

```
crates/
  agentiloop-core/      message model, Tool trait + registry, Provider trait, permission gate, the agent loop
  agentiloop-provider/  model backends (Anthropic Messages API today)
  agentiloop-tools/     built-in tools: read_file, write_file, edit_file, list_dir, bash
  agentiloop-cli/       `agentiloop` binary: REPL + one-shot mode, interactive permission prompts
```

## Build & run

```sh
# Either a standard API key (sk-ant-api…) or a Claude Code OAuth token
# (sk-ant-oat01-…, from `claude setup-token`) — auth scheme is auto-detected.
export ANTHROPIC_API_KEY=sk-ant-...
cargo run -- "list the files in this project and summarize the layout"   # one-shot
cargo run                                                                # REPL
cargo run -- --yes -C /path/to/repo "fix the failing test"               # no prompts
```

Env: `AGENTILOOP_MODEL`, `AGENTILOOP_YES`, `ANTHROPIC_BASE_URL`, `RUST_LOG=debug` for token usage.

## Roadmap

- [ ] Streaming (SSE) responses
- [ ] OpenAI-compatible provider
- [ ] Context compaction when nearing the window limit
- [ ] Session persistence / resume
- [ ] TUI (ratatui)
- [ ] MCP client
