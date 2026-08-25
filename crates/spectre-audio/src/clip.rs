// Author: Jeff
// Date: 2026-08-25
// Description: Baked sample-domain clip schedule and the render-thread player that emits its notes
// Notes: The note-side analogue of CompiledPlan — immutable, preallocated on the app thread,
//   executed on the render thread, reclaimed off-thread. It emits spectre_dsp::NoteEvent directly
//   and defines NO ordering key: sorting uses NoteEventKind::rank, which is public for this reason.

use spectre_core::{BeatTicks, SampleRate, SampleTime, TempoMap, Transport, TransportState};
use spectre_dsp::{NoteEvent, NoteEventKind};

// Clip events one note bus may contribute to one render quantum.
// Rationale row CLIP-004 in docs/01-requirements/requirements-ledger.md (PROD-003, decision 16).
// Derived from the accepted per-quantum ceiling, split so neither producer can starve the other:
// MAX_NOTE_EVENTS_PER_BLOCK is 1,024 and exceeding it returns ProcessError::EventCapacity, which
// silences the whole block. Half is reserved for clip playback and half stays available to live
// MIDI ingress, whose own bound is MAX_BLOCK_EVENTS = 1,024.
// Cross-check against Spectre's own worst legal case: at MAX_BPM = 1200 and 48 kHz,
// samples_per_tick = 48,000 x 60 / (1,200 x 960) = 2.5, so a 256-frame block spans 102.4 ticks.
// With notes at the finest representable spacing of one tick AND one note per tick, that is at
// most 102 attacks and 102 releases = 204 events. 512 clears the densest MONOPHONIC block
// Spectre's own tempo and tick bounds admit, with 2.5x margin.
// What that does NOT bound is polyphony: notes stacked at a shared start tick are representable,
// so a ten-note chord on every tick of that block would exceed the reserve. Density is bounded by
// refusal, not by this number — the block emits AllNotesOff and counts, rather than a partial set
pub const CLIP_EVENT_RESERVE: usize = 512;

// Contiguous transport segments the player will walk within one block.
// Rationale row CLIP-005. RT-001 requires bounded work on the callback, and Transport::advance
// already tolerates many wraps per block. Two segments means at most one loop wrap per block,
// which is what ordinary looping needs. The material this refuses is precisely a loop region
// shorter than one granted block, and the granted block is a driver property negotiated by R4-1,
// not a Spectre constant. At 48 kHz and 960 PPQ a block spans frames x BPM / 3,000 ticks: 256
// frames is 102.4 ticks at MAX_BPM and 10.2 ticks at 120 BPM. If a backend ever negotiates blocks
// above 1,024 frames this must be derived from plan.max_frames() rather than left at 2.
// Refusal is counted and fails closed to AllNotesOff, never to unbounded work
pub const MAX_BLOCK_SEGMENTS: usize = 2;

// Baked schedules that may be queued for the render thread at once.
// Rationale row CLIP-006. The value is 3 because 3 is the depth spsc::bounded actually delivers,
// and this constant must name the lane that exists rather than the lane the steady state would
// prefer: bounded computes requested = capacity.max(MIN_CAPACITY) + 1 and rounds up to a power of
// two, reporting capacity() = slots_len - 1. Every argument from 0 through 3 produces the same
// ring — 4 slots, usable capacity 3 — so 3 is the only request that equals what it receives.
// The steady state needs two: one schedule installed and in flight, one being published. The
// third slot is the primitive's floor, not a design allowance
pub const SCHEDULE_LANE_CAPACITY: usize = 3;

// First note ID reserved for clip playback.
// Rationale row CLIP-007. NoteEvent identity must be unique per sounding note on a bus, because
// the instruments match a release to its attack by id and MidiIngress keys its active table by
// (channel, note) with an ascending nonzero allocator starting at 1. Two allocators that both
// start at 1 would let a clip release cancel a live note of the same id. Reserving the upper half
// of u32 for clip-baked IDs makes the two spaces disjoint by construction, not by coordination
pub const CLIP_NOTE_ID_BASE: u32 = 0x8000_0000;

// Sequence band for clip events.
// Rationale row CLIP-008. Uniqueness across producers is by band assignment, not renumbering:
// MidiIngress numbers from 0 upward, so it occupies the low band. Reaching bit 63 would take
// 2^63 admitted messages — at a sustained million per second, about 292,000 years.
// The user-visible consequence, deliberate rather than accidental: at an identical frame offset
// and identical rank, a live-performance event resolves before a clip event
pub const CLIP_SEQUENCE_BAND: u64 = 1 << 63;

