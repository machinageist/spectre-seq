<!--
Author: Jeff
Date: 2026-08-16
Description: R4-5 spec — MIDI clips that play through a track by feeding the existing bounded note-event contract, not a second event path
Notes: The whole feature turns on one thing — clip playback and live MIDI ingress must arrive at
  `ProcessContext::new` as ONE array that is strictly increasing in the accepted key
  `(frame_offset, NoteEventKind::rank, sequence)`. Two independent producers with independent
  sequence counters and independent note-ID allocators collide on that key and silence the block.
  `spectre-core` already contains a SECOND, unrelated ordering key (`EventKind::order_rank`,
  `crates/spectre-core/src/event.rs:22-30`) that no production path uses; this spec forbids
  reaching for it. `rank` is reused, never restated.

  ITERATION 2 (remediation 1, 2026-08-22) — what changed, and nothing else did. Iteration 1 passed
  review at composite 2.798 with one must-fix defect: `SCHEDULE_LANE_CAPACITY = 2` does not build
  the lane it described. Re-verified against source: `spsc::bounded` computes
  `requested = capacity.max(MIN_CAPACITY) + 1`, rounds up with `next_power_of_two`, and returns
  `capacity() == mask == slots_len - 1` (`crates/spectre-audio/src/spsc.rs:57-68`, `:96-98`,
  `MIN_CAPACITY = 2` at `:16`), so `bounded(2)` allocates 4 slots and yields usable capacity 3.
  Further finding not in the scorecard: because `MIN_CAPACITY` clamps upward, EVERY argument from
  0 through 3 yields the same ring, so a usable depth of 2 is unreachable and the scorecard's
  suggested `bounded(SCHEDULE_LANE_CAPACITY - 1)` would not have fixed it. The constant is
  therefore 3 — the only value that equals what `bounded` delivers — with CLIP-006 rewritten to
  derive it from the primitive rather than from a preference, and I-10 rewritten to push until
  `Err` against the lane's own capacity in the shape `control_channel.rs:160-175` and
  `spsc_queue.rs:34-48` already use, asserting `Producer::capacity() == SCHEDULE_LANE_CAPACITY` so
  the two cannot drift again. Every other site that asserts or reasons about the lane's depth moved
  with it: §4.2's constant block, §4.3's `control.rs` lane comment, §4.7's memory ceiling (now
  5 copies per bus, 10 MiB bound, 2 MiB expected), §7.2's CLIP-001 and CLIP-006 rows, and §8 Q2.
  Also corrected: §7.1's `order_rank` transcription (`NoteOn` 2, not a second `note-off`,
  `crates/spectre-core/src/event.rs:24-28`); three `MidiIngress` field pointers and the two §4.3
  sequence-counter anchors (`midi.rs:90`/`:91`/`:92`, not `:91`/`:92`/`:93`); both recitations of
  the cloud non-goal (`docs/00-product/vision.md:57`, not `current-milestone.md:77`, which is R4's
  non-goals list and does not mention cloud); `OBS-BW53-AUTO-005` (`bitwig-studio-observations.md:40`,
  not `:41`, which is AUTO-006); CLIP-004's over-broad sufficiency claim (512 bounds the densest
  MONOPHONIC block — `insert_note` permits stacked notes at a shared start tick, and polyphonic
  density is bounded by E-8's counted refusal, not by the number); and CLIP-005's rationale, which
  was pinned to a 256-frame block and is now stated against `plan.max_frames()` with the
  buffer-size limitation named. Every line number touched or added was re-opened against the source
  on 2026-08-22. The header `Last verified` date is deliberately NOT advanced: this pass re-verified
  the citations it touched, not all ~60 in §7.1. Nothing the reviewer verified as correct was
  weakened — R4-5-MERGE, the equal-timestamp determinism, the 512 + 512 = 1,024 merged ceiling, and
  I-7's `blocks_rendered` pin are untouched.
-->

# Spec: MIDI Clips

**Feature ID:** `midi-clips`
**Parent feature:** `R4` Credible Alpha (`docs/06-plans/current-milestone.md`)
**Spec author agent:** spec author agent, R4-5
**Date:** 2026-08-16
**Iteration:** 2 (remediation 1)

- **Status:** proposed
- **Last verified:** 2026-08-16
- **Scope:** MIDI clips stored as project data, placed on a track, and played through the existing
  compiled plan by producing `spectre_dsp::NoteEvent` values on the one accepted note path.
  Clip *content editing* is limited to a list surface; a piano-roll canvas, the session/launcher
  grid, automation, and recording are all named deferred below.
- **Decision authority:** Jeff
- **Upstream sources:** `docs/README.md` (§Conflict precedence, §Status vocabulary);
  `docs/01-requirements/requirements-ledger.md` (RT-001 `:28`, RT-002 `:29`, RT-003 `:30`,
  TIME-001 `:36`, TIME-002 `:37`, TIME-003 `:38`, TIME-004 `:39`, TIME-005 `:40`, CORE-001 `:46`,
  CORE-002 `:47`, CORE-003 `:48`, GRAPH-001 `:55`, PROD-002 `:63`, PROD-003 `:64`);
  `docs/01-requirements/decision-gates.md` (rows 1 `:25`, 5 `:29`, 6 `:30`, 15 `:39`, 16 `:40`,
  17 `:41`, 21 `:45`, 22 `:47`, 23 `:49`); `docs/03-architecture/dsp-device-io.md` §Events
  `:43-52` and §Realtime contract `:75-79`; `docs/06-plans/current-milestone.md:83`;
  `docs/status/NEXT.md:27`; `docs/02-reference-research/` observation corpus.
