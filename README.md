<a href="https://fluxionai.world/register?source=github&campaign=aiagent&promo=AIAGENT"><img src="docs/sponsors/fluxion-ai-silver-ad.svg" width="900" alt="Fluxion AI, Silver Sponsor: one unified API for GPT, Claude and other leading AI models. Save up to 70% compared with official API pricing and get $3 in API credits." /></a>

# AgentiLoopCLI

🌐 [English](README.md) · [Español](README_es.md) · [Français](README_fr.md) · [Deutsch](README_de.md) · [中文 (简体)](README_zh.md) · [Русский](README_ru.md) · [한국어](README_ko.md) · [日本語](README_ja.md)

### 🎉 We just shipped release v0.0.2 for Mac, Windows and Linux!

---

**Take it for a spin!** Read the README and see how long it takes you to get AgentiLoop up and running. If you hit any issues, let us know. We'd love your feedback.

**Bonus:** A Go version is available too: **AgentiLoopGo** → https://github.com/AgentiLoop/AgentiLoopGo

Try both and tell us which one does it better: **Rust or Go?** 🦀 vs 🐹

---

### 💖 Sponsor AgentiLoop

Like what you see? Help keep AgentiLoop fast and cross-platform. Sponsor us on **[GitHub Sponsors → AgentiLoop](https://github.com/sponsors/AgentiLoop)**. Tiers and perks are in the [Sponsorship guide](https://github.com/AgentiLoop/Agent/blob/main/docs/SPONSORSHIP.md).

[![Sponsor AgentiLoop](https://img.shields.io/badge/Sponsor-AgentiLoop-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white)](https://github.com/sponsors/AgentiLoop)

---

AgentiLoop is an AI coding agent that runs in your terminal, in the spirit of Claude Code. You describe what you want in plain language. The agent reads your files, edits code and runs commands to get it done, and it asks your permission before it changes anything.

It's written in Rust and runs on macOS, Linux and Windows. It works with Claude (Anthropic), OpenAI, local models through Ollama or LM Studio, and oMLX on Apple Silicon. There is nothing to configure by hand: the first time you run it, a built-in setup wizard asks which provider you want, takes your key, checks that it works and saves it. About a minute, no config files, no `export` lines.

Prebuilt binaries are on the [Releases](https://github.com/AgentiLoop/AgentiLoopCLI/releases) page; building from source is covered under [For developers](#for-developers). Created with AgentiLoop Agent! This is our baby.

<img src="docs/pong.png" width="900" alt="AgentiLoop writing, building and launching an Atari-style Pong game in SwiftUI from a single prompt, with the live diff and the running game window" />

---

## 🚀 Get started in 5 minutes

No Rust, no compiling. You download one file, give it an API key and start chatting. Follow the steps in order.

### 1. Download AgentiLoop

First find out which file you need:

| Your computer | File to download |
|---|---|
| Mac with Apple Silicon (M1, M2, M3, M4…) | `agentiloop-macos-arm64.tar.gz` |
| Mac with an Intel chip | `agentiloop-macos-x86_64.tar.gz` |
| Linux, 64-bit PC | `agentiloop-linux-x86_64.tar.gz` |
| Linux on ARM (Raspberry Pi 4/5, ARM servers) | `agentiloop-linux-arm64.tar.gz` |
| Windows 10/11 | `agentiloop-windows-x86_64.zip` |

Not sure? On Mac or Linux, run `uname -m`. `arm64` or `aarch64` means **arm64**, and `x86_64` means **x86_64**.

**macOS and Linux.** Open Terminal and paste these lines. This example uses the Apple Silicon file, so change `macos-arm64` in the first three lines if yours is different:

```sh
curl -LO https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.2/agentiloop-macos-arm64.tar.gz
tar xzf agentiloop-macos-arm64.tar.gz
mkdir -p ~/.local/bin && mv agentiloop-macos-arm64/agentiloop ~/.local/bin/
```

That puts the program in `~/.local/bin`, a folder in your home directory. The setup wizard in step 3 offers to tell your terminal to look there.

**Windows.** Open **PowerShell** (Start menu → type "PowerShell") and paste:

```powershell
Invoke-WebRequest https://github.com/AgentiLoop/AgentiLoopCLI/releases/download/v0.0.2/agentiloop-windows-x86_64.zip -OutFile agentiloop.zip
Expand-Archive agentiloop.zip -DestinationPath $HOME\agentiloop -Force
$p = [Environment]::GetEnvironmentVariable("Path", "User")
[Environment]::SetEnvironmentVariable("Path", "$p;$HOME\agentiloop\agentiloop-windows-x86_64", "User")
```

The last two lines add AgentiLoop to your PATH. **Close PowerShell and open a new window** so it picks up the change.

### 2. Get an API key

AgentiLoop is the agent. The "brain" is an AI model that you connect it to. Pick **one**:

| Option | Where to get it | Cost |
|---|---|---|
| **Claude** (recommended) | [console.anthropic.com](https://console.anthropic.com/settings/keys) → *Create Key*. It starts with `sk-ant-` | Pay per use |
| **OpenAI** | [platform.openai.com/api-keys](https://platform.openai.com/api-keys). It starts with `sk-` | Pay per use |
| **Ollama** (runs on your own computer) | Install from [ollama.com](https://ollama.com), then run `ollama pull qwen2.5-coder` | Free, no key |

Copy the key somewhere safe. You'll paste it in the next step.

### 3. Run the setup wizard

Because `~/.local/bin` isn't on your PATH yet, start it with its full path this one time (on Windows step 1 already fixed the PATH, so just type `agentiloop`):

```sh
~/.local/bin/agentiloop
```

When no key is saved yet, the wizard starts on its own and asks five things:

1. **Which provider** — Claude, OpenAI, a local OpenAI-compatible server (Ollama, LM Studio, …) or oMLX. Type a number.
2. **Your API key** — typed hidden, nothing shows on screen. Local servers usually need none; for oMLX on the same Mac the wizard reads the key from oMLX's own settings, so it doesn't even ask.
3. **Connection check** — the wizard talks to the provider right away. If the key is wrong it tells you and offers to try again; nothing is saved until it works.
4. **Which model** — pick one from the list the provider returned, or press Enter for the default. You can change it any time later with `/model`.
5. **Where to keep the key** — press Enter for the default, `~/.agentiloop/env`, a private file only AgentiLoop reads. The other choices, for people who also want the key in their shell or in the macOS Keychain, are explained under *Advanced* below.

Last, the wizard notices that `~/.local/bin` isn't on your PATH and offers to add it. Press **Enter** (yes). Then it says `All set` and drops you at the prompt. Here is a full run choosing Claude and accepting the defaults (your model list will differ):

```text
$ ~/.local/bin/agentiloop
Welcome to AgentiLoop! Let's set things up (about a minute).
Settings are kept in /Users/you/.agentiloop. Run `agentiloop --setup` or `/setup` to redo this, `agentiloop --reset` to start over.

Which model provider do you want to use?
  1  Claude (Anthropic) — API key from console.anthropic.com
  2  OpenAI — API key from platform.openai.com
  3  Ollama, LM Studio or another OpenAI-compatible server (local, usually no key)
  4  oMLX (local Apple Silicon server; reads ~/.omlx/settings.json)
Provider [1-4, default 1]: 1
Anthropic API key (starts with sk-ant-, input hidden):
Checking the connection…
Connected (8 model(s) available).

Pick a model (change it any time with /model):
   1  claude-sonnet-5  (default)
   2  claude-…
   3  claude-…
   …
Model [1-8, an id, or Enter for claude-sonnet-5]:

Where should the credential be saved?
  1  /Users/you/.agentiloop/env (recommended; only agentiloop reads it, file mode 600)
  2  Also add it to /Users/you/.zshrc so other tools in your terminal see it
  3  macOS Keychain, with a line in /Users/you/.zshrc that reads it (nothing stored in plain text)
Save to [1-3, default 1]:
saved to /Users/you/.agentiloop/env

`/Users/you/.local/bin` is not on your PATH, so `agentiloop` only works with its full path.
Add it to PATH in /Users/you/.zshrc? [Y/n]
updated /Users/you/.zshrc (between `# >>> agentiloop >>>` and `# <<< agentiloop <<<`); it applies to new terminals

All set: anthropic / claude-sonnet-5. Type a request at the prompt, /help for commands, /exit to leave.
```

You can start chatting right here, or type `/exit` and carry on with step 4. From now on a plain `agentiloop` starts straight into the prompt.

Good to know:

- For Claude you can paste either a normal API key (`sk-ant-api…`) or a Claude Code token (`sk-ant-oat01-…`, from `claude setup-token`); AgentiLoop detects which kind it is.
- Option 3 (Ollama, LM Studio, …) also asks for the server URL. The default, `http://localhost:11434/v1`, is right for a local Ollama, so just press Enter.
- The wizard runs in whichever interface you use. Start with `--tui` and the same questions appear inside the full-screen interface; when it finishes you are already at the prompt:

<img src="docs/setup-wizard-tui.png" width="900" alt="The setup wizard running inside the full-screen TUI: provider, hidden API key, connection check, model list, where to save the key, then the first prompt" />

Rerun or redo it any time:

```bash
agentiloop --setup          # wizard on the plain terminal
agentiloop --setup --tui    # wizard inside the full-screen TUI (as in the screenshot)
/setup                      # from inside a running session (REPL or TUI)
agentiloop --reset          # forget everything and start from brand new
```

> 🔒 **Keep your key private.** `~/.agentiloop/env` is readable only by you. Don't paste the key into chats or commit it to git. On a Mac, option 3 in the "Where should the credential be saved?" question stores it in the Keychain instead, so it's never on disk in plain text.

<details>
<summary><b>Advanced: setting the key by hand</b> (skip this if the wizard worked for you)</summary>

If you'd rather manage the key yourself, or you are running AgentiLoop in a script or CI where nobody can answer the wizard, set one of these environment variables and AgentiLoop will use it without asking:

| I want to use… | Set this |
|---|---|
| **Claude** (Anthropic) | `export ANTHROPIC_API_KEY=sk-ant-...` |
| **OpenAI** | `export OPENAI_API_KEY=sk-...` |
| **Ollama, LM Studio**, or any OpenAI-compatible server | `export OPENAI_BASE_URL=http://localhost:11434/v1` (your server's address; no key needed for local servers) |
| **oMLX** (local models on Apple Silicon) | Usually nothing. Start oMLX, then run AgentiLoop with `-p omlx` |

**oMLX details.** When oMLX runs on the same Mac, AgentiLoop reads the server port and API key from oMLX's own settings file (`~/.omlx/settings.json`). If oMLX runs on another machine, or you want to override those settings:

```sh
export OMLX_BASE_URL=http://192.168.1.50:7777/v1   # the oMLX server's address (or OMLX_PORT=7777 for localhost)
export OMLX_API_KEY=...                            # the API key from oMLX's settings
```

If oMLX has API key verification turned off, no key is needed.

An `export` only lasts for the terminal tab you typed it in. To make it permanent you'd add the line to your shell profile (`~/.zshrc` on macOS), which is exactly what the wizard's **"Also add it to ~/.zshrc"** choice does for you. Likewise the wizard's **"macOS Keychain"** choice is the hands-free version of this:

```sh
# one time: store the key in your Keychain
security add-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w "sk-ant-..."

# in ~/.zshrc: load it for every new terminal
export ANTHROPIC_API_KEY="$(security find-generic-password -a "$USER" -s ANTHROPIC_API_KEY -w 2>/dev/null)"
```

</details>

### 4. Check that it works

**Open a new terminal window** so it picks up the PATH change from the wizard (Windows: a new PowerShell window). Then:

```sh
agentiloop --version
```

You should see `agentiloop 0.0.2`. Now run it with no options:

```sh
agentiloop
```

It should go straight to the prompt. If the wizard starts again instead, the key didn't get saved: go through step 3 once more.

### 5. Your first session

Start with a new, empty test folder so you can try it safely. For a real project, `cd` into that project's folder instead (for example `cd ~/code/my-app`).

```sh
mkdir -p ~/agentiloop-test
cd ~/agentiloop-test
agentiloop --tui
```

`--tui` opens the full-screen interface, which we recommend. Type what you want in plain English and press **Enter**. Some good first prompts:

```text
explain what this project does
list the files in src and tell me which one is the entry point
find the TODO comments and summarize them
add a --verbose flag to the command-line parser
run the tests and fix anything that fails
create a README.md for this project
```

You'll see the agent's replies, each tool it uses (🔧) and each result (✓ or ✖). The box at the bottom shows what it's doing right now, for example ` ✻ Thinking...  12s `. Before it writes a file or runs a command, it asks you:

- **y**: yes, this time
- **n**: no
- **a**: always allow this tool for the rest of the session
- **Esc**: skip this step, but keep going

**Ctrl-C** quits. Next time, a plain `agentiloop` starts the same way and picks up your last conversation. Using **Ollama**? To switch to another model you've pulled, type `/model` inside the session.

### Stuck? Quick fixes

| You see | Fix |
|---|---|
| `command not found: agentiloop` | `~/.local/bin` isn't on your PATH. Open a new terminal window first; if that doesn't help, run `~/.local/bin/agentiloop --setup` and say yes when it offers to add it to PATH. On Windows, open a new PowerShell window |
| `Error: no provider credentials found` | No key is saved. Run `agentiloop --setup` (step 3), then check with step 4 |
| macOS: *"agentiloop" cannot be opened* / *unidentified developer* | This happens if you downloaded with a browser instead of `curl`. Run `xattr -d com.apple.quarantine ~/.local/bin/agentiloop` |
| Windows: *Windows protected your PC* | Click **More info** → **Run anyway** |
| `401` / `invalid x-api-key` / authentication error | The key is wrong or was pasted with spaces or quotes. Copy it again and run `agentiloop --setup` to enter it afresh |
| Ollama: model not found | Run `ollama list` and pass the exact name with `-m` |
| It keeps using an old model or provider | It remembers your last choices. Pass `-p` / `-m` to change them, or run `agentiloop --reset` to start over |

Still stuck? [Open an issue](https://github.com/AgentiLoop/AgentiLoopCLI/issues) and paste the command and the error. We'll help.

---

## Three ways to use it

| Mode | Command | Good for |
|---|---|---|
| **TUI** (full screen) | `agentiloop --tui` | Everyday use: scrolling history, live status, clickable links |
| **Chat** (line by line) | `agentiloop` | Simple terminals, or if you prefer plain text |
| **One-shot** | `agentiloop "explain this project"` | One question: it answers, then exits. Handy in scripts |

Keys in the TUI: **Enter** sends · **↑ / ↓** go through earlier prompts · **PgUp / PgDn** or the mouse wheel scrolls · **Ctrl-U** clears the line · **Ctrl-C** quits.

---

## What can it do? (tools)

The agent works with five built-in tools. You don't call them yourself. You describe the goal, and the agent picks the tool:

| Tool | What it does | Asks first? |
|---|---|---|
| `read_file` | Reads a file (with line numbers) | No |
| `list_dir` | Lists the files in a folder | No |
| `write_file` | Creates a new file or overwrites one | **Yes** |
| `edit_file` | Changes an exact piece of text in a file | **Yes** |
| `bash` | Runs a shell command, like tests, builds or `git` (`sh -c` on Mac/Linux, `cmd /C` on Windows) | **Yes** |

Want more tools, like web search, databases or GitHub? Add MCP servers; see [Adding tools with MCP](#adding-tools-with-mcp-optional).

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
- API keys: those live in `~/.agentiloop/env` (written by the wizard) or your shell environment, never in `settings.json`

To forget everything, delete `~/.agentiloop/settings.json`, or run `agentiloop --reset` to go back to brand new.

---

## All options

`agentiloop --help` prints this list in the terminal. Every option can also be set with an environment variable, shown in the second column. An option you type always beats a remembered value.

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
| `--setup` | | Run the first-time wizard again (provider, key, model). Combine with `--tui` to run it inside the full-screen interface |
| `--reset` | | Back to brand new: deletes `~/.agentiloop`, the agentiloop block in your shell profile and Keychain items the wizard created (on Windows: the user environment variables it set). Hand-written `export` lines are only commented out, and only if you say yes. Add `--yes` to skip the questions |
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
| `/setup` | Run the setup wizard again (works in the TUI too). A new key or provider takes effect after a restart |
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
| `settings.json` | Remembered provider, models and options, plus what the wizard wrote elsewhere |
| `env` | Your key, written by the wizard (`KEY=value`, file mode 600). Loaded at startup; an `export` in your shell wins over it |
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

### Build from source

If you don't have Rust yet, install it from [rustup.rs](https://rustup.rs). Then:

```sh
git clone https://github.com/AgentiLoop/AgentiLoopCLI.git
cd AgentiLoopCLI
cargo install --path crates/agentiloop-cli
```

This builds the program and puts an `agentiloop` command on your PATH, in `~/.cargo/bin`. The setup wizard runs on first launch exactly as described in [step 3](#3-run-the-setup-wizard).

> **Not installing?** Everything in this README also works from inside the repo folder. Wherever you see `agentiloop <options>`, type `cargo run -- <options>` instead. Everything after the `--` goes to AgentiLoop.

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

---

<a href="https://fluxionai.world/register?source=github&campaign=aiagent&promo=AIAGENT"><img src="docs/sponsors/fluxion-ai-silver-ad.svg" width="900" alt="Fluxion AI, Silver Sponsor: one unified API for GPT, Claude and other leading AI models. Save up to 70% compared with official API pricing and get $3 in API credits." /></a>
