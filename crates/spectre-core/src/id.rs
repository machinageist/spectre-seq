// Author: Jeff
// Date: 2026-07-11
// Description: Stable 64-bit object identity (CORE-001)
// Notes: Nonzero, generator-scoped uniqueness; project-wide duplicate validation arrives with persisted object collections

use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

// Splitmix64 increment constant
const SPLITMIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

// Stable identity for every user-visible object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct ObjectId(u64);

impl<'de> Deserialize<'de> for ObjectId {
    // Decode the transparent integer while enforcing nonzero identity
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = u64::deserialize(deserializer)?;
        Self::from_raw(raw).ok_or_else(|| D::Error::custom("object ID must be nonzero"))
    }
}

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

    #[test]
    fn zero_json_is_rejected() {
        let error = serde_json::from_str::<ObjectId>("0").unwrap_err();

        assert!(error.to_string().contains("object ID must be nonzero"));
    }

    #[test]
    fn nonzero_numeric_json_round_trips_exactly() {
        for raw in [1, 1_311_768_467_463_790_320, u64::MAX] {
            let id = ObjectId::from_raw(raw).unwrap();
            let json = serde_json::to_string(&id).unwrap();

            assert_eq!(json, raw.to_string());
            assert_eq!(serde_json::from_str::<ObjectId>(&json).unwrap(), id);
        }
    }
}
