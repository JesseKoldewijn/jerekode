use crate::error::{PluginError, PluginResult};
use crate::host::{PluginHost, host_error};
use crate::sidecar::{SidecarInbound, SidecarOutbound, SidecarPort};
use crate::types::{HookCall, HookResult, HostId, LoadedPlugin, PluginSpec};
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct BunPluginHost {
    port: Arc<dyn SidecarPort>,
    next_request_id: AtomicU64,
}

impl BunPluginHost {
    pub fn new(port: Arc<dyn SidecarPort>) -> Self {
        Self {
            port,
            next_request_id: AtomicU64::new(1),
        }
    }

    async fn wait_ready(&self) -> PluginResult<()> {
        loop {
            match self.port.recv().await? {
                SidecarInbound::Ready => return Ok(()),
                SidecarInbound::Log { level, message } => {
                    tracing::debug!(%level, %message, "sidecar log while waiting for ready");
                }
                SidecarInbound::Error { message } => {
                    return Err(PluginError::Sidecar(message));
                }
                SidecarInbound::PluginEvent { .. }
                | SidecarInbound::TuiRender { .. }
                | SidecarInbound::HookResult { .. } => {}
            }
        }
    }

    async fn wait_hook_result(
        &self,
        request_id: &str,
    ) -> PluginResult<(String, serde_json::Value)> {
        loop {
            match self.port.recv().await? {
                SidecarInbound::HookResult {
                    request_id: id,
                    plugin,
                    output,
                } if id == request_id => {
                    return Ok((plugin, output));
                }
                SidecarInbound::Log { level, message } => {
                    tracing::debug!(%level, %message, "sidecar log during hook");
                }
                SidecarInbound::Ready => {}
                SidecarInbound::Error { message } => {
                    return Err(PluginError::Sidecar(message));
                }
                SidecarInbound::PluginEvent { .. } | SidecarInbound::TuiRender { .. } => {}
                SidecarInbound::HookResult { .. } => {
                    // Unrelated / out-of-order result — ignore for this wait.
                }
            }
        }
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
        // Init finishes with Ready once plugins are loaded — wait so invoke is not raced.
        self.wait_ready().await?;
        Ok(LoadedPlugin { spec: spec.clone() })
    }

    async fn invoke_hook(&self, plugin: &LoadedPlugin, hook: HookCall) -> PluginResult<HookResult> {
        if hook.hook == "tui.render" {
            self.port
                .send(SidecarOutbound::TuiRender {
                    frame: hook.payload.clone(),
                })
                .await?;
        }

        let request_id = self
            .next_request_id
            .fetch_add(1, Ordering::Relaxed)
            .to_string();
        self.port
            .send(SidecarOutbound::InvokeHook {
                request_id: request_id.clone(),
                plugin: plugin.spec.name.clone(),
                hook: hook.hook,
                payload: hook.payload,
            })
            .await?;

        let (plugin_name, output) = self.wait_hook_result(&request_id).await?;
        Ok(HookResult {
            plugin: plugin_name,
            output,
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
