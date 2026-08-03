// =============================================================================
// File: crates/geist-timeline/src/tempo.rs
// Layer: timeline
// Purpose: TempoMap: BPM automation, time signature changes
// Status: Implemented; piecewise-constant tempo + time-signature change points.
// Notes: Tempo is constant between change points (step, not ramp). Conversions
//        integrate sample<->beat across segments so a mid-song tempo change is
//        exact. Change points are kept sorted with a guaranteed origin at beat 0.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::time::{MusicalTime, MAX_EXACT_F64_INTEGER};
use spectre_core::transport::{DEFAULT_TIME_SIG_DEN, DEFAULT_TIME_SIG_NUM};

// Seconds in one minute; tempo math reads in beats per minute
const SECONDS_PER_MINUTE: f64 = 60.0;

// Slowest/fastest tempo the map will store; guards the divisor
const MIN_BPM: f64 = 1.0;
const MAX_BPM: f64 = 1_000.0;

// A tempo value taking effect at a beat position
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoPoint {
    pub beat: f64,
    pub bpm: f64,
}

// A time signature taking effect at a beat position
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimeSigPoint {
    pub beat: f64,
    pub numerator: u16,
    pub denominator: u16,
}

// Sample-rate-aware map of tempo and time-signature changes over the timeline
#[derive(Clone, Debug)]
pub struct TempoMap {
    sample_rate: f64,
    // Sorted by beat; always non-empty with an entry at beat 0
    tempos: Vec<TempoPoint>,
    // Sorted by beat; always non-empty with an entry at beat 0
    time_sigs: Vec<TimeSigPoint>,
}

impl TempoMap {
    // Build a constant-tempo map in common time
    pub fn new(sample_rate_hz: u32, bpm: f64) -> Self {
        Self {
            sample_rate: sample_rate_hz.max(1) as f64,
            tempos: vec![TempoPoint {
                beat: 0.0,
                bpm: bpm.clamp(MIN_BPM, MAX_BPM),
            }],
            time_sigs: vec![TimeSigPoint {
                beat: 0.0,
                numerator: DEFAULT_TIME_SIG_NUM,
                denominator: DEFAULT_TIME_SIG_DEN,
            }],
        }
    }

    // Samples per beat at a given tempo
    #[inline]
    fn samples_per_beat(&self, bpm: f64) -> f64 {
        SECONDS_PER_MINUTE * self.sample_rate / bpm
    }

    // Insert or replace the tempo at a beat, keeping the list sorted
    pub fn set_tempo(&mut self, beat: f64, bpm: f64) {
        let beat = beat.max(0.0);
        let bpm = bpm.clamp(MIN_BPM, MAX_BPM);
        match self.tempos.iter().position(|p| p.beat >= beat) {
            Some(i) if self.tempos[i].beat == beat => self.tempos[i].bpm = bpm,
            Some(i) => self.tempos.insert(i, TempoPoint { beat, bpm }),
            None => self.tempos.push(TempoPoint { beat, bpm }),
        }
    }

    // Insert or replace the time signature at a beat, keeping the list sorted
    pub fn set_time_signature(&mut self, beat: f64, numerator: u16, denominator: u16) {
        let beat = beat.max(0.0);
        let point = TimeSigPoint {
            beat,
            numerator,
            denominator,
        };
        match self.time_sigs.iter().position(|p| p.beat >= beat) {
            Some(i) if self.time_sigs[i].beat == beat => self.time_sigs[i] = point,
            Some(i) => self.time_sigs.insert(i, point),
            None => self.time_sigs.push(point),
        }
    }

    // Tempo in effect at a beat position
    pub fn tempo_at(&self, beat: f64) -> f64 {
        self.tempos
            .iter()
            .rev()
            .find(|p| p.beat <= beat)
            .map(|p| p.bpm)
            .unwrap_or(self.tempos[0].bpm)
    }

    // Time signature in effect at a beat position
    pub fn time_signature_at(&self, beat: f64) -> (u16, u16) {
        self.time_sigs
            .iter()
            .rev()
            .find(|p| p.beat <= beat)
            .map(|p| (p.numerator, p.denominator))
            .unwrap_or((self.time_sigs[0].numerator, self.time_sigs[0].denominator))
    }

