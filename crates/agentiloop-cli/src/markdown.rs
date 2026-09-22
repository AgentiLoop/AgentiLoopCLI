//! Markdown → styled, word-wrapped ratatui lines for the TUI transcript.
//!
//! Supports headings, paragraphs, emphasis/strong/strikethrough, inline code,
//! fenced code blocks, bullet and numbered lists (nested), block quotes,
//! links and horizontal rules. Anything else falls through as plain text.

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

/// Render `text` as markdown into lines no wider than `width` cells.
pub fn render(text: &str, width: usize) -> Vec<Line<'static>> {
    let width = width.max(1);
    let text = unfence_markdown(text);
    let mut r = Renderer { width, ..Default::default() };
    let opts = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES;
    for ev in Parser::new_ext(text, opts) {
        r.event(ev);
    }
    r.flush_para();
    // Drop the trailing blank separator so entries don't double-space.
    while r.out.last().is_some_and(|l| l.spans.is_empty()) {
        r.out.pop();
    }
    r.out
}

/// Models often wrap a whole reply in a ```markdown fence when asked for
/// markdown. Rendering that as a code block defeats the purpose, so if the
/// entire text is one such fence, return its body instead.
fn unfence_markdown(text: &str) -> &str {
    let t = text.trim();
    t.strip_prefix("```markdown\n")
        .or_else(|| t.strip_prefix("```md\n"))
        .and_then(|b| b.strip_suffix("```"))
        .unwrap_or(text)
}

enum ListKind {
    Bullet,
    Numbered(u64),
}

#[derive(Default)]
struct Renderer {
    width: usize,
    out: Vec<Line<'static>>,
    /// Inline spans of the paragraph/heading/list item being built.
    para: Vec<Span<'static>>,
    /// Style modifiers from open inline tags (strong, emphasis, …).
    styles: Vec<Style>,
    lists: Vec<ListKind>,
    quote: usize,
    /// Prefix for the first wrapped line of the current block (e.g. "1. ").
    marker: Option<String>,
    in_code: bool,
    code: String,
    /// Info string of the open fenced block ("rust", "markdown", …).
    code_lang: String,
    /// Whether the next paragraph needs a blank line before it.
    need_gap: bool,
}

impl Renderer {
    fn style(&self) -> Style {
        self.styles.iter().fold(Style::default(), |s, m| s.patch(*m))
    }

    fn text(&mut self, s: &str) {
        if self.in_code {
            self.code.push_str(s);
        } else if !s.is_empty() {
            self.para.push(Span::styled(s.to_string(), self.style()));
        }
    }

    /// Indent for continuation lines: two cells per open list plus quote bars.
    fn indent(&self) -> String {
        let mut s = String::new();
        for _ in 0..self.quote {
            s.push_str("│ ");
        }
        for _ in 0..self.lists.len() {
            s.push_str("  ");
        }
        s
    }

    fn gap(&mut self) {
        if self.need_gap && !self.out.is_empty() {
            self.out.push(Line::default());
        }
        self.need_gap = false;
    }

    fn flush_para(&mut self) {
        if self.para.is_empty() {
            return;
        }
        let spans = std::mem::take(&mut self.para);
        let cont = self.indent();
        let first = match self.marker.take() {
            // The marker replaces the innermost list indent on the first line.
            Some(m) => format!("{}{m}", &cont[..cont.len().saturating_sub(2)]),
            None => cont.clone(),
        };
        self.out.extend(wrap(spans, self.width, &first, &cont));
    }

