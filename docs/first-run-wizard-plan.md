# Plan: first-run wizard + `--reset` (Rust CLI and Go CLI)

Applies to both twins: `AgentiLoopCLI` (Rust) and `../AgentiLoopGo` (Go).
Every step is done on both sides so the on-disk format stays shared.

## Where things are today (read from the code)

| | Rust | Go |
|---|---|---|
| Config dir | `crates/agentiloop-cli/src/settings.rs:47-52` — `$AGENTILOOP_HOME` else `~/.agentiloop` | `cmd/agentiloop/settings.go:52-61` — same |
| Files in it | `settings.json`, `history.txt`, `sessions/`, `mcp.json` (`settings.rs:54-71`) | same (`settings.go:71-80`) |
| Credentials | env vars only: `ANTHROPIC_API_KEY`, `ANTHROPIC_OAUTH_TOKEN`, `OPENAI_API_KEY`, `OPENAI_BASE_URL`, `OMLX_BASE_URL`, `OMLX_PORT`, `OMLX_API_KEY` (`agentiloop-provider/src/lib.rs:20-30`) | `provider/provider.go:15-31` |
| Brand-new user today | `main.rs:101` calls `from_env(...)` → error `no provider credentials found: ...` and exits | `main.go:192` → same error |
| How README tells people to set keys | `export ...` lines in `~/.zshrc` / `~/.bashrc` (README.md:100-133) | same |

So "brand new" = no `~/.agentiloop/settings.json` **and** none of the seven credential env vars set.

## Goals

1. A brand-new user who just types `agentiloop` gets a wizard instead of an error.
2. A `--reset` that puts the machine back to brand-new so the wizard can be tested repeatedly.
3. Zero behaviour change for existing users (settings present or env vars set → no wizard).

## Design

### A. Credential storage: `~/.agentiloop/env` (new file)

- Plain `KEY=value` lines, mode `0600`. Loaded at startup **before** provider selection; a variable already in the real environment always wins (never overwrite).
- Why: cross-platform (Windows has no `.zshrc`), the CLI owns it, and `--reset` can delete it with certainty. This is the wizard's default.
- Opt-in extra: the wizard can also append a **marked block** to the shell profile so the key is visible to other tools:
  ```sh
  # >>> agentiloop >>>
  export ANTHROPIC_API_KEY="sk-ant-..."
  # <<< agentiloop <<<
  ```
  Profile picked from `$SHELL`: zsh → `~/.zshrc`, bash → `~/.bashrc` (macOS bash → `~/.bash_profile`), fish → `~/.config/fish/config.fish`, PowerShell → `$PROFILE` (`$env:X = "..."` syntax). Override with `AGENTILOOP_SHELL_PROFILE=/path` (mirrors `AGENTILOOP_HOME`) so tests never touch the real profile.
- macOS Keychain option (README already documents this pattern, README.md:308-315): store with `security add-generic-password -a $USER -s ANTHROPIC_API_KEY -w …` and write the `$(security find-generic-password …)` line inside the marked block.

**Decided:** the wizard's default is `~/.agentiloop/env`; the shell-profile block and Keychain are opt-in extras.

### B. `settings.json` gets a `setup` record (shared format)

```json
"setup": {
  "completed_at": "2026-09-27T12:00:00Z",
  "profile": "/Users/me/.zshrc",          // null if the block was not written
  "keychain": ["ANTHROPIC_API_KEY"]      // entries the wizard created, else []
}
```
`--reset` reads this **before** wiping so it can undo exactly what the wizard wrote and nothing else.

### C. Wizard trigger

Runs when **all** hold: interactive launch (no prompt args), stdin+stdout are a TTY, no `-p/--provider` flag, and after loading `~/.agentiloop/env` `from_env(None)` would still fail (no credentials). Also forced by `agentiloop --setup` at any time.
Non-TTY / one-shot launches keep today's error, with one added sentence: `run \`agentiloop --setup\` to configure a provider`.

Hook point: Rust `main.rs` just before line 101; Go `main.go` just before line 192.

### D. Wizard flow (line-based, plain stdin/stdout — runs before REPL/TUI is chosen)

1. Welcome line, where files will be saved (`~/.agentiloop`).
2. Provider: `1` Claude (Anthropic) · `2` OpenAI · `3` Ollama / LM Studio / other OpenAI-compatible server · `4` oMLX (local, reads `~/.omlx/settings.json`).
3. Credential, masked input (Rust: `rpassword`; Go: `golang.org/x/term.ReadPassword`):
   - 1/2 → API key. 3 → base URL (default `http://localhost:11434/v1`), key optional. 4 → nothing unless `~/.omlx/settings.json` is missing, then base URL.