    // Convert a beat position to an absolute sample position, integrating tempo
    pub fn beats_to_samples(&self, beat: f64) -> f64 {
        let beat = beat.max(0.0);
        let mut samples = 0.0;
        for (i, p) in self.tempos.iter().enumerate() {
            let seg_start = p.beat;
            if beat <= seg_start {
                break;
            }
            let seg_end = self
                .tempos
                .get(i + 1)
                .map(|n| n.beat)
                .unwrap_or(f64::INFINITY);
            let span = beat.min(seg_end) - seg_start;
            samples += span * self.samples_per_beat(p.bpm);
            if beat <= seg_end {
                break;
            }
        }
        samples
    }

    // Convert an absolute sample position to a beat position, integrating tempo
    pub fn samples_to_beats(&self, sample: f64) -> f64 {
        let sample = sample.max(0.0);
        let mut acc = 0.0;
        for (i, p) in self.tempos.iter().enumerate() {
            let seg_end_beat = self
                .tempos
                .get(i + 1)
                .map(|n| n.beat)
                .unwrap_or(f64::INFINITY);
            let spb = self.samples_per_beat(p.bpm);
            let seg_samples = (seg_end_beat - p.beat) * spb;
            if sample <= acc + seg_samples {
                return p.beat + (sample - acc) / spb;
            }
            acc += seg_samples;
        }
        self.tempos.last().map(|p| p.beat).unwrap_or(0.0)
    }

    // Convert canonical musical time to a rounded absolute sample position
    pub fn musical_time_to_samples(&self, time: MusicalTime) -> Option<u64> {
        rounded_exact_u64(self.beats_to_samples(time.try_as_beats()?))
    }

    // Convert an absolute sample position to canonical musical time
    pub fn samples_to_musical_time(&self, sample: u64) -> Option<MusicalTime> {
        if sample > MAX_EXACT_F64_INTEGER {
            return None;
        }
        MusicalTime::try_from_beats(self.samples_to_beats(sample as f64))
    }
}

