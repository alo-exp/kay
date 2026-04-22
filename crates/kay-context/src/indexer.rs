use crate::error::ContextError;
use crate::store::SymbolStore;
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct IndexStats {
    pub files: usize,
    pub symbols: usize,
    pub skipped_files: usize,
}

pub struct TreeSitterIndexer;

impl TreeSitterIndexer {
    pub fn new() -> Self {
        Self
    }

    pub async fn index_file(
        &self,
        _path: &Path,
        _store: &SymbolStore,
    ) -> Result<IndexStats, ContextError> {
        todo!("W-2 implementation")
    }
}

impl Default for TreeSitterIndexer {
    fn default() -> Self {
        Self::new()
    }
}
