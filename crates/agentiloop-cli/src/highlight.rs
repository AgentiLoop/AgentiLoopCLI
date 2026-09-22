//! Syntax highlighting (syntect) → ratatui spans. Used for fenced code
//! blocks in markdown and for `read_file` previews in the TUI.

use std::path::Path;
use std::sync::OnceLock;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, StyleModifier, Theme, ThemeItem};
use syntect::parsing::{SyntaxDefinition, SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;
use unicode_width::UnicodeWidthChar;

/// Grammars syntect doesn't ship (see ../syntaxes/README.md).
const EXTRA_SYNTAXES: &[&str] = &[
    include_str!("../syntaxes/Swift.sublime-syntax"),
    include_str!("../syntaxes/TOML.sublime-syntax"),
];

fn syntaxes() -> &'static SyntaxSet {
    static S: OnceLock<SyntaxSet> = OnceLock::new();
    S.get_or_init(|| {
        let mut b = SyntaxSet::load_defaults_newlines().into_builder();
        for src in EXTRA_SYNTAXES {
            // `true`: the default set is the newline-including variant.
            b.add(SyntaxDefinition::load_from_str(src, true, None).expect("bundled grammar parses"));
        }
        b.build()
    })
}

/// Xcode Dark palette (same hex values as AgentColorSyntax's CodeBlockTheme),
/// mapped onto TextMate scopes. Keywords are bold like the Agent app.
fn theme() -> &'static Theme {
    static T: OnceLock<Theme> = OnceLock::new();
    T.get_or_init(|| {
        let rgb = |h: u32| syntect::highlighting::Color { r: (h >> 16) as u8, g: (h >> 8) as u8, b: h as u8, a: 0xFF };
        let item = |scope: &str, h: u32, bold: bool| ThemeItem {
            scope: scope.parse().expect("valid scope selector"),
            style: StyleModifier {
                foreground: Some(rgb(h)),
                background: None,
                font_style: Some(if bold { FontStyle::BOLD } else { FontStyle::empty() }),
            },
        };
        let mut t = Theme::default();
        t.settings.foreground = Some(rgb(0xDFDFE0));
        // Order matters: later, more specific rules win.
        t.scopes = vec![
            item("comment, punctuation.definition.comment", 0x6C9C5A, false),
            item("keyword, storage, storage.type, storage.modifier, keyword.control, keyword.other", 0xFF7AB2, true),
            item("keyword.operator, punctuation, punctuation.separator, punctuation.terminator, punctuation.accessor", 0xDFDFE0, false),
            item("string, punctuation.definition.string, string.quoted, string.regexp", 0xFC6A5D, false),
            item("constant.numeric, constant.language, constant.character", 0xD9C97C, false),
            item("constant.other, support.constant, variable.other.constant", 0xD9C97C, false),
            item("entity.name.type, entity.name.class, entity.name.struct, entity.name.enum, entity.name.trait, entity.name.impl, entity.other.inherited-class, support.type, support.class, storage.type.class", 0xD0A8FF, false),
            item("entity.name.function, entity.name.method, meta.function-call, variable.function, meta.function-call.generic", 0x67B7A4, false),
            item("support.function, support.macro, entity.name.macro, meta.macro", 0xB281EB, false),
            item("meta.preprocessor, keyword.control.import, keyword.control.directive, keyword.other.import, keyword.other.use, keyword.other.preprocessor", 0xFFA14F, true),
            item("meta.annotation, entity.other.attribute-name, meta.attribute, support.other.attribute, variable.annotation, punctuation.definition.annotation", 0xFD8F3F, false),
            item("variable.other.member, variable.other.property, variable.other.object.property, meta.property, support.type.property-name, entity.name.tag", 0x4EB0CC, false),
            item("variable.language, variable.language.self, variable.language.this, variable.parameter.self", 0xFF7AB2, true),
            item("variable.parameter, variable.other", 0xDFDFE0, false),
        ];
        t
    })
}

/// Resolve a fence language ("rust", "py", "bash") or a file extension.
fn syntax_for(hint: &str) -> Option<&'static SyntaxReference> {
    let h = hint.trim();
    if h.is_empty() {
        return None;
    }
    let ss = syntaxes();
    ss.find_syntax_by_token(h).or_else(|| ss.find_syntax_by_extension(h))
}

/// Highlight `code` as `hint` (language token or extension). One span list per
/// line, no line terminators. `None` when the language is unknown.
pub fn highlight(code: &str, hint: &str) -> Option<Vec<Vec<Span<'static>>>> {
    let syn = syntax_for(hint)?;
    let code = code.replace('\t', "    ");
    let mut hl = HighlightLines::new(syn, theme());
    let mut out = Vec::new();
    for line in LinesWithEndings::from(&code) {
        let regions = hl.highlight_line(line, syntaxes()).ok()?;
        let spans: Vec<Span<'static>> = regions
            .into_iter()
            .map(|(st, text)| (st, text.trim_end_matches(['\n', '\r'])))
            .filter(|(_, t)| !t.is_empty())
            .map(|(st, t)| Span::styled(t.to_string(), to_style(st)))
            .collect();
        out.push(spans);
    }
    Some(out)
}

