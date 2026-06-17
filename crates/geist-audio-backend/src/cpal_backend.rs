// =============================================================================
// File: crates/geist-audio-backend/src/cpal_backend.rs
// Layer: audio I/O
// Purpose: cpal wrapper (all platforms, default)
// Status: Implemented; output backend over cpal (CoreAudio/WASAPI/ALSA).
// Notes: cpal hides its own FFI, so this stays safe Rust. f32 streams only;
//        the render callback receives device-native interleaved frames. Stream
//        errors are counted as xruns through a lock-free shared counter.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use geist_core::errors::{GeistError, GeistResult};

use crate::backend::{AudioBackend, RenderCallback, Stream};
use crate::device::DeviceInfo;
use crate::stream::{capture_ring, CaptureConsumer, StreamConfig, XrunCounter};

// Default output device handle the host exposes
pub struct CpalBackend {
    host: cpal::Host,
}

impl CpalBackend {
    // Open the platform's default audio host
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    // Find an output device by exact name
    fn find_output_device(&self, name: &str) -> GeistResult<cpal::Device> {
        let devices = self
            .host
            .output_devices()
            .map_err(|_| GeistError::UnsupportedBackend("cannot enumerate output devices"))?;
        for device in devices {
            if device.name().map(|n| n == name).unwrap_or(false) {
                return Ok(device);
            }
        }
        Err(GeistError::UnsupportedBackend("named output device not found"))
    }

    // Find an input device by exact name
    fn find_input_device(&self, name: &str) -> GeistResult<cpal::Device> {
        let devices = self
            .host
            .input_devices()
            .map_err(|_| GeistError::UnsupportedBackend("cannot enumerate input devices"))?;
        for device in devices {
            if device.name().map(|n| n == name).unwrap_or(false) {
                return Ok(device);
            }
        }
        Err(GeistError::UnsupportedBackend("named input device not found"))
    }
}

impl Default for CpalBackend {
    fn default() -> Self {
        Self::new()
    }
}

// Describe a cpal device as backend-agnostic DeviceInfo
fn describe(device: &cpal::Device) -> GeistResult<DeviceInfo> {
    let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
    let default = device
        .default_output_config()
        .map_err(|_| GeistError::UnsupportedBackend("device has no default output config"))?;

    let mut min_sr = default.sample_rate().0;
    let mut max_sr = default.sample_rate().0;
    let mut max_channels = default.channels();
    if let Ok(configs) = device.supported_output_configs() {
        for config in configs {
            min_sr = min_sr.min(config.min_sample_rate().0);
            max_sr = max_sr.max(config.max_sample_rate().0);
            max_channels = max_channels.max(config.channels());
        }
    }

    Ok(DeviceInfo {
        name,
        max_input_channels: 0,
        max_output_channels: max_channels,
        default_sample_rate_hz: default.sample_rate().0,
        min_sample_rate_hz: min_sr,
        max_sample_rate_hz: max_sr,
    })
}

// Describe a cpal input device as backend-agnostic DeviceInfo
fn describe_input(device: &cpal::Device) -> GeistResult<DeviceInfo> {
    let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
    let default = device
        .default_input_config()
        .map_err(|_| GeistError::UnsupportedBackend("device has no default input config"))?;

    let mut min_sr = default.sample_rate().0;
    let mut max_sr = default.sample_rate().0;
    let mut max_channels = default.channels();
    if let Ok(configs) = device.supported_input_configs() {
        for config in configs {
            min_sr = min_sr.min(config.min_sample_rate().0);
            max_sr = max_sr.max(config.max_sample_rate().0);
            max_channels = max_channels.max(config.channels());
        }
    }

    Ok(DeviceInfo {
        name,
        max_input_channels: max_channels,
        max_output_channels: 0,
        default_sample_rate_hz: default.sample_rate().0,
        min_sample_rate_hz: min_sr,
        max_sample_rate_hz: max_sr,
    })
}

impl AudioBackend for CpalBackend {
    fn name(&self) -> &str {
        "cpal"
    }

    fn default_output_device(&self) -> GeistResult<DeviceInfo> {
        let device = self
            .host
            .default_output_device()
            .ok_or(GeistError::UnsupportedBackend("no default output device"))?;
        describe(&device)
    }

