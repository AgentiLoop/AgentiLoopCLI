//! Stdio transport: spawn the server, newline-delimited JSON-RPC over its pipes.

use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{oneshot, Mutex};

use crate::transport::{await_response, dispatch, rpc_notification, rpc_request, Pending, Transport};

/// A single message larger than this (10 MB) disconnects the server.
const MAX_LINE: usize = 10 * 1024 * 1024;

pub struct StdioTransport {
    stdin: Mutex<ChildStdin>,
    child: StdMutex<Child>,
    pending: Pending,
    next_id: AtomicI64,
    alive: Arc<AtomicBool>,
}

impl StdioTransport {
    pub fn spawn(program: &str, args: &[String], env: &HashMap<String, String>, cwd: &Path) -> Result<Self> {
        let mut cmd = Command::new(program);
        cmd.args(args)
            .envs(env)
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let mut child = cmd.spawn().with_context(|| format!("failed to launch `{program}`"))?;
        let stdin = child.stdin.take().context("no stdin")?;
        let stdout = child.stdout.take().context("no stdout")?;
        let stderr = child.stderr.take().context("no stderr")?;

        // Drain stderr so a chatty server never blocks on a full pipe.
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(l)) = lines.next_line().await {
                tracing::debug!(target: "mcp", "{l}");
            }
        });

        let pending: Pending = Default::default();
        let alive = Arc::new(AtomicBool::new(true));
        {
            let (pending, alive) = (pending.clone(), alive.clone());
            tokio::spawn(async move {
                let mut reader = BufReader::new(stdout);
                let mut buf = Vec::new();
                loop {
                    buf.clear();
                    match reader.read_until(b'\n', &mut buf).await {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {}
                    }
                    if buf.len() > MAX_LINE {
                        tracing::warn!("MCP server sent a message over 10 MB; disconnecting");
                        break;
                    }
                    if let Ok(msg) = serde_json::from_slice::<Value>(&buf) {
                        dispatch(&pending, msg);
                    }
                }
                alive.store(false, Ordering::SeqCst);
                pending.lock().unwrap().clear(); // fail every waiter
            });
        }

        Ok(Self { stdin: Mutex::new(stdin), child: StdMutex::new(child), pending, next_id: AtomicI64::new(0), alive })
    }

    async fn write(&self, msg: &Value) -> Result<()> {
        let mut line = serde_json::to_vec(msg)?;
        line.push(b'\n');
        let mut w = self.stdin.lock().await;
        w.write_all(&line).await?;
        w.flush().await?;
        Ok(())
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        anyhow::ensure!(self.is_alive(), "MCP server process is no longer running");
        let id = self.next_id.fetch_add(1, Ordering::SeqCst) + 1;
        // Register before writing so a fast reply can't be missed.
        let (tx, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(id, tx);
        if let Err(e) = self.write(&rpc_request(id, method, params)).await {
            self.pending.lock().unwrap().remove(&id);
            anyhow::bail!("write failed: {e}");
        }
        await_response(&self.pending, id, rx, method).await
    }

    async fn notify(&self, method: &str, params: Option<Value>) -> Result<()> {
        self.write(&rpc_notification(method, params)).await
    }

    fn is_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst) && matches!(self.child.lock().unwrap().try_wait(), Ok(None))
    }

    async fn close(&self) {
        self.alive.store(false, Ordering::SeqCst);
        self.pending.lock().unwrap().clear();
        let _ = self.child.lock().unwrap().start_kill();
    }
}
