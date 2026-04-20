//! RNG seam — deterministic in tests, OsRng in prod (E2 / R-5).

pub trait RngSeam: Send + Sync {
    fn hex_nonce(&self, len: usize) -> String;
}

pub struct OsRngSeam;

impl RngSeam for OsRngSeam {
    fn hex_nonce(&self, len: usize) -> String {
        use rand::TryRng;
        let mut bytes = vec![0u8; len];
        rand::rngs::SysRng
            .try_fill_bytes(&mut bytes)
            .expect("OsRng fill_bytes failed");
        hex::encode(&bytes)
    }
}

pub struct DeterministicRng {
    pub seed: u64,
}

impl RngSeam for DeterministicRng {
    fn hex_nonce(&self, len: usize) -> String {
        // LCG with fixed seed — purely for test determinism.
        let mut state = self.seed;
        let mut bytes = vec![0u8; len];
        for b in &mut bytes {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *b = (state >> 33) as u8;
        }
        hex::encode(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_rng_produces_same_output() {
        let rng = DeterministicRng { seed: 42 };
        let a = rng.hex_nonce(8);
        let b = rng.hex_nonce(8);
        assert_eq!(a, b);
    }

    #[test]
    fn test_deterministic_rng_different_seeds_differ() {
        let a = DeterministicRng { seed: 1 }.hex_nonce(8);
        let b = DeterministicRng { seed: 2 }.hex_nonce(8);
        assert_ne!(a, b);
    }

    #[test]
    fn test_os_rng_hex_length() {
        let rng = OsRngSeam;
        let nonce = rng.hex_nonce(8);
        assert_eq!(nonce.len(), 16); // 8 bytes → 16 hex chars
    }

    #[test]
    fn test_os_rng_produces_unique_output() {
        let rng = OsRngSeam;
        let a = rng.hex_nonce(16);
        let b = rng.hex_nonce(16);
        assert_ne!(a, b);
    }
}