    fn output_devices(&self) -> GeistResult<Vec<DeviceInfo>> {
        let devices = self
            .host
            .output_devices()
            .map_err(|_| GeistError::UnsupportedBackend("cannot enumerate output devices"))?;
        let mut infos = Vec::new();
        for device in devices {
            if let Ok(info) = describe(&device) {
                infos.push(info);
            }
        }
        Ok(infos)
    }

    fn start_output(
        &mut self,
        config: &StreamConfig,
        mut callback: Box<dyn RenderCallback>,
    ) -> GeistResult<Box<dyn Stream>> {
        let device = match &config.device_name {
            Some(name) => self.find_output_device(name)?,
            None => self
                .host
                .default_output_device()
                .ok_or(GeistError::UnsupportedBackend("no default output device"))?,
        };

        let channels = config.audio.output_channels;
        // Default buffer size: not every backend honors a fixed size (CoreAudio
        // ignores it), and the BlockBridge re-blocks any callback size anyway
        let cpal_config = cpal::StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(config.audio.sample_rate_hz),
            buffer_size: cpal::BufferSize::Default,
        };

        // Stream errors (under/overruns) increment the xrun counter the UI reads
        let xruns = Arc::new(XrunCounter::new());
        let error_xruns = Arc::clone(&xruns);
        let channel_count = channels as usize;

        let stream = device
            .build_output_stream(
                &cpal_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    callback.render(&[], data, channel_count);
                },
                move |_err| {
                    error_xruns.record();
                },
                None,
            )
            .map_err(|_| GeistError::UnsupportedBackend("failed to build output stream"))?;

        stream
            .play()
            .map_err(|_| GeistError::UnsupportedBackend("failed to start output stream"))?;

        Ok(Box::new(CpalStream { _stream: stream, xruns }))
    }

    fn default_input_device(&self) -> GeistResult<DeviceInfo> {
        let device = self
            .host
            .default_input_device()
            .ok_or(GeistError::UnsupportedBackend("no default input device"))?;
        describe_input(&device)
    }

    fn start_input(
        &mut self,
        config: &StreamConfig,
    ) -> GeistResult<(Box<dyn Stream>, CaptureConsumer)> {
        let device = match &config.device_name {
            Some(name) => self.find_input_device(name)?,
            None => self
                .host
                .default_input_device()
                .ok_or(GeistError::UnsupportedBackend("no default input device"))?,
        };

        // Capture at the device's native input format; the consumer carries the
        // actual channel count and rate so the recorder can interpret frames.
        let default = device
            .default_input_config()
            .map_err(|_| GeistError::UnsupportedBackend("device has no default input config"))?;
        let channels = default.channels();
        let sample_rate = default.sample_rate().0;
        let cpal_config = cpal::StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let (mut producer, consumer) = capture_ring(channels, sample_rate);
        let xruns = Arc::new(XrunCounter::new());
        let error_xruns = Arc::clone(&xruns);

        let stream = device
            .build_input_stream(
                &cpal_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Wait-free: hand the interleaved block to the app drain
                    producer.push_block(data);
                },
                move |_err| {
                    error_xruns.record();
                },
                None,
            )
            .map_err(|_| GeistError::UnsupportedBackend("failed to build input stream"))?;

        stream
            .play()
            .map_err(|_| GeistError::UnsupportedBackend("failed to start input stream"))?;

        Ok((Box::new(CpalStream { _stream: stream, xruns }), consumer))
    }
}

// A running cpal output stream; stops the device when dropped
pub struct CpalStream {
    // Held so the device keeps running for the stream's lifetime
    _stream: cpal::Stream,
    xruns: Arc<XrunCounter>,
}

impl Stream for CpalStream {
    fn xruns(&self) -> u64 {
        self.xruns.count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_reports_its_name() {
        let backend = CpalBackend::new();
        assert_eq!(backend.name(), "cpal");
    }

    #[test]
    fn enumeration_never_panics() {
        // Devices may be absent in CI; enumeration must degrade gracefully
        let backend = CpalBackend::new();
        let _ = backend.output_devices();
        let _ = backend.default_output_device();
    }
}
