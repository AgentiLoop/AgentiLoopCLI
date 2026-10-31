//! `/history [text]`: search the saved prompts (the ↑ history shared by the REPL and the TUI).

/// Most matches `/history` prints (the newest ones).
pub const HISTORY_SHOWN: usize = 20;

/// Longest prompt preview, in characters.
const PREVIEW_CHARS: usize = 100;

/// `/history` → `Some("")`, `/history foo bar` → `Some("foo bar")`, anything else → `None`.
pub fn history_query(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix("/history")?;
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }
    Some(rest.trim())
}

/// The saved prompts containing `query` (case-insensitive; empty = all), oldest first,
/// the last `HISTORY_SHOWN` of them. `/history` lines themselves are left out.
pub fn history_listing(entries: &[String], query: &str) -> String {
    let needle = query.to_lowercase();
    let matches: Vec<&String> =
        entries.iter().filter(|e| history_query(e).is_none() && e.to_lowercase().contains(&needle)).collect();
    if matches.is_empty() {
        return if query.is_empty() { "no saved prompts yet".to_string() } else { format!("no saved prompt contains \"{query}\"") };
    }
    let shown = &matches[matches.len().saturating_sub(HISTORY_SHOWN)..];
    let what = if query.is_empty() { String::new() } else { format!(" containing \"{query}\"") };
    let count = if shown.len() < matches.len() { format!("last {} of {}", shown.len(), matches.len()) } else { matches.len().to_string() };
    let mut out = format!("{count} saved prompt(s){what}, oldest first (↑ recalls them):");
    for e in shown {
        out.push_str("\n  ");
        out.push_str(&preview(e));
    }
    out
}

/// First line of `prompt`, cut to `PREVIEW_CHARS`; `…` marks anything left out.
fn preview(prompt: &str) -> String {
    let first = prompt.lines().next().unwrap_or("");
    let mut out: String = first.chars().take(PREVIEW_CHARS).collect();
    if first.chars().count() > PREVIEW_CHARS || prompt.trim_end().len() > first.len() {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn query_parsing() {
        assert_eq!(history_query("/history"), Some(""));
        assert_eq!(history_query("  /history   fix the  build "), Some("fix the  build"));
        assert_eq!(history_query("/historyx"), None);
        assert_eq!(history_query("history"), None);
        assert_eq!(history_query("/help"), None);
    }

    #[test]
    fn empty_and_no_match() {
        assert_eq!(history_listing(&[], ""), "no saved prompts yet");
        assert_eq!(history_listing(&v(&["/history"]), ""), "no saved prompts yet");
        assert_eq!(history_listing(&v(&["hello"]), "bye"), "no saved prompt contains \"bye\"");
    }

    #[test]
    fn filters_case_insensitively_and_skips_history_lines() {
        let entries = v(&["Fix the build", "/history fix", "write tests", "fix lint\nand more"]);
        assert_eq!(history_listing(&entries, "FIX"), "2 saved prompt(s) containing \"FIX\", oldest first (↑ recalls them):\n  Fix the build\n  fix lint…");
        assert_eq!(history_listing(&entries, ""), "3 saved prompt(s), oldest first (↑ recalls them):\n  Fix the build\n  write tests\n  fix lint…");
    }

    #[test]
    fn keeps_the_newest_and_cuts_long_prompts() {
        let entries: Vec<String> = (1..=25).map(|i| format!("prompt {i}")).collect();
        let out = history_listing(&entries, "");
        assert!(out.starts_with("last 20 of 25 saved prompt(s), oldest first"));
        assert!(out.contains("\n  prompt 6\n") && out.ends_with("\n  prompt 25") && !out.contains("prompt 5\n"));
        let long = "é".repeat(150);
        let out = history_listing(&[long], "");
        assert!(out.ends_with(&format!("\n  {}…", "é".repeat(100))));
    }
}
