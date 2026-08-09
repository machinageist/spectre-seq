// Author: Jeff
// Date: 2026-07-12
// Description: Validated time signatures and meter map (TIME-004)
// Notes: Numerator 1..=99, denominator in {1,2,4,8,16}; changes strictly ordered from BeatTicks(0)

use crate::time::{BeatTicks, TICKS_PER_BEAT};
use serde::{Deserialize, Serialize};

// Inclusive numerator bounds
pub const MIN_NUMERATOR: u8 = 1;
pub const MAX_NUMERATOR: u8 = 99;

// Permitted denominators
pub const VALID_DENOMINATORS: [u8; 5] = [1, 2, 4, 8, 16];

// Meter construction failures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeterError {
    InvalidNumerator,
    InvalidDenominator,
    Empty,
    FirstChangeNotAtZero,
    NegativePosition,
    UnsortedOrDuplicate,
}

impl std::fmt::Display for MeterError {
    // Render a user-facing failure description
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            MeterError::InvalidNumerator => "numerator must be 1..=99",
            MeterError::InvalidDenominator => "denominator must be 1, 2, 4, 8, or 16",
            MeterError::Empty => "meter map must contain at least one change",
            MeterError::FirstChangeNotAtZero => "first meter change must start at tick zero",
            MeterError::NegativePosition => "meter change positions must be non-negative",
            MeterError::UnsortedOrDuplicate => "meter changes must be strictly increasing",
        };
        f.write_str(text)
    }
}

impl std::error::Error for MeterError {}

// A validated musical time signature
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawTimeSignature", into = "RawTimeSignature")]
pub struct TimeSignature {
    numerator: u8,
    denominator: u8,
}

// Unvalidated wire form backing serde
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct RawTimeSignature {
    numerator: u8,
    denominator: u8,
}

impl TryFrom<RawTimeSignature> for TimeSignature {
    type Error = MeterError;
    // Validate the wire form on deserialization
    fn try_from(raw: RawTimeSignature) -> Result<Self, MeterError> {
        TimeSignature::new(raw.numerator, raw.denominator)
    }
}

impl From<TimeSignature> for RawTimeSignature {
    // Emit the wire form on serialization
    fn from(sig: TimeSignature) -> Self {
        Self {
            numerator: sig.numerator,
            denominator: sig.denominator,
        }
    }
}

impl TimeSignature {
    // Build a validated signature
    pub fn new(numerator: u8, denominator: u8) -> Result<Self, MeterError> {
        if !(MIN_NUMERATOR..=MAX_NUMERATOR).contains(&numerator) {
            return Err(MeterError::InvalidNumerator);
        }
        if !VALID_DENOMINATORS.contains(&denominator) {
            return Err(MeterError::InvalidDenominator);
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }

    // Expose the numerator
    pub fn numerator(self) -> u8 {
        self.numerator
    }

    // Expose the denominator
    pub fn denominator(self) -> u8 {
        self.denominator
    }

    // Exact bar length in ticks; a beat tick is a quarter note
    pub fn ticks_per_bar(self) -> BeatTicks {
        BeatTicks(self.numerator as i64 * (TICKS_PER_BEAT * 4 / self.denominator as i64))
    }
}

// One meter change anchored at a tick position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeterChange {
    pub start: BeatTicks,
    pub signature: TimeSignature,
}

// Ordered meter changes; the first MUST start at tick zero
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<MeterChange>", into = "Vec<MeterChange>")]
pub struct MeterMap {
    changes: Vec<MeterChange>,
}

impl TryFrom<Vec<MeterChange>> for MeterMap {
    type Error = MeterError;
    // Validate the wire form on deserialization
    fn try_from(changes: Vec<MeterChange>) -> Result<Self, MeterError> {
        MeterMap::new(changes)
    }
}

impl From<MeterMap> for Vec<MeterChange> {
    // Emit the wire form on serialization
    fn from(map: MeterMap) -> Self {
        map.changes
    }
}

