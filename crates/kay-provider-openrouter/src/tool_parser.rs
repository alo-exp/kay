//! Tolerant two-pass tool-call arguments parser (PROV-05, D-03).
//!
//! Pass 1: `serde_json::from_str` (strict).
//! Pass 2 (on pass-1 failure): `forge_json_repair::json_repair`
//!   (battle-tested against OpenRouter's real tool-call malformations —
//!   trailing commas, unquoted keys, stringified numbers, markdown code-block
//!   wrappers, etc.).
//! Pass 3 (on pass-2 failure): `ParseOutcome::Malformed` with diagnostic.
//!
//! Invariant: NEVER PANIC. This is enforced by the crate-root lint
//! `#![deny(clippy::unwrap_used, clippy::expect_used)]` and proven by
//! the proptest in the `unit` module below.
//!
//! Post-Phase-2.5 note (Appendix A, Substitution Rule 2): the plan pre-2.5
//! said `use kay_core::forge_json_repair::json_repair;` but after the
//! sub-crate split the path is `forge_json_repair::json_repair` (direct
//! dep of `kay-provider-openrouter` Cargo.toml). Recorded as a Rule-2
//! mechanical substitution in SUMMARY.md.
//!
//! Signature note (Rule-3 deviation): the plan interface sketch showed
//! `json_repair::<Value>(raw.to_string())` — taking `String` by owned
//! value. The real signature in `crates/forge_json_repair/src/parser.rs`
//! line 1070 is `pub fn json_repair<De: for<'de> Deserialize<'de>>(text: &str) -> Result<De>`,
//! i.e. `&str`. We pass `raw` directly; no `.to_string()` allocation.

use forge_json_repair::json_repair;
use serde_json::Value;

/// Maximum bytes we accumulate into a single tool_call's `arguments_raw`
/// buffer. Arguments larger than this are reported as Malformed rather
/// than fed into the parser (TM-06 DoS mitigation).
pub const MAX_TOOL_ARGS_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseOutcome {
    /// Pass-1 strict parse succeeded.
    Clean(Value),
    /// Pass-1 failed; pass-2 `forge_json_repair` succeeded.
    Repaired(Value),
    /// Both passes failed. `error` carries a human-readable diagnostic
    /// from `forge_json_repair`'s error `Display`.
    Malformed { error: String },
}

/// Parse tool-call arguments raw bytes into a `serde_json::Value`.
///
/// Empty input is treated as the empty object `{}` (OpenRouter variance
/// per D-04: null / empty initial argument deltas are legal).
pub fn parse_tool_arguments(raw: &str) -> ParseOutcome {
    if raw.is_empty() {
        return ParseOutcome::Clean(Value::Object(Default::default()));
    }
    // Pass 1: strict
    match serde_json::from_str::<Value>(raw) {
        Ok(v) => ParseOutcome::Clean(v),
        Err(_strict_err) => {
            // Pass 2: forge_json_repair (&str per actual post-2.5 signature).
            match json_repair::<Value>(raw) {
                Ok(v) => ParseOutcome::Repaired(v),
                Err(repair_err) => ParseOutcome::Malformed { error: repair_err.to_string() },
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod unit {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn empty_input_is_empty_object() {
        let o = parse_tool_arguments("");
        assert_eq!(o, ParseOutcome::Clean(Value::Object(Default::default())));
    }

    #[test]
    fn well_formed_json_takes_strict_path() {
        let o = parse_tool_arguments(r#"{"cmd":"ls -la"}"#);
        match o {
            ParseOutcome::Clean(v) => {
                assert_eq!(v["cmd"], Value::String("ls -la".into()));
            }
            other => panic!("expected Clean, got {other:?}"),
        }
    }

    #[test]
    fn trailing_comma_repaired() {
        // Canonical OpenRouter variance.
        let o = parse_tool_arguments(r#"{"cmd":"ls",}"#);
        match o {
            ParseOutcome::Repaired(v) => {
                assert_eq!(v["cmd"], Value::String("ls".into()));
            }
            other => panic!("expected Repaired, got {other:?}"),
        }
    }

    #[test]
    fn unquoted_keys_repaired() {
        let o = parse_tool_arguments(r#"{cmd: "ls"}"#);
        // forge_json_repair handles unquoted keys.
        assert!(
            matches!(o, ParseOutcome::Repaired(_)),
            "expected Repaired for unquoted-key JSON, got {o:?}"
        );
    }

    #[test]
    fn catastrophic_input_malformed() {
        // `forge_json_repair` is extraordinarily tolerant - strings like
        // "not json !!@#", bare ":::", "\"unclosed", or "[1,2," all coerce
        // to some JSON value (string wrapping / auto-close / etc.). The
        // inputs below were empirically probed against the pass-2 repairer
        // and still yield an error (structural tokens the repairer cannot
        // reconcile):
        //   "{{}}}"            → UnexpectedCharacter at pos 1
        //   "{:}"              → ObjectKeyExpected at pos 1
        //   ",,,"              → UnexpectedEnd
        //   "null null null"   → UnexpectedCharacter at pos 5
        //   "true false"       → UnexpectedCharacter at pos 5
        for bad in ["{{}}}", "{:}", ",,,", "null null null", "true false"] {
            let o = parse_tool_arguments(bad);
            assert!(
                matches!(o, ParseOutcome::Malformed { .. }),
                "expected Malformed for {bad:?}, got {o:?}"
            );
        }
    }

    #[test]
    fn max_bytes_constant_is_one_mebibyte() {
        assert_eq!(MAX_TOOL_ARGS_BYTES, 1_048_576);
    }

    proptest! {
        /// PROV-05 never-panic invariant.
        /// For any Unicode string, `parse_tool_arguments` returns without panicking.
        /// This is the single most important correctness invariant of the tolerant
        /// parser - the whole point of D-03's two-pass design is that the second
        /// pass (`forge_json_repair`) never crashes on adversarial input.
        #[test]
        fn parser_never_panics(raw in "\\PC*") {
            let _ = parse_tool_arguments(&raw);
        }

        /// Well-formed serde-serialized JSON always takes the Clean path.
        /// Generates a HashMap<String, i64>, serializes, and asserts strict
        /// parse wins (no repair path needed).
        #[test]
        fn well_formed_json_always_clean(
            obj in proptest::collection::hash_map(
                "[a-z]{1,10}",
                proptest::prelude::any::<i64>(),
                1..10,
            )
        ) {
            let raw = serde_json::to_string(&obj).unwrap();
            match parse_tool_arguments(&raw) {
                ParseOutcome::Clean(v) => {
                    for (k, expected) in obj.iter() {
                        prop_assert_eq!(v[k].as_i64().unwrap(), *expected);
                    }
                }
                other => {
                    prop_assert!(false, "expected Clean, got {:?}", other);
                }
            }
        }
    }
}
