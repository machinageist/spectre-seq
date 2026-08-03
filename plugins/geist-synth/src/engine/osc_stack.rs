// =============================================================================
// File: plugins/geist-synth/src/engine/osc_stack.rs
// Layer: synth plugin
// Purpose: 2× wavetable oscs with unison/detune
// Status: Implemented; unison wavetable oscillator + two-osc stack with mix.
// Notes: Each oscillator spreads N detuned wavetable voices; the stack blends
//        two oscillators with independent waveform and pitch offset.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use core::f32::consts::TAU;

use spectre_dsp::prelude::{cents_to_ratio, lerp, midi_to_hz, Wavetable, WavetableOsc};

// Largest unison voice count per oscillator
const MAX_UNISON: usize = 7;

// Harmonics summed when building the default saw table
const SAW_HARMONICS: usize = 64;

// Single-cycle table length; power of two for cheap wrap
const TABLE_LEN: usize = 2048;

// One oscillator voiced as a stack of detuned wavetable readers
#[derive(Clone, Copy, Debug)]
pub struct UnisonOsc {
    voices: [WavetableOsc; MAX_UNISON],
    unison: usize,
    detune_cents: f32,
    base_freq: f32,
    sample_rate: f32,
    level_comp: f32,
}

impl UnisonOsc {
    // Build a single-voice oscillator; configure then set_frequency
    pub fn new() -> Self {
        Self {
            voices: [WavetableOsc::new(); MAX_UNISON],
            unison: 1,
            detune_cents: 0.0,
            base_freq: 0.0,
            sample_rate: 48_000.0,
            level_comp: 1.0,
        }
    }

    // Number of active unison voices, clamped to [1, MAX_UNISON]
    pub fn set_unison(&mut self, voices: usize) {
        self.unison = voices.clamp(1, MAX_UNISON);
        self.level_comp = 1.0 / (self.unison as f32).sqrt();
        self.retune();
    }

    // Total detune spread across the unison stack, in cents
    pub fn set_detune(&mut self, cents: f32) {
        self.detune_cents = cents;
        self.retune();
    }

    // Set the root frequency and sample rate
    pub fn set_frequency(&mut self, base_hz: f32, sample_rate_hz: f32) {
        self.base_freq = base_hz;
        self.sample_rate = sample_rate_hz;
        self.retune();
    }

    // Reset all voice phases
    pub fn reset(&mut self) {
        for voice in &mut self.voices {
            voice.reset();
        }
    }

    // Spread the active voices symmetrically around the root frequency
    fn retune(&mut self) {
        for i in 0..self.unison {
            let offset = if self.unison == 1 {
                0.0
            } else {
                // Map voice index to [-1, 1] then scale by the spread
                let t = i as f32 / (self.unison - 1) as f32 * 2.0 - 1.0;
                t * self.detune_cents
            };
            let freq = self.base_freq * cents_to_ratio(offset);
            self.voices[i].set_frequency(freq, self.sample_rate);
        }
    }

    // Render one sample by summing the active voices from a table
    #[inline]
    pub fn render(&mut self, table: &Wavetable) -> f32 {
        let mut sum = 0.0;
        for voice in &mut self.voices[..self.unison] {
            sum += voice.next_sample(table);
        }
        sum * self.level_comp
    }
}

impl Default for UnisonOsc {
    fn default() -> Self {
        Self::new()
    }
}

// Two unison oscillators with independent waveforms, tuning, and a blend
#[derive(Clone, Debug)]
pub struct OscStack {
    osc_a: UnisonOsc,
    osc_b: UnisonOsc,
    table_a: Wavetable,
    table_b: Wavetable,
    sample_rate: f32,
    note: f32,
    a_semitones: f32,
    b_semitones: f32,
    mix: f32,
}

impl OscStack {
    // Build a stack with a sine oscillator A and a saw oscillator B
    pub fn new(sample_rate_hz: f32) -> Self {
        Self {
            osc_a: UnisonOsc::new(),
            osc_b: UnisonOsc::new(),
            table_a: Wavetable::sine(TABLE_LEN),
            table_b: band_limited_saw(TABLE_LEN, SAW_HARMONICS),
            sample_rate: sample_rate_hz,
            note: 69.0,
            a_semitones: 0.0,
            b_semitones: 0.0,
            mix: 0.5,
        }
    }

    // Set the played MIDI note and retune both oscillators
    pub fn set_note(&mut self, midi_note: f32) {
        self.note = midi_note;
        self.retune();
    }

    // Pitch offset of oscillator A in semitones
    pub fn set_osc_a_semitones(&mut self, semitones: f32) {
        self.a_semitones = semitones;
        self.retune();
    }

    // Pitch offset of oscillator B in semitones
    pub fn set_osc_b_semitones(&mut self, semitones: f32) {
        self.b_semitones = semitones;
        self.retune();
    }

