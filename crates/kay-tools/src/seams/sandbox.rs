//! Sandbox DI seam (D-12). NoOp in Phase 3; Phase 4 swaps per-OS impls.

use std::path::Path;

use url::Url;

#[derive(Debug, Clone)]
pub struct SandboxDenial {
    pub reason: String,
    pub resource: String,
}

#[async_trait::async_trait]
pub trait Sandbox: Send + Sync {
    async fn check_shell(&self, command: &str, cwd: &Path) -> Result<(), SandboxDenial>;
    async fn check_fs_read(&self, path: &Path) -> Result<(), SandboxDenial>;
    async fn check_fs_write(&self, path: &Path) -> Result<(), SandboxDenial>;
    async fn check_fs_search(&self, path: &Path) -> Result<(), SandboxDenial> {
        self.check_fs_read(path).await
    }
    async fn check_net(&self, url: &Url) -> Result<(), SandboxDenial>;
}

pub struct NoOpSandbox;

#[async_trait::async_trait]
impl Sandbox for NoOpSandbox {
    async fn check_shell(&self, _c: &str, _w: &Path) -> Result<(), SandboxDenial> { Ok(()) }
    async fn check_fs_read(&self, _p: &Path) -> Result<(), SandboxDenial> { Ok(()) }
    async fn check_fs_write(&self, _p: &Path) -> Result<(), SandboxDenial> { Ok(()) }
    async fn check_net(&self, url: &Url) -> Result<(), SandboxDenial> {
        // Minimal Phase 3 protection (03-RESEARCH Threat #3): reject file:// even in NoOp.
        if url.scheme() == "file" {
            return Err(SandboxDenial {
                reason: "file:// scheme blocked in Phase 3 NoOpSandbox".into(),
                resource: url.to_string(),
            });
        }
        Ok(())
    }
}
