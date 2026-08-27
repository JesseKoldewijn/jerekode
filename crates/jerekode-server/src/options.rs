//! Router construction options (CORS origins, basic auth mode).

/// How the serve router applies HTTP basic auth.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum BasicAuthMode {
    /// Read `OPENCODE_SERVER_PASSWORD` / jerekode equivalents at request time.
    #[default]
    FromEnv,
    /// Force credentials (tests / explicit wiring).
    Fixed { username: String, password: String },
    /// No basic auth (tests that must ignore ambient env).
    Disabled,
}

/// Options applied when building the Axum router for `serve`.
#[derive(Debug, Clone, Default)]
pub struct RouterOptions {
    /// Extra CORS origins from `serve --cors` (repeatable).
    pub cors_origins: Vec<String>,
    /// Basic auth mode (default: env-gated like OpenCode).
    pub basic_auth: BasicAuthMode,
}
