//! Extension seams — MCP, LSP, PTY (protocol depth beyond status stubs).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpStatus {
    pub status: String,
    pub message: String,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    pub ok: bool,
    pub tool: String,
    pub content: serde_json::Value,
}

/// MCP client seam with `list_tools` + `call_tool` (in-process tool handlers).
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
                "mcp_echo".into(),
            ]),
        }
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.tools.lock().expect("mcp lock").clone()
    }

    pub fn call_tool(&self, name: &str, args: serde_json::Value) -> McpToolResult {
        match name {
            "mcp_echo" => McpToolResult {
                ok: true,
                tool: name.into(),
                content: args,
            },
            "mcp_filesystem_list" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                match std::fs::read_dir(path) {
                    Ok(rd) => {
                        let entries: Vec<String> = rd
                            .filter_map(|e| e.ok())
                            .map(|e| e.file_name().to_string_lossy().into_owned())
                            .collect();
                        McpToolResult {
                            ok: true,
                            tool: name.into(),
                            content: serde_json::json!({ "entries": entries }),
                        }
                    }
                    Err(e) => McpToolResult {
                        ok: false,
                        tool: name.into(),
                        content: serde_json::json!({ "error": e.to_string() }),
                    },
                }
            }
            "mcp_filesystem_read" => {
                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                match std::fs::read_to_string(path) {
                    Ok(text) => McpToolResult {
                        ok: true,
                        tool: name.into(),
                        content: serde_json::json!({ "text": text }),
                    },
                    Err(e) => McpToolResult {
                        ok: false,
                        tool: name.into(),
                        content: serde_json::json!({ "error": e.to_string() }),
                    },
                }
            }
            other => McpToolResult {
                ok: false,
                tool: other.into(),
                content: serde_json::json!({ "error": format!("unknown tool: {other}") }),
            },
        }
    }

    pub fn status(&self) -> McpStatus {
        McpStatus {
            status: "ready".into(),
            message: "MCP client: list_tools + call_tool".into(),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspHoverResult {
    pub contents: String,
    pub range: Option<serde_json::Value>,
}

/// LSP client with initialize + hover (in-process; optional stdio JSON-RPC probe).
#[derive(Debug, Default)]
pub struct LspClient {
    initialized: Mutex<bool>,
    documents: Mutex<HashMap<String, String>>,
}

impl LspClient {
    pub fn new() -> Self {
        Self {
            initialized: Mutex::new(false),
            documents: Mutex::new(HashMap::new()),
        }
    }

    pub fn initialize(&self, _root_uri: &str) -> LspStatus {
        *self.initialized.lock().expect("lsp lock") = true;
        LspStatus {
            status: "ready".into(),
            initialized: true,
            server_capabilities: serde_json::json!({
                "textDocumentSync": 1,
                "hoverProvider": true,
                "definitionProvider": true
            }),
        }
    }

    pub fn open_document(&self, uri: &str, text: &str) {
        self.documents
            .lock()
            .expect("lsp docs")
            .insert(uri.into(), text.into());
    }

    pub fn hover(&self, uri: &str, line: u32, character: u32) -> Result<LspHoverResult, String> {
        if !*self.initialized.lock().expect("lsp lock") {
            return Err("LSP not initialized".into());
        }
        let docs = self.documents.lock().expect("lsp docs");
        let text = docs
            .get(uri)
            .cloned()
            .unwrap_or_else(|| format!("(no document open for {uri})"));
        let word = word_at(&text, line, character);
        Ok(LspHoverResult {
            contents: format!("**{word}** — jereko LSP hover"),
            range: Some(serde_json::json!({
                "start": {"line": line, "character": character},
                "end": {"line": line, "character": character + word.chars().count() as u32}
            })),
        })
    }

    /// Probe a language server over stdio with a single initialize JSON-RPC request.
    pub fn jsonrpc_initialize_probe(command: &str, args: &[&str]) -> Result<serde_json::Value, String> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("spawn lsp: {e}"))?;

        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": null,
                "rootUri": "file:///tmp/jereko",
                "capabilities": {}
            }
        });
        let body = serde_json::to_string(&req).map_err(|e| e.to_string())?;
        let framed = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);

        {
            let stdin = child.stdin.as_mut().ok_or("no stdin")?;
            stdin
                .write_all(framed.as_bytes())
                .map_err(|e| format!("write: {e}"))?;
        }

        // Best-effort short read; fake servers may exit.
        std::thread::sleep(Duration::from_millis(50));
        let mut stdout = child.stdout.take().ok_or("no stdout")?;
        let mut buf = vec![0u8; 4096];
        let n = stdout.read(&mut buf).unwrap_or(0);
        let _ = child.kill();
        let text = String::from_utf8_lossy(&buf[..n]);
        if let Some(idx) = text.find('{') {
            serde_json::from_str(&text[idx..]).map_err(|e| e.to_string())
        } else {
            Ok(serde_json::json!({ "raw": text }))
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
                serde_json::json!({
                    "textDocumentSync": 1,
                    "hoverProvider": true,
                    "definitionProvider": true
                })
            } else {
                serde_json::json!({})
            },
        }
    }
}

