//! Provider registry designed for 75+ providers with plugin-ready extension points.
//!
//! Providers are registered by id and resolved at runtime. Built-in providers
//! ship in this crate; third-party providers can register via the Bun sidecar
//! plugin host in Phase 2.

mod error;
mod provider;
mod registry;

pub use error::{ProviderError, ProviderResult};
pub use provider::{CompletionRequest, CompletionResponse, ModelInfo, Provider, ProviderId};
pub use registry::ProviderRegistry;
