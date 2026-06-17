// =============================================================================
// File: crates/geist-dsp/src/fx/eq.rs
// Layer: DSP primitives
// Purpose: parametric EQ (chains biquads)
// Status: Implemented; cascade of independently configured biquad bands.
// Notes: Thin container over Biquad; configure each band via band_mut, then
//        process samples through the series chain. Bands allocated once.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use crate::filter::biquad::Biquad;

// Series cascade of biquad bands forming a multi-band parametric EQ
#[derive(Clone, Debug)]
pub struct ParametricEq {
    bands: Vec<Biquad>,
}

impl ParametricEq {
    // Allocate an EQ with `band_count` identity bands
    pub fn new(band_count: usize) -> Self {
        Self {
            bands: vec![Biquad::new(); band_count.max(1)],
        }
    }

    // Number of bands in the chain
    pub fn band_count(&self) -> usize {
        self.bands.len()
    }

    // Mutable access to one band for configuration via the Biquad API
    pub fn band_mut(&mut self, index: usize) -> Option<&mut Biquad> {
        self.bands.get_mut(index)
    }

    // Clear delay state in every band
    pub fn reset(&mut self) {
        for band in &mut self.bands {
            band.reset();
        }
    }

    // Process one sample through the series chain
    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        let mut y = x;
        for band in &mut self.bands {
            y = band.process_sample(y);
        }
        y
    }

    // Process a buffer in place
    pub fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 48_000.0;

    // Steady-state amplitude response at a sine frequency
    fn magnitude_at(eq: &mut ParametricEq, test_hz: f32) -> f32 {
        let total = 48_000usize;
        let settle = 24_000usize;
        let mut sum_sq = 0.0f64;
        let mut count = 0u32;
        for n in 0..total {
            let phase = core::f32::consts::TAU * test_hz * n as f32 / SAMPLE_RATE;
            let out = eq.process_sample(phase.sin());
            if n >= settle {
                sum_sq += (out as f64) * (out as f64);
                count += 1;
            }
        }
        let out_rms = (sum_sq / count as f64).sqrt() as f32;
        out_rms / core::f32::consts::FRAC_1_SQRT_2
    }

    #[test]
    fn unconfigured_eq_is_transparent() {
        let mut eq = ParametricEq::new(4);
        for x in [-0.5, 0.0, 0.2, 1.0] {
            assert_eq!(eq.process_sample(x), x);
        }
    }

    #[test]
    fn single_band_boosts_its_frequency() {
        let mut eq = ParametricEq::new(3);
        eq.band_mut(0)
            .unwrap()
            .set_peaking(2_000.0, 3.0, 6.0, SAMPLE_RATE);
        let g = magnitude_at(&mut eq, 2_000.0);
        assert!((g - 1.995).abs() < 0.15, "boost = {g}");
    }

    #[test]
    fn cascade_shapes_multiple_bands() {
        let mut eq = ParametricEq::new(3);
        eq.band_mut(0)
            .unwrap()
            .set_peaking(500.0, 3.0, 6.0, SAMPLE_RATE);
        eq.band_mut(1)
            .unwrap()
            .set_peaking(5_000.0, 3.0, 6.0, SAMPLE_RATE);

        let low = magnitude_at(&mut eq, 500.0);
        let mid = magnitude_at(&mut eq, 1_600.0);
        let high = magnitude_at(&mut eq, 5_000.0);
        assert!(low > 1.6, "low band = {low}");
        assert!(high > 1.6, "high band = {high}");
        assert!((0.8..1.4).contains(&mid), "between bands = {mid}");
    }

    #[test]
    fn highpass_band_blocks_dc() {
        let mut eq = ParametricEq::new(2);
        eq.band_mut(0).unwrap().set_highpass(
            1_000.0,
            core::f32::consts::FRAC_1_SQRT_2,
            SAMPLE_RATE,
        );
        let mut dc = 0.0;
        for _ in 0..20_000 {
            dc = eq.process_sample(1.0);
        }
        assert!(dc.abs() < 1e-3, "DC leaked: {dc}");
    }

    #[test]
    fn reset_matches_fresh_chain() {
        let mut used = ParametricEq::new(2);
        used.band_mut(0)
            .unwrap()
            .set_peaking(800.0, 2.0, 4.0, SAMPLE_RATE);
        for _ in 0..500 {
            used.process_sample(0.7);
        }
        used.reset();

        let mut fresh = ParametricEq::new(2);
        fresh
            .band_mut(0)
            .unwrap()
            .set_peaking(800.0, 2.0, 4.0, SAMPLE_RATE);
        assert_eq!(used.process_sample(0.5), fresh.process_sample(0.5));
    }

    #[test]
    fn band_access_is_bounds_checked() {
        let mut eq = ParametricEq::new(2);
        assert!(eq.band_mut(0).is_some());
        assert!(eq.band_mut(1).is_some());
        assert!(eq.band_mut(2).is_none());
        assert_eq!(eq.band_count(), 2);
    }
}
