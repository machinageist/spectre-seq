// =============================================================================
// File: crates/geist-audio-backend/src/bridge.rs
// Layer: audio I/O
// Purpose: BlockProcessor trait and BlockBridge size/layout adapter
// Status: Implemented; output path. Duplex input bridging lands with capture.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use crate::backend::RenderCallback;

// Fixed-block, channel-major processor the bridge drives once per block
// `output` is channels * frames laid out channel-major; the executor fills it
// Must not allocate, lock, block, or panic; this runs on the audio thread
pub trait BlockProcessor: Send {
    // Render exactly `frames` (== the bridge block size) into channel-major `output`
    // `input` is channel-major of the same shape, or empty when there is no capture
    fn process_block(&mut self, input: &[f32], output: &mut [f32], channels: usize, frames: usize);
}

// Adapts a backend's arbitrary-size interleaved callback to fixed channel-major blocks
// Backends honor a fixed buffer size only sometimes, so the bridge owns the carry
// All scratch is sized once on the app thread; render() never allocates
pub struct BlockBridge {
    inner: Box<dyn BlockProcessor>,
    channels: usize,
    block_frames: usize,
    // One block of channel-major output, drained frame by frame across callbacks
    out_block: Vec<f32>,
    // Next frame to emit from out_block; == block_frames means the block is spent
    out_pos: usize,
}

impl BlockBridge {
    // Build a bridge driving `inner` at a fixed block size for `channels`
    pub fn new(inner: Box<dyn BlockProcessor>, channels: usize, block_frames: usize) -> Self {
        Self {
            inner,
            channels,
            block_frames,
            out_block: vec![0.0; channels * block_frames],
            // Force the first frame to pull a fresh block
            out_pos: block_frames,
        }
    }
}

impl RenderCallback for BlockBridge {
    // Fill an interleaved output buffer of any frame count from fixed blocks
    fn render(&mut self, _input: &[f32], output: &mut [f32], channels: usize) {
        debug_assert_eq!(channels, self.channels, "callback channel count drifted");
        for frame in output.chunks_exact_mut(self.channels) {
            // Refill the block when the carry is exhausted
            if self.out_pos >= self.block_frames {
                self.inner.process_block(
                    &[],
                    &mut self.out_block,
                    self.channels,
                    self.block_frames,
                );
                self.out_pos = 0;
            }
            // Interleave one frame out of the channel-major block
            let pos = self.out_pos;
            for (ch, sample) in frame.iter_mut().enumerate() {
                *sample = self.out_block[ch * self.block_frames + pos];
            }
            self.out_pos += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Emits a per-channel ramp tagged with a monotonic global frame index
    // ch 0 gets frame index n; ch 1 gets n + 1000 so channels stay distinguishable
    struct RampProcessor {
        next_frame: u64,
    }

    impl BlockProcessor for RampProcessor {
        fn process_block(
            &mut self,
            input: &[f32],
            output: &mut [f32],
            channels: usize,
            frames: usize,
        ) {
            assert!(input.is_empty(), "output-only bridge passes no input");
            assert_eq!(output.len(), channels * frames);
            for ch in 0..channels {
                for f in 0..frames {
                    let n = self.next_frame + f as u64;
                    output[ch * frames + f] = n as f32 + (ch as f32) * 1000.0;
                }
            }
            self.next_frame += frames as u64;
        }
    }

    // Deinterleave a frame from an interleaved buffer
    fn sample(buf: &[f32], channels: usize, frame: usize, ch: usize) -> f32 {
        buf[frame * channels + ch]
    }

    #[test]
    fn bridge_interleaves_blocks_correctly() {
        let mut bridge = BlockBridge::new(Box::new(RampProcessor { next_frame: 0 }), 2, 4);
        let mut out = [0.0f32; 8]; // 4 frames, stereo, exactly one block
        bridge.render(&[], &mut out, 2);
        for f in 0..4 {
            assert_eq!(sample(&out, 2, f, 0), f as f32);
            assert_eq!(sample(&out, 2, f, 1), f as f32 + 1000.0);
        }
    }

    #[test]
    fn bridge_carries_partial_blocks_across_callbacks() {
        // Block size 4, but the backend asks for 3, then 5, then 2 frames
        let mut bridge = BlockBridge::new(Box::new(RampProcessor { next_frame: 0 }), 2, 4);
        let mut produced = Vec::new();

        for &frames in &[3usize, 5, 2] {
            let mut buf = vec![0.0f32; frames * 2];
            bridge.render(&[], &mut buf, 2);
            for f in 0..frames {
                produced.push((sample(&buf, 2, f, 0), sample(&buf, 2, f, 1)));
            }
        }

        // The global frame index must stay contiguous with no gaps or repeats
        assert_eq!(produced.len(), 10);
        for (i, (left, right)) in produced.iter().enumerate() {
            assert_eq!(*left, i as f32, "left channel discontinuity at frame {i}");
            assert_eq!(
                *right,
                i as f32 + 1000.0,
                "right discontinuity at frame {i}"
            );
        }
    }

    #[test]
    fn bridge_handles_buffer_larger_than_block() {
        // One callback spanning several blocks plus a partial block
        let mut bridge = BlockBridge::new(Box::new(RampProcessor { next_frame: 0 }), 1, 4);
        let mut out = vec![0.0f32; 11]; // mono, 11 frames = two blocks + 3
        bridge.render(&[], &mut out, 1);
        for (f, s) in out.iter().enumerate() {
            assert_eq!(*s, f as f32, "mono discontinuity at frame {f}");
        }
    }
}
