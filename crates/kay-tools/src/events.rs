//! Phase 3 AgentEvent (source of truth — moved from kay-provider-openrouter
//! per BRAINSTORM E1 so tool code can emit events without a cross-crate dep
//! cycle). The kay-provider-openrouter crate re-exports this type via
//! `pub use kay_tools::events::AgentEvent;` so existing Phase 2 call-sites
//! (e.g., `kay_provider_openrouter::event::AgentEvent`) continue to work
//! with NO behavioral change.
//!
//! **Additive extensions (D-08):** Phase 3 adds three variants —
//! `ToolOutput`, `TaskComplete`, `ImageRead`. Existing variants are
//! preserved byte-for-byte to maintain Phase 2 parity. The enum-level
//! `#[non_exhaustive]` annotation makes the additions safe under Rust's
//! exhaustiveness rules.

use serde_json::Value;

use kay_provider_errors::{ProviderError, RetryReason};

use crate::seams::verifier::VerificationOutcome;

/// Unified agent event stream. `#[non_exhaustive]` is load-bearing — Phase
/// 5 / 8 will add variants without breaking Phase 3 callers.
///
/// `Clone` / `Serialize` / `Deserialize` are intentionally NOT derived:
/// `ProviderError` contains `reqwest::Error` / `serde_json::Error`, neither
/// of which is Clone or Serialize. Auxiliary enums used in field positions
/// (`ToolOutputChunk`, `VerificationOutcome`) DO derive `Clone`.
#[non_exhaustive]
#[derive(Debug)]
pub enum AgentEvent {
    // -- Existing Phase 2 variants (UNCHANGED — D-08 parity guarantee) ----
    /// Streaming text chunk from the model's content channel.
    TextDelta { content: String },

    /// A tool call has begun; subsequent deltas carry arguments.
    ToolCallStart { id: String, name: String },

    /// Additional arguments bytes for an in-progress tool call. Empty/null
    /// argument deltas are legal per OpenRouter variance; the accumulator
    /// in plan 02-09 handles them defensively.
    ToolCallDelta { id: String, arguments_delta: String },

    /// Tool call fully assembled with valid JSON arguments. Tool-argument
    /// schema validation is the consumer's responsibility (Phase 3 TOOL-05).
    ToolCallComplete {
        id: String,
        name: String,
        arguments: Value,
    },

    /// Tool call assembled but arguments did not parse even after
    /// `forge_json_repair` fallback. `raw` carries the original bytes for
    /// diagnosis; consumers should surface this to the user, not execute.
    ToolCallMalformed {
        id: String,
        raw: String,
        error: String,
    },

    /// Usage/cost report emitted at turn end (per OpenRouter streaming docs,
    /// usage arrives on the final chunk). Fed into the cost-cap accumulator
    /// in plan 02-10.
    Usage {
        prompt_tokens: u64,
        completion_tokens: u64,
        cost_usd: f64,
    },

    /// A retryable upstream error is being retried after `delay_ms`. Emitted
    /// BEFORE the backoff sleep so UIs can show progress.
    Retry {
        attempt: u32,
        delay_ms: u64,
        reason: RetryReason,
    },

    /// Terminal, non-retryable error. The stream ends immediately after this.
    Error { error: ProviderError },

    // -- Phase 3 additive variants (D-08) --------------------------------
    /// Streamed output chunk from a running tool. Emitted during tool
    /// execution (typically `execute_commands`); one event per chunk plus
    /// a terminal `Closed` chunk. Phase 3 SHELL-03.
    ///
    /// `call_id` matches the `id` field from the preceding
    /// `AgentEvent::ToolCallComplete` so consumers can correlate output
    /// with the tool invocation.
    ToolOutput {
        call_id: String,
        chunk: ToolOutputChunk,
    },

    /// Terminal signal from the `task_complete` tool, carrying a
    /// verification outcome. Phase 3 TOOL-03 emits this with
    /// `verified: false` and `outcome: VerificationOutcome::Pending` via
    /// `NoOpVerifier`; Phase 8 swaps in a real verifier without changing
    /// the variant shape.
    TaskComplete {
        call_id: String,
        verified: bool,
        outcome: VerificationOutcome,
    },

    /// Raw image bytes read by an image-reading tool. Phase 3 IMG-01.
    /// `path` is the absolute path the tool resolved; `bytes` is the
    /// file contents. Consumers may choose to forward, truncate, or
    /// log-redact bytes in UI layers.
    ImageRead { path: String, bytes: Vec<u8> },

    /// A sandbox policy violation was detected for a tool call. Phase 4 SEC-01.
    ///
    /// **MUST NOT** be serialized back into the model's message history as a
    /// tool call result — doing so would teach the model to route around the
    /// policy (QG-C4 / prompt-injection risk). Route to the UI/user event
    /// stream only. Phase 5 planning constraint: enforce at the agent loop level.
    ///
    /// `policy_rule` MUST be one of the `RULE_*` constants from
    /// `kay-sandbox-policy` to ensure consistent cross-OS violation messages.
    SandboxViolation {
        call_id: String,
        tool_name: String,
        resource: String,
        /// Value MUST be a `kay_sandbox_policy::rules::RULE_*` constant.
        policy_rule: String,
        /// `None` = pre-flight userspace check; `Some(errno)` = kernel denial.
        os_error: Option<i32>,
    },

