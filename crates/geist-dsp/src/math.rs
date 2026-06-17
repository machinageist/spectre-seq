// =============================================================================
// File: crates/geist-dsp/src/math.rs
// Layer: DSP primitives
// Purpose: fast_tanh, poly_blep, lerp, db/linear and pitch conversions
// Status: Implemented; shared math used across oscillators, filters, and fx.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Concert-pitch anchor for MIDI-to-frequency conversion
const A4_HZ: f32 = 440.0;
const A4_MIDI_NOTE: f32 = 69.0;
const SEMITONES_PER_OCTAVE: f32 = 12.0;
const CENTS_PER_OCTAVE: f32 = 1200.0;

// Amplitude (field) decibels use 20*log10, not the 10*log10 of power
const AMPLITUDE_DB_FACTOR: f32 = 20.0;

// Smallest linear amplitude reported as finite dB; floors meters near -120 dB
const METER_FLOOR_LINEAR: f32 = 1.0e-6;

// Beyond this magnitude the tanh Pade approximant is clamped to its asymptote
const TANH_LINEAR_LIMIT: f32 = 4.0;

// Linearly interpolate between a and b by t, with t expected in [0, 1]
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

// Convert a (possibly fractional) MIDI note to frequency in Hz
#[inline]
pub fn midi_to_hz(note: f32) -> f32 {
    A4_HZ * 2.0_f32.powf((note - A4_MIDI_NOTE) / SEMITONES_PER_OCTAVE)
}

// Convert a detune in cents to a frequency multiplier
#[inline]
pub fn cents_to_ratio(cents: f32) -> f32 {
    2.0_f32.powf(cents / CENTS_PER_OCTAVE)
}

// Convert decibels to a linear gain factor
#[inline]
pub fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / AMPLITUDE_DB_FACTOR)
}

// Convert a linear gain factor to decibels, floored so silence is finite
#[inline]
pub fn linear_to_db(linear: f32) -> f32 {
    AMPLITUDE_DB_FACTOR * linear.abs().max(METER_FLOOR_LINEAR).log10()
}

// Fast, high-accuracy tanh for saturation; Pade approximant clamped to +/-1
// Error stays below ~1e-4 for |x| <= 3, the useful saturation range
#[inline]
pub fn fast_tanh(x: f32) -> f32 {
    if x.abs() >= TANH_LINEAR_LIMIT {
        return x.signum();
    }
    let x2 = x * x;
    let numerator = x * (135135.0 + x2 * (17325.0 + x2 * (378.0 + x2)));
    let denominator = 135135.0 + x2 * (62370.0 + x2 * (3150.0 + x2 * 28.0));
    (numerator / denominator).clamp(-1.0, 1.0)
}

// PolyBLEP residual that bandlimits a discontinuity at phase wrap
// `phase` is in [0, 1); `dt` is the per-sample phase increment (freq / sample_rate)
// Subtract from a naive ramp to antialias saw/square edges
#[inline]
pub fn poly_blep(phase: f32, dt: f32) -> f32 {
    if phase < dt {
        // Just after a discontinuity: rises from -1 up to 0
        let t = phase / dt;
        return t + t - t * t - 1.0;
    }
    if phase > 1.0 - dt {
        // Just before the wrap: rises from 0 up to +1
        let t = (phase - 1.0) / dt;
        return t * t + t + t + 1.0;
    }
    0.0
}

