<!--
Author: Jeff
Date: 2026-09-06
Description: Proposed contract for routing note events through the graph, so MIDI effects exist
Notes: PROPOSED, not accepted. Amends dsp-device-io.md and graph-compilation.md, both accepted,
  so it needs ratification before any of it is implemented
-->

# Note Routing and Note-Effect Devices

- **Status:** proposed
- **Last verified:** 2026-09-06
- **Scope:** how note events travel between devices, and what a note-transforming device is
- **Decision authority:** Jeff
- **Upstream sources:** accepted `dsp-device-io.md`, `graph-compilation.md`, the 2026-09-06 interview
- **Downstream dependents:** MIDI effect devices, the piano roll, R11's modular identity layer
- **Supersedes:** nothing
- **Superseded by:** none
- **Open decisions:** the four in §6, all of which need answers before implementation
- **Known gaps:** no implementation exists; every claim here is design, not evidence

## 1. Why this exists

Jeff's 2026-09-06 interview made MIDI effects — arpeggiator, chord, scale, velocity — an MVP
requirement, sequenced after the routing milestone and before the piano roll.

They are not unimplemented. They are **unrepresentable**, in three separate ways:

1. `AudioProcessor::process` takes note events by shared borrow through `ProcessContext` and
   writes only `&mut [&mut [f32]]`. A device cannot emit a note event at all.
2. `DeviceClass` is `Source | Instrument | Effect`. There is no note-transforming role.
3. `EditableGraph::add_node` refuses any layout whose `audio_outputs != 2`
   (`crates/spectre-graph/src/lib.rs:145`). A MIDI effect produces no audio, so it cannot be a
   node.

`dsp-device-io.md` names the gap already: *"analyzer/sink and event output buses are deferred
until required by a vertical slice."* This is that slice requiring them.

## 2. What note events do today

They never enter the graph. `RenderBridge` addresses a node directly through
`PlanNoteInput { node, events }`, and `CompiledPlan::process` hands that slice to exactly that
node's `ProcessContext`. One hop, no routing, no ordering question beyond the one the bridge
already solves by merging and sorting.

**That path must survive unchanged.** R4-1's transport evidence, R4-5's merge rule, R4-9's
multi-voice bridge, and every hardware qualification row run through it.

## 3. Proposed contract

### 3.1 Note buses replace the note boolean

`DeviceIo.accepts_notes: bool` becomes:

```rust
pub note_inputs: usize,   // note buses consumed
pub note_outputs: usize,  // note buses produced
```

`accepts_notes` is then `note_inputs > 0` and stays available as a method, so no existing
call site changes meaning. Every shipping device declares `note_inputs: 0 or 1, note_outputs: 0`,
which is exactly what they do now.

### 3.2 One new device class

`DeviceClass::NoteEffect` — one note input bus, one note output bus, no audio at all. It is the
first layout in the contract with `audio_outputs == 0`, which is the graph rule §3.4 relaxes.

### 3.3 A second trait, not a wider one

**Recommended:** note-transforming devices implement a separate trait.

```rust
pub trait NoteProcessor: Send {
    fn io(&self) -> DeviceIo;
    fn set_parameter(&mut self, key: DeviceParameterKey, value: f32) -> Result<(), ParameterError>;
    fn process_notes(
        &mut self,
        context: &ProcessContext<'_>,
        output: &mut NoteOutput<'_>,
    ) -> Result<(), ProcessError>;
}
```

The alternative — adding a note-output parameter to `AudioProcessor::process` — changes every
shipping device's signature so that all but one of them can ignore it. That is the wrong trade:
the accepted contract makes `set_parameter` required precisely so a device *cannot* silently
ignore a seam, and a parameter every device ignores is the opposite of that.

`NoteOutput` is a bounded writer over a plan-owned buffer. It refuses past
`MAX_NOTE_EVENTS_PER_BLOCK` rather than growing, and it allocates nothing.

