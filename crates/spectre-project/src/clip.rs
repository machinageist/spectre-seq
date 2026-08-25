// Author: Jeff
// Date: 2026-08-25
// Description: Project-owned MIDI clip data in the musical tick domain
// Notes: App-thread only; never callback-reachable. Positions are BeatTicks (decision 5) and are
//   converted to samples exactly once, at bake time, through TempoMap. Nothing here defines an
//   ordering key: the render-side key is spectre_dsp::NoteEventKind::rank and is reused as-is.

use serde::{Deserialize, Serialize};
use spectre_core::{BeatTicks, ObjectId, TICKS_PER_BEAT};

// Notes one clip may carry.
// Rationale row CLIP-001 in docs/01-requirements/requirements-ledger.md (PROD-003, decision 16).
// Derived from Spectre's own cost, not from a reference product. One baked note occupies 32 bytes
// (spectre_audio::ScheduledNote); 4,096 x 32 B = 128 KiB per track schedule. MAX_TRACKS is 16, so
// a full project schedule is 2 MiB resident, and 4 MiB while one retired schedule per bus is in
// flight on the reclaim lane. Musically: 4,096 notes on a 1/16 grid (240 ticks) spans
// 4,096 x 240 = 983,040 ticks = 1,024 beats = 256 bars of 4/4
pub const MAX_NOTES_PER_CLIP: usize = 4_096;

// Clip placements one track may carry.
// Rationale row CLIP-002. Derived from MAX_NOTES_PER_CLIP, not chosen independently: the binding
// bound is 4,096 baked notes per track, and 64 placements is the point at which the average
// placement carries 4,096 / 64 = 64 notes — the smallest average that still represents musical
// material rather than fragments. Above 64 the note bound binds first anyway
pub const MAX_CLIPS_PER_TRACK: usize = 64;

// Longest clip a v1 project may contain, in BeatTicks at 960 PPQ.
// Rationale row CLIP-003. Derived from TIME-003's own 24-hour project floor evaluated at Spectre's
// own MAX_BPM = 1200 (crates/spectre-core/src/tempo.rs): 24 h x 3,600 s = 86,400 s;
// 86,400 s x 1,200 BPM / 60 = 1,728,000 beats; 1,728,000 x TICKS_PER_BEAT = 1,658,880,000 ticks.
// A clip cannot usefully outlast the longest project the accepted time contract must represent
pub const MAX_CLIP_LENGTH_TICKS: i64 = 1_728_000 * TICKS_PER_BEAT;

// App-thread clip failure; every variant leaves the model unmutated
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClipError {
    BlankName,
    NonPositiveLength,
    ClipTooLong { ticks: i64, limit: i64 },
    NoteOutsideClip { start: i64, length: i64 },
    NoteLimit { limit: usize },
    ClipLimit { limit: usize },
    OverlappingPlacement { existing: ObjectId },
    InvalidChannel(u8),
    InvalidNote(u8),
    InvalidVelocity(f32),
    UnknownClip(ObjectId),
    UnknownPlacement(ObjectId),
    DuplicateId(ObjectId),
}

impl std::fmt::Display for ClipError {
    // Render an actionable app-thread diagnostic
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BlankName => formatter.write_str("clip name must not be blank"),
            Self::NonPositiveLength => {
                formatter.write_str("length must be greater than zero ticks")
            }
            Self::ClipTooLong { ticks, limit } => {
                write!(formatter, "{ticks} ticks exceeds the clip limit of {limit}")
            }
            Self::NoteOutsideClip { start, length } => write!(
                formatter,
                "a note at tick {start} falls outside a clip {length} ticks long"
            ),
            Self::NoteLimit { limit } => {
                write!(formatter, "a clip may hold at most {limit} notes")
            }
            Self::ClipLimit { limit } => {
                write!(formatter, "a track may hold at most {limit} clips")
            }
            Self::OverlappingPlacement { existing } => write!(
                formatter,
                "the placement overlaps placement {}",
                existing.raw()
            ),
            Self::InvalidChannel(channel) => write!(formatter, "channel {channel} exceeds 15"),
            Self::InvalidNote(note) => write!(formatter, "note {note} exceeds 127"),
            Self::InvalidVelocity(velocity) => {
                write!(
                    formatter,
                    "velocity {velocity} is not finite within 0.0..=1.0"
                )
            }
            Self::UnknownClip(id) => write!(formatter, "unknown clip {}", id.raw()),
            Self::UnknownPlacement(id) => write!(formatter, "unknown placement {}", id.raw()),
            Self::DuplicateId(id) => write!(formatter, "id {} is already in use", id.raw()),
        }
    }
}

impl std::error::Error for ClipError {}