// App-thread bake failure; never constructed on the render path
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScheduleError {
    Capacity { requested: usize, limit: usize },
    NonPositiveLength,
    InvalidChannel(u8),
    InvalidNote(u8),
    InvalidVelocity(f32),
    IdSpaceExhausted,
}

impl std::fmt::Display for ScheduleError {
    // Render an actionable app-thread diagnostic
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Capacity { requested, limit } => {
                write!(
                    formatter,
                    "{requested} notes exceeds the bake limit of {limit}"
                )
            }
            Self::NonPositiveLength => {
                formatter.write_str("a baked note must have positive length")
            }
            Self::InvalidChannel(channel) => write!(formatter, "channel {channel} exceeds 15"),
            Self::InvalidNote(note) => write!(formatter, "note {note} exceeds 127"),
            Self::InvalidVelocity(velocity) => {
                write!(
                    formatter,
                    "velocity {velocity} is not finite within 0.0..=1.0"
                )
            }
            Self::IdSpaceExhausted => formatter.write_str("clip note ID space is exhausted"),
        }
    }
}

impl std::error::Error for ScheduleError {}

// Why a block's clip emission ended the way it did
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipBlockOutcome {
    // Every event for this block was emitted
    Complete,
    // The block's events would have exceeded CLIP_EVENT_RESERVE; AllNotesOff was emitted instead
    EventReserveExceeded,
    // The block spanned more than MAX_BLOCK_SEGMENTS; AllNotesOff was emitted instead
    SegmentLimitExceeded,
}

// One clip note resolved to absolute engine sample positions
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScheduledNote {
    pub start: SampleTime,
    // Half-open; end > start is an invariant checked at bake
    pub end: SampleTime,
    // >= CLIP_NOTE_ID_BASE, nonzero by construction
    pub id: u32,
    pub velocity: f32,
    pub channel: u8,
    pub note: u8,
}

// Immutable baked schedule for one note bus. Send, so it can cross to the audio thread
#[derive(Debug)]
pub struct ClipSchedule {
    // Sorted by (start, id)
    notes: Box<[ScheduledNote]>,
}

impl ClipSchedule {
    // Bake tick-domain notes into absolute sample positions on the app thread.
    // TempoMap::ticks_to_samples is the single conversion definition per TIME-001/TIME-003; this
    // function does no beat arithmetic of its own
    pub fn bake(
        notes: impl Iterator<Item = (BeatTicks, BeatTicks, u8, u8, f32)>,
        tempo: &TempoMap,
        rate: SampleRate,
        capacity: usize,
    ) -> Result<Self, ScheduleError> {
        let mut baked: Vec<ScheduledNote> = Vec::with_capacity(capacity.min(CLIP_EVENT_RESERVE));
        for (index, (start, length, channel, note, velocity)) in notes.enumerate() {
            if baked.len() >= capacity {
                return Err(ScheduleError::Capacity {
                    requested: index + 1,
                    limit: capacity,
                });
            }
            if length.0 <= 0 {
                return Err(ScheduleError::NonPositiveLength);
            }
            if channel > 15 {
                return Err(ScheduleError::InvalidChannel(channel));
            }
            if note > 127 {
                return Err(ScheduleError::InvalidNote(note));
            }
            if !velocity.is_finite() || !(0.0..=1.0).contains(&velocity) {
                return Err(ScheduleError::InvalidVelocity(velocity));
            }
            let id_offset = u32::try_from(index).map_err(|_| ScheduleError::IdSpaceExhausted)?;
            let id = CLIP_NOTE_ID_BASE
                .checked_add(id_offset)
                .ok_or(ScheduleError::IdSpaceExhausted)?;

            let start_sample = tempo.ticks_to_samples(start, rate);
            let end_sample = tempo.ticks_to_samples(BeatTicks(start.0 + length.0), rate);
            // A note shorter than one sample period at this rate would round to a zero-length
            // span and could never emit its Off after its On; widening by one sample keeps the
            // half-open invariant without inventing a minimum duration in the tick domain
            let end_sample = if end_sample.0 > start_sample.0 {
                end_sample
            } else {
                SampleTime(start_sample.0 + 1)
            };
            baked.push(ScheduledNote {
                start: start_sample,
                end: end_sample,
                id,
                velocity,
                channel,
                note,
            });
        }
        baked.sort_unstable_by_key(|note| (note.start.0, note.id));
        Ok(Self {
            notes: baked.into_boxed_slice(),
        })
    }

    // Empty schedule; renders exact silence and is the state before any clip exists
    pub fn empty() -> Self {
        Self {
            notes: Vec::new().into_boxed_slice(),
        }
    }

