//! `/context`: what the next request would send, split into parts, as a rough token estimate
//! (about 4 bytes of UTF-8 text per token — not a tokenizer).

use crate::message::{ContentBlock, Message, Role};
use crate::provider::ToolSpec;

/// Byte sizes (and counts) of each part of a request.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ContextBreakdown {
    pub system: u64,
    pub tools: u64,
    pub tool_count: usize,
    pub prompts: u64,
    pub replies: u64,
    pub tool_calls: u64,
    pub tool_call_count: usize,
    pub tool_results: u64,
    pub tool_result_count: usize,
    pub images: usize,
}

impl ContextBreakdown {
    pub fn total(&self) -> u64 {
        self.system + self.tools + self.prompts + self.replies + self.tool_calls + self.tool_results
    }
}

/// Measure a request made of `system`, `tools` and `history`.
pub fn breakdown(system: &str, tools: &[ToolSpec], history: &[Message]) -> ContextBreakdown {
    let mut b = ContextBreakdown { system: system.len() as u64, tool_count: tools.len(), ..Default::default() };
    for t in tools {
        b.tools += (t.name.len() + t.description.len() + t.input_schema.to_string().len()) as u64;
    }
    for m in history {
        for block in &m.content {
            match block {
                ContentBlock::Text { text } if m.role == Role::User => b.prompts += text.len() as u64,
                ContentBlock::Text { text } => b.replies += text.len() as u64,
                ContentBlock::ToolUse { name, input, .. } => {
                    b.tool_calls += (name.len() + input.to_string().len()) as u64;
                    b.tool_call_count += 1;
                }
                ContentBlock::ToolResult { content, .. } => {
                    b.tool_results += content.len() as u64;
                    b.tool_result_count += 1;
                }
                ContentBlock::Image { .. } => b.images += 1,
            }
        }
    }
    b
}

/// Estimated tokens for `bytes` of text.
pub fn estimate_tokens(bytes: u64) -> u64 {
    bytes.div_ceil(4)
}

/// The `/context` report. `window` is the model's context window when known, `last_input` the
/// input tokens the provider reported for the latest request (0 = none yet).
pub fn render(b: &ContextBreakdown, window: Option<u64>, last_input: u64) -> String {
    let row = |label: String, bytes: u64| format!("  {label:<20} ~{} tokens", estimate_tokens(bytes));
    let mut lines = vec![
        "context estimate (about 4 bytes of text per token):".to_string(),
        row("system prompt".into(), b.system),
        row(format!("tools ({})", b.tool_count), b.tools),
        row("your messages".into(), b.prompts),
        row("replies".into(), b.replies),
        row(format!("tool calls ({})", b.tool_call_count), b.tool_calls),
        row(format!("tool results ({})", b.tool_result_count), b.tool_results),
    ];
    if b.images > 0 {
        lines.push(format!("  {:<20} {} (not counted)", "images", b.images));
    }
    let mut total = row("total".into(), b.total());
    if let Some(w) = window.filter(|w| *w > 0) {
        total.push_str(&format!(" ({}% of {w})", estimate_tokens(b.total()) * 100 / w));
    }
    lines.push(total);
    if last_input > 0 {
        lines.push(format!("last request: {last_input} input tokens as counted by the provider"));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn breakdown_sorts_bytes_into_parts() {
        let tools = vec![ToolSpec { name: "bash".into(), description: "run".into(), input_schema: json!({"type":"object"}) }];
        let history = vec![
            Message::user_text("hello"),
            Message {
                role: Role::Assistant,
                content: vec![
                    ContentBlock::Text { text: "héllo".into() },
                    ContentBlock::ToolUse { id: "1".into(), name: "bash".into(), input: json!({"c":"ls"}) },
                ],
            },
            Message::tool_results(vec![ContentBlock::ToolResult { tool_use_id: "1".into(), content: "a\nb".into(), is_error: false }]),
            Message::user(vec![ContentBlock::image("image/png", "AAAA")]),
        ];
        let b = breakdown("sys", &tools, &history);
        assert_eq!(
            b,
            ContextBreakdown {
                system: 3,
                tools: 4 + 3 + 17,
                tool_count: 1,
                prompts: 5,
                replies: 6,
                tool_calls: 4 + 10,
                tool_call_count: 1,
                tool_results: 3,
                tool_result_count: 1,
                images: 1,
            }
        );
        assert_eq!(b.total(), 3 + 24 + 5 + 6 + 14 + 3);
    }

    #[test]
    fn render_rounds_up_and_shows_window_images_and_last_request() {
        let b = ContextBreakdown { system: 400, tools: 801, tool_count: 2, prompts: 3, images: 2, ..Default::default() };
        assert_eq!(estimate_tokens(0), 0);
        assert_eq!(estimate_tokens(5), 2);
        let out = render(&b, Some(1000), 512);
        assert_eq!(
            out,
            "context estimate (about 4 bytes of text per token):\n\
             \x20 system prompt        ~100 tokens\n\
             \x20 tools (2)            ~201 tokens\n\
             \x20 your messages        ~1 tokens\n\
             \x20 replies              ~0 tokens\n\
             \x20 tool calls (0)       ~0 tokens\n\
             \x20 tool results (0)     ~0 tokens\n\
             \x20 images               2 (not counted)\n\
             \x20 total                ~301 tokens (30% of 1000)\n\
             last request: 512 input tokens as counted by the provider"
        );
        let bare = render(&ContextBreakdown::default(), None, 0);
        assert!(bare.ends_with("total                ~0 tokens"));
        assert!(!bare.contains("images") && !bare.contains("last request"));
    }
}
