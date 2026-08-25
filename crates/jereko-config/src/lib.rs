//! Configuration loading with OpenCode-compatible precedence semantics.
//!
//! # Precedence (lowest → highest)
//!
//! 1. **Built-in defaults** — hard-coded safe defaults in this crate.
//! 2. **Global config** — `~/.config/opencode/opencode.json` (and `tui.json`).
//! 3. **Project config** — `<project>/.opencode/opencode.json` (and `tui.json`).
//! 4. **Environment overrides** — `JEREKO_*` / `OPENCODE_*` env vars (Phase 1).
//! 5. **CLI flags** — highest precedence (Phase 1).
//!
//! Later layers override earlier layers. Arrays and maps merge by key where
//! OpenCode semantics require deep merge; scalar values are replaced.
//!
//! # JSONC
//!
//! Config files use JSONC on disk. This stub parses strict JSON via
//! `serde_json`; full JSONC support (comments, trailing commas) is Phase 1.

mod error;
mod loader;
mod types;

pub use error::{ConfigError, ConfigResult};
pub use loader::{ConfigLayer, ConfigLoader, MergeStrategy};
pub use types::{OpenCodeConfig, TuiConfig};