4. Verify: set the vars in the process env, build the provider, call `list_models()` / `ListModels()`. Failure → show the error, offer retry or go back.
5. Model: list what step 4 returned, default = provider default (or first served for oMLX). Stored via existing `set_model` / `SetModel`.
6. Save credential: `[1] ~/.agentiloop/env (recommended)  [2] also add to <profile>  [3] macOS Keychain + profile line` (3 only on macOS).
7. Optional: if the running binary's directory is not on `PATH`, offer to add `export PATH="<dir>:$PATH"` to the same marked block (README step 3, README.md:121).
8. Write `settings.json` (`last.provider`, `models[..]`, `setup`), print a summary, continue straight into the normal launch (no restart needed since the vars are already in the process env).

`Ctrl-C` / EOF at any prompt → exit 1, nothing written.

### E. `agentiloop --reset`

1. Load `settings.json` (for `setup`), then list everything that will be touched:
   - the whole `$AGENTILOOP_HOME` dir (`settings.json`, `history.txt`, `sessions/`, `mcp.json`, `env`) — warn that `mcp.json` is user-written;
   - the marked block in `setup.profile` (only lines between the markers);
   - Keychain items in `setup.keychain`;
   - any **unmarked** lines in the common profiles that set one of the seven credential vars or the `.local/bin` PATH line — shown with file:line. These were written by hand (or from the README), so they are not deleted: reset asks `y/N` and *comments them out* with a `# agentiloop-reset: ` prefix (reversible).
2. Confirm: type `reset` (skipped with `--yes`).
3. Do it, then print the one thing the CLI cannot do — clear the current shell:
   ```
   done. Variables already exported in this terminal stay until you open a new one, or run:
     unset ANTHROPIC_API_KEY ANTHROPIC_OAUTH_TOKEN OPENAI_API_KEY OPENAI_BASE_URL OMLX_BASE_URL OMLX_PORT OMLX_API_KEY
   then run `agentiloop` to see the first-run wizard.
   ```
4. Exit 0. `--reset` conflicts with prompt args and with `--setup`.

Both are **flags**, not subcommands: today `agentiloop reset` would be sent to the model as a prompt (`main.rs:75`, `main.go:101`), and adding clap subcommands would change that.

### F. Testing the wizard from a clean slate (no real files touched)

```sh
env -u ANTHROPIC_API_KEY -u ANTHROPIC_OAUTH_TOKEN -u OPENAI_API_KEY -u OPENAI_BASE_URL \
    -u OMLX_BASE_URL -u OMLX_PORT -u OMLX_API_KEY \
    AGENTILOOP_HOME="$(mktemp -d)" AGENTILOOP_SHELL_PROFILE="$(mktemp)" agentiloop
```
Unit tests drive the wizard with scripted stdin (`Cursor` / `strings.Reader`) and a fake provider; reset tests write a temp profile containing a marked block + one hand-written `export` line and assert only the right lines change. Go already has the `AGENTILOOP_HOME` test pattern in `cmd/agentiloop/main_test.go:69`.

## Files

| Step | Rust | Go |
|---|---|---|
| 1 settings | `settings.rs`: `env_path()`, `Setup` struct + field, `load_env_file()` | `settings.go`: `envPath()`, `Setup` type + field, `loadEnvFile()` |
| 2 reset | new `reset.rs`; `main.rs`: `--reset`, `--yes` reuse | new `reset.go`; `main.go`: `--reset` |
| 3 wizard | new `wizard.rs` (prompts, profile block, keychain); `main.rs`: `--setup` + trigger; `Cargo.toml`: `rpassword` | new `wizard.go`; `main.go`: `--setup` + trigger; `go.mod`: `golang.org/x/term` |
| 4 error hint | `agentiloop-provider/src/lib.rs:28` | `provider/provider.go:31` |
| 5 docs | `README.md` Quick start: "run `agentiloop` and follow the wizard"; `--setup` / `--reset` in the options table; then the 7 translated READMEs | same |

Order: 1 → 2 → 3 → 4 → 5, Rust and Go in lockstep per step (reset first so the wizard can be re-tested while it is being built). Build + tests green + commit after each step on each side.

## Out of scope (mention only)

- Migrating an existing user's hand-written `export` into `~/.agentiloop/env`.
- Multiple saved providers / switching keys inside the wizard (existing `-p` and `/model` cover this).
