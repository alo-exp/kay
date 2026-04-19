//! Shared integration-test helpers for kay-provider-openrouter.
//!
//! Per Phase 2 plan 02-01 (Wave 0): this module hosts the MockServer
//! wrapper + SSE cassette loaders that every downstream test uses.
//! See `.planning/phases/02-provider-hal-tolerant-json-parser/02-PATTERNS.md`
//! §`tests/mock_server.rs` for the analog and extension rationale.

pub mod mock_server;
