//! WasmPluginHost — wasmtime load + `jerekode_hook` export ABI (WASI-ready linker seam).

use crate::error::{PluginError, PluginResult};
use crate::host::PluginHost;
use crate::types::{HookCall, HookResult, HostId, LoadedPlugin, PluginSpec};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use wasmtime::{Engine, Instance, Linker, Module, Store, Val};

/// WasmPluginHost — loads wasm modules and invokes the `jerekode_hook` export when present.
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

    fn invoke_export(bytes: &[u8], hook: &HookCall) -> PluginResult<serde_json::Value> {
        let engine = Engine::default();
        let module = Module::new(&engine, bytes).map_err(|e| PluginError::Host {
            host: "wasm".into(),
            message: format!("invalid wasm: {e}"),
        })?;

        // WASI-capable linker seam: modules may import WASI later; empty linker works for
        // pure compute fixtures that only export `memory` + `jerekode_hook`.
        let linker: Linker<()> = Linker::new(&engine);
        let mut store = Store::new(&engine, ());
        let instance: Instance =
            linker
                .instantiate(&mut store, &module)
                .map_err(|e| PluginError::Host {
                    host: "wasm".into(),
                    message: format!("instantiate: {e}"),
                })?;

        let Some(func) = instance.get_func(&mut store, "jerekode_hook") else {
            return Ok(serde_json::json!({
                "host": "wasm",
                "hook": hook.hook,
                "loaded": true,
                "stub": false,
                "abi": "host-fallback"
            }));
        };

        let payload = serde_json::json!({
            "hook": hook.hook,
            "payload": hook.payload,
        });
        let payload_bytes = serde_json::to_vec(&payload).map_err(|e| PluginError::Host {
            host: "wasm".into(),
            message: format!("serialize payload: {e}"),
        })?;

        let memory =
            instance
                .get_memory(&mut store, "memory")
                .ok_or_else(|| PluginError::Host {
                    host: "wasm".into(),
                    message: "wasm module missing exported memory".into(),
                })?;

        let ptr = 1024i32;
        memory
            .write(&mut store, ptr as usize, &payload_bytes)
            .map_err(|e| PluginError::Host {
                host: "wasm".into(),
                message: format!("memory write: {e}"),
            })?;

        let mut results = [Val::I32(0)];
        func.call(
            &mut store,
            &[Val::I32(ptr), Val::I32(payload_bytes.len() as i32)],
            &mut results,
        )
        .map_err(|e| PluginError::Host {
            host: "wasm".into(),
            message: format!("jerekode_hook call: {e}"),
        })?;

        let out_ptr = results[0].unwrap_i32() as usize;
        let data = memory.data(&store);
        let end = data[out_ptr..]
            .iter()
            .position(|&b| b == 0)
            .map(|i| out_ptr + i)
            .unwrap_or(data.len().min(out_ptr + 4096));
        let text = std::str::from_utf8(&data[out_ptr..end]).map_err(|e| PluginError::Host {
            host: "wasm".into(),
            message: format!("utf8: {e}"),
        })?;
        serde_json::from_str(text).or_else(|_| {
            Ok(serde_json::json!({
                "host": "wasm",
                "hook": hook.hook,
                "loaded": true,
                "stub": false,
                "abi": "jerekode_hook",
                "raw": text,
            }))
        })
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
            let engine = Engine::default();
            Module::new(&engine, &bytes).map_err(|e| PluginError::Host {
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
        if let Some(bytes) = loaded.get(&plugin.spec.name) {
            let output = Self::invoke_export(bytes, &hook)?;
            return Ok(HookResult {
                plugin: plugin.spec.name.clone(),
                output,
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

    fn tiny_wasm() -> Vec<u8> {
        vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]
    }

    fn hook_wasm() -> Vec<u8> {
        let wat = r#"
(module
  (memory (export "memory") 1)
  (func (export "jerekode_hook") (param i32 i32) (result i32)
    i32.const 16)
  (data (i32.const 16) "{\"host\":\"wasm\",\"hook\":\"before_transform\",\"stub\":false,\"abi\":\"jerekode_hook\"}\00")
)
"#;
        wat::parse_str(wat).expect("wat parse")
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
        assert_eq!(result.output["abi"], "host-fallback");
    }

    #[tokio::test]
    async fn invokes_jerekode_hook_export() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hook.wasm");
        std::fs::write(&path, hook_wasm()).unwrap();

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
                    payload: serde_json::json!({"input": "hi"}),
                },
            )
            .await
            .unwrap();
        assert_eq!(result.output["stub"], false);
        assert_eq!(result.output["abi"], "jerekode_hook");
        assert_eq!(result.output["host"], "wasm");
    }

    fn raw_hook_wasm() -> Vec<u8> {
        let wat = r#"
(module
  (memory (export "memory") 1)
  (func (export "jerekode_hook") (param i32 i32) (result i32)
    i32.const 16)
  (data (i32.const 16) "not-json-output\00")
)
"#;
        wat::parse_str(wat).expect("raw hook wat")
    }

    #[tokio::test]
    async fn raw_hook_output_uses_jerekode_abi_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("raw.wasm");
        std::fs::write(&path, raw_hook_wasm()).unwrap();

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
        assert_eq!(result.output["abi"], "jerekode_hook");
        assert_eq!(result.output["raw"], "not-json-output");
    }
}
