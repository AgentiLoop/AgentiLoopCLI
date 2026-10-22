//! Undo journal for file changes made by `write_file`, `edit_file` and `apply_patch`.
//!
//! Before a tool touches a file it records the file's previous content (or that it did not exist).
//! Changes are grouped per turn (one user prompt); `/undo` restores the most recent turn that changed
//! anything. Interactive prompts also remember where they start in the conversation, so `/rewind`
//! can restore their files and cut the conversation back to before one of them. Changes made by
//! shell commands (`bash`) are not tracked.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Turns kept; older ones are forgotten.
const MAX_TURNS: usize = 50;

/// Where an interactive prompt starts: the agent's history epoch (bumped when the history is
/// replaced by /clear, /resume or compaction), its index in the history, and the prompt text.
struct Mark {
    epoch: u64,
    index: usize,
    prompt: String,
}

#[derive(Default)]
struct Turn {
    /// (path, previous content; None = the file did not exist), first snapshot per path only.
    files: Vec<(PathBuf, Option<Vec<u8>>)>,
    mark: Option<Mark>,
}

/// A prompt `/rewind` can go back to: its history index, text and how many files it changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    pub index: usize,
    pub text: String,
    pub files: usize,
}

#[derive(Default)]
struct Journal {
    turns: Vec<Turn>,
}

impl Journal {
    fn push(&mut self, turn: Turn) {
        self.turns.push(turn);
        if self.turns.len() > MAX_TURNS {
            self.turns.remove(0);
        }
    }

    fn begin_turn(&mut self) {
        if self.turns.last().is_some_and(|t| t.files.is_empty() && t.mark.is_none()) {
            return;
        }
        self.push(Turn::default());
    }

    fn begin_prompt(&mut self, epoch: u64, index: usize, prompt: &str) {
        self.push(Turn { files: Vec::new(), mark: Some(Mark { epoch, index, prompt: prompt.to_string() }) });
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

    /// Restores the latest turn that changed anything. The turn itself stays (with no files),
    /// so its prompt can still be rewound to.
    fn undo_last(&mut self) -> Option<Vec<String>> {
        let turn = self.turns.iter_mut().rev().find(|t| !t.files.is_empty())?;
        Some(restore(std::mem::take(&mut turn.files)))
    }

    fn prompts(&self, epoch: u64) -> Vec<Prompt> {
        self.turns
            .iter()
            .filter_map(|t| t.mark.as_ref().filter(|m| m.epoch == epoch).map(|m| Prompt { index: m.index, text: m.prompt.clone(), files: t.files.len() }))
            .collect()
    }

    /// Restores the files of every turn from the prompt at `index` (in `epoch`) on, newest first,
    /// and forgets those turns.
    fn rewind(&mut self, epoch: u64, index: usize) -> Vec<String> {
        let Some(start) = self.turns.iter().position(|t| t.mark.as_ref().is_some_and(|m| m.epoch == epoch && m.index == index)) else {
            return Vec::new();
        };
        self.turns.drain(start..).rev().flat_map(|t| restore(t.files)).collect()
    }
}

/// Puts each file back the way it was before the turn, newest change first.
fn restore(files: Vec<(PathBuf, Option<Vec<u8>>)>) -> Vec<String> {
    let mut lines = Vec::new();
    for (path, before) in files.into_iter().rev() {
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
    lines
}

static JOURNAL: Mutex<Journal> = Mutex::new(Journal { turns: Vec::new() });

fn lock() -> std::sync::MutexGuard<'static, Journal> {
    JOURNAL.lock().unwrap_or_else(|e| e.into_inner())
}

/// Starts a new turn; call once per user prompt. A previous turn that recorded nothing is reused.
pub fn begin_turn() {
    lock().begin_turn();
}

/// Starts a new turn for an interactive prompt that `/rewind` can go back to: `index` is the
/// agent's history length before the prompt and `epoch` the agent's history epoch.
pub fn begin_prompt(epoch: u64, index: usize, prompt: &str) {
    lock().begin_prompt(epoch, index, prompt);
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

/// The prompts of history `epoch` that `/rewind` can go back to, oldest first.
pub fn prompts(epoch: u64) -> Vec<Prompt> {
    lock().prompts(epoch)
}

/// Restores the files changed by the prompt at history `index` (in `epoch`) and every later one,
/// newest first, and forgets those prompts. Returns one line per file.
pub fn rewind(epoch: u64, index: usize) -> Vec<String> {
    lock().rewind(epoch, index)
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

    #[test]
    fn rewind_restores_the_prompt_and_every_later_one_within_its_epoch() {
        let d = std::env::temp_dir().join(format!("agentiloop-rewind-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let (a, b) = (d.join("a.txt"), d.join("b.txt"));
        std::fs::write(&a, "orig").unwrap();
        let mut j = Journal::default();

        j.begin_prompt(0, 0, "old conversation");
        j.record(&a);
        std::fs::write(&a, "old").unwrap();
        // Compaction: the history is replaced, so a new epoch starts.
        j.begin_prompt(1, 2, "edit a");
        j.record(&a);
        std::fs::write(&a, "v1").unwrap();
        j.begin_prompt(1, 6, "just a question");
        j.begin_prompt(1, 8, "create b");
        j.record(&b);
        std::fs::write(&b, "new").unwrap();

        let p = j.prompts(1);
        assert_eq!(p.iter().map(|p| (p.index, p.text.as_str(), p.files)).collect::<Vec<_>>(), vec![(2, "edit a", 1), (6, "just a question", 0), (8, "create b", 1)]);
        assert_eq!(j.prompts(0).len(), 1);

        // /undo empties the newest turn but keeps its prompt for /rewind.
        assert_eq!(j.undo_last().unwrap().len(), 1);
        assert!(!b.exists());
        assert_eq!(j.prompts(1).len(), 3);

        assert!(j.rewind(1, 99).is_empty(), "unknown index is a no-op");
        let lines = j.rewind(1, 2);
        assert_eq!(lines, vec![format!("restored {}", a.display())]);
        assert_eq!(std::fs::read_to_string(&a).unwrap(), "old", "the earlier epoch's change stays");
        assert!(j.prompts(1).is_empty());
        assert_eq!(j.prompts(0).len(), 1);
        let _ = std::fs::remove_dir_all(&d);
    }
}
