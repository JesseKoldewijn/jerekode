use jereko_config::OpenCodeConfig;
use jereko_providers::ProviderRegistry;
use std::path::PathBuf;
use std::sync::Arc;

use crate::handlers::HandlerContext;
use crate::persistence::SqliteSessionStore;
use crate::session_store::{SessionStore, SessionStorePort};

/// Shared application state passed to HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    pub ctx: Arc<HandlerContext>,
}

impl AppState {
    /// Default state for tests — stub providers, in-memory sessions.
    pub fn new(config: &OpenCodeConfig) -> Self {
        Self::with_store_and_providers(
            config,
            Arc::new(SessionStore::new()),
            Arc::new(ProviderRegistry::with_stubs()),
        )
    }

    /// Production-oriented state — real HTTP providers + optional SQLite.
    pub fn production(config: &OpenCodeConfig) -> Result<Self, String> {
        let sessions: Arc<dyn SessionStorePort> = if let Some(path) = &config.session_db {
            Arc::new(SqliteSessionStore::open(path)?)
        } else {
            Arc::new(SessionStore::new())
        };
        Ok(Self::with_store_and_providers(
            config,
            sessions,
            Arc::new(ProviderRegistry::with_defaults()),
        ))
    }

    pub fn with_store(config: &OpenCodeConfig, sessions: Arc<dyn SessionStorePort>) -> Self {
        Self::with_store_and_providers(config, sessions, Arc::new(ProviderRegistry::with_stubs()))
    }

    pub fn with_store_and_providers(
        config: &OpenCodeConfig,
        sessions: Arc<dyn SessionStorePort>,
        providers: Arc<ProviderRegistry>,
    ) -> Self {
        Self {
            ctx: Arc::new(HandlerContext {
                sessions,
                providers,
                default_provider: config.provider.clone(),
                default_model: config.model.clone(),
            }),
        }
    }

    pub fn with_sqlite(config: &OpenCodeConfig, path: impl Into<PathBuf>) -> Result<Self, String> {
        let store = SqliteSessionStore::open(path.into())?;
        Ok(Self::with_store(config, Arc::new(store)))
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(&OpenCodeConfig::default())
    }
}
