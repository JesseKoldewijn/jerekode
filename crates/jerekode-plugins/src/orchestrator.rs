use crate::error::{PluginError, PluginResult};
use crate::host::PluginHost;
use crate::types::{HookCall, HookResult, HostId, LoadedPlugin, PluginSpec};
use jerekode_config::PluginEntry;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
enum LoadTier {
    Internal = 0,
    Native = 1,
    Wasm = 2,
    Bun = 3,
}

struct RegisteredPlugin {
    tier: LoadTier,
    priority: i32,
    order: usize,
    plugin: LoadedPlugin,
    host: Arc<dyn PluginHost>,
}

/// Central plugin coordinator — ordered hook dispatch across all hosts.
pub struct PluginOrchestrator {
    hosts: Vec<Arc<dyn PluginHost>>,
    plugins: Vec<RegisteredPlugin>,
    next_order: usize,
}

impl PluginOrchestrator {
    pub fn new(hosts: Vec<Arc<dyn PluginHost>>) -> Self {
        Self {
            hosts,
            plugins: Vec::new(),
            next_order: 0,
        }
    }

    pub fn register_host(&mut self, host: Arc<dyn PluginHost>) {
        self.hosts.push(host);
    }

    pub fn resolve_host(&self, id: &str) -> Option<Arc<dyn PluginHost>> {
        self.hosts.iter().find(|h| h.host_id().0 == id).cloned()
    }

    pub async fn load_from_config(&mut self, entries: &[PluginEntry]) -> PluginResult<()> {
        for entry in entries {
            let (host_id, name, tier) = match entry {
                PluginEntry::Bun(s) => ("bun", s.clone(), LoadTier::Bun),
                PluginEntry::Native { native } => ("native", native.clone(), LoadTier::Native),
                PluginEntry::Wasm { wasm } => ("wasm", wasm.clone(), LoadTier::Wasm),
                PluginEntry::Named { name, .. } => ("bun", name.clone(), LoadTier::Bun),
            };

            let host = self.resolve_host(host_id).ok_or_else(|| {
                if host_id == "bun" {
                    PluginError::Orchestrator(format!(
                        "Bun/TS plugin '{name}' is configured but the Bun sidecar host is unavailable: {}",
                        crate::error::BUN_SIDECAR_UNAVAILABLE_MSG
                    ))
                } else {
                    PluginError::Orchestrator(format!("host not registered: {host_id}"))
                }
            })?;

            let spec = PluginSpec {
                name,
                host: HostId(host_id.into()),
            };
            let loaded = host.load(&spec).await?;
            let order = self.next_order;
            self.next_order += 1;
            self.plugins.push(RegisteredPlugin {
                tier,
                priority: 0,
                order,
                plugin: loaded,
                host,
            });
        }
        Ok(())
    }

    pub async fn dispatch_hook(&self, hook: HookCall) -> PluginResult<Vec<HookResult>> {
        let mut indices: Vec<usize> = (0..self.plugins.len()).collect();
        indices.sort_by_key(|&i| {
            let p = &self.plugins[i];
            (p.tier, std::cmp::Reverse(p.priority), p.order)
        });

        let mut results = Vec::new();
        for i in indices {
            let entry = &self.plugins[i];
            match entry.host.invoke_hook(&entry.plugin, hook.clone()).await {
                Ok(result) => results.push(result),
                Err(err) => {
                    tracing::warn!(plugin = %entry.plugin.spec.name, %err, "plugin hook failed");
                }
            }
        }
        Ok(results)
    }

    pub fn loaded_count(&self) -> usize {
        self.plugins.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_host::NativePluginHost;

    #[cfg(feature = "bun-sidecar")]
    #[tokio::test]
    async fn dispatches_across_bun_and_native_hosts() {
        use crate::bun_host::BunPluginHost;
        use crate::sidecar::InMemorySidecarPort;

        let port = Arc::new(InMemorySidecarPort::new());
        let bun = Arc::new(BunPluginHost::new(port.clone()));
        let native = Arc::new(NativePluginHost::with_library_path("./test.so"));

        let mut orchestrator = PluginOrchestrator::new(vec![native.clone(), bun.clone()]);
        orchestrator
            .load_from_config(&[
                PluginEntry::Native {
                    native: "./tools.so".into(),
                },
                PluginEntry::Bun("@acme/server-plugin".into()),
            ])
            .await
            .unwrap();

        assert_eq!(orchestrator.loaded_count(), 2);
        let results = orchestrator
            .dispatch_hook(HookCall {
                hook: "before_transform".into(),
                payload: serde_json::json!({"input": "test"}),
            })
            .await
            .unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn bun_plugin_errors_clearly_without_bun_host() {
        let native = Arc::new(NativePluginHost::with_library_path("./test.so"));
        let mut orchestrator = PluginOrchestrator::new(vec![native]);
        let err = orchestrator
            .load_from_config(&[PluginEntry::Bun("@acme/server-plugin".into())])
            .await
            .expect_err("bun plugin without bun host must fail");
        let msg = err.to_string();
        assert!(
            msg.contains("Bun/TS plugin") && msg.contains("without Bun sidecar support"),
            "unexpected error: {msg}"
        );
    }

    #[tokio::test]
    async fn native_plugin_errors_without_native_host() {
        let mut orchestrator = PluginOrchestrator::new(vec![]);
        let err = orchestrator
            .load_from_config(&[PluginEntry::Native {
                native: "./tools.so".into(),
            }])
            .await
            .expect_err("native plugin without native host must fail");
        let msg = err.to_string();
        assert!(
            msg.contains("host not registered: native"),
            "unexpected error: {msg}"
        );
    }
}
