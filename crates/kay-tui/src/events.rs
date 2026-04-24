// Kay TUI event types — mirrors kay-tauri's IpcAgentEvent for the CLI JSONL contract.
//
// WIRE FORMAT SYNC: This type must remain structurally identical to
// kay-tauri/src/ipc_event.rs::IpcAgentEvent. Any additive change to the
// event wire format must be mirrored in both files. See Phase 9.5 spec §3.
// serde rename attrs ensure wire compatibility (same JSON field names).

use serde::{Deserialize, Serialize};

// TuiEvent: IPC-safe mirror of IpcAgentEvent. All fields are JSON-serializable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum TuiEvent {
    // Phase 2 variants
    #[serde(rename = "TextDelta")]
    TextDelta { content: String },
    #[serde(rename = "ToolCallStart")]
    ToolCallStart { id: String, name: String },
    #[serde(rename = "ToolCallDelta")]
    ToolCallDelta { id: String, arguments_delta: String },
    #[serde(rename = "ToolCallComplete")]
    ToolCallComplete {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
    #[serde(rename = "ToolCallMalformed")]
    ToolCallMalformed {
        id: String,
        raw: String,
        error: String,
    },
    #[serde(rename = "Usage")]
    Usage {
        prompt_tokens: u64,
        completion_tokens: u64,
        cost_usd: f64,
    },
    #[serde(rename = "Retry")]
    Retry {
        attempt: u32,
        delay_ms: u64,
        reason: String,
    },
    #[serde(rename = "Error")]
    Error { message: String },

    // Phase 3 variants
    #[serde(rename = "ToolOutput")]
    ToolOutput {
        call_id: String,
        chunk: TuiToolOutputChunk,
    },
    #[serde(rename = "TaskComplete")]
    TaskComplete {
        call_id: String,
        verified: bool,
        outcome: TuiVerificationOutcome,
    },
    #[serde(rename = "ImageRead")]
    ImageRead { path: String, data_url: String },
    #[serde(rename = "SandboxViolation")]
    SandboxViolation {
        call_id: String,
        tool_name: String,
        resource: String,
        policy_rule: String,
        os_error: Option<i32>,
    },

    // Phase 5 variants
    #[serde(rename = "Paused")]
    Paused,
    #[serde(rename = "Aborted")]
    Aborted { reason: String },

    // Phase 7 variants
    #[serde(rename = "ContextTruncated")]
    ContextTruncated {
        dropped_symbols: usize,
        budget_tokens: usize,
    },
    #[serde(rename = "IndexProgress")]
    IndexProgress { indexed: usize, total: usize },

    // Phase 8 variants
    #[serde(rename = "Verification")]
    Verification {
        critic_role: String,
        verdict: String,
        reason: String,
        cost_usd: f64,
    },
    #[serde(rename = "VerifierDisabled")]
    VerifierDisabled { reason: String, cost_usd: f64 },
    // Catch-all: future variants are handled at the JSONL parsing layer
    // (jsonl.rs WAVE 2), not in serde. serde with #[serde(tag)] does not
    // support a catch-all variant — it fails on unknown tags rather than
    // falling back. See: docs/superpowers/specs/2026-04-24-phase9.5-tui-frontend-design.md §3.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TuiToolOutputChunk {
    #[serde(rename = "Stdout")]
    Stdout(String),
    #[serde(rename = "Stderr")]
    Stderr(String),
    #[serde(rename = "Closed")]
    Closed {
        exit_code: Option<i32>,
        marker_detected: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TuiVerificationOutcome {
    #[serde(rename = "Pending")]
    Pending { reason: String },
    #[serde(rename = "Pass")]
    Pass { note: String },
    #[serde(rename = "Fail")]
    Fail { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── TuiEvent round-trip serialization ────────────────────────────────────

    #[test]
    fn text_delta_deserializes() {
        let json = r#"{"type":"TextDelta","data":{"content":"hello world"}}"#;
        let event = serde_json::from_str::<TuiEvent>(json).unwrap();
        assert!(matches!(event, TuiEvent::TextDelta { content } if content == "hello world"));
    }

    #[test]
    fn usage_deserializes() {
        let json = r#"{"type":"Usage","data":{"prompt_tokens":100,"completion_tokens":50,"cost_usd":0.0021}}"#;
        let event = serde_json::from_str::<TuiEvent>(json).unwrap();
        match event {
            TuiEvent::Usage { prompt_tokens, completion_tokens, cost_usd } => {
                assert_eq!(prompt_tokens, 100);
                assert_eq!(completion_tokens, 50);
                assert!((cost_usd - 0.0021).abs() < 1e-6);
            }
            other => panic!("expected Usage, got {other:?}"),
        }
    }

    #[test]
    fn tool_call_complete_round_trips() {
        let json =
            r#"{"type":"ToolCallComplete","data":{"id":"abc","name":"edit_file","arguments":{}}}"#;
        let event = serde_json::from_str::<TuiEvent>(json).unwrap();
        assert!(
            matches!(event, TuiEvent::ToolCallComplete { ref id, ref name, .. } if id == "abc" && name == "edit_file")
        );
        let round_trip = serde_json::to_string(&event).unwrap();
        assert!(round_trip.contains(r#""id":"abc""#));
    }

    #[test]
    fn error_deserializes() {
        let json = r#"{"type":"Error","data":{"message":"connection reset"}}"#;
        let event = serde_json::from_str::<TuiEvent>(json).unwrap();
        match event {
            TuiEvent::Error { message } => assert_eq!(message, "connection reset"),
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn retry_round_trips() {
        let json =
            r#"{"type":"Retry","data":{"attempt":2,"delay_ms":1000,"reason":"RateLimited"}}"#;
        let event = serde_json::from_str::<TuiEvent>(json).unwrap();
        match event {
            TuiEvent::Retry { attempt, delay_ms, reason } => {
                assert_eq!(attempt, 2);
                assert_eq!(delay_ms, 1000);
                assert_eq!(reason, "RateLimited");
            }
            other => panic!("expected Retry, got {other:?}"),
        }
    }

    #[test]
    fn tool_output_chunk_closed_round_trips() {
        let json = r#"{"type":"ToolOutput","data":{"call_id":"xyz","chunk":{"Closed":{"exit_code":0,"marker_detected":true}}}}"#;
        let event = serde_json::from_str::<TuiEvent>(json).unwrap();
        match event {
            TuiEvent::ToolOutput { call_id, chunk } => {
                assert_eq!(call_id, "xyz");
                match chunk {
                    TuiToolOutputChunk::Closed { exit_code, marker_detected } => {
                        assert_eq!(exit_code, Some(0));
                        assert!(marker_detected);
                    }
                    other => panic!("expected Closed, got {other:?}"),
                }
            }
            other => panic!("expected ToolOutput, got {other:?}"),
        }
    }

    #[test]
    fn verification_outcome_pass_round_trips() {
        let json = r#"{"type":"TaskComplete","data":{"call_id":"t1","verified":true,"outcome":{"Pass":{"note":"looks good"}}}}"#;
        let event = serde_json::from_str::<TuiEvent>(json).unwrap();
        match event {
            TuiEvent::TaskComplete { verified, outcome, .. } => {
                assert!(verified);
                match outcome {
                    TuiVerificationOutcome::Pass { note } => assert_eq!(note, "looks good"),
                    other => panic!("expected Pass, got {other:?}"),
                }
            }
            other => panic!("expected TaskComplete, got {other:?}"),
        }
    }

    #[test]
    fn all_known_variants_deserialize() {
        // Verify each known variant parses without error.
        // Unknown-type events (future wire additions) are handled by the
        // JsonlParser in jsonl.rs — not by serde deserialization here.
        let variants = [
            r#"{"type":"TextDelta","data":{"content":"x"}}"#,
            r#"{"type":"ToolCallStart","data":{"id":"x","name":"y"}}"#,
            r#"{"type":"ToolCallDelta","data":{"id":"x","arguments_delta":"y"}}"#,
            r#"{"type":"ToolCallComplete","data":{"id":"x","name":"y","arguments":{}}}"#,
            r#"{"type":"ToolCallMalformed","data":{"id":"x","raw":"y","error":"z"}}"#,
            r#"{"type":"Usage","data":{"prompt_tokens":1,"completion_tokens":1,"cost_usd":0}}"#,
            r#"{"type":"Retry","data":{"attempt":1,"delay_ms":1,"reason":"x"}}"#,
            r#"{"type":"Error","data":{"message":"x"}}"#,
            r#"{"type":"ToolOutput","data":{"call_id":"x","chunk":{"Stdout":"y"}}}"#,
            r#"{"type":"TaskComplete","data":{"call_id":"x","verified":true,"outcome":{"Pass":{"note":""}}}}"#,
            r#"{"type":"ImageRead","data":{"path":"x","data_url":"y"}}"#,
            r#"{"type":"SandboxViolation","data":{"call_id":"x","tool_name":"y","resource":"z","policy_rule":"w","os_error":null}}"#,
            r#"{"type":"Paused","data":null}"#,
            r#"{"type":"Aborted","data":{"reason":"x"}}"#,
            r#"{"type":"ContextTruncated","data":{"dropped_symbols":1,"budget_tokens":100}}"#,
            r#"{"type":"IndexProgress","data":{"indexed":1,"total":10}}"#,
            r#"{"type":"Verification","data":{"critic_role":"test-engineer","verdict":"pass","reason":"ok","cost_usd":0}}"#,
            r#"{"type":"VerifierDisabled","data":{"reason":"no budget","cost_usd":0}}"#,
        ];
        assert_eq!(variants.len(), 18, "all 18 known variants must be tested");
        for (i, variant_json) in variants.iter().enumerate() {
            let result = serde_json::from_str::<TuiEvent>(variant_json);
            assert!(
                result.is_ok(),
                "variant {i} failed to parse: {variant_json}"
            );
        }
    }

    #[test]
    fn unknown_event_type_is_rejected() {
        // serde with #[serde(tag = "type")] does not support catch-all variants.
        // Unknown wire types are handled at the JsonlParser level (WAVE 2),
        // not in serde deserialization. This test documents the expected
        // behavior: unknown type tags cause a parse error here.
        let json = r#"{"type":"SomeNewFutureEvent","data":{"foo":123}}"#;
        let result = serde_json::from_str::<TuiEvent>(json);
        assert!(
            result.is_err(),
            "unknown type tag should fail serde (handled at JsonlParser level)"
        );
    }
}