    fn event(&mut self, ev: Event<'_>) {
        match ev {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(t) => self.text(&t),
            Event::Code(c) => {
                let st = self.style().fg(Color::Yellow);
                self.para.push(Span::styled(format!("`{c}`"), st));
            }
            Event::SoftBreak => self.text(" "),
            Event::HardBreak => {
                self.flush_para();
                self.marker = None;
            }
            Event::Rule => {
                self.flush_para();
                self.gap();
                self.out.push(Line::from(Span::styled("─".repeat(self.width.min(40)), Style::default().fg(Color::DarkGray))));
                self.need_gap = true;
            }
            Event::TaskListMarker(done) => self.text(if done { "[x] " } else { "[ ] " }),
            Event::Html(h) | Event::InlineHtml(h) => self.text(&h),
            Event::FootnoteReference(_) | Event::InlineMath(_) | Event::DisplayMath(_) => {}
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph => {
                self.flush_para();
                // Inside a list item the first paragraph sits on the marker line.
                if self.marker.is_none() {
                    self.gap();
                }
            }
            Tag::Heading { level, .. } => {
                self.flush_para();
                self.gap();
                let color = match level {
                    HeadingLevel::H1 => Color::Magenta,
                    HeadingLevel::H2 => Color::Blue,
                    _ => Color::Cyan,
                };
                self.styles.push(Style::default().fg(color).add_modifier(Modifier::BOLD));
                let hashes = "#".repeat(level as usize);
                self.para.push(Span::styled(format!("{hashes} "), self.style()));
            }
            Tag::BlockQuote(_) => {
                self.flush_para();
                self.gap();
                self.quote += 1;
                self.styles.push(Style::default().add_modifier(Modifier::ITALIC));
            }
            Tag::CodeBlock(kind) => {
                self.flush_para();
                self.gap();
                self.in_code = true;
                self.code.clear();
                self.code_lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.trim().to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
            }
            Tag::List(start) => {
                self.flush_para();
                if self.lists.is_empty() {
                    self.gap();
                }
                self.lists.push(match start {
                    Some(n) => ListKind::Numbered(n),
                    None => ListKind::Bullet,
                });
            }
            Tag::Item => {
                self.flush_para();
                let m = match self.lists.last_mut() {
                    Some(ListKind::Numbered(n)) => {
                        let s = format!("{n}. ");
                        *n += 1;
                        s
                    }
                    _ => "• ".to_string(),
                };
                self.marker = Some(m);
            }
            Tag::Emphasis => self.styles.push(Style::default().add_modifier(Modifier::ITALIC)),
            Tag::Strong => self.styles.push(Style::default().add_modifier(Modifier::BOLD)),
            Tag::Strikethrough => self.styles.push(Style::default().add_modifier(Modifier::CROSSED_OUT)),
            Tag::Link { .. } => self.styles.push(Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED)),
            Tag::Image { dest_url, .. } => {
                self.text("[image: ");
                self.text(&dest_url);
                self.text("]");
            }
            Tag::Table(_) | Tag::TableHead | Tag::TableRow => {}
            Tag::TableCell => self.text(" │ "),
            Tag::HtmlBlock | Tag::FootnoteDefinition(_) | Tag::DefinitionList | Tag::DefinitionListTitle | Tag::DefinitionListDefinition | Tag::MetadataBlock(_) | Tag::Superscript | Tag::Subscript => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => {
                self.flush_para();
                self.need_gap = true;
            }
            TagEnd::Heading(_) => {
                self.flush_para();
                self.styles.pop();
                self.need_gap = true;
            }
            TagEnd::BlockQuote(_) => {
                self.flush_para();
                self.quote -= 1;
                self.styles.pop();
                self.need_gap = true;
            }
            TagEnd::CodeBlock => {
                self.in_code = false;
                let code = std::mem::take(&mut self.code);
                let lang = std::mem::take(&mut self.code_lang);
                let ind = self.indent();
                // A ```markdown fence is the model showing Markdown, not code:
                // render its body instead of boxing it.
                if lang == "markdown" || lang == "md" {
                    let inner = render(&code, self.width.saturating_sub(ind.len()));
                    for l in inner {
                        let mut spans = vec![Span::raw(ind.clone())];
                        spans.extend(l.spans);
                        self.out.push(Line::from(spans));
                    }
                    self.need_gap = true;
                    return;
                }
                if !lang.is_empty() {
                    self.out.push(Line::from(Span::styled(format!("{ind}▎{lang}"), Style::default().fg(Color::DarkGray))));
                }
                let st = Style::default().fg(Color::Green);
                for raw in code.strip_suffix('\n').unwrap_or(&code).split('\n') {
                    let opts = textwrap::Options::new(self.width.saturating_sub(ind.len() + 2).max(1));
                    for piece in textwrap::wrap(raw, &opts) {
                        self.out.push(Line::from(vec![
                            Span::styled(format!("{ind}▎ "), Style::default().fg(Color::DarkGray)),
                            Span::styled(piece.into_owned(), st),
                        ]));
                    }
                }
                self.need_gap = true;
            }
            TagEnd::List(_) => {
                self.flush_para();
                self.lists.pop();
                if self.lists.is_empty() {
                    self.need_gap = true;
                }
            }
            TagEnd::Item => {
                self.flush_para();
                // An empty item still shows its marker.
                if let Some(m) = self.marker.take() {
                    let ind = self.indent();
                    self.out.push(Line::from(format!("{}{m}", &ind[..ind.len().saturating_sub(2)])));
                }
                self.need_gap = false;
            }
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link => {
                self.styles.pop();
            }
            TagEnd::TableRow | TagEnd::TableHead => {
                self.text(" │");
                self.flush_para();
            }
            TagEnd::Table => self.need_gap = true,
            _ => {}
        }
    }
}

