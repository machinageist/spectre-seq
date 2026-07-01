// Author: Jeff
// Date: 2026-06-30
// Description: Strong musical and clock time types for core DAW models.
// Notes: Keep conversions explicit so audio and UI code do not trade raw floats.

// Absolute or relative sample position/count.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct SampleTime(pub i64);

impl SampleTime {
    pub const ZERO: Self = Self(0);

    // Wrap raw sample count.
    #[inline]
    pub const fn new(samples: i64) -> Self {
        Self(samples)
    }

    // Return raw sample count.
    #[inline]
    pub const fn raw(self) -> i64 {
        self.0
    }

    // Convert samples to seconds at a fixed sample rate.
    #[inline]
    pub fn to_seconds(self, sample_rate_hz: u32) -> Seconds {
        if sample_rate_hz == 0 {
            Seconds::ZERO
        } else {
            Seconds(self.0 as f64 / sample_rate_hz as f64)
        }
    }

    // Convert seconds to nearest sample at a fixed sample rate.
    #[inline]
    pub fn from_seconds(seconds: Seconds, sample_rate_hz: u32) -> Self {
        if sample_rate_hz == 0 || !seconds.0.is_finite() {
            Self::ZERO
        } else {
            Self((seconds.0 * sample_rate_hz as f64).round() as i64)
        }
    }

    // Convert samples to beats at a constant tempo.
    #[inline]
    pub fn to_beats(self, tempo_bpm: f64, sample_rate_hz: u32) -> BeatTime {
        if tempo_bpm <= 0.0 || sample_rate_hz == 0 || !tempo_bpm.is_finite() {
            BeatTime::ZERO
        } else {
            BeatTime(self.0 as f64 * tempo_bpm / (60.0 * sample_rate_hz as f64))
        }
    }

    // Convert beats to nearest sample at a constant tempo.
    #[inline]
    pub fn from_beats(beats: BeatTime, tempo_bpm: f64, sample_rate_hz: u32) -> Self {
        if tempo_bpm <= 0.0 || sample_rate_hz == 0 || !tempo_bpm.is_finite() || !beats.0.is_finite()
        {
            Self::ZERO
        } else {
            Self((beats.0 * 60.0 * sample_rate_hz as f64 / tempo_bpm).round() as i64)
        }
    }
}

// Musical beat position/count.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct BeatTime(pub f64);

impl BeatTime {
    pub const ZERO: Self = Self(0.0);

    // Wrap raw beat count.
    #[inline]
    pub const fn new(beats: f64) -> Self {
        Self(beats)
    }

    // Return raw beat count.
    #[inline]
    pub const fn raw(self) -> f64 {
        self.0
    }
}

// Wall-clock/audio-clock seconds.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct Seconds(pub f64);

impl Seconds {
    pub const ZERO: Self = Self(0.0);

    // Wrap raw seconds.
    #[inline]
    pub const fn new(seconds: f64) -> Self {
        Self(seconds)
    }

    // Return raw seconds.
    #[inline]
    pub const fn raw(self) -> f64 {
        self.0
    }
}

// Pulses per quarter-note tick position/count.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct PpqTick(pub i64);

impl PpqTick {
    pub const ZERO: Self = Self(0);

    // Wrap raw PPQ tick count.
    #[inline]
    pub const fn new(ticks: i64) -> Self {
        Self(ticks)
    }

    // Return raw PPQ tick count.
    #[inline]
    pub const fn raw(self) -> i64 {
        self.0
    }

    // Convert beats to nearest PPQ tick.
    #[inline]
    pub fn from_beats(beats: BeatTime, ppq: i64) -> Self {
        if ppq <= 0 || !beats.0.is_finite() {
            Self::ZERO
        } else {
            Self((beats.0 * ppq as f64).round() as i64)
        }
    }

    // Convert PPQ ticks to beats.
    #[inline]
    pub fn to_beats(self, ppq: i64) -> BeatTime {
        if ppq <= 0 {
            BeatTime::ZERO
        } else {
            BeatTime(self.0 as f64 / ppq as f64)
        }
    }
}

// One-based musical bar/beat plus zero-based tick inside the beat.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct BarBeat {
    pub bar: i32,
    pub beat: i32,
    pub tick: i32,
}

impl BarBeat {
    // Wrap raw bar/beat/tick fields.
    #[inline]
    pub const fn new(bar: i32, beat: i32, tick: i32) -> Self {
        Self { bar, beat, tick }
    }

    // Convert absolute PPQ ticks to one-based bar/beat/tick.
    pub fn from_ppq(ticks: PpqTick, ppq: i64, beats_per_bar: i32) -> Self {
        if ppq <= 0 || beats_per_bar <= 0 {
            return Self::new(1, 1, 0);
        }

        let ticks_per_bar = ppq.saturating_mul(beats_per_bar as i64);
        let clamped = ticks.0.max(0);
        let bar_index = clamped / ticks_per_bar;
        let in_bar = clamped % ticks_per_bar;
        let beat_index = in_bar / ppq;
        let tick = in_bar % ppq;

        Self::new(
            bar_index.saturating_add(1) as i32,
            beat_index.saturating_add(1) as i32,
            tick as i32,
        )
    }

    // Convert one-based bar/beat/tick to absolute PPQ ticks.
    pub fn to_ppq(self, ppq: i64, beats_per_bar: i32) -> PpqTick {
        if ppq <= 0 || beats_per_bar <= 0 {
            return PpqTick::ZERO;
        }

        let bar = (self.bar.max(1) - 1) as i64;
        let beat = (self.beat.clamp(1, beats_per_bar) - 1) as i64;
        let tick = (self.tick as i64).clamp(0, ppq - 1);
        PpqTick(bar * beats_per_bar as i64 * ppq + beat * ppq + tick)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_seconds_round_trip_at_fixed_rate() {
        let sample_rate = 48_000;
        let samples = SampleTime::new(24_000);
        let seconds = samples.to_seconds(sample_rate);

        assert_eq!(seconds, Seconds::new(0.5));
        assert_eq!(SampleTime::from_seconds(seconds, sample_rate), samples);
    }

    #[test]
    fn beat_sample_round_trip_at_tempo() {
        let sample_rate = 48_000;
        let beats = BeatTime::new(2.0);
        let samples = SampleTime::from_beats(beats, 120.0, sample_rate);

        assert_eq!(samples, SampleTime::new(48_000));
        assert_eq!(samples.to_beats(120.0, sample_rate), beats);
    }

    #[test]
    fn ppq_and_bar_beat_convert_with_meter() {
        let ppq = PpqTick::from_beats(BeatTime::new(5.5), 960);
        let bar_beat = BarBeat::from_ppq(ppq, 960, 4);

        assert_eq!(ppq, PpqTick::new(5_280));
        assert_eq!(bar_beat, BarBeat::new(2, 2, 480));
        assert_eq!(bar_beat.to_ppq(960, 4), ppq);
    }

    #[test]
    fn constructors_reject_invalid_conversion_inputs() {
        assert_eq!(
            SampleTime::from_seconds(Seconds::new(1.0), 0),
            SampleTime::ZERO
        );
        assert_eq!(
            SampleTime::from_beats(BeatTime::new(1.0), 0.0, 48_000),
            SampleTime::ZERO
        );
        assert_eq!(PpqTick::from_beats(BeatTime::new(1.0), 0), PpqTick::ZERO);
    }
}
