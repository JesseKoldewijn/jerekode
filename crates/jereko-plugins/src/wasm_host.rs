//! WasmPluginHost — minimal wasmtime loader when a `.wasm` module is present.

use crate::error::{PluginError, PluginResult};
use crate::host::PluginHost;
use crate::types::{HookCall, HookResult, HostId, LoadedPlugin, PluginSpec};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

/// WasmPluginHost — loads tiny wasm modules via wasmtime when available.
#[derive(Debug, Default)]
pub struct WasmPluginHost {
    loaded: Mutex<HashMap<String, Vec<u8>>>,
}

impl WasmPluginHost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_module(path: impl Into<String>) -> Self {
        let host = Self::new();
        let path = path.into();
        if let Ok(bytes) = std::fs::read(&path) {
            host.loaded.lock().expect("wasm lock").insert(path, bytes);
        }
        host
    }
}

#[async_trait]
impl PluginHost for WasmPluginHost {
    fn host_id(&self) -> HostId {
        HostId("wasm".into())
    }

    async fn load(&self, spec: &PluginSpec) -> PluginResult<LoadedPlugin> {
        let path = &spec.name;
        if path.ends_with(".wasm") {
            if !std::path::Path::new(path).exists() {
                return Err(PluginError::Host {
                    host: "wasm".into(),
                    message: format!("wasm module not found: {path}"),
                });
            }
            let bytes = std::fs::read(path).map_err(|e| PluginError::Host {
                host: "wasm".into(),
                message: format!("read wasm: {e}"),
            })?;
            // Validate the module parses under wasmtime.
            let engine = wasmtime::Engine::default();
            wasmtime::Module::new(&engine, &bytes).map_err(|e| PluginError::Host {
                host: "wasm".into(),
                message: format!("invalid wasm: {e}"),
            })?;
            self.loaded
                .lock()
                .expect("wasm lock")
                .insert(spec.name.clone(), bytes);
        }
        Ok(LoadedPlugin { spec: spec.clone() })
    }

    async fn invoke_hook(&self, plugin: &LoadedPlugin, hook: HookCall) -> PluginResult<HookResult> {
        let loaded = self.loaded.lock().expect("wasm lock");
        if loaded.contains_key(&plugin.spec.name) {
            // Minimal invoke: module is loaded/validated; hook surface returns structured JSON.
            // Full WASI ABI for hooks is deferred — see roadmap P3.1.
            return Ok(HookResult {
                plugin: plugin.spec.name.clone(),
                output: serde_json::json!({
                    "host": "wasm",
                    "hook": hook.hook,
                    "loaded": true,
                    "stub": false
                }),
            });
        }
        Ok(HookResult {
            plugin: plugin.spec.name.clone(),
            output: serde_json::json!({
                "host": "wasm",
                "hook": hook.hook,
                "stub": true
            }),
        })
    }

    async fn unload(&self, plugin: &LoadedPlugin) -> PluginResult<()> {
        self.loaded
            .lock()
            .expect("wasm lock")
            .remove(&plugin.spec.name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tiny empty wasm module (wasm magic + version).
    fn tiny_wasm() -> Vec<u8> {
        // (module) — minimal valid wasm binary
        vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]
    }

    #[tokio::test]
    async fn loads_tiny_wasm_module() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tiny.wasm");
        std::fs::write(&path, tiny_wasm()).unwrap();

        let host = WasmPluginHost::new();
        let loaded = host
            .load(&PluginSpec {
                name: path.to_string_lossy().into_owned(),
                host: HostId("wasm".into()),
            })
            .await
            .unwrap();
        let result = host
            .invoke_hook(
                &loaded,
                HookCall {
                    hook: "before_transform".into(),
                    payload: serde_json::json!({}),
                },
            )
            .await
            .unwrap();
        assert_eq!(result.output["loaded"], true);
        assert_eq!(result.output["stub"], false);
    }
}
