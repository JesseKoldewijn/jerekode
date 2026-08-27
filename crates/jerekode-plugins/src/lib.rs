//! Plugin orchestrator and host implementations (Bun, native, WASM).

#[cfg(feature = "bun-sidecar")]
mod bun_host;
mod error;
mod hooks;
mod host;
mod native_host;
mod orchestrator;
mod sidecar;
mod tui;
mod types;
mod wasm_host;

#[cfg(feature = "bun-sidecar")]
pub use bun_host::BunPluginHost;
pub use error::{BUN_SIDECAR_UNAVAILABLE_MSG, PluginError, PluginResult};
pub use hooks::{TOOL_EXECUTE_BEFORE, apply_command_mutations, bash_before_hook, set_command_arg};
pub use host::PluginHost;
pub use native_host::NativePluginHost;
pub use orchestrator::PluginOrchestrator;
#[cfg(feature = "bun-sidecar")]
pub use sidecar::BunProcessSidecarPort;
pub use sidecar::{
    InMemorySidecarPort, SidecarInbound, SidecarOutbound, SidecarPort, run_sidecar_loop,
};
pub use tui::{render_stub_frame, run_interactive};
pub use types::{HookCall, HookResult, HostId, LoadedPlugin, PluginSpec};
pub use wasm_host::WasmPluginHost;
