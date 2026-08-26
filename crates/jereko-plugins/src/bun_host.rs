use crate::error::PluginResult;
use crate::host::{host_error, PluginHost};
use crate::sidecar::{SidecarOutbound, SidecarPort};
use crate::types::{HookCall, HookResult, HostId, LoadedPlugin, PluginSpec};
use async_trait::async_trait;
use std::sync::Arc;

pub struct BunPluginHost {
    port: Arc<dyn SidecarPort>,
}

impl BunPluginHost {
    pub fn new(port: Arc<dyn SidecarPort>) -> Self {
        Self { port }
    }
}

#[async_trait]
impl PluginHost for BunPluginHost {
    fn host_id(&self) -> HostId {
        HostId("bun".into())
    }

    async fn load(&self, spec: &PluginSpec) -> PluginResult<LoadedPlugin> {
        self.port
            .send(SidecarOutbound::Init {
                config: serde_json::json!({}),
                plugins: vec![spec.name.clone()],
            })
            .await?;
        Ok(LoadedPlugin { spec: spec.clone() })
    }

    async fn invoke_hook(&self, plugin: &LoadedPlugin, hook: HookCall) -> PluginResult<HookResult> {
        if hook.hook == "tui.render" {
            self.port
                .send(SidecarOutbound::TuiRender {
                    frame: hook.payload,
                })
                .await?;
        }
        Ok(HookResult {
            plugin: plugin.spec.name.clone(),
            output: serde_json::json!({"status": "ok"}),
        })
    }

    async fn unload(&self, _plugin: &LoadedPlugin) -> PluginResult<()> {
        self.port.send(SidecarOutbound::Shutdown).await
    }
}

#[allow(dead_code)]
pub fn bun_host_error(message: impl Into<String>) -> crate::error::PluginError {
    host_error("bun", message)
}
