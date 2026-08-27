use crate::error::{PluginError, PluginResult};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[cfg(feature = "bun-sidecar")]
use std::sync::Arc;

#[cfg(feature = "bun-sidecar")]
use std::process::Stdio;
#[cfg(feature = "bun-sidecar")]
use std::time::Duration;
#[cfg(feature = "bun-sidecar")]
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[cfg(feature = "bun-sidecar")]
use tokio::process::{Child, ChildStdin, Command};

/// JSON-line IPC messages — Rust → Sidecar.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SidecarOutbound {
    Init {
        config: serde_json::Value,
        plugins: Vec<String>,
    },
    SessionStart {
        session_id: String,
    },
    SessionMessage {
        session_id: String,
        content: String,
    },
    TuiRender {
        frame: serde_json::Value,
    },
    InvokeHook {
        request_id: String,
        plugin: String,
        hook: String,
        payload: serde_json::Value,
    },
    Shutdown,
}

/// JSON-line IPC messages — Sidecar → Rust.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SidecarInbound {
    Ready,
    TuiRender {
        frame: serde_json::Value,
    },
    PluginEvent {
        plugin: String,
        event: serde_json::Value,
    },
    HookResult {
        request_id: String,
        plugin: String,
        output: serde_json::Value,
    },
    Error {
        message: String,
    },
    Log {
        level: String,
        message: String,
    },
}

#[async_trait::async_trait]
pub trait SidecarPort: Send + Sync {
    async fn send(&self, message: SidecarOutbound) -> PluginResult<()>;
    async fn recv(&self) -> PluginResult<SidecarInbound>;
}

/// In-memory SidecarPort for tests (Layer 4 / sidecar contract tests).
#[derive(Debug, Default)]
pub struct InMemorySidecarPort {
    outbound: Mutex<Vec<SidecarOutbound>>,
    inbound: Mutex<Vec<SidecarInbound>>,
}

impl InMemorySidecarPort {
    pub fn new() -> Self {
        Self {
            outbound: Mutex::new(Vec::new()),
            inbound: Mutex::new(vec![SidecarInbound::Ready]),
        }
    }

    pub async fn recorded_outbound(&self) -> Vec<SidecarOutbound> {
        self.outbound.lock().await.clone()
    }

    pub async fn push_inbound(&self, message: SidecarInbound) {
        self.inbound.lock().await.push(message);
    }
}

#[async_trait::async_trait]
impl SidecarPort for InMemorySidecarPort {
    async fn send(&self, message: SidecarOutbound) -> PluginResult<()> {
        // Mirror process sidecar: Init loads plugins then emits Ready.
        if let SidecarOutbound::Init { .. } = &message {
            self.inbound.lock().await.push(SidecarInbound::Ready);
        }
        // Auto-respond to hook invokes so BunPluginHost request/response works in tests.
        if let SidecarOutbound::InvokeHook {
            request_id,
            plugin,
            hook,
            payload,
        } = &message
        {
            // Honest in-memory stub: passthrough only. Real Bun rewrite is proven via
            // BunProcessSidecarPort + first-party plugin e2e (not this fake host).
            let output = if hook == "tool.execute.before" {
                let command = payload
                    .get("command")
                    .and_then(|v| v.as_str())
                    .or_else(|| {
                        payload
                            .get("args")
                            .and_then(|a| a.get("command"))
                            .and_then(|v| v.as_str())
                    })
                    .unwrap_or("");
                serde_json::json!({
                    "host": "bun",
                    "hook": hook,
                    "tool": "bash",
                    "command": command,
                    "args": { "command": command },
                    "rewritten": false,
                    "stub": true,
                    "status": "ok",
                })
            } else {
                let transformed = payload
                    .get("input")
                    .cloned()
                    .unwrap_or_else(|| payload.clone());
                serde_json::json!({
                    "host": "bun",
                    "hook": hook,
                    "transformed": transformed,
                    "stub": false,
                    "status": "ok",
                })
            };
            self.inbound.lock().await.push(SidecarInbound::HookResult {
                request_id: request_id.clone(),
                plugin: plugin.clone(),
                output,
            });
        }
        self.outbound.lock().await.push(message);
        Ok(())
    }

    async fn recv(&self) -> PluginResult<SidecarInbound> {
        let mut inbound = self.inbound.lock().await;
        if inbound.is_empty() {
            return Err(PluginError::Sidecar("no inbound messages".into()));
        }
        Ok(inbound.remove(0))
    }
}

