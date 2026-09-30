//! Project instructions: `AGENTS.md` (or `CLAUDE.md`) files whose text is appended to the system prompt,
//! so the agent follows the conventions of the project it is working in.

use std::path::{Path, PathBuf};

/// File names tried in each directory, in order.
pub const FILE_NAMES: &[&str] = &["AGENTS.md", "CLAUDE.md"];
/// Longest instructions file that is sent whole; the rest is cut.
pub const MAX_BYTES: usize = 32 * 1024;

/// One loaded instructions file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instructions {
    pub path: PathBuf,
    pub text: String,
}

fn read_in(dir: &Path) -> Option<Instructions> {
    for name in FILE_NAMES {
        let path = dir.join(name);
        let Ok(bytes) = std::fs::read(&path) else { continue };
        let mut text = String::from_utf8_lossy(&bytes).into_owned();
        if text.trim().is_empty() {
            continue;
        }
        if text.len() > MAX_BYTES {
            let mut cut = MAX_BYTES;
            while !text.is_char_boundary(cut) {
                cut -= 1;
            }
            text.truncate(cut);
            text.push_str("\n…[truncated]");
        }
        return Some(Instructions { path, text });
    }
    None
}

/// Loads the user-level file from `home` (the `~/.agentiloop` folder) and the nearest project file:
/// `cwd` and then its parents, stopping after the first directory that holds `.git`.
/// User-level comes first so project instructions have the last word.
pub fn load(cwd: &Path, home: Option<&Path>) -> Vec<Instructions> {
    let mut found = Vec::new();
    if let Some(i) = home.and_then(read_in) {
        found.push(i);
    }
    for dir in cwd.ancestors() {
        if let Some(i) = read_in(dir) {
            if !found.iter().any(|f| f.path == i.path) {
                found.push(i);
            }
            break;
        }
        if dir.join(".git").exists() {
            break;
        }
    }
    found
}

/// `base` followed by the loaded instructions; `base` unchanged when there are none.
pub fn append_to_prompt(base: &str, found: &[Instructions]) -> String {
    let mut out = base.to_string();
    for i in found {
        out.push_str(&format!(
            "\n\n# Instructions from {}\nThe user keeps these instructions for their projects. Follow them.\n\n{}",
            i.path.display(),
            i.text.trim_end()
        ));
    }
    out
}

/// Starter `AGENTS.md` text for the project in `dir`, with build/test commands guessed from the files present.
pub fn init_template(dir: &Path) -> String {
    let name = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "this project".into());
    let has = |f: &str| dir.join(f).exists();
    let mut cmds: Vec<&str> = Vec::new();
    if has("Cargo.toml") {
        cmds.extend(["cargo build", "cargo test", "cargo clippy"]);
    }
    if has("go.mod") {
        cmds.extend(["go build ./...", "go test ./...", "go vet ./..."]);
    }
    if has("package.json") {
        cmds.extend(["npm install", "npm test"]);
    }
    if has("pyproject.toml") || has("requirements.txt") {
        cmds.push("python -m pytest");
    }
    if has("Makefile") {
        cmds.push("make");
    }
    let commands = if cmds.is_empty() {
        "- (add the commands to build, test and lint this project)".to_string()
    } else {
        cmds.iter().map(|c| format!("- `{c}`")).collect::<Vec<_>>().join("\n")
    };
    format!(
        "# {name}\n\nInstructions for AgentiLoop and other coding agents working in this repository.\n\n\
## Commands\n\n{commands}\n\n\
## Conventions\n\n- (code style, naming, where new code goes)\n\n\
## Do not\n\n- (things to avoid: generated files, secrets, risky commands)\n"
    )
}

/// Creates `AGENTS.md` in `dir` from [`init_template`]. Refuses to overwrite an existing file.
pub fn init(dir: &Path) -> Result<PathBuf, String> {
    let path = dir.join(FILE_NAMES[0]);
    if path.exists() {
        return Err(format!("{} already exists", path.display()));
    }
    std::fs::write(&path, init_template(dir)).map_err(|e| format!("could not write {}: {e}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("agentiloop-instr-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn finds_nearest_file_up_to_the_repo_root() {
        let root = scratch("walk");
        std::fs::create_dir_all(root.join("repo/.git")).unwrap();
        std::fs::create_dir_all(root.join("repo/a/b")).unwrap();
        std::fs::write(root.join("AGENTS.md"), "outside the repo").unwrap();
        std::fs::write(root.join("repo/AGENTS.md"), "use tabs").unwrap();
        let got = load(&root.join("repo/a/b"), None);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].text, "use tabs");
        // A repo without its own file does not pick up one from above the repo.
        std::fs::remove_file(root.join("repo/AGENTS.md")).unwrap();
        assert!(load(&root.join("repo/a/b"), None).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn agents_md_beats_claude_md_and_blank_files_are_ignored() {
        let d = scratch("names");
        std::fs::write(d.join("CLAUDE.md"), "from claude").unwrap();
        assert_eq!(load(&d, None)[0].text, "from claude");
        std::fs::write(d.join("AGENTS.md"), "from agents").unwrap();
        assert_eq!(load(&d, None)[0].text, "from agents");
        std::fs::write(d.join("AGENTS.md"), "  \n").unwrap();
        assert_eq!(load(&d, None)[0].text, "from claude");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn user_file_comes_first_and_long_files_are_cut() {
        let proj = scratch("proj");
        let home = scratch("home");
        std::fs::write(home.join("AGENTS.md"), "global").unwrap();
        std::fs::write(proj.join("AGENTS.md"), "é".repeat(MAX_BYTES)).unwrap();
        let got = load(&proj, Some(&home));
        assert_eq!(got.iter().map(|i| i.text.len() > 10).collect::<Vec<_>>(), vec![false, true]);
        assert!(got[1].text.ends_with("…[truncated]"));
        assert!(got[1].text.len() <= MAX_BYTES + "\n…[truncated]".len());
        let p = append_to_prompt("BASE", &got);
        assert!(p.starts_with("BASE\n\n# Instructions from "));
        assert!(p.find("global").unwrap() < p.find("éé").unwrap());
        assert_eq!(append_to_prompt("BASE", &[]), "BASE");
        let _ = std::fs::remove_dir_all(&proj);
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn init_writes_a_template_with_detected_commands_and_never_overwrites() {
        let d = scratch("init");
        std::fs::write(d.join("Cargo.toml"), "").unwrap();
        std::fs::write(d.join("Makefile"), "").unwrap();
        let path = init(&d).unwrap();
        assert_eq!(path, d.join("AGENTS.md"));
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("`cargo test`") && text.contains("- `make`") && !text.contains("go vet"), "{text}");
        assert!(text.starts_with("# agentiloop-instr-init-"), "{text}");
        let err = init(&d).unwrap_err();
        assert!(err.contains("already exists"), "{err}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
        // The generated file is picked up as project instructions.
        assert_eq!(load(&d, None)[0].text, text);
        let empty = scratch("init-empty");
        assert!(init_template(&empty).contains("(add the commands"));
        let _ = std::fs::remove_dir_all(&d);
        let _ = std::fs::remove_dir_all(&empty);
    }
}
