//! Plugin orchestrator and host implementations (Bun, native, WASM stubs).

mod bun_host;
mod error;
mod host;
mod native_host;
mod orchestrator;
mod sidecar;
mod types;
mod wasm_host;

pub use bun_host::BunPluginHost;
pub use error::{PluginError, PluginResult};
pub use host::PluginHost;
pub use native_host::NativePluginHost;
pub use orchestrator::PluginOrchestrator;
pub use sidecar::{
    run_sidecar_loop, BunProcessSidecarPort, InMemorySidecarPort, SidecarInbound, SidecarOutbound,
    SidecarPort,
};
pub use types::{HookCall, HookResult, HostId, LoadedPlugin, PluginSpec};
pub use wasm_host::WasmPluginHost;
