use jerekode_config::OpenCodeConfig;
use jerekode_plugins::PluginOrchestrator;
use jerekode_providers::ProviderRegistry;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::extensions::ExtensionHosts;
use crate::handlers::HandlerContext;
use crate::persistence::SqliteSessionStore;
use crate::session_store::{SessionStore, SessionStorePort};
use crate::tools::ToolExecutor;

/// Shared application state passed to HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    pub ctx: Arc<HandlerContext>,
    pub extensions: ExtensionHosts,
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
        let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            ctx: Arc::new(HandlerContext {
                sessions,
                providers,
                default_provider: config.provider.clone(),
                default_model: config.model.clone(),
                tools: ToolExecutor::new(project_root).with_bash(true),
                plugins: None,
            }),
            extensions: ExtensionHosts::new(),
        }
    }

    /// Attach a plugin orchestrator for tool hooks (e.g. `tool.execute.before`).
    pub fn with_plugins(self, orchestrator: PluginOrchestrator) -> Self {
        Self {
            ctx: Arc::new(HandlerContext {
                sessions: Arc::clone(&self.ctx.sessions),
                providers: Arc::clone(&self.ctx.providers),
                default_provider: self.ctx.default_provider.clone(),
                default_model: self.ctx.default_model.clone(),
                tools: self.ctx.tools.clone(),
                plugins: Some(Arc::new(RwLock::new(orchestrator))),
            }),
            extensions: self.extensions,
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