    // -- Phase 5 additive variants (DL-4 — agent-loop control signals) ------
    /// Emitted when the agent loop pauses (control channel receives
    /// `ControlMsg::Pause`). The loop buffers downstream events into a
    /// `VecDeque<AgentEvent>` until `ControlMsg::Resume` arrives, then
    /// replays them in order. UI consumers render this as a visible
    /// "paused" state.
    ///
    /// Intentionally a unit variant — the reason for pausing is always the
    /// same (user-initiated). Resume has no explicit event; the replayed
    /// events implicitly signal resume.
    Paused,
    /// Emitted when the agent loop terminates non-recoverably before
    /// natural turn-end. `reason` is one of the stable tag-strings:
    ///
    /// - `"user_ctrl_c"` — Ctrl-C cooperative abort (2s grace period)
    /// - `"max_turns_exceeded"` — turn budget hit (LOOP-04 safety net)
    /// - `"verifier_fail"` — `task_complete` returned `verified=false`
    /// - `"sandbox_violation_propagated"` — Sandbox denial threshold hit
    ///
    /// The wire form uses `reason` verbatim (no translation layer) so
    /// consumers can switch on it exhaustively. New reasons are an
    /// additive schema change (bump CONTRACT-AgentEvent.md + kay-cli
    /// `--version` per LOOP-02 drift policy).
    Aborted { reason: String },

    // -- Phase 7 additive variants (DL-12 — context engine signals) ----------
    /// Emitted by ContextBudget when context assembly drops symbols to fit
    /// the token budget. Consumers may surface this as a "context truncated"
    /// warning in the UI. `dropped_symbols` is the count of symbols that
    /// did not fit; `budget_tokens` is the configured available-token ceiling.
    ContextTruncated {
        dropped_symbols: usize,
        budget_tokens: usize,
    },

    /// Emitted by TreeSitterIndexer during incremental re-index. `indexed`
    /// is the count of files processed so far; `total` is the total file
    /// count in the watch scope. Consumers may use this to render a progress
    /// bar. Final emission has `indexed == total`.
    IndexProgress { indexed: usize, total: usize },
}

