//! `todo_write`: the model's task checklist for multi-step work. Each call replaces the whole list;
//! `/todos` shows it to the user. The list lives in memory for the process (cleared by `/clear`).

use std::sync::Mutex;

use agentiloop_core::{Tool, ToolContext, ToolError, ToolResult};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Pending,
    InProgress,
    Completed,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
struct Item {
    content: String,
    status: Status,
}

static LIST: Mutex<Vec<Item>> = Mutex::new(Vec::new());

fn lock() -> std::sync::MutexGuard<'static, Vec<Item>> {
    LIST.lock().unwrap_or_else(|e| e.into_inner())
}

fn render(items: &[Item]) -> String {
    if items.is_empty() {
        return "todo list is empty".to_string();
    }
    let done = items.iter().filter(|i| i.status == Status::Completed).count();
    let mut out = format!("todo list ({done}/{} done):", items.len());
    for i in items {
        let mark = match i.status {
            Status::Pending => "[ ]",
            Status::InProgress => "[~]",
            Status::Completed => "[x]",
        };
        out.push_str(&format!("\n{mark} {}", i.content));
    }
    out
}

/// Checks a proposed list: non-empty text and at most one item in progress.
fn validate(items: &[Item]) -> Result<(), String> {
    if items.iter().any(|i| i.content.trim().is_empty()) {
        return Err("every todo needs non-empty `content`".into());
    }
    if items.iter().filter(|i| i.status == Status::InProgress).count() > 1 {
        return Err("only one todo may be in_progress at a time".into());
    }
    Ok(())
}

/// The current list as text for `/todos`; `None` when it is empty.
pub fn current() -> Option<String> {
    let items = lock();
    if items.is_empty() { None } else { Some(render(&items)) }
}

/// Forgets the list (used by `/clear`).
pub fn clear() {
    lock().clear();
}

pub struct TodoWrite;

#[derive(Deserialize)]
struct Args {
    todos: Vec<Item>,
}

#[async_trait]
impl Tool for TodoWrite {
    fn name(&self) -> &str { "todo_write" }
    fn description(&self) -> &str {
        "Keep a checklist for multi-step tasks. Each call REPLACES the whole list, so send every item every time. \
         Mark an item in_progress before starting it (at most one at a time) and completed as soon as it is done. \
         Use it for work with three or more steps; skip it for simple requests."
    }
    fn input_schema(&self) -> Value {
        json!({"type":"object","properties":{
            "todos":{"type":"array","description":"The complete todo list","items":{"type":"object","properties":{
                "content":{"type":"string","description":"What needs doing, in the imperative"},
                "status":{"type":"string","enum":["pending","in_progress","completed"]}
            },"required":["content","status"]}}
        },"required":["todos"]})
    }
    async fn call(&self, _ctx: &ToolContext, input: Value) -> ToolResult {
        let a: Args = serde_json::from_value(input).map_err(|e| ToolError::InvalidInput(e.to_string()))?;
        validate(&a.todos).map_err(ToolError::InvalidInput)?;
        let text = render(&a.todos);
        *lock() = a.todos;
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(c: &str, s: Status) -> Item {
        Item { content: c.into(), status: s }
    }

    #[test]
    fn renders_marks_and_progress() {
        assert_eq!(render(&[]), "todo list is empty");
        let items = [item("a", Status::Completed), item("b", Status::InProgress), item("c", Status::Pending)];
        assert_eq!(render(&items), "todo list (1/3 done):\n[x] a\n[~] b\n[ ] c");
    }

    #[test]
    fn validation_rejects_blank_and_double_in_progress() {
        assert!(validate(&[item("a", Status::InProgress), item("b", Status::Pending)]).is_ok());
        assert!(validate(&[item("  ", Status::Pending)]).is_err());
        assert!(validate(&[item("a", Status::InProgress), item("b", Status::InProgress)]).is_err());
    }

    // The only test touching the shared list, so it cannot race with the others.
    #[tokio::test]
    async fn call_replaces_the_list_and_bad_input_is_rejected() {
        let ctx = ToolContext { cwd: std::env::temp_dir() };
        clear();
        assert!(current().is_none());
        let out = TodoWrite
            .call(&ctx, json!({"todos":[{"content":"x","status":"in_progress"},{"content":"y","status":"pending"}]}))
            .await
            .unwrap();
        assert!(out.contains("[~] x") && out.contains("[ ] y"), "{out}");
        assert_eq!(current().unwrap(), out);
        TodoWrite.call(&ctx, json!({"todos":[{"content":"z","status":"completed"}]})).await.unwrap();
        assert_eq!(current().unwrap(), "todo list (1/1 done):\n[x] z");

        assert!(TodoWrite.call(&ctx, json!({"todos":[{"content":"z","status":"bogus"}]})).await.is_err());
        assert!(TodoWrite.call(&ctx, json!({})).await.is_err());
        assert!(TodoWrite
            .call(&ctx, json!({"todos":[{"content":"a","status":"in_progress"},{"content":"b","status":"in_progress"}]}))
            .await
            .is_err());
        assert_eq!(current().unwrap(), "todo list (1/1 done):\n[x] z", "failed calls leave the list alone");
        clear();
    }
}
