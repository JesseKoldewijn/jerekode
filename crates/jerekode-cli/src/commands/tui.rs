use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};

/// Resolve owned Bun TUI entry (`packages/tui/src/index.ts`).
fn resolve_tui_entry() -> Option<PathBuf> {
    if let Ok(explicit) = env::var("JEREKODE_TUI_ENTRY") {
        let p = PathBuf::from(explicit);
        if p.exists() {
            return Some(p);
        }
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd);
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    candidates.push(manifest_dir.join("../.."));
    if let Ok(exe) = env::current_exe()
        && let Some(dir) = exe.parent()
    {
        candidates.push(dir.to_path_buf());
        candidates.push(dir.join(".."));
        candidates.push(dir.join("../.."));
    }

    for base in candidates {
        let Ok(base) = base.canonicalize() else {
            continue;
        };
        let mut dir = base;
        for _ in 0..8 {
            let entry = dir.join("packages/tui/src/index.ts");
            if entry.is_file() {
                return Some(entry);
            }
            if !dir.pop() {
                break;
            }
        }
    }
    None
}

fn bun_available() -> bool {
    Command::new("bun")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Bare invoke: start the minimal owned Bun TUI.
pub async fn execute() -> anyhow::Result<ExitCode> {
    #[cfg(not(feature = "bun-sidecar"))]
    {
        eprintln!(
            "error: bare `jerekode` starts the Bun TUI, but this binary was built without `bun-sidecar`"
        );
        eprintln!("hint: install a full (Bun) build, or pass a subcommand (`serve`, `run`, …)");
        return Ok(ExitCode::from(1));
    }

    #[cfg(feature = "bun-sidecar")]
    {
        if !bun_available() {
            eprintln!("error: Bun was not found on PATH (required for the TUI)");
            eprintln!("hint: install Bun (>= 1.1), or use `jerekode run` / `jerekode serve`");
            return Ok(ExitCode::from(1));
        }

        let Some(entry) = resolve_tui_entry() else {
            eprintln!("error: could not find packages/tui/src/index.ts");
            eprintln!("hint: set JEREKODE_TUI_ENTRY to the TUI entry script");
            return Ok(ExitCode::from(1));
        };

        let smoke = env::var("JEREKODE_TUI_SMOKE").is_ok_and(|v| !v.is_empty() && v != "0");
        let mut cmd = Command::new("bun");
        cmd.arg("run").arg(&entry);
        if smoke {
            cmd.stdin(Stdio::null());
        } else {
            cmd.stdin(Stdio::inherit());
        }
        let status = cmd
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| anyhow::anyhow!("failed to spawn Bun TUI: {e}"))?;

        if status.success() {
            return Ok(ExitCode::SUCCESS);
        }
        Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn resolve_finds_workspace_tui_from_manifest() {
        let entry = resolve_tui_entry();
        assert!(
            entry
                .as_ref()
                .is_some_and(|p| p.ends_with("packages/tui/src/index.ts")),
            "expected packages/tui entry, got {entry:?}"
        );
        assert!(entry.unwrap().is_file());
    }

    #[test]
    fn resolve_honors_explicit_env() {
        let tmp = tempfile::tempdir().unwrap();
        let script = tmp.path().join("custom-tui.ts");
        std::fs::write(&script, "// custom").unwrap();
        // SAFETY: test-only env mutation; restored below.
        unsafe {
            env::set_var("JEREKODE_TUI_ENTRY", &script);
        }
        let got = resolve_tui_entry();
        unsafe {
            env::remove_var("JEREKODE_TUI_ENTRY");
        }
        assert_eq!(got.as_deref(), Some(script.as_path()));
    }

    #[test]
    fn path_join_packages_tui() {
        let p = Path::new("/repo").join("packages/tui/src/index.ts");
        assert!(p.ends_with("packages/tui/src/index.ts"));
    }
}
