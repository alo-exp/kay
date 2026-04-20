//! Image quota bookkeeping (TOOL-04 / D-07).

use std::sync::atomic::{AtomicU32, Ordering};

use crate::error::CapScope;

pub struct ImageQuota {
    pub max_per_turn: u32,
    pub max_per_session: u32,
    per_turn: AtomicU32,
    // Read by `try_consume` in Wave 4 (03-05); the field is written by
    // `new` today so keep `#[allow(dead_code)]` localized rather than
    // suppressing the lint crate-wide.
    #[allow(dead_code)]
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

    pub fn try_consume(&self) -> Result<(), CapScope> {
        todo!("Wave 4 (03-05): increment atomics with cap check; return CapScope on breach")
    }

    pub fn reset_turn(&self) {
        self.per_turn.store(0, Ordering::Relaxed);
    }
}
