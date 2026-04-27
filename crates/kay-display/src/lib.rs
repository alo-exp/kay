//! Kay Display - Streaming markdown renderer for terminal output.
//!
//! This crate provides Forge-compatible streaming markdown rendering with:
//! - Proper heading hierarchy (H1 uppercase, H2 bold, H3+ italic)
//! - Bold and italic inline formatting
//! - Bullet and numbered lists with proper indentation
//! - Table rendering with borders
//! - Code block rendering
//! - Reasoning block dimming
//!
//! # Design (Cleanroom implementation inspired by forge_markdown_stream)
//!
//! - Buffers incoming tokens until complete lines
//! - Renders inline markdown (bold, italic, code)
//! - Handles block elements (headings, lists, tables)
//! - Supports reasoning mode (dimmed output)

pub mod markdown;
pub mod theme;
pub mod spinner;

pub use markdown::MarkdownRenderer;
pub use theme::Theme;

// Re-export for convenience
pub type Result<T> = std::result::Result<T, std::io::Error>;
