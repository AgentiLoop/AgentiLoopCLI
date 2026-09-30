//! Read-only search tools: `glob` (find files by name) and `grep` (find text in files).

use std::path::{Path, PathBuf};

use agentiloop_core::{Tool, ToolContext, ToolError, ToolResult};
use async_trait::async_trait;
use regex::RegexBuilder;
use serde::Deserialize;
use serde_json::{json, Value};

/// Directories never descended into: VCS data and dependency/build output that would drown real hits.
const SKIP_DIRS: &[&str] = &[".git", ".hg", ".svn", "node_modules", "target", "__pycache__", ".venv", "venv"];
const MAX_GLOB_RESULTS: usize = 500;
const DEFAULT_GREP_RESULTS: usize = 200;
const MAX_GREP_RESULTS: usize = 2000;
const MAX_LINE_CHARS: usize = 300;
const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;

fn resolve(ctx: &ToolContext, p: &str) -> PathBuf {
    let path = Path::new(p);
    if path.is_absolute() { path.to_path_buf() } else { ctx.cwd.join(path) }
}

fn parse<T: for<'de> Deserialize<'de>>(input: Value) -> Result<T, ToolError> {
    serde_json::from_value(input).map_err(|e| ToolError::InvalidInput(e.to_string()))
}

/// `path` as the model should quote it back: relative to the working directory when inside it.
fn display(ctx: &ToolContext, path: &Path) -> String {
    let p = path.strip_prefix(&ctx.cwd).unwrap_or(path);
    p.to_string_lossy().replace('\\', "/")
}

// ------------------------------------------------------------------- globbing

/// `a.{rs,go}` → `a.rs`, `a.go` (nested braces supported).
fn expand_braces(p: &str) -> Vec<String> {
    let chars: Vec<char> = p.chars().collect();
    let Some(open) = chars.iter().position(|&c| c == '{') else { return vec![p.to_string()] };
    let (mut depth, mut close) = (0, None);
    for (i, &c) in chars.iter().enumerate().skip(open) {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }
    let Some(close) = close else { return vec![p.to_string()] };
    let (prefix, suffix): (String, String) = (chars[..open].iter().collect(), chars[close + 1..].iter().collect());
    let (mut alts, mut cur, mut depth) = (Vec::new(), String::new(), 0);
    for &c in &chars[open + 1..close] {
        match c {
            ',' if depth == 0 => alts.push(std::mem::take(&mut cur)),
            _ => {
                if c == '{' { depth += 1 } else if c == '}' { depth -= 1 }
                cur.push(c);
            }
        }
    }
    alts.push(cur);
    alts.iter().flat_map(|a| expand_braces(&format!("{prefix}{a}{suffix}"))).collect()
}

/// `*` and `?` within one path segment.
fn segment_match(pat: &[char], s: &[char]) -> bool {
    let (mut p, mut i) = (0, 0);
    let (mut star, mut mark) = (None, 0);
    while i < s.len() {
        if p < pat.len() && (pat[p] == '?' || (pat[p] != '*' && pat[p] == s[i])) {
            p += 1;
            i += 1;
        } else if p < pat.len() && pat[p] == '*' {
            star = Some(p);
            mark = i;
            p += 1;
        } else if let Some(sp) = star {
            p = sp + 1;
            mark += 1;
            i = mark;
        } else {
            return false;
        }
    }
    while p < pat.len() && pat[p] == '*' {
        p += 1;
    }
    p == pat.len()
}

fn segments_match(pat: &[Vec<char>], path: &[Vec<char>]) -> bool {
    match pat.split_first() {
        None => path.is_empty(),
        Some((first, rest)) if first.iter().collect::<String>() == "**" => {
            (0..=path.len()).any(|skip| segments_match(rest, &path[skip..]))
        }
        Some((first, rest)) => match path.split_first() {
            Some((seg, tail)) => segment_match(first, seg) && segments_match(rest, tail),
            None => false,
        },
    }
}

/// Compiled glob: `*`/`?` within a segment, `**` across directories, `{a,b}` alternatives.
/// A pattern without `/` matches the file name at any depth.
struct Glob(Vec<Vec<Vec<char>>>);

