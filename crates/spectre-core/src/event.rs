// Author: Jeff
// Date: 2026-07-11
// Description: Timestamped event model with deterministic same-sample ordering (TIME-002)
// Notes: Ordering rank: transport > note-off > note-on > control > parameter

use crate::id::ObjectId;
use crate::time::SampleTime;
use serde::{Deserialize, Serialize};

// Event payloads the kernel understands
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    TransportSeek { to: SampleTime },
    NoteOff { channel: u8, key: u8 },
    NoteOn { channel: u8, key: u8, velocity: u8 },
    Control { channel: u8, cc: u8, value: u8 },
    ParamChange { param: ObjectId, value: f64 },
}

impl EventKind {
    // Rank for deterministic same-sample ordering
    fn order_rank(&self) -> u8 {
        match self {
            EventKind::TransportSeek { .. } => 0,
            EventKind::NoteOff { .. } => 1,
            EventKind::NoteOn { .. } => 2,
            EventKind::Control { .. } => 3,
            EventKind::ParamChange { .. } => 4,
        }
    }
}

// An event bound to an absolute sample time with a stable tiebreak sequence
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TimedEvent {
    pub time: SampleTime,
    pub sequence: u64,
    pub kind: EventKind,
}

// Convenience alias for the payload
pub type Event = EventKind;

impl TimedEvent {
    // Private typed ordering key: time, then kind rank, then arrival sequence
    fn sort_key(&self) -> (SampleTime, u8, u64) {
        (self.time, self.kind.order_rank(), self.sequence)
    }
}

// Sort a buffer of events into deterministic processing order
pub fn sort_events(events: &mut [TimedEvent]) {
    events.sort_by_key(|e| e.sort_key());
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to build an event
    fn ev(time: i64, sequence: u64, kind: EventKind) -> TimedEvent {
        TimedEvent {
            time: SampleTime(time),
            sequence,
            kind,
        }
    }

    // Note-off precedes note-on at the same sample so retriggers are unambiguous
    #[test]
    fn same_sample_off_before_on() {
        let on = EventKind::NoteOn {
            channel: 0,
            key: 60,
            velocity: 100,
        };
        let off = EventKind::NoteOff {
            channel: 0,
            key: 60,
        };
        let mut buf = vec![ev(100, 1, on), ev(100, 2, off)];
        sort_events(&mut buf);
        assert_eq!(buf[0].kind, off);
        assert_eq!(buf[1].kind, on);
    }

    // Transport events lead everything at the same sample
    #[test]
    fn transport_leads_same_sample() {
        let seek = EventKind::TransportSeek { to: SampleTime(0) };
        let off = EventKind::NoteOff {
            channel: 0,
            key: 60,
        };
        let mut buf = vec![ev(5, 1, off), ev(5, 2, seek)];
        sort_events(&mut buf);
        assert_eq!(buf[0].kind, seek);
    }

    // Equal time and kind fall back to arrival sequence
    #[test]
    fn sequence_breaks_ties() {
        let a = EventKind::Control {
            channel: 0,
            cc: 1,
            value: 10,
        };
        let b = EventKind::Control {
            channel: 0,
            cc: 1,
            value: 20,
        };
        let mut buf = vec![ev(7, 9, b), ev(7, 3, a)];
        sort_events(&mut buf);
        assert_eq!(buf[0].kind, a);
        assert_eq!(buf[1].kind, b);
    }
}
