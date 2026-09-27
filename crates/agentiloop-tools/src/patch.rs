//! `apply_patch`: the Codex patch grammar GPT models are trained on.
//!
//! ```text
//! *** Begin Patch
//! *** Add File: path        (+ lines = new content)
//! *** Delete File: path
//! *** Update File: path
//! *** Move to: new/path     (optional rename)
//! @@ optional anchor line
//!  context
//! -removed
//! +added
//! *** End Patch
//! ```
//!
//! Every change is worked out in memory first; nothing is written unless the
//! whole patch applies.

use std::path::{Path, PathBuf};

use agentiloop_core::{Tool, ToolContext, ToolError, ToolResult};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

pub struct ApplyPatch;

#[derive(Deserialize)]
struct PatchArgs {
    patch: String,
}

#[async_trait]
impl Tool for ApplyPatch {
    fn name(&self) -> &str { "apply_patch" }
    fn description(&self) -> &str {
        "Apply a patch in Codex's format: '*** Begin Patch', then '*** Add File: <path>' (+ lines), \
         '*** Delete File: <path>', or '*** Update File: <path>' (optional '*** Move to: <path>', then \
         hunks starting with '@@' made of ' ' context, '-' removed and '+' added lines), then '*** End Patch'. \
         Prefer it for multi-hunk or multi-file edits. All-or-nothing."
    }
    fn input_schema(&self) -> Value {
        json!({"type":"object","properties":{"patch":{"type":"string","description":"The full patch, '*** Begin Patch' to '*** End Patch'."}},"required":["patch"]})
    }
    fn is_mutating(&self) -> bool { true }
    async fn call(&self, ctx: &ToolContext, input: Value) -> ToolResult {
        let a: PatchArgs = serde_json::from_value(input).map_err(|e| ToolError::InvalidInput(e.to_string()))?;
        apply(&a.patch, &ctx.cwd).map_err(ToolError::Failed)
    }
}

/// One file change, fully resolved before anything touches the disk.
enum Change {
    Write { path: PathBuf, text: String, from: Option<PathBuf> },
    Delete(PathBuf),
}

pub fn apply(patch: &str, cwd: &Path) -> Result<String, String> {
    let resolve = |p: &str| {
        let p = Path::new(p.trim());
        if p.is_absolute() { p.to_path_buf() } else { cwd.join(p) }
    };
    let lines: Vec<&str> = patch.lines().collect();
    let mut i = lines
        .iter()
        .position(|l| l.trim() == "*** Begin Patch")
        .ok_or("patch must start with '*** Begin Patch'")?
        + 1;
    let mut changes = Vec::new();
    let mut summary = Vec::new();
    let is_header = |l: &str| l.starts_with("*** ") && l.trim() != "*** End of File";

    while i < lines.len() && lines[i].trim() != "*** End Patch" {
        let line = lines[i];
        i += 1;
        if let Some(p) = line.strip_prefix("*** Add File: ") {
            let mut body = Vec::new();
            while i < lines.len() && !is_header(lines[i]) {
                body.push(lines[i].strip_prefix('+').unwrap_or(lines[i]));
                i += 1;
            }
            let mut text = body.join("\n");
            text.push('\n');
            summary.push(format!("A {}", p.trim()));
            changes.push(Change::Write { path: resolve(p), text, from: None });
        } else if let Some(p) = line.strip_prefix("*** Delete File: ") {
            let path = resolve(p);
            if !path.is_file() {
                return Err(format!("cannot delete {}: no such file", p.trim()));
            }
            summary.push(format!("D {}", p.trim()));
            changes.push(Change::Delete(path));
        } else if let Some(p) = line.strip_prefix("*** Update File: ") {
            let src = resolve(p);
            let old = std::fs::read_to_string(&src).map_err(|e| format!("cannot read {}: {e}", p.trim()))?;
            let mut dest = None;
            if let Some(to) = lines.get(i).and_then(|l| l.strip_prefix("*** Move to: ")) {
                dest = Some(to.trim().to_string());
                i += 1;
            }
            let start = i;
            while i < lines.len() && !is_header(lines[i]) {
                i += 1;
            }
            let text = update(&old, &lines[start..i]).map_err(|e| format!("{}: {e}", p.trim()))?;
            match dest {
                Some(to) => {
                    summary.push(format!("R {} -> {to}", p.trim()));
                    changes.push(Change::Write { path: resolve(&to), text, from: Some(src) });
                }
                None => {
                    summary.push(format!("M {}", p.trim()));
                    changes.push(Change::Write { path: src, text, from: None });
                }
            }
        } else if !line.trim().is_empty() {
            return Err(format!("unexpected line in patch: {line}"));
        }
    }
    if changes.is_empty() {
        return Err("patch contains no file changes".into());
    }

    for c in changes {
        match c {
            Change::Write { path, text, from } => {
                if let Some(dir) = path.parent() {
                    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
                }
                std::fs::write(&path, text).map_err(|e| format!("writing {}: {e}", path.display()))?;
                if let Some(from) = from.filter(|f| *f != path) {
                    std::fs::remove_file(&from).map_err(|e| format!("removing {}: {e}", from.display()))?;
                }
            }
            Change::Delete(path) => std::fs::remove_file(&path).map_err(|e| format!("deleting {}: {e}", path.display()))?,
        }
    }
    Ok(summary.join("\n"))
}

