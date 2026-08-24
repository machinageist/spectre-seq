// Author: Jeff
// Date: 2026-08-09
// Description: Audio backend seam: device enumeration, stream lifecycle, and the render callback type
// Notes: Decision 19 — cpal is one implementation behind AudioBackend, never the interface itself.
//   Enumeration and open/close are app-thread operations and may allocate; the render callback
//   is callback-reachable and inherits RT-001. No CompiledPlan execution lives here yet.

pub mod bridge;
pub mod control;
pub mod midi;
pub mod null;
pub mod route;
pub mod spsc;

#[cfg(feature = "cpal-backend")]
pub mod cpal_backend;

// Backend identity strings kept out of branching logic
pub const NULL_BACKEND_NAME: &str = "null";
pub const CPAL_BACKEND_NAME: &str = "cpal";

// V1 live shell is stereo out, matching the graph's stereo-only buses
pub const OUTPUT_CHANNELS: u16 = 2;

// Refuse absurd configurations before they reach a driver
pub const MIN_SAMPLE_RATE: u32 = 8_000;
pub const MAX_SAMPLE_RATE: u32 = 768_000;
pub const MIN_BUFFER_FRAMES: usize = 16;
pub const MAX_BUFFER_FRAMES: usize = 8_192;

// Stable identity for an enumerated output device
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DeviceId(String);

impl DeviceId {
    // Wrap a backend-reported device key
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    // Borrow the backend-reported device key
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// One enumerated output device as reported by a backend
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub id: DeviceId,
    pub name: String,
    pub max_output_channels: u16,
    pub is_default: bool,
}

// Requested stream format, validated before any driver call
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_frames: usize,
}

impl StreamConfig {
    // Build a validated stereo configuration
    pub fn stereo(sample_rate: u32, buffer_frames: usize) -> Result<Self, BackendError> {
        let config = Self {
            sample_rate,
            channels: OUTPUT_CHANNELS,
            buffer_frames,
        };
        config.validate()?;
        Ok(config)
    }

    // Reject configurations no backend should be asked to honor
    pub fn validate(&self) -> Result<(), BackendError> {
        if !(MIN_SAMPLE_RATE..=MAX_SAMPLE_RATE).contains(&self.sample_rate) {
            return Err(BackendError::UnsupportedSampleRate(self.sample_rate));
        }
        if self.channels == 0 {
            return Err(BackendError::UnsupportedChannels(self.channels));
        }
        if !(MIN_BUFFER_FRAMES..=MAX_BUFFER_FRAMES).contains(&self.buffer_frames) {
            return Err(BackendError::UnsupportedBufferSize(self.buffer_frames));
        }
        Ok(())
    }

    // Interleaved sample count for one full callback block
    pub fn block_samples(&self) -> usize {
        self.buffer_frames * self.channels as usize
    }
}

// App-thread backend failure; never constructed on the callback path
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendError {
    NoDefaultDevice,
    UnknownDevice(DeviceId),
    UnsupportedSampleRate(u32),
    UnsupportedChannels(u16),
    UnsupportedBufferSize(usize),
    // Backend refused the open; the string is a diagnostic, not a control path
    OpenFailed(String),
    EnumerationFailed(String),
    // Lifecycle command issued against a stream already in that state
    AlreadyRunning,
    NotRunning,
    // Stream was closed and cannot be reused
    Closed,
}

impl std::fmt::Display for BackendError {
    // Render an actionable app-thread diagnostic
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoDefaultDevice => write!(f, "no default output device"),
            Self::UnknownDevice(id) => write!(f, "unknown output device {}", id.as_str()),
            Self::UnsupportedSampleRate(rate) => write!(f, "unsupported sample rate {rate}"),
            Self::UnsupportedChannels(count) => write!(f, "unsupported channel count {count}"),
            Self::UnsupportedBufferSize(frames) => write!(f, "unsupported buffer size {frames}"),
            Self::OpenFailed(reason) => write!(f, "stream open failed: {reason}"),
            Self::EnumerationFailed(reason) => write!(f, "device enumeration failed: {reason}"),
            Self::AlreadyRunning => write!(f, "stream is already running"),
            Self::NotRunning => write!(f, "stream is not running"),
            Self::Closed => write!(f, "stream is closed"),
        }
    }
}

impl std::error::Error for BackendError {}

// Interleaved output block handed to the render callback
pub struct RenderBlock<'a> {
    samples: &'a mut [f32],
    channels: u16,
}

impl<'a> RenderBlock<'a> {
    // Wrap a driver-owned interleaved buffer for one callback
    pub fn new(samples: &'a mut [f32], channels: u16) -> Self {
        Self { samples, channels }
    }

    // Frame count implied by the buffer length and channel count
    pub fn frames(&self) -> usize {
        if self.channels == 0 {
            return 0;
        }
        self.samples.len() / self.channels as usize
    }

    // Channel count of the interleaved block
    pub fn channels(&self) -> u16 {
        self.channels
    }

    // Borrow the interleaved buffer for writing
    pub fn samples_mut(&mut self) -> &mut [f32] {
        self.samples
    }

    // Write silence across the whole block without allocating
    pub fn fill_silence(&mut self) {
        self.samples.fill(0.0);
    }
}

// Render callback invoked on the backend's audio thread; inherits RT-001
pub type RenderCallback = Box<dyn FnMut(RenderBlock) + Send + 'static>;

// Lifecycle state of an opened output stream
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    Stopped,
    Running,
    Closed,
}

// An opened output stream; not required to be Send because cpal streams are thread-affine
pub trait AudioStream {
    // Begin invoking the render callback
    fn start(&mut self) -> Result<(), BackendError>;

    // Stop invoking the render callback without releasing the device
    fn stop(&mut self) -> Result<(), BackendError>;

    // Release the device; the stream is unusable afterward
    fn close(&mut self) -> Result<(), BackendError>;

    // Report the current lifecycle state
    fn state(&self) -> StreamState;

    // Report the configuration the stream was opened with
    fn config(&self) -> StreamConfig;

    // Count driver-thread stream errors recorded since open. Required rather than defaulted
    // so a new backend must answer instead of silently reporting zero
    fn stream_errors(&self) -> u64;
}

// Device enumeration and stream construction; every method is app-thread-only
pub trait AudioBackend {
    // Identify the backend implementation
    fn name(&self) -> &'static str;

    // List available output devices
    fn output_devices(&self) -> Result<Vec<DeviceInfo>, BackendError>;

    // Report the device a stream opens against when none is named
    fn default_output_device(&self) -> Result<DeviceInfo, BackendError>;

    // Report the sample rate a device is currently configured for, before any open attempt.
    // DeviceInfo carries no format, so without this the app must guess a rate
    fn default_sample_rate(&self, device: &DeviceId) -> Result<u32, BackendError>;

    // Open an output stream against a named device
    fn open_output(
        &self,
        device: &DeviceId,
        config: StreamConfig,
        callback: RenderCallback,
    ) -> Result<Box<dyn AudioStream>, BackendError>;
}
