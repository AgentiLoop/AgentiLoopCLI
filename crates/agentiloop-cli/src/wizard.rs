//! First-run wizard (`agentiloop --setup`): pick a provider, enter a key, check
//! the connection, choose a model, and save the credential to
//! `~/.agentiloop/env` (optionally also the shell profile or the macOS Keychain).
//! Runs by itself when there are no credentials and nothing in `~/.agentiloop`.

use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use agentiloop_core::{ModelInfo, Provider};
use anyhow::{Context, Result};

use crate::reset::{BLOCK_END, BLOCK_START, CREDENTIAL_VARS};
use crate::settings;

/// True when nothing in the environment names a provider.
pub fn no_credentials() -> bool {
    CREDENTIAL_VARS.iter().all(|k| std::env::var_os(k).map_or(true, |v| v.is_empty()))
}

/// Run automatically only for a plain interactive launch on a terminal with nothing configured.
pub fn should_run(interactive: bool, provider_flag: bool) -> bool {
    interactive && !provider_flag && no_credentials() && io::stdin().is_terminal() && io::stdout().is_terminal()
}

fn ask(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut line = String::new();
    if io::stdin().lock().read_line(&mut line)? == 0 {
        println!();
        anyhow::bail!("setup cancelled; nothing was saved");
    }
    Ok(line.trim().to_string())
}

fn ask_default(prompt: &str, default: &str) -> Result<String> {
    let a = ask(&format!("{prompt} [{default}]: "))?;
    Ok(if a.is_empty() { default.to_string() } else { a })
}

fn ask_secret(prompt: &str) -> Result<String> {
    // Hidden input needs a terminal; scripted stdin (tests, pipes) falls back to a plain read.
    if !io::stdin().is_terminal() {
        return ask(prompt);
    }
    Ok(rpassword::prompt_password(prompt).context("reading input")?.trim().to_string())
}

fn ask_yes(prompt: &str, default_yes: bool) -> Result<bool> {
    let a = ask(&format!("{prompt} {} ", if default_yes { "[Y/n]" } else { "[y/N]" }))?;
    Ok(match a.to_ascii_lowercase().as_str() {
        "" => default_yes,
        "y" | "yes" => true,
        _ => false,
    })
}

fn choose(prompt: &str, n: usize, default: usize) -> Result<usize> {
    loop {
        let a = ask(&format!("{prompt} [1-{n}, default {default}]: "))?;
        if a.is_empty() {
            return Ok(default);
        }
        match a.parse::<usize>() {
            Ok(i) if (1..=n).contains(&i) => return Ok(i),
            _ => println!("Please enter a number from 1 to {n}."),
        }
    }
}

// ---- shell profile ----

#[derive(Debug, Clone, Copy, PartialEq)]
enum ShellKind {
    Posix,
    Fish,
    PowerShell,
}

/// The profile the wizard may append to: `AGENTILOOP_SHELL_PROFILE`, else derived
/// from `$SHELL`, else (no `$SHELL`, i.e. Windows) the PowerShell profile.
/// `None` on shells we don't know, where only `~/.agentiloop/env` is offered.
fn shell_profile() -> Option<(PathBuf, ShellKind)> {
    let home = dirs::home_dir()?;
    if let Some(p) = std::env::var_os("AGENTILOOP_SHELL_PROFILE") {
        let p = PathBuf::from(p);
        let kind = match p.extension().and_then(|e| e.to_str()) {
            Some("fish") => ShellKind::Fish,
            Some("ps1") => ShellKind::PowerShell,
            _ => ShellKind::Posix,
        };
        return Some((p, kind));
    }
    match std::env::var("SHELL") {
        Ok(sh) => profile_for(&sh, &home, cfg!(target_os = "macos")),
        Err(_) if cfg!(windows) => Some(powershell_profile(&home)),
        Err(_) => None,
    }
}

fn profile_for(shell: &str, home: &Path, macos: bool) -> Option<(PathBuf, ShellKind)> {
    let name = Path::new(shell).file_name()?.to_str()?;
    Some(match name {
        "zsh" => (home.join(".zshrc"), ShellKind::Posix),
        "bash" if macos => (home.join(".bash_profile"), ShellKind::Posix),
        "bash" => (home.join(".bashrc"), ShellKind::Posix),
        "fish" => (home.join(".config/fish/config.fish"), ShellKind::Fish),
        "pwsh" => (home.join(".config/powershell/profile.ps1"), ShellKind::PowerShell),
        _ => return None,
    })
}

