// =============================================================================
// File: crates/geist-stacksynth/src/osc.rs
// Layer: internal synth device (generator-stack synth)
// Purpose: Analog oscillator DSP: saw/pulse/tri/sine, pulse width, hard sync,
//          phase-modulation input (spec §3.1)
// Status: S2b implemented. Waveform edges use PolyBLEP/PolyBLAMP; hard sync
//         resets naively (antialiasing of the sync edge is deferred, noted).
// Notes: Realtime-safe: Copy state, no allocation. The oscillator consumes an
//        already-resolved instantaneous frequency (see source.rs) so audio-rate
//        pitch/harmonic/shift modulation (S5) needs no change here. The slave
//        runs at freq*sync while a master phasor at freq resets it (§3.1).
//        Phase modulation is added at read time (classic FM); for the sine this
//        is exact FM, for the bandlimited waveforms it is the usual carrier
//        approximation. Negative frequency runs the phase backward (§2.3).
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::f32::consts::TAU;

use geist_dsp::prelude::{poly_blamp, poly_blep};

use crate::schema::AnalogWaveform;
use crate::source::wrap_phase;

// Slope magnitude of a unit-amplitude triangle, in value per unit phase
const TRIANGLE_SLOPE: f32 = 4.0;

// Per-voice analog oscillator state: two phase accumulators
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AnalogOsc {
    // Slave phase in [0, 1); the waveform is read from this
    phase: f32,
    // Master phase in [0, 1) driving hard-sync resets
    sync_phase: f32,
}

impl AnalogOsc {
    // Build a silent oscillator at zero phase
    pub fn new() -> Self {
        Self::default()
    }

    // Seat both phasors for a new note; `start_phase` is the seeded start (§2.3)
    pub fn reset(&mut self, start_phase: f32) {
        self.phase = wrap_phase(start_phase);
        self.sync_phase = 0.0;
    }

    // Advance one sample and read the waveform.
    // `freq_hz` is the base note frequency; the slave runs at freq*sync_ratio.
    // `read_offset` is the fixed phase offset plus any phase modulation, in
    // normalized units. `sync_ratio` >= 1.0; 1.0 disables sync.
    #[inline]
    pub fn next_sample(
        &mut self,
        freq_hz: f32,
        sync_ratio: f32,
        waveform: AnalogWaveform,
        pulse_width: f32,
        read_offset: f32,
        sample_rate_hz: f32,
    ) -> f32 {
        let master_dt = freq_hz / sample_rate_hz;
        let slave_dt = master_dt * sync_ratio.max(1.0);

        // Advance the master; on wrap, hard-reset the slave (§3.1)
        self.sync_phase += master_dt;
        if self.sync_phase >= 1.0 {
            self.sync_phase -= self.sync_phase.floor();
            if sync_ratio > 1.0 {
                self.phase = 0.0;
            }
        } else if self.sync_phase < 0.0 {
            // Negative frequency runs the master backward
            self.sync_phase -= self.sync_phase.floor();
            if sync_ratio > 1.0 {
                self.phase = 0.0;
            }
        }

        // Advance the slave, wrapping in either direction
        self.phase = wrap_phase(self.phase + slave_dt);

        // Read the waveform at the offset/modulated phase; corrections use the
        // carrier increment magnitude so they behave for negative frequency
        let read = wrap_phase(self.phase + read_offset);
        let dt = slave_dt.abs().min(0.5);
        match waveform {
            AnalogWaveform::Sawtooth => saw(read, dt),
            AnalogWaveform::Pulse => pulse(read, pulse_width.clamp(0.0, 1.0), dt),
            AnalogWaveform::Triangle => triangle(read, dt),
            AnalogWaveform::Sine => (read * TAU).sin(),
        }
    }
}

// Bandlimited rising saw in roughly [-1, 1]
#[inline]
fn saw(phase: f32, dt: f32) -> f32 {
    let naive = 2.0 * phase - 1.0;
    naive - poly_blep(phase, dt)
}

// Bandlimited variable-width pulse in roughly [-1, 1].
// Rising edge at phase 0 (+2 jump), falling edge at `width` (-2 jump).
#[inline]
fn pulse(phase: f32, width: f32, dt: f32) -> f32 {
    let naive = if phase < width { 1.0 } else { -1.0 };
    let falling = wrap_phase(phase - width);
    naive + poly_blep(phase, dt) - poly_blep(falling, dt)
}