    // Configure oscillator A unison voices and detune spread
    pub fn set_osc_a_unison(&mut self, voices: usize, detune_cents: f32) {
        self.osc_a.set_unison(voices);
        self.osc_a.set_detune(detune_cents);
        self.retune();
    }

    // Configure oscillator B unison voices and detune spread
    pub fn set_osc_b_unison(&mut self, voices: usize, detune_cents: f32) {
        self.osc_b.set_unison(voices);
        self.osc_b.set_detune(detune_cents);
        self.retune();
    }

    // Blend between oscillator A (0.0) and oscillator B (1.0)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    // Reset both oscillators' phases
    pub fn reset(&mut self) {
        self.osc_a.reset();
        self.osc_b.reset();
    }

    // Recompute both oscillator frequencies from the note and offsets
    fn retune(&mut self) {
        let freq_a = midi_to_hz(self.note + self.a_semitones);
        let freq_b = midi_to_hz(self.note + self.b_semitones);
        self.osc_a.set_frequency(freq_a, self.sample_rate);
        self.osc_b.set_frequency(freq_b, self.sample_rate);
    }

    // Render one blended sample
    #[inline]
    pub fn render(&mut self) -> f32 {
        let a = self.osc_a.render(&self.table_a);
        let b = self.osc_b.render(&self.table_b);
        lerp(a, b, self.mix)
    }
}

// Build a normalized band-limited sawtooth single-cycle table by additive synthesis
fn band_limited_saw(len: usize, harmonics: usize) -> Wavetable {
    let mut samples = vec![0.0f32; len];
    for (i, sample) in samples.iter_mut().enumerate() {
        let phase = i as f32 / len as f32;
        let mut acc = 0.0;
        for k in 1..=harmonics {
            acc += (TAU * k as f32 * phase).sin() / k as f32;
        }
        *sample = acc;
    }
    let peak = samples.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
    if peak > 0.0 {
        for sample in &mut samples {
            *sample /= peak;
        }
    }
    Wavetable::new(samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 48_000.0;

    fn rising_crossings(buf: &[f32]) -> usize {
        buf.windows(2).filter(|w| w[0] < 0.0 && w[1] >= 0.0).count()
    }

    #[test]
    fn single_voice_tracks_note_frequency() {
        let mut stack = OscStack::new(SAMPLE_RATE);
        stack.set_mix(0.0); // oscillator A only (sine)
        stack.set_note(69.0); // A4 = 440 Hz
        let mut buf = vec![0.0f32; SAMPLE_RATE as usize];
        for s in &mut buf {
            *s = stack.render();
        }
        let crossings = rising_crossings(&buf);
        assert!(
            (435..=445).contains(&crossings),
            "A4 crossings = {crossings}"
        );
    }

    #[test]
    fn octave_offset_doubles_frequency() {
        let mut stack = OscStack::new(SAMPLE_RATE);
        stack.set_mix(1.0); // oscillator B
        stack.set_osc_b_semitones(12.0); // up one octave
        stack.set_note(57.0); // A3 = 220 Hz -> osc B at 440 Hz
        let mut buf = vec![0.0f32; SAMPLE_RATE as usize];
        for s in &mut buf {
            *s = stack.render();
        }
        let crossings = rising_crossings(&buf);
        assert!(
            (435..=445).contains(&crossings),
            "octave-up crossings = {crossings}"
        );
    }

    #[test]
    fn unison_stays_bounded_and_level_compensated() {
        let mut stack = OscStack::new(SAMPLE_RATE);
        stack.set_mix(0.0);
        stack.set_osc_a_unison(7, 20.0);
        stack.set_note(60.0);
        let mut peak = 0.0f32;
        for _ in 0..48_000 {
            let v = stack.render();
            assert!(v.is_finite());
            peak = peak.max(v.abs());
        }
        // 1/sqrt(N) normalizes power, so the aligned-voice peak tops out near
        // sqrt(7) ~= 2.65 -- well below the uncompensated 7x
        assert!(peak < 3.0, "unison peak too hot: {peak}");
    }

    #[test]
    fn mix_selects_between_oscillators() {
        let mut a_only = OscStack::new(SAMPLE_RATE);
        a_only.set_mix(0.0);
        a_only.set_note(60.0);

        let mut b_only = OscStack::new(SAMPLE_RATE);
        b_only.set_mix(1.0);
        b_only.set_note(60.0);

        // Sine (A) vs saw (B) at the same note produce different signals
        let mut differ = false;
        for _ in 0..2_048 {
            if (a_only.render() - b_only.render()).abs() > 1e-3 {
                differ = true;
            }
        }
        assert!(differ, "A and B oscillators were indistinguishable");
    }

    #[test]
    fn reset_returns_to_start_phase() {
        let mut stack = OscStack::new(SAMPLE_RATE);
        stack.set_mix(0.0);
        stack.set_note(60.0);
        let first = stack.render();
        for _ in 0..500 {
            stack.render();
        }
        stack.reset();
        assert!((stack.render() - first).abs() < 1e-6);
    }
}
