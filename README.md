# AgentiLoopCLI

### We put out a pre-release v0.0.1 build for Mac, Windows and Linux! 

---

**Take it for a spin!** Read the README and see how long it takes you to get AgentiLoop up and running. If you hit any issues, let us know. We'd love your feedback.

**Bonus:** A Go version is available too: **AgentiLoopGo** → https://github.com/AgentiLoop/AgentiLoopGo

Try both and tell us which one does it better: **Rust or Go?** 🦀 vs 🐹

---

AgentiLoop is an AI coding agent that runs in your terminal, in the spirit of Claude Code. You describe what you want in plain language. The agent reads your files, edits code and runs commands to get it done, and it asks your permission before it changes anything.

It's written in Rust and runs on macOS, Linux and Windows. It works with Claude (Anthropic), OpenAI, local models through Ollama or LM Studio, and oMLX on Apple Silicon.

Created with AgentiLoop Agent! This is our baby. Prebuilt binaries for macOS, Linux and Windows are on the [Releases](https://github.com/AgentiLoop/AgentiLoopCLI/releases) page, or compile it from source with Rust.

<img width="2048" height="1152" alt="AgentiLoop Coding in action" src="https://github.com/user-attachments/assets/d910bbd2-b47d-4c4a-89af-ac0753ccf279" />

---

## Quick start

Three steps: install it, give it a model, run it.

### Step 1: Install

**Download:** grab the archive for your platform from [Releases](https://github.com/AgentiLoop/AgentiLoopCLI/releases), unpack it and put `agentiloop` (`agentiloop.exe` on Windows) on your PATH.

**Or build it:** if you don't have Rust yet, install it from [rustup.rs](https://rustup.rs). Then:

```sh
git clone https://github.com/AgentiLoop/AgentiLoopCLI.git
cd AgentiLoopCLI
cargo install --path crates/agentiloop-cli
```

This builds the program and puts an `agentiloop` command on your PATH, in `~/.cargo/bin`.

> **Not installing?** Everything in this README also works from inside the repo folder. Wherever you see `agentiloop <options>`, type `cargo run -- <options>` instead. Everything after the `--` goes to AgentiLoop.

### Step 2: Connect a model

AgentiLoop needs a model to talk to. Pick one of these:

| I want to use… | Do this |
|---|---|
| **Claude** (Anthropic) | `export ANTHROPIC_API_KEY=sk-ant-...` |
| **OpenAI** | `export OPENAI_API_KEY=sk-...` |
| **Ollama, LM Studio**, or any OpenAI-compatible server | `export OPENAI_BASE_URL=http://localhost:11434/v1` (use your server's address; no key needed for local servers) |
| **oMLX** (local models on Apple Silicon) | Usually nothing. Start oMLX, then run AgentiLoop with `-p omlx` (see below) |

For Claude you can use a normal API key (`sk-ant-api…`) or a Claude Code token (`sk-ant-oat01-…`, which you get from `claude setup-token`). AgentiLoop detects which kind it is.

**oMLX details.** When oMLX runs on the same Mac, AgentiLoop reads the server port and API key from oMLX's own settings file (`~/.omlx/settings.json`), so you don't need to export anything. If oMLX runs on another machine, or you want to override those settings, export them yourself:

```sh
export OMLX_BASE_URL=http://192.168.1.50:7777/v1   # the oMLX server's address (or OMLX_PORT=7777 for localhost)
export OMLX_API_KEY=...                            # the API key from oMLX's settings
```

If oMLX has API key verification turned off, no key is needed.

An `export` only lasts for the terminal tab you typed it in. To make it permanent, add the line to your shell profile (`~/.zshrc` on macOS). On a Mac you can keep the key in the Keychain rather than in the file:

```sh
# one time: store the key in your Keychain
security add-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w "sk-ant-..."

# in ~/.zshrc: load it for every new terminal
export ANTHROPIC_API_KEY="$(security find-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w 2>/dev/null)"
```

### Step 3: Run it

Go to the project you want to work on and start AgentiLoop:

```sh
cd ~/my-project
agentiloop --tui
```

`--tui` opens the full-screen interface, which we recommend. Type what you want, e.g. *"find where the config file is loaded and add a --verbose flag"*, and press Enter.

You'll see the agent's replies, each tool it uses (🔧) and each result (✓ or ✖). The box at the bottom shows what it's doing right now, for example ` ✻ Thinking...  12s `. Before it writes a file or runs a command, it asks you:

- **y**: yes, this time
- **n**: no
- **a**: always allow this tool for the rest of the session
- **Esc**: skip this step, but keep going

---

## Three ways to use it

| Mode | Command | Good for |
|---|---|---|
| **TUI** (full screen) | `agentiloop --tui` | Everyday use: scrolling history, live status, clickable links |
| **Chat** (line by line) | `agentiloop` | Simple terminals, or if you prefer plain text |
| **One-shot** | `agentiloop "explain this project"` | One question: it answers, then exits. Handy in scripts |

Keys in the TUI: **Enter** sends · **↑ / ↓** go through earlier prompts · **PgUp / PgDn** or the mouse wheel scrolls · **Ctrl-U** clears the line · **Ctrl-C** quits.

---

## It remembers your setup

You only type your options once. AgentiLoop saves how you launched it, so next time a plain `agentiloop` starts the same way:

```sh
agentiloop -p anthropic --tui    # first time: choose provider and TUI
agentiloop                       # from now on: same provider, same model, TUI, and your last conversation
```

What it remembers:

- **Provider** (`-p`) and **TUI on/off** (`--tui` / `--no-tui`)
- **Model**: the last one you used, separately for each provider. Switching back to a provider brings back its model.
- **Limits**: `--max-turns` and `--compact-at`
- **Your conversation**: it picks up the last conversation in the current folder, if that conversation used the same provider. The earlier messages are shown again on screen, so you can scroll back and see where you left off

To change something, pass the new option. It applies right away and is remembered from then on:

```sh
agentiloop -p omlx        # switch to oMLX (its last-used model comes back too)
agentiloop -m <model>     # switch model
agentiloop --no-tui       # back to the line-by-line chat
agentiloop --new          # start a fresh conversation (the old one stays saved)
```

Some things are **never** remembered on purpose:

- `--yes`: skipping permission prompts has to be a deliberate choice every time
- `--no-mcp`, `-C` and one-shot prompts
- API keys: those stay in your shell profile

To forget everything, delete `~/.agentiloop/settings.json`.

---

## All options

Every option can also be set with an environment variable, shown in the second column. An option you type always beats a remembered value.

| Option | Env variable | What it does |
|---|---|---|
| `-p, --provider <name>` | `AGENTILOOP_PROVIDER` | `anthropic`, `openai` or `omlx`. If you don't set one, AgentiLoop uses the last one, or detects it from your keys (Anthropic first, then OpenAI, then oMLX) |
| `-m, --model <id>` | `AGENTILOOP_MODEL` | Which model to use |
| `--tui` / `--no-tui` | `AGENTILOOP_TUI` | Full-screen interface on / off |
| `--new` | | Start a new conversation instead of continuing |
| `-c, --continue` | | Continue the last conversation here (already the default) |
| `-r, --resume <id>` | | Reopen a specific conversation (find ids with `/sessions`) |
| `-C, --cwd <folder>` | | Work in a different folder than the one you're in |
| `--yes` | `AGENTILOOP_YES` | Don't ask before running tools. ⚠️ Only for trusted, automated use |
| `--no-mcp` | `AGENTILOOP_NO_MCP` | Don't start MCP servers (see below) |
| `--max-turns <n>` | | Max steps the agent may take per request (default 50) |
| `--compact-at <tokens>` | `AGENTILOOP_COMPACT_AT` | When to summarize a long conversation (default 150000, `0` = never) |
| `-h` / `-V` | | Help / version |

Some examples:

```sh
agentiloop -p openai -m gpt-4o-mini "summarize this repo"    # one question with a specific model
agentiloop -C ../other-repo --tui                            # work on another project
agentiloop --yes "run the tests and fix any failures"        # unattended, no prompts
```

**Which model is used?** The first one that applies wins:

1. `-m` on the command line
2. the model of the conversation you're continuing
3. the last model you used with this provider
4. the provider's default: `claude-sonnet-5` for Anthropic, `gpt-4o-mini` for OpenAI, or the first model oMLX offers

---

## Commands inside the chat

Type these at the prompt, in the TUI or the chat:

| Command | What it does |
|---|---|
| `/model` | Show available models. `/model 3` or `/model <id>` switches (and is remembered) |
| `/sessions` | List your saved conversations, newest first |
| `/resume <n or id>` | Reopen one of them |
| `/clear` | Clear the conversation and start a new one |
| `/compact` | Summarize the conversation now to free up space |
| `/mcp` | Show connected MCP servers and their tools |
| `/help` | List these commands |
| `/exit` | Quit |

---

## Long conversations

Models can only read so much at once. When a conversation gets big (by default, when a request reaches 150,000 tokens), AgentiLoop asks the model to summarize it so far and carries on from the summary. You'll see a 📦 note when that happens. `/compact` does it on demand, and `--compact-at 0` turns it off.

---

## Adding tools with MCP (optional)

[MCP](https://modelcontextprotocol.io) servers give the agent extra tools, like database access, web search or your own scripts. List them in a JSON file:

- `~/.agentiloop/mcp.json`: available in every project
- `.mcp.json` in a project folder: only in that project. If a name appears in both files, this one wins.

The format is the same one Claude Code, Claude Desktop and Agent! use, so you can copy existing configs:

```json
{ "mcpServers": {
    "Local":  { "command": "my-mcp-server", "args": ["--flag"], "env": { "API_KEY": "${MY_KEY}" } },
    "Remote": { "url": "https://example.com/mcp", "headers": { "Authorization": "Bearer ${TOKEN}" } }
} }
```

How it works:

- **Two kinds of server.** A server with a `command` is a local program that AgentiLoop starts for you. A server with a `url` is reached over HTTP. Newer "Streamable HTTP" and older "SSE" servers both work; a URL ending in `/sse` (or `"transport": "sse"`) selects the older style.
- **Tool names.** Each server tool shows up for the agent as `mcp_<server>_<tool>`, e.g. `mcp_Local_search`.
- **Secrets.** `${VAR}` (or `${VAR:-default}`) is filled in from your environment, so keys don't need to be in the file.
- **Permissions.** MCP tools ask permission like any other tool, unless the server marks a tool as read-only.
- **Turning servers off.** Add `"disabled": true` to skip one server, or run with `--no-mcp` to skip them all.
- **Safety.** Plain `http://` is only allowed for localhost; remote servers need `https://`.

Type `/mcp` to see which servers connected, their tools, and any errors.

---

## Where things are saved

Everything lives in `~/.agentiloop/`. Set `AGENTILOOP_HOME` to use a different folder, e.g. a separate test profile.

| File | What's in it |
|---|---|
| `settings.json` | Remembered provider, models and options. Delete it to reset |
| `sessions/` | Your conversations, one file each |
| `mcp.json` | Your MCP servers |
| `history.txt` | Prompts you've typed (for ↑ / ↓) |

---

## Other environment variables

You'll rarely need these:

| Variable | Use |
|---|---|
| `ANTHROPIC_BASE_URL` | Send Anthropic requests to a proxy or compatible server |
| `ANTHROPIC_OAUTH_TOKEN` | Alternative to `ANTHROPIC_API_KEY` for a Claude Code token |
| `OMLX_BASE_URL`, `OMLX_PORT`, `OMLX_API_KEY` | oMLX server address and key. They override `~/.omlx/settings.json`, which is read by default (port 8000 if neither is set) |
| `RUST_LOG=debug` | Show debug logs, including token usage per request |

---

## For developers

### Build and test

```sh
cargo build --release     # → target/release/agentiloop
cargo test --workspace    # runs offline, no API keys needed
```

The tests don't touch the network. The agent loop runs against a scripted fake model, and the streaming parsers against a local test server. The MCP client is tested against a bundled example server over all three connection types (stdio, HTTP, SSE). You can run that server yourself to try MCP by hand:

```sh
cargo run -p agentiloop-mcp --example mcp-example-server -- --http 8791   # or --sse 8792, or --stdio
```

### How the code is organized

The project is split into five crates, and each one builds on the ones before it:

| Crate | What's in it |
|---|---|
| `agentiloop-core` | The heart: the agent loop, messages, the tool and provider interfaces, permissions, sessions, summarizing |
| `agentiloop-provider` | Talks to the models: Anthropic, OpenAI-compatible servers, oMLX |
| `agentiloop-tools` | Built-in tools: `read_file`, `write_file`, `edit_file`, `list_dir`, `bash` |
| `agentiloop-mcp` | The MCP client, ported from Agent!'s Swift AgentMCP |
| `agentiloop-cli` | The `agentiloop` program: options, chat, TUI, settings |

### Dependencies

We keep dependencies small: 18 external crates, and each crate lists only the ones it actually uses.

| Crates | Used for |
|---|---|
| `tokio`, `futures`, `async-trait` | Running things concurrently |
| `serde`, `serde_json` | Reading and writing JSON |
| `reqwest` | HTTP requests to models and MCP servers |
| `anyhow`, `thiserror` | Error handling |
| `tracing`, `tracing-subscriber` | Logging |
| `dirs` | Finding your home folder |
| `clap` | Command-line options |
| `rustyline` | The line-by-line chat |
| `ratatui`, `unicode-width`, `textwrap` | The TUI |
| `pulldown-cmark`, `syntect` | Markdown and code highlighting |

---

## Roadmap

- [x] Streaming responses
- [x] OpenAI-compatible providers
- [x] Summarizing long conversations
- [x] Saved conversations
- [x] Full-screen TUI
- [x] MCP client
- [ ] What's NeXT?

## License

[PolyForm Noncommercial 1.0.0](LICENSE). You may use, change and share this software for personal and noncommercial purposes. Commercial use, including building or selling commercial versions, is reserved to AgentiLoop. Contact AgentiLoop for a commercial license.
