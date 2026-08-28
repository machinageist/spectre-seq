// Author: Jeff
// Date: 2026-08-25
// Description: Single FNV-1a fold shared by the offline harness, the bounce, and the bridge test
// Notes: One implementation, two named traversals. The constants are the ones already in the
//   workspace, moved rather than restated, so every existing hash assertion still passes.

// FNV-1a 64-bit offset basis. Moved verbatim from this crate's own render fold and from
// crates/spectre-audio/tests/bridge_plan.rs, the two sites this refactor rewrites.
// crates/spectre-offline/tests/harness.rs holds two more copies that stay hand-written on
// purpose: they are the independent instruments that verify this one
pub const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

// FNV-1a 64-bit prime, moved from the same two sites
pub const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

// Incremental FNV-1a fold over f32 sample bits; the only *shared* hash implementation in the
// workspace. Three hand-written folds remain by design, because two independently written
// instruments agreeing over one specimen is verification, while one shared instrument compared
// against itself is not
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampleHasher {
    state: u64,
}

impl Default for SampleHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl SampleHasher {
    pub fn new() -> Self {
        Self {
            state: FNV_OFFSET_BASIS,
        }
    }

    // Fold one sample's four little-endian bits bytes.
    // Bits, never the float: RT-003 flushes denormals to *signed* zero, so -0.0 and +0.0 are
    // distinct engine states, and NaN != NaN would make a float fold disagree with itself
    pub fn write(&mut self, sample: f32) {
        for byte in sample.to_bits().to_le_bytes() {
            self.state ^= u64::from(byte);
            self.state = self.state.wrapping_mul(FNV_PRIME);
        }
    }

    pub fn finish(self) -> u64 {
        self.state
    }
}

// Channel-major traversal over one rendered quantum: all of channel 0, then all of channel 1.
// Reproduces the existing RenderReport.hash bit for bit, which is why render_plan calls it.
// It is NOT composable across blocks — concatenating two quanta's channel-major walks is not
// the channel-major walk of the concatenation — so a multi-block render uses hash_block instead
pub fn hash_planar_quantum(output: [&[f32]; 2]) -> u64 {
    let mut hasher = SampleHasher::new();
    for sample in output[0].iter().chain(output[1].iter()) {
        hasher.write(*sample);
    }
    hasher.finish()
}

// Frame-major traversal over one interleaved block, folded into a running hasher.
// `samples` is in driver memory order — samples[frame * channels + channel] — which is exactly
// what RenderBridge::interleave writes
pub fn hash_block(hasher: &mut SampleHasher, samples: &[f32], channels: usize, frames: usize) {
    for frame in 0..frames {
        for channel in 0..channels {
            hasher.write(samples[frame * channels + channel]);
        }
    }
}

// Frame-major traversal over one planar quantum, folded into a running hasher. Produces the
// identical byte sequence hash_block produces for the same audio, without materializing an
// interleaved buffer — which is what lets a bounce that never interleaves be compared against a
// live path that always does
pub fn hash_planar_block(hasher: &mut SampleHasher, output: [&[f32]; 2], frames: usize) {
    for (left, right) in output[0][..frames].iter().zip(&output[1][..frames]) {
        hasher.write(*left);
        hasher.write(*right);
    }
}