    pub fn notes(&self) -> &[ScheduledNote] {
        &self.notes
    }

    pub fn len(&self) -> usize {
        self.notes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.notes.is_empty()
    }
}

impl Default for ClipSchedule {
    fn default() -> Self {
        Self::empty()
    }
}

// One note the player has attacked and not yet released
#[derive(Debug, Clone, Copy)]
struct Sounding {
    id: u32,
    channel: u8,
    note: u8,
    end: SampleTime,
}

// Render-thread cursor and sounding-note table for one note bus.
// Owns no heap growth after construction; every collection is preallocated
pub struct ClipPlayer {
    schedule: Box<ClipSchedule>,
    // Fixed table, not a growing set. Capacity is the reserve, so it cannot exceed what one
    // block could legally emit
    sounding: Vec<Sounding>,
    // Set when the caller has decided every sounding note must be released at the next block's
    // frame 0: a seek, a stop, or a schedule swap
    release_pending: bool,
}

impl ClipPlayer {
    // Preallocate for the worst legal block
    pub fn new(reserve: usize) -> Self {
        Self {
            schedule: Box::new(ClipSchedule::empty()),
            sounding: Vec::with_capacity(reserve.max(1)),
            release_pending: false,
        }
    }

    pub fn schedule(&self) -> &ClipSchedule {
        &self.schedule
    }

    pub fn sounding_count(&self) -> usize {
        self.sounding.len()
    }

    // Install a new schedule, returning the retired one for off-thread reclamation.
    // Callback-safe: this is a pointer swap. The caller MUST hand the returned schedule to the
    // reclaim lane rather than drop it, because dropping it here would deallocate on the audio
    // thread and violate RT-001
    #[must_use]
    pub fn install(&mut self, schedule: Box<ClipSchedule>) -> Box<ClipSchedule> {
        // The notes of the outgoing schedule stop existing for the player, so anything it has
        // sounding must be released rather than left hanging on the instrument
        self.release_pending = true;
        std::mem::replace(&mut self.schedule, schedule)
    }

    // Release every sounding note at frame 0 of the next block: seek, stop, or schedule swap
    pub fn release_all(&mut self) {
        self.release_pending = true;
    }

    // Emit this block's clip events into `out`, in contract order, and return the outcome.
    // Callback-safe: no allocation, no locks, no I/O, no logging, no panics.
    // `out` is the bridge's note scratch, already cleared. `first_sequence` is the block-local
    // sequence base the bridge assigns
    pub fn emit_block(
        &mut self,
        transport: &Transport,
        frames: usize,
        out: &mut Vec<NoteEvent>,
        first_sequence: u64,
    ) -> ClipBlockOutcome {
        let mut sequence = first_sequence;
        if self.release_pending {
            self.release_pending = false;
            self.sounding.clear();
            push_all_notes_off(out, 0, sequence);
            sequence += 1;
        }
        if transport.state == TransportState::Stopped || frames == 0 {
            return ClipBlockOutcome::Complete;
        }

        // Walk the block as at most MAX_BLOCK_SEGMENTS contiguous transport spans. A loop wrap
        // splits the block; more than one wrap is refused rather than walked unboundedly
        let mut segments: [(SampleTime, usize, usize); MAX_BLOCK_SEGMENTS] =
            [(SampleTime(0), 0, 0); MAX_BLOCK_SEGMENTS];
        let count = match plan_segments(transport, frames, &mut segments) {
            Some(count) => count,
            None => {
                self.sounding.clear();
                out.clear();
                push_all_notes_off(out, 0, first_sequence);
                return ClipBlockOutcome::SegmentLimitExceeded;
            }
        };

        let base = out.len();
        for &(origin, offset, span) in &segments[..count] {
            if !self.emit_segment(origin, offset, span, out, &mut sequence) {
                self.sounding.clear();
                out.truncate(base);
                out.clear();
                push_all_notes_off(out, 0, first_sequence);
                return ClipBlockOutcome::EventReserveExceeded;
            }
        }
        // Emission walks the sounding table and then the schedule, so its output is grouped by
        // kind rather than by frame. Ordering it here — with the same key the bridge uses over
        // the merged array — is what makes this output independently valid rather than only
        // valid after the bridge has merged it. sort_unstable_by does not allocate
        out.sort_unstable_by(contract_order);
        ClipBlockOutcome::Complete
    }