/// Production SidecarPort — spawns Bun and exchanges JSON-line messages over stdio.
#[cfg(feature = "bun-sidecar")]
pub struct BunProcessSidecarPort {
    sidecar_entry: String,
    stdin: Mutex<ChildStdin>,
    inbound_rx: Mutex<tokio::sync::mpsc::UnboundedReceiver<SidecarInbound>>,
    child: Mutex<Child>,
}

#[cfg(feature = "bun-sidecar")]
impl BunProcessSidecarPort {
    /// Spawn `bun run <entry>` with piped stdio and a stdout reader task.
    pub async fn spawn(sidecar_entry: impl Into<String>) -> PluginResult<Arc<Self>> {
        let sidecar_entry = sidecar_entry.into();
        let mut child = Command::new("bun")
            .arg("run")
            .arg(&sidecar_entry)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                let hint = if e.kind() == std::io::ErrorKind::NotFound {
                    " Bun was not found on PATH. Install Bun (>= 1.1), or use a native-only build without Bun/TS plugins."
                } else {
                    ""
                };
                PluginError::Sidecar(format!(
                    "failed to spawn bun for sidecar entry '{sidecar_entry}': {e}.{hint}"
                ))
            })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| PluginError::Sidecar("bun stdin unavailable".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| PluginError::Sidecar("bun stdout unavailable".into()))?;
        let stderr = child.stderr.take();

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match serde_json::from_str::<SidecarInbound>(trimmed) {
                    Ok(msg) => {
                        if tx.send(msg).is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(SidecarInbound::Error {
                            message: format!("invalid JSON-line from sidecar: {err}"),
                        });
                    }
                }
            }
        });

        if let Some(stderr) = stderr {
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::debug!(target: "jerekode_sidecar", "{line}");
                }
            });
        }

        Ok(Arc::new(Self {
            sidecar_entry,
            stdin: Mutex::new(stdin),
            inbound_rx: Mutex::new(rx),
            child: Mutex::new(child),
        }))
    }

    /// Drain the startup `ready` emitted when the sidecar process boots.
    pub async fn wait_startup_ready(self: &Arc<Self>) -> PluginResult<()> {
        loop {
            match self.recv().await? {
                SidecarInbound::Ready => return Ok(()),
                SidecarInbound::Log { level, message } => {
                    tracing::debug!(%level, %message, "sidecar startup log");
                }
                SidecarInbound::Error { message } => {
                    return Err(PluginError::Sidecar(message));
                }
                _ => {}
            }
        }
    }

    pub fn entry(&self) -> &str {
        &self.sidecar_entry
    }

    async fn wait_child_exit(&self) -> PluginResult<()> {
        let mut child = self.child.lock().await;
        match tokio::time::timeout(Duration::from_secs(5), child.wait()).await {
            Ok(Ok(status)) => {
                if status.success() || status.code() == Some(0) {
                    Ok(())
                } else {
                    Err(PluginError::Sidecar(format!(
                        "sidecar exited with status {status}"
                    )))
                }
            }
            Ok(Err(err)) => Err(PluginError::Sidecar(format!("wait failed: {err}"))),
            Err(_) => {
                let _ = child.kill().await;
                Err(PluginError::Sidecar(
                    "sidecar did not exit after shutdown within timeout".into(),
                ))
            }
        }
    }
}

#[async_trait::async_trait]
#[cfg(feature = "bun-sidecar")]
impl SidecarPort for BunProcessSidecarPort {
    async fn send(&self, message: SidecarOutbound) -> PluginResult<()> {
        let is_shutdown = matches!(message, SidecarOutbound::Shutdown);
        let mut line = serde_json::to_string(&message)
            .map_err(|e| PluginError::Sidecar(format!("serialize outbound: {e}")))?;
        line.push('\n');

        {
            let mut stdin = self.stdin.lock().await;
            stdin
                .write_all(line.as_bytes())
                .await
                .map_err(|e| PluginError::Sidecar(format!("write stdin: {e}")))?;
            stdin
                .flush()
                .await
                .map_err(|e| PluginError::Sidecar(format!("flush stdin: {e}")))?;
        }

        if is_shutdown {
            self.wait_child_exit().await?;
        }
        Ok(())
    }

    async fn recv(&self) -> PluginResult<SidecarInbound> {
        let mut rx = self.inbound_rx.lock().await;
        rx.recv()
            .await
            .ok_or_else(|| PluginError::Sidecar("sidecar stdout closed".into()))
    }
}

