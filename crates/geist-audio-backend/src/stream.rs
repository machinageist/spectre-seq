// =============================================================================
// File: crates/geist-audio-backend/src/stream.rs
// Layer: audio I/O
// Purpose: StreamConfig and the lock-free XrunCounter
// Status: Implemented.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::sync::atomic::{AtomicU64, Ordering};

use geist_core::config::AudioConfig;
use rtrb::{Consumer, Producer, RingBuffer};

// Depth of the input capture ring in samples; sized to outpace the app drain
pub const CAPTURE_RING_CAPACITY: usize = 1 << 16;

// Audio-thread end of the capture ring: the input callback pushes interleaved
// frames here. Wait-free; samples are dropped if the app falls behind.
pub struct CaptureProducer {
    tx: Producer<f32>,
}

impl CaptureProducer {
    // Push one interleaved input block, dropping the tail if the ring is full
    pub fn push_block(&mut self, samples: &[f32]) {
        for &s in samples {
            if self.tx.push(s).is_err() {
                break;
            }
        }
    }
}

// App-thread end of the capture ring: drained by the recorder each frame
pub struct CaptureConsumer {
    pub channels: u16,
    pub sample_rate_hz: u32,
    rx: Consumer<f32>,
}

impl CaptureConsumer {
    // Drain all available captured samples into `out`, returning the count moved
    pub fn drain(&mut self, out: &mut Vec<f32>) -> usize {
        let mut moved = 0;
        while let Ok(sample) = self.rx.pop() {
            out.push(sample);
            moved += 1;
        }
        moved
    }
}

// Build the paired ends of a capture ring for `channels` at `sample_rate_hz`
pub fn capture_ring(channels: u16, sample_rate_hz: u32) -> (CaptureProducer, CaptureConsumer) {
    let (tx, rx) = RingBuffer::new(CAPTURE_RING_CAPACITY);
    (
        CaptureProducer { tx },
        CaptureConsumer {
            channels,
            sample_rate_hz,
            rx,
        },
    )
}

// Lock-free count of buffer xruns reported from the audio callback
// The audio thread records; the UI thread reads; neither blocks
#[derive(Debug, Default)]
pub struct XrunCounter {
    count: AtomicU64,
}

impl XrunCounter {
    // Build a counter starting at zero
    pub fn new() -> Self {
        Self::default()
    }

    // Record one xrun from the audio thread
    pub fn record(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    // Read the running total
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    // Reset the total from the app thread
    pub fn reset(&self) {
        self.count.store(0, Ordering::Relaxed);
    }
}

// Requested stream parameters: device-agnostic audio config plus device choice
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StreamConfig {
    // None selects the backend's default device
    pub device_name: Option<String>,
    pub audio: AudioConfig,
}

impl StreamConfig {
    // Request a stream on the default device
    pub fn new(audio: AudioConfig) -> Self {
        Self {
            device_name: None,
            audio,
        }
    }

    // Request a stream on a named device
    pub fn on_device(audio: AudioConfig, device_name: impl Into<String>) -> Self {
        Self {
            device_name: Some(device_name.into()),
            audio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xrun_counter_records_and_resets() {
        let counter = XrunCounter::new();
        assert_eq!(counter.count(), 0);
        counter.record();
        counter.record();
        counter.record();
        assert_eq!(counter.count(), 3);
        counter.reset();
        assert_eq!(counter.count(), 0);
    }

    #[test]
    fn capture_ring_round_trips_frames() {
        let (mut tx, mut rx) = capture_ring(2, 48_000);
        assert_eq!(rx.channels, 2);
        assert_eq!(rx.sample_rate_hz, 48_000);
        tx.push_block(&[0.1, 0.2, 0.3, 0.4]);
        let mut out = Vec::new();
        assert_eq!(rx.drain(&mut out), 4);
        assert_eq!(out, vec![0.1, 0.2, 0.3, 0.4]);
        // A second drain with nothing pending moves zero
        assert_eq!(rx.drain(&mut out), 0);
    }

    #[test]
    fn capture_ring_drops_tail_when_full() {
        let (mut tx, mut rx) = capture_ring(1, 48_000);
        // Push more than the ring holds; push_block must not panic
        let flood = vec![0.5f32; CAPTURE_RING_CAPACITY + 1_000];
        tx.push_block(&flood);
        let mut out = Vec::new();
        let moved = rx.drain(&mut out);
        assert!(moved <= CAPTURE_RING_CAPACITY);
        assert!(out.iter().all(|&s| s == 0.5));
    }

    #[test]
    fn stream_config_defaults_to_default_device() {
        let audio = AudioConfig::new(48_000, 512, 0, 2).unwrap();
        let config = StreamConfig::new(audio);
        assert_eq!(config.device_name, None);
        assert_eq!(config.audio, audio);

        let named = StreamConfig::on_device(audio, "Scarlett 2i2");
        assert_eq!(named.device_name.as_deref(), Some("Scarlett 2i2"));
    }
}
