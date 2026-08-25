//! NativePluginHost — in-process dylib via stable C ABI (Phase 2.5 stub).

use crate::error::{PluginError, PluginResult};
use crate::host::PluginHost;
use crate::types::{HookCall, HookResult, HostId, LoadedPlugin, PluginSpec};
use async_trait::async_trait;

/// Stub native host — libloading integration pending real dylib test harness.
pub struct NativePluginHost {
    _library_path: String,
}

impl NativePluginHost {
    pub fn new(library_path: impl Into<String>) -> Self {
        Self {
            _library_path: library_path.into(),
        }
    }

    /// Attempt to load a native plugin library (stub — validates path only).
    pub fn load_library_stub(path: &str) -> PluginResult<()> {
        if path.is_empty() {
            return Err(PluginError::Native("empty library path".into()));
        }
        tracing::debug!(path, "native plugin load stub");
        Ok(())
    }
}

#[async_trait]
impl PluginHost for NativePluginHost {
    fn host_id(&self) -> HostId {
        HostId("native".into())
    }

    async fn load(&self, spec: &PluginSpec) -> PluginResult<LoadedPlugin> {
        Self::load_library_stub(&spec.name)?;
        Ok(LoadedPlugin { spec: spec.clone() })
    }

    async fn invoke_hook(&self, plugin: &LoadedPlugin, hook: HookCall) -> PluginResult<HookResult> {
        Ok(HookResult {
            plugin: plugin.spec.name.clone(),
            output: serde_json::json!({
                "host": "native",
                "hook": hook.hook,
                "stub": true
            }),
        })
    }

    async fn unload(&self, _plugin: &LoadedPlugin) -> PluginResult<()> {
        Ok(())
    }
}
