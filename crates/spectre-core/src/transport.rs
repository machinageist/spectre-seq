// Author: Jeff
// Date: 2026-05-27
// Description: Transport state, tempo mapping, and audio-thread-readable transport snapshots.
// Notes: App thread mutates transport; audio reads immutable or atomic snapshots.

use std::sync::atomic::{fence, AtomicU32, AtomicU64, Ordering};

// Conservative defaults used before a project sets real values
pub const DEFAULT_TEMPO_BPM: f64 = 120.0;
pub const DEFAULT_TIME_SIG_NUM: u16 = 4;
pub const DEFAULT_TIME_SIG_DEN: u16 = 4;

// Seconds in one minute; tempo math reads in beats per minute
const SECONDS_PER_MINUTE: f64 = 60.0;

// High-level transport run state
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TransportState {
    Stopped,
    Playing,
    Recording,
    Paused,
}

impl TransportState {
    // Encode as a u8 for atomic publication
    #[inline]
    const fn to_u8(self) -> u8 {
        match self {
            TransportState::Stopped => 0,
            TransportState::Playing => 1,
            TransportState::Recording => 2,
            TransportState::Paused => 3,
        }
    }

    // Decode from a published u8, defaulting to Stopped
    #[inline]
    const fn from_u8(v: u8) -> Self {
        match v {
            1 => TransportState::Playing,
            2 => TransportState::Recording,
            3 => TransportState::Paused,
            _ => TransportState::Stopped,
        }
    }
}

// Block-stable transport readout consumed by the audio thread
// Copy and self-contained so a node never reaches back into shared state
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct TransportSnapshot {
    pub state: TransportState,
    pub sample_pos: u64,
    pub tempo_bpm: f64,
    pub time_sig_num: u16,
    pub time_sig_den: u16,
    pub loop_enabled: bool,
    pub loop_start: u64,
    pub loop_end: u64,
    pub sample_rate_hz: u32,
}

impl TransportSnapshot {
    // Build a stopped snapshot at the origin for the given sample rate
    pub fn stopped(sample_rate_hz: u32) -> Self {
        Self {
            state: TransportState::Stopped,
            sample_pos: 0,
            tempo_bpm: DEFAULT_TEMPO_BPM,
            time_sig_num: DEFAULT_TIME_SIG_NUM,
            time_sig_den: DEFAULT_TIME_SIG_DEN,
            loop_enabled: false,
            loop_start: 0,
            loop_end: 0,
            sample_rate_hz,
        }
    }

    // Convert a sample count to beats at the snapshot tempo
    #[inline]
    pub fn samples_to_beats(&self, samples: u64) -> f64 {
        samples as f64 * self.tempo_bpm / (SECONDS_PER_MINUTE * self.sample_rate_hz as f64)
    }

    // Convert beats to the nearest whole sample at the snapshot tempo
    #[inline]
    pub fn beats_to_samples(&self, beats: f64) -> u64 {
        (beats * SECONDS_PER_MINUTE * self.sample_rate_hz as f64 / self.tempo_bpm).round() as u64
    }

    // Beat position of the current playhead
    #[inline]
    pub fn beat_position(&self) -> f64 {
        self.samples_to_beats(self.sample_pos)
    }

    // Fold a sample position into the active loop region
    // Leaves the position untouched when looping is off or the region is empty
    pub fn wrap_position(&self, pos: u64) -> u64 {
        if self.loop_enabled && self.loop_end > self.loop_start && pos >= self.loop_end {
            let len = self.loop_end - self.loop_start;
            self.loop_start + (pos - self.loop_start) % len
        } else {
            pos
        }
    }
}

// Lock-free publication boundary between the app thread and the audio thread
// Implemented as a seqlock over atomics; readers retry on a torn snapshot
// One writer (app thread), many readers; no allocation, no unsafe, wait-free in practice
#[derive(Debug)]
pub struct AtomicTransport {
    // Even value means stable, odd means a write is in progress
    seq: AtomicU32,
    state: AtomicU32,
    sample_pos: AtomicU64,
    tempo_bits: AtomicU64,
    // num in the high 16 bits, den in the low 16 bits
    time_sig: AtomicU32,
    // 0 = looping off, 1 = looping on
    loop_enabled: AtomicU32,
    loop_start: AtomicU64,
    loop_end: AtomicU64,
    sample_rate_hz: AtomicU32,
}

impl AtomicTransport {
    // Publish an initial snapshot
    pub fn new(initial: TransportSnapshot) -> Self {
        let this = Self {
            seq: AtomicU32::new(0),
            state: AtomicU32::new(0),
            sample_pos: AtomicU64::new(0),
            tempo_bits: AtomicU64::new(0),
            time_sig: AtomicU32::new(0),
            loop_enabled: AtomicU32::new(0),
            loop_start: AtomicU64::new(0),
            loop_end: AtomicU64::new(0),
            sample_rate_hz: AtomicU32::new(0),
        };
        this.store(&initial);
        this
    }

