//! Extension seam stubs — MCP, LSP, PTY (Phase 4).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpStubResponse {
    pub status: String,
    pub message: String,
}

pub fn mcp_status_stub() -> McpStubResponse {
    McpStubResponse {
        status: "stub".into(),
        message: "MCP integration pending".into(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspStubResponse {
    pub status: String,
}

pub fn lsp_status_stub() -> LspStubResponse {
    LspStubResponse {
        status: "stub".into(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyStubResponse {
    pub status: String,
}

pub fn pty_status_stub() -> PtyStubResponse {
    PtyStubResponse {
        status: "stub".into(),
    }
}
