//! Built-in tool tests against a scratch directory.

use std::path::PathBuf;

use agentiloop_core::{Tool, ToolContext, ToolError};
use agentiloop_tools::{default_registry, Bash, EditFile, ListDir, ReadFile, WriteFile};
use serde_json::json;

/// Fresh temp dir per test; removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("agentiloop-tools-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
    fn ctx(&self) -> ToolContext {
        ToolContext { cwd: self.0.clone() }
    }
    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.0.join(rel)).unwrap()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test]
async fn write_then_read_with_line_numbers() {
    let s = Scratch::new("write-read");
    let out = WriteFile.call(&s.ctx(), json!({"path": "sub/a.txt", "content": "one\ntwo\n"})).await.unwrap();
    assert!(out.starts_with("wrote 8 bytes"), "{out}");
    assert_eq!(s.read("sub/a.txt"), "one\ntwo\n");

    let out = ReadFile.call(&s.ctx(), json!({"path": "sub/a.txt"})).await.unwrap();
    assert_eq!(out, "    1\u{2502}one\n    2\u{2502}two");
}

#[tokio::test]
async fn read_missing_file_fails() {
    let s = Scratch::new("read-missing");
    let err = ReadFile.call(&s.ctx(), json!({"path": "nope.txt"})).await.unwrap_err();
    assert!(matches!(err, ToolError::Failed(_)), "{err:?}");
}

#[tokio::test]
async fn read_rejects_bad_input() {
    let s = Scratch::new("read-bad");
    let err = ReadFile.call(&s.ctx(), json!({"nope": 1})).await.unwrap_err();
    assert!(matches!(err, ToolError::InvalidInput(_)), "{err:?}");
}

#[tokio::test]
async fn absolute_paths_are_not_joined_to_cwd() {
    let s = Scratch::new("abs");
    let abs = s.0.join("abs.txt");
    WriteFile.call(&s.ctx(), json!({"path": abs.to_str().unwrap(), "content": "x"})).await.unwrap();
    assert_eq!(s.read("abs.txt"), "x");
}

#[tokio::test]
async fn edit_replaces_single_match() {
    let s = Scratch::new("edit-one");
    std::fs::write(s.0.join("f.rs"), "fn a() {}\nfn b() {}\n").unwrap();
    let out = EditFile
        .call(&s.ctx(), json!({"path": "f.rs", "old_string": "fn b()", "new_string": "fn c()"}))
        .await
        .unwrap();
    assert!(out.starts_with("replaced 1 occurrence"), "{out}");
    assert_eq!(s.read("f.rs"), "fn a() {}\nfn c() {}\n");
}

#[tokio::test]
async fn edit_refuses_ambiguous_match_unless_replace_all() {
    let s = Scratch::new("edit-ambiguous");
    std::fs::write(s.0.join("f.txt"), "x x x").unwrap();
    let err = EditFile
        .call(&s.ctx(), json!({"path": "f.txt", "old_string": "x", "new_string": "y"}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("matched 3 times"), "{err}");
    assert_eq!(s.read("f.txt"), "x x x", "file must be untouched on refusal");

    EditFile
        .call(&s.ctx(), json!({"path": "f.txt", "old_string": "x", "new_string": "y", "replace_all": true}))
        .await
        .unwrap();
    assert_eq!(s.read("f.txt"), "y y y");
}

#[tokio::test]
async fn edit_reports_missing_old_string() {
    let s = Scratch::new("edit-missing");
    std::fs::write(s.0.join("f.txt"), "abc").unwrap();
    let err = EditFile
        .call(&s.ctx(), json!({"path": "f.txt", "old_string": "zzz", "new_string": "y"}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("not found"), "{err}");
}

#[tokio::test]
async fn list_dir_sorts_and_marks_directories() {
    let s = Scratch::new("list");
    std::fs::create_dir(s.0.join("dir")).unwrap();
    std::fs::write(s.0.join("b.txt"), "").unwrap();
    std::fs::write(s.0.join("a.txt"), "").unwrap();
    let out = ListDir.call(&s.ctx(), json!({})).await.unwrap();
    assert_eq!(out, "a.txt\nb.txt\ndir/");
    let out = ListDir.call(&s.ctx(), json!({"path": "dir"})).await.unwrap();
    assert_eq!(out, "");
}

#[tokio::test]
async fn bash_runs_in_cwd_and_captures_output() {
    let s = Scratch::new("bash");
    std::fs::write(s.0.join("marker"), "").unwrap();
    let out = Bash.call(&s.ctx(), json!({"command": "ls && echo err 1>&2"})).await.unwrap();
    assert!(out.starts_with("marker\n"), "{out:?}");
    assert!(out.contains("err"), "{out:?}");
    assert!(!out.contains("[exit status"), "{out:?}");
}

#[tokio::test]
async fn bash_reports_nonzero_exit() {
    let s = Scratch::new("bash-exit");
    let out = Bash.call(&s.ctx(), json!({"command": "exit 3"})).await.unwrap();
    assert!(out.ends_with("[exit status: 3]"), "{out}");
}

#[tokio::test]
async fn bash_times_out() {
    let s = Scratch::new("bash-timeout");
    let err = Bash.call(&s.ctx(), json!({"command": "sleep 5", "timeout_secs": 1})).await.unwrap_err();
    assert!(err.to_string().contains("timed out after 1s"), "{err}");
}

#[test]
fn default_registry_has_all_builtins() {
    let r = default_registry();
    assert_eq!(r.len(), 5);
    for name in ["read_file", "write_file", "edit_file", "list_dir", "bash"] {
        assert!(r.get(name).is_some(), "missing {name}");
    }
    assert!(!r.get("read_file").unwrap().is_mutating());
    assert!(!r.get("list_dir").unwrap().is_mutating());
    for name in ["write_file", "edit_file", "bash"] {
        assert!(r.get(name).unwrap().is_mutating(), "{name} should be mutating");
    }
}
