use std::path::PathBuf;

/// Returns the Kay home directory.
///
/// Resolves `KAY_HOME` env var first; falls back to `~/.kay`.
/// Re-evaluated on every call — no caching — so tests can override via env var
/// without inter-test interference (DL-3, I-4).
pub fn kay_home() -> PathBuf {
    if let Ok(path) = std::env::var("KAY_HOME") {
        return PathBuf::from(path);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".kay")
}