// Round a scalar only where every integer is exactly representable by f64
fn rounded_exact_u64(value: f64) -> Option<u64> {
    if !value.is_finite() || value < 0.0 || value > MAX_EXACT_F64_INTEGER as f64 {
        return None;
    }
    Some(value.round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectre_core::time::{MusicalTime, MAX_EXACT_MUSICAL_TIME_TICKS};

    const SR: u32 = 48_000;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn constant_tempo_converts_linearly() {
        let map = TempoMap::new(SR, 120.0);
        // 120 BPM at 48 kHz: one beat = 24000 samples
        assert!(close(map.beats_to_samples(1.0), 24_000.0));
        assert!(close(map.beats_to_samples(4.0), 96_000.0));
        assert!(close(map.samples_to_beats(24_000.0), 1.0));
        assert!(close(map.samples_to_beats(96_000.0), 4.0));
    }

    #[test]
    fn origin_maps_to_zero() {
        let map = TempoMap::new(SR, 140.0);
        assert!(close(map.beats_to_samples(0.0), 0.0));
        assert!(close(map.samples_to_beats(0.0), 0.0));
    }

    #[test]
    fn mid_timeline_tempo_change_integrates() {
        let mut map = TempoMap::new(SR, 120.0);
        map.set_tempo(4.0, 60.0); // halve tempo at beat 4
                                  // First 4 beats at 120: 4 * 24000 = 96000 samples
                                  // Next 4 beats at 60: 4 * 48000 = 192000 samples
        assert!(close(map.beats_to_samples(4.0), 96_000.0));
        assert!(close(map.beats_to_samples(8.0), 288_000.0));
        // Round-trip back to beats
        assert!(close(map.samples_to_beats(288_000.0), 8.0));
        assert!(close(map.samples_to_beats(96_000.0), 4.0));
        // A point inside the slow segment
        assert!(close(map.beats_to_samples(6.0), 96_000.0 + 96_000.0));
    }

    #[test]
    fn tempo_at_reads_active_segment() {
        let mut map = TempoMap::new(SR, 120.0);
        map.set_tempo(8.0, 90.0);
        assert!(close(map.tempo_at(0.0), 120.0));
        assert!(close(map.tempo_at(7.9), 120.0));
        assert!(close(map.tempo_at(8.0), 90.0));
        assert!(close(map.tempo_at(100.0), 90.0));
    }

    #[test]
    fn setting_tempo_at_existing_beat_replaces() {
        let mut map = TempoMap::new(SR, 120.0);
        map.set_tempo(0.0, 100.0); // replace origin tempo
        assert!(close(map.tempo_at(0.0), 100.0));
        assert!(close(
            map.beats_to_samples(1.0),
            SECONDS_PER_MINUTE * SR as f64 / 100.0
        ));
    }

    #[test]
    fn repeated_set_at_same_beat_does_not_accumulate() {
        // Changing BPM (always at beat 0) must replace the origin point, not
        // append; otherwise the map would grow without bound as tempo is tweaked
        let mut map = TempoMap::new(SR, 120.0);
        for bpm in 100..200 {
            map.set_tempo(0.0, bpm as f64);
        }
        assert_eq!(
            map.tempos.len(),
            1,
            "origin tempo must be replaced, not appended"
        );
        assert!(close(map.tempo_at(0.0), 199.0));
    }

    #[test]
    fn time_signature_changes_are_looked_up() {
        let mut map = TempoMap::new(SR, 120.0);
        assert_eq!(map.time_signature_at(0.0), (4, 4));
        map.set_time_signature(8.0, 3, 4);
        assert_eq!(map.time_signature_at(7.9), (4, 4));
        assert_eq!(map.time_signature_at(8.0), (3, 4));
        assert_eq!(map.time_signature_at(50.0), (3, 4));
    }

    #[test]
    fn tempo_points_stay_sorted_when_inserted_out_of_order() {
        let mut map = TempoMap::new(SR, 120.0);
        map.set_tempo(16.0, 80.0);
        map.set_tempo(8.0, 100.0);
        // Conversions remain monotonic, proving the ordering held
        let s8 = map.beats_to_samples(8.0);
        let s16 = map.beats_to_samples(16.0);
        let s24 = map.beats_to_samples(24.0);
        assert!(s8 < s16 && s16 < s24);
    }

    #[test]
    fn musical_time_converts_at_constant_tempo() {
        let map = TempoMap::new(SR, 120.0);
        assert_eq!(
            map.musical_time_to_samples(MusicalTime::from_ticks(960)),
            Some(24_000)
        );
        assert_eq!(
            map.samples_to_musical_time(96_000),
            Some(MusicalTime::from_ticks(3_840))
        );
    }

    #[test]
    fn musical_time_conversion_integrates_tempo_changes() {
        let mut map = TempoMap::new(SR, 120.0);
        map.set_tempo(4.0, 60.0);
        let beat_eight = MusicalTime::from_ticks(8 * 960);
        assert_eq!(map.musical_time_to_samples(beat_eight), Some(288_000));
        assert_eq!(map.samples_to_musical_time(288_000), Some(beat_eight));
    }

    #[test]
    fn musical_time_sample_round_trip_is_within_one_tick() {
        let map = TempoMap::new(SR, 137.0);
        let original = MusicalTime::from_ticks(12_345);
        let samples = map.musical_time_to_samples(original).unwrap();
        let recovered = map.samples_to_musical_time(samples).unwrap();
        assert!(original.ticks().abs_diff(recovered.ticks()) <= 1);
    }

    #[test]
    fn zero_sample_rate_is_not_used_as_a_divisor() {
        let map = TempoMap::new(0, 120.0);
        assert_eq!(
            map.musical_time_to_samples(MusicalTime::from_ticks(960)),
            Some(1)
        );
    }

    #[test]
    fn unrepresentable_conversions_are_rejected() {
        let sample_heavy = TempoMap::new(u32::MAX, 1.0);
        assert_eq!(sample_heavy.musical_time_to_samples(MusicalTime::MAX), None);

        let tick_heavy = TempoMap::new(1, 1_000.0);
        assert_eq!(tick_heavy.samples_to_musical_time(u64::MAX), None);
    }

    #[test]
    fn conversion_enforces_sample_and_musical_precision_boundaries() {
        let identity = TempoMap::new(1_920, 120.0);
        let tick_boundary = MusicalTime::from_ticks(MAX_EXACT_MUSICAL_TIME_TICKS);
        let above_tick_boundary = MusicalTime::from_ticks(MAX_EXACT_MUSICAL_TIME_TICKS + 1);

        assert_eq!(
            identity.musical_time_to_samples(tick_boundary),
            Some(MAX_EXACT_MUSICAL_TIME_TICKS)
        );
        assert_eq!(identity.musical_time_to_samples(above_tick_boundary), None);

        let two_samples_per_tick = TempoMap::new(3_840, 120.0);
        assert_eq!(
            two_samples_per_tick.samples_to_musical_time(MAX_EXACT_F64_INTEGER),
            Some(tick_boundary)
        );
        assert_eq!(
            two_samples_per_tick.samples_to_musical_time(MAX_EXACT_F64_INTEGER + 1),
            None
        );
    }
}
