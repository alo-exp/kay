use thiserror::Error;

#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("sandbox backend unavailable: {0}")]
    BackendUnavailable(String),
    #[error("sandbox platform not supported on this OS")]
    PlatformNotSupported,
    #[error("sandbox policy build failed: {0}")]
    PolicyBuild(String),
    #[error("sandbox enforcement error: {0}")]
    Enforcement(String),
}
