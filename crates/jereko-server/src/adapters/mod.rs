//! HTTP adapter layer: v1 and v2 normalize into canonical internal types.
//!
//! ```text
//!  Client (v1) ──► v1 adapter ──► normalized ──► handlers ──► normalized ──► v1 response
//!  Client (v2) ──► v2 adapter ──► normalized ──► handlers ──► normalized ──► v2 response
//! ```
//!
//! Handlers MUST only accept/return `normalized` types. Version-specific
//! serde shapes stay inside their adapter modules.

pub mod normalized;
pub mod v1;
pub mod v2;
