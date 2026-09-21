use std::collections::HashSet;
use std::io::{self, BufRead, Write};
use std::sync::Arc;

use agentiloop_core::permission::{AllowAll, Permission, PermissionPolicy, SharedPolicy};
use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::Mutex;

pub fn policy(yes: bool) -> SharedPolicy {
    if yes { Arc::new(AllowAll) } else { Arc::new(Interactive::default()) }
}

/// Prompts on stderr for every mutating tool call. `a` = always allow this tool for the session.
#[derive(Default)]
struct Interactive {
    always: Mutex<HashSet<String>>,
}

#[async_trait]
impl PermissionPolicy for Interactive {
    async fn check(&self, tool: &str, is_mutating: bool, input: &Value) -> Permission {
        if !is_mutating || self.always.lock().await.contains(tool) {
            return Permission::Allow;
        }

        eprintln!("\n\u{26a0} {tool} wants to run:\n{}", serde_json::to_string_pretty(input).unwrap_or_default());
        eprint!("Allow? [y]es / [n]o / [a]lways for `{tool}`: ");
        let _ = io::stderr().flush();

        let mut line = String::new();
        let _ = io::stdin().lock().read_line(&mut line);
        match line.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => Permission::Allow,
            "a" | "always" => {
                self.always.lock().await.insert(tool.to_string());
                Permission::Allow
            }
            _ => Permission::Deny,
        }
    }
}
