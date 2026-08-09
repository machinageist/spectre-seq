// Author: Jeff
// Date: 2026-08-09
// Description: R3 slice 2 evidence — backend seam, device enumeration, and stream lifecycle
// Notes: Every test here runs without hardware. The null backend pumps deterministically, so
//   block counts are exact and no test sleeps or races.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use spectre_audio::null::{NullBackend, NULL_DEVICE_KEY, NULL_DEVICE_NAME};
use spectre_audio::{
    AudioBackend, AudioStream, BackendError, DeviceId, RenderBlock, StreamConfig, StreamState,
    MAX_BUFFER_FRAMES, MAX_SAMPLE_RATE, MIN_BUFFER_FRAMES, MIN_SAMPLE_RATE, NULL_BACKEND_NAME,
    OUTPUT_CHANNELS,
};

const SAMPLE_RATE: u32 = 48_000;
const BUFFER_FRAMES: usize = 256;

// Build the standard stereo test configuration
fn config() -> StreamConfig {
    StreamConfig::stereo(SAMPLE_RATE, BUFFER_FRAMES).unwrap()
}

// Build a silence-writing callback that counts its invocations
fn counting_callback(counter: Arc<AtomicUsize>) -> spectre_audio::RenderCallback {
    Box::new(move |mut block: RenderBlock| {
        block.fill_silence();
        counter.fetch_add(1, Ordering::Relaxed);
    })
}

#[test]
fn null_backend_enumerates_one_default_device() {
    let backend = NullBackend::new();
    assert_eq!(backend.name(), NULL_BACKEND_NAME);

    let devices = backend.output_devices().unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].id.as_str(), NULL_DEVICE_KEY);
    assert_eq!(devices[0].name, NULL_DEVICE_NAME);
    assert_eq!(devices[0].max_output_channels, OUTPUT_CHANNELS);
    assert!(devices[0].is_default);

    let default = backend.default_output_device().unwrap();
    assert_eq!(default.id, devices[0].id);
}

#[test]
fn stream_config_rejects_out_of_range_input() {
    assert_eq!(
        StreamConfig::stereo(MIN_SAMPLE_RATE - 1, BUFFER_FRAMES).unwrap_err(),
        BackendError::UnsupportedSampleRate(MIN_SAMPLE_RATE - 1)
    );
    assert_eq!(
        StreamConfig::stereo(MAX_SAMPLE_RATE + 1, BUFFER_FRAMES).unwrap_err(),
        BackendError::UnsupportedSampleRate(MAX_SAMPLE_RATE + 1)
    );
    assert_eq!(
        StreamConfig::stereo(SAMPLE_RATE, MIN_BUFFER_FRAMES - 1).unwrap_err(),
        BackendError::UnsupportedBufferSize(MIN_BUFFER_FRAMES - 1)
    );
    assert_eq!(
        StreamConfig::stereo(SAMPLE_RATE, MAX_BUFFER_FRAMES + 1).unwrap_err(),
        BackendError::UnsupportedBufferSize(MAX_BUFFER_FRAMES + 1)
    );

    let zero_channels = StreamConfig {
        sample_rate: SAMPLE_RATE,
        channels: 0,
        buffer_frames: BUFFER_FRAMES,
    };
    assert_eq!(
        zero_channels.validate().unwrap_err(),
        BackendError::UnsupportedChannels(0)
    );

    // Inclusive bounds are accepted, not merely near-misses
    assert!(StreamConfig::stereo(MIN_SAMPLE_RATE, MIN_BUFFER_FRAMES).is_ok());
    assert!(StreamConfig::stereo(MAX_SAMPLE_RATE, MAX_BUFFER_FRAMES).is_ok());
}

#[test]
fn block_samples_follows_frames_and_channels() {
    assert_eq!(config().block_samples(), BUFFER_FRAMES * 2);
}

#[test]
fn open_rejects_unknown_device() {
    let backend = NullBackend::new();
    let missing = DeviceId::new("does-not-exist");
    let error = backend
        .open_output(&missing, config(), counting_callback(Arc::default()))
        .err()
        .unwrap();
    assert_eq!(error, BackendError::UnknownDevice(missing));
}

#[test]
fn open_rejects_invalid_config_before_touching_the_device() {
    let backend = NullBackend::new();
    let bad = StreamConfig {
        sample_rate: SAMPLE_RATE,
        channels: OUTPUT_CHANNELS,
        buffer_frames: 0,
    };
    let error = backend
        .open_output(
            &DeviceId::new(NULL_DEVICE_KEY),
            bad,
            counting_callback(Arc::default()),
        )
        .err()
        .unwrap();
    assert_eq!(error, BackendError::UnsupportedBufferSize(0));
}

