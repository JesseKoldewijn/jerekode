use crate::error::{PluginError, PluginResult};
use serde::{Deserialize, Serialize};

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
    outbound: tokio::sync::Mutex<Vec<SidecarOutbound>>,
    inbound: tokio::sync::Mutex<Vec<SidecarInbound>>,
}

impl InMemorySidecarPort {
    pub fn new() -> Self {
        Self {
            outbound: tokio::sync::Mutex::new(Vec::new()),
            inbound: tokio::sync::Mutex::new(vec![SidecarInbound::Ready]),
        }
    }

    pub async fn recorded_outbound(&self) -> Vec<SidecarOutbound> {
        self.outbound.lock().await.clone()
    }
}

#[async_trait::async_trait]
impl SidecarPort for InMemorySidecarPort {
    async fn send(&self, message: SidecarOutbound) -> PluginResult<()> {
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

/// Production SidecarPort — spawns Bun process with JSON-line stdio (stub transport).
pub struct BunProcessSidecarPort {
    sidecar_entry: String,
}

impl BunProcessSidecarPort {
    pub fn new(sidecar_entry: impl Into<String>) -> Self {
        Self {
            sidecar_entry: sidecar_entry.into(),
        }
    }
}

#[async_trait::async_trait]
impl SidecarPort for BunProcessSidecarPort {
    async fn send(&self, message: SidecarOutbound) -> PluginResult<()> {
        tracing::debug!(entry = %self.sidecar_entry, ?message, "sidecar send (stub transport)");
        Ok(())
    }

    async fn recv(&self) -> PluginResult<SidecarInbound> {
        Ok(SidecarInbound::Ready)
    }
}

pub async fn run_sidecar_loop(port: &dyn SidecarPort) -> PluginResult<()> {
    loop {
        match port.recv().await? {
            SidecarInbound::Ready => break,
            SidecarInbound::Error { message } => {
                return Err(PluginError::Sidecar(message));
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