// Validate one note's channel, number, and velocity against the accepted event contract.
// These bounds are NOT re-derived here: they are exactly what NoteEvent::is_valid enforces
// (crates/spectre-dsp/src/io.rs), checked here so an invalid note can never be persisted, and
// checked again there because the plan does not trust its callers
fn validate_note_fields(channel: u8, note: u8, velocity: f32) -> Result<(), ClipError> {
    if channel > 15 {
        return Err(ClipError::InvalidChannel(channel));
    }
    if note > 127 {
        return Err(ClipError::InvalidNote(note));
    }
    if !velocity.is_finite() || !(0.0..=1.0).contains(&velocity) {
        return Err(ClipError::InvalidVelocity(velocity));
    }
    Ok(())
}

// One note inside a clip, positioned in the clip's own tick domain
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClipNote {
    start: BeatTicks,
    length: BeatTicks,
    channel: u8,
    note: u8,
    velocity: f32,
}

impl ClipNote {
    // Reject a note the accepted event contract could not carry
    pub fn new(
        start: BeatTicks,
        length: BeatTicks,
        channel: u8,
        note: u8,
        velocity: f32,
    ) -> Result<Self, ClipError> {
        if length.0 <= 0 {
            return Err(ClipError::NonPositiveLength);
        }
        if start.0 < 0 {
            return Err(ClipError::NoteOutsideClip {
                start: start.0,
                length: 0,
            });
        }
        validate_note_fields(channel, note, velocity)?;
        Ok(Self {
            start,
            length,
            channel,
            note,
            velocity,
        })
    }

    pub fn start(self) -> BeatTicks {
        self.start
    }

    pub fn length(self) -> BeatTicks {
        self.length
    }

    // start + length; half-open [start, end) per TIME-002
    pub fn end(self) -> BeatTicks {
        BeatTicks(self.start.0.saturating_add(self.length.0))
    }

    pub fn channel(self) -> u8 {
        self.channel
    }

    pub fn note(self) -> u8 {
        self.note
    }

    pub fn velocity(self) -> f32 {
        self.velocity
    }
}

// One clip: identity, name, length, and its notes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MidiClip {
    id: ObjectId,
    name: String,
    length: BeatTicks,
    notes: Vec<ClipNote>,
}

impl MidiClip {
    // Caller owns ID allocation from the project IdGen (CORE-001)
    pub fn new(id: ObjectId, name: &str, length: BeatTicks) -> Result<Self, ClipError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(ClipError::BlankName);
        }
        validate_length(length)?;
        Ok(Self {
            id,
            name: name.to_string(),
            length,
            notes: Vec::new(),
        })
    }

    pub fn id(&self) -> ObjectId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn length(&self) -> BeatTicks {
        self.length
    }

    // Sorted by (start, note). Sorting here is a data-model convenience for the note list and for
    // a deterministic bake order; it is NOT the event-ordering contract and uses no rank
    pub fn notes(&self) -> &[ClipNote] {
        &self.notes
    }

    pub fn note_count(&self) -> usize {
        self.notes.len()
    }

    pub fn set_name(&mut self, name: &str) -> Result<(), ClipError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(ClipError::BlankName);
        }
        self.name = name.to_string();
        Ok(())
    }

    // Shorten or lengthen the clip. Shortening refuses rather than silently dropping the notes it
    // would orphan: a destructive edit disguised as a resize is how material disappears
    pub fn set_length(&mut self, length: BeatTicks) -> Result<(), ClipError> {
        validate_length(length)?;
        if let Some(orphan) = self.notes.iter().find(|note| note.end().0 > length.0) {
            return Err(ClipError::NoteOutsideClip {
                start: orphan.start.0,
                length: length.0,
            });
        }
        self.length = length;
        Ok(())
    }

    // Refuses past MAX_NOTES_PER_CLIP and refuses a note outside [0, length); returns the index
    // the note landed at, so an undo can remove exactly it
    pub fn insert_note(&mut self, note: ClipNote) -> Result<usize, ClipError> {
        if self.notes.len() >= MAX_NOTES_PER_CLIP {
            return Err(ClipError::NoteLimit {
                limit: MAX_NOTES_PER_CLIP,
            });
        }
        if note.start.0 < 0 || note.end().0 > self.length.0 {
            return Err(ClipError::NoteOutsideClip {
                start: note.start.0,
                length: self.length.0,
            });
        }
        let index = self
            .notes
            .partition_point(|held| (held.start.0, held.note) <= (note.start.0, note.note));
        self.notes.insert(index, note);
        Ok(index)
    }

    // Remove by index, returning the note whole so an undo can reinsert it unchanged
    pub fn remove_note(&mut self, index: usize) -> Result<ClipNote, ClipError> {
        if index >= self.notes.len() {
            return Err(ClipError::NoteLimit {
                limit: self.notes.len(),
            });
        }
        Ok(self.notes.remove(index))
    }
}

// One placement of one clip on a track's timeline
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClipPlacement {
    id: ObjectId,
    clip: ObjectId,
    start: BeatTicks,
    active: bool,
}

