//! Image quota bookkeeping (TOOL-04 / D-07).
//!
//! Wave 4 (03-05) gave `try_consume` its real body: atomically increment
//! the per-turn counter first (it's the tighter cap) and then the
//! per-session counter; on overflow, roll the increment back and return
//! the `CapScope` variant identifying the breached dimension. The tool
//! (`image_read`) is responsible for translating that `CapScope` into a
//! `ToolError::ImageCapExceeded` with the appropriate `limit` field.

use std::sync::atomic::{AtomicU32, Ordering};

use crate::error::CapScope;

pub struct ImageQuota {
    pub max_per_turn: u32,
    pub max_per_session: u32,
    per_turn: AtomicU32,
    per_session: AtomicU32,
}

impl ImageQuota {
    pub fn new(max_per_turn: u32, max_per_session: u32) -> Self {
        Self {
            max_per_turn,
            max_per_session,
            per_turn: AtomicU32::new(0),
            per_session: AtomicU32::new(0),
        }
    }

    /// Reserve one image slot, charging both the per-turn and per-session
    /// counters. Returns `Ok(())` on success, or the first `CapScope`
    /// whose cap was breached (checked in order: PerTurn, PerSession).
    /// A breach rolls back the already-applied increment so the counters
    /// never observe a "leaked" reservation.
    pub fn try_consume(&self) -> Result<(), CapScope> {
        // Bump per-turn first — it's the tighter window.
        let turn_now = self.per_turn.fetch_add(1, Ordering::AcqRel);
        if turn_now >= self.max_per_turn {
            // Roll back; nothing was consumed.
            self.per_turn.fetch_sub(1, Ordering::AcqRel);
            return Err(CapScope::PerTurn);
        }
        let session_now = self.per_session.fetch_add(1, Ordering::AcqRel);
        if session_now >= self.max_per_session {
            // Roll back both counters.
            self.per_turn.fetch_sub(1, Ordering::AcqRel);
            self.per_session.fetch_sub(1, Ordering::AcqRel);
            return Err(CapScope::PerSession);
        }
        Ok(())
    }

    /// Per-turn reset (called by the agent loop at turn boundaries).
    /// The per-session counter is never reset within a session.
    pub fn reset_turn(&self) {
        self.per_turn.store(0, Ordering::Relaxed);
    }

    /// Release a previously-consumed slot (reverses a successful
    /// `try_consume`). Used by `ImageReadTool` when a filesystem read
    /// fails AFTER the quota has been charged — without this, a malicious
    /// prompt supplying non-existent paths can drain the per-session cap
    /// without ever reading a byte (M-02 / low-effort DoS against IMG-01).
    ///
    /// Saturating subtraction protects against imbalanced `release` calls
    /// (should never happen in practice, but an underflow here would be
    /// worse than a stuck cap).
    pub fn release(&self) {
        // Saturating: if for some reason the counter is already 0 we
        // refuse to wrap around. `fetch_update` lets us do this in one
        // atomic op.
        let _ = self.per_turn.fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
            Some(n.saturating_sub(1))
        });
        let _ = self.per_session.fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
            Some(n.saturating_sub(1))
        });
    }

    /// Observability helper for tests — current per-turn count.
    pub fn per_turn_count(&self) -> u32 {
        self.per_turn.load(Ordering::Acquire)
    }

    /// Observability helper for tests — current per-session count.
    pub fn per_session_count(&self) -> u32 {
        self.per_session.load(Ordering::Acquire)
    }

    /// Surface the cap breach as a `ToolError::ImageCapExceeded`, picking
    /// the appropriate `limit` field based on scope. Kept on `ImageQuota`
    /// (not `CapScope`) so the tool code stays a one-liner.
    pub fn limit_for(&self, scope: CapScope) -> u32 {
        match scope {
            CapScope::PerTurn => self.max_per_turn,
            CapScope::PerSession => self.max_per_session,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn consumes_up_to_per_turn_limit_then_rejects() {
        let q = ImageQuota::new(2, 10);
        assert!(q.try_consume().is_ok());
        assert!(q.try_consume().is_ok());
        let err = q.try_consume().unwrap_err();
        assert_eq!(err, CapScope::PerTurn);
        // Rollback keeps count at the cap, not above.
        assert_eq!(q.per_turn_count(), 2);
    }

    #[test]
    fn reset_turn_allows_more_consumption() {
        let q = ImageQuota::new(1, 10);
        assert!(q.try_consume().is_ok());
        assert!(q.try_consume().is_err());
        q.reset_turn();
        assert!(q.try_consume().is_ok());
        assert_eq!(q.per_session_count(), 2);
    }

    #[test]
    fn per_session_cap_breach_rolls_back_both_counters() {
        let q = ImageQuota::new(u32::MAX, 1);
        assert!(q.try_consume().is_ok());
        let err = q.try_consume().unwrap_err();
        assert_eq!(err, CapScope::PerSession);
        assert_eq!(q.per_turn_count(), 1);
        assert_eq!(q.per_session_count(), 1);
    }

    #[test]
    fn limit_for_returns_configured_caps() {
        let q = ImageQuota::new(3, 7);
        assert_eq!(q.limit_for(CapScope::PerTurn), 3);
        assert_eq!(q.limit_for(CapScope::PerSession), 7);
    }
}
