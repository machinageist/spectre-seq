// =============================================================================
// Author: Jeff
// Date: 2026-07-31
// Description: Canonical 960-PPQ musical-time primitives.
// Notes: Float-mediated conversions preserve adjacent 960-PPQ tick resolution.
//
// File: crates/geist-timeline/src/time.rs
// Layer: timeline
// Purpose: Canonical 960-PPQ musical-time primitives
// Status: Implemented; opaque ticks, checked arithmetic, and beat quantization.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Canonical tick resolution; one quarter-note beat equals this many ticks
pub const TICKS_PER_QUARTER: u32 = 960;

// Largest tick count whose adjacent 1/960-beat step remains distinct in f64
pub const MAX_EXACT_MUSICAL_TIME_TICKS: u64 = 1_u64 << (f64::MANTISSA_DIGITS - 1);

// Largest integer represented exactly by f64
pub(crate) const MAX_EXACT_F64_INTEGER: u64 = 1_u64 << f64::MANTISSA_DIGITS;

// Nonnegative musical position or duration in canonical ticks
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MusicalTime(u64);

impl MusicalTime {
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(u64::MAX);

    // Build time from an exact tick count
    pub const fn from_ticks(ticks: u64) -> Self {
        Self(ticks)
    }

    // Read the exact tick count
    pub const fn ticks(self) -> u64 {
        self.0
    }

    // Convert ticks to beats only while adjacent tick resolution remains distinct
    pub fn try_as_beats(self) -> Option<f64> {
        (self.0 <= MAX_EXACT_MUSICAL_TIME_TICKS).then(|| self.0 as f64 / TICKS_PER_QUARTER as f64)
    }

    // Quantize beats to ticks; exact half-tick ties round upward
    pub fn try_from_beats(beats: f64) -> Option<Self> {
        if !beats.is_finite() || beats < 0.0 {
            return None;
        }
        let scaled = beats * TICKS_PER_QUARTER as f64;
        if !scaled.is_finite() || scaled > MAX_EXACT_MUSICAL_TIME_TICKS as f64 {
            return None;
        }
        Some(Self(scaled.round() as u64))
    }

    // Add time without wrapping
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        match self.0.checked_add(rhs.0) {
            Some(ticks) => Some(Self(ticks)),
            None => None,
        }
    }

    // Subtract time without crossing the origin
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        match self.0.checked_sub(rhs.0) {
            Some(ticks) => Some(Self(ticks)),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_ticks_round_trip() {
        let time = MusicalTime::from_ticks(1_920);
        assert_eq!(time.ticks(), 1_920);
        assert_eq!(time.try_as_beats(), Some(2.0));
    }

    #[test]
    fn beat_input_quantizes_to_the_nearest_tick() {
        let below_half = 100.49 / TICKS_PER_QUARTER as f64;
        let at_half = 100.5 / TICKS_PER_QUARTER as f64;
        assert_eq!(
            MusicalTime::try_from_beats(below_half).unwrap().ticks(),
            100
        );
        assert_eq!(MusicalTime::try_from_beats(at_half).unwrap().ticks(), 101);
    }

    #[test]
    fn invalid_beat_input_is_rejected() {
        assert_eq!(MusicalTime::try_from_beats(-0.001), None);
        assert_eq!(MusicalTime::try_from_beats(f64::NAN), None);
        assert_eq!(MusicalTime::try_from_beats(f64::INFINITY), None);
        assert_eq!(
            MusicalTime::try_from_beats(
                (MAX_EXACT_MUSICAL_TIME_TICKS as f64 + 960.0) / TICKS_PER_QUARTER as f64
            ),
            None
        );
    }

    #[test]
    fn checked_arithmetic_never_wraps() {
        let one = MusicalTime::from_ticks(1);
        assert_eq!(MusicalTime::MAX.checked_add(one), None);
        assert_eq!(MusicalTime::ZERO.checked_sub(one), None);
        assert_eq!(one.checked_add(one), Some(MusicalTime::from_ticks(2)));
        assert_eq!(one.checked_sub(one), Some(MusicalTime::ZERO));
    }

    #[test]
    fn beat_conversion_rejects_ticks_above_exact_musical_range() {
        let boundary = MusicalTime::from_ticks(MAX_EXACT_MUSICAL_TIME_TICKS);
        let above = MusicalTime::from_ticks(MAX_EXACT_MUSICAL_TIME_TICKS + 1);
        assert!(boundary.try_as_beats().is_some());
        assert_eq!(above.try_as_beats(), None);
    }
}
