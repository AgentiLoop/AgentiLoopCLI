use std::path::PathBuf;

use agentiloop_core::{ContentBlock, Message, Role, Session};
use serde_json::json;

struct Scratch(PathBuf);
impl Scratch {
    fn new(name: &str) -> Self {
        let d = std::env::temp_dir().join(format!("agentiloop-sessions-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        Self(d)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn sample_history() -> Vec<Message> {
    vec![
        Message::user_text("fix the failing test\nplease"),
        Message {
            role: Role::Assistant,
            content: vec![ContentBlock::ToolUse { id: "t1".into(), name: "bash".into(), input: json!({"command": "cargo test"}) }],
        },
        Message::tool_results(vec![ContentBlock::ToolResult { tool_use_id: "t1".into(), content: "ok".into(), is_error: false }]),
        Message { role: Role::Assistant, content: vec![ContentBlock::Text { text: "done".into() }] },
    ]
}

#[test]
fn save_then_load_round_trips_history_and_metadata() {
    let s = Scratch::new("roundtrip");
    let mut sess = Session::new("/proj", "openai", "qwen3:4b");
    sess.history = sample_history();
    let path = sess.save(&s.0).unwrap();
    assert!(path.ends_with(format!("{}.json", sess.id)));
    assert!(!s.0.join(format!(".{}.json.tmp", sess.id)).exists(), "temp file must be renamed away");

    let back = Session::load(&s.0, &sess.id).unwrap();
    assert_eq!(back.id, sess.id);
    assert_eq!(back.cwd, PathBuf::from("/proj"));
    assert_eq!(back.provider, "openai");
    assert_eq!(back.model, "qwen3:4b");
    assert_eq!(back.history.len(), 4);
    assert_eq!(back.history[1].tool_uses().count(), 1);
    assert_eq!(back.history[3].text(), "done");
}

#[test]
fn title_is_first_user_line_truncated() {
    let mut sess = Session::new("/p", "a", "m");
    assert_eq!(sess.title(), "");
    sess.history = sample_history();
    assert_eq!(sess.title(), "fix the failing test");
    sess.history = vec![Message::user_text("x".repeat(100))];
    let t = sess.title();
    assert_eq!(t.chars().count(), 61);
    assert!(t.ends_with('…'));
}

#[test]
fn list_is_newest_first_and_skips_junk() {
    let s = Scratch::new("list");
    let mut older = Session::new("/a", "p", "m");
    older.history = vec![Message::user_text("older")];
    older.save(&s.0).unwrap();
    // force distinct ordering without sleeping
    let mut newer = Session::new("/b", "p", "m");
    newer.id = "newer".into();
    newer.history = vec![Message::user_text("newer")];
    newer.save(&s.0).unwrap();
    newer.updated = older.updated + 10;
    std::fs::write(Session::path(&s.0, "newer"), serde_json::to_vec(&newer).unwrap()).unwrap();

    std::fs::write(s.0.join("garbage.json"), "{not json").unwrap();
    std::fs::write(s.0.join("notes.txt"), "ignored").unwrap();
    std::fs::write(s.0.join(".hidden.json.tmp"), "{}").unwrap();

    let all = Session::list(&s.0).unwrap();
    let ids: Vec<_> = all.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, vec!["newer", older.id.as_str()]);
}

#[test]
fn list_on_missing_dir_is_empty() {
    let s = Scratch::new("missing");
    assert!(Session::list(&s.0).unwrap().is_empty());
}

#[test]
fn latest_for_matches_cwd() {
    let s = Scratch::new("latest");
    let mut a = Session::new("/a", "p", "m");
    a.save(&s.0).unwrap();
    let mut b = Session::new("/b", "p", "m");
    b.id = "b".into();
    b.save(&s.0).unwrap();
    assert_eq!(Session::latest_for(&s.0, &PathBuf::from("/a")).unwrap().unwrap().id, a.id);
    assert_eq!(Session::latest_for(&s.0, &PathBuf::from("/b")).unwrap().unwrap().id, "b");
    assert!(Session::latest_for(&s.0, &PathBuf::from("/zzz")).unwrap().is_none());
}

#[test]
fn load_missing_session_errors_with_id() {
    let s = Scratch::new("load-missing");
    let err = Session::load(&s.0, "nope").unwrap_err().to_string();
    assert!(err.contains("no session nope"), "{err}");
}