**Consequence worth stating: `ProcessContext` and the device-facing audio API do not change at
all.** An instrument downstream of an arpeggiator receives its events through
`context.events()` exactly as it does today; only where the plan *sourced* that slice differs.

### 3.4 Graph rules

- `add_node` accepts a layout when `audio_outputs == 2`, **or** when `audio_outputs == 0 and
  note_outputs > 0`. Nothing else is relaxed.
- Note edges are their own connection type. Audio and note buses are separate namespaces, so a
  note output cannot be wired to an audio input by construction rather than by validation.
- One connection per note input bus, matching the accepted audio rule. Merging two note streams
  is an explicit later decision, exactly as input-bus summing is for audio.
- Reachability walks both edge types: a MIDI effect feeding an instrument is an ancestor of the
  output node and must be included and ordered before it. The existing Kahn sort already covers
  this once note edges are in the edge set.

### 3.5 Buffers and realtime safety

The plan preallocates one note buffer per note output bus at compile time, each of
`MAX_NOTE_EVENTS_PER_BLOCK` capacity. Nothing is allocated or resized in the callback, matching
how audio channel buffers already work.

**Stated cost:** `NoteEvent` is 32 bytes, so one note bus is 32 KiB. A project with a MIDI effect
on every one of `MAX_TRACKS` tracks reserves 1 MiB. Bounded, app-thread allocated, and refused
rather than grown.

### 3.6 The existing bridge path

`PlanNoteInput { node, events }` keeps its exact meaning: it delivers to a node's note input bus
0. If that node is an instrument, behavior is identical to today. If it is a note effect, the
effect transforms and its output feeds downstream.

A node whose note input bus is fed by *both* a graph edge and a `PlanNoteInput` is refused, for
the same reason a doubly-fed audio bus is.

## 4. What this does not do

- No merging of note streams; one source per note input bus.
- No note feedback; note edges join the existing implicit-cycle rejection.
- No new event vocabulary. Pitch bend, aftertouch, and CC remain absent — `NoteEventKind` is
  still on/off/all-notes-off, and widening it is a separate decision.
- **This is not the typed signal model.** The vision names an original pitch/gate/phase/audio
  contract as distinctly Spectre, gated at R11 behind decision 7. This is the note half only,
  scoped to make MIDI effects work. R11 should extend it, and §6's answers constrain how.

## 5. Migration and evidence

No persisted schema changes: note routing is graph shape, and the project model's device chain
already carries ordered devices. A track chain holding a note effect before its instrument is
representable in schema 3 today.

The evidence bar is the engine one, in full: a MIDI effect in a chain changes what the instrument
plays, proven live and offline at equal block size through the existing FNV-1a fold; allocation
guards extended to the note path; the ordering contract falsified by breaking it.

## 6. Open decisions — all of these need answering before implementation

1. **Sequence numbers for generated events.** CLIP-008 assigns bands: clips at `1 << 63` and
   above, live ingress from 0, so a live event resolves before a clip event at an equal offset
   and rank. An arpeggiator *creates* events that came from neither. It needs its own band and a
   stated precedence against both, or the ordering contract has a hole.
2. **Note IDs for generated events.** `CLIP_NOTE_ID_BASE` is `0x8000_0000`. Generated notes need
   their own range, and a rule for what happens when a note effect is asked to release an ID it
   did not mint.
3. **Whether a note effect may consume its own output across blocks.** An arpeggiator holding
   state across a block boundary is ordinary; one whose output re-enters its input is feedback,
   which decision 7 gates. The first is required, the second must be refused, and the boundary
   between them should be written down before either is built.
4. **Where a note effect sits in the track chain.** The chain is currently instrument → effects →
   gain. A note effect must sit *before* the instrument, so a track's chain is no longer uniform
   and `build_track_graph`'s fold needs to split on device class. That is a small change, but it
   makes the chain's ordering rules a product-visible thing rather than an internal one.
