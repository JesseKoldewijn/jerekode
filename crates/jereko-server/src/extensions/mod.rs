//! Extension seams — MCP, LSP, PTY (minimal working stubs).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpStatus {
    pub status: String,
    pub message: String,
    pub tools: Vec<String>,
}

/// Minimal MCP client seam that can list configured tool names.
#[derive(Debug, Default)]
pub struct McpClient {
    tools: Mutex<Vec<String>>,
}

impl McpClient {
    pub fn new() -> Self {
        Self {
            tools: Mutex::new(vec![
                "mcp_filesystem_read".into(),
                "mcp_filesystem_list".into(),
            ]),
        }
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.tools.lock().expect("mcp lock").clone()
    }

    pub fn status(&self) -> McpStatus {
        McpStatus {
            status: "ready".into(),
            message: "MCP client seam active (list_tools only)".into(),
            tools: self.list_tools(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspStatus {
    pub status: String,
    pub initialized: bool,
    pub server_capabilities: serde_json::Value,
}

/// Minimal LSP initialize handshake stub.
#[derive(Debug, Default)]
pub struct LspClient {
    initialized: Mutex<bool>,
}

impl LspClient {
    pub fn new() -> Self {
        Self {
            initialized: Mutex::new(false),
        }
    }

    pub fn initialize(&self, _root_uri: &str) -> LspStatus {
        *self.initialized.lock().expect("lsp lock") = true;
        LspStatus {
            status: "ready".into(),
            initialized: true,
            server_capabilities: serde_json::json!({
                "textDocumentSync": 1,
                "hoverProvider": true
            }),
        }
    }

    pub fn status(&self) -> LspStatus {
        let initialized = *self.initialized.lock().expect("lsp lock");
        LspStatus {
            status: if initialized {
                "ready".into()
            } else {
                "idle".into()
            },
            initialized,
            server_capabilities: if initialized {
                serde_json::json!({"textDocumentSync": 1, "hoverProvider": true})
            } else {
                serde_json::json!({})
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyStatus {
    pub status: String,
    pub sessions: Vec<String>,
}

/// Minimal PTY session registry (spawn stub; portable-pty optional later).
#[derive(Debug, Default)]
pub struct PtyManager {
    sessions: Mutex<HashMap<String, String>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Register a logical PTY session id. Real PTY bytes require the `pty` feature path later.
    pub fn spawn_stub(&self, session_id: impl Into<String>, command: impl Into<String>) -> String {
        let id = session_id.into();
        let cmd = command.into();
        self.sessions
            .lock()
            .expect("pty lock")
            .insert(id.clone(), cmd);
        id
    }

    pub fn status(&self) -> PtyStatus {
        let sessions: Vec<String> = self
            .sessions
            .lock()
            .expect("pty lock")
            .keys()
            .cloned()
            .collect();
        PtyStatus {
            status: if sessions.is_empty() {
                "idle".into()
            } else {
                "ready".into()
            },
            sessions,
        }
    }
}

// Backward-compatible aliases used by router.
pub type McpStubResponse = McpStatus;
pub type LspStubResponse = LspStatus;
pub type PtyStubResponse = PtyStatus;

pub fn mcp_status_stub() -> McpStatus {
    McpClient::new().status()
}

pub fn lsp_status_stub() -> LspStatus {
    let client = LspClient::new();
    client.initialize("file:///tmp/jereko");
    client.status()
}

pub fn pty_status_stub() -> PtyStatus {
    let mgr = PtyManager::new();
    let _ = mgr.spawn_stub("default", "bash");
    mgr.status()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_lists_tools() {
        let client = McpClient::new();
        assert!(!client.list_tools().is_empty());
        assert_eq!(client.status().status, "ready");
    }

    #[test]
    fn lsp_initialize_handshake() {
        let client = LspClient::new();
        let status = client.initialize("file:///project");
        assert!(status.initialized);
        assert!(client.status().initialized);
    }

    #[test]
    fn pty_spawn_stub_registers_session() {
        let mgr = PtyManager::new();
        mgr.spawn_stub("s1", "bash");
        assert!(mgr.status().sessions.contains(&"s1".into()));
    }
}