/// Windows: the CurrentUserAllHosts profile — PowerShell 7's folder if it exists, else Windows PowerShell 5's.
fn powershell_profile(home: &Path) -> (PathBuf, ShellKind) {
    let ps7 = home.join("Documents").join("PowerShell");
    let dir = if ps7.is_dir() { ps7 } else { home.join("Documents").join("WindowsPowerShell") };
    (dir.join("profile.ps1"), ShellKind::PowerShell)
}

fn export_line(kind: ShellKind, key: &str, value: &str) -> String {
    match kind {
        ShellKind::Posix => format!("export {key}=\"{value}\""),
        ShellKind::Fish => format!("set -gx {key} \"{value}\""),
        ShellKind::PowerShell => format!("$env:{key} = \"{value}\""),
    }
}

fn path_line(kind: ShellKind, dir: &Path) -> String {
    match kind {
        ShellKind::Posix => format!("export PATH=\"{}:$PATH\"", dir.display()),
        ShellKind::Fish => format!("fish_add_path {}", dir.display()),
        ShellKind::PowerShell => format!("$env:PATH = \"{}\" + [IO.Path]::PathSeparator + $env:PATH", dir.display()),
    }
}

/// Replaces (or appends) the marked agentiloop block in `path` with `lines`.
fn write_block(path: &Path, lines: &[String]) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let mut text = crate::reset::remove_block(&existing);
    if !text.is_empty() && !text.ends_with("\n\n") {
        text.push('\n');
    }
    text.push_str(BLOCK_START);
    text.push('\n');
    for l in lines {
        text.push_str(l);
        text.push('\n');
    }
    text.push_str(BLOCK_END);
    text.push('\n');
    std::fs::write(path, text).with_context(|| format!("writing {}", path.display()))
}

/// Directory of the running binary, if it is not already on `PATH`.
fn exe_dir_missing_from_path() -> Option<PathBuf> {
    let dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let on_path = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d == dir))
        .unwrap_or(false);
    (!on_path).then_some(dir)
}

// ---- macOS Keychain ----

#[cfg(target_os = "macos")]
fn keychain_store(service: &str, value: &str) -> Result<()> {
    let user = std::env::var("USER").unwrap_or_default();
    let status = std::process::Command::new("security")
        .args(["add-generic-password", "-U", "-a", &user, "-s", service, "-w", value])
        .stdout(std::process::Stdio::null())
        .status()
        .context("running `security`")?;
    anyhow::ensure!(status.success(), "security add-generic-password failed");
    Ok(())
}

fn keychain_line(kind: ShellKind, key: &str) -> String {
    let lookup = format!("security find-generic-password -a \"$USER\" -s {key} -w 2>/dev/null");
    match kind {
        ShellKind::Posix => format!("export {key}=\"$({lookup})\""),
        ShellKind::Fish => format!("set -gx {key} ({lookup})"),
        ShellKind::PowerShell => format!("$env:{key} = ({})", lookup.replace("\"$USER\"", "$env:USER").replace("2>/dev/null", "2>$null")),
    }
}

fn rfc3339_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Howard Hinnant's days-to-civil.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}

// ---- the wizard ----

struct Connected {
    name: &'static str,
    vars: Vec<(String, String)>,
    provider: std::sync::Arc<dyn Provider>,
    models: Vec<ModelInfo>,
}

