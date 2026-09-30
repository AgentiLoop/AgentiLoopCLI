//! Undo journal for file changes made by `write_file`, `edit_file` and `apply_patch`.
//!
//! Before a tool touches a file it records the file's previous content (or that it did not exist).
//! Changes are grouped per turn (one user prompt); `/undo` restores the most recent turn that changed
//! anything. Changes made by shell commands (`bash`) are not tracked.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Turns kept; older ones are forgotten.
const MAX_TURNS: usize = 20;

#[derive(Default)]
struct Turn {
    /// (path, previous content; None = the file did not exist), first snapshot per path only.
    files: Vec<(PathBuf, Option<Vec<u8>>)>,
}

#[derive(Default)]
struct Journal {
    turns: Vec<Turn>,
}

impl Journal {
    fn begin_turn(&mut self) {
        if self.turns.last().is_some_and(|t| t.files.is_empty()) {
            return;
        }
        self.turns.push(Turn::default());
        if self.turns.len() > MAX_TURNS {
            self.turns.remove(0);
        }
    }

    fn record(&mut self, path: &Path) {
        if self.turns.is_empty() {
            self.turns.push(Turn::default());
        }
        let turn = self.turns.last_mut().unwrap();
        if turn.files.iter().any(|(p, _)| p == path) {
            return;
        }
        turn.files.push((path.to_path_buf(), std::fs::read(path).ok()));
    }

    fn undo_last(&mut self) -> Option<Vec<String>> {
        while let Some(turn) = self.turns.pop() {
            if turn.files.is_empty() {
                continue;
            }
            let mut lines = Vec::new();
            for (path, before) in turn.files.into_iter().rev() {
                let shown = path.display();
                match before {
                    Some(bytes) => {
                        if let Some(dir) = path.parent() {
                            let _ = std::fs::create_dir_all(dir);
                        }
                        match std::fs::write(&path, bytes) {
                            Ok(()) => lines.push(format!("restored {shown}")),
                            Err(e) => lines.push(format!("could not restore {shown}: {e}")),
                        }
                    }
                    None => match std::fs::remove_file(&path) {
                        Ok(()) => lines.push(format!("removed {shown} (it did not exist before)")),
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                        Err(e) => lines.push(format!("could not remove {shown}: {e}")),
                    },
                }
            }
            return Some(lines);
        }
        None
    }
}

static JOURNAL: Mutex<Journal> = Mutex::new(Journal { turns: Vec::new() });

fn lock() -> std::sync::MutexGuard<'static, Journal> {
    JOURNAL.lock().unwrap_or_else(|e| e.into_inner())
}

/// Starts a new turn; call once per user prompt. A previous turn that recorded nothing is reused.
pub fn begin_turn() {
    lock().begin_turn();
}

/// Remembers `path`'s current state before it is changed. Call before the first write in a turn.
pub fn record(path: &Path) {
    lock().record(path);
}

/// Restores every file changed in the latest turn that changed anything, newest change first.
/// Returns one line per file, or `None` when there is nothing to undo.
pub fn undo_last() -> Option<Vec<String>> {
    lock().undo_last()
}

/// Forgets everything (used by `/clear`).
pub fn clear() {
    lock().turns.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    // Uses a private journal: the global one is shared with the other tests in this process.
    #[test]
    fn undo_restores_modified_created_and_deleted_files_per_turn() {
        let d = std::env::temp_dir().join(format!("agentiloop-undo-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let (a, b, c) = (d.join("a.txt"), d.join("sub/b.txt"), d.join("c.txt"));
        std::fs::write(&a, "orig").unwrap();
        std::fs::write(&c, "keep me").unwrap();
        let mut j = Journal::default();
        assert!(j.undo_last().is_none());

        // Turn 1: modify a twice (only the first snapshot counts), create b.
        j.begin_turn();
        j.record(&a);
        std::fs::write(&a, "v1").unwrap();
        j.record(&a);
        std::fs::write(&a, "v2").unwrap();
        j.record(&b);
        std::fs::create_dir_all(d.join("sub")).unwrap();
        std::fs::write(&b, "new").unwrap();

        // Turn 2: delete c. A turn that records nothing is skipped by undo.
        j.begin_turn();
        j.record(&c);
        std::fs::remove_file(&c).unwrap();
        j.begin_turn();
        j.begin_turn();

        let lines = j.undo_last().unwrap();
        assert_eq!(lines, vec![format!("restored {}", c.display())]);
        assert_eq!(std::fs::read_to_string(&c).unwrap(), "keep me");
        assert_eq!(std::fs::read_to_string(&a).unwrap(), "v2");

        let lines = j.undo_last().unwrap();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("removed ") && lines[1].starts_with("restored "), "{lines:?}");
        assert_eq!(std::fs::read_to_string(&a).unwrap(), "orig");
        assert!(!b.exists());
        assert!(j.undo_last().is_none());
        let _ = std::fs::remove_dir_all(&d);
    }
}
