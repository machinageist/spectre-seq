// =============================================================================
// File: app/geist-daw/src/recorder.rs
// Layer: application binary
// Purpose: Drain captured input frames into a buffer and write recorded clips
// Status: Implemented; app-thread recorder over the backend capture ring.
// Notes: The recorder lives on the UI thread. It drains the lock-free capture
//        ring each frame: accumulating while armed, discarding otherwise so the
//        ring never overflows. On stop it yields the interleaved buffer, which
//        the app writes as a 32-bit float WAV and hands to the engine as an asset.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::path::Path;

use geist_audio_backend::prelude::CaptureConsumer;

// A finished recording: interleaved samples plus their capture format
pub struct RecordedAudio {
    pub samples: Vec<f32>,
    pub channels: u16,
    pub sample_rate_hz: u32,
}

impl RecordedAudio {
    // Length in frames (interleaved samples / channels)
    pub fn frames(&self) -> u64 {
        if self.channels == 0 {
            0
        } else {
            (self.samples.len() / self.channels as usize) as u64
        }
    }
}

// Drains the capture ring into a growing buffer while recording
pub struct AudioRecorder {
    capture: CaptureConsumer,
    buffer: Vec<f32>,
    discard: Vec<f32>,
    recording: bool,
}

impl AudioRecorder {
    pub fn new(capture: CaptureConsumer) -> Self {
        Self {
            capture,
            buffer: Vec::new(),
            discard: Vec::new(),
            recording: false,
        }
    }

    // Begin a fresh recording, discarding anything already queued
    pub fn start(&mut self) {
        self.buffer.clear();
        self.discard.clear();
        self.capture.drain(&mut self.discard);
        self.discard.clear();
        self.recording = true;
    }

    // Pull pending input: accumulate while recording, otherwise discard so the
    // ring stays drained and never overflows
    pub fn poll(&mut self) {
        if self.recording {
            self.capture.drain(&mut self.buffer);
        } else {
            self.discard.clear();
            self.capture.drain(&mut self.discard);
        }
    }

    // Finish recording and take the captured buffer
    pub fn stop(&mut self) -> RecordedAudio {
        self.recording = false;
        self.capture.drain(&mut self.buffer);
        RecordedAudio {
            samples: std::mem::take(&mut self.buffer),
            channels: self.capture.channels,
            sample_rate_hz: self.capture.sample_rate_hz,
        }
    }
}

// Write interleaved f32 samples as a 32-bit float WAV at `path`
pub fn write_wav(path: &Path, audio: &RecordedAudio) -> Result<(), hound::Error> {
    let spec = hound::WavSpec {
        channels: audio.channels.max(1),
        sample_rate: audio.sample_rate_hz,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;
    for &sample in &audio.samples {
        writer.write_sample(sample)?;
    }
    writer.finalize()
}

// Read a 32-bit float WAV back into interleaved f32 samples and its format,
// reconstructing a take saved earlier by `write_wav` for engine playback
pub fn read_wav(path: &Path) -> Result<RecordedAudio, hound::Error> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    let samples: Result<Vec<f32>, _> = reader.samples::<f32>().collect();
    Ok(RecordedAudio {
        samples: samples?,
        channels: spec.channels,
        sample_rate_hz: spec.sample_rate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use geist_audio_backend::prelude::capture_ring;

    #[test]
    fn recorder_accumulates_only_while_recording() {
        let (mut tx, rx) = capture_ring(1, 48_000);
        let mut rec = AudioRecorder::new(rx);

        // Frames pushed before start are discarded
        tx.push_block(&[9.0, 9.0]);
        rec.poll();
        rec.start();
        tx.push_block(&[0.1, 0.2, 0.3]);
        rec.poll();
        tx.push_block(&[0.4]);
        let out = rec.stop();
        assert_eq!(out.samples, vec![0.1, 0.2, 0.3, 0.4]);
        assert_eq!(out.channels, 1);
        assert_eq!(out.frames(), 4);
    }

    #[test]
    fn wav_round_trips_the_samples() {
        let path = std::env::temp_dir().join(format!("geist-rec-{}.wav", std::process::id()));
        let audio = RecordedAudio {
            samples: vec![0.0, 0.25, -0.5, 0.75],
            channels: 1,
            sample_rate_hz: 48_000,
        };
        write_wav(&path, &audio).unwrap();
        let mut reader = hound::WavReader::open(&path).unwrap();
        let read: Vec<f32> = reader.samples::<f32>().map(|s| s.unwrap()).collect();
        std::fs::remove_file(&path).ok();
        assert_eq!(read, audio.samples);
    }

    #[test]
    fn read_wav_recovers_samples_and_format() {
        let path = std::env::temp_dir().join(format!("geist-read-{}.wav", std::process::id()));
        let audio = RecordedAudio {
            samples: vec![0.1, -0.2, 0.3, -0.4],
            channels: 2,
            sample_rate_hz: 44_100,
        };
        write_wav(&path, &audio).unwrap();
        let back = read_wav(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(back.samples, audio.samples);
        assert_eq!(back.channels, 2);
        assert_eq!(back.sample_rate_hz, 44_100);
        assert_eq!(back.frames(), 2);
    }
}
