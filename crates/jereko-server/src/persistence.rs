//! SQLite session persistence stub (Phase 4).

use jereko_core::Session;

#[derive(Debug, Default)]
pub struct SqliteSessionStore {
    enabled: bool,
}

impl SqliteSessionStore {
    pub fn new_stub() -> Self {
        Self { enabled: false }
    }

    pub fn persist(&self, _session: &Session) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }
        Err("SQLite persistence not yet implemented".into())
    }
}
