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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use url::Url;

    use super::*;

    #[tokio::test]
    async fn noop_allows_default_fs_ops() {
        let s = NoOpSandbox;
        let p = Path::new("/tmp/x");
        assert!(s.check_fs_read(p).await.is_ok());
        assert!(s.check_fs_write(p).await.is_ok());
        assert!(s.check_fs_search(p).await.is_ok());
        assert!(s.check_shell("echo hi", p).await.is_ok());
    }

    #[tokio::test]
    async fn noop_blocks_file_url_scheme() {
        let s = NoOpSandbox;
        let u = match Url::parse("file:///etc/passwd") {
            Ok(u) => u,
            Err(e) => panic!("parse failed: {e}"),
        };
        let res = s.check_net(&u).await;
        assert!(res.is_err(), "file:// must be rejected by Phase 3 NoOpSandbox");
        if let Err(d) = res {
            assert!(d.reason.contains("file://"), "reason: {}", d.reason);
            assert!(
                d.resource.contains("file:///etc/passwd"),
                "resource: {}",
                d.resource
            );
        }
    }

    #[tokio::test]
    async fn noop_allows_http_and_https() {
        let s = NoOpSandbox;
        let u = match Url::parse("https://openrouter.ai/api") {
            Ok(u) => u,
            Err(e) => panic!("parse https failed: {e}"),
        };
        assert!(s.check_net(&u).await.is_ok());
        let u2 = match Url::parse("http://example.com") {
            Ok(u) => u,
            Err(e) => panic!("parse http failed: {e}"),
        };
        assert!(s.check_net(&u2).await.is_ok());
    }
}