/// Highlight `read_file`-style lines (`"    1│code"`) by `path`'s extension,
/// keeping the gutter. `None` if the extension is unknown or a line has no gutter.
pub fn highlight_numbered(lines: &[&str], path: &str) -> Option<Vec<Vec<Span<'static>>>> {
    let ext = Path::new(path).extension()?.to_str()?;
    let mut gutters = Vec::with_capacity(lines.len());
    let mut bodies = Vec::with_capacity(lines.len());
    for l in lines {
        let (g, b) = l.split_once('│')?;
        gutters.push(format!("{g}│"));
        bodies.push(b);
    }
    let code = highlight(&bodies.join("\n"), ext)?;
    let gutter = Style::default().fg(Color::DarkGray);
    Some(
        gutters
            .into_iter()
            .zip(code.into_iter().chain(std::iter::repeat(Vec::new())))
            .map(|(g, mut spans)| {
                spans.insert(0, Span::styled(g, gutter));
                spans
            })
            .collect(),
    )
}

fn to_style(st: syntect::highlighting::Style) -> Style {
    let fg = st.foreground;
    let mut s = Style::default().fg(Color::Rgb(fg.r, fg.g, fg.b));
    if st.font_style.contains(FontStyle::BOLD) {
        s = s.add_modifier(Modifier::BOLD);
    }
    if st.font_style.contains(FontStyle::ITALIC) {
        s = s.add_modifier(Modifier::ITALIC);
    }
    if st.font_style.contains(FontStyle::UNDERLINE) {
        s = s.add_modifier(Modifier::UNDERLINED);
    }
    s
}

/// Hard-wrap styled spans at `width` cells (code is never word-wrapped).
/// Always yields at least one (possibly empty) piece.
pub fn hard_wrap(spans: Vec<Span<'static>>, width: usize) -> Vec<Vec<Span<'static>>> {
    let width = width.max(1);
    let mut out = Vec::new();
    let mut cur: Vec<Span<'static>> = Vec::new();
    let mut col = 0;
    for span in spans {
        let mut buf = String::new();
        for ch in span.content.chars() {
            let w = ch.width().unwrap_or(0);
            if col > 0 && col + w > width {
                if !buf.is_empty() {
                    cur.push(Span::styled(std::mem::take(&mut buf), span.style));
                }
                out.push(std::mem::take(&mut cur));
                col = 0;
            }
            buf.push(ch);
            col += w;
        }
        if !buf.is_empty() {
            cur.push(Span::styled(buf, span.style));
        }
    }
    out.push(cur);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(spans: &[Span<'static>]) -> String {
        spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn rust_keywords_get_a_color_and_text_is_preserved() {
        let hl = highlight("fn main() {}\n", "rust").unwrap();
        assert_eq!(hl.len(), 1);
        assert_eq!(text(&hl[0]), "fn main() {}");
        let kw = hl[0].iter().find(|s| s.content == "fn").unwrap();
        assert!(matches!(kw.style.fg, Some(Color::Rgb(..))));
        // Different token classes get different colors.
        let name = hl[0].iter().find(|s| s.content == "main").unwrap();
        assert_ne!(kw.style.fg, name.style.fg);
    }

    #[test]
    fn bundled_swift_and_toml_grammars_load() {
        let hl = highlight("let x: Int = 1\n", "swift").unwrap();
        assert!(hl[0].iter().any(|s| s.content == "let"));
        assert!(highlight("[package]\nname = \"a\"\n", "toml").is_some());
        assert!(highlight_numbered(&["    1│import SwiftUI"], "App/AgentApp.swift").is_some());
    }

    #[test]
    fn xcode_dark_palette_is_applied() {
        let hl = highlight("pub fn f() { let s = \"hi\"; } // note\n", "rust").unwrap();
        let find = |t: &str| hl[0].iter().find(|s| s.content == t).unwrap().style;
        let kw = find("fn");
        assert_eq!(kw.fg, Some(Color::Rgb(0xFF, 0x7A, 0xB2)));
        assert!(kw.add_modifier.contains(Modifier::BOLD));
        assert_eq!(find("f").fg, Some(Color::Rgb(0x67, 0xB7, 0xA4)));
        assert_eq!(find("hi").fg, Some(Color::Rgb(0xFC, 0x6A, 0x5D)));
        assert_eq!(find(" note").fg, Some(Color::Rgb(0x6C, 0x9C, 0x5A)));
        let sw = highlight("import SwiftUI\nstruct A { var x: Int = 1 }\n", "swift").unwrap();
        // `import` is a preprocessor-class keyword → Xcode orange.
        let imp = sw[0].iter().find(|s| s.content == "import").unwrap().style;
        assert_eq!(imp.fg, Some(Color::Rgb(0xFF, 0xA1, 0x4F)));
        assert_eq!(sw[1].iter().find(|s| s.content == "struct").unwrap().style.fg, Some(Color::Rgb(0xFF, 0x7A, 0xB2)));
        assert_eq!(sw[1].iter().find(|s| s.content == "Int").unwrap().style.fg, Some(Color::Rgb(0xD0, 0xA8, 0xFF)));
    }

    #[test]
    fn unknown_language_is_none() {
        assert!(highlight("x", "nope-not-a-lang").is_none());
        assert!(highlight("x", "").is_none());
    }

    #[test]
    fn numbered_lines_keep_gutter() {
        let lines = ["    1│fn x() {}", "    2│"];
        let hl = highlight_numbered(&lines, "src/a.rs").unwrap();
        assert_eq!(text(&hl[0]), "    1│fn x() {}");
        assert_eq!(text(&hl[1]), "    2│");
        assert!(highlight_numbered(&["no gutter"], "a.rs").is_none());
        assert!(highlight_numbered(&lines, "README").is_none());
    }

    #[test]
    fn hard_wrap_splits_at_width() {
        let pieces = hard_wrap(vec![Span::raw("abcdefgh")], 3);
        let got: Vec<String> = pieces.iter().map(|p| text(p)).collect();
        assert_eq!(got, vec!["abc", "def", "gh"]);
        assert_eq!(hard_wrap(Vec::new(), 3).len(), 1);
    }
}
