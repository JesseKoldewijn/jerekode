use jereko_config::OpenCodeConfig;
use jereko_providers::ProviderRegistry;
use std::sync::Arc;

use crate::handlers::HandlerContext;
use crate::session_store::SessionStore;

/// Shared application state passed to HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    pub ctx: Arc<HandlerContext>,
}

impl AppState {
    pub fn new(config: &OpenCodeConfig) -> Self {
        Self {
            ctx: Arc::new(HandlerContext {
                sessions: Arc::new(SessionStore::new()),
                providers: Arc::new(ProviderRegistry::with_stubs()),
                default_provider: config.provider.clone(),
                default_model: config.model.clone(),
            }),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(&OpenCodeConfig::default())
    }
}
