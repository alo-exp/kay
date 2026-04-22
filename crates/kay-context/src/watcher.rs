use crate::error::ContextError;
use notify_debouncer_mini::{DebounceEventResult, DebouncedEventKind, new_debouncer};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

const DEBOUNCE_MS: u64 = 500;

/// Source file extensions that should trigger cache invalidation.
const WATCHED_EXTS: &[&str] = &["rs", "ts", "tsx", "py", "go"];

/// Path substrings that should be ignored (build artefacts, VCS metadata).
const IGNORED_SEGMENTS: &[&str] = &["target/", ".git/"];

/// File extensions that should be ignored (lock files, editor temporaries).
const IGNORED_EXTS: &[&str] = &["lock", "tmp", "swp"];

pub struct FileWatcher {
    _watcher: notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>,
}

impl FileWatcher {
    /// Start watching `root` recursively. `on_invalidate` is called (from a
    /// background thread) whenever a source file changes, after a 500 ms
    /// debounce window.
    pub fn new(
        root: &Path,
        on_invalidate: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, ContextError> {
        // Use Arc<dyn Fn() + Send + Sync> rather than Arc<Mutex<_>>.
        // A Mutex is unnecessary here because the callback is never replaced —
        // only called — and Mutex::lock() would be permanently poisoned if the
        // callback ever panicked, silently killing the watcher.
        let callback: Arc<dyn Fn() + Send + Sync> = Arc::new(on_invalidate);

        let mut debouncer = new_debouncer(
            Duration::from_millis(DEBOUNCE_MS),
            move |result: DebounceEventResult| {
                let events = match result {
                    Ok(events) => events,
                    Err(_) => return,
                };
                let triggered = events.iter().any(|e| {
                    // DebouncedEventKind only has Any and AnyContinuous; both
                    // indicate a real FS change so we accept either.
                    matches!(
                        e.kind,
                        DebouncedEventKind::Any | DebouncedEventKind::AnyContinuous
                    ) && is_source_file(&e.path)
                        && !should_ignore(&e.path)
                });
                if triggered {
                    callback();
                }
            },
        )
        .map_err(|e| std::io::Error::other(format!("notify debouncer init: {e}")))?;

        debouncer
            .watcher()
            .watch(root, notify::RecursiveMode::Recursive)
            .map_err(|e| std::io::Error::other(format!("notify watch: {e}")))?;

        Ok(Self { _watcher: debouncer })
    }

    /// Stop the watcher. Dropping achieves the same effect; this method is
    /// provided for call-site clarity.
    pub fn stop(self) {
        // Dropping `_watcher` stops the background thread automatically.
    }
}

/// Returns `true` when the file has a source extension we care about.
fn is_source_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| WATCHED_EXTS.contains(&ext))
        .unwrap_or(false)
}

/// Returns `true` when the file should be silently ignored.
fn should_ignore(path: &std::path::Path) -> bool {
    let path_str = path.to_string_lossy();
    if IGNORED_SEGMENTS.iter().any(|seg| path_str.contains(seg)) {
        return true;
    }
    if path
        .extension()
        .and_then(|e| e.to_str())
        .map(|ext| IGNORED_EXTS.contains(&ext))
        .unwrap_or(false)
    {
        return true;
    }
    false
}