// PolyBLAMP residual that bandlimits a slope discontinuity (a ramp corner)
// Equals (1/dt) times the phase-integral of `poly_blep`; peaks at 1/3 at the corner
// Add `(slope_change / 2) * dt * poly_blamp(...)` at each corner of a piecewise-linear wave
#[inline]
pub fn poly_blamp(phase: f32, dt: f32) -> f32 {
    if phase < dt {
        // Just after the corner
        let t = phase / dt - 1.0; // [-1, 0)
        return -1.0 / 3.0 * t * t * t;
    }
    if phase > 1.0 - dt {
        // Just before the corner at wrap
        let t = (phase - 1.0) / dt + 1.0; // (0, 1]
        return 1.0 / 3.0 * t * t * t;
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32, tol: f32) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn lerp_spans_endpoints_and_midpoint() {
        assert_eq!(lerp(2.0, 6.0, 0.0), 2.0);
        assert_eq!(lerp(2.0, 6.0, 1.0), 6.0);
        assert_eq!(lerp(2.0, 6.0, 0.5), 4.0);
    }

    #[test]
    fn midi_to_hz_matches_equal_temperament() {
        assert!(close(midi_to_hz(69.0), 440.0, 1e-3));
        assert!(close(midi_to_hz(57.0), 220.0, 1e-3)); // A3, one octave down
        assert!(close(midi_to_hz(81.0), 880.0, 1e-3)); // A5, one octave up
        assert!(close(midi_to_hz(60.0), 261.6256, 1e-2)); // middle C
    }

    #[test]
    fn cents_to_ratio_is_octave_consistent() {
        assert!(close(cents_to_ratio(0.0), 1.0, 1e-6));
        assert!(close(cents_to_ratio(1200.0), 2.0, 1e-5));
        assert!(close(cents_to_ratio(-1200.0), 0.5, 1e-5));
    }

    #[test]
    fn decibel_conversions_round_trip() {
        assert!(close(db_to_linear(0.0), 1.0, 1e-6));
        assert!(close(db_to_linear(6.0206), 2.0, 1e-3));
        assert!(close(db_to_linear(-6.0206), 0.5, 1e-3));
        assert!(close(linear_to_db(1.0), 0.0, 1e-4));
        assert!(close(linear_to_db(2.0), 6.0206, 1e-3));
        // Round-trip through both directions
        assert!(close(linear_to_db(db_to_linear(-12.0)), -12.0, 1e-3));
    }

    #[test]
    fn linear_to_db_floors_silence() {
        // Zero must not produce -inf; it floors to the meter floor in dB
        let floor_db = AMPLITUDE_DB_FACTOR * METER_FLOOR_LINEAR.log10();
        assert!(linear_to_db(0.0).is_finite());
        assert!(close(linear_to_db(0.0), floor_db, 1e-3));
    }

    #[test]
    fn fast_tanh_tracks_reference() {
        for &x in &[-3.0, -2.0, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0, 3.0] {
            let reference = (x as f64).tanh() as f32;
            assert!(
                close(fast_tanh(x), reference, 1e-4),
                "fast_tanh({x}) = {} vs {reference}",
                fast_tanh(x)
            );
        }
    }

    #[test]
    fn fast_tanh_saturates_and_is_odd() {
        assert_eq!(fast_tanh(0.0), 0.0);
        assert!(close(fast_tanh(10.0), 1.0, 1e-6));
        assert!(close(fast_tanh(-10.0), -1.0, 1e-6));
        assert!(close(fast_tanh(0.7), -fast_tanh(-0.7), 1e-6));
        // Monotonic across the active range
        let mut prev = fast_tanh(-3.0);
        let mut x = -3.0;
        while x <= 3.0 {
            let y = fast_tanh(x);
            assert!(y >= prev - 1e-6, "non-monotonic at {x}");
            prev = y;
            x += 0.1;
        }
    }

    #[test]
    fn poly_blep_is_zero_away_from_edges_and_continuous() {
        let dt = 0.1;
        // Middle of the cycle needs no correction
        assert_eq!(poly_blep(0.5, dt), 0.0);
        // Region boundaries resolve to zero, so the residual is continuous
        assert_eq!(poly_blep(dt, dt), 0.0);
        assert_eq!(poly_blep(1.0 - dt, dt), 0.0);
        // Endpoints of each branch carry the full unit correction
        assert!(close(poly_blep(0.0, dt), -1.0, 1e-6));
        assert!(close(poly_blep(0.999, dt), 1.0, 2e-2));
        // A point inside the trailing branch is in (0, 1)
        let r = poly_blep(0.95, dt);
        assert!(r > 0.0 && r < 1.0, "trailing residual out of range: {r}");
    }

    #[test]
    fn poly_blamp_is_zero_away_from_corners_and_peaks_at_third() {
        let dt = 0.1;
        // No correction mid-segment
        assert_eq!(poly_blamp(0.5, dt), 0.0);
        // Returns to zero at both outer edges of the correction window
        assert_eq!(poly_blamp(dt, dt), 0.0);
        assert_eq!(poly_blamp(1.0 - dt, dt), 0.0);
        // The residual peaks at 1/3 at the corner, from both sides of the wrap
        assert!(close(poly_blamp(0.0, dt), 1.0 / 3.0, 1e-6));
        assert!(close(poly_blamp(1.0 - 1e-7, dt), 1.0 / 3.0, 1e-3));
    }
}
