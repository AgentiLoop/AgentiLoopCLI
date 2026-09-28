use std::collections::HashSet;
use std::io::{self, BufRead, Write};
use std::sync::Arc;

use agentiloop_core::permission::{AllowAll, Permission, PermissionPolicy, SharedPolicy};
use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::Mutex;

/// What the REPL shares with its permission prompts: the line editor that owns
/// stdin (None outside the REPL), and the signal that Ctrl-C at a prompt
/// cancels the whole request.
#[derive(Default)]
pub struct ReplIo {
    pub editor: std::sync::Mutex<Option<rustyline::DefaultEditor>>,
    pub cancel: tokio::sync::Notify,
}

pub fn policy(yes: bool, io: Arc<ReplIo>) -> SharedPolicy {
    if yes { Arc::new(AllowAll) } else { Arc::new(Interactive { always: Mutex::default(), io }) }
}

/// Prompts on stderr for every mutating tool call. `a` = always allow this tool for the session.
struct Interactive {
    always: Mutex<HashSet<String>>,
    io: Arc<ReplIo>,
}

#[async_trait]
impl PermissionPolicy for Interactive {
    async fn check(&self, tool: &str, is_mutating: bool, input: &Value) -> Permission {
        if !is_mutating || self.always.lock().await.contains(tool) {
            return Permission::Allow;
        }

        eprintln!("\n\u{26a0} {tool} wants to run:\n{}", serde_json::to_string_pretty(input).unwrap_or_default());
        let question = format!("Allow? [y]es / [n]o / [a]lways for `{tool}`: ");
        let answer = self.io.editor.lock().unwrap().as_mut().map(|rl| rl.readline(&question));
        let line = match answer {
            Some(Ok(line)) => line,
            // Ctrl-C at the prompt stops the whole request: the REPL drops this run.
            Some(Err(rustyline::error::ReadlineError::Interrupted)) => {
                self.io.cancel.notify_one();
                return std::future::pending().await;
            }
            Some(Err(_)) => String::new(),
            None => {
                eprint!("{question}");
                let _ = io::stderr().flush();
                let mut line = String::new();
                let _ = io::stdin().lock().read_line(&mut line);
                line
            }
        };
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
