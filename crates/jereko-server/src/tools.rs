//! Core agent tools: read, write, edit, bash, grep.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolName {
    Read,
    Write,
    Edit,
    Bash,
    Grep,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: ToolName,
    pub args: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub ok: bool,
    pub output: String,
}

/// Filesystem/bash tool executor rooted at a project directory.
///
/// Bash is intentionally constrained: commands run with the project as cwd.
/// Callers should treat this as a privileged seam and apply policy upstream.
pub struct ToolExecutor {
    root: PathBuf,
    allow_bash: bool,
}

impl ToolExecutor {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            allow_bash: true,
        }
    }

    pub fn with_bash(mut self, allow: bool) -> Self {
        self.allow_bash = allow;
        self
    }

    pub fn execute(&self, call: &ToolCall) -> ToolResult {
        match call.name {
            ToolName::Read => self.read(call),
            ToolName::Write => self.write(call),
            ToolName::Edit => self.edit(call),
            ToolName::Bash => self.bash(call),
            ToolName::Grep => self.grep(call),
        }
    }

    fn resolve(&self, rel: &str) -> Result<PathBuf, String> {
        let candidate = self.root.join(rel);
        let canonical_root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let canonical = if candidate.exists() {
            candidate.canonicalize().map_err(|e| e.to_string())?
        } else if let Some(parent) = candidate.parent() {
            let parent = if parent.exists() {
                parent.canonicalize().map_err(|e| e.to_string())?
            } else {
                return Err(format!("path escapes project root: {rel}"));
            };
            parent.join(candidate.file_name().unwrap_or_default())
        } else {
            return Err(format!("invalid path: {rel}"));
        };
        if !canonical.starts_with(&canonical_root) {
            return Err(format!("path escapes project root: {rel}"));
        }
        Ok(canonical)
    }

    fn read(&self, call: &ToolCall) -> ToolResult {
        let path = match call.args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => {
                return ToolResult {
                    ok: false,
                    output: "missing path".into(),
                }
            }
        };
        match self
            .resolve(path)
            .and_then(|p| std::fs::read_to_string(p).map_err(|e| e.to_string()))
        {
            Ok(content) => ToolResult {
                ok: true,
                output: content,
            },
            Err(e) => ToolResult {
                ok: false,
                output: e,
            },
        }
    }

    fn write(&self, call: &ToolCall) -> ToolResult {
        let path = match call.args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => {
                return ToolResult {
                    ok: false,
                    output: "missing path".into(),
                }
            }
        };
        let content = call
            .args
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        match self.resolve(path).and_then(|p| {
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(p, content).map_err(|e| e.to_string())
        }) {
            Ok(()) => ToolResult {
                ok: true,
                output: format!("wrote {path}"),
            },
            Err(e) => ToolResult {
                ok: false,
                output: e,
            },
        }
    }

    fn edit(&self, call: &ToolCall) -> ToolResult {
        let path = match call.args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => {
                return ToolResult {
                    ok: false,
                    output: "missing path".into(),
                }
            }
        };
        let old = call
            .args
            .get("old_string")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let new = call
            .args
            .get("new_string")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        match self.resolve(path).and_then(|p| {
            let content = std::fs::read_to_string(&p).map_err(|e| e.to_string())?;
            if !content.contains(old) {
                return Err("old_string not found".into());
            }
            let updated = content.replacen(old, new, 1);
            std::fs::write(p, updated).map_err(|e| e.to_string())
        }) {
            Ok(()) => ToolResult {
                ok: true,
                output: format!("edited {path}"),
            },
            Err(e) => ToolResult {
                ok: false,
                output: e,
            },
        }
    }

    fn bash(&self, call: &ToolCall) -> ToolResult {
        if !self.allow_bash {
            return ToolResult {
                ok: false,
                output: "bash disabled by policy".into(),
            };
        }
        let command = match call.args.get("command").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => {
                return ToolResult {
                    ok: false,
                    output: "missing command".into(),
                }
            }
        };
        // Safety note: bash runs in the project root with no network sandbox.
        // Prefer allowlists / deny-by-default in production policy layers.
        let output = if cfg!(windows) {
            Command::new("cmd")
                .args(["/C", command])
                .current_dir(&self.root)
                .output()
        } else {
            Command::new("sh")
                .args(["-c", command])
                .current_dir(&self.root)
                .output()
        };
        match output {
            Ok(out) => {
                let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
                let err = String::from_utf8_lossy(&out.stderr);
                if !err.is_empty() {
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(&err);
                }
                ToolResult {
                    ok: out.status.success(),
                    output: text,
                }
            }
            Err(e) => ToolResult {
                ok: false,
                output: e.to_string(),
            },
        }
    }

    fn grep(&self, call: &ToolCall) -> ToolResult {
        let pattern = match call.args.get("pattern").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => {
                return ToolResult {
                    ok: false,
                    output: "missing pattern".into(),
                }
            }
        };
        let path = call
            .args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");
        let target = match self.resolve(path) {
            Ok(p) => p,
            Err(e) => {
                return ToolResult {
                    ok: false,
                    output: e,
                }
            }
        };
        let mut matches = Vec::new();
        if let Err(e) = walk_grep(&target, pattern, &self.root, &mut matches) {
            return ToolResult {
                ok: false,
                output: e,
            };
        }
        ToolResult {
            ok: true,
            output: matches.join("\n"),
        }
    }
}

fn walk_grep(path: &Path, pattern: &str, root: &Path, out: &mut Vec<String>) -> Result<(), String> {
    if path.is_file() {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        for (idx, line) in content.lines().enumerate() {
            if line.contains(pattern) {
                let rel = path.strip_prefix(root).unwrap_or(path);
                out.push(format!("{}:{}:{}", rel.display(), idx + 1, line));
            }
        }
        return Ok(());
    }
    for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let p = entry.path();
        if p.file_name().and_then(|s| s.to_str()) == Some("target") {
            continue;
        }
        walk_grep(&p, pattern, root, out)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn read_write_grep_round_trip() {
        let dir = TempDir::new().unwrap();
        let tools = ToolExecutor::new(dir.path()).with_bash(false);
        let write = tools.execute(&ToolCall {
            name: ToolName::Write,
            args: serde_json::json!({"path": "a.txt", "content": "hello world"}),
        });
        assert!(write.ok);
        let read = tools.execute(&ToolCall {
            name: ToolName::Read,
            args: serde_json::json!({"path": "a.txt"}),
        });
        assert_eq!(read.output, "hello world");
        let grep = tools.execute(&ToolCall {
            name: ToolName::Grep,
            args: serde_json::json!({"pattern": "hello", "path": "."}),
        });
        assert!(grep.ok);
        assert!(grep.output.contains("a.txt"));
    }

    #[test]
    fn edit_replaces_once() {
        let dir = TempDir::new().unwrap();
        let tools = ToolExecutor::new(dir.path());
        tools.execute(&ToolCall {
            name: ToolName::Write,
            args: serde_json::json!({"path": "b.txt", "content": "aa bb aa"}),
        });
        let edit = tools.execute(&ToolCall {
            name: ToolName::Edit,
            args: serde_json::json!({
                "path": "b.txt",
                "old_string": "aa",
                "new_string": "xx"
            }),
        });
        assert!(edit.ok);
        let read = tools.execute(&ToolCall {
            name: ToolName::Read,
            args: serde_json::json!({"path": "b.txt"}),
        });
        assert_eq!(read.output, "xx bb aa");
    }
}
