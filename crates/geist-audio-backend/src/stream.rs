// =============================================================================
// File: crates/geist-audio-backend/src/stream.rs
// Layer: audio I/O
// Purpose: StreamConfig and the lock-free XrunCounter
// Status: Implemented.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::sync::atomic::{AtomicU64, Ordering};

use geist_core::config::AudioConfig;

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
    fn stream_config_defaults_to_default_device() {
        let audio = AudioConfig::new(48_000, 512, 0, 2).unwrap();
        let config = StreamConfig::new(audio);
        assert_eq!(config.device_name, None);
        assert_eq!(config.audio, audio);

        let named = StreamConfig::on_device(audio, "Scarlett 2i2");
        assert_eq!(named.device_name.as_deref(), Some("Scarlett 2i2"));
    }
}
