use std::path::Path;
use crate::error::SessionError;

#[derive(Debug)]
pub struct SessionStore {
    // TODO W-1 GREEN: add conn: rusqlite::Connection, sessions_dir: PathBuf
}

impl SessionStore {
    pub fn open(_root: &Path) -> Result<Self, SessionError> {
        unimplemented!("W-1 GREEN: implement SessionStore::open")
    }
}
