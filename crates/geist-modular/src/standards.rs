// =============================================================================
// File: crates/geist-modular/src/standards.rs
// Layer: modular utilities
// Purpose: Rack signal contract: volt levels, Schmitt, pulses, v/oct, NaN flush
// Status: Implemented; numbers pinned to docs/modular_rack_spec.md §2.
// Notes: Gates are active at 10 V; triggers are 1 ms gates. Trigger inputs use
//        Schmitt hysteresis (fire >= 1 V, rearm <= 0.1 V) so bandlimited
//        ringing cannot retrigger. Pitch CV is 1 V/oct around a zero-volt
//        anchor: C4 (261.6256 Hz) for audio, 2 Hz (120 BPM) for LFOs/clocks.
//        Non-finite samples flush to 0 V at unstable boundaries.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Gate/trigger voltage levels (spec §2.4)
pub const GATE_V: f32 = 10.0;
// Trigger pulse length in seconds (spec §2.4)
pub const TRIGGER_SECONDS: f32 = 0.001;
// Schmitt thresholds: fire at/above high, rearm at/below low (spec §2.4)
pub const SCHMITT_HIGH_V: f32 = 1.0;
pub const SCHMITT_LOW_V: f32 = 0.1;
// Zero-volt pitch anchors for 1 V/oct conversion (spec §2.6)
pub const AUDIO_ZERO_V_HZ: f32 = 261.6256;
pub const LFO_ZERO_V_HZ: f32 = 2.0;
// Most channels one modular cable can carry (spec §3.1)
pub const POLY_MAX: usize = 16;

// Hysteretic trigger detector; `step` reports each rising fire exactly once
#[derive(Clone, Copy, Debug, Default)]
pub struct Schmitt {
    armed_high: bool,
}

impl Schmitt {
    pub fn new() -> Self {
        Self::default()
    }

    // Advance one sample; true only on the sample that crosses the fire level
    #[inline]
    pub fn step(&mut self, volts: f32) -> bool {
        if self.armed_high {
            if volts <= SCHMITT_LOW_V {
                self.armed_high = false;
            }
            false
        } else if volts >= SCHMITT_HIGH_V {
            self.armed_high = true;
            true
        } else {
            false
        }
    }

    // Whether the detector is in its fired (high) state
    #[inline]
    pub fn is_high(&self) -> bool {
        self.armed_high
    }
}

// Fixed-length trigger pulse generator; emits GATE_V for TRIGGER_SECONDS
#[derive(Clone, Copy, Debug, Default)]
pub struct Pulse {
    remaining: u32,
}

impl Pulse {
    pub fn new() -> Self {
        Self::default()
    }

    // Start (or restart) a 1 ms pulse at the given stream rate
    #[inline]
    pub fn fire(&mut self, sample_rate_hz: u32) {
        let samples = (sample_rate_hz as f32 * TRIGGER_SECONDS).round() as u32;
        self.remaining = samples.max(1);
    }

    // Advance one sample: GATE_V while the pulse runs, 0 V afterwards
    #[inline]
    pub fn step(&mut self) -> f32 {
        if self.remaining > 0 {
            self.remaining -= 1;
            GATE_V
        } else {
            0.0
        }
    }

    // Whether the pulse is still emitting
    #[inline]
    pub fn active(&self) -> bool {
        self.remaining > 0
    }
}

// Convert a 1 V/oct pitch CV to Hz around a zero-volt anchor frequency
#[inline]
pub fn volts_to_hz(volts: f32, zero_v_hz: f32) -> f32 {
    zero_v_hz * (volts).exp2()
}

// Convert a frequency in Hz to 1 V/oct pitch CV around a zero-volt anchor
#[inline]
pub fn hz_to_volts(hz: f32, zero_v_hz: f32) -> f32 {
    (hz / zero_v_hz).log2()
}

// Flush NaN/infinity to 0 V; finite values pass unchanged (spec §2.7)
#[inline]
pub fn flush_non_finite(volts: f32) -> f32 {
    if volts.is_finite() {
        volts
    } else {
        0.0
    }
}

// Flush every sample of a buffer in place
#[inline]
pub fn flush_non_finite_slice(buffer: &mut [f32]) {
    for sample in buffer {
        *sample = flush_non_finite(*sample);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn contract_numbers_match_the_spec() {
        assert_eq!(GATE_V, 10.0);
        assert_eq!(TRIGGER_SECONDS, 0.001);
        assert_eq!(SCHMITT_HIGH_V, 1.0);
        assert_eq!(SCHMITT_LOW_V, 0.1);
        assert_eq!(AUDIO_ZERO_V_HZ, 261.6256);
        assert_eq!(LFO_ZERO_V_HZ, 2.0);
        assert_eq!(POLY_MAX, 16);
    }

    #[test]
    fn schmitt_fires_once_and_rearms_below_low() {
        let mut s = Schmitt::new();
        assert!(!s.step(0.0));
        assert!(s.step(1.0), "fires at the high threshold");
        assert!(s.is_high());
        // Ringing between the thresholds neither refires nor rearms
        assert!(!s.step(0.5));
        assert!(!s.step(5.0));
        assert!(!s.step(0.2));
        assert!(s.is_high());
        // Rearm at/below low, then a new edge fires again
        assert!(!s.step(0.1));
        assert!(!s.is_high());
        assert!(!s.step(0.9), "below fire level stays armed-off");
        assert!(s.step(10.0));
    }

    #[test]
    fn pulse_runs_one_millisecond_at_gate_volts() {
        let mut p = Pulse::new();
        assert_eq!(p.step(), 0.0, "idle pulse is 0 V");
        p.fire(48_000);
        let mut high = 0;
        while p.active() {
            assert_eq!(p.step(), GATE_V);
            high += 1;
        }
        assert_eq!(high, 48, "1 ms at 48 kHz");
        assert_eq!(p.step(), 0.0);
        // Minimum one sample even at absurdly low rates
        p.fire(1);
        assert_eq!(p.step(), GATE_V);
        assert_eq!(p.step(), 0.0);
    }

    #[test]
    fn volts_per_octave_round_trips_the_anchors() {
        assert!(close(volts_to_hz(0.0, AUDIO_ZERO_V_HZ), 261.6256));
        assert!(close(volts_to_hz(1.0, AUDIO_ZERO_V_HZ), 523.2512));
        assert!(close(volts_to_hz(-1.0, AUDIO_ZERO_V_HZ), 130.8128));
        // A4 = 440 Hz sits nine semitones above C4
        assert!(close(hz_to_volts(440.0, AUDIO_ZERO_V_HZ), 9.0 / 12.0));
        assert!(close(volts_to_hz(0.0, LFO_ZERO_V_HZ), 2.0));
        assert!(close(volts_to_hz(2.0, LFO_ZERO_V_HZ), 8.0));
        let v = hz_to_volts(1234.5, AUDIO_ZERO_V_HZ);
        assert!(close(volts_to_hz(v, AUDIO_ZERO_V_HZ), 1234.5));
    }

    #[test]
    fn non_finite_values_flush_to_zero_volts() {
        assert_eq!(flush_non_finite(f32::NAN), 0.0);
        assert_eq!(flush_non_finite(f32::INFINITY), 0.0);
        assert_eq!(flush_non_finite(f32::NEG_INFINITY), 0.0);
        assert_eq!(flush_non_finite(-11.7), -11.7);
        let mut buf = [1.0, f32::NAN, -2.0, f32::INFINITY];
        flush_non_finite_slice(&mut buf);
        assert_eq!(buf, [1.0, 0.0, -2.0, 0.0]);
    }
}
