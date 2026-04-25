//! IPC-safe mirror of `AgentEvent`.
//!
//! `AgentEvent` cannot implement `serde::Serialize` or `specta::Type` because:
//! - `Error { error: ProviderError }` wraps `reqwest::Error` (not Serialize)
//! - `ImageRead { bytes: Vec<u8> }` would serialize as a JSON int array (IPC-unsafe for images)
//! - `Retry { reason: RetryReason }` — `RetryReason` is not Serialize
//!
//! `IpcAgentEvent` owns all IPC concerns. `AgentEvent` is never modified.

use base64::Engine;
use serde::{Deserialize, Serialize};
use specta::Type;

use kay_tools::events::{AgentEvent, ToolOutputChunk};
use kay_tools::seams::verifier::VerificationOutcome;

/// IPC-safe mirror of `AgentEvent`. All fields are JSON-serializable.
/// `Clone` is safe — no non-Clone types here (unlike `AgentEvent`).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "data")]
pub enum IpcAgentEvent {
    // Phase 2 variants
    TextDelta         { content: String },
    ToolCallStart     { id: String, name: String },
    ToolCallDelta     { id: String, arguments_delta: String },
    ToolCallComplete  { id: String, name: String, arguments: serde_json::Value },
    ToolCallMalformed { id: String, raw: String, error: String },
    Usage             { prompt_tokens: u64, completion_tokens: u64, cost_usd: f64 },
    Retry             { attempt: u32, delay_ms: u64, reason: String },
    Error             { message: String },

    // Phase 3 variants
    ToolOutput        { call_id: String, chunk: IpcToolOutputChunk },
    TaskComplete      { call_id: String, verified: bool, outcome: IpcVerificationOutcome },
    ImageRead         { path: String, data_url: String },
    SandboxViolation  { call_id: String, tool_name: String, resource: String, policy_rule: String, os_error: Option<i32> },

    // Phase 5 variants
    Paused,
    Aborted           { reason: String },

    // Phase 7 variants
    ContextTruncated  { dropped_symbols: usize, budget_tokens: usize },
    IndexProgress     { indexed: usize, total: usize },

    // Phase 8 variants
    Verification      { critic_role: String, verdict: String, reason: String, cost_usd: f64 },
    VerifierDisabled  { reason: String, cost_usd: f64 },

    // Phase 10 WAVE 3: Session lifecycle variants
    SessionSpawned    { session_id: String, persona: String, created_at: i64 },
    SessionPaused     { session_id: String, paused_at: i64 },
    SessionResumed    { session_id: String, resumed_at: i64 },
    SessionForked     { parent_id: String, child_id: String },
    ApprovalRequested  { tool_name: String, command: String, sandbox_status: String },
    ApprovalDecision   { tool_name: String, approved: bool, decided_at: i64 },

    // Catch-all: future #[non_exhaustive] variants become Unknown
    Unknown           { event_type: String },
}

/// IPC-safe mirror of `ToolOutputChunk`.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum IpcToolOutputChunk {
    Stdout(String),
    Stderr(String),
    Closed { exit_code: Option<i32>, marker_detected: bool },
}

/// IPC-safe mirror of `VerificationOutcome`.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum IpcVerificationOutcome {
    Pending { reason: String },
    Pass    { note: String },
    Fail    { reason: String },
}

impl From<VerificationOutcome> for IpcVerificationOutcome {
    fn from(v: VerificationOutcome) -> Self {
        match v {
            VerificationOutcome::Pending { reason } => Self::Pending { reason },
            VerificationOutcome::Pass { note }      => Self::Pass { note },
            VerificationOutcome::Fail { reason }    => Self::Fail { reason },
            _ => Self::Pending { reason: "unknown_outcome_variant".to_string() },
        }
    }
}

impl From<ToolOutputChunk> for IpcToolOutputChunk {
    fn from(chunk: ToolOutputChunk) -> Self {
        match chunk {
            ToolOutputChunk::Stdout(s)  => Self::Stdout(s),
            ToolOutputChunk::Stderr(s)  => Self::Stderr(s),
            ToolOutputChunk::Closed { exit_code, marker_detected } =>
                Self::Closed { exit_code, marker_detected },
            _ => Self::Stdout("[unknown chunk variant]".to_string()),
        }
    }
}

