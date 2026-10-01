//! Custom slash commands: every `*.md` file in `<project>/.agentiloop/commands` and in the user's
//! `~/.agentiloop/commands` becomes `/<file name>`. Typing it sends the file's text as the prompt,
//! with `$ARGUMENTS` replaced by whatever follows the command (or appended when the file has no
//! placeholder). A project file wins over a user file of the same name; built-in commands always win.

use std::path::{Path, PathBuf};

/// Commands built into the program; a file with one of these names is ignored.
pub const BUILTIN: &[&str] = &[
    "clear", "usage", "export", "init", "diff", "todos", "undo", "model", "compact", "sessions", "resume", "mcp", "commands",
    "help", "setup", "exit", "quit",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomCommand {
    pub name: String,
    pub path: PathBuf,
    pub template: String,
}

impl CustomCommand {
    /// First non-blank line of the file, without leading `#`, at most 70 characters.
    pub fn description(&self) -> String {
        let line = self.template.lines().map(|l| l.trim().trim_start_matches('#').trim()).find(|l| !l.is_empty()).unwrap_or("");
        let mut d: String = line.chars().take(70).collect();
        if line.chars().count() > 70 {
            d.push('…');
        }
        d
    }

    /// The prompt to send for `/name args`.
    pub fn expand(&self, args: &str) -> String {
        if self.template.contains("$ARGUMENTS") {
            self.template.replace("$ARGUMENTS", args).trim().to_string()
        } else if args.is_empty() {
            self.template.trim().to_string()
        } else {
            format!("{}\n\n{args}", self.template.trim_end())
        }
    }
}

fn read_dir(dir: &Path, out: &mut Vec<CustomCommand>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let path = e.path();
        if path.extension().and_then(|x| x.to_str()) != Some("md") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|s| s.to_str()).map(str::to_string) else { continue };
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') || BUILTIN.contains(&name.as_str()) {
            continue;
        }
        let Ok(bytes) = std::fs::read(&path) else { continue };
        let template = String::from_utf8_lossy(&bytes).into_owned();
        if template.trim().is_empty() {
            continue;
        }
        out.retain(|c| c.name != name);
        out.push(CustomCommand { name, path, template });
    }
}

/// All custom commands, sorted by name. `home` is the `~/.agentiloop` folder.
pub fn load(cwd: &Path, home: Option<&Path>) -> Vec<CustomCommand> {
    let mut out = Vec::new();
    if let Some(h) = home {
        read_dir(&h.join("commands"), &mut out);
    }
    read_dir(&cwd.join(".agentiloop").join("commands"), &mut out);
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// The prompt for a typed line like `/review src/main.rs`, or `None` when it is not a custom command.
pub fn resolve(line: &str, cwd: &Path, home: Option<&Path>) -> Option<String> {
    let rest = line.trim().strip_prefix('/')?;
    let (name, args) = rest.split_once(char::is_whitespace).map_or((rest, ""), |(n, a)| (n, a.trim()));
    load(cwd, home).into_iter().find(|c| c.name == name).map(|c| c.expand(args))
}

/// Text for `/commands`.
pub fn listing(cmds: &[CustomCommand]) -> String {
    if cmds.is_empty() {
        return "no custom commands. Put a prompt in .agentiloop/commands/<name>.md (project) or ~/.agentiloop/commands/<name>.md, then type /<name> [arguments]".into();
    }
    cmds.iter().map(|c| format!("/{:<14} {}", c.name, c.description())).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("agentiloop-cmds-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn put(dir: &Path, file: &str, body: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join(file), body).unwrap();
    }

    #[test]
    fn loads_project_over_user_and_skips_builtins_and_bad_names() {
        let d = scratch("load");
        let (home, proj) = (d.join("home"), d.join("proj"));
        put(&home.join("commands"), "review.md", "# User review\nlook at $ARGUMENTS");
        put(&home.join("commands"), "only-user.md", "hello");
        put(&proj.join(".agentiloop/commands"), "review.md", "# Project review\nlook at $ARGUMENTS");
        put(&proj.join(".agentiloop/commands"), "clear.md", "builtin name");
        put(&proj.join(".agentiloop/commands"), "bad name.md", "x");
        put(&proj.join(".agentiloop/commands"), "blank.md", "  \n");
        put(&proj.join(".agentiloop/commands"), "notes.txt", "x");
        let cmds = load(&proj, Some(&home));
        let names: Vec<&str> = cmds.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["only-user", "review"]);
        assert_eq!(cmds[1].description(), "Project review");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn expand_replaces_or_appends_arguments_and_resolve_only_matches_custom() {
        let d = scratch("expand");
        put(&d.join(".agentiloop/commands"), "fix.md", "Fix this bug: $ARGUMENTS. Add a test.\n");
        put(&d.join(".agentiloop/commands"), "plain.md", "Summarize the repo.\n");
        assert_eq!(resolve("/fix  the crash on exit ", &d, None).unwrap(), "Fix this bug: the crash on exit. Add a test.");
        assert_eq!(resolve("/plain", &d, None).unwrap(), "Summarize the repo.");
        assert_eq!(resolve("/plain in one line", &d, None).unwrap(), "Summarize the repo.\n\nin one line");
        assert!(resolve("/nothing", &d, None).is_none());
        assert!(resolve("not a command", &d, None).is_none());
        assert!(listing(&load(&d, None)).contains("/fix"));
        assert!(listing(&[]).contains("no custom commands"));
        let _ = std::fs::remove_dir_all(&d);
    }
}
