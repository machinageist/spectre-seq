// Author: Jeff
// Date: 2026-08-09
// Description: cpal implementation of the AudioBackend seam
// Notes: Decision 19 — this file is the only place cpal types appear. The data callback and
//   the error callback are both callback-reachable and inherit RT-001: they must not
//   allocate, lock, log, or panic. Stream errors are counted into an atomic and read
//   off-thread instead of being reported from the audio thread.

use crate::{
    AudioBackend, AudioStream, BackendError, DeviceId, DeviceInfo, RenderBlock, RenderCallback,
    StreamConfig, StreamState, CPAL_BACKEND_NAME,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// cpal reports no stable device identity, so the reported name is the key
fn device_key(device: &cpal::Device) -> Result<String, BackendError> {
    device
        .name()
        .map_err(|error| BackendError::EnumerationFailed(error.to_string()))
}

// Largest output channel count the device advertises
fn max_output_channels(device: &cpal::Device) -> u16 {
    let configs = match device.supported_output_configs() {
        Ok(configs) => configs,
        Err(_) => return 0,
    };
    configs.map(|config| config.channels()).max().unwrap_or(0)
}

// Backend that enumerates and opens real hardware through cpal
pub struct CpalBackend {
    host: cpal::Host,
}

impl Default for CpalBackend {
    // Build against the platform's default host
    fn default() -> Self {
        Self::new()
    }
}

impl CpalBackend {
    // Build against the platform's default host
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    // Resolve an enumerated device by its reported key
    fn find_device(&self, id: &DeviceId) -> Result<cpal::Device, BackendError> {
        let devices = self
            .host
            .output_devices()
            .map_err(|error| BackendError::EnumerationFailed(error.to_string()))?;
        for device in devices {
            if device_key(&device)? == id.as_str() {
                return Ok(device);
            }
        }
        Err(BackendError::UnknownDevice(id.clone()))
    }
}

impl AudioBackend for CpalBackend {
    // Identify the backend implementation
    fn name(&self) -> &'static str {
        CPAL_BACKEND_NAME
    }

    // List every output device the host reports
    fn output_devices(&self) -> Result<Vec<DeviceInfo>, BackendError> {
        let default_key = self
            .host
            .default_output_device()
            .as_ref()
            .and_then(|device| device_key(device).ok());
        let devices = self
            .host
            .output_devices()
            .map_err(|error| BackendError::EnumerationFailed(error.to_string()))?;
        let mut reported = Vec::new();
        for device in devices {
            let key = device_key(&device)?;
            let is_default = default_key.as_deref() == Some(key.as_str());
            reported.push(DeviceInfo {
                id: DeviceId::new(key.clone()),
                name: key,
                max_output_channels: max_output_channels(&device),
                is_default,
            });
        }
        Ok(reported)
    }

    // Report the host's default output device
    fn default_output_device(&self) -> Result<DeviceInfo, BackendError> {
        let device = self
            .host
            .default_output_device()
            .ok_or(BackendError::NoDefaultDevice)?;
        let key = device_key(&device)?;
        Ok(DeviceInfo {
            id: DeviceId::new(key.clone()),
            name: key,
            max_output_channels: max_output_channels(&device),
            is_default: true,
        })
    }

    // Open a stopped output stream against a named device
    fn open_output(
        &self,
        device: &DeviceId,
        config: StreamConfig,
        mut callback: RenderCallback,
    ) -> Result<Box<dyn AudioStream>, BackendError> {
        config.validate()?;
        let device = self.find_device(device)?;
        let errors = Arc::new(AtomicU64::new(0));
        let error_sink = Arc::clone(&errors);
        let channels = config.channels;
        let cpal_config = cpal::StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(config.sample_rate),
            buffer_size: cpal::BufferSize::Fixed(config.buffer_frames as u32),
        };
        let stream = device
            .build_output_stream(
                &cpal_config,
                // Audio thread: no allocation, no locks, no logging, no panics
                move |samples: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    callback(RenderBlock::new(samples, channels));
                },
                // Audio/driver thread: count only, never format or log here
                move |_error| {
                    error_sink.fetch_add(1, Ordering::Relaxed);
                },
                None,
            )
            .map_err(|error| BackendError::OpenFailed(error.to_string()))?;
        Ok(Box::new(CpalStream {
            stream: Some(stream),
            config,
            state: StreamState::Stopped,
            errors,
        }))
    }
}

// An opened cpal stream; thread-affine, so it is not Send
pub struct CpalStream {
    // Taken on close so the device is released deterministically
    stream: Option<cpal::Stream>,
    config: StreamConfig,
    state: StreamState,
    errors: Arc<AtomicU64>,
}

impl CpalStream {
    // Read the stream-error count recorded by the audio thread
    pub fn error_count(&self) -> u64 {
        self.errors.load(Ordering::Relaxed)
    }
}

impl AudioStream for CpalStream {
    // Begin invoking the render callback
    fn start(&mut self) -> Result<(), BackendError> {
        match self.state {
            StreamState::Closed => return Err(BackendError::Closed),
            StreamState::Running => return Err(BackendError::AlreadyRunning),
            StreamState::Stopped => {}
        }
        let stream = self.stream.as_ref().ok_or(BackendError::Closed)?;
        stream
            .play()
            .map_err(|error| BackendError::OpenFailed(error.to_string()))?;
        self.state = StreamState::Running;
        Ok(())
    }

    // Stop invoking the render callback without releasing the device
    fn stop(&mut self) -> Result<(), BackendError> {
        match self.state {
            StreamState::Closed => return Err(BackendError::Closed),
            StreamState::Stopped => return Err(BackendError::NotRunning),
            StreamState::Running => {}
        }
        let stream = self.stream.as_ref().ok_or(BackendError::Closed)?;
        stream
            .pause()
            .map_err(|error| BackendError::OpenFailed(error.to_string()))?;
        self.state = StreamState::Stopped;
        Ok(())
    }

    // Release the device by dropping the underlying stream
    fn close(&mut self) -> Result<(), BackendError> {
        if self.state == StreamState::Closed {
            return Err(BackendError::Closed);
        }
        self.stream = None;
        self.state = StreamState::Closed;
        Ok(())
    }

    // Report the current lifecycle state
    fn state(&self) -> StreamState {
        self.state
    }

    // Report the configuration the stream was opened with
    fn config(&self) -> StreamConfig {
        self.config
    }
}