// Bandlimited triangle in roughly [-1, 1] via PolyBLAMP corner correction
#[inline]
fn triangle(phase: f32, dt: f32) -> f32 {
    let naive = if phase < 0.5 {
        TRIANGLE_SLOPE * phase - 1.0
    } else {
        3.0 - TRIANGLE_SLOPE * phase
    };
    let half = wrap_phase(phase + 0.5);
    naive + TRIANGLE_SLOPE * dt * (poly_blamp(phase, dt) - poly_blamp(half, dt))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: f32 = 48_000.0;

    // Render n samples of a steady tone with no sync, offset, or modulation
    fn render(osc: &mut AnalogOsc, wave: AnalogWaveform, freq: f32, n: usize) -> Vec<f32> {
        (0..n)
            .map(|_| osc.next_sample(freq, 1.0, wave, 0.5, 0.0, SR))
            .collect()
    }

    #[test]
    fn every_waveform_is_finite_and_bounded() {
        for wave in [
            AnalogWaveform::Sawtooth,
            AnalogWaveform::Pulse,
            AnalogWaveform::Triangle,
            AnalogWaveform::Sine,
        ] {
            let mut osc = AnalogOsc::new();
            let buf = render(&mut osc, wave, 2_000.0, 4096);
            assert!(
                buf.iter().all(|s| s.is_finite() && s.abs() <= 1.5),
                "{wave:?} out of bounds"
            );
        }
    }

    #[test]
    fn saw_and_triangle_have_near_zero_dc() {
        // 100 Hz over 4800 samples at 4800 sr = integer cycles
        for wave in [AnalogWaveform::Sawtooth, AnalogWaveform::Triangle] {
            let mut osc = AnalogOsc::new();
            let buf: Vec<f32> = (0..4_800)
                .map(|_| osc.next_sample(100.0, 1.0, wave, 0.5, 0.0, 4_800.0))
                .collect();
            let mean: f32 = buf.iter().sum::<f32>() / buf.len() as f32;
            assert!(mean.abs() < 1e-2, "{wave:?} DC = {mean}");
        }
    }

    #[test]
    fn sine_matches_reference_within_tolerance() {
        let mut osc = AnalogOsc::new();
        let freq = 440.0;
        let mut phase = 0.0f32;
        for _ in 0..1_000 {
            let got = osc.next_sample(freq, 1.0, AnalogWaveform::Sine, 0.5, 0.0, SR);
            phase = wrap_phase(phase + freq / SR);
            let want = (phase * TAU).sin();
            assert!((got - want).abs() < 1e-4, "sine drift: {got} vs {want}");
        }
    }

    #[test]
    fn pulse_width_sets_duty_cycle() {
        // A narrow pulse spends less time high; a wide one spends more
        let duty = |width: f32| {
            let mut osc = AnalogOsc::new();
            let high = (0..4_800)
                .filter(|_| {
                    osc.next_sample(100.0, 1.0, AnalogWaveform::Pulse, width, 0.0, 4_800.0) > 0.0
                })
                .count();
            high as f32 / 4_800.0
        };
        assert!((duty(0.25) - 0.25).abs() < 0.03, "25% duty");
        assert!((duty(0.75) - 0.75).abs() < 0.03, "75% duty");
    }

    #[test]
    fn hard_sync_forces_the_master_period() {
        // At sync ratio 3 the slave restarts every master cycle; the output
        // period equals the master period, i.e. one fundamental at freq
        let mut osc = AnalogOsc::new();
        let freq = 200.0;
        let period = (SR / freq) as usize;
        let buf: Vec<f32> = (0..period * 4)
            .map(|_| osc.next_sample(freq, 3.0, AnalogWaveform::Sawtooth, 0.5, 0.0, SR))
            .collect();
        // Samples one master period apart are near-identical once settled
        for i in period..(period * 3) {
            assert!(
                (buf[i] - buf[i + period]).abs() < 0.05,
                "sync period mismatch at {i}"
            );
        }
    }

    #[test]
    fn sync_ratio_one_is_a_plain_oscillator() {
        // Sync 1.0 must not reset early; equals an un-synced saw sample-for-sample
        let mut synced = AnalogOsc::new();
        let mut plain = AnalogOsc::new();
        for _ in 0..2_000 {
            let a = synced.next_sample(330.0, 1.0, AnalogWaveform::Sawtooth, 0.5, 0.0, SR);
            let b = plain.next_sample(330.0, 1.0, AnalogWaveform::Sawtooth, 0.5, 0.0, SR);
            assert_eq!(a, b);
        }
    }

    #[test]
    fn phase_offset_shifts_the_read_point() {
        // At a frozen accumulator (freq 0), a 0.25 read offset advances the
        // sine read a quarter cycle: sin(0) = 0 becomes sin(90deg) = 1
        let mut c = AnalogOsc::new();
        c.reset(0.0);
        let s0 = c.next_sample(0.0, 1.0, AnalogWaveform::Sine, 0.5, 0.0, SR);
        let mut d = AnalogOsc::new();
        d.reset(0.0);
        let s90 = d.next_sample(0.0, 1.0, AnalogWaveform::Sine, 0.5, 0.25, SR);
        assert!((s0 - 0.0).abs() < 1e-6);
        assert!((s90 - 1.0).abs() < 1e-3, "90deg offset = {s90}");
    }

    #[test]
    fn phase_modulation_produces_fm_sidebands() {
        // A sine carrier phase-modulated by a sine at a fixed index spreads
        // energy away from the carrier bin: peak-to-mean ratio drops vs a pure
        // tone. We check the modulated signal has materially more spread.
        let carrier = 1_000.0;
        let modr = 200.0;
        let index = 2.0;
        let mut osc = AnalogOsc::new();
        let mut mod_phase = 0.0f32;
        let mut modulated = Vec::with_capacity(4_096);
        for _ in 0..4_096 {
            let pm = index * (mod_phase * TAU).sin() / TAU;
            mod_phase = wrap_phase(mod_phase + modr / SR);
            modulated.push(osc.next_sample(carrier, 1.0, AnalogWaveform::Sine, 0.5, pm, SR));
        }
        let mut pure = AnalogOsc::new();
        let clean: Vec<f32> = (0..4_096)
            .map(|_| pure.next_sample(carrier, 1.0, AnalogWaveform::Sine, 0.5, 0.0, SR))
            .collect();
        // Energy at the carrier bin via a naive Goertzel-like projection
        let carrier_energy = |buf: &[f32]| {
            let (mut re, mut im) = (0.0f32, 0.0f32);
            for (n, &s) in buf.iter().enumerate() {
                let a = TAU * carrier * n as f32 / SR;
                re += s * a.cos();
                im += s * a.sin();
            }
            (re * re + im * im).sqrt() / buf.len() as f32
        };
        // FM moves energy out of the carrier bin, so its share drops
        assert!(
            carrier_energy(&modulated) < carrier_energy(&clean) * 0.9,
            "expected FM to reduce carrier-bin energy"
        );
    }

    #[test]
    fn negative_frequency_runs_backward() {
        // A negative frequency must decrease phase, mirroring the positive tone
        let mut fwd = AnalogOsc::new();
        let mut rev = AnalogOsc::new();
        let f: Vec<f32> = (0..200)
            .map(|_| fwd.next_sample(300.0, 1.0, AnalogWaveform::Sine, 0.5, 0.0, SR))
            .collect();
        let r: Vec<f32> = (0..200)
            .map(|_| rev.next_sample(-300.0, 1.0, AnalogWaveform::Sine, 0.5, 0.0, SR))
            .collect();
        // sin(-x) = -sin(x): the reverse tone is the negation of the forward one
        for (a, b) in f.iter().zip(&r) {
            assert!((a + b).abs() < 1e-4, "not mirrored: {a} vs {b}");
        }
    }

    #[test]
    fn block_split_is_continuous() {
        let mut whole = AnalogOsc::new();
        let full = render(&mut whole, AnalogWaveform::Sawtooth, 220.0, 64);
        let mut split = AnalogOsc::new();
        let a = render(&mut split, AnalogWaveform::Sawtooth, 220.0, 32);
        let b = render(&mut split, AnalogWaveform::Sawtooth, 220.0, 32);
        for i in 0..32 {
            assert_eq!(full[i], a[i]);
            assert_eq!(full[32 + i], b[i]);
        }
    }
}