#[test]
fn lifecycle_walks_stopped_running_stopped_closed() {
    let backend = NullBackend::new();
    let calls = Arc::new(AtomicUsize::new(0));
    let mut stream = backend
        .open_null_output(
            &DeviceId::new(NULL_DEVICE_KEY),
            config(),
            counting_callback(Arc::clone(&calls)),
        )
        .unwrap();

    assert_eq!(stream.state(), StreamState::Stopped);
    assert_eq!(stream.config(), config());

    // A stopped stream refuses to render
    assert_eq!(stream.pump().unwrap_err(), BackendError::NotRunning);
    assert_eq!(calls.load(Ordering::Relaxed), 0);

    stream.start().unwrap();
    assert_eq!(stream.state(), StreamState::Running);
    for _ in 0..4 {
        stream.pump().unwrap();
    }
    assert_eq!(calls.load(Ordering::Relaxed), 4);
    assert_eq!(stream.blocks_rendered(), 4);
    assert_eq!(stream.frames_rendered(), 4 * BUFFER_FRAMES as u64);

    stream.stop().unwrap();
    assert_eq!(stream.state(), StreamState::Stopped);
    assert_eq!(stream.pump().unwrap_err(), BackendError::NotRunning);
    assert_eq!(calls.load(Ordering::Relaxed), 4);

    stream.close().unwrap();
    assert_eq!(stream.state(), StreamState::Closed);
}

#[test]
fn invalid_lifecycle_transitions_fail_closed() {
    let backend = NullBackend::new();
    let mut stream = backend
        .open_null_output(
            &DeviceId::new(NULL_DEVICE_KEY),
            config(),
            counting_callback(Arc::default()),
        )
        .unwrap();

    assert_eq!(stream.stop().unwrap_err(), BackendError::NotRunning);
    stream.start().unwrap();
    assert_eq!(stream.start().unwrap_err(), BackendError::AlreadyRunning);

    stream.close().unwrap();
    // Every operation on a closed stream reports Closed, including a second close
    assert_eq!(stream.start().unwrap_err(), BackendError::Closed);
    assert_eq!(stream.stop().unwrap_err(), BackendError::Closed);
    assert_eq!(stream.close().unwrap_err(), BackendError::Closed);
    assert_eq!(stream.pump().unwrap_err(), BackendError::Closed);
}

#[test]
fn callback_receives_exact_block_geometry_and_writes_silence() {
    let backend = NullBackend::new();
    let observed = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&observed);
    let callback: spectre_audio::RenderCallback = Box::new(move |mut block: RenderBlock| {
        seen.store(block.frames(), Ordering::Relaxed);
        assert_eq!(block.channels(), OUTPUT_CHANNELS);
        assert_eq!(block.samples_mut().len(), BUFFER_FRAMES * 2);
        // Write a nonzero ramp, then prove fill_silence clears the whole block
        for (index, sample) in block.samples_mut().iter_mut().enumerate() {
            *sample = index as f32;
        }
        block.fill_silence();
    });

    let mut stream = backend
        .open_null_output(&DeviceId::new(NULL_DEVICE_KEY), config(), callback)
        .unwrap();
    stream.start().unwrap();
    stream.pump().unwrap();

    assert_eq!(observed.load(Ordering::Relaxed), BUFFER_FRAMES);
    assert!(stream.last_block().iter().all(|sample| *sample == 0.0));
}

#[test]
fn stream_drives_the_callback_off_the_enumerating_thread() {
    let backend = NullBackend::new();
    let calls = Arc::new(AtomicUsize::new(0));
    let mut stream = backend
        .open_null_output(
            &DeviceId::new(NULL_DEVICE_KEY),
            config(),
            counting_callback(Arc::clone(&calls)),
        )
        .unwrap();
    stream.start().unwrap();

    // The stream moves to a render thread while the app thread keeps enumerating
    let render = std::thread::spawn(move || {
        for _ in 0..16 {
            stream.pump().unwrap();
        }
        stream.blocks_rendered()
    });

    let backend_devices = backend.output_devices().unwrap();
    let default = backend.default_output_device().unwrap();
    assert_eq!(backend_devices.len(), 1);
    assert_eq!(default.id.as_str(), NULL_DEVICE_KEY);

    assert_eq!(render.join().unwrap(), 16);
    assert_eq!(calls.load(Ordering::Relaxed), 16);
}

#[test]
fn trait_object_open_reports_the_same_lifecycle() {
    let backend = NullBackend::new();
    let mut stream: Box<dyn AudioStream> = backend
        .open_output(
            &DeviceId::new(NULL_DEVICE_KEY),
            config(),
            counting_callback(Arc::default()),
        )
        .unwrap();
    assert_eq!(stream.state(), StreamState::Stopped);
    stream.start().unwrap();
    assert_eq!(stream.state(), StreamState::Running);
    stream.stop().unwrap();
    stream.close().unwrap();
    assert_eq!(stream.state(), StreamState::Closed);
}