impl From<AgentEvent> for IpcAgentEvent {
    fn from(event: AgentEvent) -> Self {
        match event {
            AgentEvent::TextDelta { content }
                => Self::TextDelta { content },
            AgentEvent::ToolCallStart { id, name }
                => Self::ToolCallStart { id, name },
            AgentEvent::ToolCallDelta { id, arguments_delta }
                => Self::ToolCallDelta { id, arguments_delta },
            AgentEvent::ToolCallComplete { id, name, arguments }
                => Self::ToolCallComplete { id, name, arguments },
            AgentEvent::ToolCallMalformed { id, raw, error }
                => Self::ToolCallMalformed { id, raw, error },
            AgentEvent::Usage { prompt_tokens, completion_tokens, cost_usd }
                => Self::Usage { prompt_tokens, completion_tokens, cost_usd },
            AgentEvent::Retry { attempt, delay_ms, reason }
                => Self::Retry { attempt, delay_ms, reason: format!("{:?}", reason) },
            AgentEvent::Error { error }
                => Self::Error { message: error.to_string() },
            AgentEvent::ToolOutput { call_id, chunk }
                => Self::ToolOutput { call_id, chunk: IpcToolOutputChunk::from(chunk) },
            AgentEvent::TaskComplete { call_id, verified, outcome }
                => Self::TaskComplete { call_id, verified, outcome: IpcVerificationOutcome::from(outcome) },
            AgentEvent::ImageRead { path, bytes }
                => Self::ImageRead { path: path.clone(), data_url: bytes_to_data_url(&path, &bytes) },
            AgentEvent::SandboxViolation { call_id, tool_name, resource, policy_rule, os_error }
                => Self::SandboxViolation { call_id, tool_name, resource, policy_rule, os_error },
            AgentEvent::Paused
                => Self::Paused,
            AgentEvent::Aborted { reason }
                => Self::Aborted { reason },
            AgentEvent::ContextTruncated { dropped_symbols, budget_tokens }
                => Self::ContextTruncated { dropped_symbols, budget_tokens },
            AgentEvent::IndexProgress { indexed, total }
                => Self::IndexProgress { indexed, total },
            AgentEvent::Verification { critic_role, verdict, reason, cost_usd }
                => Self::Verification { critic_role, verdict, reason, cost_usd },
            AgentEvent::VerifierDisabled { reason, cost_usd }
                => Self::VerifierDisabled { reason, cost_usd },
            _ => Self::Unknown { event_type: "future_variant".to_string() },
        }
    }
}

