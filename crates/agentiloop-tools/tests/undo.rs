//! The file tools feed the undo journal. Own test binary: the journal is process-global.

use agentiloop_core::{Tool, ToolContext};
use agentiloop_tools::{undo, ApplyPatch, EditFile, WriteFile};
use serde_json::json;

#[tokio::test]
async fn file_tools_are_undoable_per_turn() {
    let d = std::env::temp_dir().join(format!("agentiloop-undo-tools-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    let ctx = ToolContext { cwd: d.clone() };
    std::fs::write(d.join("a.txt"), "one\ntwo\n").unwrap();
    std::fs::write(d.join("gone.txt"), "bye\n").unwrap();

    undo::begin_turn();
    WriteFile.call(&ctx, json!({"path": "new/n.txt", "content": "hi"})).await.unwrap();
    EditFile.call(&ctx, json!({"path": "a.txt", "old_string": "two", "new_string": "2"})).await.unwrap();

    undo::begin_turn();
    let patch = "*** Begin Patch\n*** Update File: a.txt\n@@\n one\n-2\n+TWO\n*** Delete File: gone.txt\n*** End Patch";
    ApplyPatch.call(&ctx, json!({"patch": patch})).await.unwrap();
    assert_eq!(std::fs::read_to_string(d.join("a.txt")).unwrap(), "one\nTWO\n");
    assert!(!d.join("gone.txt").exists());

    // Undo the patch turn: a.txt back to the edited text, gone.txt back.
    let lines = undo::undo_last().unwrap();
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert_eq!(std::fs::read_to_string(d.join("a.txt")).unwrap(), "one\n2\n");
    assert_eq!(std::fs::read_to_string(d.join("gone.txt")).unwrap(), "bye\n");

    // Undo the first turn: a.txt original, new file removed.
    undo::undo_last().unwrap();
    assert_eq!(std::fs::read_to_string(d.join("a.txt")).unwrap(), "one\ntwo\n");
    assert!(!d.join("new/n.txt").exists());
    assert!(undo::undo_last().is_none());
    let _ = std::fs::remove_dir_all(&d);
}
