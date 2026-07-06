// =============================================================================
// File: crates/geist-stacksynth/src/source.rs
// Layer: internal synth device (generator-stack synth)
// Purpose: Shared frequency/phase math for every sound source (spec §2.3)
// Status: S2a implemented; pure math, no oscillator state yet.
// Notes: Realtime-safe: no allocation, no branches on NaN. The instantaneous
//        frequency folds pitch (exponential), harmonic (linear multiply), and
//        shift (linear Hz add) exactly as §7.2 requires for FM techniques.
//        Per-note random phase is drawn from a deterministic hash so offline
//        bounces reproduce (spec §3.2, §15.1).
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_dsp::prelude::midi_to_hz;

use crate::schema::CommonGenParams;

// Per-sample audio-rate modulation offsets applied to the frequency terms.
// Values are summed route contributions resolved by the render loop (S5);
// at S2 they are zero unless a test drives them.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FreqMod {
    // Added to pitch in semitones before the exponential (exponential FM)
    pub pitch_semis: f32,
    // Added to the harmonic multiplier (linear FM)
    pub harmonic: f32,
    // Added to shift in Hz (linear FM)
    pub shift_hz: f32,
}

// Convert a played MIDI note plus master transpose to the source base frequency
#[inline]
pub fn base_hz(note: f32, master_pitch_semis: f32) -> f32 {
    midi_to_hz(note + master_pitch_semis)
}

// Instantaneous oscillator frequency in Hz for a sound source (spec §2.3, §7.2).
// keytracked term = base * 2^((pitch+mod)/12) * (harmonic+mod); shift adds Hz.
// harmonic 0 removes keytracking, leaving the shift term (may be negative).
#[inline]
pub fn instantaneous_hz(base_hz: f32, common: &CommonGenParams, m: &FreqMod) -> f32 {
    let ratio = exp2_semitones(common.pitch_semis + m.pitch_semis);
    base_hz * ratio * (common.harmonic + m.harmonic) + (common.shift_hz + m.shift_hz)
}

// Per-sample phase increment for a (possibly negative) frequency.
// Negative frequency yields a negative increment so the phasor runs backward,
// which the spec explicitly allows (§2.3).
#[inline]
pub fn phase_increment(freq_hz: f32, sample_rate_hz: f32) -> f32 {
    freq_hz / sample_rate_hz
}

// Wrap a phase into [0, 1) for any real input, including large negatives
#[inline]
pub fn wrap_phase(phase: f32) -> f32 {
    let w = phase - phase.floor();
    // floor() of a already-integer negative can round-trip to exactly 1.0
    if w >= 1.0 {
        0.0
    } else {
        w
    }
}

// Fixed start-phase offset in normalized units from the degrees parameter
#[inline]
pub fn phase_offset_norm(common: &CommonGenParams) -> f32 {
    wrap_phase(common.phase_offset_deg / 360.0)
}

// Deterministic per-note random start phase in [0, random_range) normalized.
// Keyed on note, unison sub-voice, and a patch seed so a given render is
// reproducible while still differing per note and per unison voice (§2.3).
#[inline]
pub fn random_phase_norm(common: &CommonGenParams, note: u8, voice: u16, seed: u64) -> f32 {
    if common.phase_random_deg <= 0.0 {
        return 0.0;
    }
    let key = mix_seed(seed, note, voice);
    let unit = (key >> 40) as f32 / (1u64 << 24) as f32; // 24-bit fraction in [0,1)
    // Range is a span, not a phase position: 360deg = a full cycle, so clamp
    // to [0,1] rather than wrapping (which would fold a full cycle to zero)
    let range = (common.phase_random_deg / 360.0).clamp(0.0, 1.0);
    unit * range
}

// Combined start phase for a voice: fixed offset plus seeded randomization
#[inline]
pub fn start_phase(common: &CommonGenParams, note: u8, voice: u16, seed: u64) -> f32 {
    wrap_phase(phase_offset_norm(common) + random_phase_norm(common, note, voice, seed))
}

// 2^x with x in semitones/12 folded in; kept explicit for readability
#[inline]
fn exp2_semitones(semitones: f32) -> f32 {
    (semitones / 12.0).exp2()
}

