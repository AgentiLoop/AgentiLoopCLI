use agentiloop_core::{Tool, ToolContext, ToolError, ToolResult};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::process::Command;

const MAX_OUTPUT: usize = 30_000;

pub struct Bash;

#[derive(Deserialize)]
struct BashArgs {
    command: String,
    #[serde(default = "default_timeout")]
    timeout_secs: u64,
}

fn default_timeout() -> u64 {
    120
}

#[async_trait]
impl Tool for Bash {
    fn name(&self) -> &str { "bash" }
    fn description(&self) -> &str {
        "Run a shell command in the project directory and return stdout+stderr. \
         Uses `sh -c` on Unix and `cmd /C` on Windows."
    }
    fn input_schema(&self) -> Value {
        json!({"type":"object","properties":{
            "command":{"type":"string"},
            "timeout_secs":{"type":"integer","default":120}
        },"required":["command"]})
    }
    fn is_mutating(&self) -> bool { true }

    async fn call(&self, ctx: &ToolContext, input: Value) -> ToolResult {
        let a: BashArgs = serde_json::from_value(input).map_err(|e| ToolError::InvalidInput(e.to_string()))?;

        let mut cmd = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.args(["/C", &a.command]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", &a.command]);
            c
        };
        cmd.current_dir(&ctx.cwd).kill_on_drop(true);

        let fut = cmd.output();
        let out = tokio::time::timeout(std::time::Duration::from_secs(a.timeout_secs), fut)
            .await
            .map_err(|_| ToolError::Failed(format!("timed out after {}s", a.timeout_secs)))??;

        let mut s = String::new();
        s.push_str(&String::from_utf8_lossy(&out.stdout));
        if !out.stderr.is_empty() {
            if !s.is_empty() {
                s.push('\n');
            }
            s.push_str(&String::from_utf8_lossy(&out.stderr));
        }
        if s.len() > MAX_OUTPUT {
            let cut = s.floor_char_boundary(MAX_OUTPUT);
            s.truncate(cut);
            s.push_str("\n…[truncated]");
        }
        if !out.status.success() {
            s.push_str(&format!("\n[exit status: {}]", out.status.code().unwrap_or(-1)));
        }
        Ok(s)
    }
}
