//! `web_fetch`: download a web page or text document and return it as plain text.

use std::sync::OnceLock;
use std::time::Duration;

use agentiloop_core::{Tool, ToolContext, ToolError, ToolResult};
use async_trait::async_trait;
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};

const DEFAULT_MAX_CHARS: usize = 30_000;
const MAX_MAX_CHARS: usize = 100_000;
/// Bytes downloaded before giving up on the rest of the body.
const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(30);

fn re(cell: &'static OnceLock<Regex>, pattern: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).unwrap())
}

/// Readable text from HTML: drops scripts, styles and comments, turns block tags into line breaks,
/// strips the remaining tags, decodes common entities and squeezes whitespace.
pub fn html_to_text(html: &str) -> String {
    static SCRIPT: OnceLock<Regex> = OnceLock::new();
    static STYLE: OnceLock<Regex> = OnceLock::new();
    static COMMENT: OnceLock<Regex> = OnceLock::new();
    static BLOCK: OnceLock<Regex> = OnceLock::new();
    static TAG: OnceLock<Regex> = OnceLock::new();
    static NUMERIC: OnceLock<Regex> = OnceLock::new();
    static SPACES: OnceLock<Regex> = OnceLock::new();

    let s = re(&SCRIPT, r"(?is)<script\b.*?</script\s*>").replace_all(html, "");
    let s = re(&STYLE, r"(?is)<style\b.*?</style\s*>").replace_all(&s, "");
    let s = re(&COMMENT, r"(?s)<!--.*?-->").replace_all(&s, "");
    let s = re(&BLOCK, r"(?i)</?(?:p|div|br|li|tr|h[1-6]|ul|ol|table|section|article|header|footer|pre|blockquote)\b[^>]*>")
        .replace_all(&s, "\n");
    let s = re(&TAG, r"<[^>]*>").replace_all(&s, "");
    let s = re(&NUMERIC, r"&#([xX][0-9a-fA-F]+|[0-9]+);").replace_all(&s, |c: &regex::Captures| {
        let n = &c[1];
        let code = match n.strip_prefix(['x', 'X']) {
            Some(h) => u32::from_str_radix(h, 16),
            None => n.parse(),
        };
        code.ok().and_then(char::from_u32).map(String::from).unwrap_or_else(|| c[0].to_string())
    });
    let s = s
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&");
    let mut out: Vec<String> = Vec::new();
    for line in s.lines() {
        let line = re(&SPACES, r"[ \t\u{a0}]+").replace_all(line, " ");
        let line = line.trim();
        if line.is_empty() && out.last().map_or(true, |l| l.is_empty()) {
            continue;
        }
        out.push(line.to_string());
    }
    out.join("\n").trim().to_string()
}

pub struct WebFetch;

#[derive(Deserialize)]
struct FetchArgs {
    url: String,
    max_chars: Option<usize>,
}

#[async_trait]
impl Tool for WebFetch {
    fn name(&self) -> &str { "web_fetch" }
    fn description(&self) -> &str {
        "Fetch an http(s) URL and return its text: HTML pages are converted to plain text, text, JSON and XML \
         are returned as they are. Follows redirects. Use it to read documentation or an API response. \
         Binary content is refused. Long pages are cut at `max_chars`."
    }
    fn input_schema(&self) -> Value {
        json!({"type":"object","properties":{
            "url":{"type":"string","description":"http:// or https:// URL"},
            "max_chars":{"type":"integer","description":"Maximum characters to return","default":30000}
        },"required":["url"]})
    }
    // Makes a network request on the user's behalf, so it asks first like the other non-local tools.
    fn is_mutating(&self) -> bool { true }
    async fn call(&self, _ctx: &ToolContext, input: Value) -> ToolResult {
        let a: FetchArgs = serde_json::from_value(input).map_err(|e| ToolError::InvalidInput(e.to_string()))?;
        let url = reqwest::Url::parse(a.url.trim()).map_err(|e| ToolError::InvalidInput(format!("bad url: {e}")))?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(ToolError::InvalidInput(format!("only http and https URLs are supported, not {}", url.scheme())));
        }
        let limit = a.max_chars.unwrap_or(DEFAULT_MAX_CHARS).clamp(1, MAX_MAX_CHARS);
        let client = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .user_agent(concat!("agentiloop/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| ToolError::Failed(e.to_string()))?;
        let mut resp = client.get(url.clone()).send().await.map_err(|e| ToolError::Failed(format!("request failed: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(ToolError::Failed(format!("HTTP {status} for {url}")));
        }
        let ctype = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_ascii_lowercase();
        let textual = ctype.is_empty()
            || ["text/", "json", "xml", "javascript", "yaml", "x-sh", "markdown"].iter().any(|t| ctype.contains(t));
        if !textual {
            return Err(ToolError::Failed(format!("unsupported content type {ctype}; web_fetch only reads text")));
        }
        let mut body: Vec<u8> = Vec::new();
        let mut body_cut = false;
        while let Some(chunk) = resp.chunk().await.map_err(|e| ToolError::Failed(format!("download failed: {e}")))? {
            body.extend_from_slice(&chunk);
            if body.len() >= MAX_BODY_BYTES {
                body.truncate(MAX_BODY_BYTES);
                body_cut = true;
                break;
            }
        }
        let raw = String::from_utf8_lossy(&body).into_owned();
        let is_html = ctype.contains("html") || (ctype.is_empty() && raw.trim_start().to_ascii_lowercase().starts_with("<!doctype html"));
        let mut text = if is_html { html_to_text(&raw) } else { raw };
        let over = text.chars().count() > limit;
        if over {
            text = text.chars().take(limit).collect();
        }
        if over || body_cut {
            text.push_str(&format!("\n…[truncated at {limit} characters]"));
        }
        Ok(format!("{url} ({status})\n\n{text}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_becomes_readable_text() {
        let html = "<!doctype html><html><head><title>T</title><style>p{color:red}</style>\
<script>var x = '<p>no</p>';</script></head><body><!-- hidden -->\
<h1>Hello &amp; welcome</h1><p>One  <b>bold</b>\ttext&nbsp;here.</p><ul><li>a &lt;b&gt;</li><li>&#65;&#x42;&quot;</li></ul>\
<div><br></div><div>end</div></body></html>";
        assert_eq!(html_to_text(html), "T\nHello & welcome\n\nOne bold text here.\n\na <b>\n\nAB\"\n\nend");
    }

    #[test]
    fn unknown_entities_survive() {
        assert_eq!(html_to_text("<p>&bogus; &#99999999999;</p>"), "&bogus; &#99999999999;");
    }
}