// SplitMix64-style avalanche of the seed with note and sub-voice folded in.
// Deterministic and allocation-free; used only for reproducible randomization.
#[inline]
fn mix_seed(seed: u64, note: u8, voice: u16) -> u64 {
    let mut z = seed
        .wrapping_add(u64::from(note).wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add(u64::from(voice).wrapping_mul(0xD1B5_4A32_D192_ED03));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Build common params with a given harmonic/shift, everything else neutral
    fn common(harmonic: f32, shift_hz: f32) -> CommonGenParams {
        CommonGenParams {
            harmonic,
            shift_hz,
            ..CommonGenParams::default()
        }
    }

    #[test]
    fn a4_base_frequency_is_concert_pitch() {
        let hz = base_hz(69.0, 0.0);
        assert!((hz - 440.0).abs() < 1e-3, "A4 = {hz}");
    }

    #[test]
    fn master_pitch_transposes_an_octave() {
        let up = base_hz(69.0, 12.0);
        assert!((up - 880.0).abs() < 1e-2, "A5 = {up}");
    }

    #[test]
    fn twelve_semitones_doubles_frequency() {
        let mut c = common(1.0, 0.0);
        c.pitch_semis = 12.0;
        let hz = instantaneous_hz(440.0, &c, &FreqMod::default());
        assert!((hz - 880.0).abs() < 1e-2, "octave up = {hz}");
    }

    #[test]
    fn harmonic_multiplies_frequency() {
        let c = common(4.0, 0.0);
        let hz = instantaneous_hz(100.0, &c, &FreqMod::default());
        assert!((hz - 400.0).abs() < 1e-3, "x4 harmonic = {hz}");
    }

    #[test]
    fn zero_harmonic_disables_keytracking_leaving_shift() {
        // With harmonic 0 the keytracked term vanishes; frequency = shift only
        let c = common(0.0, 250.0);
        let hz = instantaneous_hz(440.0, &c, &FreqMod::default());
        assert!((hz - 250.0).abs() < 1e-3, "shift-only = {hz}");
    }

    #[test]
    fn negative_shift_can_drive_frequency_below_zero() {
        // Spec allows negative frequency; the math must not clamp at zero
        let c = common(1.0, -600.0);
        let hz = instantaneous_hz(440.0, &c, &FreqMod::default());
        assert!(hz < 0.0, "expected negative frequency, got {hz}");
        assert!(phase_increment(hz, 48_000.0) < 0.0);
    }

    #[test]
    fn pitch_mod_is_exponential() {
        // +12 semitones of modulation doubles frequency (exponential FM)
        let c = common(1.0, 0.0);
        let m = FreqMod {
            pitch_semis: 12.0,
            ..FreqMod::default()
        };
        let hz = instantaneous_hz(440.0, &c, &m);
        assert!((hz - 880.0).abs() < 1e-2, "exp FM octave = {hz}");
    }

    #[test]
    fn shift_mod_is_linear_hz() {
        let c = common(1.0, 0.0);
        let m = FreqMod {
            shift_hz: 55.0,
            ..FreqMod::default()
        };
        let hz = instantaneous_hz(440.0, &c, &m);
        assert!((hz - 495.0).abs() < 1e-3, "linear FM = {hz}");
    }

    #[test]
    fn wrap_phase_handles_large_negatives() {
        assert!((wrap_phase(-0.25) - 0.75).abs() < 1e-6);
        assert!((wrap_phase(-3.5) - 0.5).abs() < 1e-6);
        assert_eq!(wrap_phase(0.0), 0.0);
        assert!(wrap_phase(2.0) < 1e-6);
    }

    #[test]
    fn phase_offset_maps_degrees_to_normalized() {
        let mut c = common(1.0, 0.0);
        c.phase_offset_deg = 90.0;
        assert!((phase_offset_norm(&c) - 0.25).abs() < 1e-6);
    }

    #[test]
    fn random_phase_is_zero_when_range_is_zero() {
        let c = common(1.0, 0.0);
        assert_eq!(random_phase_norm(&c, 60, 0, 1), 0.0);
    }

    #[test]
    fn random_phase_stays_within_range_and_is_deterministic() {
        let mut c = common(1.0, 0.0);
        c.phase_random_deg = 180.0; // half a cycle
        let range = 0.5;
        for note in 0u8..=127 {
            for voice in 0u16..8 {
                let a = random_phase_norm(&c, note, voice, 42);
                let b = random_phase_norm(&c, note, voice, 42);
                assert_eq!(a, b, "not reproducible for note {note} voice {voice}");
                assert!((0.0..range).contains(&a), "out of range: {a}");
            }
        }
    }

    #[test]
    fn random_phase_differs_across_notes_and_voices() {
        let mut c = common(1.0, 0.0);
        c.phase_random_deg = 360.0;
        let a = random_phase_norm(&c, 60, 0, 7);
        let b = random_phase_norm(&c, 61, 0, 7);
        let d = random_phase_norm(&c, 60, 1, 7);
        assert_ne!(a, b, "adjacent notes should differ");
        assert_ne!(a, d, "unison sub-voices should differ");
    }

    #[test]
    fn start_phase_combines_offset_and_random_within_unit() {
        let mut c = common(1.0, 0.0);
        c.phase_offset_deg = 270.0;
        c.phase_random_deg = 180.0;
        let p = start_phase(&c, 64, 2, 99);
        assert!((0.0..1.0).contains(&p), "phase out of unit interval: {p}");
    }
}