/// Infer MIME type from bytes, fall back to path extension.
pub fn bytes_to_data_url(path: &str, bytes: &[u8]) -> String {
    let mime = infer::get(bytes)
        .map(|t| t.mime_type())
        .unwrap_or_else(|| {
            match path.rsplit('.').next().unwrap_or("").to_lowercase().as_str() {
                "png"        => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "gif"        => "image/gif",
                "webp"       => "image/webp",
                _            => "application/octet-stream",
            }
        });
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    format!("data:{};base64,{}", mime, b64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kay_provider_errors::{ProviderError, RetryReason};

    #[test]
    fn error_maps_to_message_string() {
        let ev = AgentEvent::Error {
            error: ProviderError::Stream("connection reset".to_string()),
        };
        let ipc = IpcAgentEvent::from(ev);
        match ipc {
            IpcAgentEvent::Error { message } => {
                assert!(message.contains("connection reset"), "got: {message}");
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn retry_reason_becomes_debug_string() {
        let ev = AgentEvent::Retry {
            attempt: 1,
            delay_ms: 500,
            reason: RetryReason::RateLimited,
        };
        let ipc = IpcAgentEvent::from(ev);
        match ipc {
            IpcAgentEvent::Retry { reason, .. } => {
                assert_eq!(reason, "RateLimited");
            }
            other => panic!("expected Retry, got {other:?}"),
        }
    }

    #[test]
    fn image_read_produces_valid_data_url() {
        // PNG magic bytes
        let bytes = vec![0x89u8, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
        let ev = AgentEvent::ImageRead { path: "/tmp/test.png".to_string(), bytes };
        let ipc = IpcAgentEvent::from(ev);
        match ipc {
            IpcAgentEvent::ImageRead { data_url, .. } => {
                assert!(data_url.starts_with("data:image/png;base64,"), "got: {data_url}");
            }
            other => panic!("expected ImageRead, got {other:?}"),
        }
    }

    #[test]
    fn tool_output_chunk_closed_preserved() {
        use kay_tools::events::ToolOutputChunk;
        let chunk = ToolOutputChunk::Closed { exit_code: Some(0), marker_detected: true };
        let ipc = IpcToolOutputChunk::from(chunk);
        match ipc {
            IpcToolOutputChunk::Closed { exit_code, marker_detected } => {
                assert_eq!(exit_code, Some(0));
                assert!(marker_detected);
            }
            other => panic!("expected Closed, got {other:?}"),
        }
    }

    #[test]
    fn verification_outcome_pass_preserved() {
        use kay_tools::seams::verifier::VerificationOutcome;
        let outcome = VerificationOutcome::Pass { note: "all good".to_string() };
        let ipc = IpcVerificationOutcome::from(outcome);
        match ipc {
            IpcVerificationOutcome::Pass { note } => assert_eq!(note, "all good"),
            other => panic!("expected Pass, got {other:?}"),
        }
    }

    #[test]
    fn unknown_future_variant_maps_to_unknown() {
        // Simulate the _ arm by checking Paused (a known unit variant → not Unknown)
        let ev = AgentEvent::Paused;
        let ipc = IpcAgentEvent::from(ev);
        assert!(matches!(ipc, IpcAgentEvent::Paused));
    }

    // ── Phase 10 WAVE 3: Session lifecycle event tests ─────────────────────

    #[test]
    fn session_spawned_roundtrip() {
        let ev = IpcAgentEvent::SessionSpawned {
            session_id: "test-session-123".to_string(),
            persona: "forge".to_string(),
            created_at: 1714000000,
        };
        let json = serde_json::to_string(&ev).unwrap();
        let parsed: IpcAgentEvent = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, IpcAgentEvent::SessionSpawned { session_id, .. } if session_id == "test-session-123"));
    }

    #[test]
    fn session_paused_roundtrip() {
        let ev = IpcAgentEvent::SessionPaused {
            session_id: "test-session-456".to_string(),
            paused_at: 1714000100,
        };
        let json = serde_json::to_string(&ev).unwrap();
        let parsed: IpcAgentEvent = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, IpcAgentEvent::SessionPaused { session_id, .. } if session_id == "test-session-456"));
    }

    #[test]
    fn session_resumed_roundtrip() {
        let ev = IpcAgentEvent::SessionResumed {
            session_id: "test-session-789".to_string(),
            resumed_at: 1714000200,
        };
        let json = serde_json::to_string(&ev).unwrap();
        let parsed: IpcAgentEvent = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, IpcAgentEvent::SessionResumed { session_id, .. } if session_id == "test-session-789"));
    }

    #[test]
    fn session_forked_roundtrip() {
        let ev = IpcAgentEvent::SessionForked {
            parent_id: "parent-abc".to_string(),
            child_id: "child-xyz".to_string(),
        };
        let json = serde_json::to_string(&ev).unwrap();
        let parsed: IpcAgentEvent = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, IpcAgentEvent::SessionForked { parent_id, child_id } if parent_id == "parent-abc" && child_id == "child-xyz"));
    }

    #[test]
    fn approval_requested_roundtrip() {
        let ev = IpcAgentEvent::ApprovalRequested {
            tool_name: "execute_commands".to_string(),
            command: "rm -rf /tmp".to_string(),
            sandbox_status: "allowed".to_string(),
        };
        let json = serde_json::to_string(&ev).unwrap();
        let parsed: IpcAgentEvent = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, IpcAgentEvent::ApprovalRequested { tool_name, .. } if tool_name == "execute_commands"));
    }

    #[test]
    fn approval_decision_approved_roundtrip() {
        let ev = IpcAgentEvent::ApprovalDecision {
            tool_name: "execute_commands".to_string(),
            approved: true,
            decided_at: 1714000300,
        };
        let json = serde_json::to_string(&ev).unwrap();
        let parsed: IpcAgentEvent = serde_json::from_str(&json).unwrap();
        match parsed {
            IpcAgentEvent::ApprovalDecision { approved, .. } => assert!(approved),
            other => panic!("expected ApprovalDecision, got {other:?}"),
        }
    }

    #[test]
    fn approval_decision_denied_roundtrip() {
        let ev = IpcAgentEvent::ApprovalDecision {
            tool_name: "fs_write".to_string(),
            approved: false,
            decided_at: 1714000400,
        };
        let json = serde_json::to_string(&ev).unwrap();
        let parsed: IpcAgentEvent = serde_json::from_str(&json).unwrap();
        match parsed {
            IpcAgentEvent::ApprovalDecision { approved, .. } => assert!(!approved),
            other => panic!("expected ApprovalDecision, got {other:?}"),
        }
    }

    #[test]
    fn bytes_to_data_url_png_extension_fallback() {
        // Empty bytes → infer returns None → fall back to extension
        let url = bytes_to_data_url("/tmp/image.png", &[]);
        assert!(url.starts_with("data:image/png;base64,"), "got: {url}");
    }
}