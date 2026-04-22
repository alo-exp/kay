use crate::error::ContextError;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

pub struct SymbolStore {
    pub conn: Connection,
    pub db_path: PathBuf,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Method,
    Trait,
    Struct,
    Enum,
    Module,
    Class,
    FileBoundary,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: i64,
    pub name: String,
    pub kind: SymbolKind,
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub sig: String,
}

impl SymbolStore {
    pub fn open(_root: &Path) -> Result<Self, ContextError> {
        todo!("W-1 implementation")
    }
}