async fn connect() -> Result<Connected> {
    loop {
        println!();
        println!("Which model provider do you want to use?");
        println!("  1  Claude (Anthropic) — API key from console.anthropic.com");
        println!("  2  OpenAI — API key from platform.openai.com");
        println!("  3  Ollama, LM Studio or another OpenAI-compatible server (local, usually no key)");
        println!("  4  oMLX (local Apple Silicon server; reads ~/.omlx/settings.json)");
        let (name, vars): (&'static str, Vec<(String, String)>) = match choose("Provider", 4, 1)? {
            1 => ("anthropic", vec![("ANTHROPIC_API_KEY".into(), ask_secret("Anthropic API key (starts with sk-ant-, input hidden): ")?)]),
            2 => ("openai", vec![("OPENAI_API_KEY".into(), ask_secret("OpenAI API key (starts with sk-, input hidden): ")?)]),
            3 => {
                let mut v = vec![("OPENAI_BASE_URL".into(), ask_default("Server URL", "http://localhost:11434/v1")?)];
                let key = ask_secret("API key (press Enter if the server needs none, input hidden): ")?;
                if !key.is_empty() {
                    v.push(("OPENAI_API_KEY".into(), key));
                }
                ("openai", v)
            }
            _ => {
                let omlx_settings = dirs::home_dir().map(|h| h.join(".omlx/settings.json")).filter(|p| p.is_file());
                let v = match omlx_settings {
                    Some(_) => Vec::new(),
                    None => vec![("OMLX_BASE_URL".into(), ask_default("oMLX server URL", "http://localhost:8000/v1")?)],
                };
                ("omlx", v)
            }
        };
        if vars.iter().any(|(k, v)| k.ends_with("_KEY") && v.is_empty()) {
            println!("The key is empty.");
            continue;
        }
        for (k, v) in &vars {
            std::env::set_var(k, v);
        }
        print!("Checking the connection… ");
        io::stdout().flush()?;
        let result = match agentiloop_provider::from_env(Some(name)) {
            Ok(p) => p.list_models().await.map(|m| (p, m)),
            Err(e) => Err(e),
        };
        match result {
            Ok((provider, models)) => {
                println!("ok ({} model(s) available).", models.len());
                return Ok(Connected { name, vars, provider, models });
            }
            Err(e) => {
                println!("failed.\n  {e:#}");
                for (k, _) in &vars {
                    std::env::remove_var(k);
                }
                if !ask_yes("Try again?", true)? {
                    anyhow::bail!("setup cancelled; nothing was saved");
                }
            }
        }
    }
}

fn pick_model(c: &Connected) -> Result<String> {
    let default = c.provider.default_model();
    let default = if !default.is_empty() { default.to_string() } else { c.models.first().map(|m| m.id.clone()).unwrap_or_default() };
    if c.models.is_empty() {
        println!("The server lists no models; using `{default}`. Change it later with /model.");
        return Ok(default);
    }
    println!();
    println!("Pick a model (change it any time with /model):");
    let shown = c.models.iter().take(15).collect::<Vec<_>>();
    for (i, m) in shown.iter().enumerate() {
        let mark = if m.id == default { "  (default)" } else { "" };
        println!("  {:>2}  {}{mark}", i + 1, m.id);
    }
    if c.models.len() > shown.len() {
        println!("      … and {} more (type the id)", c.models.len() - shown.len());
    }
    loop {
        let a = ask(&format!("Model [1-{}, an id, or Enter for {default}]: ", shown.len()))?;
        if a.is_empty() {
            return Ok(default);
        }
        if let Ok(i) = a.parse::<usize>() {
            if (1..=shown.len()).contains(&i) {
                return Ok(shown[i - 1].id.clone());
            }
        }
        if c.models.iter().any(|m| m.id == a) {
            return Ok(a);
        }
        if ask_yes(&format!("`{a}` is not in the list; use it anyway?"), false)? {
            return Ok(a);
        }
    }
}

