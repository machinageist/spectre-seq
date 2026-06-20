// =============================================================================
// File: crates/geist-dsp/src/fx/mod.rs
// Layer: DSP primitives
// Purpose: Effects (delay, chorus, flanger, phaser, distortion, EQ, reverb)
// Status: Implemented; delay + saturator + chorus + flanger + phaser +
//         distortion + EQ + convolution reverb. Shared fractional delay line.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

pub mod chorus;
pub mod delay;
pub mod distortion;
pub mod eq;
pub mod flanger;
pub mod phaser;
pub mod reverb;
pub mod saturator;

use crate::math::lerp;

// Shortest delay that still leaves room for fractional interpolation
pub(crate) const MIN_DELAY_SAMPLES: f32 = 1.0;

// Circular delay line with fractional (linear-interpolated) read
// Shared by delay, chorus, and reverb; buffer is a power of two for mask wrap
#[derive(Clone, Debug)]
pub(crate) struct DelayLine {
    buffer: Vec<f32>,
    mask: usize,
    write: usize,
    delay: f32,
}

impl DelayLine {
    pub(crate) fn new(max_delay_samples: usize) -> Self {
        let len = (max_delay_samples + 2).next_power_of_two();
        Self {
            buffer: vec![0.0; len],
            mask: len - 1,
            write: 0,
            delay: MIN_DELAY_SAMPLES,
        }
    }

    pub(crate) fn set_delay(&mut self, samples: f32) {
        let max = (self.buffer.len() - 2) as f32;
        self.delay = samples.clamp(MIN_DELAY_SAMPLES, max);
    }

    // Read the delayed sample (does not advance the line)
    #[inline]
    pub(crate) fn read(&self) -> f32 {
        let d_int = self.delay as usize;
        let frac = self.delay - d_int as f32;
        let i0 = self.write.wrapping_sub(d_int) & self.mask;
        let i1 = self.write.wrapping_sub(d_int + 1) & self.mask;
        lerp(self.buffer[i0], self.buffer[i1], frac)
    }

    // Write a sample and advance the write head
    #[inline]
    pub(crate) fn write(&mut self, x: f32) {
        self.buffer[self.write] = x;
        self.write = (self.write + 1) & self.mask;
    }

    pub(crate) fn clear(&mut self) {
        self.buffer.iter_mut().for_each(|s| *s = 0.0);
        self.write = 0;
    }
}
