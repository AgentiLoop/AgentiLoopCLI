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

/// Deepest chain of `@file` imports followed.
const MAX_IMPORT_DEPTH: usize = 3;

/// Replaces each line that is just `@path` with that file's text, so AGENTS.md can pull in other docs.
/// Paths are relative to the importing file (or absolute, or `~/…`). Lines inside code fences, missing
/// files, import cycles and chains deeper than [`MAX_IMPORT_DEPTH`] are left as they are.
fn expand_imports(text: &str, dir: &Path, depth: usize, stack: &mut Vec<PathBuf>) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut fenced = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("```") || t.starts_with("~~~") {
            fenced = !fenced;
        }
        let rel = t.strip_prefix('@').filter(|r| !fenced && !r.is_empty() && !r.contains(char::is_whitespace));
        let imported = rel.and_then(|rel| {
            if depth >= MAX_IMPORT_DEPTH {
                return None;
            }
            let path = match rel.strip_prefix("~/") {
                Some(r) => PathBuf::from(std::env::var_os("HOME")?).join(r),
                None => dir.join(rel),
            };
            let canon = path.canonicalize().ok()?;
            if stack.contains(&canon) {
                return None;
            }
            let body = String::from_utf8_lossy(&std::fs::read(&canon).ok()?).into_owned();
            stack.push(canon.clone());
            let body = expand_imports(&body, canon.parent().unwrap_or(dir), depth + 1, stack);
            stack.pop();
            Some(format!("[imported from {}]\n{}", path.display(), body.trim_end()))
        });
        out.push(imported.unwrap_or_else(|| line.to_string()));
    }
    let mut joined = out.join("\n");
    if text.ends_with('\n') {
        joined.push('\n');
    }
    joined
}