impl ClipPlacement {
    pub fn new(id: ObjectId, clip: ObjectId, start: BeatTicks) -> Result<Self, ClipError> {
        if start.0 < 0 {
            return Err(ClipError::NoteOutsideClip {
                start: start.0,
                length: 0,
            });
        }
        Ok(Self {
            id,
            clip,
            start,
            active: true,
        })
    }

    pub fn id(&self) -> ObjectId {
        self.id
    }

    pub fn clip(&self) -> ObjectId {
        self.clip
    }

    pub fn start(&self) -> BeatTicks {
        self.start
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

// One track's ordered, non-overlapping placements.
//
// Non-overlapping is an invariant, not a policy preference: R4's instruments are monophonic —
// PulseInstrument and Filament each hold one active voice and a second attack replaces the first —
// so overlapping placements would produce a defined-but-musically-wrong result. Supporting them
// properly needs polyphony, which is a later milestone's decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackClips {
    // Sorted by start, non-overlapping. The stored length is the placed clip's length at insert
    // time, kept alongside so overlap can be decided without a lookup into the clip table
    placements: Vec<ClipPlacement>,
    lengths: Vec<BeatTicks>,
    // Advances on any edit that changes what the baked schedule would contain. Session-local, so
    // a reload does not inherit a stale revision
    #[serde(skip)]
    bake_revision: u64,
}

// Equality is over document content only. bake_revision is #[serde(skip)] session bookkeeping,
// so a derived PartialEq would make a saved-and-reloaded collection compare unequal to the one it
// came from — the exact comparison R4-7's persistence evidence rests on
impl PartialEq for TrackClips {
    fn eq(&self, other: &Self) -> bool {
        self.placements == other.placements && self.lengths == other.lengths
    }
}

impl Default for TrackClips {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackClips {
    pub fn new() -> Self {
        Self {
            placements: Vec::new(),
            lengths: Vec::new(),
            bake_revision: 0,
        }
    }

    // Sorted by start, non-overlapping
    pub fn placements(&self) -> &[ClipPlacement] {
        &self.placements
    }

    // The placed length of the placement at `index`, in ticks
    pub fn length_at(&self, index: usize) -> Option<BeatTicks> {
        self.lengths.get(index).copied()
    }

    pub fn len(&self) -> usize {
        self.placements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.placements.is_empty()
    }

    pub fn bake_revision(&self) -> u64 {
        self.bake_revision
    }

    pub fn get(&self, id: ObjectId) -> Option<&ClipPlacement> {
        self.placements.iter().find(|entry| entry.id == id)
    }

    // Mutable access advances the bake revision unconditionally, because every settable field on
    // a placement — active, and any later position edit — changes what the schedule would contain
    pub fn get_mut(&mut self, id: ObjectId) -> Option<&mut ClipPlacement> {
        let found = self.placements.iter_mut().find(|entry| entry.id == id);
        if found.is_some() {
            self.bake_revision += 1;
        }
        found
    }

    // Refuses past MAX_CLIPS_PER_TRACK, refuses a duplicate id, and refuses a placement
    // overlapping an existing one. Adjacency is legal: intervals are half-open [start, start+len)
    pub fn insert(
        &mut self,
        placement: ClipPlacement,
        length: BeatTicks,
    ) -> Result<usize, ClipError> {
        if self.placements.len() >= MAX_CLIPS_PER_TRACK {
            return Err(ClipError::ClipLimit {
                limit: MAX_CLIPS_PER_TRACK,
            });
        }
        validate_length(length)?;
        if self.placements.iter().any(|held| held.id == placement.id) {
            return Err(ClipError::DuplicateId(placement.id));
        }
        let end = placement.start.0.saturating_add(length.0);
        for (held, held_length) in self.placements.iter().zip(&self.lengths) {
            let held_end = held.start.0.saturating_add(held_length.0);
            if placement.start.0 < held_end && held.start.0 < end {
                return Err(ClipError::OverlappingPlacement { existing: held.id });
            }
        }
        let index = self
            .placements
            .partition_point(|held| held.start.0 <= placement.start.0);
        self.placements.insert(index, placement);
        self.lengths.insert(index, length);
        self.bake_revision += 1;
        Ok(index)
    }

    // Remove by identity, returning the placement whole so an undo can reinsert it unchanged
    pub fn remove(&mut self, id: ObjectId) -> Result<ClipPlacement, ClipError> {
        let index = self
            .placements
            .iter()
            .position(|entry| entry.id == id)
            .ok_or(ClipError::UnknownPlacement(id))?;
        self.lengths.remove(index);
        self.bake_revision += 1;
        Ok(self.placements.remove(index))
    }
}

// One length check, used by every constructor and setter that accepts one
fn validate_length(length: BeatTicks) -> Result<(), ClipError> {
    if length.0 <= 0 {
        return Err(ClipError::NonPositiveLength);
    }
    if length.0 > MAX_CLIP_LENGTH_TICKS {
        return Err(ClipError::ClipTooLong {
            ticks: length.0,
            limit: MAX_CLIP_LENGTH_TICKS,
        });
    }
    Ok(())
}
