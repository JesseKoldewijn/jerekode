use jereko_providers::ProviderRegistry;
use std::sync::Arc;

/// Shared application state passed to HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    pub providers: Arc<ProviderRegistry>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            providers: Arc::new(ProviderRegistry::with_stubs()),
        }
    }
}