/// Apply `@@` hunks in order: each hunk's context + `-` lines must appear (in
/// order, at or after the previous hunk) and are replaced by context + `+` lines.
/// An `@@ text` header first moves past the line containing `text`.
fn update(old: &str, body: &[&str]) -> Result<String, String> {
    let mut file: Vec<String> = old.lines().map(str::to_string).collect();
    let mut cursor = 0;
    // Each hunk: optional anchor, then (op, text) lines with op ' ', '-' or '+'.
    let mut hunks: Vec<(Option<&str>, Vec<(char, &str)>)> = Vec::new();
    for l in body {
        if let Some(anchor) = l.strip_prefix("@@") {
            let anchor = anchor.trim();
            hunks.push(((!anchor.is_empty()).then_some(anchor), Vec::new()));
            continue;
        }
        if l.trim() == "*** End of File" {
            continue;
        }
        if hunks.is_empty() {
            hunks.push((None, Vec::new()));
        }
        let h = &mut hunks.last_mut().unwrap().1;
        match l.chars().next() {
            Some(op @ ('+' | '-' | ' ')) => h.push((op, &l[1..])),
            // A bare empty line is an empty context line.
            None => h.push((' ', "")),
            _ => return Err(format!("bad hunk line (needs ' ', '-' or '+' prefix): {l}")),
        }
    }
    for (anchor, ops) in hunks {
        if let Some(a) = anchor {
            let at = file[cursor..].iter().position(|l| l.contains(a)).ok_or_else(|| format!("anchor not found: {a}"))?;
            // Hunk context usually repeats the anchor line itself, so search from it.
            cursor += at;
        }
        let before: Vec<&str> = ops.iter().filter(|(op, _)| *op != '+').map(|(_, t)| *t).collect();
        let at = if before.is_empty() {
            file.len()
        } else {
            find(&file, &before, cursor).ok_or_else(|| format!("hunk does not match:\n{}", before.join("\n")))?
        };
        // Context lines keep the file's own text (it may differ in trailing whitespace).
        let mut j = at;
        let mut after = Vec::new();
        for (op, t) in &ops {
            match op {
                ' ' => {
                    after.push(file[j].clone());
                    j += 1;
                }
                '-' => j += 1,
                _ => after.push(t.to_string()),
            }
        }
        cursor = at + after.len();
        file.splice(at..j, after);
    }
    let mut out = file.join("\n");
    if !file.is_empty() {
        out.push('\n');
    }
    Ok(out)
}

