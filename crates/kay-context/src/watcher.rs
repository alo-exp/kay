use crate::error::ContextError;
use std::path::Path;

pub struct FileWatcher;

impl FileWatcher {
    pub fn new(
        _root: &Path,
        _on_invalidate: impl Fn() + Send + 'static,
    ) -> Result<Self, ContextError> {
        todo!("W-7 implementation")
    }

    pub fn stop(self) {}
}
