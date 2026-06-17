// Author: Jeff
// Date: 2026-05-27
// Description: Musical, MIDI, transport, and parameter events crossing block boundaries.
// Notes: Event queues are bounded before the audio callback consumes them.

use crate::ids::ParamId;

// Sentinel note id meaning the host did not assign a unique voice id
// Matches the CLAP convention so plugin interop stays lossless
pub const NOTE_ID_UNSPECIFIED: i32 = -1;

// Whether a note begins or ends
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum NoteEventKind {
    On,
    Off,
}

// Musical note event timestamped within the current block
// Copy and allocation-free so it lives in fixed-size realtime queues
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct NoteEvent {
    pub sample_offset: u32,
    pub kind: NoteEventKind,
    // Unique voice id or NOTE_ID_UNSPECIFIED
    pub note_id: i32,
    // MIDI channel 0..15
    pub channel: u8,
    // MIDI key 0..127
    pub key: u8,
    // Normalized 0.0..1.0 velocity
    pub velocity: f32,
}

impl NoteEvent {
    // Build a note-on at the given block offset
    pub fn on(sample_offset: u32, channel: u8, key: u8, velocity: f32) -> Self {
        Self {
            sample_offset,
            kind: NoteEventKind::On,
            note_id: NOTE_ID_UNSPECIFIED,
            channel,
            key,
            velocity: velocity.clamp(0.0, 1.0),
        }
    }

    // Build a note-off at the given block offset
    pub fn off(sample_offset: u32, channel: u8, key: u8) -> Self {
        Self {
            sample_offset,
            kind: NoteEventKind::Off,
            note_id: NOTE_ID_UNSPECIFIED,
            channel,
            key,
            velocity: 0.0,
        }
    }

    // Report whether this event lands inside a block of the given length
    #[inline]
    pub fn in_block(&self, block_frames: u32) -> bool {
        self.sample_offset < block_frames
    }
}

// Longest MIDI 1.0 channel-voice message this packet carries
pub const MIDI_MAX_BYTES: usize = 3;

// Raw MIDI 1.0 message timestamped within the current block
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct MidiEvent {
    pub sample_offset: u32,
    pub data: [u8; MIDI_MAX_BYTES],
    // Count of valid bytes in `data`, 1..=3
    pub len: u8,
}

impl MidiEvent {
    // Build a MIDI event from up to three status/data bytes
    pub fn new(sample_offset: u32, bytes: &[u8]) -> Option<Self> {
        let len = bytes.len();
        if len == 0 || len > MIDI_MAX_BYTES {
            return None;
        }
        let mut data = [0u8; MIDI_MAX_BYTES];
        data[..len].copy_from_slice(bytes);
        Some(Self {
            sample_offset,
            data,
            len: len as u8,
        })
    }

    // Borrow the valid byte span of the message
    #[inline]
    pub fn bytes(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }

    // Report whether this event lands inside a block of the given length
    #[inline]
    pub fn in_block(&self, block_frames: u32) -> bool {
        self.sample_offset < block_frames
    }
}

// Transport command applied at a block boundary
// Sample-accurate seeking lives in the timeline layer, not core
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TransportEvent {
    Play,
    Stop,
    Record(bool),
    Loop(bool),
    Seek { sample: u64 },
    Tempo { bpm: f64 },
    TimeSignature { numerator: u16, denominator: u16 },
}

// Parameter value change timestamped within the current block
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ParameterChange {
    pub sample_offset: u32,
    pub param: ParamId,
    // Plain value in the parameter's declared range
    pub value: f32,
}

impl ParameterChange {
    // Report whether this change lands inside a block of the given length
    #[inline]
    pub fn in_block(&self, block_frames: u32) -> bool {
        self.sample_offset < block_frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_helpers_clamp_velocity_and_set_kind() {
        let on = NoteEvent::on(10, 0, 60, 2.0);
        assert_eq!(on.kind, NoteEventKind::On);
        assert_eq!(on.velocity, 1.0);
        assert_eq!(on.note_id, NOTE_ID_UNSPECIFIED);

        let off = NoteEvent::off(20, 0, 60);
        assert_eq!(off.kind, NoteEventKind::Off);
        assert_eq!(off.velocity, 0.0);
    }

    #[test]
    fn note_offset_validation() {
        let ev = NoteEvent::on(127, 0, 60, 1.0);
        assert!(ev.in_block(128));
        assert!(!ev.in_block(128 - 1));
    }

    #[test]
    fn midi_packs_valid_byte_spans() {
        let ev = MidiEvent::new(0, &[0x90, 60, 100]).unwrap();
        assert_eq!(ev.bytes(), &[0x90, 60, 100]);
        assert_eq!(ev.len, 3);

        let one = MidiEvent::new(0, &[0xF8]).unwrap();
        assert_eq!(one.bytes(), &[0xF8]);
    }

    #[test]
    fn midi_rejects_empty_and_oversized() {
        assert!(MidiEvent::new(0, &[]).is_none());
        assert!(MidiEvent::new(0, &[1, 2, 3, 4]).is_none());
    }

    #[test]
    fn parameter_change_offset_validation() {
        let pc = ParameterChange {
            sample_offset: 64,
            param: ParamId::new(3),
            value: 0.5,
        };
        assert!(pc.in_block(128));
        assert!(!pc.in_block(64));
    }
}
