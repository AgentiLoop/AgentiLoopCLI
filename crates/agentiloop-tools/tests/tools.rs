//! Built-in tool tests against a scratch directory.

use std::path::PathBuf;

use agentiloop_core::{Tool, ToolContext, ToolError};
use agentiloop_tools::{default_registry, Bash, EditFile, GlobFiles, Grep, ListDir, ReadFile, WebFetch, WriteFile};
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
    assert_eq!(r.len(), 8);
    for name in ["read_file", "write_file", "edit_file", "list_dir", "glob", "grep", "web_fetch", "bash"] {
        assert!(r.get(name).is_some(), "missing {name}");
    }
    assert!(!r.get("read_file").unwrap().is_mutating());
    for name in ["list_dir", "glob", "grep"] {
        assert!(!r.get(name).unwrap().is_mutating(), "{name} should be read-only");
    }
    for name in ["write_file", "edit_file", "web_fetch", "bash"] {
        assert!(r.get(name).unwrap().is_mutating(), "{name} should be mutating");
    }
}

#[cfg(unix)]
fn alive(pid: &str) -> bool {
    std::process::Command::new("kill").args(["-0", pid]).stderr(std::process::Stdio::null()).status().unwrap().success()
}

#[cfg(unix)]
#[tokio::test]
async fn bash_cancel_kills_the_whole_process_tree() {
    let s = Scratch::new("bash-cancel");
    let pidfile = s.0.join("pid");
    let cmd = format!("sleep 30 & echo $! > {}; wait", pidfile.display());
    // Dropping the call is what Esc does to a running tool.
    let r = tokio::time::timeout(std::time::Duration::from_millis(500), Bash.call(&s.ctx(), json!({"command": cmd}))).await;
    assert!(r.is_err(), "the command should still have been running");
    let pid = std::fs::read_to_string(&pidfile).unwrap();
    let pid = pid.trim();
    for _ in 0..50 {
        if !alive(pid) {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("grandchild {pid} outlived the cancelled command");
}

#[cfg(unix)]
#[tokio::test]
async fn bash_does_not_wait_for_background_children_holding_the_pipes() {
    let s = Scratch::new("bash-background");
    let started = std::time::Instant::now();
    let out = Bash.call(&s.ctx(), json!({"command": "sleep 30 & echo started"})).await.unwrap();
    assert!(out.contains("started"), "{out}");
    assert!(started.elapsed() < std::time::Duration::from_secs(5), "waited {:?}", started.elapsed());
}

fn seed_tree(s: &Scratch) {
    for (rel, body) in [
        ("src/main.rs", "fn main() {\n    println!(\"Hello\");\n}\n"),
        ("src/lib/util.rs", "pub fn hello() {}\n// TODO: later\n"),
        ("README.md", "hello world\n"),
        ("target/debug/junk.rs", "fn hello() {}\n"),
        ("node_modules/x/index.js", "hello\n"),
        (".git/config", "hello\n"),
    ] {
        let p = s.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
    std::fs::write(s.0.join("blob.bin"), b"hello\0\x01\x02").unwrap();
}

#[tokio::test]
async fn glob_finds_files_and_skips_vendor_dirs() {
    let s = Scratch::new("glob");
    seed_tree(&s);
    let out = GlobFiles.call(&s.ctx(), json!({"pattern": "*.rs"})).await.unwrap();
    assert_eq!(out, "src/lib/util.rs\nsrc/main.rs");
    let out = GlobFiles.call(&s.ctx(), json!({"pattern": "src/**/*.{rs,md}"})).await.unwrap();
    assert_eq!(out, "src/lib/util.rs\nsrc/main.rs");
    let out = GlobFiles.call(&s.ctx(), json!({"pattern": "*.rs", "path": "src/lib"})).await.unwrap();
    assert_eq!(out, "src/lib/util.rs");
    let out = GlobFiles.call(&s.ctx(), json!({"pattern": "*.zzz"})).await.unwrap();
    assert_eq!(out, "no matches");
    let err = GlobFiles.call(&s.ctx(), json!({"pattern": "*", "path": "nope"})).await.unwrap_err();
    assert!(matches!(err, ToolError::Failed(_)), "{err:?}");
}

#[tokio::test]
async fn glob_caps_results() {
    let s = Scratch::new("glob-cap");
    for i in 0..510 {
        std::fs::write(s.0.join(format!("f{i:03}.txt")), "").unwrap();
    }
    let out = GlobFiles.call(&s.ctx(), json!({"pattern": "*.txt"})).await.unwrap();
    assert_eq!(out.lines().count(), 501, "{out}");
    assert!(out.contains("truncated"), "{out}");
}

#[tokio::test]
async fn grep_reports_path_line_text_and_skips_binary_and_vendor() {
    let s = Scratch::new("grep");
    seed_tree(&s);
    let out = Grep.call(&s.ctx(), json!({"pattern": "hello"})).await.unwrap();
    assert_eq!(out, "README.md:1:hello world\nsrc/lib/util.rs:1:pub fn hello() {}");
    let out = Grep.call(&s.ctx(), json!({"pattern": "hello", "case_insensitive": true, "glob": "*.rs"})).await.unwrap();
    assert_eq!(out, "src/lib/util.rs:1:pub fn hello() {}\nsrc/main.rs:2:    println!(\"Hello\");");
    let out = Grep.call(&s.ctx(), json!({"pattern": "TODO", "path": "src/lib/util.rs"})).await.unwrap();
    assert_eq!(out, "src/lib/util.rs:2:// TODO: later");
    let out = Grep.call(&s.ctx(), json!({"pattern": "nothing-like-this"})).await.unwrap();
    assert_eq!(out, "no matches");
}

#[tokio::test]
async fn grep_limits_and_validates_input() {
    let s = Scratch::new("grep-limit");
    std::fs::write(s.0.join("a.txt"), "x\nx\nx\n").unwrap();
    let out = Grep.call(&s.ctx(), json!({"pattern": "x", "max_results": 2})).await.unwrap();
    assert!(out.starts_with("a.txt:1:x\na.txt:2:x\n…[truncated at 2"), "{out}");
    let err = Grep.call(&s.ctx(), json!({"pattern": "("})).await.unwrap_err();
    assert!(matches!(err, ToolError::InvalidInput(_)), "{err:?}");
    let err = Grep.call(&s.ctx(), json!({"pattern": "x", "path": "missing"})).await.unwrap_err();
    assert!(matches!(err, ToolError::Failed(_)), "{err:?}");
    let err = Grep.call(&s.ctx(), json!({})).await.unwrap_err();
    assert!(matches!(err, ToolError::InvalidInput(_)), "{err:?}");
}

#[tokio::test]
async fn grep_truncates_long_lines() {
    let s = Scratch::new("grep-long");
    std::fs::write(s.0.join("a.txt"), format!("{}\n", "y".repeat(1000))).unwrap();
    let out = Grep.call(&s.ctx(), json!({"pattern": "y"})).await.unwrap();
    assert_eq!(out, format!("a.txt:1:{}…", "y".repeat(300)));
}

#[tokio::test]
async fn glob_and_grep_honor_nested_gitignore() {
    let s = Scratch::new("gitignore");
    for (rel, body) in [
        (".gitignore", "dist/\n*.log\n!keep.log\n/top.txt\n"),
        ("a.rs", "needle\n"),
        ("dist/out.rs", "needle\n"),
        ("debug.log", "needle\n"),
        ("keep.log", "needle\n"),
        ("top.txt", "needle\n"),
        ("sub/top.txt", "needle\n"),
        ("sub/.gitignore", "local.rs\n"),
        ("sub/local.rs", "needle\n"),
        ("sub/deep/local.rs", "needle\n"),
        ("sub/other.rs", "needle\n"),
        ("other/local.rs", "needle\n"),
    ] {
        let p = s.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
    let out = GlobFiles.call(&s.ctx(), json!({"pattern": "*"})).await.unwrap();
    assert_eq!(out, ".gitignore\na.rs\nkeep.log\nother/local.rs\nsub/.gitignore\nsub/other.rs\nsub/top.txt");
    let out = Grep.call(&s.ctx(), json!({"pattern": "needle"})).await.unwrap();
    assert_eq!(out, "a.rs:1:needle\nkeep.log:1:needle\nother/local.rs:1:needle\nsub/other.rs:1:needle\nsub/top.txt:1:needle");
    // Searching inside an ignored folder on purpose still works: only rules at or below it apply.
    let out = GlobFiles.call(&s.ctx(), json!({"pattern": "*.rs", "path": "dist"})).await.unwrap();
    assert_eq!(out, "dist/out.rs");
}

#[tokio::test]
async fn grep_context_lines_merge_and_separate_groups() {
    let s = Scratch::new("grep-context");
    std::fs::write(s.0.join("a.txt"), "1\n2\nHIT\n4\nHIT\n6\n7\n8\n9\nHIT\n").unwrap();
    std::fs::write(s.0.join("b.txt"), "HIT\nz\n").unwrap();
    let out = Grep.call(&s.ctx(), json!({"pattern": "HIT", "context": 1})).await.unwrap();
    assert_eq!(
        out,
        "a.txt-2-2\na.txt:3:HIT\na.txt-4-4\na.txt:5:HIT\na.txt-6-6\n--\na.txt-9-9\na.txt:10:HIT\n--\nb.txt:1:HIT\nb.txt-2-z"
    );
    let out = Grep.call(&s.ctx(), json!({"pattern": "HIT", "context": 0, "path": "b.txt"})).await.unwrap();
    assert_eq!(out, "b.txt:1:HIT");
}

/// Tiny HTTP server: `/page` HTML, `/data` JSON, `/img` PNG, `/big` 5000 x's, `/go` redirect to /page, anything else 404.
async fn serve() -> String {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        loop {
            let (mut sock, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                let mut buf = vec![0u8; 4096];
                let n = sock.read(&mut buf).await.unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]).into_owned();
                let path = req.split_whitespace().nth(1).unwrap_or("/").to_string();
                let big = "x".repeat(5000);
                let (status, extra, ctype, body): (&str, &str, &str, &str) = match path.as_str() {
                    "/page" => ("200 OK", "", "text/html; charset=utf-8", "<html><body><h1>Hi</h1><script>bad()</script><p>there &amp; back</p></body></html>"),
                    "/data" => ("200 OK", "", "application/json", "{\"a\": 1}"),
                    "/img" => ("200 OK", "", "image/png", "PNG"),
                    "/big" => ("200 OK", "", "text/plain", big.as_str()),
                    "/go" => ("302 Found", "Location: /page\r\n", "text/plain", ""),
                    _ => ("404 Not Found", "", "text/plain", "nope"),
                };
                let resp = format!(
                    "HTTP/1.1 {status}\r\n{extra}Content-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = sock.write_all(resp.as_bytes()).await;
                let _ = sock.shutdown().await;
            });
        }
    });
    format!("http://{addr}")
}

