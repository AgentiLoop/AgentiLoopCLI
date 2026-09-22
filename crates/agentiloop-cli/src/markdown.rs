//! Markdown → styled, word-wrapped ratatui lines for the TUI transcript.
//!
//! Supports headings, paragraphs, emphasis/strong/strikethrough, inline code,
//! fenced code blocks, bullet and numbered lists (nested), block quotes,
//! tables, links and horizontal rules. Anything else falls through as plain text.

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

/// Link color: macOS system blue (dark-mode variant). ANSI `Blue` is far
/// too dark on dark themes; truecolor is supported by every mainstream
/// terminal on macOS, Linux and Windows.
const LINK_BLUE: Color = Color::Rgb(10, 132, 255);

/// A link's cell range on one rendered line, for click handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPos {
    pub line: usize,
    pub start: usize,
    pub end: usize,
    pub url: String,
}

/// Render `text` as markdown into lines no wider than `width` cells, also
/// returning where each link landed (for click handling).
pub fn render_links(text: &str, width: usize) -> (Vec<Line<'static>>, Vec<LinkPos>) {
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
    (r.out, r.links)
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
    /// Inline spans of the paragraph/heading/list item/table cell being built.
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
    /// Level of the heading being built (for the underline rule at its end).
    heading: Option<HeadingLevel>,
    /// Rows of cells collected for the open table; row 0 is the header.
    table: Vec<Vec<Vec<Span<'static>>>>,
    /// Whether the next paragraph needs a blank line before it.
    need_gap: bool,
    /// Link destinations in order of appearance; spans carry their index
    /// in `underline_color` (see `link_tag`) until `flush_para` records them.
    urls: Vec<String>,
    links: Vec<LinkPos>,
}

/// Smuggle a link index through `Style` so it survives word-wrapping and
/// span merging; stripped again before the line is emitted.
fn link_tag(id: usize) -> Color {
    Color::Rgb((id >> 16) as u8, (id >> 8) as u8, id as u8)
}

