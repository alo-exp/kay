//! image_read — read an image file, enforce `ImageQuota`, emit
//! `AgentEvent::ImageRead`, return a `data:<mime>;base64,<blob>` URI.
//!
//! # Rule-3 reconciliations (03-05 Wave 4)
//!
//! 1. **Single `try_consume()` call** instead of separate
//!    `try_consume(CapScope::Turn)` + `try_consume(CapScope::Session)`
//!    calls. Reason: `ImageQuota::try_consume` checks both dimensions
//!    atomically in one call and returns the breached scope. Two
//!    sequential calls would leak a per-turn reservation if the
//!    per-session cap then failed. Keeps the tool a one-liner and
//!    makes the "rollback on breach" invariant visible.
//! 2. **MIME detection from extension only** (not magic bytes). The
//!    forge_services/image_read impl does the same — `ImageFormat`
//!    enum over `.jpg/.jpeg/.png/.webp/.gif`. Kay's ImageReadTool
//!    stays consistent with upstream.
//! 3. **Direct `tokio::fs::read` for the raw bytes**, but the path IS
//!    checked via `ctx.sandbox.check_fs_read(&path)` first (M-05).
//!    NoOpSandbox is a pass-through today; Phase 4's per-OS sandbox
//!    gains real enforcement here without further changes to this tool.
//!    Any size-limit / content-auth check Phase 5 adds will continue
//!    to happen at the sandbox layer.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use forge_domain::{ToolName, ToolOutput};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::contract::Tool;
use crate::error::ToolError;
use crate::events::AgentEvent;
use crate::runtime::context::ToolCallContext;
use crate::schema::{TruncationHints, harden_tool_schema};

/// Default `max_image_bytes` cap applied by `ImageReadTool::new`:
/// 20 MiB. Rationale (R-2): a prompt-injected
/// `image_read {"path": "/tmp/20GB.img"}` call must NOT cause the
/// agent to allocate gigabytes into `Vec<u8>`. 20 MiB comfortably
/// covers legitimate UI screenshots / assets while keeping the
/// allocation bounded. Callers that need a different ceiling use
/// `ImageReadTool::with_size_cap`.
pub const DEFAULT_MAX_IMAGE_BYTES: u64 = 20 * 1024 * 1024;

/// Input schema for `image_read`. Kay defines its own input struct
/// because ForgeCode's `ToolCatalog` does not expose an image-read
/// variant — the tool is Kay-specific (KIRA trio, IMG-01).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct ImageReadArgs {
    /// Absolute path to the image file to read.
    pub path: String,
}

pub struct ImageReadTool {
    name: ToolName,
    description: String,
    input_schema: Value,
    quota: Arc<crate::quota::ImageQuota>,
    /// Maximum file size (bytes, inclusive) that `invoke` will read.
    /// Enforced BEFORE `tokio::fs::read` via a `tokio::fs::metadata`
    /// length check — R-2 refuses pre-read to keep an over-cap call
    /// from allocating the file into memory.
    max_image_bytes: u64,
}

impl ImageReadTool {
    /// Construct with the default 20 MiB cap
    /// (`DEFAULT_MAX_IMAGE_BYTES`).
    pub fn new(quota: Arc<crate::quota::ImageQuota>) -> Self {
        Self::with_size_cap(quota, DEFAULT_MAX_IMAGE_BYTES)
    }

    /// Construct with an explicit `max_image_bytes` cap. The cap is
    /// INCLUSIVE — a file whose size is exactly `max_image_bytes`
    /// succeeds; `size > max_image_bytes` rejects with
    /// `ToolError::ImageTooLarge`.
    pub fn with_size_cap(
        quota: Arc<crate::quota::ImageQuota>,
        max_image_bytes: u64,
    ) -> Self {
        let name = ToolName::new("image_read");
        let description = "Read an image file from disk (JPEG/PNG/WebP/GIF) and return a \
            base64 data URI. Subject to per-turn and per-session image caps."
            .to_string();
        let mut schema = serde_json::to_value(schemars::schema_for!(ImageReadArgs))
            .unwrap_or_else(|_| serde_json::json!({ "type": "object" }));
        harden_tool_schema(
            &mut schema,
            &TruncationHints {
                output_truncation_note: Some(
                    "Image quota: max 2 per turn, 20 per session.".to_string(),
                ),
            },
        );
        Self { name, description, input_schema: schema, quota, max_image_bytes }
    }

    /// Configured maximum image file size (bytes, inclusive).
    pub fn max_image_bytes(&self) -> u64 {
        self.max_image_bytes
    }
}

/// Detect MIME type from file extension. Mirrors
/// `forge_services::tool_services::image_read::ImageFormat`.
fn detect_mime(path: &Path) -> Option<&'static str> {
    let ext = path.extension().and_then(|e| e.to_str())?;
    match ext.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "webp" => Some("image/webp"),
        "gif" => Some("image/gif"),
        _ => None,
    }
}

#[async_trait]
impl Tool for ImageReadTool {
    fn name(&self) -> &ToolName {
        &self.name
    }
    fn description(&self) -> &str {
        &self.description
    }
    fn input_schema(&self) -> Value {
        self.input_schema.clone()
    }

