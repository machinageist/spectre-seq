// Author: Jeff
// Date: 2026-07-11
// Description: Stable 64-bit object identity (CORE-001)
// Notes: Nonzero, generator-scoped uniqueness; project-wide duplicate validation arrives with persisted object collections

use serde::{Deserialize, Serialize};

// Splitmix64 increment constant
const SPLITMIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

// Stable identity for every user-visible object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ObjectId(u64);

impl ObjectId {
    // Construct from a raw nonzero value
    pub fn from_raw(raw: u64) -> Option<Self> {
        if raw == 0 {
            None
        } else {
            Some(Self(raw))
        }
    }

    // Expose the raw value for persistence
    pub fn raw(self) -> u64 {
        self.0
    }
}

// Deterministic splitmix64 ID generator; seeded per project
#[derive(Debug, Clone)]
pub struct IdGen {
    state: u64,
}

impl IdGen {
    // Create a generator from a seed
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    // Produce the next nonzero ID
    pub fn next_id(&mut self) -> ObjectId {
        loop {
            self.state = self.state.wrapping_add(SPLITMIX_GAMMA);
            let mut z = self.state;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^= z >> 31;
            if let Some(id) = ObjectId::from_raw(z) {
                return id;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // Same seed must reproduce the same sequence
    #[test]
    fn generator_is_deterministic() {
        let mut a = IdGen::new(42);
        let mut b = IdGen::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_id(), b.next_id());
        }
    }

    // A generous run must not collide
    #[test]
    fn no_collisions_in_large_run() {
        let mut g = IdGen::new(7);
        let mut seen = HashSet::new();
        for _ in 0..100_000 {
            assert!(seen.insert(g.next_id()));
        }
    }

    // Zero is not a valid identity
    #[test]
    fn zero_is_rejected() {
        assert!(ObjectId::from_raw(0).is_none());
        assert!(ObjectId::from_raw(1).is_some());
    }
}