#[tokio::test]
async fn web_fetch_reads_html_json_and_follows_redirects() {
    let s = Scratch::new("web");
    let base = serve().await;
    let out = WebFetch.call(&s.ctx(), json!({"url": format!("{base}/page")})).await.unwrap();
    assert!(out.ends_with("(200 OK)\n\nHi\n\nthere & back"), "{out}");
    let out = WebFetch.call(&s.ctx(), json!({"url": format!("{base}/go")})).await.unwrap();
    assert!(out.ends_with("Hi\n\nthere & back"), "{out}");
    let out = WebFetch.call(&s.ctx(), json!({"url": format!("{base}/data")})).await.unwrap();
    assert!(out.ends_with("\n\n{\"a\": 1}"), "{out}");
}

#[tokio::test]
async fn web_fetch_refuses_bad_input_errors_and_binary_and_truncates() {
    let s = Scratch::new("web-err");
    let base = serve().await;
    for url in ["ftp://example.com/x", "not a url", "file:///etc/passwd"] {
        let err = WebFetch.call(&s.ctx(), json!({"url": url})).await.unwrap_err();
        assert!(matches!(err, ToolError::InvalidInput(_)), "{url}: {err:?}");
    }
    let err = WebFetch.call(&s.ctx(), json!({})).await.unwrap_err();
    assert!(matches!(err, ToolError::InvalidInput(_)), "{err:?}");
    let err = WebFetch.call(&s.ctx(), json!({"url": format!("{base}/missing")})).await.unwrap_err();
    assert!(err.to_string().contains("HTTP 404"), "{err}");
    let err = WebFetch.call(&s.ctx(), json!({"url": format!("{base}/img")})).await.unwrap_err();
    assert!(err.to_string().contains("image/png"), "{err}");
    let out = WebFetch.call(&s.ctx(), json!({"url": format!("{base}/big"), "max_chars": 100})).await.unwrap();
    assert!(out.ends_with(&format!("{}\n…[truncated at 100 characters]", "x".repeat(100))), "{out}");
    let err = WebFetch.call(&s.ctx(), json!({"url": "http://127.0.0.1:1/"})).await.unwrap_err();
    assert!(matches!(err, ToolError::Failed(_)), "{err:?}");
}
