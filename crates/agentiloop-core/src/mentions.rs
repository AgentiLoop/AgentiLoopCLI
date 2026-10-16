//! `@path` mentions: a word starting with `@` in a prompt that names an existing file or folder
//! (relative to the project folder, or absolute) attaches it to the prompt the model receives —
//! the file's text with line numbers, or the folder's entries. The typed text is kept as is;
//! the attachments follow it. Words that name nothing (`@alice`, an e-mail address) are left alone.

use std::path::{Path, PathBuf};

/// Files longer than this are cut (bytes).
pub const FILE_LIMIT: usize = 64 * 1024;
/// Folders list at most this many entries.
pub const DIR_LIMIT: usize = 200;

/// One attached path, for the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    /// As typed, without the `@`.
    pub mention: String,
    pub path: PathBuf,
    /// Lines of a file, entries of a folder.
    pub lines: usize,
    pub is_dir: bool,
}

/// Candidate mentions in `text`: `@` at the start or after whitespace, then non-whitespace,
/// with trailing punctuation trimmed. Duplicates are kept once, in order of first appearance.
pub fn candidates(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        let word = word.trim_start_matches(|c: char| matches!(c, '(' | '[' | '{' | '"' | '\'' | '<'));
        let Some(rest) = word.strip_prefix('@') else { continue };
        let rest = rest.trim_end_matches(|c: char| matches!(c, ',' | '.' | ';' | ':' | ')' | ']' | '}' | '"' | '\'' | '?' | '!' | '>'));
        if rest.is_empty() || rest.starts_with('@') || out.iter().any(|o| o == rest) {
            continue;
        }
        out.push(rest.to_string());
    }
    out
}

fn resolve(cwd: &Path, mention: &str) -> PathBuf {
    let p = Path::new(mention);
    if p.is_absolute() {
        p.to_path_buf()
    } else if let Some(rest) = mention.strip_prefix("~/") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(rest)).unwrap_or_else(|| cwd.join(mention))
    } else {
        cwd.join(p)
    }
}

/// The prompt with every mentioned file or folder appended, plus what was attached.
pub fn expand(text: &str, cwd: &Path) -> (String, Vec<Attachment>) {
    let mut out = text.to_string();
    let mut attached = Vec::new();
    for mention in candidates(text) {
        let path = resolve(cwd, &mention);
        let Ok(meta) = std::fs::metadata(&path) else { continue };
        if meta.is_dir() {
            let Ok(rd) = std::fs::read_dir(&path) else { continue };
            let mut names: Vec<String> = rd
                .flatten()
                .map(|e| {
                    let name = e.file_name().to_string_lossy().into_owned();
                    if e.path().is_dir() { format!("{name}/") } else { name }
                })
                .collect();
            names.sort();
            let total = names.len();
            names.truncate(DIR_LIMIT);
            let mut body = names.join("\n");
            if total > DIR_LIMIT {
                body.push_str(&format!("\n… {} more entries", total - DIR_LIMIT));
            }
            out.push_str(&format!("\n\n<folder path=\"{}\">\n{body}\n</folder>", mention));
            attached.push(Attachment { mention, path, lines: total, is_dir: true });
        } else if meta.is_file() {
            let Ok(bytes) = std::fs::read(&path) else { continue };
            let (bytes, cut) = if bytes.len() > FILE_LIMIT { (&bytes[..FILE_LIMIT], true) } else { (&bytes[..], false) };
            let text = String::from_utf8_lossy(bytes);
            let mut body = String::new();
            let mut lines = 0;
            for (i, l) in text.lines().enumerate() {
                body.push_str(&format!("{:>4}\t{l}\n", i + 1));
                lines = i + 1;
            }
            if cut {
                body.push_str(&format!("… cut at {} bytes ({} bytes in all)\n", FILE_LIMIT, meta.len()));
            }
            out.push_str(&format!("\n\n<file path=\"{}\">\n{}</file>", mention, body));
            attached.push(Attachment { mention, path, lines, is_dir: false });
        }
    }
    (out, attached)
}

/// One line for the UI: `📎 src/main.rs (120 lines)`.
pub fn describe(a: &Attachment) -> String {
    if a.is_dir {
        format!("\u{1f4ce} {}/ ({} entries)", a.mention.trim_end_matches('/'), a.lines)
    } else {
        format!("\u{1f4ce} {} ({} lines)", a.mention, a.lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_need_a_leading_at_and_drop_punctuation() {
        assert_eq!(candidates("look at @src/main.rs, then @README.md. mail bob@x.com (@docs) @@ @"), ["src/main.rs", "README.md", "docs"]);
        assert_eq!(candidates("@a @a @b"), ["a", "b"]);
        assert!(candidates("nothing here").is_empty());
    }

    #[test]
    fn expand_attaches_files_and_folders_and_skips_unknown() {
        let d = std::env::temp_dir().join(format!("agentiloop-mentions-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("src")).unwrap();
        std::fs::write(d.join("src/main.rs"), "fn main() {}\nfn b() {}\n").unwrap();
        std::fs::write(d.join("src/lib.rs"), "").unwrap();
        let (out, att) = expand("explain @src/main.rs and list @src; ignore @nobody", &d);
        assert!(out.starts_with("explain @src/main.rs and list @src; ignore @nobody\n\n"), "{out}");
        assert!(out.contains("<file path=\"src/main.rs\">\n   1\tfn main() {}\n   2\tfn b() {}\n</file>"), "{out}");
        assert!(out.contains("<folder path=\"src\">\nlib.rs\nmain.rs\n</folder>"), "{out}");
        assert!(!out.contains("nobody\">"));
        assert_eq!(att.len(), 2);
        assert_eq!((att[0].lines, att[0].is_dir), (2, false));
        assert_eq!((att[1].lines, att[1].is_dir), (2, true));
        assert_eq!(describe(&att[0]), "\u{1f4ce} src/main.rs (2 lines)");
        assert_eq!(describe(&att[1]), "\u{1f4ce} src/ (2 entries)");

        let (same, none) = expand("no mentions", &d);
        assert_eq!(same, "no mentions");
        assert!(none.is_empty());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn big_files_are_cut() {
        let d = std::env::temp_dir().join(format!("agentiloop-mentions-big-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let big: String = (0..10_000).map(|i| format!("line {i}\n")).collect();
        std::fs::write(d.join("big.txt"), &big).unwrap();
        let (out, att) = expand("@big.txt", &d);
        assert!(out.contains("… cut at 65536 bytes"), "{}", &out[out.len() - 200..]);
        assert!(att[0].lines < 10_000);
        let _ = std::fs::remove_dir_all(&d);
    }
}