    // Publish a new snapshot from the single writer thread
    pub fn store(&self, s: &TransportSnapshot) {
        let begin = self.seq.load(Ordering::Relaxed).wrapping_add(1);
        // Mark write in progress, then order field writes after the marker
        self.seq.store(begin, Ordering::Relaxed);
        fence(Ordering::Release);
        self.state.store(s.state.to_u8() as u32, Ordering::Relaxed);
        self.sample_pos.store(s.sample_pos, Ordering::Relaxed);
        self.tempo_bits
            .store(s.tempo_bpm.to_bits(), Ordering::Relaxed);
        self.time_sig.store(
            ((s.time_sig_num as u32) << 16) | s.time_sig_den as u32,
            Ordering::Relaxed,
        );
        self.loop_enabled
            .store(s.loop_enabled as u32, Ordering::Relaxed);
        self.loop_start.store(s.loop_start, Ordering::Relaxed);
        self.loop_end.store(s.loop_end, Ordering::Relaxed);
        self.sample_rate_hz
            .store(s.sample_rate_hz, Ordering::Relaxed);
        // Order field writes before clearing the in-progress marker
        fence(Ordering::Release);
        self.seq.store(begin.wrapping_add(1), Ordering::Relaxed);
    }

    // Read a consistent snapshot; retries only while a write is mid-flight
    pub fn load(&self) -> TransportSnapshot {
        loop {
            let s1 = self.seq.load(Ordering::Relaxed);
            fence(Ordering::Acquire);
            if s1 & 1 == 1 {
                std::hint::spin_loop();
                continue;
            }
            let time_sig = self.time_sig.load(Ordering::Relaxed);
            let snap = TransportSnapshot {
                state: TransportState::from_u8(self.state.load(Ordering::Relaxed) as u8),
                sample_pos: self.sample_pos.load(Ordering::Relaxed),
                tempo_bpm: f64::from_bits(self.tempo_bits.load(Ordering::Relaxed)),
                time_sig_num: (time_sig >> 16) as u16,
                time_sig_den: (time_sig & 0xFFFF) as u16,
                loop_enabled: self.loop_enabled.load(Ordering::Relaxed) != 0,
                loop_start: self.loop_start.load(Ordering::Relaxed),
                loop_end: self.loop_end.load(Ordering::Relaxed),
                sample_rate_hz: self.sample_rate_hz.load(Ordering::Relaxed),
            };
            fence(Ordering::Acquire);
            if self.seq.load(Ordering::Relaxed) == s1 {
                return snap;
            }
            std::hint::spin_loop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_round_trips_through_u8() {
        for state in [
            TransportState::Stopped,
            TransportState::Playing,
            TransportState::Recording,
            TransportState::Paused,
        ] {
            assert_eq!(TransportState::from_u8(state.to_u8()), state);
        }
    }

    #[test]
    fn beat_sample_conversions_round_trip() {
        let mut snap = TransportSnapshot::stopped(48_000);
        snap.tempo_bpm = 120.0;
        // One beat at 120 BPM and 48 kHz is 24000 samples
        assert_eq!(snap.beats_to_samples(1.0), 24_000);
        assert!((snap.samples_to_beats(24_000) - 1.0).abs() < 1e-9);
        snap.sample_pos = 48_000;
        assert!((snap.beat_position() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn loop_wraps_position_inside_region() {
        let mut snap = TransportSnapshot::stopped(48_000);
        snap.loop_enabled = true;
        snap.loop_start = 100;
        snap.loop_end = 200;
        assert_eq!(snap.wrap_position(150), 150);
        assert_eq!(snap.wrap_position(200), 100);
        assert_eq!(snap.wrap_position(250), 150);
    }

    #[test]
    fn loop_disabled_leaves_position() {
        let snap = TransportSnapshot::stopped(48_000);
        assert_eq!(snap.wrap_position(9_999), 9_999);
    }

    #[test]
    fn atomic_transport_round_trips_snapshot() {
        let mut snap = TransportSnapshot::stopped(44_100);
        snap.state = TransportState::Recording;
        snap.sample_pos = 123_456;
        snap.tempo_bpm = 174.0;
        snap.time_sig_num = 7;
        snap.time_sig_den = 8;
        snap.loop_enabled = true;
        snap.loop_start = 10;
        snap.loop_end = 20;

        let shared = AtomicTransport::new(snap);
        assert_eq!(shared.load(), snap);

        let mut next = snap;
        next.sample_pos = 999;
        shared.store(&next);
        assert_eq!(shared.load(), next);
    }
}
