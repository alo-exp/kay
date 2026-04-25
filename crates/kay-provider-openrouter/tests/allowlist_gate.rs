//! Integration tests for PROV-04 / AC-07 / D-07.
//!
//! Asserts the launch allowlist (anthropic/claude-sonnet-4.6,
//! anthropic/claude-opus-4.6, openai/gpt-5.4, minimax/minimax-m2.7,
//! minimax/minimax-m2.5, minimax/minimax-m2.1) behaves per D-07 and that
//! the allowlist gate rejects non-allowlisted models BEFORE any HTTP call
//! — this test does not spin a mock server; it only exercises the pre-flight
//! gate. Plan 02-08's streaming_happy_path.rs adds the HTTP-side proof.

#![allow(clippy::unwrap_used)] // tests may unwrap to surface diagnostics

use kay_provider_openrouter::{Allowlist, ProviderError};

fn launch_allowlist() -> Allowlist {
    let path = format!(
        "{}/tests/fixtures/config/allowlist.json",
        env!("CARGO_MANIFEST_DIR")
    );
    Allowlist::from_path(&path).expect("load fixture")
}

#[test]
fn launch_allowlist_accepts_all_three_launch_models() {
    let a = launch_allowlist();
    assert!(a.check("anthropic/claude-sonnet-4.6").is_ok());
    assert!(a.check("anthropic/claude-opus-4.6").is_ok());
    assert!(a.check("openai/gpt-5.4").is_ok());
}

#[test]
fn launch_allowlist_rejects_random_model() {
    let a = launch_allowlist();
    let r = a.check("mistral/mixtral-8x22b");
    match r {
        Err(ProviderError::ModelNotAllowlisted { requested, allowed }) => {
            assert_eq!(requested, "mistral/mixtral-8x22b");
            assert_eq!(allowed.len(), 6);
        }
        other => panic!("expected ModelNotAllowlisted, got {other:?}"),
    }
}

#[test]
fn wire_model_always_has_exacto_suffix() {
    let a = launch_allowlist();
    assert_eq!(
        a.to_wire_model("anthropic/claude-sonnet-4.6"),
        "anthropic/claude-sonnet-4.6:exacto"
    );
    assert_eq!(a.to_wire_model("openai/gpt-5.4"), "openai/gpt-5.4:exacto");
}

#[test]
fn crlf_smuggling_rejected_before_allowlist_compare() {
    let a = launch_allowlist();
    // Even though the prefix matches an allowlisted model, the control char
    // causes rejection — prevents CRLF injection into downstream HTTP headers.
    assert!(matches!(
        a.check("anthropic/claude-sonnet-4.6\r\nX-Evil: 1"),
        Err(ProviderError::ModelNotAllowlisted { .. })
    ));
}

#[test]
fn mixed_case_input_accepted() {
    let a = launch_allowlist();
    assert!(a.check("Anthropic/Claude-Sonnet-4.6").is_ok());
}

#[test]
fn exacto_suffix_input_accepted_and_canonicalized() {
    let a = launch_allowlist();
    // Pitfall 8 — user passes :exacto-suffixed form; check canonicalizes.
    assert!(a.check("anthropic/claude-sonnet-4.6:exacto").is_ok());
}