pub async fn run(saved: &mut settings::Settings) -> Result<()> {
    let home = settings::home().context("no home directory")?;
    println!("Welcome to AgentiLoop! Let's set things up (about a minute).");
    println!("Settings are kept in {}. Run `agentiloop --setup` to redo this, `agentiloop --reset` to start over.", home.display());

    let c = connect().await?;
    let model = pick_model(&c)?;

    // Where the credential lives. ~/.agentiloop/env is always the baseline unless the Keychain holds it.
    let profile = shell_profile();
    let mut setup = settings::Setup { completed_at: Some(rfc3339_now()), profile: None, keychain: Vec::new() };
    let mut block: Vec<String> = Vec::new();
    if !c.vars.is_empty() {
        println!();
        println!("Where should the credential be saved?");
        println!("  1  {} (recommended; only agentiloop reads it, file mode 600)", home.join("env").display());
        let mut n = 1;
        if let Some((p, _)) = &profile {
            n = 2;
            println!("  2  Also add it to {} so other tools in your terminal see it", p.display());
            if cfg!(target_os = "macos") {
                n = 3;
                println!("  3  macOS Keychain, with a line in {} that reads it (nothing stored in plain text)", p.display());
            }
        }
        let choice = choose("Save to", n, 1)?;
        let (path, kind) = profile.clone().unzip();
        match choice {
            3 => {
                #[cfg(target_os = "macos")]
                for (k, v) in &c.vars {
                    if k.ends_with("_KEY") || k.ends_with("_TOKEN") {
                        keychain_store(k, v)?;
                        setup.keychain.push(k.clone());
                        block.push(keychain_line(kind.unwrap(), k));
                        println!("stored {k} in the Keychain");
                    } else {
                        block.push(export_line(kind.unwrap(), k, v));
                    }
                }
                let _ = path;
            }
            2 => {
                settings::save_env_file(&c.vars)?;
                block.extend(c.vars.iter().map(|(k, v)| export_line(kind.unwrap(), k, v)));
            }
            _ => {
                let p = settings::save_env_file(&c.vars)?;
                println!("saved to {}", p.display());
            }
        }
    }

    // PATH: offer once, only when a profile is being written or would be.
    if let (Some(dir), Some((p, kind))) = (exe_dir_missing_from_path(), &profile) {
        println!();
        println!("`{}` is not on your PATH, so `agentiloop` only works with its full path.", dir.display());
        if ask_yes(&format!("Add it to PATH in {}?", p.display()), true)? {
            block.push(path_line(*kind, &dir));
        }
    }
    if !block.is_empty() {
        let (p, _) = profile.as_ref().expect("block implies profile");
        write_block(p, &block)?;
        setup.profile = Some(p.clone());
        println!("updated {} (between `{BLOCK_START}` and `{BLOCK_END}`); it applies to new terminals", p.display());
    }

    saved.set_model(c.name, &model);
    saved.last.provider = Some(c.name.to_string());
    saved.setup = setup;
    settings::save(saved)?;

    println!();
    println!("All set: {} / {}. Type a request at the prompt, /help for commands, /exit to leave.", c.name, model);
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_follows_shell_and_os() {
        let h = Path::new("/h");
        assert_eq!(profile_for("/bin/zsh", h, true).unwrap().0, h.join(".zshrc"));
        assert_eq!(profile_for("/bin/bash", h, true).unwrap().0, h.join(".bash_profile"));
        assert_eq!(profile_for("/usr/bin/bash", h, false).unwrap().0, h.join(".bashrc"));
        assert_eq!(profile_for("/opt/fish", h, false).unwrap(), (h.join(".config/fish/config.fish"), ShellKind::Fish));
        assert_eq!(profile_for("/usr/bin/pwsh", h, false).unwrap(), (h.join(".config/powershell/profile.ps1"), ShellKind::PowerShell));
        assert!(profile_for("/bin/tcsh", h, false).is_none());
        assert_eq!(powershell_profile(h).0, h.join("Documents").join("WindowsPowerShell").join("profile.ps1"));
    }

    #[test]
    fn export_lines_per_shell() {
        assert_eq!(export_line(ShellKind::Posix, "A", "b"), "export A=\"b\"");
        assert_eq!(export_line(ShellKind::Fish, "A", "b"), "set -gx A \"b\"");
        assert_eq!(export_line(ShellKind::PowerShell, "A", "b"), "$env:A = \"b\"");
        assert_eq!(path_line(ShellKind::PowerShell, Path::new("C:\\bin")), "$env:PATH = \"C:\\bin\" + [IO.Path]::PathSeparator + $env:PATH");
        assert!(keychain_line(ShellKind::Posix, "K").starts_with("export K=\"$(security find-generic-password"));
        assert_eq!(keychain_line(ShellKind::PowerShell, "K"), "$env:K = (security find-generic-password -a $env:USER -s K -w 2>$null)");
        // Everything the wizard writes must be recognised by --reset's stray-line scan when unmarked.
        for l in [export_line(ShellKind::PowerShell, "OPENAI_API_KEY", "x"), export_line(ShellKind::Fish, "OMLX_PORT", "1")] {
            assert_eq!(crate::reset::stray_lines(&l).len(), 1, "{l}");
        }
    }

    #[test]
    fn write_block_replaces_existing_block_and_keeps_the_rest() {
        let dir = std::env::temp_dir().join(format!("agentiloop-wizard-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("zshrc");
        std::fs::write(&p, "alias a=b\n# >>> agentiloop >>>\nexport OLD=1\n# <<< agentiloop <<<\n").unwrap();
        write_block(&p, &["export NEW=2".into()]).unwrap();
        let t = std::fs::read_to_string(&p).unwrap();
        assert_eq!(t, "alias a=b\n\n# >>> agentiloop >>>\nexport NEW=2\n# <<< agentiloop <<<\n");
        assert!(crate::reset::stray_lines(&t).is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rfc3339_shape() {
        let s = rfc3339_now();
        assert_eq!(s.len(), 20);
        assert!(s.ends_with('Z') && &s[4..5] == "-" && &s[10..11] == "T");
        assert!(s.starts_with("20"));
    }
}
