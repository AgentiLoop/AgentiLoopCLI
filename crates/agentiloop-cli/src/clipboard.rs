//! `/copy`: put text on the system clipboard — the platform's copy tool when
//! one runs, else an OSC 52 escape so the terminal (also over SSH) copies it.

use std::io::Write;
use std::process::{Command, Stdio};

/// Copy tools to try in order on this OS: program and its arguments.
pub(crate) fn tools(os: &str) -> &'static [(&'static str, &'static [&'static str])] {
    match os {
        "macos" => &[("pbcopy", &[])],
        "windows" => &[("clip", &[])],
        _ => &[("wl-copy", &[]), ("xclip", &["-selection", "clipboard"]), ("xsel", &["--clipboard", "--input"])],
    }
}

/// What a copy tool reads on stdin: clip.exe wants UTF-16LE with a BOM to keep non-ASCII text.
pub(crate) fn tool_input(program: &str, text: &str) -> Vec<u8> {
    if program == "clip" {
        let mut out = vec![0xFF, 0xFE];
        out.extend(text.encode_utf16().flat_map(|u| u.to_le_bytes()));
        out
    } else {
        text.as_bytes().to_vec()
    }
}

/// The OSC 52 "set clipboard" sequence for `text`.
pub(crate) fn osc52(text: &str) -> String {
    format!("\x1b]52;c;{}\x07", agentiloop_core::mentions::base64(text.as_bytes()))
}

/// Copies `text` and says how: the tool that took it, or "the terminal (OSC 52)".
pub(crate) fn copy(text: &str) -> String {
    for (program, args) in tools(std::env::consts::OS) {
        let Ok(mut child) = Command::new(program).args(*args).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn() else {
            continue;
        };
        let wrote = child.stdin.take().is_some_and(|mut stdin| stdin.write_all(&tool_input(program, text)).is_ok());
        if child.wait().is_ok_and(|s| s.success()) && wrote {
            return (*program).to_string();
        }
    }
    let mut err = std::io::stderr();
    let _ = err.write_all(osc52(text).as_bytes()).and_then(|_| err.flush());
    "the terminal (OSC 52)".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_os_has_copy_tools_in_preference_order() {
        assert_eq!(tools("macos")[0].0, "pbcopy");
        assert_eq!(tools("windows")[0].0, "clip");
        let linux: Vec<_> = tools("linux").iter().map(|t| t.0).collect();
        assert_eq!(linux, ["wl-copy", "xclip", "xsel"]);
        assert_eq!(tools("freebsd").len(), 3);
    }

    #[test]
    fn clip_gets_utf16_with_a_bom_and_others_utf8() {
        assert_eq!(tool_input("clip", "aé"), [0xFF, 0xFE, b'a', 0, 0xE9, 0]);
        assert_eq!(tool_input("pbcopy", "aé"), "aé".as_bytes());
    }

    #[test]
    fn osc52_wraps_base64_of_the_text() {
        assert_eq!(osc52("ab"), "\x1b]52;c;YWI=\x07");
    }
}