/// Word-wrap styled spans to `width`, prefixing the first line with `first`
/// and the rest with `cont`.
fn wrap(spans: Vec<Span<'static>>, width: usize, first: &str, cont: &str) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut cur: Vec<Span<'static>> = vec![Span::raw(first.to_string())];
    let mut col = first.width();
    let mut fresh = true;
    for span in spans {
        for word in split_words(&span.content) {
            let w = word.width();
            if !fresh && col + w > width {
                trim_end(&mut cur);
                lines.push(Line::from(std::mem::take(&mut cur)));
                cur.push(Span::raw(cont.to_string()));
                col = cont.width();
                fresh = true;
                if word.trim().is_empty() {
                    continue;
                }
            }
            if let Some(last) = cur.last_mut().filter(|l| l.style == span.style && !fresh) {
                last.content.to_mut().push_str(word);
            } else {
                cur.push(Span::styled(word.to_string(), span.style));
            }
            col += w;
            fresh = false;
        }
    }
    trim_end(&mut cur);
    lines.push(Line::from(cur));
    lines
}

/// Split into words, each carrying its trailing whitespace.
fn split_words(s: &str) -> Vec<&str> {
    let mut words = Vec::new();
    let mut start = 0;
    let mut in_space = false;
    for (i, c) in s.char_indices() {
        if c.is_whitespace() {
            in_space = true;
        } else if in_space {
            words.push(&s[start..i]);
            start = i;
            in_space = false;
        }
    }
    if start < s.len() {
        words.push(&s[start..]);
    }
    words
}

fn trim_end(spans: &mut Vec<Span<'static>>) {
    if let Some(last) = spans.last_mut() {
        let t = last.content.trim_end().to_string();
        last.content = t.into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(md: &str, width: usize) -> Vec<String> {
        render(md, width).iter().map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect()).collect()
    }

    #[test]
    fn headings_lists_and_code() {
        let md = "# Title\n\nSome *em* and **strong** text.\n\n- one\n- two\n  - nested\n\n1. first\n2. second\n\n```rust\nfn x() {}\n```\n";
        let r = rows(md, 40);
        assert_eq!(
            r,
            vec![
                "# Title",
                "",
                "Some em and strong text.",
                "",
                "• one",
                "• two",
                "  • nested",
                "",
                "1. first",
                "2. second",
                "",
                "▎rust",
                "▎ fn x() {}",
            ]
        );
        let lines = render(md, 40);
        assert!(lines[0].spans.iter().any(|s| s.content == "# Title" && s.style.add_modifier.contains(Modifier::BOLD)));
        assert!(lines[2].spans.iter().any(|s| s.content == "em" && s.style.add_modifier.contains(Modifier::ITALIC)));
        assert!(lines[2].spans.iter().any(|s| s.content == "strong" && s.style.add_modifier.contains(Modifier::BOLD)));
    }

    #[test]
    fn wraps_at_width_with_continuation_indent() {
        let r = rows("- alpha beta gamma delta epsilon", 14);
        assert_eq!(r, vec!["• alpha beta", "  gamma delta", "  epsilon"]);
        let r = rows("word ".repeat(10).trim_end(), 12);
        assert!(r.iter().all(|l| l.width() <= 12), "{r:?}");
        assert_eq!(r.join(" "), "word ".repeat(10).trim_end());
    }

    #[test]
    fn blockquote_and_inline_code() {
        let r = rows("> quoted `code` here", 40);
        assert_eq!(r, vec!["│ quoted `code` here"]);
    }

    #[test]
    fn paragraphs_separated_by_one_blank_row() {
        assert_eq!(rows("para one\n\npara two", 20), vec!["para one", "", "para two"]);
    }

    #[test]
    fn markdown_fences_render_as_markdown() {
        let r = rows("```markdown\n# Hi\n\n- a\n```\n", 20);
        assert_eq!(r, vec!["# Hi", "", "• a"]);
        // An embedded ```markdown block renders too; other languages stay code.
        let r = rows("Intro\n\n```markdown\n# Hi\n```\n\n```rust\n# x\n```", 20);
        assert_eq!(r, vec!["Intro", "", "# Hi", "", "▎rust", "▎ # x"]);
    }
}
