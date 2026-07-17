// Author: Jeff
// Date: 2026-07-11
// Description: Piecewise-constant tempo map with single-definition beat/sample conversion (TIME-003)
// Notes: Segment anchors are accumulated without allocation during conversion

use crate::time::{BeatTicks, SampleRate, SampleTime, TICKS_PER_BEAT};
use serde::{Deserialize, Serialize};

// Tempo bounds pending a Geist-rationale ledger row (PROD-003); wide enough for observed practice
pub const MIN_BPM: f64 = 10.0;
pub const MAX_BPM: f64 = 1200.0;

// One constant-tempo region starting at a tick position
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TempoSegment {
    pub start: BeatTicks,
    pub bpm: f64,
}

// Ordered tempo segments; the first MUST start at tick zero
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TempoMap {
    segments: Vec<TempoSegment>,
}

// Tempo map construction failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TempoMapError {
    Empty,
    FirstSegmentNotAtZero,
    UnsortedSegments,
    BpmOutOfRange,
}

impl TempoMap {
    // Build a validated map from segments
    pub fn new(segments: Vec<TempoSegment>) -> Result<Self, TempoMapError> {
        if segments.is_empty() {
            return Err(TempoMapError::Empty);
        }
        if segments[0].start != BeatTicks(0) {
            return Err(TempoMapError::FirstSegmentNotAtZero);
        }
        for pair in segments.windows(2) {
            if pair[1].start <= pair[0].start {
                return Err(TempoMapError::UnsortedSegments);
            }
        }
        if segments
            .iter()
            .any(|s| !(MIN_BPM..=MAX_BPM).contains(&s.bpm) || !s.bpm.is_finite())
        {
            return Err(TempoMapError::BpmOutOfRange);
        }
        Ok(Self { segments })
    }

    // Build a single-tempo map
    pub fn constant(bpm: f64) -> Result<Self, TempoMapError> {
        Self::new(vec![TempoSegment {
            start: BeatTicks(0),
            bpm,
        }])
    }

    // Expose segments for persistence and display
    pub fn segments(&self) -> &[TempoSegment] {
        &self.segments
    }

    // Samples per tick inside one segment at a rate
    fn samples_per_tick(rate: SampleRate, bpm: f64) -> f64 {
        rate.hz() as f64 * 60.0 / (bpm * TICKS_PER_BEAT as f64)
    }

    // Accumulate exact sample anchors without allocating
    fn sample_anchor(&self, index: usize, rate: SampleRate) -> f64 {
        let mut acc = 0.0f64;
        for pair in self.segments.windows(2).take(index) {
            let ticks = (pair[1].start - pair[0].start).0 as f64;
            acc += ticks * Self::samples_per_tick(rate, pair[0].bpm);
        }
        acc
    }

    // Convert a musical position to engine samples
    pub fn ticks_to_samples(&self, pos: BeatTicks, rate: SampleRate) -> SampleTime {
        let idx = match self.segments.binary_search_by(|s| s.start.cmp(&pos)) {
            Ok(i) => i,
            Err(0) => 0,
            Err(i) => i - 1,
        };
        let seg = self.segments[idx];
        let dt = (pos - seg.start).0 as f64 * Self::samples_per_tick(rate, seg.bpm);
        SampleTime((self.sample_anchor(idx, rate) + dt).round() as i64)
    }

    // Convert engine samples to a musical position
    pub fn samples_to_ticks(&self, pos: SampleTime, rate: SampleRate) -> BeatTicks {
        let target = pos.0 as f64;
        let mut idx = 0;
        let mut anchor = 0.0f64;
        for (i, pair) in self.segments.windows(2).enumerate() {
            let ticks = (pair[1].start - pair[0].start).0 as f64;
            let next_anchor = anchor + ticks * Self::samples_per_tick(rate, pair[0].bpm);
            if next_anchor > target {
                break;
            }
            anchor = next_anchor;
            idx = i + 1;
        }
        let seg = self.segments[idx];
        let ticks = (target - anchor) / Self::samples_per_tick(rate, seg.bpm);
        BeatTicks(seg.start.0 + ticks.round() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper rate for tests
    fn rate() -> SampleRate {
        SampleRate::new(48_000).unwrap()
    }

    // 120 BPM places one beat at half a second
    #[test]
    fn constant_tempo_maps_beats_to_samples() {
        let map = TempoMap::constant(120.0).unwrap();
        let one_beat = map.ticks_to_samples(BeatTicks::from_beats(1), rate());
        assert_eq!(one_beat, SampleTime(24_000));
    }

    // Conversion inverts across a tempo change
    #[test]
    fn round_trip_across_tempo_change() {
        let map = TempoMap::new(vec![
            TempoSegment {
                start: BeatTicks(0),
                bpm: 120.0,
            },
            TempoSegment {
                start: BeatTicks::from_beats(8),
                bpm: 174.0,
            },
            TempoSegment {
                start: BeatTicks::from_beats(64),
                bpm: 140.0,
            },
        ])
        .unwrap();
        for beat in [0i64, 1, 7, 8, 9, 63, 64, 65, 400] {
            let ticks = BeatTicks::from_beats(beat);
            let samples = map.ticks_to_samples(ticks, rate());
            let back = map.samples_to_ticks(samples, rate());
            let err_ticks = (back - ticks).0.abs();
            let seg_bpm = map
                .segments()
                .iter()
                .rev()
                .find(|s| s.start <= ticks)
                .unwrap()
                .bpm;
            let ticks_per_sample = 1.0 / TempoMap::samples_per_tick(rate(), seg_bpm);
            assert!(
                (err_ticks as f64) <= ticks_per_sample.ceil(),
                "beat {beat}: round-trip error {err_ticks} ticks exceeds one sample"
            );
        }
    }

    // Validation rejects malformed maps
    #[test]
    fn validation_rejects_bad_maps() {
        assert_eq!(TempoMap::new(vec![]).unwrap_err(), TempoMapError::Empty);
        assert_eq!(
            TempoMap::new(vec![TempoSegment {
                start: BeatTicks(5),
                bpm: 120.0
            }])
            .unwrap_err(),
            TempoMapError::FirstSegmentNotAtZero
        );
        assert_eq!(
            TempoMap::constant(0.0).unwrap_err(),
            TempoMapError::BpmOutOfRange
        );
        assert_eq!(
            TempoMap::new(vec![
                TempoSegment {
                    start: BeatTicks(0),
                    bpm: 120.0
                },
                TempoSegment {
                    start: BeatTicks(0),
                    bpm: 140.0
                },
            ])
            .unwrap_err(),
            TempoMapError::UnsortedSegments
        );
    }
}