- **Downstream dependents:** R4-6 (small synth — replaces `PulseInstrument` under the same note
  contract), R4-7 (persistence — clips are the first persisted child collection and therefore
  CORE-001's reorder evidence), R4-8 (offline bounce — must render the same schedule), R4-9
  (end-to-end fixture and manual QA), R7 (recording), R9 (automation), R10 (session grid).
- **Supersedes:** nothing. There is no prior clip spec in `gauntlet-output/specs/`.
- **Superseded by:** none.
- **Open decisions:** eleven, all in §8. Q1 (note-ID band exhaustion in the accepted ingress),
  Q2 (schedule-handoff lane vs. decision 21's three-lane split), and Q5 (note chase on seek) are
  the three that need Jeff before implementation starts.
- **Known gaps:** (a) no benchmark in Jeff's AAA set has a citable record of *how* a clip's notes
  are converted into a scheduler's per-block event stream — the corpus documents clip behavior,
  not clip engines, and §Appendix A records that as a research need rather than filling it in;
  (b) Logic Pro contributes nothing at all (`docs/02-reference-research/logic-pro.md:10-11`,
  `draft` / `inventory-only`) and Serum 2's two records
  (`synth-modular-observations.md:54`, `:55`) are about CPU guidance and a support-site
  inventory, neither of which touches clips; (c) this spec's numeric bounds are derived from
  Spectre's own arithmetic and are `proposed` until Jeff accepts the ledger rows in §7.2.

---

## 1. Purpose

### 1.1 One-sentence job

As a musician, I want a repeatable phrase I wrote once to play back from a track at the right
musical positions every time I press Play, so that I can hear an idea in context and keep
building on it instead of re-performing it.

### 1.2 Why it matters

R4 is named "Credible Alpha" and its exit list includes exactly one clip row:
*"A MIDI clip plays through a track into master"* (`docs/06-plans/current-milestone.md:83`).
Until that row closes, Spectre has an audio engine that can be driven only by a live performance
that nobody is playing: `spectre-audio` has a callback bridge and MIDI ingress, but
`./spectre` constructs none of it, so Play changes a model field and produces no sound
(`docs/status/STATUS.md:37`). A clip is the smallest object that lets a person write something
down and hear it again — it is what turns a signal chain into a sequencer.

The specific pain this addresses is **loss of the idea between hearing it and keeping it**. The
vision's loop is sketch → branch → audition → grow. Without a clip, every step after "sketch"
requires replaying by hand, and the branch/audition steps are impossible because there is nothing
to branch.

There is a second, narrower reason this feature exists as its own slice rather than as part of
the track model. `docs/status/NEXT.md:27` phrases slice 5 as a constraint before it phrases it as
a feature: *"reusing the existing bounded event-ordering contract and MIDI ingress rather than a
second event path."* The most likely way to ship a working-looking clip player is to give it its
own event type, its own sort, and its own sequence numbering — which is exactly how the ordering
contract silently forks. This spec is written primarily to prevent that.

### 1.3 Success signal

**Primary, automatable:** with a clip containing a known note list placed on a track, two
independent renders of the same transport span produce the *same FNV-1a hash* using the existing
walk (`crates/spectre-offline/src/lib.rs:271-283`; interleaved variant at
`crates/spectre-audio/tests/bridge_plan.rs:80`), the peak is nonzero, and the live bridge render
hashes identically to the offline render of the same schedule. Determinism against a silent
buffer is not a signal, so the nonzero-peak condition is part of the assertion, following the
precedent the bridge equivalence test already set (`docs/06-plans/current-milestone.md:49`).

**Secondary, and the one that proves "not a second event path":** with clip playback active,
`ProcessContext::new` (`crates/spectre-dsp/src/io.rs:77-107`) never returns
`ProcessError::UnsortedEvents` or `ProcessError::EventCapacity`, and
`BridgeTelemetry::plan_errors` (`crates/spectre-audio/src/bridge.rs:67-69`) stays at zero across
the run. The plan's own validator is the referee; a test that only inspects the scheduler's
output array can agree with itself while violating the contract, which is the exact reasoning
`docs/status/NEXT.md:42` records for the ingress slice.

---

## 2. User Stories

> As a musician sketching a part, I want to place a clip on a track and press Play, so that the
> phrase I entered sounds at its written positions instead of only when I hold a key.

> As a musician auditioning a loop, I want the transport loop to repeat my clip cleanly, so that
> I can listen around an idea without a note hanging over the wrap or a click at the seam.

> As a musician who stops mid-phrase, I want every sounding clip note to be released the instant I
> press Stop or move the playhead, so that a note never sustains into silence with no way to
> cancel it.

> As a musician on a busy machine, I want a block that the engine cannot render to fall back to
> silence rather than to stale or corrupted audio, and I want that refusal counted where I can see
> it, so that I can tell "the machine is struggling" apart from "my clip is wrong."

> **(Edge case / error state.)** As a musician who pastes in an enormous amount of material, I
> want the application to refuse the clip with a specific reason and leave my project untouched,
> so that I never discover the limit as a crash or as a silent truncation of my notes.

> **(Accessibility.)** As a keyboard-only or screen-reader user, I want to reach, inspect, and
> edit a clip's notes without a pointer and without a canvas, so that clip work is not gated on
> pixel-accurate dragging (decision 17, `docs/01-requirements/decision-gates.md:41`).

> **(Secondary persona — the offline/bounce path.)** As the offline renderer, I want the same
> baked schedule the live path uses, so that a bounce is the same computation rather than a
> parallel implementation that happens to agree today (R4-8).

---

## 3. UX Specification

R4's shell is egui (decision 8). The existing app has four lenses — `Arrange`, `Build`, `Shape`,
`Mix` (`crates/spectre-app/src/lib.rs:13-22`) — and `AppModel` currently holds
`tracks: Vec<TrackView>` plus `selected_track: Option<ObjectId>`
(`crates/spectre-app/src/lib.rs:205-214`, `:289-311`). This feature's surfaces live in `Arrange`.

### 3.1 Screen / view inventory

| Surface | Navigation path | New / modified | Layout pattern |
|---|---|---|---|
| **Clip lane** (per track) | `Arrange` lens, inside the existing track row | New | Horizontal strip inside the track row; one rectangle per placement |
| **Clip inspector** | `Arrange` → select a clip placement → inspector panel | New | Trailing side panel over the existing lens body |
| **Note list** | Inside the clip inspector | New | Vertical, scrollable, virtualized table: position, length, pitch, velocity, channel |
| **Track row** | `Arrange` lens | Modified | Gains the clip lane; existing name / mute / solo controls unchanged |
| **Transport readout** | Existing header | Modified | Gains a bars-beats readout derived from `MeterMap::signature_at` (`crates/spectre-core/src/meter.rs:173`) and `TempoMap` |

**Deliberately not introduced in R4:** a piano-roll canvas, a session/launcher grid
(R10 — Ableton's slot model, `OBS-AB12-SES-001` at `ableton-live-observations.md:40`), an
automation lane (R9 — `OBS-AB12-AUTO-002` at `:125`), a record-arm/step-record surface
(R7 — `OBS-AB12-REC-004` and `OBS-AB12-REC-005` at `:113-114`), and per-clip time tools
(`OBS-AB12-CLIP-011` at `:59`). Each is named here so it is deferred rather than smuggled.

The note **list** rather than a piano roll is a deliberate R4 choice, not an aesthetic one:
decision 17 (`docs/01-requirements/decision-gates.md:41`) makes keyboard-complete operation and
screen-reader labels a beta gate with a scoped audit at R4, and a list is keyboard-complete on
day one while a canvas has to be retrofitted with a parallel accessible representation. The piano
roll is additive later over the same model. This forecloses nothing.

### 3.2 Interaction flows

**Primary flow — write a phrase and hear it.**

1. User selects a track in `Arrange`. Selection is already `ObjectId`-keyed
   (`crates/spectre-app/src/lib.rs:297-301`); this flow does not change selection semantics.
2. User invokes *New clip* from the context-scoped command surface (see §3.4 — no default key is
   specified). The app allocates a clip `ObjectId` from the project `IdGen`
   (`crates/spectre-core/src/id.rs:45-58`) and creates an empty clip placed at the current
   playhead, snapped to the bar per `MeterMap::signature_at`.
3. User adds notes in the note list. Each row edit is one `Transaction`
   (`crates/spectre-project/src/command.rs:72-112`) applied through `EditHistory`
   (`:114-153`), so every clip edit is undoable by the mechanism already in the project crate.
4. User presses Play. The app publishes a **baked clip schedule** to the render thread (§4.3) and
   sends `TransportCommand::Play` (`crates/spectre-core/src/transport.rs:53-59`) over the
   existing transport lane.
5. Sound. The bridge advances the transport, the clip player emits `NoteEvent`s for the block,
   they merge with any live-MIDI events into one sorted array, and the plan renders them.

**Branch A — user edits a note while the transport is running.** The edit mutates project data on
the app thread, the app re-bakes the schedule, and publishes it. The render thread swaps to the
new schedule at the next block boundary and returns the old one for app-thread reclamation. Notes
already sounding are released at the swap (see §3.6 E-4) rather than left to be matched against a
schedule that no longer contains their release. Editing clip *content* never recompiles the plan
(GRAPH-001, `requirements-ledger.md:55`); only track structure does, which is R4-4's concern.

**Branch B — transport loop wraps inside a block.** The block window is split at the loop
boundary. All clip notes sounding at the wrap are released *at the wrap frame*, then material
from the loop start is emitted from the same frame onward. Because
`NoteEventKind::rank` scores `Off` and `AllNotesOff` as `0` and `On` as `1`
(`crates/spectre-dsp/src/io.rs:152-157`), the releases sort before the attacks at that identical
frame offset without any special-casing in the scheduler.

**Branch C — user seeks or stops.** Any transport position discontinuity (a `Seek`, a `Stop`, or a
loop-region change that relocates the playhead) causes `NoteEventKind::AllNotesOff { channel:
None }` at frame offset 0 of the next rendered block and resets the schedule cursor.
`PulseInstrument` handles that variant by clearing its active note
(`crates/spectre-dsp/src/source.rs:194-197`).

**Branch D — the block cannot be rendered.** Two distinct refusals, and they behave differently;
§3.6 E-6 and E-7 carry the detail. The important user-visible property is the same in both: exact
silence, counted, never stale audio.

**Cues.** No haptics (desktop). No sound cue is added — the only sound this feature makes is the
music. One animation: the clip lane's playhead marker, covered in §3.5.

### 3.3 Layout descriptions

**Clip lane** (inside the existing track row, leading → trailing):

- Leading: the existing track name and mixer controls, unchanged.
- Trailing: a horizontal lane spanning the arrangement's visible tick range. Each clip placement
  draws as a rectangle whose leading edge is its start tick and whose width is its length in
  ticks, converted to pixels by the lens zoom.
- **Data source:** `TrackClips` (§4.2) for placements, the project's `TempoMap` and `MeterMap`
  for the tick→bar ruler, and `AppModel::selected_clip` for the highlight.
- **Empty state:** the lane draws its grid and one centered line of copy —
  `"No clips on this track."` — with the *New clip* command named in the same line as text, not
  as a key glyph (§3.4). The empty state must state the fact, not advertise; the release bar in
  `docs/00-product/vision.md` prohibits fake surfaces, and an empty lane that looks populated is
  one.

**Clip inspector** (top → bottom):

1. Header: clip name (editable text), clip `ObjectId` shown as a stable identity field.
2. Placement fields: start (bars.beats.ticks), length (bars.beats.ticks), loop on/off, active
   on/off.
3. Note list header row: Position, Length, Pitch, Velocity, Channel.
4. Note list body: one row per note, sorted by position then pitch. Virtualized — the list must
   render `MAX_NOTES_PER_CLIP` rows without materializing them all, because §4.2 permits 4,096.
5. Footer: note count against the bound, e.g. `312 / 4096 notes`. This is a truthful capacity
   readout, not a warning banner.

**Data sources:** every field reads `MidiClip` / `ClipPlacement` (§4.2) directly. There is no
per-lens copy of clip state and no view model that owns a second version of a note — criterion 2B
is satisfied structurally rather than by convention.

**Empty state (inspector):** `"No clip selected. Select a clip in Arrange to edit its notes."` —
mirroring the wording pattern `SHAPE_EMPTY_MESSAGE` already uses
(`crates/spectre-app/src/lib.rs:92`).

### 3.4 Input & gestures

- **Pointer:** click selects a clip placement; drag moves it along the lane; drag on an edge
  changes its length. Movement and resize snap to the grid derived from `MeterMap`; a held
  modifier bypasses snapping. Ableton converges here (`OBS-AB12-ARR-004`,
  `ableton-live-observations.md:28`: clips snap to the editing grid, other clip edges, locators,
  and time-signature changes, with grid snapping bypassed by a held modifier).
- **Keyboard:** every action reachable by pointer is reachable by keyboard. Focus moves between
  the clip lane and the note list; arrow keys move the selection; typed values edit the focused
  cell; the command surface is searchable by name.
- **Keyboard shortcuts:** **none are specified by this spec, and none may be.** AF-5 prohibits a
  default shortcut map, sourced to
  `docs/02-reference-research/workflow-field-study/product-implications.md`
  §"Prohibited conclusions at current evidence level". What this spec *does* fix is structural
  and is permitted by criterion 2D: commands are **context-scoped** (a clip command resolves only
  while the clip lane or inspector has focus) and **remappable** (no binding is hard-coded into a
  handler). Which physical keys ship is Jeff's, informed by evidence this project does not yet
  have — §8 Q9.
- **Specialized input:** none. No stylus, controller, voice, or camera path. External MIDI
  controller input is the existing ingress path and is unchanged by this spec.
- **Responsive behavior:** the clip lane scrolls horizontally with the arrangement and never
  forces a minimum window width; the inspector collapses to a full-width sheet below a threshold
  width rather than clipping its note list.

### 3.5 Transitions & animation

- **Navigation transitions:** none. Selecting a clip reveals the inspector with no animated
  entrance, consistent with the "calm UI" standard.
- **In-view state change:** exactly one moving element — a one-pixel playhead marker in the clip
  lane, driven by `RenderBridge::transport()` (`crates/spectre-audio/src/bridge.rs:157-159`)
  read once per UI frame. It is a position readout, not an effect.
- **Reduced motion:** with reduced motion requested, the playhead marker updates on a coarser
  cadence (bar boundaries) instead of continuously. Nothing else animates, so nothing else needs
  an alternative. The marker is never the only indication of transport state — the existing
  transport control carries that.

### 3.6 Error states

| ID | Trigger | Presentation | Recovery | Data loss |
|---|---|---|---|---|
| E-1 | Clip note count would exceed `MAX_NOTES_PER_CLIP` | Inline, at the note list footer; the offending edit is refused | Delete notes, or split into a second clip | No — the edit never applies; `Transaction` is atomic (`command.rs:78-112`) |
| E-2 | Placement count would exceed `MAX_CLIPS_PER_TRACK` | Inline, on the clip lane | Delete a clip or use another track | No |
| E-3 | Clip length would exceed `MAX_CLIP_LENGTH_TICKS`, or a note falls outside `[0, length)` | Inline, on the offending field | Shorten the clip, or move the note inside it | No |
| E-4 | Schedule swap while notes are sounding | None (audible only): sounding clip notes are released at frame 0 of the swap block | Automatic | No |
| E-5 | Schedule-handoff lane full — the app thread published faster than the render thread consumed | None immediate; a counter (`schedule_overflows`) increments and the app retries on the next UI frame | Automatic | No — `Producer::push` returns the rejected value rather than consuming it (`crates/spectre-audio/src/spsc.rs:80`), so the schedule is retained, not dropped |
| E-6 | **Frame-capacity refusal.** The driver delivered a block outside `1..=plan.max_frames()` | None immediate; `frame_capacity_rejections` increments (`bridge.rs:73`) and the block is exact silence | Reopen the stream at a supported buffer size | No, **and no playback skip** — see the note below |
| E-7 | **Plan error.** `CompiledPlan::process` refused the block | None immediate; `plan_errors` increments (`bridge.rs:67-69`); the block is exact silence | Off-thread diagnostic; a nonzero count is a defect, not weather | Audible gap only, bounded to that block — see the note below |
| E-8 | Clip events for one block would exceed `CLIP_EVENT_RESERVE` | None immediate; `clip_events_refused` increments; that bus emits `AllNotesOff` for the block instead of a partial event set | Reduce note density; re-bake | Audible: that block's clip material is skipped, deliberately, rather than emitted half-formed |
| E-9 | Block spans more loop wraps than `MAX_BLOCK_SEGMENTS` permits | None immediate; `loop_segments_refused` increments; `AllNotesOff` for the block | Lengthen the loop region past one block | Audible, as E-8 |
| E-10 | A loaded project's clip data fails validation (non-finite velocity, note outside range, unsorted, ID collision) | Modal on load, naming the specific invariant | Load fails closed; the previous project stays open | No — nothing partial is admitted, matching `spectre_project::validate` (`crates/spectre-project/src/lib.rs:105-124`) |

**E-6 and E-7 differ in a way this feature must not get wrong, and the difference comes from where
`render` returns.** `RenderBridge::render` performs the frame-capacity check *first* and returns
at `crates/spectre-audio/src/bridge.rs:166-173`, **before** `apply_transport()` at `:175`, before
`collect_notes()` at `:176`, and before `blocks_rendered` increments at `:204-206`. So on E-6 the
transport has not advanced and no clip material has been consumed: the playhead stalls and the
output is silent, but nothing is skipped and no note-on is stranded without its note-off. On E-7
the failure occurs at `:192-200`, *after* the transport advanced and after clip events were placed
into the scratch, so that block's material is genuinely lost. That loss cannot hang a note,
because a stranded release reaches `PulseInstrument` as an `Off` whose id matches nothing and is
ignored (`crates/spectre-dsp/src/source.rs:198`), and a stranded attack is simply never heard.
Both are counted; neither is silent-in-the-telemetry-sense.

Every presentation choice above is inline rather than a toast or a modal, with one exception
(E-10, which is modal because it changes what project is open). The reason is uniform: these
errors are all *about a specific field or a specific counter*, and moving them away from the field
would make the user hunt. E-4 through E-9 have no immediate presentation at all because they occur
on the render thread, where surfacing anything at all would violate RT-001
(`requirements-ledger.md:28`); they are counters read off-thread, matching the pattern
`BridgeTelemetry` (`bridge.rs:24-39`) already establishes.

### 3.7 Accessibility

- **Screen reader labels.** Every clip rectangle carries a label of the form
  `"Clip {name}, bar {n} beat {m}, {k} bars long, {j} notes, {active|inactive}"`. Every note row
  carries `"{pitch name}, bar {n} beat {m}, length {…}, velocity {…}, channel {…}"`. Pitch is
  announced by name (`C3`), never only as a MIDI number, because the number is not the thing the
  user is thinking about.
- **Custom actions.** The clip rectangle exposes named actions — *Move to next bar*, *Rename*,
  *Toggle active*, *Delete* — so a screen-reader user is never required to drag. This is the
  accessible equivalent of the pointer gestures in §3.4, not a reduced substitute.
- **Text scaling.** The note list is a text table and reflows with the platform text size. The
  clip lane's labels truncate with an ellipsis while the rectangle keeps its geometry, because
  the rectangle encodes time and must not be resized by a font setting.
- **Color-independent state.** Clip **active/inactive** state is carried by a text token in the
  label and by a hatched fill, never by hue alone. Ableton's convergent behavior here is
  `OBS-AB12-CLIP-001` (`ableton-live-observations.md:49`): deactivated clips do not play when
  launched or during arrangement playback. Spectre follows the *behavior*; the presentation is
  its own.
- **Focus order.** Track row → clip lane → clips in tick order → inspector header → placement
  fields → note list → note rows in position order. Focus is never trapped in the lane, and
  selecting a clip does not steal focus into the inspector.
- **Trajectory, not delivery.** Decision 17 (`decision-gates.md:41`) makes this a beta gate with
  a scoped audit at R4. This spec does not claim the audit passes. It claims the design does not
  foreclose it, and the list-over-canvas choice in §3.1 is the concrete evidence of that.

---

## 4. Implementation Specification

### 4.1 Architecture placement

Three crates change; the dependency graph does not.

| Crate | Role | Existing dependencies |
|---|---|---|
| `spectre-project` | **Owns clip data.** New module `crates/spectre-project/src/clip.rs`. | `spectre-core`, `serde`, `serde_json` (`crates/spectre-project/Cargo.toml`) |
| `spectre-audio` | **Owns the baked schedule and the render-side player.** New module `crates/spectre-audio/src/clip.rs`; `bridge.rs` and `control.rs` modified. | `spectre-core`, `spectre-dsp`, `spectre-graph`, optional `cpal` (`crates/spectre-audio/Cargo.toml`) |
| `spectre-app` | **Owns the surfaces and the bake call.** | `eframe`, `spectre-core`, `spectre-dsp`, `spectre-project`, `serde`, `serde_json` (`crates/spectre-app/Cargo.toml`) |

The split matters. `spectre-audio` does **not** depend on `spectre-project` today, and this spec
does not add that edge: the bake function in `spectre-audio::clip` takes tick-domain notes plus a
`&TempoMap` and a `SampleRate` — all `spectre-core` types, which `spectre-audio` already imports
(`crates/spectre-audio/src/bridge.rs:12`) — and never sees a `MidiClip`. `spectre-app` already
depends on both sides and is the crate that hands one to the other. This is the same shape the
workspace already uses for `DeviceParameterSnapshot`: a renderer-neutral DTO defined in the
lower crate, populated by the app (`crates/spectre-app/src/lib.rs:348-383`).

`spectre-app` does **not** depend on `spectre-audio` today (`crates/spectre-app/Cargo.toml` lists
`eframe`, `spectre-core`, `spectre-dsp`, `spectre-project`, `serde`, `serde_json` and no audio
crate). Adding that edge is R4-1's job, not this spec's; §7.4 records the dependency.

The GRAPH-001 split (`requirements-ledger.md:55`) is preserved exactly. `ClipSchedule` is to note
events what `CompiledPlan` is to audio processing: an immutable, preallocated structure built on
the app thread and executed on the render thread. It is not a graph node, it is not an
`AudioProcessor`, and it does not participate in compilation. Editing a clip therefore never
triggers plan recompilation — which is what decision 22 (`decision-gates.md:47`) rejected option
(b) for in the parameter case, and the reasoning transfers unchanged.

### 4.2 Data model

Every file below carries Jeff's header block. Fields are private with accessors, matching
`DeviceParameterSnapshot`'s containment pattern (`crates/spectre-dsp/src/parameter.rs`).

```rust
// crates/spectre-project/src/clip.rs
//
// Author: Jeff
// Date: 2026-08-16
// Description: Project-owned MIDI clip data in the musical tick domain
// Notes: App-thread only; never callback-reachable. Positions are BeatTicks (decision 5) and are
//   converted to samples exactly once, at bake time, through TempoMap. Nothing here defines an
//   ordering key: the render-side key is spectre_dsp::NoteEventKind::rank and is reused as-is.

// Notes one clip may carry.
// Rationale row scheduled into docs/01-requirements/requirements-ledger.md (PROD-003, decision 16).
// Derived from Spectre's own cost, not from a reference product. One baked note occupies 32 bytes
// (see ScheduledNote below); 4,096 x 32 B = 128 KiB per track schedule. R4-4 proposes MAX_TRACKS
// = 16, so a full project schedule is 2 MiB resident, and 4 MiB while one retired schedule per bus
// is in flight on the reclaim lane. That 4 MiB is the reclaim-path figure, not the ceiling: the
// schedule lane can hold SCHEDULE_LANE_CAPACITY more, and §4.7 states the combined bound.
// Musically: 4,096 notes on a 1/16 grid (240 ticks) spans
// 4,096 x 240 = 983,040 ticks = 1,024 beats = 256 bars of 4/4.
pub const MAX_NOTES_PER_CLIP: usize = 4_096;

// Clip placements one track may carry.
// Rationale row scheduled into the ledger. Derived from MAX_NOTES_PER_CLIP, not chosen
// independently: the binding bound is 4,096 baked notes per track, and 64 placements is the point
// at which the average placement carries 4,096 / 64 = 64 notes — the smallest average that still
// represents musical material rather than fragments. Above 64 the note bound binds first anyway.
pub const MAX_CLIPS_PER_TRACK: usize = 64;

// Longest clip a v1 project may contain, in BeatTicks at 960 PPQ.
// Rationale row scheduled into the ledger. Derived from TIME-003's own 24-hour project floor
// (requirements-ledger.md:38) evaluated at Spectre's own MAX_BPM = 1200 (crates/spectre-core/
// src/tempo.rs:11): 24 h x 3,600 s = 86,400 s; 86,400 s x 1,200 BPM / 60 = 1,728,000 beats;
// 1,728,000 x TICKS_PER_BEAT (960, crates/spectre-core/src/time.rs:9) = 1,658,880,000 ticks.
// A clip cannot usefully outlast the longest project the accepted time contract must represent.
// Comfortably inside i64 (BeatTicks, crates/spectre-core/src/time.rs:22-24).
pub const MAX_CLIP_LENGTH_TICKS: i64 = 1_658_880_000;

// One note inside a clip, positioned in the clip's own tick domain
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClipNote { /* private */ }

impl ClipNote {
    // Reject a note the accepted event contract could not carry. Channel, note number, and
    // velocity bounds are NOT re-derived here: they are the same bounds NoteEvent::is_valid
    // enforces (crates/spectre-dsp/src/io.rs:122-146) — channel <= 15, note <= 127, velocity
    // finite in [0, 1] — checked here so an invalid note can never be persisted, and checked
    // again there because the plan does not trust its callers.
    pub fn new(
        start: BeatTicks,
        length: BeatTicks,
        channel: u8,
        note: u8,
        velocity: f32,
    ) -> Result<Self, ClipError>;
    pub fn start(self) -> BeatTicks;
    pub fn length(self) -> BeatTicks;   // strictly positive
    pub fn end(self) -> BeatTicks;      // start + length; half-open [start, end) per TIME-002
    pub fn channel(self) -> u8;
    pub fn note(self) -> u8;
    pub fn velocity(self) -> f32;
}

// One clip: identity, name, length, and its notes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MidiClip { /* private */ }

impl MidiClip {
    // Caller owns ID allocation from the project IdGen (CORE-001)
    pub fn new(id: ObjectId, name: &str, length: BeatTicks) -> Result<Self, ClipError>;
    pub fn id(&self) -> ObjectId;
    pub fn name(&self) -> &str;
    pub fn length(&self) -> BeatTicks;
    // Sorted by (start, note). Sorting here is a data-model convenience for the note list and
    // for a deterministic bake order; it is NOT the event-ordering contract and does not use a
    // rank. See the bake note in §4.3.
    pub fn notes(&self) -> &[ClipNote];
    pub fn set_name(&mut self, name: &str) -> Result<(), ClipError>;
    pub fn set_length(&mut self, length: BeatTicks) -> Result<(), ClipError>;
    // Refuses past MAX_NOTES_PER_CLIP and refuses a note outside [0, length)
    pub fn insert_note(&mut self, note: ClipNote) -> Result<usize, ClipError>;
    pub fn remove_note(&mut self, index: usize) -> Result<ClipNote, ClipError>;
    pub fn note_count(&self) -> usize;
}

// One placement of one clip on a track's timeline
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClipPlacement { /* private */ }

impl ClipPlacement {
    pub fn new(id: ObjectId, clip: ObjectId, start: BeatTicks) -> Result<Self, ClipError>;
    pub fn id(&self) -> ObjectId;
    pub fn clip(&self) -> ObjectId;
    pub fn start(&self) -> BeatTicks;
    // Half-open [start, start + clip.length()) — TIME-002 (requirements-ledger.md:37)
    pub fn is_active(&self) -> bool;
    pub fn set_active(&mut self, active: bool);
    pub fn set_start(&mut self, start: BeatTicks);
}

// One track's ordered, non-overlapping placements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackClips { /* private */ }

impl TrackClips {
    pub fn new() -> Self;
    pub fn placements(&self) -> &[ClipPlacement];  // sorted by start, non-overlapping
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    // Refuses past MAX_CLIPS_PER_TRACK and refuses a placement overlapping an existing one
    pub fn insert(&mut self, placement: ClipPlacement, length: BeatTicks) -> Result<usize, ClipError>;
    pub fn remove(&mut self, id: ObjectId) -> Result<ClipPlacement, ClipError>;
    pub fn get(&self, id: ObjectId) -> Option<&ClipPlacement>;
    pub fn get_mut(&mut self, id: ObjectId) -> Option<&mut ClipPlacement>;
    // Advances on any edit that changes what the baked schedule would contain
    #[serde(skip)]
    pub fn bake_revision(&self) -> u64;
}

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
```

**Non-overlapping placements are an invariant, not a policy preference.** R4's only instrument is
`PulseInstrument`, which is monophonic — it holds `active_note: Option<(u32, u8)>`
(`crates/spectre-dsp/src/source.rs:117`) and a second attack replaces the first
(`:181-187`). Overlapping placements would produce a defined-but-musically-wrong result, and
supporting them properly needs polyphony, which is R4-6's or later's decision. Refusing the
overlap keeps the model honest about what the engine can do. §8 Q4 routes the question of whether
overlap should be permitted once a polyphonic instrument exists.

```rust
// crates/spectre-audio/src/clip.rs
//
// Author: Jeff
// Date: 2026-08-16
// Description: Baked sample-domain clip schedule and the render-thread player that emits its notes
// Notes: This is the note-side analogue of CompiledPlan — immutable, preallocated on the app
//   thread, executed on the render thread, reclaimed off-thread. It emits spectre_dsp::NoteEvent
//   directly. It defines NO ordering key: sorting uses NoteEventKind::rank
//   (crates/spectre-dsp/src/io.rs:148-158), which was made public for exactly this reason.

// Clip events one note bus may contribute to one render quantum.
// Rationale row scheduled into docs/01-requirements/requirements-ledger.md (PROD-003, decision 16).
// Derived from the accepted per-quantum ceiling, split so neither producer can starve the other:
// MAX_NOTE_EVENTS_PER_BLOCK is 1,024 (crates/spectre-dsp/src/io.rs:52), and exceeding it returns
// ProcessError::EventCapacity (io.rs:88-90), which silences the whole block. Half of that ceiling
// is reserved for clip playback and half stays available to live MIDI ingress, whose own bound is
// MAX_BLOCK_EVENTS = 1,024 (crates/spectre-audio/src/midi.rs:14).
// Cross-check against Spectre's own worst legal case: at MAX_BPM = 1200 and 48 kHz,
// samples_per_tick = 48,000 x 60 / (1,200 x 960) = 2.5 (crates/spectre-core/src/tempo.rs:72-74),
// so a 256-frame block spans 256 / 2.5 = 102.4 ticks. With notes at the finest representable
// spacing of one tick AND one note per tick, that is at most 102 attacks and 102 releases = 204
// events. 512 clears the densest MONOPHONIC block Spectre's own tempo and tick bounds admit,
// with 2.5x margin.
// What that check does not bound is polyphony, and the distinction is stated rather than implied.
// MidiClip::insert_note refuses only an over-capacity note and a note outside [0, length); notes
// stacked at a shared start tick are representable, so a 10-note chord on every tick of that same
// block would emit roughly 1,020 attacks before a single release, and exceed the reserve.
// Density is bounded by refusal, not by this number: E-8 increments clip_events_refused and emits
// AllNotesOff for that bus's block rather than a partial event set. 512 is a capacity guarantee
// for monophonic-density material, not a proof that overflow cannot occur.
pub const CLIP_EVENT_RESERVE: usize = 512;

// Contiguous transport segments the player will walk within one block.
// Rationale row scheduled into the ledger. RT-001 (requirements-ledger.md:28) requires bounded
// work on the callback, and Transport::advance already tolerates many wraps per block — its own
// test drives a 1-sample loop and reports u32::MAX wraps (crates/spectre-core/src/transport.rs:
// 163-172). Two segments means at most one loop wrap per block, which is what ordinary looping
// needs. The material this refuses is precisely a loop region shorter than ONE GRANTED BLOCK —
// and the granted block is a driver property negotiated by R4-1 (decision 20's ALSA baseline,
// decision-gates.md:44), not a Spectre constant, so the bound is stated against plan.max_frames()
// rather than against a fixed 256. Generally, at 48 kHz and 960 PPQ a block spans
// frames x BPM / 3,000 ticks:
//     256 frames -> 102.4 ticks at MAX_BPM 1200;          10.2 ticks at 120 BPM
//   1,024 frames -> 409.6 ticks (~0.43 beat) at MAX_BPM;  41.0 ticks at 120 BPM
//   2,048 frames -> 819.2 ticks (~0.85 beat) at MAX_BPM;  81.9 ticks at 120 BPM
// The limitation, stated explicitly: at 256 frames the refusal cannot reach a loop longer than a
// 32nd note (120 ticks at 960 PPQ) at any tempo, but that immunity is a property of the granted
// block, not of the bound. At ordinary tempos even 2,048 frames stays under a 32nd note; it is
// the joint extreme of a large granted block AND near-maximum tempo that would refuse a
// musically ordinary short loop. Consequence for the implementer: if R4-1 negotiates blocks above
// 1,024 frames, MAX_BLOCK_SEGMENTS must be derived from plan.max_frames() rather than left at 2.
// Refusal is counted (E-9, loop_segments_refused) and fails closed to AllNotesOff, never to
// unbounded work, so the condition surfaces as a nonzero counter rather than as a mystery.
pub const MAX_BLOCK_SEGMENTS: usize = 2;

// Baked schedules that may be queued for the render thread at once.
// Rationale row scheduled into the ledger. The value is 3 because 3 is the depth spsc::bounded
// actually delivers, and this constant must name the lane that exists rather than the lane the
// steady state would prefer. bounded computes requested = capacity.max(MIN_CAPACITY) + 1, rounds
// up with next_power_of_two, and reports capacity() = slots_len - 1 because one slot stays empty
// so full is distinguishable from empty (crates/spectre-audio/src/spsc.rs:57-68, :96-98;
// MIN_CAPACITY = 2 at :16). Every argument from 0 through 3 therefore produces the SAME ring —
// 4 slots, usable capacity 3. A usable depth of 2 is not reachable through this primitive at any
// argument, so bounded(SCHEDULE_LANE_CAPACITY) with the value 3 is the only request that equals
// what it receives.
// What the depth buys: the steady state needs two (one schedule installed and in flight, one
// being published). The third slot is the primitive's floor, not a design allowance, and it is
// named as such. The lane refuses on the FOURTH queued schedule — four publishes with no
// intervening consume. At 48 kHz / 256 frames the render thread drains at most one per block, or
// 48,000 / 256 = 187.5 per second, so reaching four means the render thread is stopped or the app
// thread has a defect; it does not mean a fast editor. Overflow is counted and the rejected
// schedule is returned, not dropped (crates/spectre-audio/src/spsc.rs:80).
// Invariant, asserted by I-10: producer.capacity() == SCHEDULE_LANE_CAPACITY. Iteration 1 set this
// to 2 and reasoned about a lane bounded(2) does not build; the assertion exists so the constant
// and the ring cannot silently disagree again.
pub const SCHEDULE_LANE_CAPACITY: usize = 3;

// First note ID reserved for clip playback.
// Rationale row scheduled into the ledger. NoteEvent identity must be unique per sounding note on
// a bus, because PulseInstrument matches a release to its attack by id
// (crates/spectre-dsp/src/source.rs:188-189) and MidiIngress keys its active table by
// (channel, note) with an ascending nonzero allocator starting at 1
// (crates/spectre-audio/src/midi.rs:89-92, :199-203). Two allocators that both start at 1 would
// let a clip release cancel a live note of the same id. Reserving the upper half of u32 for
// clip-baked IDs makes the two spaces disjoint by construction rather than by coordination.
// Collision horizon, stated rather than assumed: MidiIngress reaches 0x8000_0000 after
// 2,147,483,648 note-ons, which at a sustained 1,000 note-ons per second is 2,147,484 s ~= 24.9
// days of continuous input. That is reachable by a machine left running, not by a player. §8 Q1
// asks Jeff whether the accepted ingress should refuse at the band boundary; this spec does not
// change ingress.
pub const CLIP_NOTE_ID_BASE: u32 = 0x8000_0000;

// One clip note resolved to absolute engine sample positions. 32 bytes with padding:
// two i64 (16) + u32 (4) + f32 (4) + two u8 (2) + 6 padding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScheduledNote {
    pub start: SampleTime,
    pub end: SampleTime,       // half-open; end > start is an invariant checked at bake
    pub id: u32,               // >= CLIP_NOTE_ID_BASE, nonzero by construction
    pub velocity: f32,         // finite, [0, 1]
    pub channel: u8,           // <= 15
    pub note: u8,              // <= 127
}

// Immutable baked schedule for one note bus. Send, so it can cross to the audio thread.
#[derive(Debug)]
pub struct ClipSchedule { /* private: notes: Box<[ScheduledNote]> sorted by (start, id) */ }

impl ClipSchedule {
    // Bake tick-domain notes into absolute sample positions on the app thread.
    // TempoMap::ticks_to_samples (crates/spectre-core/src/tempo.rs:87-96) is the single
    // conversion definition per TIME-001/TIME-003; this function does not do beat arithmetic
    // of its own. Refuses: an empty-length note, a note whose bounds NoteEvent::is_valid would
    // reject, more notes than CLIP_EVENT_RESERVE could ever be needed for is NOT checked here
    // (that is a per-block condition), and more than `capacity` notes total.
    pub fn bake(
        notes: impl Iterator<Item = (BeatTicks, BeatTicks, u8, u8, f32)>,
        tempo: &TempoMap,
        rate: SampleRate,
        capacity: usize,
    ) -> Result<Self, ScheduleError>;

    // Empty schedule; renders exact silence and is the state before any clip exists
    pub fn empty() -> Self;
    pub fn notes(&self) -> &[ScheduledNote];
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}

// Render-thread cursor and sounding-note table for one note bus.
// Owns no heap growth after construction; every collection is preallocated.
pub struct ClipPlayer { /* private */ }

impl ClipPlayer {
    // Preallocate for the worst legal block. `sounding` is a fixed table, not a growing set.
    pub fn new(reserve: usize) -> Self;

    // Install a new schedule, returning the retired one for off-thread reclamation.
    // Callback-safe: this is a pointer swap. The caller MUST hand the returned schedule to
    // ControlReceiver::retire (crates/spectre-audio/src/control.rs:310-324) rather than drop it,
    // because dropping it here would deallocate on the audio thread and violate RT-001.
    #[must_use]
    pub fn install(&mut self, schedule: Box<ClipSchedule>) -> Box<ClipSchedule>;

    // Emit this block's clip events into `out`, in contract order, and return the outcome.
    // Callback-safe: no allocation, no locks, no I/O, no logging, no panics.
    // `out` is the bridge's note scratch, already cleared. `first_sequence` is the block-local
    // sequence base the bridge assigns (see §4.3).
    pub fn emit_block(
        &mut self,
        transport: &Transport,
        frames: usize,
        out: &mut Vec<NoteEvent>,
        first_sequence: u64,
    ) -> ClipBlockOutcome;

    // Release every sounding note at frame 0 of the next block: seek, stop, or schedule swap
    pub fn release_all(&mut self);
}

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
```

**Migrations.** The `ProjectDoc` envelope (`crates/spectre-project/src/lib.rs:29-38`) gains a
`clips` field. `SCHEMA_VERSION` is `1` and `MAX_READABLE_SCHEMA` is `1` (`:13`, `:16`); whether
adding a field to the R4 document bumps the version is **R4-7's decision**, not this spec's, and
§8 Q7 routes it. Until R4-7 lands, clip data can ride in the existing `unknown` flatten map
(`:36-37`), which already round-trips unknown fields (`crates/spectre-project/src/lib.rs:157-173`
proves it) — but this spec does not propose shipping that as the durable representation.

### 4.3 API contracts

**The merge rule. This is the contract this feature exists to preserve.**

Today `RenderBridge::collect_notes` (`crates/spectre-audio/src/bridge.rs:257-271`) pops events from
the RT-002 note lane in FIFO order and pushes them into the scratch **without sorting**. That is
correct today because `MidiIngress::block_events` (`crates/spectre-audio/src/midi.rs:135-145`)
sorts before its events enter the lane. Adding a second producer breaks that assumption in two
independent ways, and both would surface as `ProcessError::UnsortedEvents`
(`crates/spectre-dsp/src/io.rs:98`), which silences the entire block through
`PlanError::Process` (`crates/spectre-graph/src/lib.rs:487-492`) and
`bridge.rs:192-200`:

1. **Interleaving.** Clip events for frame 40 appended after lane events for frame 200 produce a
   non-monotonic `frame_offset`.
2. **Key collision.** `MidiIngress` numbers its events from a counter starting at `0`
   (`midi.rs:91`, incremented at `:130`). A clip player with its own counter starting at `0`
   produces two events that can share `(frame_offset, rank, sequence)` exactly.
   `ProcessContext::new` rejects `order <= previous` (`io.rs:97-99`) — equality is a rejection,
   not a tie.

The rule, therefore:

> **R4-5-MERGE.** All note events destined for one plan node in one render quantum are collected
> into one array and sorted **once**, by `(frame_offset, NoteEventKind::rank(), sequence)` — the
> same tuple `ProcessContext::new` validates at `crates/spectre-dsp/src/io.rs:96`. The comparison
> calls `NoteEventKind::rank` (`io.rs:148-158`). No second rank function, no second sort key, and
> no `Ord` impl on `NoteEventKind` is introduced. `spectre_core::sort_events` and
> `spectre_core::EventKind::order_rank` (`crates/spectre-core/src/event.rs:22-30`, `:52-54`) are
> a **different** contract over a **different** type (`TimedEvent`, absolute `SampleTime`) and
> MUST NOT be used on this path.

Sorting on the callback is permitted by the precedent the accepted ingress already relies on:
`sort_unstable_by` does not allocate, and the key is a total order because `sequence` is unique,
so an unstable sort is still deterministic (`crates/spectre-audio/src/midi.rs:6-8`, `:136-143`).
This spec reuses that argument rather than inventing one.

`sequence` uniqueness across producers is guaranteed by **band assignment, not renumbering**:

- Live-MIDI events keep the sequence `MidiIngress` assigned. Its counter starts at `0` and
  increments by one per admitted message (`midi.rs:91`, `:130`), so its values occupy the low
  band of `u64`.
- Clip events take `sequence = CLIP_SEQUENCE_BAND | block_local_index`, where
  `CLIP_SEQUENCE_BAND = 1 << 63` and `block_local_index` counts from `0` within the block. Only
  uniqueness *within one block* is required, because `ProcessContext::new` validates only the
  block's slice (`io.rs:91-101`).
- The bands cannot meet. `MidiIngress` would need `2^63 = 9,223,372,036,854,775,808` admitted
  messages to reach bit 63; at a sustained one million messages per second that is
  9.223e12 seconds, or about 292,000 years.

The user-visible consequence, stated rather than left as an accident: **at an identical frame
offset and identical rank, a live-performance event resolves before a clip event.** That is a
deliberate tie-break — the thing the player just did wins the tie — and it is deterministic.

**New and modified functions.**

```rust
// crates/spectre-audio/src/control.rs — MODIFIED

// Existing lane depths are unchanged: DEFAULT_NOTE_CAPACITY = 1_024,
// DEFAULT_TRANSPORT_CAPACITY = 64, DEFAULT_RECLAIM_CAPACITY = 32 (control.rs:17-19).

// A fourth lane carrying baked schedules to the render thread. Strict FIFO, counted overflow,
// rejected value returned to the caller. Built as spsc::bounded(SCHEDULE_LANE_CAPACITY) with the
// constant passed through unchanged: at 3 the request and the delivered capacity() are equal, so
// the lane admits exactly SCHEDULE_LANE_CAPACITY schedules and refuses the next (§4.2, I-10).
// See §8 Q2: decision 21 (decision-gates.md:45) names three lanes with two policies; adding a
// fourth is a shape change to an accepted row and needs Jeff's ratification, not this spec's
// assertion.
impl ControlSender {
    // Publish a baked schedule. Overflow is a counted app-thread defect and the schedule comes
    // back to the caller rather than being dropped.
    pub fn send_schedule(
        &mut self,
        schedule: Box<ClipSchedule>,
    ) -> Result<(), (ControlError, Box<ClipSchedule>)>;
}

impl ControlReceiver {
    // Take the next queued schedule, or None
    pub fn next_schedule(&mut self) -> Option<Box<ClipSchedule>>;
}

impl ControlError {
    // New variant: ScheduleLaneFull
}

impl ControlTelemetry {
    pub fn schedule_overflows(&self) -> u64;
}
```

```rust
// crates/spectre-audio/src/bridge.rs — MODIFIED

impl RenderBridge {
    // Attach clip playback to the bridge's single note node. Existing `new` is unchanged, so
    // every current caller compiles untouched; a bridge with no player behaves exactly as today.
    pub fn with_clip_player(self, player: ClipPlayer) -> Self;
}

impl BridgeTelemetry {
    // Blocks whose clip events would have exceeded CLIP_EVENT_RESERVE
    pub fn clip_events_refused(&self) -> u64;
    // Blocks that spanned more than MAX_BLOCK_SEGMENTS
    pub fn loop_segments_refused(&self) -> u64;
    // Schedules the render thread installed
    pub fn schedules_installed(&self) -> u64;
    // Retired schedules the reclaim lane refused, which the render thread must keep holding
    pub fn schedules_held(&self) -> u64;
}
```

**`RenderBridge::render`, revised order of operations.** Numbers are the current line positions;
insertions are marked.

| # | Step | Current location |
|---|---|---|
| 1 | `Instant::now()` | `bridge.rs:165` |
| 2 | Frame-capacity refusal — silence, count, **return** | `:166-173` |
| 3 | *(new)* Install a pending schedule if one is queued; retire the old one; on reclaim-lane overflow, hold it and count | — |
| 4 | *(new)* Capture `transport.position` before commands are applied | — |
| 5 | `apply_transport()` | `:175` |
| 6 | *(new)* If the position changed discontinuously, or a schedule was installed, `player.release_all()` | — |
| 7 | *(new)* `notes.clear()`; `player.emit_block(&transport, frames, &mut notes, CLIP_SEQUENCE_BAND)` | — |
| 8 | `collect_notes()`, appending lane events after the clip events **without clearing** | `:176`, `:258` |
| 9 | *(new)* Sort `notes` once by the R4-5-MERGE key | — |
| 10 | *(new)* `transport.advance(SampleDuration::new(frames as u64))` (`crates/spectre-core/src/transport.rs:96-114`) | — |
| 11 | `drain_parameters` | `:181-186` |
| 12 | `plan.process(...)` — on error, silence and count, **return** | `:188-200` |
| 13 | `publish_containment()`, `interleave()`, `blocks_rendered += 1`, `publish_headroom()` | `:202-207` |

Three consequences of this ordering are load-bearing and are asserted in §5:

- Step 2 returns before steps 3–10, so a frame-capacity refusal advances nothing and consumes
  nothing (§3.6 E-6).
- Step 10 precedes step 12, so a plan error loses that block's material with the playhead already
  moved (§3.6 E-7). The alternative — advancing after a successful process — would re-emit the
  same block, producing a stutter instead of a gap. Criterion 1F asks for silence rather than
  stale audio, so the gap is chosen deliberately.
- Step 8 must be changed from clearing the scratch (`bridge.rs:258`) to appending, or the clip
  events emitted at step 7 are discarded. This is the single smallest edit in the feature and the
  single easiest one to omit; §5.2 asserts it directly.

**Note scratch sizing.** `DEFAULT_NOTE_SCRATCH` is `256` (`bridge.rs:20`) and `collect_notes` fills
only up to `self.notes.capacity()` (`:259`). With `CLIP_EVENT_RESERVE = 512` reserved for clips,
a bridge with a clip player must be constructed with `note_scratch = MAX_NOTE_EVENTS_PER_BLOCK`
(1,024, `io.rs:52`) so that neither producer's reserve can push the other into
`notes_deferred` (`bridge.rs:78-80`) on a legal block. `DEFAULT_NOTE_SCRATCH` stays as-is for
buses with no schedule; this spec does not change the constant.

**Auth / permissions / pagination / rate limiting: N/A** — this is a local desktop application
with no network surface, no accounts, and no server.

### 4.4 State management

| State | Owner | Thread | Lifetime |
|---|---|---|---|
| `MidiClip`, `ClipPlacement`, `TrackClips` | `ProjectDoc` (`crates/spectre-project/src/lib.rs:29-38`) | App | Persisted (R4-7) |
| Selection (`selected_clip`) | `AppModel` (`crates/spectre-app/src/lib.rs:205-214`) | App | Session only, not persisted |
| Undo/redo of clip edits | `EditHistory` (`crates/spectre-project/src/command.rs:114-153`) | App | Session only |
| `ClipSchedule` | Built by the app, owned by `ClipPlayer` after install | App → render → app (reclaim) | Until superseded |
| Cursor + sounding table | `ClipPlayer` | Render | Until seek, stop, or swap |
| Counters | `BridgeTelemetry` / `ControlTelemetry` | Written render, read app | Process |

**No new state container is introduced on the app side.** Clip data lives in the existing project
document; selection lives in the existing `AppModel`; undo lives in the existing `EditHistory`.
The one genuinely new owner is `ClipPlayer`, injected into `RenderBridge` at construction via
`with_clip_player` (§4.3) and never reachable from the app thread afterward.

**RT-002 disposition (criterion 1B), stated explicitly.** Three existing lanes are used
unchanged: transport commands ride the strict-FIFO transport lane
(`control.rs:261-271`), live notes ride the strict-FIFO note lane (`:248-258`), and retired
schedules travel back on the existing reclaim lane (`:310-324`), which already returns the value
to the caller on overflow precisely so the audio thread never runs a destructor. The parameter
lane is untouched by this feature. One new lane is proposed for schedule delivery, with strict
FIFO and counted overflow — the decision-21 policy for non-parameter traffic
(`decision-gates.md:45`) — and its addition is routed to §8 Q2 because decision 21 names three
lanes and this changes that shape.

**Offline / draft persistence.** None beyond the project document. Autosave is decision 14's
concern at R5. A clip edit is not written to disk until the user saves, exactly as every other
project edit today.

**Local vs. server-synced:** N/A — there is no server. Non-goal per
`docs/00-product/vision.md:57` ("No cloud services, collaboration servers, or content stores").

### 4.5 Dependencies

- **New packages: none.** No crate is added to any `Cargo.toml`. The bake uses `TempoMap`
  (`spectre-core`); the schedule uses `NoteEvent` (`spectre-dsp`); the lane uses the existing
  `spsc` ring (`crates/spectre-audio/src/spsc.rs:57`).
- **New inter-crate edges: none from this spec.** `spectre-audio` gains no dependency on
  `spectre-project` (§4.1). `spectre-app` must gain a dependency on `spectre-audio`, but that is
  R4-1's edit (§7.4), not this one's.
- **New assets or resources: none.** No fonts, samples, images, or bundled content. §6.2.
- **Infrastructure: none.** No database, CDN, or third-party service.

### 4.6 Platform-specific considerations

- **No platform-conditional code.** `ClipSchedule` and `ClipPlayer` are integer and `f32`
  arithmetic over preallocated slices; they contain no `#[cfg(target_os)]`, no syscall, and no
  driver interaction. The bake uses `TempoMap`, whose arithmetic is `f64` and portable
  (`crates/spectre-core/src/tempo.rs:72-96`).
- **The Linux risk is inherited, not created.** Decision 23 (`decision-gates.md:49`) records that
  no Linux audio device has ever been opened, and `docs/06-plans/current-milestone.md:114` shows
  the Linux qualification row empty. Clip playback therefore runs on Linux only insofar as the
  backend does, and this spec authorizes **no Linux claim**. Decision 1 (`:25`) keeps macOS and
  Linux co-first-class, and this feature does not narrow that: its test surface (§5.1, §5.2) runs
  on both platforms today because it needs no device — the deterministic half of the bridge
  suite already runs without one.
- **The one platform-shaped hazard is buffer geometry, and it is E-6.** A backend that grants a
  block larger than `plan.max_frames()` causes the frame-capacity refusal at `bridge.rs:167`.
  R4-1 flags ALSA buffer-size negotiation as a live risk. For clips specifically this is benign
  in the sense that nothing is skipped (§3.6 E-6) and severe in the sense that the playhead
  stalls silently, so `frame_capacity_rejections` (`bridge.rs:73`) must be surfaced by whatever
  health readout R4-1 builds. This spec does not build one.
- **Version compatibility:** no OS API is used. `eframe 0.32.3` and Rust edition are unchanged.
- **Feature flags / gradual rollout:** none. A bridge constructed without `with_clip_player`
  behaves exactly as it does today, which is the rollout seam; no cargo feature is added.

### 4.7 Performance budget

- **Memory.** One `ScheduledNote` is 32 bytes (§4.2). A full schedule at `MAX_NOTES_PER_CLIP` is
  4,096 x 32 B = **128 KiB**. With R4-4's proposed `MAX_TRACKS = 16`, a full project's schedules
  are **2 MiB** resident, and **4 MiB** transiently while one retired copy per bus is in flight on
  the reclaim lane. The schedule lane adds to that only when the app thread publishes without the
  render thread consuming: at `SCHEDULE_LANE_CAPACITY = 3` (§4.2) a bus can hold one installed
  schedule, three queued, and one retired-but-unreclaimed copy, so the absolute ceiling is five
  copies per bus — **10 MiB** at sixteen simultaneously full 4,096-note schedules. That ceiling
  requires a stopped render thread and a maximal project at the same time; it is stated as the
  bound, not as the expected figure, and the expected figure remains 2 MiB.
  The bridge's note scratch grows from
  `DEFAULT_NOTE_SCRATCH` (256) to `MAX_NOTE_EVENTS_PER_BLOCK` (1,024) `NoteEvent`s on a
  clip-playing bus; `NoteEvent` is `usize + u64 + NoteEventKind` (`io.rs:24-29`), so at 32 bytes
  each that is a growth of 768 x 32 B = **24 KiB**. All of it is allocated once, on the app
  thread.
- **CPU / render time.** Per block, the player performs: a cursor scan bounded by
  `CLIP_EVENT_RESERVE` (512), at most `MAX_BLOCK_SEGMENTS` (2) segment walks, and one
  `sort_unstable_by` over at most `MAX_NOTE_EVENTS_PER_BLOCK` (1,024) elements. The sort is the
  dominant term at O(n log n) on 1,024 elements — but only when the block actually carries 1,024
  events, which the arithmetic in §4.2 shows requires a pathological 1-tick grid at maximum
  tempo. The realistic steady state is single-digit events per block and the sort is trivial.
  **No time budget or headroom threshold is asserted.** AF-5 prohibits a monitoring-latency
  threshold, and this project has exactly one hardware measurement — worst-case headroom 0.990 on
  one macOS device on a chain with no clip player (`docs/06-plans/current-milestone.md:113`).
  Extrapolating from it would be inventing a number. §5.4 specifies that headroom be *observed
  and recorded*, not asserted against a bound.
- **Network payload:** N/A — no network.
- **Storage.** One `ClipNote` serialized to the JSON codec (decision 3) is roughly 60–80 bytes of
  text; a 4,096-note clip is therefore on the order of 250–320 KB of JSON. This is an estimate of
  a text encoding, not a measurement, and it is flagged as such. R4-7 owns the codec's actual
  cost and §8 Q7 routes it.
- **Startup.** Bake cost is O(n) over the project's notes with one `TempoMap` binary search per
  note (`crates/spectre-core/src/tempo.rs:88-92`); at the 16-track ceiling that is 65,536 notes,
  which is bounded work on the app thread at load. Not measured; §8 Q10.

---

## 5. Test Specification

**Honesty statement.** Every command in this section runs today on macOS and Linux with no audio
hardware, because every test here drives the bridge or the plan directly — the pattern
`crates/spectre-audio/tests/bridge_plan.rs` already uses. No test in this section requires a
device, and none of them establishes anything about a real driver. The workspace gate is:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

(`docs/status/STATUS.md:46-48`.) Targeted runs during development:

```sh
cargo test -p spectre-project --test clip_model
cargo test -p spectre-audio  --test clip_schedule
cargo test -p spectre-audio  --test clip_bridge
cargo test -p spectre-app    --test app_model
```

### 5.1 Unit tests

**`crates/spectre-project/tests/clip_model.rs`** — constructors used: `ClipNote::new`,
`MidiClip::new`, `MidiClip::insert_note`, `MidiClip::set_length`, `ClipPlacement::new`,
`TrackClips::insert`, `TrackClips::remove`, `IdGen::new`/`next_id`
(`crates/spectre-core/src/id.rs:51`, `:56`), `BeatTicks::from_beats`
(`crates/spectre-core/src/time.rs:111`).

| # | Name | Setup | Assertion | Edge covered |
|---|---|---|---|---|
| U-1 | `a_note_outside_the_clip_is_refused` | Clip length 4 beats; note at beat 4 | `Err(ClipError::NoteOutsideClip { .. })`, `note_count()` unchanged | Half-open `[0, length)` per TIME-002 |
| U-2 | `a_zero_length_note_is_refused` | `ClipNote::new(start, BeatTicks(0), ..)` | `Err(ClipError::NonPositiveLength)` | A note with no duration cannot produce an `Off` after its `On` |
| U-3 | `note_bounds_match_the_event_contract` | Channel 16; note 128; velocity `f32::NAN`, `-0.1`, `1.1`, `f32::INFINITY` | Each `Err`; the accepted values 15 / 127 / 0.0 / 1.0 each `Ok` | Exactly the bounds `NoteEvent::is_valid` enforces (`io.rs:122-146`) |
| U-4 | `the_note_limit_is_refused_not_truncated` | Insert `MAX_NOTES_PER_CLIP` notes, then one more | `Err(ClipError::NoteLimit { limit: 4096 })` and `note_count() == 4096` | Refusal, never silent truncation |
| U-5 | `a_clip_longer_than_the_limit_is_refused` | `set_length(BeatTicks(MAX_CLIP_LENGTH_TICKS + 1))` | `Err(ClipError::ClipTooLong { .. })`; `MAX_CLIP_LENGTH_TICKS` itself is `Ok` | The bound is inclusive and the arithmetic is pinned |
| U-6 | `overlapping_placements_are_refused` | Placement at bar 1 length 4 bars; second at bar 3 | `Err(ClipError::OverlappingPlacement { .. })`; adjacent placement at bar 5 is `Ok` | Half-open adjacency is legal; overlap is not |
| U-7 | `placements_stay_sorted_by_start` | Insert at bars 9, 1, 5 in that order | `placements()` starts are `[1, 5, 9]` in ticks | Deterministic bake order |
| U-8 | `clip_ids_survive_reorder_and_round_trip` | Insert three placements, reorder, serialize, deserialize | Every `ObjectId` identical before and after | **CORE-001 reorder evidence** (`requirements-ledger.md:46`), which R4 explicitly gates on the first persisted collection |
| U-9 | `a_clip_edit_is_one_undoable_transaction` | Apply a note insert through `EditHistory::apply`, then `undo` | `note_count()` returns to its prior value; `redo` restores it | Atomicity via the existing command layer |
| U-10 | `bake_revision_advances_only_on_bakeable_edits` | Rename a clip; then move a placement | Rename leaves `bake_revision()` unchanged; the move advances it | A rename must not force a schedule republish |

**`crates/spectre-audio/tests/clip_schedule.rs`** — constructors used: `ClipSchedule::bake`,
`ClipSchedule::empty`, `ClipSchedule::notes`, `ClipPlayer::new`, `ClipPlayer::install`,
`ClipPlayer::emit_block`, `ClipPlayer::release_all`, `TempoMap::constant`
(`crates/spectre-core/src/tempo.rs:59`), `SampleRate::new`
(`crates/spectre-core/src/time.rs:38`), `Transport::new`/`apply`/`advance`
(`crates/spectre-core/src/transport.rs:72`, `:82`, `:96`), `ProcessContext::new`
(`crates/spectre-dsp/src/io.rs:77`).

| # | Name | Setup | Assertion | Edge covered |
|---|---|---|---|---|
| S-1 | `bake_places_a_beat_at_the_tempo_maps_sample` | `TempoMap::constant(120.0)`, 48 kHz, note at beat 1 | `notes()[0].start == SampleTime(24_000)` | Reuses the exact value `tempo.rs:129-133` already pins; the bake adds no second conversion |
| S-2 | `bake_ids_are_disjoint_from_ingress_ids` | Bake 100 notes | Every `id >= CLIP_NOTE_ID_BASE` and every `id != 0` | The disjointness `PulseInstrument`'s id matching depends on (`source.rs:188-189`) |
| S-3 | `emit_block_produces_contract_ordered_events` | Two notes ending and two starting at the same frame | `ProcessContext::new(rate, frames, out)` is `Ok`; the two `Off`s precede the two `On`s | The `rank` ordering (`io.rs:152-157`) is honored without restating it |
| S-4 | `clip_sequences_never_collide_with_ingress_sequences` | Merge one clip event and one ingress event with identical `frame_offset` and identical `rank` | `ProcessContext::new` is `Ok` — i.e. the two `sequence` values differ | The key-collision failure mode in §4.3 |
| S-5 | `a_note_spanning_blocks_emits_on_then_off_once_each` | Note of 3 blocks' length; render 5 blocks | Exactly one `On` in block 0 and exactly one `Off` in block 2, matching ids | Notes are not re-attacked per block |
| S-6 | `a_loop_wrap_releases_then_reattacks_at_the_same_frame` | Loop region ending mid-block; a note sounding at the wrap | At the wrap frame: an `Off` for the sounding id, then an `On` for the loop-start material; `ProcessContext::new` is `Ok` | Branch B, and that `rank` alone orders it |
| S-7 | `a_block_over_a_sub_block_loop_refuses_and_releases` | Loop region of 1 sample (legal — `transport.rs:163-172`), 256-frame block | Outcome is `SegmentLimitExceeded`; `out` contains exactly one `AllNotesOff` at offset 0 | `MAX_BLOCK_SEGMENTS`, fail closed, bounded work |
| S-8 | `exceeding_the_event_reserve_refuses_the_block` | Schedule producing `CLIP_EVENT_RESERVE + 2` events in one block | Outcome is `EventReserveExceeded`; `out.len() == 1` and it is `AllNotesOff`; no partial set | `CLIP_EVENT_RESERVE`, and no hung note from a half-emitted block |
| S-9 | `release_all_emits_all_notes_off_at_frame_zero` | Two notes sounding; call `release_all`; emit next block | First event is `AllNotesOff { channel: None }` at `frame_offset == 0` | Branch C; `PulseInstrument` clears on it (`source.rs:194-197`) |
| S-10 | `install_returns_the_previous_schedule` | Install schedule B over A | Returned box is A; `ClipPlayer` holds B | Off-thread reclamation; nothing is dropped on the render thread |
| S-11 | `an_empty_schedule_emits_nothing` | `ClipSchedule::empty()`, render 10 blocks | `out.is_empty()` every block; `ProcessContext::new(rate, frames, &[])` is `Ok` | Zero clips is a legal, silent state |
| S-12 | `bake_refuses_a_non_finite_velocity` | Iterator yielding `f32::NAN` velocity | `Err(ScheduleError::InvalidVelocity(_))`; `ClipSchedule` is not constructed | RT-003-adjacent containment at the app-thread boundary (see §5.2 I-6 for the plan-side half) |
| S-13 | `emit_block_allocates_nothing` | RT guard section around 1,000 `emit_block` calls at the event reserve | Zero allocations and zero deallocations recorded | RT-001, using the existing `rt_guard.rs` harness |

### 5.2 Integration tests

**`crates/spectre-audio/tests/clip_bridge.rs`** — constructors used: `EditableGraph::new`/
`add_node`/`connect`/`compile` (`crates/spectre-graph/src/lib.rs:129`, `:134`, `:149`, `:196`),
`RenderBridge::new` (`crates/spectre-audio/src/bridge.rs:133`), `RenderBridge::with_clip_player`,
`RenderBridge::telemetry` (`:152`), `RenderBridge::render` (`:162`), `RenderBlock`,
`control_channel` (`crates/spectre-audio/src/control.rs:195`), `ControlSender::send_note` (`:248`),
`ControlSender::send_transport` (`:261`), `ControlSender::reclaim` (`:274`), `PulseInstrument::new`
(`crates/spectre-dsp/src/source.rs:122`), `Gain`, `Saturator`.

| # | Name | Assertion | Why it would fail if the behavior regressed |
|---|---|---|---|
| I-1 | `a_clip_renders_audible_output_through_the_plan` | Peak of the interleaved output is nonzero, and `plan_errors() == 0` | If the merge or the emit is wrong the plan refuses and the output is exactly silent — the assertion is on a *nonzero* peak specifically so two silent buffers cannot agree |
| I-2 | `clip_and_live_notes_merge_into_one_sorted_block` | Push live-MIDI events onto the note lane *and* run a clip in the same block; `plan_errors() == 0` across 200 blocks | This is the whole feature. A missing sort, a cleared scratch (`bridge.rs:258`), or a colliding sequence each produce `UnsortedEvents` and a nonzero `plan_errors` |
| I-3 | `collect_notes_appends_rather_than_clears` | Emit 3 clip events, queue 3 lane events, render one block; the block's event count is 6 | Directly pins step 8 of §4.3, the smallest and most omittable edit |
| I-4 | `repeated_renders_of_one_schedule_hash_identically` | Two runs of 200 blocks; `hash_interleaved` (`crates/spectre-audio/tests/bridge_plan.rs:80`) equal; peak nonzero | Criterion 1D determinism, using the **existing** FNV-1a walk, not a new one |
| I-5 | `the_live_bridge_hash_matches_the_offline_render` | Same schedule, same devices, same seed; bridge hash equals the offline hash from `spectre-offline`'s walk (`crates/spectre-offline/src/lib.rs:271-283`) | Criterion 1D live/offline equivalence, extending the approach `docs/06-plans/current-milestone.md:49` records rather than adding a second comparison |
| I-6 | `a_non_finite_velocity_cannot_reach_a_device` | Construct a `NoteEvent` with `f32::NAN` velocity directly and feed it through the plan | `PlanError::Process { error: ProcessError::InvalidEvent, .. }` — `NoteEvent::is_valid` (`io.rs:122-146`) refuses it before any device sees it, and the block is exact silence |
| I-7 | `a_frame_capacity_refusal_does_not_advance_the_playhead` | Render a block larger than `plan.max_frames()`; then a legal block | `frame_capacity_rejections()` is 1, `blocks_rendered()` is 1 not 2, and the legal block emits the material that would have been at the stalled position | Pins §3.6 E-6 — including that `blocks_rendered` does **not** increment on the refusal path (`bridge.rs:166-173`) |
| I-8 | `a_seek_releases_every_sounding_clip_note` | Start a long note; send `TransportCommand::Seek`; render | Next block's first event is `AllNotesOff`; the instrument output returns to exact zero | Branch C, end to end |
| I-9 | `a_schedule_swap_reclaims_on_the_app_thread` | Install 5 schedules across 5 blocks with a drop-witness type; call `ControlSender::reclaim` | Every retired schedule drops on the app thread, never the render thread | RT-001 no-deallocation, using the drop-witness pattern the control suite already established (`docs/06-plans/current-milestone.md:45`) |
| I-10 | `a_full_schedule_lane_returns_the_schedule` | Publish schedules in a loop **until `send_schedule` returns `Err`**, never rendering — the same shape `crates/spectre-audio/tests/control_channel.rs:160-175` uses to fill the note lane and `crates/spectre-audio/tests/spsc_queue.rs:34-48` uses to fill the ring, with no arithmetic on the publish count. Assert, in order: (a) the loop accepted exactly `SCHEDULE_LANE_CAPACITY` schedules, which pins the lane's `Producer::capacity()` to the constant — the note-lane test asserts `>=` its requested depth because `bounded` rounds up, and `==` is asserted here only because 3 is a fixed point of `bounded` (CLIP-006); (b) the refusing call returned `Err((ControlError::ScheduleLaneFull, schedule))` with the box intact and its notes still readable; (c) `schedule_overflows() == 1`; (d) after one `next_schedule()`, exactly one further publish succeeds and the one after it refuses again, matching `spsc_queue.rs:43-46`'s "draining one element makes room for exactly one more" | Counted, not dropped — decision 21's policy shape. Assertion (a) is the one that would have caught the iteration-1 defect: that version published `SCHEDULE_LANE_CAPACITY + 1` and asserted the last call failed, which at `bounded`'s delivered capacity of 3 could never happen, so the test could never pass and never reached the counted-overflow path it names |
| I-11 | `render_with_a_clip_player_allocates_nothing` | RT guard section around 1,000 renders with clips and live notes both active | Zero allocations, zero deallocations | RT-001, extending `rt_guard.rs`'s existing coverage of `render` |
| I-12 | `a_deactivated_clip_produces_no_events` | Placement with `set_active(false)`; render 100 blocks | Zero events emitted; peak is exactly zero | `OBS-AB12-CLIP-001` behavior (`ableton-live-observations.md:49`), verified rather than assumed |

**`crates/spectre-project/tests/` addition:** a round-trip test asserting a project containing
clips serializes and deserializes to an equal document, extending the pattern at
`crates/spectre-project/src/lib.rs:148-154`. The atomic-write half is R4-7's.

### 5.3 UI / E2E tests

`crates/spectre-app/tests/app_model.rs` is a renderer-neutral model suite (27 tests per
`docs/status/STATUS.md` §Validation), and `smoke_cli.rs` is a headless launch check. Both patterns
extend here; neither drives pixels.

| # | Name | Assertion |
|---|---|---|
| E-1 | `creating_a_clip_selects_it_and_leaves_track_selection_intact` | `selected_clip()` is the new id; `selected_track_id()` is unchanged — the "no lost context" property in criterion 2A, asserted rather than described |
| E-2 | `selecting_a_clip_does_not_change_the_lens` | Lens stays `Arrange`; contrast with `open_device_in_shape`, which changes lens deliberately (`crates/spectre-app/src/lib.rs:338-345`) |
| E-3 | `an_invalid_clip_edit_reports_and_rolls_back` | The model is byte-identical after the refused edit and a feedback string names the specific `ClipError`, following `set_device_parameter_from_ui`'s pattern (`crates/spectre-app/src/lib.rs:462-474`) |
| E-4 | `the_empty_clip_lane_reports_no_clips` | Empty-state copy is exactly the §3.3 string — a truthful empty surface, per criterion 4F |
| E-5 | `every_clip_row_exposes_a_non_empty_accessible_label` | For a populated clip, the label contains name, position, length, note count, and active state |
| E-6 | `the_headless_launch_still_succeeds_with_clips_present` | `smoke_cli` exits zero with a project containing clips |

**No automated screenshot or pixel test is proposed.** The project has none today and adding an
image-comparison harness is out of this slice's scope.

### 5.4 Visual / manual verification

Configurations to check by hand, recorded in R4-9's manual QA protocol rather than asserted here:

- **Empty vs. populated:** empty clip lane; one clip; a track at `MAX_CLIPS_PER_TRACK`; a clip at
  `MAX_NOTES_PER_CLIP` (confirming the virtualized note list stays responsive).
- **Text size extremes:** smallest and largest platform text size — the clip rectangle must keep
  its time geometry while its label truncates (§3.7).
- **Screen size extremes:** narrow window (inspector collapses to a full-width sheet) and wide.
- **Theme variants:** light and dark. Active/inactive state must remain readable in both, and must
  be legible with hue removed — the color-independence requirement in §3.7.
- **Reduced motion:** with the platform setting on, confirm the playhead marker updates on bar
  boundaries and nothing else animates.
- **Telemetry observation, not assertion:** during a several-minute playback run, record
  `blocks_rendered`, `xruns`, `worst_headroom`, `plan_errors`, `frame_capacity_rejections`,
  `notes_deferred`, `clip_events_refused`, `loop_segments_refused`. `plan_errors` and
  `clip_events_refused` at nonzero are defects. **`xruns` and `worst_headroom` are recorded, not
  graded** — this project has one hardware measurement on one macOS device and no basis for a
  threshold, and AF-5 prohibits inventing one.

---

## 6. Compliance & Safety Gate

### 6.1 Sensitive data classification

- [x] **No sensitive data involvement.** A clip contains musical note positions, pitches,
  velocities, channels, and a user-chosen name. No personal, financial, health, credential, or
  location data is touched, stored, transmitted, or displayed. Nothing leaves the machine: there
  is no network path in the workspace, and cloud services are a named non-goal
  (`docs/00-product/vision.md:57`).
- [ ] Handles sensitive data
- [ ] Uses synthetic/test data only until compliance gate clears

The one thing worth naming: a clip name is free user text and appears in the project file and in
accessible labels. It is not escaped into any interpreter — the JSON codec is `serde_json`
(`crates/spectre-project/src/lib.rs:81`, `:91`) and labels go to egui as text.

### 6.2 Asset provenance

- [x] **No third-party assets.** No fonts (egui's `default_fonts` feature is already in
  `crates/spectre-app/Cargo.toml` and is unchanged), no images, no icons, no sample content, no
  demo project, no preset library. Clip test fixtures are note lists this spec authors.
- [ ] Uses third-party assets

**Format originality (criterion 3D).** The persisted representation is Spectre's own JSON schema
under the accepted envelope (decision 3), not Standard MIDI File, not a `.als`, not any vendor
format. No import or export of a foreign clip format is proposed; cross-DAW project compatibility
is a named non-goal (§6.4). The MIDI *value* conventions this spec uses — channel `0..=15`, note
`0..=127`, velocity — are the public MIDI 1.0 numeric space and are already fixed by the accepted
event contract (`docs/03-architecture/dsp-device-io.md:46`, `:51`), not borrowed from any product.

### 6.3 Language / claims audit

- [ ] Makes claims not supported by evidence — **no.** Every behavioral claim carries a source
  path, a requirement ID, a decision row, or an `OBS-` ID. Where evidence is absent it is named
  absent (§Appendix A). The two numbers this spec estimates rather than measures — JSON size in
  §4.7 and bake cost in §4.7 — are labeled as estimates in place.
- [ ] Promises capabilities not yet built — **no.** §7.1 states that `./spectre` produces no
  sound today (`docs/status/STATUS.md:37`), that R4-1 and R4-2 are specified and **not
  implemented**, and that this feature is `proposed`. Nothing in §1–§5 is written as though clip
  playback exists.
- [ ] Uses language restricted by domain regulations — **no.** No medical, financial, safety, or
  advertising claims.

**Self-audit against AF-6.** The words "should work", "simply", "just", "straightforward",
"seamless", "robust", "production-ready", and "of course" appear nowhere as descriptions of state
or difficulty in this document. Where an outcome is uncertain, §8 carries the question. Where a
number is unmeasured, §4.7 and §5.4 say so.

### 6.4 Regulatory alignment

Lens 3 of `gauntlet-output/criteria.md`, criterion by criterion:

- **3A — Milestone fit.** R4's scope is *"track to master, MIDI clip, a small original
  synth/effect, minimal UI, save/reload, bounce"* (`docs/06-plans/current-milestone.md:12`) and
  exit row `:83` is *"A MIDI clip plays through a track into master."* This spec produces exactly
  that row. Explicitly deferred and named as such: the session/launcher grid (R10 — `PROD-001` at
  `requirements-ledger.md:62`, provenance `OBS-AB12-SES-005` at `ableton-live-observations.md:44`
  and `OBS-BW53-LAUNCH-001` at `bitwig-studio-observations.md:30`), automation lanes (R9 —
  `PROD-002` at `requirements-ledger.md:63`), MIDI recording and overdub (R7 —
  `OBS-AB12-REC-004` at `ableton-live-observations.md:113`), MIDI transformations and time tools
  (`OBS-AB12-CLIP-011`/`-012` at `:59-60`), program-change events (`OBS-AB12-CLIP-010` at `:58`),
  per-clip time signature (`OBS-AB12-CLIP-004` at `:52`), grooves, follow actions
  (`OBS-AB12-LAUNCH-005` at `:81`), and the piano roll.
- **3B — Non-goal respect.** Nothing here proposes CLAP/LV2/AU hosting, plugin-format authoring of
  a first-party device, cross-DAW preset or project compatibility, cloud services or content
  stores, or video scoring. The clip format is Spectre's own (§6.2), which is the specific place
  a "compatibility" non-goal is most easily violated.
- **3C — Deliberately small first devices.** This spec adds **no DSP device**. `ClipPlayer` is not
  an `AudioProcessor` and does not implement the trait (`crates/spectre-dsp/src/io.rs:163-172`).
  `PulseInstrument` is unchanged; R4-6 replaces it under decision 15 (`decision-gates.md:39`) and
  inherits the same note contract with no clip-side edit.
- **3D — Originality.** Every numeric bound in §4.2 derives from Spectre's own arithmetic —
  `MAX_NOTE_EVENTS_PER_BLOCK`, `MAX_BPM`, `TICKS_PER_BEAT`, the `ScheduledNote` byte size, the
  block rate — and each derivation is printed beside its constant and rechecked in §7.2. No
  vendor limit is copied (decision 16, `decision-gates.md:40`; PROD-003,
  `requirements-ledger.md:64`). Ableton's clip *behaviors* inform three choices (snapping,
  deactivation, note chase) and each is cited to an `OBS-` ID and re-decided on Spectre's own
  terms, once by divergence (§Appendix A).
- **3E — Platform commitment.** §4.6. No platform-conditional code; no Linux claim; decision 23's
  debt named as inherited and undischarged.
- **3F — Accessibility trajectory.** §3.7, and the list-over-canvas choice in §3.1 which is the
  concrete non-foreclosure. Decision 17 (`decision-gates.md:41`) is a beta gate; this spec does
  not claim to satisfy it.

---

## 7. Gap Analysis vs. Current State

### 7.1 What exists today

Every claim below was checked by opening the file at the line given.

**Implemented — the note-event contract this feature must reuse.**

- `NoteEvent { frame_offset: usize, sequence: u64, kind: NoteEventKind }` at
  `crates/spectre-dsp/src/io.rs:24-29`. `NoteEventKind::{On, Off, AllNotesOff}` at `:33-49`.
- `MAX_NOTE_EVENTS_PER_BLOCK = 1_024` at `io.rs:52`.
- `ProcessContext::new` at `io.rs:77-107`. It rejects `events.len() > MAX_NOTE_EVENTS_PER_BLOCK`
  with `ProcessError::EventCapacity` (`:88-90`), rejects `frame_offset >= frames` or an invalid
  event with `ProcessError::InvalidEvent` (`:93-95`), builds the ordering tuple
  `(event.frame_offset, event.kind.rank(), event.sequence)` at `:96`, and rejects
  `order <= previous` with `ProcessError::UnsortedEvents` at `:97-99`. **The comparison is `<=`,
  so equal keys are a rejection.**
- `NoteEvent::is_valid` at `io.rs:122-146`: nonzero id, channel `<= 15`, note `<= 127`, velocity
  finite in `[0.0, 1.0]`; `AllNotesOff`'s channel is `None` or `<= 15`.
- **`NoteEventKind::rank` is public** at `io.rs:148-158`, returning `0` for `Off` and
  `AllNotesOff` and `1` for `On` (`:152-157`). Its doc comment at `:149-151` states the reason
  verbatim: *"Public so event producers sort by the same key block validation enforces, rather
  than maintaining a second definition that can drift out of step."*
- `validate_buffers` at `io.rs:175-198` returns `ProcessError::InvalidEvent` when a device that
  does not accept notes is handed a non-empty event slice (`:194-196`).

**Implemented — MIDI ingress (R3 slice 8), which this feature must merge with, not replace.**

- `crates/spectre-audio/src/midi.rs`, 223 lines. `MAX_BLOCK_EVENTS = 1_024` at `:14`.
- `MidiIngress` at `:65-74`: a `Box<[u32]>` active table of `CHANNELS * NOTES` = 16 x 128 (`:87`),
  a preallocated `pending: Vec<NoteEvent>` (`:88`), `next_id` starting at `1` (`:90`), `sequence`
  starting at `0` (`:91`), and a `late_messages` counter (`:92`).
- `push` at `:102-132`: refuses on a full buffer with `MidiError::BufferFull` (`:108-110`);
  computes `offset = timestamp - block_start` and refuses `offset >= frames` with
  `MidiError::AfterBlock` (`:111-114`); clamps a negative offset to `0` and increments
  `late_messages` (`:115-122`); assigns `sequence` and increments it (`:126-130`).
- `block_events` at `:135-145` sorts with `sort_unstable_by` on
  `(frame_offset, kind.rank(), sequence)` (`:137-143`) — the reuse this spec extends.
- `take_id` at `:199-203` allocates ascending from `1`, wrapping and skipping `0`.
- `classify` at `:153-196` handles note-on, note-off, running-status note-on with velocity `0` as
  a release (`:162`, `:173`), and CC 123 as `AllNotesOff` (`:188-193`).
- **Nothing constructs `MidiIngress` outside `crates/spectre-audio/tests/midi_ingress.rs`.** No
  production path feeds it from a driver today. Its 14 tests include
  `ingress_output_is_accepted_by_the_real_plan` at `midi_ingress.rs:287`.

**Implemented — the RT-002 transport and the callback bridge.**

- `crates/spectre-audio/src/control.rs`: `DEFAULT_NOTE_CAPACITY = 1_024`,
  `DEFAULT_TRANSPORT_CAPACITY = 64`, `DEFAULT_RECLAIM_CAPACITY = 32` at `:17-19`. `send_note` at
  `:248-258` counts an overflow and returns `ControlError::NoteLaneFull`. `retire` at `:310-324`
  hands retired state back to the caller on overflow, with the comment at `:311-313` stating that
  running a destructor there would violate RT-001. `RetiredState = Box<dyn Send>` at `:192`.
- `crates/spectre-audio/src/spsc.rs`: `bounded` at `:57`, `push` returning `Result<(), T>` at
  `:80`, `pop` at `:103`, `is_empty` at `:118`.
- `crates/spectre-audio/src/bridge.rs`: `DEFAULT_NOTE_SCRATCH = 256` at `:20`. `BridgeTelemetry`
  at `:24-39` with eleven counters and accessors at `:63-115`. `RenderBridge::render` at
  `:162-208`. **The frame-capacity refusal at `:166-173` fills silence, increments
  `frame_capacity_rejections`, and returns — before `apply_transport()` at `:175`, before
  `collect_notes()` at `:176`, and before `blocks_rendered` increments at `:204-206`.**
  `collect_notes` at `:257-271` clears the scratch (`:258`), pops from the note lane while
  capacity remains (`:259-264`), and increments `notes_deferred` if the lane is still non-empty
  (`:266-270`). `interleave` at `:274-289`.

**Implemented — time, transport, and the graph.**

- `TICKS_PER_BEAT = 960` at `crates/spectre-core/src/time.rs:9`; `SampleTime(pub i64)` at `:12-14`;
  `BeatTicks(pub i64)` at `:22-24`.
- `MIN_BPM = 10.0` and `MAX_BPM = 1200.0` at `crates/spectre-core/src/tempo.rs:10-11`;
  `samples_per_tick` at `:72-74`; `ticks_to_samples` at `:87-96`; `samples_to_ticks` at `:99-115`.
- `LoopRegion` is half-open with `new` requiring `end > start`
  (`crates/spectre-core/src/transport.rs:26-32`) and `contains` at `:35-37`. `Transport::apply` at
  `:82-93`; `Transport::advance` at `:96-114` wraps inside an enabled loop and returns a wrap
  count. A 1-sample loop is legal and exercised at `:163-172`.
- `CompiledPlan::process` at `crates/spectre-graph/src/lib.rs:456-533` validates note inputs
  (`:465-480`) and calls `ProcessContext::new` per step (`:487-492`).
  `PlanNoteInput { node, events }` at `:344-348`. `MAX_FLAT_INPUTS = 4` at `:15`.
- `PulseInstrument` at `crates/spectre-dsp/src/source.rs:111-119` is **monophonic**:
  `active_note: Option<(u32, u8)>` at `:117`. Its `process` at `:165-214` applies events at their
  frame offsets (`:176-201`), matches a release to its attack **by id** (`:188-193`), clears on
  `AllNotesOff` (`:194-197`), and **ignores a release whose id does not match** (`:198`).
  `accepts_notes: true` at `:24`.

**Implemented — project and app.**

- `ProjectEnvelope`/`ProjectDoc` at `crates/spectre-project/src/lib.rs:19-38`;
  `SCHEMA_VERSION = 1` at `:13`; `MAX_READABLE_SCHEMA = 1` at `:16`; unknown-field preservation
  via `#[serde(flatten)]` at `:25`, `:36-37`, proven at `:157-173`; `validate` at `:105-124`.
- `ProjectCommand` at `crates/spectre-project/src/command.rs:37`, `Transaction` at `:72`,
  `EditHistory` at `:114` with `apply` `:134`, `undo` `:146`, `redo` `:151`.
- `AppModel` at `crates/spectre-app/src/lib.rs:205-214` holds `tracks: Vec<TrackView>`,
  `selected_track`, `selected_device`, an `IdGen`, and a feedback string. `TrackView` at `:37-45`
  carries `id`, `name`, `muted`, `solo`, `armed`, `level`. `add_track` at `:405-422`.
  `open_device_in_shape` at `:338-345`.
- `crates/spectre-offline/src/lib.rs`: `fixture_events` at `:178-201` (one held note released on
  the final frame), the FNV-1a walk at `:271-283`, `render_vertical_slice` at `:288`.

**Absent — verified absent, not assumed.**

- **No clip type of any kind.** There is no `Clip`, `MidiClip`, `ClipNote`, `ClipPlacement`,
  `TrackClips`, or clip module in any crate. `crates/spectre-project/src/` contains exactly
  `command.rs` and `lib.rs`.
- **No scheduler.** Nothing converts a musical position into a per-block event stream. The only
  producer of `NoteEvent` outside tests is `MidiIngress`.
- **Nothing advances the transport on the render thread.** `RenderBridge::render`
  (`bridge.rs:162-208`) calls `apply_transport()` (`:250-254`), which drains commands, but never
  calls `Transport::advance`. The bridge's playhead does not move.
- **No `Ord` or `PartialOrd` on `NoteEvent` or `NoteEventKind`** (`io.rs:24`, `:32` derive only
  `Debug, Clone, Copy, PartialEq`). Sorting is by explicit tuple everywhere.

**Absent but easy to mistake for present — the second ordering key.**
`crates/spectre-core/src/event.rs` defines `EventKind` (`:12-18`), a **private** `order_rank`
(`:22-30`) ranking transport `0` > note-off `1` > note-on `2` > control `3` > param `4`,
`TimedEvent` (`:34-39`), a private `sort_key` (`:46-48`), and a public `sort_events` (`:52-54`),
all exported at `crates/spectre-core/src/lib.rs:14`. This is TIME-002's model
(`requirements-ledger.md:37`) over **absolute `SampleTime`**, and it is a **different type and a
different key** from `spectre_dsp::NoteEvent`. `grep -rn "sort_events\|TimedEvent" crates` returns
matches only in `crates/spectre-core/src/event.rs`, `crates/spectre-core/src/lib.rs:14`,
`crates/spectre-core/tests/properties.rs`, and
`crates/spectre-core/tests/time_transport_contract.rs` — **no production code path uses it.** A
clip scheduler that reached for `sort_events` because it is the obvious-looking "event sort" in
`spectre-core` would produce a second ordering definition, which is precisely what
`NoteEventKind::rank` was made public to prevent.

**Planned, specified, and not implemented.** R4 opens with no implementation
(`docs/06-plans/current-milestone.md:18`). R4-1 (live audio wiring) and R4-2 (runtime parameter
seam) are `gauntlet-output/specs/` documents that passed review; **neither is code**. R4-4 (track
model) is likewise a spec. `./spectre` does not use `spectre-audio` and Play produces no sound
(`docs/status/STATUS.md:37`).

**Gated.** Decision 22's parameter seam is `Accepted (design)` with implementation at R4
(`decision-gates.md:47`). Decision 23's Linux debt is undischarged (`:49`). Decision 17's
accessibility audit is a beta gate (`:41`).

### 7.2 Delta to spec

**New files**

1. `crates/spectre-project/src/clip.rs` — `ClipNote`, `MidiClip`, `ClipPlacement`, `TrackClips`,
   `ClipError`, `MAX_NOTES_PER_CLIP`, `MAX_CLIPS_PER_TRACK`, `MAX_CLIP_LENGTH_TICKS`.
2. `crates/spectre-audio/src/clip.rs` — `ScheduledNote`, `ClipSchedule`, `ClipPlayer`,
   `ClipBlockOutcome`, `ScheduleError`, `CLIP_EVENT_RESERVE`, `MAX_BLOCK_SEGMENTS`,
   `SCHEDULE_LANE_CAPACITY`, `CLIP_NOTE_ID_BASE`, `CLIP_SEQUENCE_BAND`.
3. `crates/spectre-project/tests/clip_model.rs` — U-1…U-10.
4. `crates/spectre-audio/tests/clip_schedule.rs` — S-1…S-13.
5. `crates/spectre-audio/tests/clip_bridge.rs` — I-1…I-12.

**Modified files**

| File | Change |
|---|---|
| `crates/spectre-project/src/lib.rs` | `pub mod clip;` re-exports; `ProjectDoc` gains a clips field; `validate` (`:105-124`) gains clip invariants |
| `crates/spectre-audio/src/lib.rs` | `pub mod clip;` and re-exports |
| `crates/spectre-audio/src/control.rs` | Fourth lane; `send_schedule` / `next_schedule`; `ControlError::ScheduleLaneFull`; `ControlTelemetry::schedule_overflows` |
| `crates/spectre-audio/src/bridge.rs` | `with_clip_player`; the §4.3 step insertions; **`collect_notes` at `:258` stops clearing**; four new telemetry counters |
| `crates/spectre-audio/tests/rt_guard.rs` | Guard `emit_block` and clip-active `render` (S-13, I-11) |
| `crates/spectre-app/src/lib.rs` | `selected_clip`, clip commands, clip presentation structs, feedback on `ClipError` |
| `crates/spectre-app/src/main.rs` | Clip lane and clip inspector in the `Arrange` lens |
| `crates/spectre-app/tests/app_model.rs` | E-1…E-5 |
| **`docs/01-requirements/requirements-ledger.md`** | **Seven rationale rows — see below. Required by PROD-003 (`:64`) and decision 16 (`decision-gates.md:40`); prose beside a constant does not discharge it** |
| `docs/01-requirements/traceability.md` | Rows linking TIME-002 and CORE-001 reorder evidence to the new tests |
| `docs/status/STATUS.md`, `docs/status/NEXT.md` | Slice 5 closure and current state, on landing |
| `docs/03-architecture/dsp-device-io.md` | One paragraph under §Events recording R4-5-MERGE — that a bus may have multiple producers, that they merge into one sorted array, and that `rank` is the only key |

**Rationale rows scheduled into `docs/01-requirements/requirements-ledger.md`** (proposed until
Jeff accepts; each carries its own derivation, none copies a vendor limit):

| ID | Bound | Value | Rationale, restated and re-checked |
|---|---|---|---|
| CLIP-001 | `MAX_NOTES_PER_CLIP` | 4,096 | 32 B/note x 4,096 = 131,072 B = **128 KiB** per track schedule; x16 tracks (R4-4 proposed) = **2 MiB**; x2 for one retired schedule per bus in flight on the reclaim lane = **4 MiB** (the reclaim-path figure; the schedule lane adds up to `SCHEDULE_LANE_CAPACITY` more per bus and §4.7 states the combined ceiling). Musical check: 4,096 x 240 ticks (1/16 grid) = 983,040 ticks; ÷960 = **1,024 beats** = **256 bars** of 4/4 |
| CLIP-002 | `MAX_CLIPS_PER_TRACK` | 64 | 4,096 ÷ 64 = **64** notes per placement average — the smallest average that is still material rather than fragments. Derived from CLIP-001, not chosen separately |
| CLIP-003 | `MAX_CLIP_LENGTH_TICKS` | 1,658,880,000 | TIME-003's 24-hour floor (`requirements-ledger.md:38`) at `MAX_BPM = 1200` (`tempo.rs:11`): 86,400 s x 1,200 ÷ 60 = **1,728,000 beats**; x 960 (`time.rs:9`) = **1,658,880,000 ticks**. Well inside `i64` |
| CLIP-004 | `CLIP_EVENT_RESERVE` | 512 | Half of `MAX_NOTE_EVENTS_PER_BLOCK` = 1,024 (`io.rs:52`), so neither producer can push the merged array past the ceiling that returns `EventCapacity` (`io.rs:88-90`). Density check at 48 kHz / 256 frames / `MAX_BPM`: `samples_per_tick` = 48,000 x 60 ÷ (1,200 x 960) = **2.5**; 256 ÷ 2.5 = **102.4 ticks**; at 1-tick spacing with **one note per tick** that is 102 attacks + 102 releases = **204** events. 512 ≥ 204 with 2.5x margin. Scope of the claim, stated: this bounds the densest **monophonic** block, not the densest representable one — `MidiClip::insert_note` refuses only over-capacity and out-of-`[0, length)` notes, so notes stacked on a shared start tick are legal and a 10-note chord per tick would emit ~1,020 attacks before a single release. Polyphonic density is bounded by refusal, not by 512: E-8 counts `clip_events_refused` and emits `AllNotesOff` for the block |
| CLIP-005 | `MAX_BLOCK_SEGMENTS` | 2 | One loop wrap per block. What it refuses is a loop region shorter than **one granted block**, and the granted block is negotiated by the driver (R4-1; decision 20's ALSA baseline, `decision-gates.md:44`), not fixed by Spectre — so the bound is stated against `plan.max_frames()`, not against 256. At 48 kHz / 960 PPQ a block spans `frames × BPM ÷ 3,000` ticks: 256 → **102.4** ticks at `MAX_BPM` (10.2 at 120 BPM), 1,024 → **409.6** (~0.43 beat) at `MAX_BPM` (41.0 at 120 BPM), 2,048 → **819.2** (~0.85 beat) at `MAX_BPM` (81.9 at 120 BPM). **Limitation stated rather than assumed:** the "shorter than a 32nd note (120 ticks)" immunity holds at 256 frames only; it is a property of the granted block, not of the bound. At ordinary tempos even 2,048 frames stays under a 32nd note, so only the joint extreme of a large granted block *and* near-maximum tempo refuses musically ordinary material — and if R4-1 negotiates blocks above 1,024 frames this bound must be derived from `plan.max_frames()` rather than left at 2. RT-001 (`:28`) requires bounded callback work; `Transport::advance` itself tolerates `u32::MAX` wraps (`transport.rs:163-172`); refusal is counted (E-9) and fails closed to `AllNotesOff` |
| CLIP-006 | `SCHEDULE_LANE_CAPACITY` | **3** | **Derived from the primitive, not from a preference.** `spsc::bounded` computes `requested = capacity.max(MIN_CAPACITY) + 1` (`MIN_CAPACITY = 2`, `spsc.rs:16`), rounds up with `next_power_of_two`, and reports `capacity() = slots_len - 1` because one slot is intentionally left empty (`spsc.rs:57-68`, `:96-98`). Every argument from 0 through 3 yields the same ring — **4 slots, usable capacity 3** — so a usable depth of 2 is unreachable at any argument, and 3 is the only request that equals what it receives. The steady state needs two (one installed and in flight, one publishing); the third slot is the primitive's floor and is named as such rather than justified after the fact. The lane refuses on the **fourth** queued schedule, i.e. four publishes with no intervening consume; the render thread drains at most one per block, 48,000 ÷ 256 = **187.5** per second, so a refusal means a stopped render thread or an app-thread defect, not a fast editor. Overflow is counted and the rejected schedule is returned, not dropped (`spsc.rs:80`). I-10 asserts `Producer::capacity() == SCHEDULE_LANE_CAPACITY` so the constant and the ring cannot drift again |
| CLIP-007 | `CLIP_NOTE_ID_BASE` | `0x8000_0000` | Upper half of `u32` reserved so clip IDs are disjoint from `MidiIngress`'s ascending-from-1 allocator (`midi.rs:199-203`), because `PulseInstrument` matches a release to its attack by id (`source.rs:188-189`). Collision horizon: 2,147,483,648 ingress note-ons ÷ 1,000 per second = 2,147,484 s ≈ **24.9 days** of continuous input — reachable by a machine, not a player. §8 Q1 |

**Migrations / schema changes:** one field on `ProjectDoc`. Whether it bumps `SCHEMA_VERSION`
(`crates/spectre-project/src/lib.rs:13`) is R4-7's call — §8 Q7.

**New dependencies:** none (§4.5).

### 7.3 Estimated scope

**L.** Justification, by part:

- **M** — the project-side clip model. It is validation and collection management over existing
  patterns; `TrackClips` closely resembles what R4-4 proposes for `TrackList`.
- **M** — the app surfaces. Two new panels in an existing lens, no new lens, no canvas.
- **L on its own** — `ClipPlayer::emit_block`. Segment splitting across a loop boundary, a
  sounding-note table that survives block boundaries, correct release emission at wrap and at
  seek, and doing all of it with no allocation is where the defects will be. S-5, S-6, S-7, and
  S-9 exist because each is a distinct way to hang a note or silence a block.
- **S but high-consequence** — the bridge merge. Four inserted steps and one changed line
  (`collect_notes` at `bridge.rs:258`). Small, and the single most likely place the feature
  breaks silently.

Not XL: no DSP device, no graph change, no new crate, no new external dependency, no persistence
implementation (R4-7), and no UI framework work beyond two egui panels.

### 7.4 Blocking dependencies

| Dependency | State | Why it blocks |
|---|---|---|
| **R4-1 — live audio wiring** | Spec passed review at 2.950; **not implemented** | `./spectre` does not depend on `spectre-audio` at all (`crates/spectre-app/Cargo.toml`). Without R4-1 a clip can be modeled and unit-tested but cannot be *heard*, and §1.3's primary signal is unreachable from the application |
| **R4-4 — track model** | Spec exists; **not implemented** | A clip is placed on a track. Today `TrackView` (`crates/spectre-app/src/lib.rs:37-45`) is presentation state with no signal path and no project-side `Track`. `TrackClips` needs an owner |
| **R4-2 — runtime parameter seam** | Spec passed at 3.000; **not implemented** | Soft. Clips play without it. It becomes hard the moment a clip's track needs a live level change, which is R4-4's mixer path |
| **R4-6 — small synth** | Not specified here | Soft. `PulseInstrument` is sufficient to prove the contract, but it is monophonic (`source.rs:117`), so clip polyphony is unhearable until R4-6 |
| **R4-7 — persistence** | Spec exists; not implemented | Downstream, not blocking. Clips are the first persisted child collection and therefore carry CORE-001's reorder evidence (`requirements-ledger.md:46`); U-8 provides the test, R4-7 provides the file |
| **§8 Q1, Q2, Q5** | Open | Q1 and Q2 change accepted or implemented artifacts (the ingress allocator; decision 21's lane shape). Q5 changes audible behavior. None should be decided by an implementer |

No external gate. No hardware requirement — every test in §5 runs without a device.

---

## 8. Open Questions

- **Q1 — Should `MidiIngress` refuse at the clip note-ID band boundary?** `take_id`
  (`crates/spectre-audio/src/midi.rs:199-203`) increments and wraps, skipping only zero. Reserving
  `>= 0x8000_0000` for clips (CLIP-007) is safe by construction *until* ingress reaches that
  value, which the arithmetic in §7.2 puts at roughly 24.9 days of continuous 1,000-note-per-second
  input — reachable by a machine left running. Options: (a) leave ingress unchanged and accept the
  horizon; (b) make `take_id` wrap to `1` at `CLIP_NOTE_ID_BASE`, a two-line change to implemented,
  tested code; (c) widen note identity beyond `u32`, which changes the accepted event contract.
  This spec proposes (b) and asserts nothing. — **blocks:** §4.2 `CLIP_NOTE_ID_BASE`, §7.2
  CLIP-007.
- **Q2 — Does the schedule lane fit decision 21, or amend it?** Decision 21
  (`docs/01-requirements/decision-gates.md:45`) names three lanes with two policies. A schedule is
  neither a parameter (latest-wins per target) nor a note (strict FIFO, never dropped) — it is a
  structural swap. The proposal in §4.3 is a fourth strict-FIFO lane with counted overflow and
  reclamation on the existing reclaim lane, which follows decision 21's *shape*. But adding a lane
  to an accepted row is Jeff's, not an implementer's. The alternative is latest-wins on a single
  slot, which loses nothing because only the newest schedule matters — and which would be a
  cleaner fit than FIFO, at the cost of a `Box` that has to be swapped rather than a value that
  has to be published. Iteration 2 adds one fact to that trade: `spsc::bounded` cannot build a lane
  shallower than three (`MIN_CAPACITY = 2` at `crates/spectre-audio/src/spsc.rs:16`, plus the
  always-empty slot and the power-of-two round-up at `:57-68`), so the FIFO option necessarily
  carries a slot the design has no use for — which CLIP-006 now states rather than hides. That is
  evidence for the single-slot alternative, not a decision. — **blocks:** §4.3, §4.4, §7.2
  CLIP-006.
- **Q3 — Are seven new ledger rows the right granularity?** PROD-003 (`:64`) requires a rationale
  for every numeric limit. CLIP-002 derives from CLIP-001 and CLIP-005 derives from the same block
  geometry as CLIP-004; they could be one row each or two rows with sub-derivations. This spec
  chose one row per constant so a future reader finds the constant by name. — **blocks:** §7.2.
- **Q4 — Should overlapping placements be permitted once a polyphonic instrument exists?** §4.2
  refuses them because R4's only instrument is monophonic (`source.rs:117`). That is the right
  R4 answer and possibly the wrong R5 answer. Deciding now costs nothing; discovering it after
  R4-6 costs a schema change. — **blocks:** §4.2 `TrackClips::insert`, and R4-6's intake.
- **Q5 — Does Spectre chase notes on seek?** `OBS-AB12-ARR-010`
  (`docs/02-reference-research/ableton-live-observations.md:34`) records that Ableton chases MIDI
  notes **by default**, so a note already sounding at the seek target is heard when playback starts
  mid-note, with an Options-menu toggle. This spec proposes that Spectre does **not** chase in R4:
  a seek into the middle of a note produces silence until the next attack. The reason is honest
  rather than principled — chasing requires deciding what velocity and what phase a partially
  elapsed note resumes with, and `PulseInstrument` resets phase on every attack
  (`crates/spectre-dsp/src/source.rs:186`), so a chased note would start from phase zero and sound
  like a late attack rather than a continuation. That is a real behavioral divergence from the one
  benchmark with a citable record on this exact surface, and it is Jeff's to accept or reverse. —
  **blocks:** §3.2 Branch C, §5.1 S-9.
- **Q6 — Gap or stutter on a plan error?** §4.3 advances the transport before `plan.process`, so a
  refused block loses its material and the playhead moves on (a gap). Advancing after a successful
  process would re-emit the same block (a stutter). Criterion 1F's "silence rather than stale
  audio" points at the gap and this spec chose it, but `plan_errors` should be zero in a healthy
  system and the choice only matters when something is already wrong. — **blocks:** §4.3 step 10,
  §3.6 E-7.
- **Q7 — Does adding a clips field bump `SCHEMA_VERSION`?** `SCHEMA_VERSION = 1` and
  `MAX_READABLE_SCHEMA = 1` (`crates/spectre-project/src/lib.rs:13`, `:16`). Clip data could ride
  in the existing `unknown` flatten map (`:36-37`) without a bump, which the round-trip test at
  `:157-173` proves survives a rewrite — but that is a workaround, not a representation. R4-7 owns
  the persistence contract and should own this. — **blocks:** §4.2 migrations, §7.2.
- **Q8 — What does R4's clip creation surface actually look like?** §3.1 specifies a lane, an
  inspector, and a note list. The workflow field study's prohibited-conclusions block forbids
  promoting any workflow archetype, so this spec cannot claim that list-first entry is what users
  want; it claims only that it is keyboard-complete on day one and does not foreclose a piano roll
  (§3.7). If Jeff wants the piano roll in R4, §3.1 changes and the accessibility argument needs a
  parallel accessible representation. — **blocks:** §3.1, §3.3.
- **Q9 — Which physical keys?** AF-5 forbids this spec from fixing a default map, and it does not.
  Someone must eventually choose, on evidence this project does not yet have. Recording the
  question here so it is not decided by accident in an implementation. — **blocks:** §3.4.
- **Q10 — Is the bake cost acceptable at the project ceiling?** §4.7 estimates O(n) with a
  `TempoMap` binary search per note (`crates/spectre-core/src/tempo.rs:88-92`), 65,536 notes at
  the 16-track ceiling. That is an estimate, not a measurement, and it happens on the app thread
  at load and on every structural clip edit. If it is slow it needs incremental re-bake, which is
  a different design. — **blocks:** §4.7.
- **Q11 — Should `spectre_core::sort_events` be marked or removed?** §7.1 establishes that
  `TimedEvent` and `EventKind::order_rank` (`crates/spectre-core/src/event.rs:22-30`, `:52-54`) are
  used by no production path and are exported at `crates/spectre-core/src/lib.rs:14`. They are
  TIME-002's model and may well be the right shape for a future event kernel, but as long as they
  are exported next to a *different* ordering key they are a standing invitation to fork the
  contract. Options: leave them, document the distinction in `dsp-device-io.md` §Events (this spec
  proposes that edit in §7.2 regardless), or narrow their visibility. **This spec does not propose
  deleting them** — they are pre-existing and out of this slice's surgical scope. — **blocks:**
  nothing; raised because it is the trap this feature is most likely to fall into.

---

## Appendix A — Benchmark evidence used, and where it does not exist

Jeff's AAA benchmark set is **Ableton Live, Logic Pro, Serum 2, Phase Plant, VCV Rack 2**
(`gauntlet-output/criteria.md` Lens 2). Bitwig Studio is not in the set but is valid corroborating
evidence.

**What the corpus supports and this spec used:**

| Claim | Record | Location | How Spectre treats it |
|---|---|---|---|
| Clips snap to the grid, other clip edges, and time-signature changes; a held modifier bypasses snapping | `OBS-AB12-ARR-004` | `ableton-live-observations.md:28` | **Follows.** §3.4 |
| Deactivated clips do not play when launched or during arrangement playback | `OBS-AB12-CLIP-001` | `:49` | **Follows** the behavior; presentation is Spectre's own (§3.7, tested at I-12) |
| Playback chases MIDI notes by default so notes sound when playback starts mid-note | `OBS-AB12-ARR-010` | `:34` | **Diverges** in R4, with the reason stated and routed to §8 Q5 |
| Clip time signature is display-only and does not affect playback | `OBS-AB12-CLIP-004` | `:52` | **Deferred.** R4 has no per-clip signature; the project `MeterMap` is the only source |
| Track device chains are always stereo even with mono input | `OBS-AB12-ROUTE-003` | `:89` | **Corroborates** the existing stereo-only v1 graph (`crates/spectre-graph/src/lib.rs:12`) |
| Clips are containers for notes plus control and automation data; "one DAW, two sequencers" over the same tracks | `OBS-BW53-CON-002` | `bitwig-studio-observations.md:26` | **Partially follows.** R4 ships the notes half only; the control/automation half is R9 and the second sequencer is R10 |
| Arranger clips play at designated timeline positions; Launcher clips are launched at will, with an explicit return-authority control | `OBS-BW53-LAUNCH-001` | `:30` | **Deferred, and named.** This is PROD-001's provenance (`requirements-ledger.md:62`) together with `OBS-AB12-SES-005` (`ableton-live-observations.md:44`). R4 ships timeline authority only; §3.1 defers the grid to R10 |
| Modules should output 0 on NaN/infinity | `OBS-VCV-VOLT-006` | `synth-modular-observations.md:50` | **Follows**, already, via RT-003 (`requirements-ledger.md:30`). This feature adds no DSP node; its containment obligation is that a non-finite velocity cannot reach one (S-12, I-6) |

**Convergent patterns (criterion 2E).** Two independent products converge on three things this
spec follows: clips as first-class named, positioned, toggleable containers of notes
(`OBS-AB12-CLIP-001`, `OBS-BW53-CON-002`); grid snapping with a modifier bypass
(`OBS-AB12-ARR-004`, and Bitwig's point-edit modifier convention at `OBS-BW53-AUTO-005`,
`bitwig-studio-observations.md:40`, is the same idiom in a different lane); and an explicit
timeline-versus-launcher playback authority per track (`OBS-AB12-SES-005`, `OBS-BW53-LAUNCH-001`),
which Spectre already carries as PROD-001 and which this spec **does not implement** — it ships
only the timeline half and says so.

**Where the corpus is silent, named rather than filled in (criterion 2G):**

1. **No benchmark record describes how a clip's notes become a scheduler's per-block event
   stream.** The corpus documents clip *behavior* — launch modes, follow actions, quantization,
   warping — because it is extracted from user manuals, which do not document engines. Every
   engine-level decision in §4.3 (the merge rule, the sequence banding, the ID banding, the
   segment bound) is therefore derived from Spectre's own accepted contracts, and none of it cites
   a benchmark. **Recorded as a research need.**
2. **Logic Pro contributes nothing.** `docs/02-reference-research/logic-pro.md:10-11` is `draft` /
   `inventory-only` and there are **zero** `OBS-` records. No Logic Pro behavior is asserted
   anywhere in this spec. **Recorded as a research need**, and specifically for clip editing,
   where Logic's model would be the most useful contrast to Ableton's.
3. **Serum 2 has exactly two records**, `OBS-SR2-CPU-001` (`synth-modular-observations.md:54`,
   per-oscillator unison CPU guidance) and `OBS-SR2-KB-001` (`:55`, a six-article support-site
   inventory with no public manual). Its dossier is `draft` / `blocked-source-gap`
   (`docs/02-reference-research/serum-2.md:10-11`). **Neither touches clips**, and neither is
   cited here.
4. **Phase Plant and VCV Rack 2 have no clip records at all.** Their 11 and 6 records are
   generator, routing, unison, and voltage-convention material (`synth-modular-observations.md:25`
   onward, `:45` onward). Phase Plant is not a sequencer host and VCV's sequencing is modular.
   Their one relevant contribution is `OBS-VCV-VOLT-006`, already carried by RT-003.
5. **No benchmark record on clip capacity limits.** No product in the corpus documents a maximum
   notes-per-clip or clips-per-track. That silence is why every bound in §7.2 is derived from
   Spectre's own arithmetic — the absence of a vendor number is not a gap to fill by looking one
   up, it is the condition decision 16 (`docs/01-requirements/decision-gates.md:40`) assumes.

**Differentiation (criterion 2F).** One thing Spectre does here that the benchmark set is not
documented as doing, and one honest limitation of that claim.

The differentiator is **a single event path with a single ordering key, stated as a contract and
enforced by the renderer itself.** Spectre's clip player and its live-MIDI ingress produce the same
type, sort by the same public function, and are validated by the same code that the offline
renderer uses (`crates/spectre-dsp/src/io.rs:96`, `crates/spectre-graph/src/lib.rs:487-492`). A
merge defect is not a listening bug found later; it is a refused block and a nonzero counter. No
record in the corpus describes a competitor's ordering guarantee, so this is not a claim that they
lack one — it is a claim about what Spectre commits to and can demonstrate.

The honest limitation: this is an *engineering* differentiator that a musician experiences only as
"it always plays the same." The benchmark set is far ahead of R4 on everything a musician would
name — launch modes (`OBS-AB12-LAUNCH-001`, `ableton-live-observations.md:77`), follow actions
(`OBS-AB12-LAUNCH-005`, `:81`), quantization (`OBS-AB12-LAUNCH-003`, `:79`), MIDI transformations
(`OBS-AB12-CLIP-012`, `:60`), and recording (`OBS-AB12-REC-010`, `:119`). Parity is not
completeness and completeness is not the goal (`docs/00-product/vision.md` non-goals); R4 is a
credible alpha, and this spec claims nothing beyond one clip playing correctly through one track.

---

**End of spec.**
