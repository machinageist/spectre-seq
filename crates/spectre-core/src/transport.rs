// Author: Jeff
// Date: 2026-07-11
// Description: Deterministic transport state machine with half-open loop wrap (TIME-002, TIME-005)
// Notes: Pure data; testable without an audio device; loop region is [start, end)

use crate::time::{SampleDuration, SampleTime};
use serde::{Deserialize, Serialize};

// Playback state independent of loop configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportState {
    Stopped,
    Playing,
    Recording,
}

// Half-open loop region [start, end)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoopRegion {
    pub start: SampleTime,
    pub end: SampleTime,
}

impl LoopRegion {
    // Construct a validated region; end must exceed start
    pub fn new(start: SampleTime, end: SampleTime) -> Option<Self> {
        if end.0 > start.0 {
            Some(Self { start, end })
        } else {
            None
        }
    }

    // Half-open membership test
    pub fn contains(&self, pos: SampleTime) -> bool {
        pos.0 >= self.start.0 && pos.0 < self.end.0
    }

    // Region length in samples
    pub fn len(&self) -> SampleDuration {
        let samples = (i128::from(self.end.0) - i128::from(self.start.0)) as u64;
        SampleDuration::new(samples)
    }

    // Emptiness is impossible by construction
    pub fn is_empty(&self) -> bool {
        false
    }
}

// Commands the control layer may issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportCommand {
    Play,
    Stop,
    Record,
    Seek(SampleTime),
    SetLoop(Option<LoopRegion>),
}

// Transport position plus state plus loop configuration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transport {
    pub state: TransportState,
    pub position: SampleTime,
    pub loop_region: Option<LoopRegion>,
    pub loop_enabled: bool,
}

impl Transport {
    // Construct a stopped transport at zero
    pub fn new() -> Self {
        Self {
            state: TransportState::Stopped,
            position: SampleTime(0),
            loop_region: None,
            loop_enabled: false,
        }
    }

    // Apply one command deterministically
    pub fn apply(&mut self, cmd: TransportCommand) {
        match cmd {
            TransportCommand::Play => self.state = TransportState::Playing,
            TransportCommand::Record => self.state = TransportState::Recording,
            TransportCommand::Stop => self.state = TransportState::Stopped,
            TransportCommand::Seek(pos) => self.position = pos,
            TransportCommand::SetLoop(region) => {
                self.loop_region = region;
                self.loop_enabled = region.is_some();
            }
        }
    }

    // Advance by a block, wrapping inside an enabled loop; returns wrap count
    pub fn advance(&mut self, samples: SampleDuration) -> u32 {
        if self.state == TransportState::Stopped || samples.samples() == 0 {
            return 0;
        }
        match (self.loop_enabled, self.loop_region) {
            (true, Some(region)) if region.contains(self.position) => {
                let start = region.start.0 as i128;
                let length = region.end.0 as i128 - start;
                let total = self.position.0 as i128 - start + i128::from(samples.samples());
                let wraps = total / length;
                self.position = SampleTime((start + total.rem_euclid(length)) as i64);
                wraps.min(u32::MAX as i128) as u32
            }
            _ => {
                self.position = self.position.saturating_add_duration(samples);
                0
            }
        }
    }
}

impl Default for Transport {
    // Default matches new()
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn duration(samples: u64) -> SampleDuration {
        SampleDuration::new(samples)
    }

    // Loop end is exclusive: landing exactly on end wraps to start
    #[test]
    fn loop_end_is_exclusive() {
        let mut t = Transport::new();
        t.apply(TransportCommand::SetLoop(LoopRegion::new(
            SampleTime(100),
            SampleTime(200),
        )));
        t.apply(TransportCommand::Seek(SampleTime(150)));
        t.apply(TransportCommand::Play);
        let wraps = t.advance(duration(50));
        assert_eq!(wraps, 1);
        assert_eq!(t.position, SampleTime(100));
    }

    // Multiple wraps in one large block are counted
    #[test]
    fn large_block_wraps_multiple_times() {
        let mut t = Transport::new();
        t.apply(TransportCommand::SetLoop(LoopRegion::new(
            SampleTime(0),
            SampleTime(10),
        )));
        t.apply(TransportCommand::Play);
        let wraps = t.advance(duration(35));
        assert_eq!(wraps, 3);
        assert_eq!(t.position, SampleTime(5));
    }

    // Huge advances wrap in constant time and saturate the public wrap count
    #[test]
    fn huge_advance_over_tiny_loop_is_bounded() {
        let mut t = Transport::new();
        t.apply(TransportCommand::SetLoop(LoopRegion::new(
            SampleTime(0),
            SampleTime(1),
        )));
        t.apply(TransportCommand::Play);
        assert_eq!(t.advance(duration(i64::MAX as u64)), u32::MAX);
        assert_eq!(t.position, SampleTime(0));
    }

    // Non-looping transport saturates rather than overflowing
    #[test]
    fn non_looping_advance_saturates_at_sample_limit() {
        let mut t = Transport::new();
        t.apply(TransportCommand::Seek(SampleTime(i64::MAX - 5)));
        t.apply(TransportCommand::Play);
        assert_eq!(t.advance(duration(10)), 0);
        assert_eq!(t.position, SampleTime(i64::MAX));
    }

    // Outside the loop region playback does not wrap
    #[test]
    fn no_wrap_outside_region() {
        let mut t = Transport::new();
        t.apply(TransportCommand::SetLoop(LoopRegion::new(
            SampleTime(100),
            SampleTime(200),
        )));
        t.apply(TransportCommand::Seek(SampleTime(300)));
        t.apply(TransportCommand::Play);
        assert_eq!(t.advance(duration(50)), 0);
        assert_eq!(t.position, SampleTime(350));
    }

    // Stopped transport does not move
    #[test]
    fn stopped_transport_holds_position() {
        let mut t = Transport::new();
        t.apply(TransportCommand::Seek(SampleTime(42)));
        assert_eq!(t.advance(duration(512)), 0);
        assert_eq!(t.position, SampleTime(42));
    }

    // Degenerate loop regions are unrepresentable
    #[test]
    fn degenerate_loop_rejected() {
        assert!(LoopRegion::new(SampleTime(5), SampleTime(5)).is_none());
        assert!(LoopRegion::new(SampleTime(9), SampleTime(5)).is_none());
    }
}
