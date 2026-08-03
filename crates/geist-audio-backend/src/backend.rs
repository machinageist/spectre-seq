// =============================================================================
// File: crates/geist-audio-backend/src/backend.rs
// Layer: audio I/O
// Purpose: AudioBackend trait
// Status: Implemented; abstraction only. Platform impls (cpal, JACK) land next.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::errors::GeistResult;

use crate::device::DeviceInfo;
use crate::stream::{CaptureConsumer, StreamConfig};

// Realtime render callback driven by the backend's audio thread
// Buffers are device-native interleaved frames; the host bridges to channel-major
// Must not allocate, lock, block, or panic across the backend boundary
pub trait RenderCallback: Send {
    // Fill `output` and consume `input`, both interleaved by `channels`
    fn render(&mut self, input: &[f32], output: &mut [f32], channels: usize);
}

// A running audio stream; stops and releases the device when dropped
pub trait Stream {
    // Buffer xruns observed since the stream started
    fn xruns(&self) -> u64;
}

// Platform audio I/O backend hiding cpal, JACK, PipeWire, and friends
// Enumeration and start happen on the app thread; only render runs on audio
pub trait AudioBackend {
    // Stable backend identifier for settings and diagnostics
    fn name(&self) -> &str;

    // The system default output device, if one exists
    fn default_output_device(&self) -> GeistResult<DeviceInfo>;

    // Every output-capable device the backend can see
    fn output_devices(&self) -> GeistResult<Vec<DeviceInfo>>;

    // Open and start an output stream driving the render callback
    fn start_output(
        &mut self,
        config: &StreamConfig,
        callback: Box<dyn RenderCallback>,
    ) -> GeistResult<Box<dyn Stream>>;

    // The system default input (capture) device, if one exists
    fn default_input_device(&self) -> GeistResult<DeviceInfo>;

    // Open and start an input stream, returning the running stream plus the
    // app-thread consumer that drains captured frames. Default impls without a
    // capture path may return UnsupportedBackend.
    fn start_input(
        &mut self,
        config: &StreamConfig,
    ) -> GeistResult<(Box<dyn Stream>, CaptureConsumer)>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectre_core::config::AudioConfig;

    // Backend stub that runs the callback inline without touching hardware
    struct MockBackend;

    struct MockStream;
    impl Stream for MockStream {
        fn xruns(&self) -> u64 {
            0
        }
    }

    impl AudioBackend for MockBackend {
        fn name(&self) -> &str {
            "mock"
        }
        fn default_output_device(&self) -> GeistResult<DeviceInfo> {
            Ok(self.output_devices()?.remove(0))
        }
        fn output_devices(&self) -> GeistResult<Vec<DeviceInfo>> {
            Ok(vec![DeviceInfo {
                name: "Mock Output".to_string(),
                max_input_channels: 0,
                max_output_channels: 2,
                default_sample_rate_hz: 48_000,
                min_sample_rate_hz: 48_000,
                max_sample_rate_hz: 48_000,
            }])
        }
        fn start_output(
            &mut self,
            _config: &StreamConfig,
            mut callback: Box<dyn RenderCallback>,
        ) -> GeistResult<Box<dyn Stream>> {
            // Drive one block so the test can observe the contract end to end
            let mut output = [0.0f32; 8];
            callback.render(&[], &mut output, 2);
            assert!(output.iter().all(|&s| s == 0.5));
            Ok(Box::new(MockStream))
        }
        fn default_input_device(&self) -> GeistResult<DeviceInfo> {
            Err(spectre_core::errors::GeistError::UnsupportedBackend(
                "mock has no input",
            ))
        }
        fn start_input(
            &mut self,
            _config: &StreamConfig,
        ) -> GeistResult<(Box<dyn Stream>, crate::stream::CaptureConsumer)> {
            Err(spectre_core::errors::GeistError::UnsupportedBackend(
                "mock has no input",
            ))
        }
    }

    // Callback that writes a constant to every output sample
    struct ConstCallback;
    impl RenderCallback for ConstCallback {
        fn render(&mut self, _input: &[f32], output: &mut [f32], _channels: usize) {
            output.fill(0.5);
        }
    }

    #[test]
    fn backend_trait_enumerates_and_starts() {
        let mut backend = MockBackend;
        assert_eq!(backend.name(), "mock");
        assert_eq!(backend.output_devices().unwrap().len(), 1);
        assert!(backend.default_output_device().unwrap().is_output());

        let config = StreamConfig::new(AudioConfig::new(48_000, 4, 0, 2).unwrap());
        let stream = backend
            .start_output(&config, Box::new(ConstCallback))
            .unwrap();
        assert_eq!(stream.xruns(), 0);
    }
}
