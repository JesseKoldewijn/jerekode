//! NativePluginHost — in-process dylib via stable C ABI.

use crate::error::{PluginError, PluginResult};
use crate::host::PluginHost;
use crate::types::{HookCall, HookResult, HostId, LoadedPlugin, PluginSpec};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

/// Native host — loads dylibs via `libloading` when a path is provided.
///
/// Without a loaded library, invoke returns a structured stub for orchestrator tests.
pub struct NativePluginHost {
    /// Optional default library path used when the plugin spec name looks like a path.
    default_path: Option<String>,
    libraries: Mutex<HashMap<String, libloading::Library>>,
}

impl NativePluginHost {
    pub fn new() -> Self {
        Self {
            default_path: None,
            libraries: Mutex::new(HashMap::new()),
        }
    }

    pub fn with_library_path(library_path: impl Into<String>) -> Self {
        Self {
            default_path: Some(library_path.into()),
            libraries: Mutex::new(HashMap::new()),
        }
    }

    fn resolve_path(&self, spec: &PluginSpec) -> String {
        if spec.name.contains('/') || spec.name.contains('\\') || spec.name.contains('.') {
            return spec.name.clone();
        }
        self.default_path
            .clone()
            .unwrap_or_else(|| spec.name.clone())
    }
}

impl Default for NativePluginHost {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PluginHost for NativePluginHost {
    fn host_id(&self) -> HostId {
        HostId("native".into())
    }

    async fn load(&self, spec: &PluginSpec) -> PluginResult<LoadedPlugin> {
        let path = self.resolve_path(spec);
        if path.is_empty() {
            return Err(PluginError::Native("empty library path".into()));
        }

        // Attempt real load; if the file is missing, keep a soft stub for config smoke paths.
        if std::path::Path::new(&path).exists() {
            // SAFETY: plugin authors must export the C ABI from jereko_plugin.h.
            let lib = unsafe { libloading::Library::new(&path) }.map_err(|e| {
                PluginError::Native(format!("failed to load native plugin '{path}': {e}"))
            })?;
            self.libraries
                .lock()
                .expect("native host lock poisoned")
                .insert(spec.name.clone(), lib);
        } else {
            tracing::debug!(%path, "native plugin path missing; stub mode");
        }

        Ok(LoadedPlugin { spec: spec.clone() })
    }

    async fn invoke_hook(&self, plugin: &LoadedPlugin, hook: HookCall) -> PluginResult<HookResult> {
        let libraries = self.libraries.lock().expect("native host lock poisoned");
        if let Some(lib) = libraries.get(&plugin.spec.name) {
            type InfoFn = unsafe extern "C" fn() -> JerekoPluginInfo;
            type InvokeFn = unsafe extern "C" fn(
                *const std::ffi::c_char,
                *const std::ffi::c_char,
            ) -> JerekoHookResult;

            // SAFETY: symbols match jereko_plugin.h.
            let invoke: libloading::Symbol<InvokeFn> = unsafe {
                lib.get(b"jereko_plugin_invoke\0").map_err(|e| {
                    PluginError::Native(format!("missing jereko_plugin_invoke: {e}"))
                })?
            };
            let _info: libloading::Symbol<InfoFn> = unsafe {
                lib.get(b"jereko_plugin_info\0")
                    .map_err(|e| PluginError::Native(format!("missing jereko_plugin_info: {e}")))?
            };

            let hook_c = std::ffi::CString::new(hook.hook.as_str())
                .map_err(|e| PluginError::Native(format!("hook name: {e}")))?;
            let payload = serde_json::to_string(&hook.payload)
                .map_err(|e| PluginError::Native(format!("payload json: {e}")))?;
            let payload_c = std::ffi::CString::new(payload)
                .map_err(|e| PluginError::Native(format!("payload cstr: {e}")))?;

            let result = unsafe { invoke(hook_c.as_ptr(), payload_c.as_ptr()) };
            if result.status != 0 {
                return Err(PluginError::Native(format!(
                    "plugin returned status {}",
                    result.status
                )));
            }
            let output = if result.json_output.is_null() {
                serde_json::json!({})
            } else {
                let cstr = unsafe { std::ffi::CStr::from_ptr(result.json_output) };
                let text = cstr
                    .to_str()
                    .map_err(|e| PluginError::Native(format!("output utf8: {e}")))?;
                serde_json::from_str(text).unwrap_or_else(|_| serde_json::json!({ "raw": text }))
            };
            return Ok(HookResult {
                plugin: plugin.spec.name.clone(),
                output,
            });
        }

        Ok(HookResult {
            plugin: plugin.spec.name.clone(),
            output: serde_json::json!({
                "host": "native",
                "hook": hook.hook,
                "stub": true
            }),
        })
    }

    async fn unload(&self, plugin: &LoadedPlugin) -> PluginResult<()> {
        self.libraries
            .lock()
            .expect("native host lock poisoned")
            .remove(&plugin.spec.name);
        Ok(())
    }
}

#[repr(C)]
struct JerekoPluginInfo {
    abi_version: u32,
    name: *const std::ffi::c_char,
    version: *const std::ffi::c_char,
}

#[repr(C)]
struct JerekoHookResult {
    status: i32,
    json_output: *const std::ffi::c_char,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_dylib_path() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop();
        path.pop();
        path.push("target");
        path.push("debug");
        #[cfg(target_os = "windows")]
        path.push("jereko_test_native_plugin.dll");
        #[cfg(target_os = "macos")]
        path.push("libjereko_test_native_plugin.dylib");
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        path.push("libjereko_test_native_plugin.so");
        path
    }

    #[tokio::test]
    async fn loads_invokes_and_unloads_test_dylib() {
        let path = test_dylib_path();
        if !path.exists() {
            let msg = format!(
                "native dylib missing at {}; build jereko-test-native-plugin first",
                path.display()
            );
            if std::env::var_os("CI").is_some() {
                panic!("{msg}");
            }
            eprintln!("skipping: {msg}");
            return;
        }

        let host = NativePluginHost::new();
        let spec = PluginSpec {
            name: path.to_string_lossy().into_owned(),
            host: HostId("native".into()),
        };
        let loaded = host.load(&spec).await.expect("load dylib");
        let result = host
            .invoke_hook(
                &loaded,
                HookCall {
                    hook: "before_transform".into(),
                    payload: serde_json::json!({"input": "hello"}),
                },
            )
            .await
            .expect("invoke");
        assert_eq!(result.output["host"], "native");
        assert_eq!(result.output["stub"], false);
        assert_eq!(result.output["transformed"], "hello");
        host.unload(&loaded).await.expect("unload");
    }

    #[tokio::test]
    async fn empty_path_errors() {
        let host = NativePluginHost::new();
        let err = host
            .load(&PluginSpec {
                name: String::new(),
                host: HostId("native".into()),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, PluginError::Native(_)));
    }
}
