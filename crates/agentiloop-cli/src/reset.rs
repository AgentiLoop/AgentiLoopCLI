//! `agentiloop --reset`: put the machine back to "brand new" so the first-run
//! wizard can be tested again. Removes `~/.agentiloop`, the marked block the
//! wizard wrote to the shell profile, and Keychain items it created. Hand-written
//! `export` lines are never deleted, only commented out (with permission).

use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::settings;

/// Every environment variable the providers read for credentials.
pub const CREDENTIAL_VARS: [&str; 7] = [
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_OAUTH_TOKEN",
    "OPENAI_API_KEY",
    "OPENAI_BASE_URL",
    "OMLX_BASE_URL",
    "OMLX_PORT",
    "OMLX_API_KEY",
];
pub const BLOCK_START: &str = "# >>> agentiloop >>>";
pub const BLOCK_END: &str = "# <<< agentiloop <<<";
const COMMENT_PREFIX: &str = "# agentiloop-reset: ";

/// Shell profiles worth looking at: `AGENTILOOP_SHELL_PROFILE` alone when set
/// (so tests never touch the real ones), else the usual zsh/bash/fish/PowerShell files.
pub fn candidate_profiles() -> Vec<PathBuf> {
    if let Some(p) = std::env::var_os("AGENTILOOP_SHELL_PROFILE") {
        return vec![PathBuf::from(p)];
    }
    let Some(home) = dirs::home_dir() else { return Vec::new() };
    [
        ".zshrc",
        ".zprofile",
        ".bashrc",
        ".bash_profile",
        ".profile",
        ".config/fish/config.fish",
        // PowerShell: pwsh on Unix, then PowerShell 7 and Windows PowerShell 5 on Windows.
        ".config/powershell/profile.ps1",
        ".config/powershell/Microsoft.PowerShell_profile.ps1",
        "Documents/PowerShell/profile.ps1",
        "Documents/PowerShell/Microsoft.PowerShell_profile.ps1",
        "Documents/WindowsPowerShell/profile.ps1",
        "Documents/WindowsPowerShell/Microsoft.PowerShell_profile.ps1",
    ]
    .iter()
    .map(|f| home.join(f))
    .filter(|p| p.is_file())
    .collect()
}

/// Does this profile line set a credential variable or the `~/.local/bin` PATH entry?
fn is_stray(line: &str) -> bool {
    let l = line.trim();
    if l.is_empty() || l.starts_with('#') {
        return false;
    }
    let sets = |var: &str| {
        l.starts_with(&format!("{var}="))
            || l.starts_with(&format!("export {var}="))
            || l.starts_with(&format!("set -gx {var} "))
            || l.starts_with(&format!("set -x {var} "))
            || l.starts_with(&format!("$env:{var} "))
            || l.starts_with(&format!("$env:{var}="))
    };
    CREDENTIAL_VARS.iter().any(|v| sets(v)) || (sets("PATH") && l.contains(".local/bin"))
}

/// 1-based line numbers (and text) of hand-written lines outside the marked block.
pub fn stray_lines(text: &str) -> Vec<(usize, String)> {
    let mut inside = false;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        match line.trim() {
            BLOCK_START => inside = true,
            BLOCK_END => inside = false,
            _ if !inside && is_stray(line) => out.push((i + 1, line.to_string())),
            _ => {}
        }
    }
    out
}

pub fn has_block(text: &str) -> bool {
    text.lines().any(|l| l.trim() == BLOCK_START)
}

/// Re-applies the original file's line endings: a CRLF profile (Windows PowerShell,
/// Git `autocrlf`) stays CRLF instead of being silently rewritten as LF.
pub fn match_line_endings(original: &str, text: String) -> String {
    if original.contains("\r\n") {
        text.replace("\r\n", "\n").replace('\n', "\r\n")
    } else {
        text
    }
}

/// Drops everything from `BLOCK_START` through `BLOCK_END` (inclusive).
pub fn remove_block(text: &str) -> String {
    let mut inside = false;
    let mut out = String::new();
    for line in text.lines() {
        match line.trim() {
            BLOCK_START => inside = true,
            BLOCK_END => inside = false,
            _ if !inside => {
                out.push_str(line);
                out.push('\n');
            }
            _ => {}
        }
    }
    out
}

