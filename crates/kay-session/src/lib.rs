//! kay-session — Session store, JSONL transcript, and snapshot persistence.
//!
//! See .planning/phases/06-session-store/06-CONTEXT.md for decisions DL-1..DL-9.
#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod config;
pub mod error;
pub mod store;

pub use config::kay_home;
pub use error::SessionError;
pub use store::SessionStore;