/// First index at or after `from` where `needle` matches; exact first, then ignoring trailing whitespace.
fn find(hay: &[String], needle: &[&str], from: usize) -> Option<usize> {
    let fits = |eq: &dyn Fn(&str, &str) -> bool| {
        (from..=hay.len().saturating_sub(needle.len())).find(|&s| needle.iter().enumerate().all(|(k, n)| eq(&hay[s + k], n)))
    };
    fits(&|a, b| a == b).or_else(|| fits(&|a, b| a.trim_end() == b.trim_end()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> PathBuf {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let d = std::env::temp_dir().join(format!("agentiloop-patch-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn add_update_delete_and_move() {
        let d = tmp();
        std::fs::write(d.join("a.txt"), "one\ntwo\nthree\nfour\n").unwrap();
        std::fs::write(d.join("gone.txt"), "x\n").unwrap();
        std::fs::write(d.join("old.txt"), "keep\nfix me\n").unwrap();
        let patch = "*** Begin Patch
*** Add File: sub/new.txt
+hello
+world
*** Update File: a.txt
@@
 one
-two
+TWO
 three
@@
-four
+FOUR
*** Delete File: gone.txt
*** Update File: old.txt
*** Move to: moved.txt
@@
 keep
-fix me
+fixed
*** End Patch";
        let out = apply(patch, &d).unwrap();
        assert_eq!(out, "A sub/new.txt\nM a.txt\nD gone.txt\nR old.txt -> moved.txt");
        assert_eq!(std::fs::read_to_string(d.join("sub/new.txt")).unwrap(), "hello\nworld\n");
        assert_eq!(std::fs::read_to_string(d.join("a.txt")).unwrap(), "one\nTWO\nthree\nFOUR\n");
        assert!(!d.join("gone.txt").exists() && !d.join("old.txt").exists());
        assert_eq!(std::fs::read_to_string(d.join("moved.txt")).unwrap(), "keep\nfixed\n");
    }

    #[test]
    fn anchor_picks_the_right_occurrence() {
        let d = tmp();
        std::fs::write(d.join("f.rs"), "fn a() {\n    x();\n}\nfn b() {\n    x();\n}\n").unwrap();
        apply("*** Begin Patch\n*** Update File: f.rs\n@@ fn b() {\n-    x();\n+    y();\n*** End Patch", &d).unwrap();
        assert_eq!(std::fs::read_to_string(d.join("f.rs")).unwrap(), "fn a() {\n    x();\n}\nfn b() {\n    y();\n}\n");
    }

    #[test]
    fn mismatch_writes_nothing() {
        let d = tmp();
        std::fs::write(d.join("a.txt"), "one\n").unwrap();
        let patch = "*** Begin Patch\n*** Add File: b.txt\n+b\n*** Update File: a.txt\n@@\n-nope\n+x\n*** End Patch";
        let err = apply(patch, &d).unwrap_err();
        assert!(err.contains("a.txt: hunk does not match"), "{err}");
        assert!(!d.join("b.txt").exists(), "earlier Add File must not be written");
        assert_eq!(std::fs::read_to_string(d.join("a.txt")).unwrap(), "one\n");
    }

    #[test]
    fn rejects_malformed_patches() {
        let d = tmp();
        assert!(apply("no header", &d).unwrap_err().contains("Begin Patch"));
        assert!(apply("*** Begin Patch\n*** End Patch", &d).unwrap_err().contains("no file changes"));
        assert!(apply("*** Begin Patch\n*** Delete File: nope\n*** End Patch", &d).unwrap_err().contains("no such file"));
        assert!(apply("*** Begin Patch\n*** Update File: nope\n*** End Patch", &d).unwrap_err().contains("cannot read nope"));
    }

    #[test]
    fn trailing_whitespace_is_tolerated_when_matching() {
        let d = tmp();
        std::fs::write(d.join("a.txt"), "a  \nb\n").unwrap();
        apply("*** Begin Patch\n*** Update File: a.txt\n@@\n a\n-b\n+c\n*** End Patch", &d).unwrap();
        assert_eq!(std::fs::read_to_string(d.join("a.txt")).unwrap(), "a  \nc\n");
    }
}
