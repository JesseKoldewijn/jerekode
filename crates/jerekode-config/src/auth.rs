//! Credential store for providers.
//!
//! Jerekode writes only to a jerekode-specific path (`~/.config/jerekode/auth.json`
//! by default). OpenCode's store may be **imported** but is never overwritten.

use crate::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// On-disk auth document (provider id → credential record).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthStore {
    #[serde(default)]
    pub providers: BTreeMap<String, AuthCredential>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthCredential {
    /// API key or token material.
    pub api_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
}

impl AuthStore {
    pub fn list_ids(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub fn upsert(&mut self, provider: impl Into<String>, cred: AuthCredential) {
        self.providers.insert(provider.into(), cred);
    }

    pub fn remove(&mut self, provider: &str) -> bool {
        self.providers.remove(provider).is_some()
    }
}

/// `~/.config/jerekode/auth.json` (override with `JEREKODE_AUTH_PATH`).
pub fn jerekode_auth_path() -> PathBuf {
    if let Ok(p) = std::env::var("JEREKODE_AUTH_PATH")
        && !p.is_empty()
    {
        return PathBuf::from(p);
    }
    config_home().join("jerekode").join("auth.json")
}

/// OpenCode credential locations we may import from (never write).
pub fn opencode_auth_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(p) = std::env::var("OPENCODE_AUTH_PATH")
        && !p.is_empty()
    {
        out.push(PathBuf::from(p));
    }
    let home = home_dir();
    // Docs: ~/.local/share/opencode/auth.json
    out.push(home.join(".local/share/opencode/auth.json"));
    // Also accept config-dir layout if present.
    out.push(config_home().join("opencode").join("auth.json"));
    out
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn config_home() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME")
        && !xdg.is_empty()
    {
        return PathBuf::from(xdg);
    }
    home_dir().join(".config")
}

pub fn load_store(path: &Path) -> ConfigResult<AuthStore> {
    if !path.exists() {
        return Ok(AuthStore::default());
    }
    let raw = fs::read_to_string(path).map_err(|source| ConfigError::ReadFile {
        path: path.display().to_string(),
        source,
    })?;
    // Accept either `{ "providers": { ... } }` or a flat `{ "openai": { "api_key": ... } }`.
    if let Ok(store) = serde_json::from_str::<AuthStore>(&raw)
        && (!store.providers.is_empty() || raw.contains("\"providers\""))
    {
        return Ok(store);
    }
    let flat: BTreeMap<String, AuthCredential> =
        serde_json::from_str(&raw).map_err(|source| ConfigError::Parse {
            path: path.display().to_string(),
            source,
        })?;
    Ok(AuthStore { providers: flat })
}

pub fn save_store(path: &Path, store: &AuthStore) -> ConfigResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| ConfigError::ReadFile {
            path: parent.display().to_string(),
            source,
        })?;
    }
    let raw = serde_json::to_string_pretty(store).map_err(|source| ConfigError::Parse {
        path: path.display().to_string(),
        source,
    })?;
    fs::write(path, format!("{raw}\n")).map_err(|source| ConfigError::ReadFile {
        path: path.display().to_string(),
        source,
    })?;
    Ok(())
}

/// Import OpenCode credentials into the jerekode store (merge; jerekode wins on conflict).
pub fn import_opencode_into(jerekode_path: &Path) -> ConfigResult<(AuthStore, Option<PathBuf>)> {
    let mut store = load_store(jerekode_path)?;
    let mut source = None;
    for cand in opencode_auth_candidates() {
        if !cand.exists() {
            continue;
        }
        let imported = load_store(&cand)?;
        for (id, cred) in imported.providers {
            store.providers.entry(id).or_insert(cred);
        }
        source = Some(cand);
        break;
    }
    save_store(jerekode_path, &store)?;
    Ok((store, source))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn round_trip_store() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        let mut store = AuthStore::default();
        store.upsert(
            "openai",
            AuthCredential {
                api_key: "sk-test".into(),
                method: Some("api".into()),
            },
        );
        save_store(&path, &store).unwrap();
        let loaded = load_store(&path).unwrap();
        assert_eq!(loaded.list_ids(), vec!["openai".to_string()]);
        assert_eq!(loaded.providers["openai"].api_key, "sk-test");
    }

    #[test]
    fn import_merges_without_clobbering_jerekode() {
        let _g = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let oc = dir.path().join("opencode-auth.json");
        let jk = dir.path().join("jerekode-auth.json");
        fs::write(
            &oc,
            r#"{"anthropic":{"api_key":"from-oc"},"openai":{"api_key":"oc-openai"}}"#,
        )
        .unwrap();
        let mut existing = AuthStore::default();
        existing.upsert(
            "openai",
            AuthCredential {
                api_key: "keep-me".into(),
                method: None,
            },
        );
        save_store(&jk, &existing).unwrap();
        // SAFETY: test-only env
        unsafe {
            std::env::set_var("OPENCODE_AUTH_PATH", &oc);
            std::env::set_var("JEREKODE_AUTH_PATH", &jk);
        }
        let (merged, src) = import_opencode_into(&jk).unwrap();
        unsafe {
            std::env::remove_var("OPENCODE_AUTH_PATH");
            std::env::remove_var("JEREKODE_AUTH_PATH");
        }
        assert_eq!(src, Some(oc));
        assert_eq!(merged.providers["openai"].api_key, "keep-me");
        assert_eq!(merged.providers["anthropic"].api_key, "from-oc");
    }

    #[test]
    fn jerekode_path_uses_override() {
        let _g = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("JEREKODE_AUTH_PATH", "/tmp/custom-auth.json");
        }
        let p = jerekode_auth_path();
        unsafe {
            std::env::remove_var("JEREKODE_AUTH_PATH");
        }
        assert_eq!(p, PathBuf::from("/tmp/custom-auth.json"));
    }
}
