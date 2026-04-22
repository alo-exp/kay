//! kay-context — local tree-sitter symbol store + hybrid retrieval.
//! Phase 7: CTX-01..CTX-05.
#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod budget;
pub mod embedder;
pub mod engine;
pub mod error;
pub mod hardener;
pub mod indexer;
pub mod language;
pub mod retriever;
pub mod store;
pub mod watcher;

pub use budget::{ContextBudget, ContextPacket, estimate_tokens};
pub use embedder::{EmbeddingProvider, NoOpEmbedder};
pub use engine::{ContextEngine, KayContextEngine, NoOpContextEngine};
pub use error::ContextError;
pub use hardener::SchemaHardener;
pub use indexer::{IndexStats, TreeSitterIndexer};
pub use language::Language;
pub use retriever::Retriever;
pub use store::{Symbol, SymbolKind, SymbolStore};
pub use watcher::FileWatcher;
