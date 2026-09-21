use std::path::{Path, PathBuf};

use agentiloop_core::{Tool, ToolContext, ToolError, ToolResult};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

fn resolve(ctx: &ToolContext, p: &str) -> PathBuf {
    let path = Path::new(p);
    if path.is_absolute() { path.to_path_buf() } else { ctx.cwd.join(path) }
}

fn parse<T: for<'de> Deserialize<'de>>(input: Value) -> Result<T, ToolError> {
    serde_json::from_value(input).map_err(|e| ToolError::InvalidInput(e.to_string()))
}

// ---------------------------------------------------------------- read_file

pub struct ReadFile;

#[derive(Deserialize)]
struct ReadArgs {
    path: String,
}

#[async_trait]
impl Tool for ReadFile {
    fn name(&self) -> &str { "read_file" }
    fn description(&self) -> &str { "Read a UTF-8 text file. Returns the full contents with 1-based line numbers." }
    fn input_schema(&self) -> Value {
        json!({"type":"object","properties":{"path":{"type":"string","description":"File path (absolute or relative to cwd)"}},"required":["path"]})
    }
    async fn call(&self, ctx: &ToolContext, input: Value) -> ToolResult {
        let a: ReadArgs = parse(input)?;
        let text = tokio::fs::read_to_string(resolve(ctx, &a.path)).await?;
        Ok(text
            .lines()
            .enumerate()
            .map(|(i, l)| format!("{:>5}\u{2502}{}", i + 1, l))
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

// --------------------------------------------------------------- write_file

pub struct WriteFile;

#[derive(Deserialize)]
struct WriteArgs {
    path: String,
    content: String,
}

#[async_trait]
impl Tool for WriteFile {
    fn name(&self) -> &str { "write_file" }
    fn description(&self) -> &str { "Create or overwrite a file with the given content. Creates parent directories." }
    fn input_schema(&self) -> Value {
        json!({"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]})
    }
    fn is_mutating(&self) -> bool { true }
    async fn call(&self, ctx: &ToolContext, input: Value) -> ToolResult {
        let a: WriteArgs = parse(input)?;
        let path = resolve(ctx, &a.path);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&path, &a.content).await?;
        Ok(format!("wrote {} bytes to {}", a.content.len(), path.display()))
    }
}

// ---------------------------------------------------------------- edit_file

pub struct EditFile;

#[derive(Deserialize)]
struct EditArgs {
    path: String,
    old_string: String,
    new_string: String,
    #[serde(default)]
    replace_all: bool,
}

#[async_trait]
impl Tool for EditFile {
    fn name(&self) -> &str { "edit_file" }
    fn description(&self) -> &str {
        "Replace an exact string in a file. `old_string` must match exactly once unless `replace_all` is true."
    }
    fn input_schema(&self) -> Value {
        json!({"type":"object","properties":{
            "path":{"type":"string"},
            "old_string":{"type":"string"},
            "new_string":{"type":"string"},
            "replace_all":{"type":"boolean","default":false}
        },"required":["path","old_string","new_string"]})
    }
    fn is_mutating(&self) -> bool { true }
    async fn call(&self, ctx: &ToolContext, input: Value) -> ToolResult {
        let a: EditArgs = parse(input)?;
        let path = resolve(ctx, &a.path);
        let text = tokio::fs::read_to_string(&path).await?;
        let n = text.matches(&a.old_string).count();
        if n == 0 {
            return Err(ToolError::Failed("old_string not found".into()));
        }
        if n > 1 && !a.replace_all {
            return Err(ToolError::Failed(format!("old_string matched {n} times; add context or set replace_all")));
        }
        let out = if a.replace_all {
            text.replace(&a.old_string, &a.new_string)
        } else {
            text.replacen(&a.old_string, &a.new_string, 1)
        };
        tokio::fs::write(&path, out).await?;
        Ok(format!("replaced {n} occurrence(s) in {}", path.display()))
    }
}

// ----------------------------------------------------------------- list_dir

pub struct ListDir;

#[derive(Deserialize)]
struct ListArgs {
    #[serde(default = "default_dot")]
    path: String,
}

fn default_dot() -> String {
    ".".into()
}

#[async_trait]
impl Tool for ListDir {
    fn name(&self) -> &str { "list_dir" }
    fn description(&self) -> &str { "List entries in a directory. Directories are suffixed with '/'." }
    fn input_schema(&self) -> Value {
        json!({"type":"object","properties":{"path":{"type":"string","default":"."}}})
    }
    async fn call(&self, ctx: &ToolContext, input: Value) -> ToolResult {
        let a: ListArgs = parse(input)?;
        let mut rd = tokio::fs::read_dir(resolve(ctx, &a.path)).await?;
        let mut names = Vec::new();
        while let Some(e) = rd.next_entry().await? {
            let mut n = e.file_name().to_string_lossy().into_owned();
            if e.file_type().await.map(|t| t.is_dir()).unwrap_or(false) {
                n.push('/');
            }
            names.push(n);
        }
        names.sort();
        Ok(names.join("\n"))
    }
}