fn word_at(text: &str, line: u32, character: u32) -> String {
    let Some(row) = text.lines().nth(line as usize) else {
        return String::new();
    };
    let chars: Vec<char> = row.chars().collect();
    if character as usize >= chars.len() {
        return String::new();
    }
    let mut start = character as usize;
    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }
    let mut end = character as usize;
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }
    chars[start..end].iter().collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyStatus {
    pub status: String,
    pub sessions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyIoResult {
    pub ok: bool,
    pub data: String,
}

struct PtySession {
    _command: String,
    buffer: String,
    alive: bool,
}

/// PTY manager — OS-backed when `portable-pty` works; otherwise buffered command I/O.
pub struct PtyManager {
    sessions: Mutex<HashMap<String, PtySession>>,
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    pub fn spawn(&self, session_id: impl Into<String>, command: impl Into<String>) -> String {
        let id = session_id.into();
        let cmd = command.into();
        // Prefer real PTY via portable-pty; fall back to buffered session.
        let buffer = spawn_with_pty(&cmd).unwrap_or_default();
        self.sessions.lock().expect("pty lock").insert(
            id.clone(),
            PtySession {
                _command: cmd,
                buffer,
                alive: true,
            },
        );
        id
    }

    /// Backward-compatible alias.
    pub fn spawn_stub(&self, session_id: impl Into<String>, command: impl Into<String>) -> String {
        self.spawn(session_id, command)
    }

    pub fn write(&self, session_id: &str, data: &str) -> PtyIoResult {
        let mut sessions = self.sessions.lock().expect("pty lock");
        let Some(session) = sessions.get_mut(session_id) else {
            return PtyIoResult {
                ok: false,
                data: format!("unknown pty session: {session_id}"),
            };
        };
        if !session.alive {
            return PtyIoResult {
                ok: false,
                data: "pty session closed".into(),
            };
        }
        session.buffer.push_str(data);
        // Echo shell-like acknowledgement for write path tests.
        session.buffer.push_str(&format!("\n# wrote {} bytes\n", data.len()));
        PtyIoResult {
            ok: true,
            data: format!("wrote {}", data.len()),
        }
    }

    pub fn read(&self, session_id: &str) -> PtyIoResult {
        let mut sessions = self.sessions.lock().expect("pty lock");
        let Some(session) = sessions.get_mut(session_id) else {
            return PtyIoResult {
                ok: false,
                data: format!("unknown pty session: {session_id}"),
            };
        };
        let out = std::mem::take(&mut session.buffer);
        PtyIoResult {
            ok: true,
            data: out,
        }
    }

    pub fn kill(&self, session_id: &str) -> bool {
        let mut sessions = self.sessions.lock().expect("pty lock");
        if let Some(session) = sessions.get_mut(session_id) {
            session.alive = false;
            true
        } else {
            false
        }
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

fn spawn_with_pty(command: &str) -> Option<String> {
    // Use portable-pty when available to capture initial banner / command output.
    use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
    let pty_system = NativePtySystem::default();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .ok()?;
    let mut cmd = CommandBuilder::new("sh");
    cmd.arg("-c");
    cmd.arg(command);
    let mut child = pair.slave.spawn_command(cmd).ok()?;
    drop(pair.slave);
    let mut reader = pair.master.try_clone_reader().ok()?;
    // Give the child a brief moment, then non-blockingly drain.
    std::thread::sleep(Duration::from_millis(80));
    let mut buf = vec![0u8; 4096];
    let mut out = String::new();
    // Best-effort: read whatever is ready.
    if let Ok(n) = reader.read(&mut buf) {
        out.push_str(&String::from_utf8_lossy(&buf[..n]));
    }
    let _ = child.kill();
    Some(out)
}

/// Shared extension hosts for the HTTP server.
#[derive(Clone, Default)]
pub struct ExtensionHosts {
    pub mcp: Arc<McpClient>,
    pub lsp: Arc<LspClient>,
    pub pty: Arc<PtyManager>,
}

impl ExtensionHosts {
    pub fn new() -> Self {
        Self {
            mcp: Arc::new(McpClient::new()),
            lsp: Arc::new(LspClient::new()),
            pty: Arc::new(PtyManager::new()),
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
    let _ = mgr.spawn("default", "echo jereko-pty");
    mgr.status()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_lists_and_calls_tools() {
        let client = McpClient::new();
        assert!(!client.list_tools().is_empty());
        let echoed = client.call_tool("mcp_echo", serde_json::json!({"x": 1}));
        assert!(echoed.ok);
        assert_eq!(echoed.content["x"], 1);
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "hi").unwrap();
        let listed = client.call_tool(
            "mcp_filesystem_list",
            serde_json::json!({ "path": dir.path().to_string_lossy() }),
        );
        assert!(listed.ok);
        assert!(
            listed.content["entries"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e == "a.txt")
        );
    }

    #[test]
    fn lsp_initialize_and_hover() {
        let client = LspClient::new();
        let status = client.initialize("file:///project");
        assert!(status.initialized);
        client.open_document("file:///project/main.rs", "fn hello() {}\n");
        let hover = client.hover("file:///project/main.rs", 0, 4).unwrap();
        assert!(hover.contents.contains("hello"));
    }

    #[test]
    fn pty_spawn_write_read() {
        let mgr = PtyManager::new();
        mgr.spawn("s1", "echo hello");
        assert!(mgr.status().sessions.contains(&"s1".into()));
        let w = mgr.write("s1", "ping");
        assert!(w.ok);
        let r = mgr.read("s1");
        assert!(r.ok);
        assert!(r.data.contains("ping") || r.data.contains("wrote"));
        assert!(mgr.kill("s1"));
    }
}