fn read_in(dir: &Path) -> Option<Instructions> {
    for name in FILE_NAMES {
        let path = dir.join(name);
        let Ok(bytes) = std::fs::read(&path) else { continue };
        let mut text = String::from_utf8_lossy(&bytes).into_owned();
        if text.trim().is_empty() {
            continue;
        }
        let mut stack = vec![path.canonicalize().unwrap_or_else(|_| path.clone())];
        text = expand_imports(&text, dir, 0, &mut stack);
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

/// The note behind a `# note` prompt line: the text after `# `, or None when the line isn't
/// one (`#` alone, a `#hashtag`, or a line that merely contains a `#`).
pub fn note_line(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix("# ")?.trim();
    (!rest.is_empty()).then_some(rest)
}

/// Heading under which `remember` collects notes.
pub const NOTES_HEADING: &str = "## Notes";

/// The project instruction file a note goes to: the nearest `AGENTS.md` / `CLAUDE.md` from
/// `cwd` upwards (stopping at the repository root), else a new `AGENTS.md` in `cwd`.
pub fn notes_path(cwd: &Path) -> PathBuf {
    for dir in cwd.ancestors() {
        if let Some(path) = FILE_NAMES.iter().map(|n| dir.join(n)).find(|p| p.is_file()) {
            return path;
        }
        if dir.join(".git").exists() {
            break;
        }
    }
    cwd.join(FILE_NAMES[0])
}

/// Appends `note` as a bullet at the end of the `NOTES_HEADING` section of the project's
/// instruction file (`notes_path`), adding the section when missing and creating the file
/// from `init_template` when there is none. Returns the file.
pub fn remember(cwd: &Path, note: &str) -> Result<PathBuf, String> {
    let path = notes_path(cwd);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => init_template(path.parent().unwrap_or(cwd)),
        Err(e) => return Err(format!("could not read {}: {e}", path.display())),
    };
    let bullet = format!("- {}", note.trim().replace('\n', " "));
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    match lines.iter().position(|l| l.trim_end() == NOTES_HEADING) {
        Some(h) => {
            // The section ends at the next heading (or the file's end), trailing blank lines excluded.
            let mut end = lines[h + 1..].iter().position(|l| l.starts_with("## ")).map_or(lines.len(), |i| h + 1 + i);
            while end > h + 1 && lines[end - 1].trim().is_empty() {
                end -= 1;
            }
            let insert = if end == h + 1 { vec![String::new(), bullet] } else { vec![bullet] };
            lines.splice(end..end, insert);
        }
        None => {
            if lines.last().is_some_and(|l| !l.trim().is_empty()) {
                lines.push(String::new());
            }
            lines.extend([NOTES_HEADING.to_string(), String::new(), bullet]);
        }
    }
    let out = lines.join("\n") + "\n";
    std::fs::write(&path, out).map_err(|e| format!("could not write {}: {e}", path.display()))?;
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
    fn at_imports_inline_files_and_leave_unresolvable_lines_alone() {
        let d = scratch("imports");
        std::fs::create_dir_all(d.join("docs")).unwrap();
        std::fs::write(d.join("docs/style.md"), "use tabs\n@more.md\n").unwrap();
        std::fs::write(d.join("docs/more.md"), "and snake_case\n").unwrap();
        std::fs::write(d.join("loop.md"), "@AGENTS.md\n").unwrap();
        std::fs::write(
            d.join("AGENTS.md"),
            "# Rules\n@docs/style.md\n@missing.md\n@loop.md\nemail me @ home\n```\n@docs/style.md\n```\n",
        )
        .unwrap();
        let text = load(&d, None).remove(0).text;
        assert!(text.contains("[imported from") && text.contains("use tabs\n[imported from"), "{text}");
        assert!(text.contains("and snake_case"), "nested import: {text}");
        assert!(text.contains("\n@missing.md\n"), "missing file stays: {text}");
        assert!(text.contains("email me @ home"));
        assert!(text.contains("```\n@docs/style.md\n```"), "fenced lines stay: {text}");
        // loop.md imports AGENTS.md, which is already being read: that line is left alone.
        assert!(text.contains("[imported from") && text.contains("\n@AGENTS.md"), "cycle: {text}");
        assert_eq!(text.matches("use tabs").count(), 1);
        let _ = std::fs::remove_dir_all(&d);
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

    #[test]
    fn note_lines_need_a_hash_and_a_space() {
        assert_eq!(note_line("# always run cargo fmt"), Some("always run cargo fmt"));
        assert_eq!(note_line("  #  spaced  "), Some("spaced"));
        assert_eq!(note_line("#"), None);
        assert_eq!(note_line("# "), None);
        assert_eq!(note_line("#hashtag"), None);
        assert_eq!(note_line("fix bug #12"), None);
    }

    #[test]
    fn remember_creates_appends_and_keeps_the_notes_section_together() {
        let d = scratch("remember");
        std::fs::write(d.join("Cargo.toml"), "").unwrap();
        // No file: the template is created with a Notes section at the end.
        let path = remember(&d, "use tabs").unwrap();
        assert_eq!(path, d.join("AGENTS.md"));
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("# agentiloop-instr-remember-") && text.ends_with("## Do not\n\n- (things to avoid: generated files, secrets, risky commands)\n\n## Notes\n\n- use tabs\n"), "{text}");
        // Second note joins the list.
        remember(&d, "  no unwrap\nin lib code ").unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.ends_with("## Notes\n\n- use tabs\n- no unwrap in lib code\n"), "{text}");
        // A Notes section in the middle keeps its place; the file keeps its trailing text.
        std::fs::write(&path, "# P\n\n## Notes\n\n- one\n\n\n## Do not\n\n- x\n").unwrap();
        remember(&d, "two").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "# P\n\n## Notes\n\n- one\n- two\n\n\n## Do not\n\n- x\n");
        // An empty Notes heading at the end gets a blank line, then the bullet.
        std::fs::write(&path, "# P\n\n## Notes").unwrap();
        remember(&d, "three").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "# P\n\n## Notes\n\n- three\n");
        // CLAUDE.md in a parent (inside the repo) is the file; a .git boundary stops the walk.
        let repo = scratch("remember-repo");
        std::fs::create_dir_all(repo.join(".git")).unwrap();
        std::fs::create_dir_all(repo.join("sub/deep")).unwrap();
        std::fs::write(repo.join("CLAUDE.md"), "rules\n").unwrap();
        assert_eq!(notes_path(&repo.join("sub/deep")), repo.join("CLAUDE.md"));
        std::fs::write(repo.parent().unwrap().join("AGENTS.md"), "outside\n").unwrap();
        std::fs::remove_file(repo.join("CLAUDE.md")).unwrap();
        assert_eq!(notes_path(&repo.join("sub/deep")), repo.join("sub/deep/AGENTS.md"));
        std::fs::remove_file(repo.parent().unwrap().join("AGENTS.md")).unwrap();
        let _ = std::fs::remove_dir_all(&d);
        let _ = std::fs::remove_dir_all(&repo);
    }
}