impl Glob {
    fn new(pattern: &str) -> Glob {
        let pattern = pattern.replace('\\', "/");
        let pattern = pattern.trim_start_matches("./");
        let alts = expand_braces(pattern)
            .into_iter()
            .map(|p| {
                let p = if p.contains('/') { p } else { format!("**/{p}") };
                p.split('/').filter(|s| !s.is_empty()).map(|s| s.chars().collect()).collect()
            })
            .collect();
        Glob(alts)
    }

    fn matches(&self, rel: &str) -> bool {
        let path: Vec<Vec<char>> = rel.split('/').map(|s| s.chars().collect()).collect();
        self.0.iter().any(|p| segments_match(p, &path))
    }
}

/// Depth-first, name-sorted walk of `root`; `visit(abs, rel)` returns false to stop.
/// Symlinked directories are not followed.
fn walk(root: &Path, visit: &mut dyn FnMut(&Path, &str) -> bool) {
    fn go(dir: &Path, rel: &str, visit: &mut dyn FnMut(&Path, &str) -> bool) -> bool {
        let Ok(rd) = std::fs::read_dir(dir) else { return true };
        let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let name = e.file_name().to_string_lossy().into_owned();
            let child_rel = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_dir() {
                if !SKIP_DIRS.contains(&name.as_str()) && !go(&e.path(), &child_rel, visit) {
                    return false;
                }
            } else if ft.is_file() && !visit(&e.path(), &child_rel) {
                return false;
            }
        }
        true
    }
    go(root, "", visit);
}

// ----------------------------------------------------------------------- glob

pub struct GlobFiles;

#[derive(Deserialize)]
struct GlobArgs {
    pattern: String,
    #[serde(default = "default_dot")]
    path: String,
}

fn default_dot() -> String {
    ".".into()
}

#[async_trait]
impl Tool for GlobFiles {
    fn name(&self) -> &str { "glob" }
    fn description(&self) -> &str {
        "Find files by name pattern, searching recursively and skipping .git, node_modules, target and similar. \
         `*` and `?` match within one path segment, `**` matches any number of directories, `{a,b}` offers alternatives. \
         A pattern without `/` (like `*.rs`) matches file names at any depth; one with `/` (like `src/**/*.go`) \
         matches the path relative to `path`. Returns paths sorted by name."
    }
    fn input_schema(&self) -> Value {
        json!({"type":"object","properties":{
            "pattern":{"type":"string","description":"Glob, e.g. `**/*.{ts,tsx}` or `*.md`"},
            "path":{"type":"string","description":"Directory to search (default: cwd)","default":"."}
        },"required":["pattern"]})
    }
    async fn call(&self, ctx: &ToolContext, input: Value) -> ToolResult {
        let a: GlobArgs = parse(input)?;
        let root = resolve(ctx, &a.path);
        if !root.is_dir() {
            return Err(ToolError::Failed(format!("{} is not a directory", root.display())));
        }
        let ctx = ctx.clone();
        tokio::task::spawn_blocking(move || {
            let glob = Glob::new(&a.pattern);
            let mut found: Vec<String> = Vec::new();
            let mut more = false;
            walk(&root, &mut |abs, rel| {
                if !glob.matches(rel) {
                    return true;
                }
                if found.len() == MAX_GLOB_RESULTS {
                    more = true;
                    return false;
                }
                found.push(display(&ctx, abs));
                true
            });
            if found.is_empty() {
                return "no matches".to_string();
            }
            let mut out = found.join("\n");
            if more {
                out.push_str(&format!("\n…[truncated: more than {MAX_GLOB_RESULTS} matches; narrow the pattern]"));
            }
            out
        })
        .await
        .map_err(|e| ToolError::Failed(e.to_string()))
    }
}

// ----------------------------------------------------------------------- grep

pub struct Grep;

#[derive(Deserialize)]
struct GrepArgs {
    pattern: String,
    #[serde(default = "default_dot")]
    path: String,
    glob: Option<String>,
    #[serde(default)]
    case_insensitive: bool,
    max_results: Option<usize>,
}

fn truncate_line(line: &str) -> String {
    let line = line.trim_end_matches('\r');
    if line.chars().count() > MAX_LINE_CHARS {
        let cut: String = line.chars().take(MAX_LINE_CHARS).collect();
        format!("{cut}…")
    } else {
        line.to_string()
    }
}

