// =============================================================================
// File: crates/geist-timeline/src/transport.rs
// Layer: timeline
// Purpose: play/pause/record/loop state machine
// Status: Implemented; transport state, playhead advance, snapshot publication.
// Notes: App-thread owner of run state, playhead, tempo map, and loop region.
//        Advances only while playing or recording; publishes a block-stable
//        TransportSnapshot into a spectre-core AtomicTransport for the audio thread.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::transport::{AtomicTransport, TransportSnapshot, TransportState};

use crate::playhead::{LoopRegion, Playhead};
use crate::tempo::TempoMap;

// App-thread transport: run state, playhead, tempo, and loop region
#[derive(Clone, Debug)]
pub struct Transport {
    state: TransportState,
    playhead: Playhead,
    loop_region: LoopRegion,
    tempo: TempoMap,
    sample_rate: u32,
}

impl Transport {
    // Build a stopped transport at a sample rate and starting tempo
    pub fn new(sample_rate_hz: u32, bpm: f64) -> Self {
        Self {
            state: TransportState::Stopped,
            playhead: Playhead::new(),
            loop_region: LoopRegion::disabled(),
            tempo: TempoMap::new(sample_rate_hz, bpm),
            sample_rate: sample_rate_hz,
        }
    }

    // Current run state
    pub fn state(&self) -> TransportState {
        self.state
    }

    // Whether the playhead is moving this block
    pub fn is_rolling(&self) -> bool {
        matches!(
            self.state,
            TransportState::Playing | TransportState::Recording
        )
    }

    // Current absolute sample position
    pub fn position(&self) -> u64 {
        self.playhead.position()
    }

    // Current beat position via the tempo map
    pub fn beat_position(&self) -> f64 {
        self.tempo.samples_to_beats(self.playhead.position() as f64)
    }

    // Borrow the tempo map
    pub fn tempo_map(&self) -> &TempoMap {
        &self.tempo
    }

    // Borrow the tempo map mutably for edits
    pub fn tempo_map_mut(&mut self) -> &mut TempoMap {
        &mut self.tempo
    }

    // Begin playback from the current position
    pub fn play(&mut self) {
        self.state = TransportState::Playing;
    }

    // Begin recording from the current position
    pub fn record(&mut self) {
        self.state = TransportState::Recording;
    }

    // Hold position and stop rolling
    pub fn pause(&mut self) {
        self.state = TransportState::Paused;
    }

    // Stop and return the playhead to the origin
    pub fn stop(&mut self) {
        self.state = TransportState::Stopped;
        self.playhead.reset();
    }

    // Jump the playhead to an absolute sample position
    pub fn seek(&mut self, sample_pos: u64) {
        self.playhead.seek(sample_pos);
    }

    // Enable a loop over a sample region
    pub fn set_loop(&mut self, start: u64, end: u64) {
        self.loop_region = LoopRegion::new(start, end);
    }

    // Disable looping
    pub fn clear_loop(&mut self) {
        self.loop_region = LoopRegion::disabled();
    }

    // Current loop region
    pub fn loop_region(&self) -> LoopRegion {
        self.loop_region
    }

    // Advance one block of `frames`; a no-op unless rolling
    pub fn advance(&mut self, frames: u64) {
        if self.is_rolling() {
            self.playhead.advance(frames, &self.loop_region);
        }
    }

    // Build a block-stable snapshot for the audio thread
    pub fn snapshot(&self) -> TransportSnapshot {
        let beat = self.tempo.samples_to_beats(self.playhead.position() as f64);
        let (num, den) = self.tempo.time_signature_at(beat);
        TransportSnapshot {
            state: self.state,
            sample_pos: self.playhead.position(),
            tempo_bpm: self.tempo.tempo_at(beat),
            time_sig_num: num,
            time_sig_den: den,
            loop_enabled: self.loop_region.is_active(),
            loop_start: self.loop_region.start,
            loop_end: self.loop_region.end,
            sample_rate_hz: self.sample_rate,
        }
    }

    // Publish the current snapshot into a shared atomic transport
    pub fn publish(&self, shared: &AtomicTransport) {
        shared.store(&self.snapshot());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: u32 = 48_000;

    #[test]
    fn starts_stopped_at_origin() {
        let t = Transport::new(SR, 120.0);
        assert_eq!(t.state(), TransportState::Stopped);
        assert_eq!(t.position(), 0);
        assert!(!t.is_rolling());
    }

    #[test]
    fn only_advances_while_rolling() {
        let mut t = Transport::new(SR, 120.0);
        t.advance(512); // stopped: no movement
        assert_eq!(t.position(), 0);

        t.play();
        t.advance(512);
        assert_eq!(t.position(), 512);

        t.pause(); // paused: holds position
        t.advance(512);
        assert_eq!(t.position(), 512);

        t.record(); // recording rolls too
        t.advance(8);
        assert_eq!(t.position(), 520);
    }

    #[test]
    fn stop_resets_to_origin() {
        let mut t = Transport::new(SR, 120.0);
        t.play();
        t.advance(10_000);
        t.stop();
        assert_eq!(t.state(), TransportState::Stopped);
        assert_eq!(t.position(), 0);
    }

    #[test]
    fn advance_wraps_within_loop() {
        let mut t = Transport::new(SR, 120.0);
        t.set_loop(0, 1000);
        t.seek(900);
        t.play();
        t.advance(200); // 1100 -> wrap to 100
        assert_eq!(t.position(), 100);
    }

    #[test]
    fn beat_position_tracks_tempo() {
        let mut t = Transport::new(SR, 120.0);
        t.play();
        t.advance(24_000); // one beat at 120 BPM / 48 kHz
        assert!((t.beat_position() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn snapshot_round_trips_through_atomic_transport() {
        let mut t = Transport::new(SR, 140.0);
        t.set_loop(100, 500);
        t.seek(250);
        t.record();

        let shared = AtomicTransport::new(TransportSnapshot::stopped(SR));
        t.publish(&shared);
        let snap = shared.load();

        assert_eq!(snap.state, TransportState::Recording);
        assert_eq!(snap.sample_pos, 250);
        assert!((snap.tempo_bpm - 140.0).abs() < 1e-6);
        assert!(snap.loop_enabled);
        assert_eq!(snap.loop_start, 100);
        assert_eq!(snap.loop_end, 500);
        assert_eq!(snap.sample_rate_hz, SR);
    }

    #[test]
    fn snapshot_reads_tempo_at_playhead() {
        let mut t = Transport::new(SR, 120.0);
        t.tempo_map_mut().set_tempo(4.0, 60.0); // slow down at beat 4
        t.play();
        t.seek(t.tempo_map().beats_to_samples(6.0) as u64); // inside slow segment
        let snap = t.snapshot();
        assert!((snap.tempo_bpm - 60.0).abs() < 1e-6);
    }
}
