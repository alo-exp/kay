//! kay-session — Session store, JSONL transcript, and snapshot persistence.
//!
//! See .planning/phases/06-session-store/06-CONTEXT.md for decisions DL-1..DL-9.
#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod config;
pub mod error;
pub mod export;
pub mod fork;
pub mod index;
pub mod snapshot;
pub mod store;
pub mod transcript;

pub use config::kay_home;
pub use error::SessionError;
pub use export::{ExportManifest, export_session, import_session, replay};
pub use index::{
    Session, SessionSummary, SessionStatus,
    create_session, list_sessions, close_session, resume_session, mark_session_lost,
};
pub use snapshot::{SessConfig, list_rewind_paths};
pub use store::SessionStore;
pub use transcript::TranscriptWriter;
