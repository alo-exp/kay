// Kay TUI event types — mirrors kay-tauri's IpcAgentEvent for the CLI JSONL contract.
//
// WIRE FORMAT SYNC: This type must remain structurally identical to
// kay-tauri/src/ipc_event.rs::IpcAgentEvent. Any additive change to the
// event wire format must be mirrored in both files. See Phase 9.5 spec §3.
// serde rename attrs ensure wire compatibility (same JSON field names).

use serde::de::{self, Visitor, Deserializer};
use serde::{Deserialize, Serialize};
use std::fmt;

// TuiEvent: IPC-safe mirror of IpcAgentEvent. All fields are JSON-serializable.
// Uses a custom deserializer to route unknown "type" values to an Unknown variant.
// serde's built-in #[serde(tag = "type")] does not support catch-all — we implement
// it manually via a custom Deserialize impl.
#[derive(Debug, Clone, Serialize)]
// NOTE: Deserialize is NOT derived — TuiEvent uses a custom Deserialize impl
// below that routes unknown "type" values to the Unknown catch-all variant.
// serde's #[serde(tag = "type")] does not support catch-all variants,
// so we implement Deserialize manually.
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
    // Catch-all: future variants deserialize here. Custom Deserialize impl
    // routes unknown "type" values here rather than failing.
    #[serde(rename = "Unknown")]
    Unknown { event_type: String },
}

// ─── Custom Deserialize impl for TuiEvent ──────────────────────────────
// serde's #[serde(tag = "type")] does not support catch-all variants.
// We implement Deserialize manually to route unknown type tags to the
// Unknown variant instead of failing.

