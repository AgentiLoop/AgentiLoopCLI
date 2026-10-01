use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Allow,
    Deny,
    /// Skip just this call (e.g. Esc in the TUI); the agent keeps working.
    Cancel,
}

/// Decides whether a tool call may run. The CLI supplies an interactive
/// implementation; tests/CI can supply `AllowAll`.
#[async_trait]
pub trait PermissionPolicy: Send + Sync {
    async fn check(&self, tool: &str, is_mutating: bool, input: &Value) -> Permission;
}

pub struct AllowAll;

#[async_trait]
impl PermissionPolicy for AllowAll {
    async fn check(&self, _tool: &str, _is_mutating: bool, _input: &Value) -> Permission {
        Permission::Allow
    }
}

/// Name patterns for `--allow-tool` / `--deny-tool`: an exact tool name, or a prefix ending in `*`
/// (`mcp_*`); `*` alone matches every tool.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolPatterns(Vec<String>);

impl ToolPatterns {
    pub fn new(patterns: &[String]) -> Self {
        Self(patterns.iter().flat_map(|p| p.split(',')).map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn matches(&self, tool: &str) -> bool {
        self.0.iter().any(|p| match p.strip_suffix('*') {
            Some(prefix) => tool.starts_with(prefix),
            None => p == tool,
        })
    }
}

/// Applies `--deny-tool` (always refused) and `--allow-tool` (never asked) before the wrapped policy.
/// Deny wins when a tool matches both.
pub struct Rules {
    pub allow: ToolPatterns,
    pub deny: ToolPatterns,
    pub inner: SharedPolicy,
}

#[async_trait]
impl PermissionPolicy for Rules {
    async fn check(&self, tool: &str, is_mutating: bool, input: &Value) -> Permission {
        if self.deny.matches(tool) {
            Permission::Deny
        } else if self.allow.matches(tool) {
            Permission::Allow
        } else {
            self.inner.check(tool, is_mutating, input).await
        }
    }
}

pub type SharedPolicy = Arc<dyn PermissionPolicy>;

#[cfg(test)]
mod tests {
    use super::*;

    fn pats(p: &[&str]) -> ToolPatterns {
        ToolPatterns::new(&p.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn patterns_match_exact_prefix_and_all_and_split_commas() {
        let p = pats(&["bash", "mcp_*, edit_file", " "]);
        assert!(p.matches("bash") && p.matches("mcp_Local_echo") && p.matches("edit_file"));
        assert!(!p.matches("bash2") && !p.matches("write_file") && !p.matches("mcp"));
        assert!(pats(&["*"]).matches("anything"));
        assert!(pats(&[]).is_empty() && !pats(&[]).matches("bash"));
    }

    struct Ask;
    #[async_trait]
    impl PermissionPolicy for Ask {
        async fn check(&self, _: &str, _: bool, _: &Value) -> Permission {
            Permission::Cancel
        }
    }

    #[tokio::test]
    async fn deny_beats_allow_and_unlisted_tools_reach_the_inner_policy() {
        let r = Rules { allow: pats(&["write_file", "bash"]), deny: pats(&["bash", "web_*"]), inner: Arc::new(Ask) };
        let v = serde_json::json!({});
        assert_eq!(r.check("bash", true, &v).await, Permission::Deny);
        assert_eq!(r.check("web_fetch", true, &v).await, Permission::Deny);
        assert_eq!(r.check("write_file", true, &v).await, Permission::Allow);
        assert_eq!(r.check("edit_file", true, &v).await, Permission::Cancel);
    }
}