    async fn invoke(
        &self,
        args: Value,
        ctx: &ToolCallContext,
        _call_id: &str,
    ) -> Result<ToolOutput, ToolError> {
        let args = if args.is_null() {
            serde_json::json!({})
        } else {
            args
        };
        let input: ImageReadArgs = serde_json::from_value(args).map_err(|e| {
            ToolError::InvalidArgs { tool: self.name.clone(), reason: e.to_string() }
        })?;

        // Reserve a quota slot BEFORE touching the filesystem — a
        // failed read shouldn't consume quota.
        self.quota
            .try_consume()
            .map_err(|scope| ToolError::ImageCapExceeded {
                scope,
                limit: self.quota.limit_for(scope),
            })?;

        let path_buf = std::path::PathBuf::from(&input.path);
        let mime = detect_mime(&path_buf).ok_or_else(|| ToolError::InvalidArgs {
            tool: self.name.clone(),
            reason: format!(
                "unsupported image extension for {}: expect jpg/jpeg/png/webp/gif",
                input.path
            ),
        })?;

        // M-05: consult the sandbox BEFORE reading. Mirrors `net_fetch`'s
        // pattern — NoOpSandbox is a pass-through today; Phase 4's real
        // sandbox will enforce filesystem scoping here without further
        // changes to this tool. Release the quota slot on denial so a
        // blocked read does not silently consume the cap.
        if let Err(denial) = ctx.sandbox.check_fs_read(&path_buf).await {
            self.quota.release();
            return Err(ToolError::SandboxDenied {
                tool: self.name.clone(),
                reason: denial.reason,
            });
        }

        // R-2: METADATA-FIRST size gate. Call `tokio::fs::metadata`
        // BEFORE `tokio::fs::read` so a 20 GiB pathological payload
        // never hits `Vec<u8>`. The cap is INCLUSIVE: `len <= cap`
        // succeeds, `len > cap` rejects with `ImageTooLarge` and
        // rolls the quota reservation back — parity with the `Io`
        // and `SandboxDenied` arms so a failed call never counts
        // toward per-turn / per-session image caps. This is the
        // observable guarantee the R-2 regression suite locks:
        // over-cap calls must NOT emit `AgentEvent::ImageRead`
        // (raw bytes are never read), because the event is emitted
        // below from `bytes` which this gate has proven we never
        // allocate.
        let meta = match tokio::fs::metadata(&path_buf).await {
            Ok(m) => m,
            Err(e) => {
                self.quota.release();
                return Err(ToolError::Io(e));
            }
        };
        if meta.len() > self.max_image_bytes {
            self.quota.release();
            return Err(ToolError::ImageTooLarge {
                path: input.path.clone(),
                actual_size: meta.len(),
                cap: self.max_image_bytes,
            });
        }

        // M-02: release the quota slot if the FS read fails — otherwise a
        // prompt supplying 20 non-existent paths drains the per-session
        // cap without reading a byte (low-effort DoS against IMG-01).
        let bytes = match tokio::fs::read(&path_buf).await {
            Ok(b) => b,
            Err(e) => {
                self.quota.release();
                return Err(ToolError::Io(e));
            }
        };

        // Emit the ImageRead event BEFORE the base64 encoding so
        // downstream consumers receive the raw bytes. The returned
        // ToolOutput carries the data-URI form for the model to consume.
        (ctx.stream_sink)(AgentEvent::ImageRead { path: input.path.clone(), bytes: bytes.clone() });

        let encoded = BASE64.encode(&bytes);
        Ok(ToolOutput::text(format!("data:{mime};base64,{encoded}")))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn construct_produces_hardened_schema() {
        let t = ImageReadTool::new(Arc::new(crate::quota::ImageQuota::new(2, 20)));
        let schema = t.input_schema();
        let obj = schema.as_object().expect("object");
        assert_eq!(
            obj.get("additionalProperties"),
            Some(&serde_json::json!(false))
        );
        assert!(obj.get("required").is_some());
    }

    #[test]
    fn name_is_image_read() {
        let t = ImageReadTool::new(Arc::new(crate::quota::ImageQuota::new(2, 20)));
        assert_eq!(t.name().as_str(), "image_read");
    }

    #[test]
    fn detect_mime_covers_supported_formats() {
        assert_eq!(detect_mime(Path::new("/a.png")), Some("image/png"));
        assert_eq!(detect_mime(Path::new("/a.PNG")), Some("image/png"));
        assert_eq!(detect_mime(Path::new("/a.jpg")), Some("image/jpeg"));
        assert_eq!(detect_mime(Path::new("/a.jpeg")), Some("image/jpeg"));
        assert_eq!(detect_mime(Path::new("/a.webp")), Some("image/webp"));
        assert_eq!(detect_mime(Path::new("/a.gif")), Some("image/gif"));
        assert_eq!(detect_mime(Path::new("/a.bmp")), None);
        assert_eq!(detect_mime(Path::new("/noext")), None);
    }
}
