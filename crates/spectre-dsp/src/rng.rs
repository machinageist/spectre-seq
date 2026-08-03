// =============================================================================
// File: crates/spectre-dsp/src/rng.rs
// Layer: DSP primitives
// Purpose: Small, fast, allocation-free PRNG shared by noise and modulation
// Status: Implemented; xorshift64* with good 1D distribution.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// xorshift64* output mixing multiplier
const XORSHIFT_STAR_MULT: u64 = 0x2545_F491_4F6C_DD1D;

// Scale a 24-bit integer mantissa into the unit interval
const RNG_24_BIT_SCALE: f32 = 1.0 / 16_777_216.0;

// Deterministic, seedable generator suitable for the audio thread
#[derive(Clone, Copy, Debug)]
pub(crate) struct Rng {
    state: u64,
}

impl Rng {
    // Seed the generator, forcing a nonzero state
    pub(crate) fn new(seed: u64) -> Self {
        Self { state: seed | 1 }
    }

    // Advance and return the next 64-bit value
    #[inline]
    pub(crate) fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(XORSHIFT_STAR_MULT)
    }

    // Next sample uniformly in [-1, 1)
    #[inline]
    pub(crate) fn next_bipolar(&mut self) -> f32 {
        let bits = (self.next_u64() >> 40) as u32; // top 24 bits
        (bits as f32) * (2.0 * RNG_24_BIT_SCALE) - 1.0
    }

    // Next sample uniformly in [0, 1)
    #[inline]
    pub(crate) fn next_unit(&mut self) -> f32 {
        let bits = (self.next_u64() >> 40) as u32;
        (bits as f32) * RNG_24_BIT_SCALE
    }
}
