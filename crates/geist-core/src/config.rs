// Author: Jeff
// Date: 2026-05-27
// Description: Audio engine configuration shared by backend, graph, DSP, and tests.
// Notes: Configuration is immutable once a stream starts unless rebuilt on the app thread.

use crate::errors::{GeistError, GeistResult};

// Accepted sample-rate window covering pro-audio devices
pub const MIN_SAMPLE_RATE_HZ: u32 = 8_000;
pub const MAX_SAMPLE_RATE_HZ: u32 = 768_000;

// Accepted block-size window; small blocks lower latency, large blocks lower overhead
pub const MIN_BLOCK_FRAMES: u32 = 1;
pub const MAX_BLOCK_FRAMES: u32 = 8_192;

// Upper bound on channels per stream; guards descriptor and buffer sizing
pub const MAX_CHANNELS: u16 = 256;

// Immutable audio stream configuration
// Validated on construction so the audio thread never re-checks it
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct AudioConfig {
    pub sample_rate_hz: u32,
    pub block_size_frames: u32,
    pub input_channels: u16,
    pub output_channels: u16,
}

impl AudioConfig {
    // Build a validated configuration or report why it was rejected
    pub fn new(
        sample_rate_hz: u32,
        block_size_frames: u32,
        input_channels: u16,
        output_channels: u16,
    ) -> GeistResult<Self> {
        if !(MIN_SAMPLE_RATE_HZ..=MAX_SAMPLE_RATE_HZ).contains(&sample_rate_hz) {
            return Err(GeistError::BadConfig("sample rate out of supported range"));
        }
        if !(MIN_BLOCK_FRAMES..=MAX_BLOCK_FRAMES).contains(&block_size_frames) {
            return Err(GeistError::BadConfig("block size out of supported range"));
        }
        if input_channels > MAX_CHANNELS || output_channels > MAX_CHANNELS {
            return Err(GeistError::BadConfig("channel count exceeds maximum"));
        }
        if input_channels == 0 && output_channels == 0 {
            return Err(GeistError::BadConfig("stream has no channels"));
        }
        Ok(Self {
            sample_rate_hz,
            block_size_frames,
            input_channels,
            output_channels,
        })
    }

    // Frames processed per block as a slice length
    #[inline]
    pub const fn frames_per_block(&self) -> usize {
        self.block_size_frames as usize
    }

    // Wall-clock duration of one processing block in seconds
    #[inline]
    pub fn block_duration_seconds(&self) -> f64 {
        self.block_size_frames as f64 / self.sample_rate_hz as f64
    }

    // Highest representable frequency for this sample rate
    #[inline]
    pub fn nyquist_hz(&self) -> f64 {
        self.sample_rate_hz as f64 / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_config_constructs() {
        let cfg = AudioConfig::new(48_000, 512, 2, 2).unwrap();
        assert_eq!(cfg.frames_per_block(), 512);
    }

    #[test]
    fn rejects_zero_sample_rate() {
        assert!(AudioConfig::new(0, 512, 2, 2).is_err());
    }

    #[test]
    fn rejects_zero_block_size() {
        assert!(AudioConfig::new(48_000, 0, 2, 2).is_err());
    }

    #[test]
    fn rejects_oversized_block() {
        assert!(AudioConfig::new(48_000, MAX_BLOCK_FRAMES + 1, 2, 2).is_err());
    }

    #[test]
    fn rejects_no_channels() {
        assert!(AudioConfig::new(48_000, 512, 0, 0).is_err());
    }

    #[test]
    fn derived_math_is_correct() {
        let cfg = AudioConfig::new(48_000, 480, 0, 2).unwrap();
        assert!((cfg.block_duration_seconds() - 0.01).abs() < 1e-9);
        assert!((cfg.nyquist_hz() - 24_000.0).abs() < 1e-9);
    }
}