fn link_id(style: &Style) -> Option<usize> {
    match style.underline_color {
        Some(Color::Rgb(a, b, c)) => Some((a as usize) << 16 | (b as usize) << 8 | c as usize),
        _ => None,
    }
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
        let mut lines = wrap(spans, self.width, &first, &cont);
        for (i, line) in lines.iter_mut().enumerate() {
            let mut col = 0;
            for s in &mut line.spans {
                let w = s.content.width();
                if let Some(id) = link_id(&s.style) {
                    s.style.underline_color = None;
                    let url = &self.urls[id];
                    // Same link continuing in the next span (e.g. `code` inside it).
                    match self.links.last_mut() {
                        Some(l) if l.line == self.out.len() + i && l.end == col && l.url == *url => l.end = col + w,
                        _ => self.links.push(LinkPos { line: self.out.len() + i, start: col, end: col + w, url: url.clone() }),
                    }
                }
                col += w;
            }
        }
        self.out.extend(lines);
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
                // No font sizes in a terminal: H1 is a filled banner, H2 is
                // bold with a rule under it, H3 bold yellow, H4+ plain bold.
                let style = match level {
                    HeadingLevel::H1 => Style::default().fg(Color::White).bg(Color::Magenta).add_modifier(Modifier::BOLD),
                    HeadingLevel::H2 => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    HeadingLevel::H3 => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    _ => Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                };
                self.styles.push(style);
                self.heading = Some(level);
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
            Tag::Link { dest_url, .. } => {
                let id = self.urls.len();
                self.urls.push(dest_url.to_string());
                self.styles.push(Style::default().fg(LINK_BLUE).add_modifier(Modifier::UNDERLINED).underline_color(link_tag(id)));
            }
            Tag::Image { dest_url, .. } => {
                self.text("[image: ");
                self.text(&dest_url);
                self.text("]");
            }
            Tag::Table(_) => {
                self.flush_para();
                self.gap();
                self.table.clear();
            }
            Tag::TableHead | Tag::TableRow => self.table.push(Vec::new()),
            Tag::TableCell => self.para.clear(),
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
                let level = self.heading.take();
                let st = self.style();
                if level == Some(HeadingLevel::H1) {
                    // Banner: pad the text so the background reads as a block.
                    self.para.insert(0, Span::styled(" ", st));
                }
                self.flush_para();
                if level == Some(HeadingLevel::H1) {
                    // Right pad goes on after wrapping, which trims trailing spaces.
                    if let Some(l) = self.out.last_mut() {
                        l.spans.push(Span::styled(" ", st));
                    }
                }
                self.styles.pop();
                if level == Some(HeadingLevel::H2) {
                    let ind = self.indent();
                    let n = self.width.saturating_sub(ind.len()).min(60).max(1);
                    self.out.push(Line::from(Span::styled(format!("{ind}{}", "─".repeat(n)), Style::default().fg(Color::Cyan))));
                }
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
                    let (inner, links) = render_links(&code, self.width.saturating_sub(ind.len()));
                    let (base, off) = (self.out.len(), ind.width());
                    self.links.extend(links.into_iter().map(|l| LinkPos { line: base + l.line, start: l.start + off, end: l.end + off, url: l.url }));
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
                // Syntax-highlight when the fence language is known; plain
                // green otherwise. Code is hard-wrapped, never word-wrapped.
                let lines = crate::highlight::highlight(&code, &lang).unwrap_or_else(|| {
                    let st = Style::default().fg(Color::Green);
                    code.strip_suffix('\n')
                        .unwrap_or(&code)
                        .split('\n')
                        .map(|l| vec![Span::styled(l.to_string(), st)])
                        .collect()
                });
                let avail = self.width.saturating_sub(ind.len() + 2).max(1);
                for line in lines {
                    for piece in crate::highlight::hard_wrap(line, avail) {
                        let mut spans = vec![Span::styled(format!("{ind}▎ "), Style::default().fg(Color::DarkGray))];
                        spans.extend(piece);
                        self.out.push(Line::from(spans));
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
            TagEnd::TableCell => {
                let cell = std::mem::take(&mut self.para);
                if let Some(row) = self.table.last_mut() {
                    row.push(cell);
                }
            }
            TagEnd::TableRow | TagEnd::TableHead => {}
            TagEnd::Table => {
                let rows = std::mem::take(&mut self.table);
                self.out.extend(draw_table(rows, &self.indent()));
                self.need_gap = true;
            }
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

/// Box-draw a table with columns sized to their widest cell. The first row
/// is the header (bold) and is separated from the body by a rule.
fn draw_table(rows: Vec<Vec<Vec<Span<'static>>>>, ind: &str) -> Vec<Line<'static>> {
    let ncols = rows.iter().map(Vec::len).max().unwrap_or(0);
    if ncols == 0 {
        return Vec::new();
    }
    // Cell text as one trimmed string per span (markdown leaves padding
    // spaces around cell content).
    let cells: Vec<Vec<Vec<(String, Style)>>> = rows
        .iter()
        .map(|r| {
            r.iter()
                .map(|c| {
                    let n = c.len();
                    c.iter()
                        .enumerate()
                        .map(|(i, s)| {
                            let mut t = s.content.as_ref();
                            if i == 0 {
                                t = t.trim_start();
                            }
                            if i + 1 == n {
                                t = t.trim_end();
                            }
                            // Table links are styled but not clickable.
                            let mut st = s.style;
                            st.underline_color = None;
                            (t.to_string(), st)
                        })
                        .collect()
                })
                .collect()
        })
        .collect();
    let cell_w = |c: &Vec<(String, Style)>| c.iter().map(|(t, _)| t.width()).sum::<usize>();
    let mut widths = vec![0usize; ncols];
    for r in &cells {
        for (i, c) in r.iter().enumerate() {
            widths[i] = widths[i].max(cell_w(c));
        }
    }
    let border = Style::default().fg(Color::DarkGray);
    let rule = |l: &str, m: &str, r: &str| {
        let body: Vec<String> = widths.iter().map(|w| "─".repeat(w + 2)).collect();
        Line::from(Span::styled(format!("{ind}{l}{}{r}", body.join(m)), border))
    };
    let mut out = vec![rule("┌", "┬", "┐")];
    for (ri, r) in cells.iter().enumerate() {
        let mut spans = vec![Span::styled(format!("{ind}│"), border)];
        for (i, w) in widths.iter().enumerate() {
            let used = r.get(i).map_or(0, cell_w);
            spans.push(Span::raw(" "));
            if let Some(c) = r.get(i) {
                for (t, st) in c {
                    let st = if ri == 0 { st.add_modifier(Modifier::BOLD) } else { *st };
                    spans.push(Span::styled(t.clone(), st));
                }
            }
            spans.push(Span::raw(" ".repeat(w - used + 1)));
            spans.push(Span::styled("│", border));
        }
        out.push(Line::from(spans));
        if ri == 0 && cells.len() > 1 {
            out.push(rule("├", "┼", "┤"));
        }
    }
    out.push(rule("└", "┴", "┘"));
    out
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

    fn render(md: &str, width: usize) -> Vec<Line<'static>> {
        render_links(md, width).0
    }

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
                " Title ",
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
        assert!(lines[0].spans.iter().any(|s| s.content.contains("Title") && s.style.bg == Some(Color::Magenta)));
        assert!(lines[2].spans.iter().any(|s| s.content == "em" && s.style.add_modifier.contains(Modifier::ITALIC)));
        assert!(lines[2].spans.iter().any(|s| s.content == "strong" && s.style.add_modifier.contains(Modifier::BOLD)));
    }

    #[test]
    fn heading_levels_drop_hashes() {
        assert_eq!(rows("## Two\n\n### Three\n\n#### Four", 40), vec!["Two", &"─".repeat(40), "", "Three", "", "Four"]);
    }

    #[test]
    fn tables_are_boxed_and_aligned() {
        let md = "| Feature | Groovy? |\n|---|---|\n| Dynamic | ✅ |\n| **Scripting** | yes |";
        assert_eq!(
            rows(md, 60),
            vec![
                "┌───────────┬─────────┐",
                "│ Feature   │ Groovy? │",
                "├───────────┼─────────┤",
                "│ Dynamic   │ ✅      │",
                "│ Scripting │ yes     │",
                "└───────────┴─────────┘",
            ]
        );
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
    fn link_positions_are_reported_and_tag_is_stripped() {
        let (lines, links) = render_links("see [the docs](https://a.io/x) and [b](https://b.io)", 80);
        assert_eq!(rows("see [the docs](https://a.io/x) and [b](https://b.io)", 80), vec!["see the docs and b"]);
        assert_eq!(
            links,
            vec![
                LinkPos { line: 0, start: 4, end: 12, url: "https://a.io/x".into() },
                LinkPos { line: 0, start: 17, end: 18, url: "https://b.io".into() },
            ]
        );
        assert!(lines[0].spans.iter().all(|s| s.style.underline_color.is_none()));
        // A link that wraps reports one range per line, inside a list indent.
        let (_, links) = render_links("- [alpha beta gamma](https://w.io)", 12);
        let ranges: Vec<_> = links.iter().map(|l| (l.line, l.start, l.end)).collect();
        assert_eq!(ranges, vec![(0, 2, 7), (1, 2, 12)]);
        assert!(links.iter().all(|l| l.url == "https://w.io"));
    }

    #[test]
    fn markdown_fences_render_as_markdown() {
        let r = rows("```markdown\n# Hi\n\n- a\n```\n", 20);
        assert_eq!(r, vec![" Hi ", "", "• a"]);
        // An embedded ```markdown block renders too; other languages stay code.
        let r = rows("Intro\n\n```markdown\n### Hi\n```\n\n```rust\n# x\n```", 20);
        assert_eq!(r, vec!["Intro", "", "Hi", "", "▎rust", "▎ # x"]);
    }
}
