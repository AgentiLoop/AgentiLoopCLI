use agentiloop_core::{Tool, ToolContext, ToolError, ToolResult};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;

const MAX_OUTPUT: usize = 30_000;

pub struct Bash;

/// Kills the command's whole process tree when dropped (Esc cancel, timeout),
/// so children of `sh -c` / `cmd /C` don't outlive it. `None` = disarmed.
struct KillTree(Option<u32>);

impl Drop for KillTree {
    fn drop(&mut self) {
        let Some(pid) = self.0 else { return };
        #[cfg(unix)]
        // SAFETY: plain syscall; the group id is the child's pid (process_group(0)),
        // which is still reserved because the child has not been reaped yet.
        unsafe {
            libc::kill(-(pid as i32), libc::SIGKILL);
        }
        #[cfg(windows)]
        let _ = std::process::Command::new("taskkill")
            .args(["/T", "/F", "/PID", &pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

/// Reads a pipe to the end; the bytes read so far stay in `buf` if this is dropped.
async fn drain(pipe: Option<impl AsyncRead + Unpin>, buf: &mut Vec<u8>) {
    if let Some(mut p) = pipe {
        let _ = p.read_to_end(buf).await;
    }
}

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
        cmd.current_dir(&ctx.cwd)
            .kill_on_drop(true)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // Own process group, so a cancel or timeout can kill everything `sh -c` started.
        #[cfg(unix)]
        cmd.process_group(0);

        let mut child = cmd.spawn()?;
        let mut tree = KillTree(child.id());
        let (out, err) = (child.stdout.take(), child.stderr.take());
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        let work = async {
            let mut reads = std::pin::pin!(async { tokio::join!(drain(out, &mut stdout), drain(err, &mut stderr)) });
            let mut reads_done = false;
            let status = loop {
                tokio::select! {
                    s = child.wait() => break s?,
                    _ = &mut reads, if !reads_done => reads_done = true,
                }
            };
            // Background grandchildren may keep the pipes open; wait for them at most a second.
            if !reads_done {
                let _ = tokio::time::timeout(Duration::from_secs(1), reads).await;
            }
            Ok::<_, std::io::Error>(status)
        };
        let status = tokio::time::timeout(Duration::from_secs(a.timeout_secs), work)
            .await
            .map_err(|_| ToolError::Failed(format!("timed out after {}s", a.timeout_secs)))??;
        // Finished on its own: leave any background processes it started alone.
        tree.0 = None;

        let mut s = String::new();
        s.push_str(&String::from_utf8_lossy(&stdout));
        if !stderr.is_empty() {
            if !s.is_empty() {
                s.push('\n');
            }
            s.push_str(&String::from_utf8_lossy(&stderr));
        }
        if s.len() > MAX_OUTPUT {
            let cut = s.floor_char_boundary(MAX_OUTPUT);
            s.truncate(cut);
            s.push_str("\n…[truncated]");
        }
        if !status.success() {
            s.push_str(&format!("\n[exit status: {}]", status.code().unwrap_or(-1)));
        }
        Ok(s)
    }
}
