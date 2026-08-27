//! Sandbox / tool policy applied before tool execution.

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPolicy {
    pub allow_bash: bool,
    /// Relative path prefixes that are denied even inside the project root.
    pub deny_path_prefixes: Vec<String>,
    /// Max bash wall-clock time.
    pub bash_timeout_secs: u64,
}

impl Default for ToolPolicy {
    fn default() -> Self {
        Self {
            allow_bash: true,
            deny_path_prefixes: vec![".git/".into()],
            bash_timeout_secs: 30,
        }
    }
}

impl ToolPolicy {
    pub fn bash_timeout(&self) -> Duration {
        Duration::from_secs(self.bash_timeout_secs.max(1))
    }

    pub fn path_denied(&self, rel: &str) -> bool {
        let normalized = rel.replace('\\', "/");
        self.deny_path_prefixes.iter().any(|p| {
            let pref = p.replace('\\', "/");
            normalized == pref.trim_end_matches('/') || normalized.starts_with(&pref)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denies_git_prefix_by_default() {
        let p = ToolPolicy::default();
        assert!(p.path_denied(".git/config"));
        assert!(!p.path_denied("src/main.rs"));
    }
}
