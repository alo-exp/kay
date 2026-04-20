//! Per-session cost cap (PROV-06, D-10) — minimum-viable stub.
//!
//! Plan 02-08 T2 pre-introduces `cost_cap: Arc<CostCap>` on the
//! `OpenRouterProvider` so the struct shape is stable from Wave 8 (checker
//! BLOCKER #5). Plan 02-10 T1 EXTENDS this file with `with_cap()` validation
//! tests, `spent_usd()` / `cap_usd()` accessors, and the 7-test unit suite.
//! It does NOT change the field types or visibility.
//!
//! Keep this file minimal: only what plan 02-08 needs to compile + provide
//! an uncapped default.

use std::sync::Mutex;

use crate::error::ProviderError;

/// Tracks session-level cost accumulation with an optional cap (D-10).
/// `uncapped()` is the default for plan 02-08 (no CLI `--max-usd` yet).
pub struct CostCap {
    cap_usd: Option<f64>,
    spent_usd: Mutex<f64>,
}

impl CostCap {
    /// Default uncapped session (D-10). Plan 02-10 T2 wires
    /// `OpenRouterProviderBuilder::max_usd(n)` through `with_cap`.
    pub fn uncapped() -> Self {
        Self { cap_usd: None, spent_usd: Mutex::new(0.0) }
    }

    /// Cap a session at `cap_usd` dollars. Returns error for non-positive /
    /// non-finite inputs. Full validation + unit tests land in plan 02-10 T1.
    pub fn with_cap(cap: f64) -> Result<Self, ProviderError> {
        if !cap.is_finite() || cap <= 0.0 {
            return Err(ProviderError::Stream(
                "--max-usd must be a positive finite number".into(),
            ));
        }
        Ok(Self { cap_usd: Some(cap), spent_usd: Mutex::new(0.0) })
    }

    /// Pre-turn gate (D-10). Surface `CostCapExceeded` if already over cap.
    /// Uncapped sessions always pass.
    ///
    /// `Mutex::lock().unwrap_or_else(|e| e.into_inner())` is clippy-clean:
    /// it does NOT invoke `.unwrap()` / `.expect()` which are what the
    /// crate-wide `#![deny(clippy::unwrap_used, clippy::expect_used)]`
    /// lint forbids. Poisoned mutex from a panicked writer is rare and
    /// graceful recovery is preferred over a second panic.
    pub fn check(&self) -> Result<(), ProviderError> {
        let Some(cap) = self.cap_usd else {
            return Ok(());
        };
        let spent = *self.spent_usd.lock().unwrap_or_else(|e| e.into_inner());
        if spent > cap {
            Err(ProviderError::CostCapExceeded { cap_usd: cap, spent_usd: spent })
        } else {
            Ok(())
        }
    }

    /// Accumulate a turn's cost. Plan 02-10 T2 wires this from the
    /// translator's Usage-emission site; plan 02-08 leaves the hook
    /// here so that wiring is a one-line change. Negative cost values
    /// (wire-level noise) are clamped to 0.
    ///
    /// Same poisoned-mutex recovery rationale as `check()` above.
    pub fn accumulate(&self, cost_usd: f64) {
        let mut s = self.spent_usd.lock().unwrap_or_else(|e| e.into_inner());
        *s += cost_usd.max(0.0);
    }

    /// Read accessor for diagnostics / tests.
    pub fn spent_usd(&self) -> f64 {
        *self.spent_usd.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Read accessor for diagnostics / tests. Returns `None` for uncapped
    /// sessions.
    pub fn cap_usd(&self) -> Option<f64> {
        self.cap_usd
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod unit {
    use super::*;

    #[test]
    fn uncapped_always_passes() {
        let c = CostCap::uncapped();
        assert!(c.check().is_ok());
        c.accumulate(100_000.0);
        assert!(c.check().is_ok());
        assert_eq!(c.cap_usd(), None);
        assert!((c.spent_usd() - 100_000.0).abs() < 1e-9);
    }

    #[test]
    fn with_cap_rejects_zero() {
        assert!(matches!(
            CostCap::with_cap(0.0),
            Err(ProviderError::Stream(_))
        ));
    }

    #[test]
    fn with_cap_rejects_negative() {
        assert!(matches!(
            CostCap::with_cap(-1.0),
            Err(ProviderError::Stream(_))
        ));
    }

    #[test]
    fn with_cap_rejects_nan() {
        assert!(matches!(
            CostCap::with_cap(f64::NAN),
            Err(ProviderError::Stream(_))
        ));
    }

    #[test]
    fn with_cap_rejects_infinity() {
        assert!(matches!(
            CostCap::with_cap(f64::INFINITY),
            Err(ProviderError::Stream(_))
        ));
    }

    #[test]
    fn under_cap_passes_over_cap_fails() {
        let c = CostCap::with_cap(1.0).unwrap();
        c.accumulate(0.5);
        assert!(c.check().is_ok());
        c.accumulate(0.6); // total 1.1 → over cap
        match c.check() {
            Err(ProviderError::CostCapExceeded { cap_usd, spent_usd }) => {
                assert_eq!(cap_usd, 1.0);
                assert!((spent_usd - 1.1).abs() < 1e-9);
            }
            other => panic!("expected CostCapExceeded, got {other:?}"),
        }
        assert_eq!(c.cap_usd(), Some(1.0));
        assert!((c.spent_usd() - 1.1).abs() < 1e-9);
    }

    #[test]
    fn negative_accumulate_clamped() {
        let c = CostCap::with_cap(1.0).unwrap();
        c.accumulate(-5.0);
        assert_eq!(c.spent_usd(), 0.0);
        assert!(c.check().is_ok());
    }
}