/// A single streamed output frame from a tool. Phase 3 SHELL-03.
///
/// `#[non_exhaustive]` on this enum is intentional: Phase 5 (agent loop)
/// and Phase 8 (UI hooks) may add variants such as resize hints or
/// structured progress markers without a breaking change.
///
/// `Clone` is safe because every payload is `String` or primitive.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ToolOutputChunk {
    /// One line (or buffered chunk) of stdout — newline preserved if the
    /// producer emits it, but not required. Consumers should not assume
    /// line boundaries.
    Stdout(String),

    /// One line (or buffered chunk) of stderr — same rules as `Stdout`.
    /// PTY-mode execution merges stderr into stdout and therefore only
    /// emits `Stdout` (see 03-RESEARCH.md §5).
    Stderr(String),

    /// The tool has finished producing output and (if applicable) the
    /// child process has exited. `exit_code` is `None` when no meaningful
    /// exit code is available (e.g., PTY dropped, parent killed the
    /// child). `marker_detected` indicates whether the KIRA
    /// `__CMDEND_<nonce>_<seq>__` sentinel was observed — `false` means
    /// the command terminated abnormally (crash, SIGKILL, orphan).
    Closed {
        exit_code: Option<i32>,
        marker_detected: bool,
    },
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod phase3_additions {
    use super::*;

    #[test]
    fn tool_output_variant_shape() {
        // U-15: full ToolOutput variant-shape lock.
        let ev = AgentEvent::ToolOutput {
            call_id: "call_123".to_string(),
            chunk: ToolOutputChunk::Stdout("hello\n".to_string()),
        };
        let dbg = format!("{ev:?}");
        assert!(dbg.contains("ToolOutput"), "missing ToolOutput: {dbg}");
        assert!(dbg.contains("call_123"), "missing call_id: {dbg}");
        assert!(dbg.contains("Stdout"), "missing Stdout: {dbg}");
        assert!(dbg.contains("hello"), "missing payload: {dbg}");

        let ev_err = AgentEvent::ToolOutput {
            call_id: "c".to_string(),
            chunk: ToolOutputChunk::Stderr("err".to_string()),
        };
        let dbg_err = format!("{ev_err:?}");
        assert!(dbg_err.contains("Stderr"), "missing Stderr: {dbg_err}");
        assert!(dbg_err.contains("err"), "missing stderr payload: {dbg_err}");

        let ev_closed = AgentEvent::ToolOutput {
            call_id: "c".to_string(),
            chunk: ToolOutputChunk::Closed { exit_code: Some(0), marker_detected: true },
        };
        let dbg_c = format!("{ev_closed:?}");
        assert!(dbg_c.contains("Closed"), "missing Closed: {dbg_c}");
        assert!(dbg_c.contains("Some(0)"), "missing exit code: {dbg_c}");
        assert!(
            dbg_c.contains("marker_detected: true"),
            "missing marker flag: {dbg_c}"
        );

        let ev_none = AgentEvent::ToolOutput {
            call_id: "c".to_string(),
            chunk: ToolOutputChunk::Closed { exit_code: None, marker_detected: false },
        };
        let dbg_n = format!("{ev_none:?}");
        assert!(
            dbg_n.contains("exit_code: None"),
            "missing None exit: {dbg_n}"
        );
        assert!(
            dbg_n.contains("marker_detected: false"),
            "missing false marker: {dbg_n}"
        );
    }

    #[test]
    fn task_complete_variant_shape() {
        // U-16: TaskComplete carries VerificationOutcome imported from
        // crate::seams::verifier — NOT redefined here.
        let ev = AgentEvent::TaskComplete {
            call_id: "c1".to_string(),
            verified: false,
            outcome: VerificationOutcome::Pending { reason: "wave 2 stub".to_string() },
        };
        if let AgentEvent::TaskComplete { verified, outcome, .. } = &ev {
            assert!(
                !*verified,
                "Phase 3 NoOpVerifier never reports verified=true"
            );
            assert!(
                matches!(outcome, VerificationOutcome::Pending { .. }),
                "expected Pending outcome, got {outcome:?}"
            );
        } else {
            panic!("not a TaskComplete: {ev:?}");
        }
    }

    #[test]
    fn image_read_variant_shape() {
        // U-17: ImageRead carries path + raw bytes.
        let ev = AgentEvent::ImageRead {
            path: "/tmp/x.png".to_string(),
            bytes: vec![0x89, 0x50, 0x4e, 0x47],
        };
        if let AgentEvent::ImageRead { path, bytes } = &ev {
            assert_eq!(path, "/tmp/x.png");
            assert_eq!(bytes.len(), 4);
            assert_eq!(bytes[0], 0x89);
        } else {
            panic!("not an ImageRead: {ev:?}");
        }
    }

    #[test]
    fn sandbox_violation_variant_shape() {
        // U-37: SandboxViolation has all 5 required fields (QG-C3/QG-C4).
        let ev = AgentEvent::SandboxViolation {
            call_id: "cid-42".to_string(),
            tool_name: "fs_write".to_string(),
            resource: "/etc/passwd".to_string(),
            policy_rule: "write-outside-project-root".to_string(),
            os_error: Some(13),
        };
        let dbg = format!("{ev:?}");
        assert!(dbg.contains("SandboxViolation"), "missing variant: {dbg}");
        assert!(dbg.contains("cid-42"), "missing call_id: {dbg}");
        assert!(dbg.contains("fs_write"), "missing tool_name: {dbg}");
        assert!(dbg.contains("/etc/passwd"), "missing resource: {dbg}");
        assert!(
            dbg.contains("write-outside-project-root"),
            "missing policy_rule: {dbg}"
        );
        assert!(dbg.contains("13"), "missing os_error: {dbg}");
    }

    #[test]
    fn sandbox_violation_preflight_has_no_os_error() {
        // U-38: pre-flight violations have os_error = None.
        let ev = AgentEvent::SandboxViolation {
            call_id: "cid-43".to_string(),
            tool_name: "net_fetch".to_string(),
            resource: "http://evil.com".to_string(),
            policy_rule: "net-not-allowlisted".to_string(),
            os_error: None,
        };
        if let AgentEvent::SandboxViolation { os_error, .. } = &ev {
            assert!(os_error.is_none(), "preflight must have no OS error");
        } else {
            panic!("not a SandboxViolation: {ev:?}");
        }
    }

    #[test]
    fn emits_in_order() {
        // U-18: the canonical Phase 3 emission sequence.
        let seq = [
            AgentEvent::ToolOutput {
                call_id: "c".into(),
                chunk: ToolOutputChunk::Stdout("one".into()),
            },
            AgentEvent::ToolOutput {
                call_id: "c".into(),
                chunk: ToolOutputChunk::Closed { exit_code: Some(0), marker_detected: true },
            },
            AgentEvent::TaskComplete {
                call_id: "c".into(),
                verified: false,
                outcome: VerificationOutcome::Pending { reason: "p".into() },
            },
            AgentEvent::ImageRead { path: "/tmp/i.png".into(), bytes: vec![0xff] },
        ];
        assert!(matches!(&seq[0], AgentEvent::ToolOutput { .. }));
        assert!(matches!(
            &seq[1],
            AgentEvent::ToolOutput { chunk: ToolOutputChunk::Closed { .. }, .. }
        ));
        assert!(matches!(&seq[2], AgentEvent::TaskComplete { .. }));
        assert!(matches!(&seq[3], AgentEvent::ImageRead { .. }));
    }

    #[test]
    fn tool_output_chunk_is_clone() {
        let original = ToolOutputChunk::Stdout("x".to_string());
        let cloned = original.clone();
        assert_eq!(format!("{original:?}"), format!("{cloned:?}"));
    }
}
