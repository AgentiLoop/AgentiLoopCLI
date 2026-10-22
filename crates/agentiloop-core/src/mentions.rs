//! `@path` mentions: a word starting with `@` in a prompt that names an existing file or folder
//! (relative to the project folder, or absolute) attaches it to the prompt the model receives —
//! the file's text with line numbers, the folder's entries, or an image (png/jpeg/gif/webp) as an
//! image block. The typed text is kept as is; the attachments follow it. Words that name nothing
//! (`@alice`, an e-mail address) are left alone.

use std::path::{Path, PathBuf};

use crate::message::{ContentBlock, ImageSource};

/// Files longer than this are cut (bytes).
pub const FILE_LIMIT: usize = 64 * 1024;
/// Folders list at most this many entries.
pub const DIR_LIMIT: usize = 200;
/// Images larger than this (bytes) are not attached (the Anthropic API limit).
pub const IMAGE_LIMIT: usize = 5 * 1024 * 1024;

/// One attached path, for the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    /// As typed, without the `@`.
    pub mention: String,
    pub path: PathBuf,
    /// Lines of a file, entries of a folder, bytes of an image.
    pub lines: usize,
    pub is_dir: bool,
    /// Set for an image: it goes to the model as an image block, not as text.
    pub image: Option<ImageSource>,
}

/// The media type of an image file, by extension.
pub fn image_media_type(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

/// Standard base64 with padding.
pub fn base64(bytes: &[u8]) -> String {
    const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = (u32::from(c[0]) << 16) | (u32::from(*c.get(1).unwrap_or(&0)) << 8) | u32::from(*c.get(2).unwrap_or(&0));
        for i in 0..4 {
            out.push(if i <= c.len() { A[(n >> (18 - 6 * i) & 63) as usize] as char } else { '=' });
        }
    }
    out
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
            attached.push(Attachment { mention, path, lines: total, is_dir: true, image: None });
        } else if let Some(media_type) = image_media_type(&path).filter(|_| meta.is_file()) {
            if meta.len() as usize > IMAGE_LIMIT {
                continue;
            }
            let Ok(bytes) = std::fs::read(&path) else { continue };
            let image = ImageSource { kind: "base64".into(), media_type: media_type.into(), data: base64(&bytes) };
            attached.push(Attachment { mention, path, lines: bytes.len(), is_dir: false, image: Some(image) });
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
            attached.push(Attachment { mention, path, lines, is_dir: false, image: None });
        }
    }
    (out, attached)
}

/// The user turn for the model: the expanded text, then one image block per attached image.
pub fn content(text: impl Into<String>, attached: &[Attachment]) -> Vec<ContentBlock> {
    let mut blocks = vec![ContentBlock::Text { text: text.into() }];
    blocks.extend(attached.iter().filter_map(|a| a.image.clone()).map(|source| ContentBlock::Image { source }));
    blocks
}

/// One line for the UI: `📎 src/main.rs (120 lines)`.
pub fn describe(a: &Attachment) -> String {
    if a.is_dir {
        format!("\u{1f4ce} {}/ ({} entries)", a.mention.trim_end_matches('/'), a.lines)
    } else if let Some(image) = &a.image {
        format!("\u{1f4ce} {} ({}, {} KB)", a.mention, image.media_type, a.lines.div_ceil(1024))
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
    fn images_become_image_blocks_not_text() {
        let d = std::env::temp_dir().join(format!("agentiloop-mentions-img-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("shot.PNG"), b"\x89PNG\r\n").unwrap();
        let (out, att) = expand("what is in @shot.PNG?", &d);
        assert_eq!(out, "what is in @shot.PNG?");
        assert_eq!(att.len(), 1);
        let img = att[0].image.as_ref().unwrap();
        assert_eq!((img.kind.as_str(), img.media_type.as_str(), img.data.as_str()), ("base64", "image/png", "iVBORw0K"));
        assert_eq!(describe(&att[0]), "\u{1f4ce} shot.PNG (image/png, 1 KB)");
        let blocks = content(out, &att);
        assert_eq!(blocks.len(), 2);
        assert!(matches!(&blocks[0], ContentBlock::Text { text } if text == "what is in @shot.PNG?"));
        assert_eq!(serde_json::to_value(&blocks[1]).unwrap(), serde_json::json!({"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": "iVBORw0K"}}));
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"ab"), "YWI=");
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
