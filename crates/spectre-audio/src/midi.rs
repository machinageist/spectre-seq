// Author: Jeff
// Date: 2026-08-09
// Description: Timestamped MIDI ingress converting driver messages into block-relative note events
// Notes: MIDI carries no note identity, so ingress allocates a nonzero ID per note-on and matches
//   the note-off by (channel, note) from a fixed table. Ordering at equal timestamps reuses the
//   accepted contract key exactly — (frame_offset, NoteEventKind::rank, sequence) — instead of
//   restating it. Sorting uses sort_unstable_by, which does not allocate; the key is a total
//   order because sequence is unique, so the result is deterministic despite being unstable.

use spectre_core::SampleTime;
use spectre_dsp::{NoteEvent, NoteEventKind};

// One device event bus admits at most this many events per quantum, matching the DSP contract
pub const MAX_BLOCK_EVENTS: usize = 1_024;

// MIDI channel and note-number space, sized for the active-note table
const CHANNELS: usize = 16;
const NOTES: usize = 128;

// Status nibbles and controller numbers this ingress models
const STATUS_NOTE_OFF: u8 = 0x80;
const STATUS_NOTE_ON: u8 = 0x90;
const STATUS_CONTROL_CHANGE: u8 = 0xB0;
const CONTROLLER_ALL_NOTES_OFF: u8 = 123;
const MAX_VELOCITY: f32 = 127.0;

// One timestamped short MIDI message as delivered by a driver
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MidiMessage {
    // Absolute sample-domain time the message applies at
    pub timestamp: SampleTime,
    pub status: u8,
    pub data1: u8,
    pub data2: u8,
}

impl MidiMessage {
    // Build a short message at an absolute sample position
    pub fn new(timestamp: SampleTime, status: u8, data1: u8, data2: u8) -> Self {
        Self {
            timestamp,
            status,
            data1,
            data2,
        }
    }
}

// Why a message did not become a block event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiError {
    // Message type this v1 event vocabulary does not model
    Unsupported,
    // Timestamp lands at or beyond the end of the current block
    AfterBlock,
    // Block event buffer is full; the message is refused, not silently dropped
    BufferFull,
    // Note-off with no matching active note-on
    UnmatchedNoteOff,
    // Channel or note outside the MIDI range
    OutOfRange,
}

// Converts timestamped MIDI into sorted, contract-valid block events
pub struct MidiIngress {
    // Active note ID per (channel, note); zero means no sounding note
    active: Box<[u32]>,
    // Preallocated block buffer; never resized after construction
    pending: Vec<NoteEvent>,
    next_id: u32,
    sequence: u64,
    // Messages refused because they arrived after the block they belong to
    late_messages: u64,
}

impl Default for MidiIngress {
    // Build with the contract's per-quantum event capacity
    fn default() -> Self {
        Self::new(MAX_BLOCK_EVENTS)
    }
}

impl MidiIngress {
    // Build with a bounded per-block event capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            active: vec![0; CHANNELS * NOTES].into_boxed_slice(),
            pending: Vec::with_capacity(capacity.clamp(1, MAX_BLOCK_EVENTS)),
            // Note IDs are nonzero, matching the zero-excluded identity rule
            next_id: 1,
            sequence: 0,
            late_messages: 0,
        }
    }

    // Discard the previous block's events without releasing the buffer
    pub fn begin_block(&mut self) {
        self.pending.clear();
    }

    // Convert one message into a block event, returning why it was refused
    pub fn push(
        &mut self,
        message: MidiMessage,
        block_start: SampleTime,
        frames: usize,
    ) -> Result<(), MidiError> {
        if self.pending.len() == self.pending.capacity() {
            return Err(MidiError::BufferFull);
        }
        let offset = message.timestamp.0 - block_start.0;
        if offset >= frames as i64 {
            return Err(MidiError::AfterBlock);
        }
        // A message timestamped before this block is late, not invalid: clamping it to the
        // block start plays it as soon as possible instead of discarding a real performance
        let frame_offset = if offset < 0 {
            self.late_messages += 1;
            0
        } else {
            offset as usize
        };

        let kind = self.classify(message)?;
        self.pending.push(NoteEvent {
            frame_offset,
            sequence: self.sequence,
            kind,
        });
        self.sequence += 1;
        Ok(())
    }

    // Sort the block's events into the order block validation requires
    pub fn block_events(&mut self) -> &[NoteEvent] {
        // sort_unstable_by does not allocate; the key is total because sequence is unique
        self.pending.sort_unstable_by(|left, right| {
            (left.frame_offset, left.kind.rank(), left.sequence).cmp(&(
                right.frame_offset,
                right.kind.rank(),
                right.sequence,
            ))
        });
        &self.pending
    }

    // Count messages that arrived after the block they were timestamped for
    pub fn late_messages(&self) -> u64 {
        self.late_messages
    }

    // Translate a short message into the v1 event vocabulary
    fn classify(&mut self, message: MidiMessage) -> Result<NoteEventKind, MidiError> {
        let channel = message.status & 0x0F;
        let status = message.status & 0xF0;
        if message.data1 > 127 || message.data2 > 127 {
            return Err(MidiError::OutOfRange);
        }

        match status {
            // Running-status note-on with zero velocity is the conventional note-off
            STATUS_NOTE_ON if message.data2 > 0 => {
                let id = self.take_id();
                self.slot(channel, message.data1)?;
                self.active[Self::index(channel, message.data1)] = id;
                Ok(NoteEventKind::On {
                    id,
                    channel,
                    note: message.data1,
                    velocity: f32::from(message.data2) / MAX_VELOCITY,
                })
            }
            STATUS_NOTE_OFF | STATUS_NOTE_ON => {
                self.slot(channel, message.data1)?;
                let slot = Self::index(channel, message.data1);
                let id = self.active[slot];
                if id == 0 {
                    return Err(MidiError::UnmatchedNoteOff);
                }
                self.active[slot] = 0;
                Ok(NoteEventKind::Off {
                    id,
                    channel,
                    note: message.data1,
                    velocity: f32::from(message.data2) / MAX_VELOCITY,
                })
            }
            STATUS_CONTROL_CHANGE if message.data1 == CONTROLLER_ALL_NOTES_OFF => {
                self.release_channel(channel);
                Ok(NoteEventKind::AllNotesOff {
                    channel: Some(channel),
                })
            }
            _ => Err(MidiError::Unsupported),
        }
    }

    // Allocate the next nonzero note ID, skipping zero on wrap
    fn take_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        id
    }

    // Reject channel/note pairs outside the MIDI range
    fn slot(&self, channel: u8, note: u8) -> Result<(), MidiError> {
        if usize::from(channel) >= CHANNELS || usize::from(note) >= NOTES {
            return Err(MidiError::OutOfRange);
        }
        Ok(())
    }

    // Flatten a channel/note pair into the active-note table index
    fn index(channel: u8, note: u8) -> usize {
        usize::from(channel) * NOTES + usize::from(note)
    }

    // Clear every sounding note on one channel
    fn release_channel(&mut self, channel: u8) {
        let base = usize::from(channel) * NOTES;
        self.active[base..base + NOTES].fill(0);
    }
}