pub async fn run_sidecar_loop(port: &dyn SidecarPort) -> PluginResult<()> {
    loop {
        match port.recv().await? {
            SidecarInbound::Ready => break,
            SidecarInbound::Error { message } => {
                return Err(PluginError::Sidecar(message));
            }
            SidecarInbound::Log { level, message } => {
                tracing::debug!(%level, %message, "sidecar log");
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "bun-sidecar")]
    use std::path::PathBuf;
    #[cfg(feature = "bun-sidecar")]
    use std::process::Command as StdCommand;

    #[tokio::test]
    async fn in_memory_sidecar_records_outbound() {
        let port = InMemorySidecarPort::new();
        port.send(SidecarOutbound::Init {
            config: serde_json::json!({}),
            plugins: vec!["@acme/plugin".into()],
        })
        .await
        .unwrap();
        assert_eq!(port.recorded_outbound().await.len(), 1);
    }

    #[cfg(feature = "bun-sidecar")]
    fn bun_available() -> bool {
        StdCommand::new("bun")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(feature = "bun-sidecar")]
    fn sidecar_entry() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sidecar/src/index.ts")
    }

    #[cfg(feature = "bun-sidecar")]
    fn require_or_skip(reason: &str) {
        // GitHub Actions sets CI=true. Fail hard there; allow local soft-skip.
        if std::env::var_os("CI").is_some() {
            panic!("{reason}");
        }
        eprintln!("skipping: {reason}");
    }

        #[cfg(feature = "bun-sidecar")]
#[tokio::test]
    async fn bun_process_init_ready_and_shutdown() {
        if !bun_available() {
            require_or_skip("bun_process_init_ready_and_shutdown requires bun on PATH");
            return;
        }

        let entry = sidecar_entry();
        assert!(
            entry.exists(),
            "sidecar entry missing at {}",
            entry.display()
        );

        let port = BunProcessSidecarPort::spawn(entry.to_string_lossy().into_owned())
            .await
            .expect("spawn bun sidecar");

        // Drain startup logs until ready.
        run_sidecar_loop(port.as_ref())
            .await
            .expect("wait for initial ready");

        port.send(SidecarOutbound::Init {
            config: serde_json::json!({"test": true}),
            plugins: vec!["@acme/plugin".into()],
        })
        .await
        .expect("send init");

        // Init emits another ready (after optional log).
        run_sidecar_loop(port.as_ref())
            .await
            .expect("wait for init ready");

        port.send(SidecarOutbound::Shutdown)
            .await
            .expect("shutdown");
    }

        #[cfg(feature = "bun-sidecar")]
#[tokio::test]
    async fn bun_process_loads_fixture_plugin_and_invokes_hook() {
        if !bun_available() {
            require_or_skip("bun_process_loads_fixture_plugin_and_invokes_hook requires bun");
            return;
        }

        let entry = sidecar_entry();
        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sidecar/fixtures/echo-plugin.ts");
        assert!(fixture.exists(), "fixture plugin missing");

        let port = BunProcessSidecarPort::spawn(entry.to_string_lossy().into_owned())
            .await
            .expect("spawn bun sidecar");

        run_sidecar_loop(port.as_ref())
            .await
            .expect("wait for initial ready");

        let plugin_path = fixture
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        port.send(SidecarOutbound::Init {
            config: serde_json::json!({}),
            plugins: vec![plugin_path.clone()],
        })
        .await
        .expect("send init");

        run_sidecar_loop(port.as_ref())
            .await
            .expect("wait for init ready");

        port.send(SidecarOutbound::InvokeHook {
            request_id: "42".into(),
            plugin: plugin_path,
            hook: "before_transform".into(),
            payload: serde_json::json!({"input": "parity"}),
        })
        .await
        .expect("invoke hook");

        let mut got = None;
        for _ in 0..20 {
            match port.recv().await.expect("recv") {
                SidecarInbound::HookResult {
                    request_id, output, ..
                } if request_id == "42" => {
                    got = Some(output);
                    break;
                }
                SidecarInbound::Log { .. } | SidecarInbound::Ready => {}
                other => {
                    tracing::debug!("skip {:?}", other);
                }
            }
        }
        let output = got.expect("hook_result");
        assert_eq!(output["stub"], false);
        assert_eq!(output["transformed"], "parity");
        assert_eq!(output["plugin"], "fixture-echo");

        port.send(SidecarOutbound::Shutdown)
            .await
            .expect("shutdown");
    }
}