impl MeterMap {
    // Build a validated map from changes
    pub fn new(changes: Vec<MeterChange>) -> Result<Self, MeterError> {
        if changes.is_empty() {
            return Err(MeterError::Empty);
        }
        if changes.iter().any(|c| c.start.0 < 0) {
            return Err(MeterError::NegativePosition);
        }
        if changes[0].start != BeatTicks(0) {
            return Err(MeterError::FirstChangeNotAtZero);
        }
        for pair in changes.windows(2) {
            if pair[1].start <= pair[0].start {
                return Err(MeterError::UnsortedOrDuplicate);
            }
        }
        Ok(Self { changes })
    }

    // Build a single-signature map
    pub fn constant(signature: TimeSignature) -> Self {
        Self {
            changes: vec![MeterChange {
                start: BeatTicks(0),
                signature,
            }],
        }
    }

    // Expose changes for persistence and display
    pub fn changes(&self) -> &[MeterChange] {
        &self.changes
    }

    // Signature governing a tick position; positions before zero use the first change
    pub fn signature_at(&self, pos: BeatTicks) -> TimeSignature {
        let idx = match self.changes.binary_search_by(|c| c.start.cmp(&pos)) {
            Ok(i) => i,
            Err(0) => 0,
            Err(i) => i - 1,
        };
        self.changes[idx].signature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Shorthand validated signature
    fn sig(n: u8, d: u8) -> TimeSignature {
        TimeSignature::new(n, d).unwrap()
    }

    // Numerator bounds are enforced inclusively
    #[test]
    fn numerator_bounds_enforced() {
        assert_eq!(
            TimeSignature::new(0, 4).unwrap_err(),
            MeterError::InvalidNumerator
        );
        assert_eq!(
            TimeSignature::new(100, 4).unwrap_err(),
            MeterError::InvalidNumerator
        );
        assert!(TimeSignature::new(1, 4).is_ok());
        assert!(TimeSignature::new(99, 4).is_ok());
    }

    // Only the five permitted denominators pass
    #[test]
    fn denominator_set_enforced() {
        for d in VALID_DENOMINATORS {
            assert!(
                TimeSignature::new(4, d).is_ok(),
                "denominator {d} must be valid"
            );
        }
        for d in [0u8, 3, 5, 6, 7, 9, 12, 32, 255] {
            assert_eq!(
                TimeSignature::new(4, d).unwrap_err(),
                MeterError::InvalidDenominator,
                "denominator {d} must be rejected"
            );
        }
    }

    // Bar length is exact in ticks for every permitted denominator
    #[test]
    fn ticks_per_bar_is_exact() {
        assert_eq!(sig(4, 4).ticks_per_bar(), BeatTicks(4 * TICKS_PER_BEAT));
        assert_eq!(sig(3, 4).ticks_per_bar(), BeatTicks(3 * TICKS_PER_BEAT));
        assert_eq!(sig(6, 8).ticks_per_bar(), BeatTicks(3 * TICKS_PER_BEAT));
        assert_eq!(
            sig(7, 16).ticks_per_bar(),
            BeatTicks(7 * TICKS_PER_BEAT / 4)
        );
        assert_eq!(sig(1, 1).ticks_per_bar(), BeatTicks(4 * TICKS_PER_BEAT));
        // The tick grid divides every permitted denominator exactly
        for d in VALID_DENOMINATORS {
            assert_eq!(TICKS_PER_BEAT * 4 % d as i64, 0);
        }
    }

    // Empty maps are rejected
    #[test]
    fn empty_map_rejected() {
        assert_eq!(MeterMap::new(vec![]).unwrap_err(), MeterError::Empty);
    }

    // The first change must anchor at tick zero
    #[test]
    fn first_change_must_be_at_zero() {
        let changes = vec![MeterChange {
            start: BeatTicks(10),
            signature: sig(4, 4),
        }];
        assert_eq!(
            MeterMap::new(changes).unwrap_err(),
            MeterError::FirstChangeNotAtZero
        );
    }

    // Negative positions are rejected explicitly
    #[test]
    fn negative_position_rejected() {
        let changes = vec![
            MeterChange {
                start: BeatTicks(0),
                signature: sig(4, 4),
            },
            MeterChange {
                start: BeatTicks(-5),
                signature: sig(3, 4),
            },
        ];
        let err = MeterMap::new(changes).unwrap_err();
        assert!(
            err == MeterError::NegativePosition || err == MeterError::UnsortedOrDuplicate,
            "negative positions must fail validation, got {err:?}"
        );
        let only_negative = vec![MeterChange {
            start: BeatTicks(-1),
            signature: sig(4, 4),
        }];
        let err = MeterMap::new(only_negative).unwrap_err();
        assert!(
            err == MeterError::NegativePosition || err == MeterError::FirstChangeNotAtZero,
            "a lone negative start must fail validation, got {err:?}"
        );
    }

    // Duplicate and unsorted positions are rejected
    #[test]
    fn duplicates_and_unsorted_rejected() {
        let dup = vec![
            MeterChange {
                start: BeatTicks(0),
                signature: sig(4, 4),
            },
            MeterChange {
                start: BeatTicks(0),
                signature: sig(3, 4),
            },
        ];
        assert_eq!(
            MeterMap::new(dup).unwrap_err(),
            MeterError::UnsortedOrDuplicate
        );
        let unsorted = vec![
            MeterChange {
                start: BeatTicks(0),
                signature: sig(4, 4),
            },
            MeterChange {
                start: BeatTicks::from_beats(8),
                signature: sig(3, 4),
            },
            MeterChange {
                start: BeatTicks::from_beats(4),
                signature: sig(7, 8),
            },
        ];
        assert_eq!(
            MeterMap::new(unsorted).unwrap_err(),
            MeterError::UnsortedOrDuplicate
        );
    }

    // Lookup returns the governing change for any position
    #[test]
    fn signature_lookup_is_deterministic() {
        let map = MeterMap::new(vec![
            MeterChange {
                start: BeatTicks(0),
                signature: sig(4, 4),
            },
            MeterChange {
                start: BeatTicks::from_beats(16),
                signature: sig(3, 4),
            },
            MeterChange {
                start: BeatTicks::from_beats(28),
                signature: sig(7, 8),
            },
        ])
        .unwrap();
        assert_eq!(map.signature_at(BeatTicks(0)), sig(4, 4));
        assert_eq!(map.signature_at(BeatTicks::from_beats(15)), sig(4, 4));
        assert_eq!(map.signature_at(BeatTicks::from_beats(16)), sig(3, 4));
        assert_eq!(map.signature_at(BeatTicks::from_beats(27)), sig(3, 4));
        assert_eq!(map.signature_at(BeatTicks::from_beats(28)), sig(7, 8));
        assert_eq!(map.signature_at(BeatTicks::from_beats(9999)), sig(7, 8));
    }

    // Serde round trip preserves the map exactly
    #[test]
    fn serde_round_trip_preserves_map() {
        let map = MeterMap::new(vec![
            MeterChange {
                start: BeatTicks(0),
                signature: sig(4, 4),
            },
            MeterChange {
                start: BeatTicks::from_beats(12),
                signature: sig(6, 8),
            },
        ])
        .unwrap();
        let json = serde_json::to_string(&map).unwrap();
        let back: MeterMap = serde_json::from_str(&json).unwrap();
        assert_eq!(back, map);
    }

    // Deserialization enforces the same invariants as construction
    #[test]
    fn serde_rejects_invalid_payloads() {
        let bad_sig = r#"{ "numerator": 0, "denominator": 4 }"#;
        assert!(serde_json::from_str::<TimeSignature>(bad_sig).is_err());
        let bad_den = r#"{ "numerator": 4, "denominator": 5 }"#;
        assert!(serde_json::from_str::<TimeSignature>(bad_den).is_err());
        let unsorted_map = r#"[
            { "start": 0, "signature": { "numerator": 4, "denominator": 4 } },
            { "start": 0, "signature": { "numerator": 3, "denominator": 4 } }
        ]"#;
        assert!(serde_json::from_str::<MeterMap>(unsorted_map).is_err());
        let empty_map = "[]";
        assert!(serde_json::from_str::<MeterMap>(empty_map).is_err());
    }
}
