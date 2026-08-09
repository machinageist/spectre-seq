// Author: Jeff
// Date: 2026-08-09
// Description: Deterministic null backend for CI, offline use, and lifecycle tests
// Notes: Drives the render callback through an explicit pump instead of a timer, so tests
//   observe exact block counts with no sleeps and no hardware. The output buffer is
//   allocated at open on the app thread and reused for every pump.

use crate::{
    AudioBackend, AudioStream, BackendError, DeviceId, DeviceInfo, RenderBlock, RenderCallback,
    StreamConfig, StreamState, NULL_BACKEND_NAME, OUTPUT_CHANNELS,
};

// Identity of the synthetic device the null backend always reports
pub const NULL_DEVICE_KEY: &str = "null-output";
pub const NULL_DEVICE_NAME: &str = "Null Output";

// Backend that enumerates one synthetic device and never touches hardware
#[derive(Debug, Default, Clone, Copy)]
pub struct NullBackend;

impl NullBackend {
    // Build the backend
    pub fn new() -> Self {
        Self
    }

    // Describe the single synthetic device
    fn device(&self) -> DeviceInfo {
        DeviceInfo {
            id: DeviceId::new(NULL_DEVICE_KEY),
            name: NULL_DEVICE_NAME.to_string(),
            max_output_channels: OUTPUT_CHANNELS,
            is_default: true,
        }
    }

    // Open a concrete null stream for tests and offline drivers
    pub fn open_null_output(
        &self,
        device: &DeviceId,
        config: StreamConfig,
        callback: RenderCallback,
    ) -> Result<NullStream, BackendError> {
        config.validate()?;
        if device.as_str() != NULL_DEVICE_KEY {
            return Err(BackendError::UnknownDevice(device.clone()));
        }
        Ok(NullStream::new(config, callback))
    }
}

impl AudioBackend for NullBackend {
    // Identify the backend implementation
    fn name(&self) -> &'static str {
        NULL_BACKEND_NAME
    }

    // List the single synthetic output device
    fn output_devices(&self) -> Result<Vec<DeviceInfo>, BackendError> {
        Ok(vec![self.device()])
    }

    // Report the synthetic device as the default
    fn default_output_device(&self) -> Result<DeviceInfo, BackendError> {
        Ok(self.device())
    }

    // Open the synthetic device behind the trait object
    fn open_output(
        &self,
        device: &DeviceId,
        config: StreamConfig,
        callback: RenderCallback,
    ) -> Result<Box<dyn AudioStream>, BackendError> {
        Ok(Box::new(self.open_null_output(device, config, callback)?))
    }
}

// Deterministic stream whose callback runs only when explicitly pumped
pub struct NullStream {
    config: StreamConfig,
    callback: RenderCallback,
    state: StreamState,
    // Allocated once at open and reused; pumping must not allocate
    buffer: Vec<f32>,
    blocks_rendered: u64,
    frames_rendered: u64,
}

impl NullStream {
    // Build a stopped stream with its output buffer preallocated
    fn new(config: StreamConfig, callback: RenderCallback) -> Self {
        Self {
            config,
            callback,
            state: StreamState::Stopped,
            buffer: vec![0.0; config.block_samples()],
            blocks_rendered: 0,
            frames_rendered: 0,
        }
    }

    // Invoke the render callback for exactly one block, mirroring a driver callback
    pub fn pump(&mut self) -> Result<(), BackendError> {
        match self.state {
            StreamState::Closed => return Err(BackendError::Closed),
            StreamState::Stopped => return Err(BackendError::NotRunning),
            StreamState::Running => {}
        }
        let block = RenderBlock::new(self.buffer.as_mut_slice(), self.config.channels);
        (self.callback)(block);
        self.blocks_rendered += 1;
        self.frames_rendered += self.config.buffer_frames as u64;
        Ok(())
    }

    // Borrow the most recently rendered block
    pub fn last_block(&self) -> &[f32] {
        &self.buffer
    }

    // Count blocks the callback has rendered
    pub fn blocks_rendered(&self) -> u64 {
        self.blocks_rendered
    }

    // Count frames the callback has rendered
    pub fn frames_rendered(&self) -> u64 {
        self.frames_rendered
    }
}

impl AudioStream for NullStream {
    // Begin accepting pumps
    fn start(&mut self) -> Result<(), BackendError> {
        match self.state {
            StreamState::Closed => Err(BackendError::Closed),
            StreamState::Running => Err(BackendError::AlreadyRunning),
            StreamState::Stopped => {
                self.state = StreamState::Running;
                Ok(())
            }
        }
    }

    // Stop accepting pumps without releasing the stream
    fn stop(&mut self) -> Result<(), BackendError> {
        match self.state {
            StreamState::Closed => Err(BackendError::Closed),
            StreamState::Stopped => Err(BackendError::NotRunning),
            StreamState::Running => {
                self.state = StreamState::Stopped;
                Ok(())
            }
        }
    }

    // Release the stream permanently
    fn close(&mut self) -> Result<(), BackendError> {
        if self.state == StreamState::Closed {
            return Err(BackendError::Closed);
        }
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
