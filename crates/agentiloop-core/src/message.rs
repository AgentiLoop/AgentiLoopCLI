use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String, input: Value },
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        is_error: bool,
    },
    /// A pasted or `@mentioned` image, stored in the Anthropic shape
    /// (`{"type":"image","source":{"type":"base64","media_type":…,"data":…}}`).
    Image { source: ImageSource },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub kind: String,
    pub media_type: String,
    /// Base64 of the file bytes.
    pub data: String,
}

impl ContentBlock {
    pub fn image(media_type: impl Into<String>, data: impl Into<String>) -> Self {
        Self::Image { source: ImageSource { kind: "base64".into(), media_type: media_type.into(), data: data.into() } }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
}

impl Message {
    pub fn user_text(text: impl Into<String>) -> Self {
        Self { role: Role::User, content: vec![ContentBlock::Text { text: text.into() }] }
    }

    pub fn user(content: Vec<ContentBlock>) -> Self {
        Self { role: Role::User, content }
    }

    pub fn tool_results(results: Vec<ContentBlock>) -> Self {
        Self { role: Role::User, content: results }
    }

    pub fn tool_uses(&self) -> impl Iterator<Item = (&str, &str, &Value)> {
        self.content.iter().filter_map(|b| match b {
            ContentBlock::ToolUse { id, name, input } => Some((id.as_str(), name.as_str(), input)),
            _ => None,
        })
    }

    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|b| match b {
                ContentBlock::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

/// The text of the `n`th assistant reply counting back from the newest
/// (1 = the last one); replies that are only tool calls don't count.
pub fn nth_reply(history: &[Message], n: usize) -> Option<String> {
    let mut replies = history.iter().rev().filter(|m| m.role == Role::Assistant).map(|m| m.text()).filter(|t| !t.trim().is_empty());
    replies.nth(n.checked_sub(1)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assistant(blocks: Vec<ContentBlock>) -> Message {
        Message { role: Role::Assistant, content: blocks }
    }

    #[test]
    fn nth_reply_counts_back_over_text_replies_only() {
        let text = |s: &str| ContentBlock::Text { text: s.into() };
        let call = ContentBlock::ToolUse { id: "1".into(), name: "bash".into(), input: Value::Null };
        let history = vec![
            Message::user_text("hi"),
            assistant(vec![text("first")]),
            Message::user_text("more"),
            assistant(vec![text("second "), call.clone(), text("part")]),
            Message::tool_results(vec![]),
            assistant(vec![call]),
            assistant(vec![text("  ")]),
        ];
        assert_eq!(nth_reply(&history, 1).as_deref(), Some("second part"));
        assert_eq!(nth_reply(&history, 2).as_deref(), Some("first"));
        assert_eq!(nth_reply(&history, 3), None);
        assert_eq!(nth_reply(&history, 0), None);
        assert_eq!(nth_reply(&[], 1), None);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    ToolUse,
    MaxTokens,
    StopSequence,
    /// The provider refused the turn (Anthropic `stop_reason: "refusal"`).
    Refusal,
    Other,
}