#[async_trait]
impl Tool for Grep {
    fn name(&self) -> &str { "grep" }
    fn description(&self) -> &str {
        "Search file contents with a regular expression (RE2-style syntax). Searches recursively, skipping .git, \
         node_modules, target, binary and very large files. Returns `path:line:text` for each matching line. \
         Use `glob` to restrict which files are searched (same syntax as the glob tool)."
    }
    fn input_schema(&self) -> Value {
        json!({"type":"object","properties":{
            "pattern":{"type":"string","description":"Regular expression to look for"},
            "path":{"type":"string","description":"File or directory to search (default: cwd)","default":"."},
            "glob":{"type":"string","description":"Only search files matching this glob, e.g. `*.rs`"},
            "case_insensitive":{"type":"boolean","default":false},
            "max_results":{"type":"integer","description":"Maximum matching lines to return","default":200}
        },"required":["pattern"]})
    }
    async fn call(&self, ctx: &ToolContext, input: Value) -> ToolResult {
        let a: GrepArgs = parse(input)?;
        let re = RegexBuilder::new(&a.pattern)
            .case_insensitive(a.case_insensitive)
            .build()
            .map_err(|e| ToolError::InvalidInput(format!("bad regex: {e}")))?;
        let root = resolve(ctx, &a.path);
        if !root.exists() {
            return Err(ToolError::Failed(format!("{} does not exist", root.display())));
        }
        let limit = a.max_results.unwrap_or(DEFAULT_GREP_RESULTS).clamp(1, MAX_GREP_RESULTS);
        let filter = a.glob.as_deref().map(Glob::new);
        let ctx = ctx.clone();
        tokio::task::spawn_blocking(move || {
            let mut hits: Vec<String> = Vec::new();
            let mut more = false;
            let mut search = |abs: &Path, rel: &str| -> bool {
                if filter.as_ref().is_some_and(|g| !g.matches(rel)) {
                    return true;
                }
                if std::fs::metadata(abs).map(|m| m.len() > MAX_FILE_BYTES).unwrap_or(true) {
                    return true;
                }
                let Ok(bytes) = std::fs::read(abs) else { return true };
                if bytes[..bytes.len().min(8000)].contains(&0) {
                    return true;
                }
                let text = String::from_utf8_lossy(&bytes);
                let shown = display(&ctx, abs);
                for (i, line) in text.lines().enumerate() {
                    if re.is_match(line) {
                        if hits.len() == limit {
                            more = true;
                            return false;
                        }
                        hits.push(format!("{shown}:{}:{}", i + 1, truncate_line(line)));
                    }
                }
                true
            };
            if root.is_file() {
                let name = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                search(&root, &name);
            } else {
                walk(&root, &mut search);
            }
            if hits.is_empty() {
                return "no matches".to_string();
            }
            let mut out = hits.join("\n");
            if more {
                out.push_str(&format!("\n…[truncated at {limit} matches; narrow the pattern or path, or raise max_results]"));
            }
            out
        })
        .await
        .map_err(|e| ToolError::Failed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(pat: &str, path: &str) -> bool {
        Glob::new(pat).matches(path)
    }

    #[test]
    fn bare_patterns_match_names_at_any_depth() {
        assert!(m("*.rs", "main.rs"));
        assert!(m("*.rs", "a/b/c/main.rs"));
        assert!(!m("*.rs", "main.go"));
        assert!(m("Cargo.toml", "crates/x/Cargo.toml"));
    }

    #[test]
    fn slash_patterns_match_the_relative_path() {
        assert!(m("src/*.rs", "src/main.rs"));
        assert!(!m("src/*.rs", "src/a/main.rs"));
        assert!(m("src/**/*.rs", "src/main.rs"));
        assert!(m("src/**/*.rs", "src/a/b/main.rs"));
        assert!(!m("src/**/*.rs", "lib/main.rs"));
        assert!(m("**/test?.go", "x/test1.go"));
    }

    #[test]
    fn braces_expand() {
        assert!(m("*.{rs,go}", "a/b.go"));
        assert!(m("*.{rs,go}", "b.rs"));
        assert!(!m("*.{rs,go}", "b.py"));
        assert!(m("{src,lib}/**/x.rs", "lib/a/x.rs"));
        assert_eq!(expand_braces("a{b,c{d,e}}f"), vec!["abf", "acdf", "acef"]);
        assert_eq!(expand_braces("no{braces"), vec!["no{braces"]);
    }
}
