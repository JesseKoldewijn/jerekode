//! Plugin orchestrator and host implementations (Bun, native, WASM).

mod bun_host;
mod error;
mod host;
mod native_host;
mod orchestrator;
mod sidecar;
mod tui;
mod types;
mod wasm_host;

pub use bun_host::BunPluginHost;
pub use error::{PluginError, PluginResult};
pub use host::PluginHost;
pub use native_host::NativePluginHost;
pub use orchestrator::PluginOrchestrator;
pub use sidecar::{
    BunProcessSidecarPort, InMemorySidecarPort, SidecarInbound, SidecarOutbound, SidecarPort,
    run_sidecar_loop,
};
pub use tui::{render_stub_frame, run_interactive};
pub use types::{HookCall, HookResult, HostId, LoadedPlugin, PluginSpec};
pub use wasm_host::WasmPluginHost;