/// Prefixes the given 1-based lines with `# agentiloop-reset: ` so they can be restored by hand.
pub fn comment_out(text: &str, lines: &[usize]) -> String {
    let mut out = String::new();
    for (i, line) in text.lines().enumerate() {
        if lines.contains(&(i + 1)) {
            out.push_str(COMMENT_PREFIX);
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn ask(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut line = String::new();
    if io::stdin().lock().read_line(&mut line)? == 0 {
        anyhow::bail!("aborted");
    }
    Ok(line.trim().to_string())
}

pub fn unset_hint() -> String {
    if cfg!(windows) {
        return format!(
            "Variables already set in this window stay until you open a new one, or run:\n  Remove-Item Env:{}\nthen run `agentiloop` to see the first-run wizard.",
            CREDENTIAL_VARS.join(", Env:")
        );
    }
    format!(
        "Variables already exported in this terminal stay until you open a new one, or run:\n  unset {}\nthen run `agentiloop` to see the first-run wizard.",
        CREDENTIAL_VARS.join(" ")
    )
}

/// Windows: which credential variables are persisted as user environment variables
/// (`HKCU\\Environment`, as written by `setx`). Empty elsewhere.
fn user_env_present() -> Vec<String> {
    if !cfg!(windows) {
        return Vec::new();
    }
    CREDENTIAL_VARS
        .iter()
        .filter(|k| {
            std::process::Command::new("reg")
                .args(["query", "HKCU\\Environment", "/v", k])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .is_ok_and(|s| s.success())
        })
        .map(|k| k.to_string())
        .collect()
}

/// Windows: remove a user environment variable and broadcast the change (what `setx` cannot do).
fn user_env_delete(key: &str) -> Result<()> {
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &format!("[Environment]::SetEnvironmentVariable('{key}', $null, 'User')")])
        .stdout(std::process::Stdio::null())
        .status()
        .context("running `powershell`")?;
    anyhow::ensure!(status.success(), "could not remove user environment variable {key}");
    Ok(())
}

#[cfg(target_os = "macos")]
fn delete_keychain_item(service: &str) -> Result<()> {
    let user = std::env::var("USER").unwrap_or_default();
    let status = std::process::Command::new("security")
        .args(["delete-generic-password", "-a", &user, "-s", service])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;
    anyhow::ensure!(status.success(), "security delete-generic-password -s {service} failed");
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn delete_keychain_item(_service: &str) -> Result<()> {
    Ok(())
}

pub fn run(yes: bool) -> Result<()> {
    let saved = settings::load();
    let home = settings::home();
    let home_exists = home.as_deref().is_some_and(Path::exists);

    // Profiles with a marked block: the one settings.json points at, plus any other candidate.
    let mut profiles = candidate_profiles();
    if let Some(p) = &saved.setup.profile {
        if p.is_file() && !profiles.contains(p) {
            profiles.push(p.clone());
        }
    }
    let mut blocks: Vec<PathBuf> = Vec::new();
    let mut strays: Vec<(PathBuf, Vec<(usize, String)>)> = Vec::new();
    for p in &profiles {
        let Ok(text) = std::fs::read_to_string(p) else { continue };
        if has_block(&text) {
            blocks.push(p.clone());
        }
        let s = stray_lines(&text);
        if !s.is_empty() {
            strays.push((p.clone(), s));
        }
    }
    let keychain = &saved.setup.keychain;
    let mut user_env = saved.setup.user_env.clone();
    // Windows: credential variables persisted by hand with `setx` (the README's path).
    let stray_env: Vec<String> = user_env_present().into_iter().filter(|k| !user_env.contains(k)).collect();

    if !home_exists && blocks.is_empty() && strays.is_empty() && keychain.is_empty() && user_env.is_empty() && stray_env.is_empty() {
        println!("Nothing to reset: no {} and no agentiloop lines in your shell profile.", display(&home));
        println!("{}", unset_hint());
        return Ok(());
    }

    println!("This will:");
    if home_exists {
        println!("  • delete {} (settings.json, env, history.txt, sessions/, mcp.json — mcp.json is yours, back it up first)", display(&home));
    }
    for p in &blocks {
        println!("  • remove the `{BLOCK_START}` … `{BLOCK_END}` block from {}", p.display());
    }
    for k in keychain {
        println!("  • delete the macOS Keychain item `{k}`");
    }
    for k in &user_env {
        println!("  • remove the Windows user environment variable `{k}` (set by the wizard)");
    }
    if !stray_env.is_empty() {
        println!("  • found user environment variables set by hand (`setx`), not removed unless you say so: {}", stray_env.join(", "));
    }
    for (p, lines) in &strays {
        println!("  • found hand-written lines in {} (not deleted, see below):", p.display());
        for (n, l) in lines {
            println!("      {}:{n}: {}", p.display(), l.trim());
        }
    }

    if !yes && ask("\nType `reset` to continue: ")? != "reset" {
        println!("Cancelled; nothing changed.");
        return Ok(());
    }
    let comment = !strays.is_empty()
        && (yes || ask("Comment out the hand-written lines above (prefix `# agentiloop-reset: `)? [y/N] ")?.eq_ignore_ascii_case("y"));
    if !stray_env.is_empty()
        && (yes || ask("Remove those user environment variables too? [y/N] ")?.eq_ignore_ascii_case("y"))
    {
        user_env.extend(stray_env.iter().cloned());
    }

    for p in &blocks {
        let text = std::fs::read_to_string(p)?;
        std::fs::write(p, match_line_endings(&text, remove_block(&text))).with_context(|| format!("writing {}", p.display()))?;
        println!("removed block from {}", p.display());
    }
    if comment {
        for (p, lines) in &strays {
            // Line numbers were taken before the block was removed; recompute on the current text.
            let text = std::fs::read_to_string(p)?;
            let nums: Vec<usize> = stray_lines(&text).into_iter().map(|(n, _)| n).collect();
            std::fs::write(p, match_line_endings(&text, comment_out(&text, &nums))).with_context(|| format!("writing {}", p.display()))?;
            println!("commented out {} line(s) in {}", lines.len(), p.display());
        }
    }
    for k in keychain {
        match delete_keychain_item(k) {
            Ok(()) => println!("deleted Keychain item {k}"),
            Err(e) => println!("warning: {e}"),
        }
    }
    for k in &user_env {
        match user_env_delete(k) {
            Ok(()) => println!("removed user environment variable {k}"),
            Err(e) => println!("warning: {e}"),
        }
    }
    if let (true, Some(h)) = (home_exists, &home) {
        std::fs::remove_dir_all(h).with_context(|| format!("removing {}", h.display()))?;
        println!("deleted {}", h.display());
    }

    println!("\nDone. {}", unset_hint());
    Ok(())
}

fn display(p: &Option<PathBuf>) -> String {
    p.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "~/.agentiloop".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROFILE: &str = "export PATH=\"$HOME/.local/bin:$PATH\"\n# export OPENAI_API_KEY=old\nalias ll='ls -l'\n# >>> agentiloop >>>\nexport ANTHROPIC_API_KEY=\"sk-ant-1\"\n# <<< agentiloop <<<\nexport OPENAI_BASE_URL=http://localhost:11434/v1\nset -gx OMLX_PORT 7777\n";

    #[test]
    fn strays_skip_comments_and_the_marked_block() {
        let nums: Vec<usize> = stray_lines(PROFILE).into_iter().map(|(n, _)| n).collect();
        assert_eq!(nums, vec![1, 7, 8]);
    }

    #[test]
    fn remove_block_keeps_everything_else() {
        let out = remove_block(PROFILE);
        assert!(!out.contains("agentiloop") && !out.contains("sk-ant-1"));
        assert_eq!(out.lines().count(), 5);
        assert!(!has_block(&out) && has_block(PROFILE));
    }

    #[test]
    fn comment_out_is_reversible_by_eye() {
        let out = comment_out("a\nexport OPENAI_API_KEY=x\nb\n", &[2]);
        assert_eq!(out, "a\n# agentiloop-reset: export OPENAI_API_KEY=x\nb\n");
        assert!(stray_lines(&out).is_empty());
    }

    #[test]
    fn crlf_profiles_stay_crlf() {
        let crlf = "a\r\n# >>> agentiloop >>>\r\nexport OPENAI_API_KEY=x\r\n# <<< agentiloop <<<\r\nb\r\n";
        assert_eq!(match_line_endings(crlf, remove_block(crlf)), "a\r\nb\r\n");
        assert_eq!(match_line_endings("a\nb\n", "a\nb\n".into()), "a\nb\n");
    }
}