impl<'de> Deserialize<'de> for TuiEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TuiEventVisitor;

        impl<'de> Visitor<'de> for TuiEventVisitor {
            type Value = TuiEvent;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a TuiEvent JSON object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                // Collect all key-value pairs into a serde_json::Map.
                // This avoids the type conflict when using next_value::<Value>() directly.
                let mut fields = serde_json::Map::new();
                while let Some(key) = map.next_key()? {
                    let value: serde_json::Value = map.next_value()?;
                    fields.insert(key, value);
                }

                let type_field = fields
                    .get("type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| de::Error::missing_field("type"))?;

                let data = fields.get("data").cloned().unwrap_or(serde_json::Value::Null);

                // Route to the correct variant based on type tag
                let type_str: &str = type_field;
                match type_str {
                    "TextDelta" => {
                        let content = data.get("content")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing content field"))?;
                        Ok(TuiEvent::TextDelta { content: content.to_string() })
                    }
                    "ToolCallStart" => {
                        let id = data.get("id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing id field"))?;
                        let name = data.get("name")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing name field"))?;
                        Ok(TuiEvent::ToolCallStart {
                            id: id.to_string(),
                            name: name.to_string(),
                        })
                    }
                    "ToolCallDelta" => {
                        let id = data.get("id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing id field"))?;
                        let arguments_delta = data.get("arguments_delta")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing arguments_delta field"))?;
                        Ok(TuiEvent::ToolCallDelta {
                            id: id.to_string(),
                            arguments_delta: arguments_delta.to_string(),
                        })
                    }
                    "ToolCallComplete" => {
                        let id = data.get("id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing id field"))?;
                        let name = data.get("name")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing name field"))?;
                        Ok(TuiEvent::ToolCallComplete {
                            id: id.to_string(),
                            name: name.to_string(),
                            arguments: data.clone(),
                        })
                    }
                    "ToolCallMalformed" => {
                        let id = data.get("id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing id field"))?;
                        let raw = data.get("raw")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing raw field"))?;
                        let error = data.get("error")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing error field"))?;
                        Ok(TuiEvent::ToolCallMalformed {
                            id: id.to_string(),
                            raw: raw.to_string(),
                            error: error.to_string(),
                        })
                    }
                    "Usage" => {
                        let prompt_tokens = data.get("prompt_tokens")
                            .and_then(|v| v.as_u64())
                            .ok_or_else(|| de::Error::custom("missing prompt_tokens"))?;
                        let completion_tokens = data.get("completion_tokens")
                            .and_then(|v| v.as_u64())
                            .ok_or_else(|| de::Error::custom("missing completion_tokens"))?;
                        let cost_usd = data.get("cost_usd")
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| de::Error::custom("missing cost_usd"))?;
                        Ok(TuiEvent::Usage {
                            prompt_tokens,
                            completion_tokens,
                            cost_usd,
                        })
                    }
                    "Retry" => {
                        let attempt = data.get("attempt")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as u32)
                            .ok_or_else(|| de::Error::custom("missing attempt"))?;
                        let delay_ms = data.get("delay_ms")
                            .and_then(|v| v.as_u64())
                            .ok_or_else(|| de::Error::custom("missing delay_ms"))?;
                        let reason = data.get("reason")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing reason"))?;
                        Ok(TuiEvent::Retry {
                            attempt,
                            delay_ms,
                            reason: reason.to_string(),
                        })
                    }
                    "Error" => {
                        let message = data.get("message")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing message field"))?;
                        Ok(TuiEvent::Error { message: message.to_string() })
                    }
                    "ToolOutput" => {
                        let call_id = data.get("call_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing call_id"))?;
                        let chunk: TuiToolOutputChunk = serde_json::from_value(data.get("chunk").cloned().unwrap_or(serde_json::Value::Null))
                            .map_err(|e| de::Error::custom(e))?;
                        Ok(TuiEvent::ToolOutput {
                            call_id: call_id.to_string(),
                            chunk,
                        })
                    }
                    "TaskComplete" => {
                        let call_id = data.get("call_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing call_id"))?;
                        let verified = data.get("verified")
                            .and_then(|v| v.as_bool())
                            .ok_or_else(|| de::Error::custom("missing verified"))?;
                        let outcome: TuiVerificationOutcome = serde_json::from_value(data.get("outcome").cloned().unwrap_or(serde_json::Value::Null))
                            .map_err(|e| de::Error::custom(e))?;
                        Ok(TuiEvent::TaskComplete {
                            call_id: call_id.to_string(),
                            verified,
                            outcome,
                        })
                    }
                    "ImageRead" => {
                        let path = data.get("path")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing path"))?;
                        let data_url = data.get("data_url")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing data_url"))?;
                        Ok(TuiEvent::ImageRead {
                            path: path.to_string(),
                            data_url: data_url.to_string(),
                        })
                    }
                    "SandboxViolation" => {
                        let call_id = data.get("call_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing call_id"))?;
                        let tool_name = data.get("tool_name")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing tool_name"))?;
                        let resource = data.get("resource")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing resource"))?;
                        let policy_rule = data.get("policy_rule")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing policy_rule"))?;
                        let os_error = data.get("os_error")
                            .and_then(|v| v.as_i64())
                            .map(|v| v as i32);
                        Ok(TuiEvent::SandboxViolation {
                            call_id: call_id.to_string(),
                            tool_name: tool_name.to_string(),
                            resource: resource.to_string(),
                            policy_rule: policy_rule.to_string(),
                            os_error,
                        })
                    }
                    "Paused" => Ok(TuiEvent::Paused),
                    "Aborted" => {
                        let reason = data.get("reason")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing reason field"))?;
                        Ok(TuiEvent::Aborted { reason: reason.to_string() })
                    }
                    "ContextTruncated" => {
                        let dropped_symbols = data.get("dropped_symbols")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as usize)
                            .ok_or_else(|| de::Error::custom("missing dropped_symbols"))?;
                        let budget_tokens = data.get("budget_tokens")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as usize)
                            .ok_or_else(|| de::Error::custom("missing budget_tokens"))?;
                        Ok(TuiEvent::ContextTruncated {
                            dropped_symbols,
                            budget_tokens,
                        })
                    }
                    "IndexProgress" => {
                        let indexed = data.get("indexed")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as usize)
                            .ok_or_else(|| de::Error::custom("missing indexed"))?;
                        let total = data.get("total")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as usize)
                            .ok_or_else(|| de::Error::custom("missing total"))?;
                        Ok(TuiEvent::IndexProgress {
                            indexed,
                            total,
                        })
                    }
                    "Verification" => {
                        let critic_role = data.get("critic_role")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing critic_role"))?;
                        let verdict = data.get("verdict")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing verdict"))?;
                        let reason = data.get("reason")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing reason"))?;
                        let cost_usd = data.get("cost_usd")
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| de::Error::custom("missing cost_usd"))?;
                        Ok(TuiEvent::Verification {
                            critic_role: critic_role.to_string(),
                            verdict: verdict.to_string(),
                            reason: reason.to_string(),
                            cost_usd,
                        })
                    }
                    "VerifierDisabled" => {
                        let reason = data.get("reason")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| de::Error::custom("missing reason"))?;
                        let cost_usd = data.get("cost_usd")
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| de::Error::custom("missing cost_usd"))?;
                        Ok(TuiEvent::VerifierDisabled {
                            reason: reason.to_string(),
                            cost_usd,
                        })
                    }
                    // Unknown event type → route to catch-all variant
                    _ => Ok(TuiEvent::Unknown { event_type: type_field.to_string() }),
                }
            }
        }

        deserializer.deserialize_map(TuiEventVisitor)
    }
}

