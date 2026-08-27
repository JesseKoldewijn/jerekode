use crate::error::{PluginError, PluginResult};
use crate::types::{HookCall, HookResult, HostId, LoadedPlugin, PluginSpec};
use async_trait::async_trait;

#[async_trait]
pub trait PluginHost: Send + Sync {
    fn host_id(&self) -> HostId;

    async fn load(&self, spec: &PluginSpec) -> PluginResult<LoadedPlugin>;

    async fn invoke_hook(&self, plugin: &LoadedPlugin, hook: HookCall) -> PluginResult<HookResult>;

    async fn unload(&self, plugin: &LoadedPlugin) -> PluginResult<()>;
}

#[allow(dead_code)]
pub fn host_error(host: &str, message: impl Into<String>) -> PluginError {
    PluginError::Host {
        host: host.into(),
        message: message.into(),
    }
}
