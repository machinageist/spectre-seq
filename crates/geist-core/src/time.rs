// Author: Jeff
// Date: 2026-07-11
// Description: Explicit time newtypes (TIME-001); beats/samples conversion lives only in TempoMap
// Notes: TICKS_PER_BEAT divides evenly by 2..=6, 8, 10, 12, 16, 32, 64 for exact grids and tuplets

use serde::{Deserialize, Serialize};

// Musical tick resolution per quarter-note beat
pub const TICKS_PER_BEAT: i64 = 960;

// Absolute engine time in samples; signed for pre-roll and count-in (OBS-AB12-REC-008 precedent)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SampleTime(pub i64);

// Nonnegative sample count/delta; distinct from an absolute SampleTime
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SampleDuration(u64);

// Musical position/duration in ticks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BeatTicks(pub i64);

// Wall-clock seconds for display and device math
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Seconds(pub f64);

// Nonzero sample rate in Hz
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SampleRate(u32);

impl SampleRate {
    // Construct a validated sample rate
    pub fn new(hz: u32) -> Option<Self> {
        if hz == 0 {
            None
        } else {
            Some(Self(hz))
        }
    }

    // Expose the raw rate
    pub fn hz(self) -> u32 {
        self.0
    }
}

impl SampleDuration {
    // Construct an exact sample count
    pub const fn new(samples: u64) -> Self {
        Self(samples)
    }

    // Expose the sample count for device and persistence boundaries
    pub const fn samples(self) -> u64 {
        self.0
    }

    // Add durations only when the result is representable
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    // Add durations, clamping at the maximum representable count
    pub fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    // Convert nonnegative finite wall time to the nearest sample; half-sample ties round up
    pub fn checked_from_seconds(seconds: Seconds, rate: SampleRate) -> Option<Self> {
        const U64_MAX_EXCLUSIVE: f64 = 18_446_744_073_709_551_616.0;

        if !seconds.0.is_finite() || seconds.0 < 0.0 {
            return None;
        }
        let rounded = (seconds.0 * f64::from(rate.hz())).round();
        if !rounded.is_finite() || rounded >= U64_MAX_EXCLUSIVE {
            None
        } else {
            Some(Self(rounded as u64))
        }
    }

    // Convert to f64 wall-clock seconds; integer conversion and division may round
    pub fn to_seconds(self, rate: SampleRate) -> Seconds {
        Seconds(self.0 as f64 / f64::from(rate.hz()))
    }
}

impl SampleTime {
    // Add a nonnegative duration only when the absolute position remains representable
    pub fn checked_add_duration(self, duration: SampleDuration) -> Option<Self> {
        i64::try_from(i128::from(self.0) + i128::from(duration.0))
            .ok()
            .map(Self)
    }

    // Add a nonnegative duration, clamping at the latest representable position
    pub fn saturating_add_duration(self, duration: SampleDuration) -> Self {
        self.checked_add_duration(duration)
            .unwrap_or(Self(i64::MAX))
    }
}

impl BeatTicks {
    // Build from whole beats, panicking when the tick count is out of range
    pub fn from_beats(beats: i64) -> Self {
        Self::checked_from_beats(beats).expect("whole-beat value exceeds BeatTicks range")
    }

    // Build from whole beats when the resulting tick count fits in i64
    pub fn checked_from_beats(beats: i64) -> Option<Self> {
        beats.checked_mul(TICKS_PER_BEAT).map(Self)
    }

    // Convert to fractional beats for display
    pub fn as_beats_f64(self) -> f64 {
        self.0 as f64 / TICKS_PER_BEAT as f64
    }
}

impl std::ops::Add for BeatTicks {
    type Output = BeatTicks;
    // Add tick durations
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for BeatTicks {
    type Output = BeatTicks;
    // Subtract tick positions
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::Add for SampleTime {
    type Output = SampleTime;
    // Add sample durations
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for SampleTime {
    type Output = SampleTime;
    // Subtract sample positions
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tick resolution must divide common grids exactly
    #[test]
    fn ticks_divide_common_grids() {
        for d in [2i64, 3, 4, 5, 6, 8, 10, 12, 16, 24, 32, 48, 64] {
            assert_eq!(TICKS_PER_BEAT % d, 0, "grid 1/{d} must be exact");
        }
    }

    // Zero sample rate is invalid
    #[test]
    fn sample_rate_rejects_zero() {
        assert!(SampleRate::new(0).is_none());
        assert_eq!(SampleRate::new(48_000).unwrap().hz(), 48_000);
    }
}
