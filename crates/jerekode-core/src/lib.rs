//! Core domain types and session models for Jerekode.
//!
//! This crate holds provider-agnostic business logic. HTTP adapters and CLI
//! wiring live in other crates; provider implementations live in `jerekode-providers`.

pub mod error;
pub mod session;

pub use error::{CoreError, CoreResult};
pub use session::{Message, MessageRole, Session, SessionId, SessionStatus};
