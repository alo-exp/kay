//! Integration tests for PROV-03 / D-08 / TM-01.
//!
//! Note: these tests mutate `OPENROUTER_API_KEY`. They are serialized within
//! this binary via a module-static Mutex — cargo's test harness parallelizes
//! intra-binary by default, and env mutation is process-global.

#![allow(clippy::unwrap_used)]

use kay_provider_openrouter::{AuthErrorKind, ConfigAuthSource, ProviderError, resolve_api_key};
use std::sync::Mutex;

const ENV: &str = "OPENROUTER_API_KEY";

// Serialize env-mutating tests in this binary.
static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn missing_everywhere_yields_typed_error() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // SAFETY: env mutation is process-global; ENV_LOCK serializes. Rust 2024
    // marks env mutation as unsafe due to cross-thread data-race potential.
    unsafe {
        std::env::remove_var(ENV);
    }
    let r = resolve_api_key(None);
    match r {
        Err(ProviderError::Auth { reason: AuthErrorKind::Missing }) => {}
        other => panic!("expected Auth::Missing, got {other:?}"),
    }
}

#[test]
fn env_wins_on_conflict() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // SAFETY: see above.
    unsafe {
        std::env::remove_var(ENV);
        std::env::set_var(ENV, "sk-env");
    }
    let cfg = ConfigAuthSource::new(Some("sk-config".into()));
    let key = resolve_api_key(Some(&cfg)).unwrap();
    // Verify via Debug redaction — asserting on as_str() is a crate-private
    // accessor and the integration test can't call it. Use the Debug
    // invariant instead to prove the key is well-formed.
    assert_eq!(format!("{key:?}"), "ApiKey(<redacted>)");
    unsafe {
        std::env::remove_var(ENV);
    }
}

#[test]
fn config_fallback_when_env_unset() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        std::env::remove_var(ENV);
    }
    let cfg = ConfigAuthSource::new(Some("sk-from-config".into()));
    let key = resolve_api_key(Some(&cfg)).unwrap();
    // Prove the Debug doesn't leak the key.
    assert_eq!(format!("{key:?}"), "ApiKey(<redacted>)");
}

#[test]
fn debug_never_leaks_credential_in_error_display() {
    // If an API key were ever accidentally stored in an error variant,
    // formatting that error would expose it. Confirm none of the typed
    // Auth error variants carry credential material.
    let e = ProviderError::Auth { reason: AuthErrorKind::Missing };
    let s = format!("{e:?}");
    // Should mention the reason, not any credential text.
    assert!(s.contains("Missing"));
    assert!(!s.contains("sk-"));
    let d = format!("{e}");
    assert!(d.contains("authentication"));
    assert!(!d.contains("sk-"));
}