// ─── End TuiEvent Deserialize impl ─────────────────────────────────────

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
            TuiEvent::Unknown { .. } => unreachable!("usage test should never hit Unknown"),
            _ => unreachable!("usage test hit unexpected variant"),
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
    fn unknown_variant_round_trips() {
        // The Unknown variant is the catch-all for future events.
        // Wire format: {"Unknown":{"event_type":"SomeNewFutureEvent","data":{"foo":123}}}
        // Our serde derive uses #[serde(rename = "Unknown")] on the Unknown variant,
        // so serialized output uses "Unknown" as the tag (same as other variant renames).
        let json = r#"{"type":"SomeNewFutureEvent","data":{"foo":123}}"#;
        let event = serde_json::from_str::<TuiEvent>(json).unwrap();
        match event {
            TuiEvent::Unknown { ref event_type } => {
                assert_eq!(event_type, "SomeNewFutureEvent");
            }
            _ => unreachable!("unknown test hit unexpected variant"),
        }
        // Unknown events round-trip — the variant tag serializes as "Unknown" per serde rename
        let round_trip = serde_json::to_string(&event).unwrap();
        assert!(
            round_trip.contains(r#""Unknown""#),
            "Unknown variant should serialize with 'Unknown' tag, got: {round_trip}"
        );
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
            TuiEvent::Unknown { .. } => unreachable!("retry test should never hit Unknown"),
            _ => unreachable!("retry test hit unexpected variant"),
        }
    }

    #[test]
    fn error_deserializes() {
        let json = r#"{"type":"Error","data":{"message":"connection reset"}}"#;
        let event = serde_json::from_str::<TuiEvent>(json).unwrap();
        match event {
            TuiEvent::Error { ref message } => assert_eq!(message, "connection reset"),
            TuiEvent::Unknown { .. } => unreachable!("error test should never hit Unknown"),
            _ => unreachable!("error test hit unexpected variant"),
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
                    _ => unreachable!("closed chunk test hit unexpected variant"),
                }
            }
            TuiEvent::Unknown { .. } => unreachable!("tool_output test should never hit Unknown"),
            _ => unreachable!("tool_output test hit unexpected variant"),
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
                    _ => unreachable!("pass outcome test hit unexpected variant"),
                }
            }
            TuiEvent::Unknown { .. } => unreachable!("task_complete test should never hit Unknown"),
            _ => unreachable!("task_complete test hit unexpected variant"),
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
    fn unknown_event_type_routes_to_unknown_variant() {
        // With the Unknown variant added, serde successfully deserializes
        // previously-unknown event types into the Unknown catch-all.
        // This is the correct behavior — JsonlParser layers its own
        // UnknownEventType error on top for UI-level handling.
        let json = r#"{"type":"SomeNewFutureEvent","data":{"foo":123}}"#;
        let result = serde_json::from_str::<TuiEvent>(json);
        assert!(
            result.is_ok(),
            "Unknown variant should successfully deserialize future event types"
        );
        match result.unwrap() {
            TuiEvent::Unknown { ref event_type } => {
                assert_eq!(event_type, "SomeNewFutureEvent");
            }
            _ => panic!("expected Unknown variant"),
        }
    }

    #[test]
    fn debug_tool_call_start() {
        let json = r#"{"type":"ToolCallStart","data":{"id":"x","name":"y"}}"#;
        let result = serde_json::from_str::<TuiEvent>(json);
        eprintln!("Result: {:?}", result);
        match result {
            Ok(TuiEvent::ToolCallStart { id, name }) => {
                eprintln!("id={}, name={}", id, name);
                assert_eq!(id, "x");
                assert_eq!(name, "y");
            }
            Ok(other) => panic!("wrong variant: {:?}", other),
            Err(e) => {
                eprintln!("ERROR: {}", e);
                panic!("deserialization failed");
            }
        }
    }

    #[test]
    fn debug_serde_json_map() {
        let json = r#"{"type":"ToolCallStart","data":{"id":"x","name":"y"}}"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        eprintln!("value = {:?}", value);
        if let serde_json::Value::Object(map) = value {
            eprintln!("map keys: {:?}", map.keys().collect::<Vec<_>>());
            for (k, v) in &map {
                eprintln!("  {}: {:?}", k, v);
            }
        }
    }
}