    // Emit one contiguous span [origin, origin + span) landing at block frames [offset, ..).
    // Returns false when the reserve would be exceeded.
    //
    // Half-open throughout, per TIME-002: a note sounds on frames [start, end), so its release
    // belongs at frame `end` — the first frame it is silent. A note whose end lands exactly on
    // the span boundary therefore stays in the sounding table and releases at frame 0 of the next
    // block. Placing the release at end-1 instead would shorten every note by one sample
    fn emit_segment(
        &mut self,
        origin: SampleTime,
        offset: usize,
        span: usize,
        out: &mut Vec<NoteEvent>,
        sequence: &mut u64,
    ) -> bool {
        let end = origin.0.saturating_add(span as i64);

        // Releases for notes carried in from an earlier block or segment
        let mut index = 0;
        while index < self.sounding.len() {
            let held = self.sounding[index];
            if held.end.0 >= origin.0 && held.end.0 < end {
                if out.len() >= CLIP_EVENT_RESERVE {
                    return false;
                }
                push_off(
                    out,
                    offset + (held.end.0 - origin.0) as usize,
                    sequence,
                    held,
                );
                self.sounding.swap_remove(index);
                continue;
            }
            index += 1;
        }

        for note in self.schedule.notes.iter() {
            if note.start.0 < origin.0 || note.start.0 >= end {
                continue;
            }
            if out.len() >= CLIP_EVENT_RESERVE {
                return false;
            }
            out.push(NoteEvent {
                frame_offset: offset + (note.start.0 - origin.0) as usize,
                sequence: *sequence,
                kind: NoteEventKind::On {
                    id: note.id,
                    channel: note.channel,
                    note: note.note,
                    velocity: note.velocity,
                },
            });
            *sequence += 1;

            let held = Sounding {
                id: note.id,
                channel: note.channel,
                note: note.note,
                end: note.end,
            };
            // A note that both starts and ends inside this span releases in the same walk
            if note.end.0 < end {
                if out.len() >= CLIP_EVENT_RESERVE {
                    return false;
                }
                push_off(
                    out,
                    offset + (note.end.0 - origin.0) as usize,
                    sequence,
                    held,
                );
            } else {
                if self.sounding.len() >= self.sounding.capacity() {
                    return false;
                }
                self.sounding.push(held);
            }
        }
        true
    }
}

// Push one release for a sounding note
fn push_off(out: &mut Vec<NoteEvent>, frame_offset: usize, sequence: &mut u64, held: Sounding) {
    out.push(NoteEvent {
        frame_offset,
        sequence: *sequence,
        kind: NoteEventKind::Off {
            id: held.id,
            channel: held.channel,
            note: held.note,
            velocity: 0.0,
        },
    });
    *sequence += 1;
}

// The one ordering key for note events destined for one plan node in one quantum: the same tuple
// ProcessContext::new validates, comparing through NoteEventKind::rank rather than a second rank
// function of this crate's own. Defined once here and called by both the player (so its own
// output is independently valid) and the bridge (over the merged array)
pub fn contract_order(left: &NoteEvent, right: &NoteEvent) -> std::cmp::Ordering {
    (left.frame_offset, left.kind.rank(), left.sequence).cmp(&(
        right.frame_offset,
        right.kind.rank(),
        right.sequence,
    ))
}

// Push one all-notes-off at a frame offset
fn push_all_notes_off(out: &mut Vec<NoteEvent>, frame_offset: usize, sequence: u64) {
    out.push(NoteEvent {
        frame_offset,
        sequence,
        kind: NoteEventKind::AllNotesOff { channel: None },
    });
}

// Split the block into contiguous transport spans, or refuse past MAX_BLOCK_SEGMENTS.
// Reads the transport rather than advancing it: the bridge owns advancing, once, per block
fn plan_segments(
    transport: &Transport,
    frames: usize,
    out: &mut [(SampleTime, usize, usize); MAX_BLOCK_SEGMENTS],
) -> Option<usize> {
    let looping = transport.loop_enabled
        && transport
            .loop_region
            .is_some_and(|region| region.contains(transport.position));
    let Some(region) = transport.loop_region.filter(|_| looping) else {
        out[0] = (transport.position, 0, frames);
        return Some(1);
    };

    let remaining = region.end.0 - transport.position.0;
    if remaining >= frames as i64 {
        out[0] = (transport.position, 0, frames);
        return Some(1);
    }
    let first = remaining.max(0) as usize;
    let rest = frames - first;
    let length = region.end.0 - region.start.0;
    // A second wrap inside one block would need a third segment; refuse rather than walk on
    if length <= 0 || rest as i64 > length {
        return None;
    }
    out[0] = (transport.position, 0, first);
    out[1] = (region.start, first, rest);
    Some(2)
}
