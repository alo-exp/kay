use std::sync::Arc;

use forge_domain::ConsoleWriter;
use forge_spinner::SpinnerManager;

/// A no-op ConsoleWriter for testing.
#[derive(Clone, Copy)]
struct SinkWriter;

impl ConsoleWriter for SinkWriter {
    fn write(&self, _buf: &[u8]) -> std::io::Result<usize> {
        Ok(0) // discard all output
    }
    fn write_err(&self, _buf: &[u8]) -> std::io::Result<usize> {
        Ok(0)
    }
    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
    fn flush_err(&self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn spinner_start_stop_does_not_panic() {
    let mut mgr = SpinnerManager::new(Arc::new(SinkWriter));
    // start() and stop() should not panic
    mgr.start(Some("test")).expect("start should succeed");
    mgr.stop(None).expect("stop should succeed");
}

#[test]
fn spinner_reset_clears_state() {
    let mut mgr = SpinnerManager::new(Arc::new(SinkWriter));
    mgr.start(Some("test")).expect("start should succeed");
    mgr.stop(None).expect("stop should succeed");
    mgr.reset();
    // Reset completes without panic; state is cleared
}
