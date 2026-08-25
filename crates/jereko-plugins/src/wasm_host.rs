use crate::error::PluginResult;
use crate::host::PluginHost;
use crate::types::{HookCall, HookResult, HostId, LoadedPlugin, PluginSpec};
use async_trait::async_trait;

/// WasmPluginHost stub (Phase 4 — sandboxed untrusted plugins).
pub struct WasmPluginHost;

#[async_trait]
impl PluginHost for WasmPluginHost {
    fn host_id(&self) -> HostId {
        HostId("wasm".into())
    }

    async fn load(&self, spec: &PluginSpec) -> PluginResult<LoadedPlugin> {
        Ok(LoadedPlugin { spec: spec.clone() })
    }

    async fn invoke_hook(
        &self,
        plugin: &LoadedPlugin,
        _hook: HookCall,
    ) -> PluginResult<HookResult> {
        Ok(HookResult {
            plugin: plugin.spec.name.clone(),
            output: serde_json::json!({"host": "wasm", "stub": true}),
        })
    }

    async fn unload(&self, _plugin: &LoadedPlugin) -> PluginResult<()> {
        Ok(())
    }
}
