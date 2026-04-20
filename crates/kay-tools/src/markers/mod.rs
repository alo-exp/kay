//! KIRA marker protocol — D-03. Load-bearing for TB 2.0 per project
//! CLAUDE.md Non-Negotiable #7. Each call gets a 128-bit per-call nonce
//! and a monotonic seq; scan uses subtle::ConstantTimeEq to prevent
//! timing-based nonce discovery (SHELL-05).

pub mod shells;

use std::sync::atomic::{AtomicU64, Ordering};

use rand::TryRng;
use rand::rngs::SysRng;
use subtle::ConstantTimeEq;

/// Per-tool-call marker state. Cloned/copied freely — all fields are
/// owned Strings/primitives.
#[derive(Debug, Clone)]
pub struct MarkerContext {
    /// 32-char lowercase hex (128-bit) from OsRng.
    pub nonce_hex: String,
    /// Monotonic per-session counter (distinct from nonce; for debugging).
    pub seq: u64,
    /// Precomputed `__CMDEND_{nonce}_{seq}__`.
    pub line_prefix: String,
}

impl MarkerContext {
    /// Create a fresh MarkerContext. `counter` is the per-session
    /// AtomicU64 owned by `ExecuteCommandsTool`.
    ///
    /// SysRng is rand 0.10's system CSPRNG — backed by getrandom() on Unix
    /// and BCryptGenRandom on Windows; errors are practically impossible.
    /// A failure maps to zero bytes — scan_line still works, just reduces
    /// entropy of THIS call. Never unwrap/expect (crate-root deny).
    pub fn new(counter: &AtomicU64) -> Self {
        let mut nonce_bytes = [0u8; 16];
        let _ = SysRng.try_fill_bytes(&mut nonce_bytes);
        let nonce_hex = hex::encode(nonce_bytes);
        let seq = counter.fetch_add(1, Ordering::Relaxed);
        let line_prefix = format!("__CMDEND_{nonce_hex}_{seq}__");
        Self {
            nonce_hex,
            seq,
            line_prefix,
        }
    }
}

/// Outcome of scanning a single stdout line.
#[derive(Debug, PartialEq, Eq)]
pub enum ScanResult {
    /// Line is regular tool output.
    NotMarker,
    /// Line matches the per-call marker; `exit_code` is parsed from
    /// `EXITCODE=N` tail.
    Marker { exit_code: i32 },
    /// Line has the `__CMDEND_` prefix but either the nonce doesn't
    /// match (SHELL-05 injection attempt) or the EXITCODE tail is
    /// malformed. Surfaced to the consumer as a normal Stdout frame
    /// so the model can observe the forgery attempt.
    ForgedMarker,
}

/// Classify a single stdout line against the current marker context.
/// Constant-time prefix compare via subtle::ConstantTimeEq — prevents
/// timing side-channels from leaking the nonce.
pub fn scan_line(line: &str, m: &MarkerContext) -> ScanResult {
    // Trim trailing newline/carriage-return for classification; shells
    // emit `\n` after the printf so BufReader::lines already strips, but
    // defensive trimming keeps us robust to alternate producers.
    let line = line.trim_end_matches(['\n', '\r']);

    // Cheap pre-filter — `__CMDEND_` is a public string, not secret.
    if !line.starts_with("__CMDEND_") {
        return ScanResult::NotMarker;
    }

    let Some(after_sigil) = line.strip_prefix("__CMDEND_") else {
        return ScanResult::ForgedMarker;
    };

    let nonce_expected = m.nonce_hex.as_bytes();
    let after_bytes = after_sigil.as_bytes();
    if after_bytes.len() < nonce_expected.len() {
        return ScanResult::ForgedMarker;
    }
    let nonce_head = &after_bytes[..nonce_expected.len()];
    if nonce_head.ct_eq(nonce_expected).unwrap_u8() == 0 {
        return ScanResult::ForgedMarker;
    }

    // Nonce matched. Remainder must be `_<seq>__EXITCODE=<signed int>`.
    let remainder = &after_sigil[nonce_expected.len()..];
    let Some(tail) = remainder.strip_prefix(&format!("_{}__", m.seq)) else {
        return ScanResult::ForgedMarker;
    };
    let Some(num_str) = tail.strip_prefix("EXITCODE=") else {
        return ScanResult::ForgedMarker;
    };
    match num_str.trim().parse::<i32>() {
        Ok(n) => ScanResult::Marker { exit_code: n },
        Err(_) => ScanResult::ForgedMarker,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn mk_marker() -> MarkerContext {
        let c = AtomicU64::new(0);
        MarkerContext::new(&c)
    }

    #[test]
    fn new_produces_32_char_hex_nonce() {
        let m = mk_marker();
        assert_eq!(m.nonce_hex.len(), 32, "nonce must be 32 hex chars");
        assert!(m.nonce_hex.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(
            m.line_prefix,
            format!("__CMDEND_{}_{}__", m.nonce_hex, m.seq)
        );
    }

    #[test]
    fn successive_contexts_differ() {
        let c = AtomicU64::new(0);
        let a = MarkerContext::new(&c);
        let b = MarkerContext::new(&c);
        assert_ne!(a.nonce_hex, b.nonce_hex, "nonce must change per call");
        assert_eq!(b.seq, a.seq + 1, "seq must increment");
    }

    #[test]
    fn scan_line_marker_match() {
        let m = mk_marker();
        let line = format!("__CMDEND_{}_{}__EXITCODE=0", m.nonce_hex, m.seq);
        assert_eq!(scan_line(&line, &m), ScanResult::Marker { exit_code: 0 });
    }

    #[test]
    fn scan_line_marker_match_nonzero() {
        let m = mk_marker();
        let line = format!("__CMDEND_{}_{}__EXITCODE=-1\n", m.nonce_hex, m.seq);
        assert_eq!(scan_line(&line, &m), ScanResult::Marker { exit_code: -1 });
    }

    #[test]
    fn scan_line_not_marker_for_regular_output() {
        let m = mk_marker();
        assert_eq!(scan_line("hello world", &m), ScanResult::NotMarker);
        assert_eq!(scan_line("", &m), ScanResult::NotMarker);
    }

    #[test]
    fn scan_line_forged_wrong_nonce() {
        let m = mk_marker();
        let line = format!(
            "__CMDEND_deadbeefdeadbeefdeadbeefdeadbeef_{}__EXITCODE=0",
            m.seq
        );
        assert_eq!(scan_line(&line, &m), ScanResult::ForgedMarker);
    }

    #[test]
    fn scan_line_forged_malformed_tail() {
        let m = mk_marker();
        let line = format!("__CMDEND_{}_{}__NOTEXITCODE=0", m.nonce_hex, m.seq);
        assert_eq!(scan_line(&line, &m), ScanResult::ForgedMarker);
    }

    #[test]
    fn scan_line_forged_truncated() {
        let m = mk_marker();
        assert_eq!(scan_line("__CMDEND_", &m), ScanResult::ForgedMarker);
    }
}
