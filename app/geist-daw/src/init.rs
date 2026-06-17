// =============================================================================
// File: app/geist-daw/src/init.rs
// Layer: application binary
// Purpose: Startup: pick a device-compatible config and start the audio stream
// Status: Implemented; output-only stream driving the demo synth processor.
// Notes: The config is derived from the default output device so the cpal stream
//        opens across hardware. The synth node is prepared on this app thread
//        before it moves to the audio thread; prepare() does the allocation.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_audio_backend::prelude::{AudioBackend, BlockBridge, CpalBackend, StreamConfig};
use geist_core::config::{AudioConfig, MAX_SAMPLE_RATE_HZ, MIN_SAMPLE_RATE_HZ};
use geist_core::errors::GeistResult;

use crate::control::{control_plane, EngineControl};
use crate::engine::{
    default_grid_for, Engine, SynthProcessor, Track, DEFAULT_BPM, NUM_TRACKS, TRACK_BASE_MIDI,
};
use crate::fx::FxChain;

// Fixed render block size; small enough for low latency, widely supported
const BLOCK_FRAMES: u32 = 512;
// Fallback rate when the device reports an unusable default
const FALLBACK_SAMPLE_RATE_HZ: u32 = 48_000;
// Prefer stereo; mono devices still work via the synth's channel fan-out
const PREFERRED_CHANNELS: u16 = 2;
// Voice count for the synth
const POLYPHONY: usize = 16;

// Open the default device, start the stream, and return the engine + UI control
// `rolling` seeds whether the transport plays the sequence at startup
pub fn start(rolling: bool) -> GeistResult<(Engine, EngineControl)> {
    let mut backend = CpalBackend::new();
    let device = backend.default_output_device()?;

    let sample_rate_hz = usable_rate(device.default_sample_rate_hz);
    let channels = device.max_output_channels.clamp(1, PREFERRED_CHANNELS);
    let audio = AudioConfig::new(sample_rate_hz, BLOCK_FRAMES, 0, channels)?;

    // Build and prepare each track's instrument on the app thread before it
    // crosses to the audio thread
    let mut tracks = Vec::with_capacity(NUM_TRACKS);
    for (index, &base_midi) in TRACK_BASE_MIDI.iter().enumerate() {
        let mut track = Track::new(sample_rate_hz, POLYPHONY, base_midi, default_grid_for(index));
        track.prepare(&audio);
        tracks.push(track);
    }

    // Build and prepare the effects chain on the app thread
    let mut fx = FxChain::new(channels as usize, BLOCK_FRAMES as usize, sample_rate_hz);
    fx.prepare(&audio);

    let block_len = channels as usize * BLOCK_FRAMES as usize;
    let (control, sink) = control_plane(NUM_TRACKS);
    let processor = SynthProcessor::new(
        tracks,
        sample_rate_hz,
        block_len,
        sink,
        fx,
        rolling,
        DEFAULT_BPM,
    );
    let bridge = BlockBridge::new(
        Box::new(processor),
        channels as usize,
        BLOCK_FRAMES as usize,
    );

    let config = StreamConfig::new(audio);
    let stream = backend.start_output(&config, Box::new(bridge))?;

    Ok((Engine::new(backend, stream, sample_rate_hz, channels), control))
}

// Clamp a device-reported rate into the supported window, or fall back
fn usable_rate(reported: u32) -> u32 {
    if (MIN_SAMPLE_RATE_HZ..=MAX_SAMPLE_RATE_HZ).contains(&reported) {
        reported
    } else {
        FALLBACK_SAMPLE_RATE_HZ
    }
}
