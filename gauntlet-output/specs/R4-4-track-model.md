<!--
Author: Jeff
Date: 2026-08-15
Description: R4-4 spec — a persisted-ready track model and an explicit track-to-master summing path compiled through the existing GRAPH-001 split
Notes: The feature is unstarted. What exists today is a UI-only TrackView whose mute/solo booleans
  reach no signal path at all; AppModel::add_track and the track sidebar do exist, contrary to the
  discarded 2026-08-12 spec's framing, and §7.1 states exactly what they are and are not. The one
  genuinely new architectural fact is that today's graph refuses two connections into one input bus,
  so a track-to-master path is impossible without an explicit summing device. ./spectre makes no
  sound today and R4-1 is spec'd but not implemented.
-->

# Spec: Track Model

**Feature ID:** `R4-4` (`track-model`)
**Parent feature:** `R4` Credible Alpha (root)
**Spec author agent:** gauntlet spec agent, R4-4 leaf
**Date:** 2026-08-15
**Iteration:** 1

- **Status:** proposed
- **Last verified:** 2026-08-15 (source read at commit `2e005e5`, branch `rename/geist-to-spectre`)
- **Scope:** an ordered track model with stable identity, and a compiled signal path from every track through an explicit summing bus into a master output — model, routing, compilation, and the app surfaces that read them
- **Decision authority:** Jeff
- **Upstream sources:** `docs/00-product/vision.md`, `docs/01-requirements/requirements-ledger.md` (CORE-001, CORE-002, GRAPH-001, GRAPH-002, RT-001/002/003, PROD-001, PROD-003), `docs/01-requirements/decision-gates.md` (rows 1, 4, 6, 8, 13, 15, 16, 17, 21, 22, 23), `docs/03-architecture/dsp-device-io.md`, `docs/06-plans/current-milestone.md`, `docs/06-plans/rebuild-roadmap.md`, `docs/status/NEXT.md` slice 4
- **Downstream dependents:** R4-5 (midi-clips), R4-6 (first-devices), R4-7 (project-persistence), R4-8 (offline-bounce), R4-9 (e2e-and-qa); R6 (tracks/routing/mixer) inherits this model
- **Supersedes:** the discarded `specs/track-management.md` (authored 2026-08-12, deleted 2026-08-14, recorded in `gauntlet-output/manifest.md` §"Run history"). That file described a track model that did not exist and does not exist now, asserted `Track::arm()` and `Track::mute()` methods that have never existed, and fixed a `⌘T`/`⌘U`/`⌘M` default shortcut map. This spec supersedes it in subject only; none of its content is carried forward.
- **Superseded by:** none
- **Open decisions:** §8 Q1–Q10. Q1 (the track ceiling), Q4 (the fifth device layout), and Q5 (widening the graph's flat-input bound) change accepted material and are escalated to `gauntlet-output/decisions-needed.md` rather than asserted.
- **Known gaps:** the accepted corpus contains **no** citable observation about a maximum track count, a summing-bus architecture, or a fader law for any of the five benchmarks; Logic Pro has zero behavioral records and Serum 2 has two, neither about tracks or mixing. All three are recorded as research needs in Appendix A, not filled in.

This spec is subordinate to the conflict precedence in `docs/README.md`. Where it proposes
changing accepted material — the device-layout list in `docs/03-architecture/dsp-device-io.md`
and the v1 flattened-input bound in `crates/spectre-graph/src/lib.rs` — it says so explicitly,
proposes supersession, and routes the question to §8 and `decisions-needed.md`, per AF-1. It
asserts no amendment by implication.

---

## 1. Purpose

### 1.1 One-sentence job

A musician can create, name, order, level, mute, and solo more than one track, and hear
every unmuted track summed into a single master output — so that "a track" becomes a
thing in the audio path rather than a row in a list.

### 1.2 Why it matters

Spectre has a signal path with exactly one lane. `crates/spectre-offline/src/lib.rs:204–285`
compiles precisely three nodes — `PulseInstrument → Gain → Saturator` — and that fixed
chain is the only thing the compiled plan has ever executed. There is no place to put a
second instrument, no bus that two sources can reach, and no concept anywhere in
`spectre-core`, `spectre-dsp`, `spectre-graph`, `spectre-project`, `spectre-audio`, or
`spectre-offline` called a track (verified: `grep -rn track crates --include='*.rs'`
returns hits only in `crates/spectre-app/`).

What exists in the app is a **presentation row**. `TrackView`
(`crates/spectre-app/src/lib.rs:37–45`) carries `muted`, `solo`, `armed`, and `level`
as plain public fields. `muted` and `solo` are written by the inspector's toggles
(`crates/spectre-app/src/main.rs:177–178`) and read by nothing. `level` drives one
`ProgressBar` in the Mix lens (`main.rs:469–473`). `armed` picks a `●`/`○` glyph in the
sidebar (`main.rs:131`). None of them touches DSP, because nothing in `spectre-app`
touches DSP at run time at all. The inspector even prints the string
`"MIDI clip  →  Native synth  →  Master"` (`main.rs:184`) beneath those toggles: a label
describing a signal path that does not exist. Under the vision's alpha bar — *"honest
telemetry, no fake surfaces"* — that is the exact defect R4-4 removes.

The architectural obstacle is concrete and is the reason this slice is not trivial.
`EditableGraph::connect` rejects a second connection into an already-fed input bus with
`GraphError::InputBusOccupied` (`crates/spectre-graph/src/lib.rs:171–180`), and
`compile` rejects any included input bus that is *not* fed with `GraphError::MissingInput`
(`lib.rs:227–240`). Two tracks therefore cannot both reach one destination today, at any
price. A track-to-master path requires an explicit summing node, and that node needs more
stereo input buses than `MAX_FLAT_INPUTS = 4` (`lib.rs:16`) permits — two.

### 1.3 Success signal

An offline render of a three-track project — two tracks unmuted, one muted — produces a
peak-nonzero result whose FNV-1a hash equals the hash of the same project with the muted
track deleted, and both differ from the hash of the same project with all three unmuted.
The comparison uses the existing FNV-1a walk in `crates/spectre-offline/src/lib.rs:271–278`
(offset basis `0xcbf2_9ce4_8422_2325`, prime `0x0000_0100_0000_01b3`), not a new one.
Asserted by `cargo test -p spectre-offline --test track_render`.

---

## 2. User Stories

> As an electronic musician sketching a loop, I want to add a second track and hear it
> alongside the first, so that "arrangement" means more than one sound at a time.

> As that musician auditioning a variation, I want to mute a track and hear it drop out
> of the mix without losing my selection, my lens, or the transport position, so that
> comparing two ideas costs one toggle instead of a rebuild.

> As a musician who has soloed a track to check it, I want an unmistakable indication
> that other tracks are being silenced *because* of solo rather than because they are
> muted, so that I never leave a session confused about why something is inaudible.

> As a musician reordering tracks, I want each track to keep its identity — its name,
> level, mute, solo, and its devices — across the move, so that reordering is a
> presentation act and never a data-loss act (CORE-001).

> As a musician who has hit the alpha's track ceiling, I want the app to refuse the
> creation with a plain statement of the limit rather than silently doing nothing or
> creating a track that produces no sound, so that a limit is a fact I can plan around.

> As a keyboard-only or screen-reader user, I want every track control — name, level,
> mute, solo, reorder, delete — to be a focusable element with a text label whose state
> is legible without color, so that decision 17's beta accessibility bar stays reachable.

> As the maintainer reviewing R4-5 and R4-8, I want one builder that turns the track
> model into an editable graph, used by both the live engine and the offline harness, so
> that clip playback and bounce compare the same computation rather than two.

---

## 3. UX Specification

### 3.1 Screen / view inventory

R4-4 introduces **no new screen, modal, sheet, popover, or drawer**. It modifies three
existing regions of the single-window shell in `crates/spectre-app/src/main.rs` and adds
content to a fourth.

| Region | Navigation path | New or modified | Layout pattern |
|---|---|---|---|
| Track sidebar (`track_list()`, `main.rs:115–160`) | always visible, left side, resizable, 220 px default / 180 px minimum (`main.rs:118–119`) | **modified** — rows gain mute/solo/level state and reorder/delete actions; the `armed` glyph at `main.rs:131` is removed | left side panel, vertical list + inline composer |
| Inspector "CONTEXT" block (`inspector()`, `main.rs:172–186`) | always visible, right side | **modified** — the toggles now write the real model; the literal signal-path string at `main.rs:184` is replaced by a model-derived one | right side panel, form |
| Mix lens (`mix_surface()`, `main.rs:458–478`) | lens selector → Mix | **modified** — strips read the real track model and gain a master strip | central panel, horizontal strip row |
| Transport bar (`transport()`, `main.rs:50–84`) | always visible | **modified, one label only** — a stale-plan notice, §3.6 E5 | full-width top panel |

Nothing is added to Arrange, Build, or Shape. Build and Shape continue to present the
existing flat device list (`AppModel::devices`, `crates/spectre-app/src/lib.rs:313–315`);
**re-parenting the device browser onto tracks is not this slice's work** — it is R6
(`docs/06-plans/rebuild-roadmap.md:31`, "Track types, groups, sends, returns, monitoring,
compensation, meters").

### 3.2 Interaction flows

**Primary flow — add a track and hear it.**

1. In the track sidebar the user types a name into the existing composer field
   (`main.rs:141–145`) and activates `+`.
2. `AppModel::add_track` validates the name, allocates a stable `ObjectId` from the
   project `IdGen`, appends a `Track` at the end of the list, and selects it. Name
   validation is the behavior that exists today (`lib.rs:407–410`): a blank or
   whitespace-only name is refused without mutation.
3. Because the track list changed shape, the compiled plan the engine is running no
   longer matches the model. `TrackList::structure_revision` advances (§4.2), the
   transport bar shows `Track layout changed — rebuild engine to hear it` with a
   `Rebuild engine` action, and **audio continues unchanged until the user rebuilds**.
   This is stated plainly rather than hidden: R4-4 does not hot-swap a live plan. §4.4
   explains why, and §8 Q3 carries the follow-on.
4. The user activates `Rebuild engine`. The app rebuilds the editable graph from the
   model, compiles it, stops the stream, installs the new bridge, and restarts. There is
   an audible gap across the restart, and the copy says so.
5. On the next block the new track's instrument is in the plan, summed with the others
   into the master.

**Flow — mute, solo, level.** These are **not** structural. Setting a track's level,
mute, or solo publishes the track's effective linear gain (§4.3) on the RT-002
latest-wins parameter lane; the plan is not rebuilt and the stream is not touched. Until
R4-2 lands decision 22's runtime parameter seam, the bridge counts the change as
`parameters_pending` (`crates/spectre-audio/src/bridge.rs:181–186`) and it takes effect at
the next engine build. **This spec does not claim mute is audible in R4-4.** The UI
therefore shows the same "applies at next rebuild" reason R4-1 specified for Shape edits,
and the reason disappears when R4-2 ships.

**Flow — solo.** Solo is additive: any number of tracks may be soloed at once. While at
least one track is soloed, every non-soloed track's effective gain is zero. The sidebar
shows soloed tracks as `SOLO` and silenced-by-solo tracks as `silenced by solo` — a
distinct state from `muted`, because conflating them is the confusion story in §2.

**Flow — reorder.** The user activates `Move up` / `Move down` on a selected track row.
The track's `ObjectId`, name, level, mute, solo, and instrument travel with it
(CORE-001). Reorder is structural: it changes the summing bus order, so it advances
`structure_revision` and shows the same rebuild notice. Reorder changes the order in
which the summing bus accumulates, which is not a bit-identical no-op — §4.3 states the
arithmetic consequence rather than pretending float addition is associative.

**Flow — delete.** `Delete track` removes the track and its devices from the model and
selects the nearest surviving track, or none if the list is empty. An empty track list is
a legal state that compiles to a valid plan producing exact silence (§4.3).

**Sound, haptic, animation cues.** No new animation, no haptics, no UI sound. Two audible
consequences are introduced and neither may be described as a feature: the engine rebuild
in step 4 produces a gap, and once R4-2 lands, a mute toggle is a 1 → 0 step on a `Gain`
that **does not smooth** (§7.1, §8 Q6) and will click.

### 3.3 Layout descriptions

**Track sidebar, top → bottom:**

1. `TRACKS` section label — unchanged (`main.rs:126`).
2. One row per track, in model order. Each row, leading → trailing:
   - selection affordance and track name (existing `selectable_label`, `main.rs:132`);
   - a state word, exactly one of `` (empty), `MUTED`, `SOLO`, or `silenced by solo`;
   - a numeric level readout, two decimals, monospace.
   The `●`/`○` armed glyph at `main.rs:131` is **removed**. Arming has no meaning before
   recording, which is R7 (`rebuild-roadmap.md:32`), and a control that has never done
   anything is the fake surface the alpha bar prohibits. Data source: `AppModel::tracks()`
   returning `&[spectre_project::Track]`.
3. Row action cluster for the selected track only: `Move up`, `Move down`, `Delete track`.
   Placing them on the selected row rather than on every row keeps the list calm
   (`vision.md`, "no spreadsheet density").
4. Track composer — unchanged text field and `+` button (`main.rs:140–152`).
5. `BROWSER` block — unchanged, still disabled (`main.rs:154–158`).

**Empty state.** With no tracks, the list area reads
`No tracks. Name one and press + to start.` and the Mix lens shows only the master strip.

**Ceiling state.** At `MAX_TRACKS` the composer's `+` is disabled with the hover reason
`Track limit reached (16 in the alpha). Delete a track to add another.` The number is
rendered from the constant, never typed into the copy.

**Inspector CONTEXT block.** The `Mute` / `Solo` toggles stay; `Arm` is removed for the
reason above. The `Level` slider's range changes from `0.0..=1.0` (`main.rs:181`) to the
range of the accepted gain descriptor, `GAIN_PARAMETERS[0].minimum()..=maximum()`
(`crates/spectre-dsp/src/effect.rs:19–20`, `0.0..=2.0`), and its label carries the
descriptor's unit exactly as Shape already does (`main.rs:369`). UI code does not
redefine a DSP range (`dsp-device-io.md:60`). The hard-coded string at `main.rs:184` is
replaced by a model-derived line: `<instrument> → Track gain → Master`, and when the
track is silenced the line appends `(silent: muted)` or `(silent: solo elsewhere)`.

**Mix lens.** One strip per track in model order plus a trailing `MASTER` strip. Each
track strip: name, level readout, mute/solo state word, and the existing `ProgressBar`
driven by `Track::level()` normalized through `GAIN_PARAMETERS[0].to_normalized`
(`crates/spectre-dsp/src/parameter.rs:53–56`) so the bar and the slider agree. The
`"Master route"` literal at `main.rs:474` is replaced by the destination read from the
model. **No meters.** Peak/RMS metering is `OBS-AB12-MIX-001` behavior and is R6; a bar
driven by a fader position is a fader position and is labeled as one.

### 3.4 Input & gestures

- Track selection, mute, solo, level, reorder, delete: pointer click on standard egui
  widgets, plus egui's built-in focus traversal and Enter/Space activation.
- **No keyboard shortcut is assigned to any of these commands, and none is proposed.**
  `docs/02-reference-research/workflow-field-study/product-implications.md`
  §"Prohibited conclusions at current evidence level" (lines 90–102) lists a default
  shortcut map among the conclusions the corpus cannot support. What this spec *does*
  commit to is the shape the vision already accepts: every track command is a named,
  context-scoped, remappable command with a stable identifier, so that when the command
  system lands the bindings are configuration rather than a rewrite. The prototype's
  existing `Space` and `1`–`4` handlers (`main.rs:425–437`) are pre-existing scaffolding;
  this spec neither extends, ratifies, nor documents them.
- Reorder is **not** drag-only. `Move up` / `Move down` are ordinary buttons, which keeps
  reorder reachable by keyboard from day one (decision 17). A drag affordance may be
  added later as an accelerator, never as the only path.
- Specialized input (stylus, controller, voice, camera): N/A — R4-4 adds no such surface.
- Responsive behavior: the sidebar is resizable with a 180 px minimum (`main.rs:119`) on
  a window with a 1060 px minimum width (`main.rs:508`). At the minimum, the track name
  truncates; the state word and the level readout do not.

### 3.5 Transitions & animation

- Navigation transitions: none. No view is added or removed.
- In-view state change: rows appear, disappear, and change position instantly on the
  existing 250 ms repaint cadence (`main.rs:443`). No reorder animation, no fade, no
  easing, and no new timing constant.
- Reduced motion: R4-4 introduces no animation, so a reduced-motion setting has no effect
  on this feature. That is the complete answer, not a deferral.

### 3.6 Error states

Presentation for per-action failures reuses the existing `feedback_status` line in the
inspector (`main.rs:210`), the channel `set_device_parameter_from_ui` already uses
(`crates/spectre-app/src/lib.rs:462–474`). Presentation for the persistent plan-staleness
condition is inline in the transport bar, because it is a condition rather than an event
and the transport bar is the only region visible from all four lenses.

| # | Trigger | Presentation | Recovery path | Data loss |
|---|---|---|---|---|
| E1 | Blank or whitespace-only track name | inspector line: `Track name must contain a visible character.` The composer text is retained. | retype and press `+` again | no |
| E2 | `TrackError::TrackLimit { limit }` — the list is at `MAX_TRACKS` | `+` is already disabled with the ceiling hover reason; if the command is reached another way the inspector line reads `Track limit reached (<limit>). Delete a track to add another.` | delete a track | no |
| E3 | `TrackError::UnknownTrack(id)` — a command names a track that no longer exists (stale UI id after a delete) | inspector line: `That track no longer exists. Select a track and try again.` **No model mutation occurs**, following the atomic-rollback pattern `open_device_in_shape` already uses (`lib.rs:338–345`) | select an existing track | no |
| E4 | `TrackError::IndexOutOfRange` — reorder past either end | the button is disabled at the ends; if reached, the inspector line states the position is already first/last. No mutation. | none needed | no |
| E5 | Track layout changed while the engine holds an older plan | transport bar: `Track layout changed — rebuild engine to hear it`, with a `Rebuild engine` action. Persistent until rebuilt. | press `Rebuild engine`; expect an audible gap | no |
| E6 | `RoutingError::TooManyTracks { count, limit }` at build time (the model is valid but exceeds what the graph can flatten — reachable only if `MAX_TRACKS` and the graph bound disagree) | transport bar: `Engine unavailable — plan build failed: <Display>`. This is a defect, and the message says so rather than implying user error. | none in-app | no |
| E7 | `GraphError` from `EditableGraph::compile` while building the track plan | as E6, carrying the `GraphError` `Display` (`crates/spectre-graph/src/lib.rs:64–108`) | none in-app | no |
| E8 | Every track is muted, or soloed tracks are all also muted | **not an error.** The master renders exact silence and the Mix lens master strip reads `silent: all tracks muted`. | unmute a track | no |

Two rules bind every row. **No error is reported from the audio thread**: every string
above is formatted on the app thread from an app-thread call or a value the render side
published into an atomic (RT-001). **No threshold-derived alarm state is specified** —
there is no "too many tracks for your CPU" warning, because Spectre owns exactly one
callback-headroom measurement (macOS, 2026-08-09, worst headroom 0.990 on a three-node
chain, `docs/06-plans/current-milestone.md:113`) and one data point cannot justify a
threshold (decision 16, PROD-003).

### 3.7 Accessibility

- Every new control is a standard egui button, toggle, slider, or text label. There is no
  icon-only control, no custom-painted widget, and **no color-only state**: mute, solo,
  and solo-silencing are carried by the words `MUTED`, `SOLO`, and `silenced by solo`, so
  a monochrome or color-blind reading loses nothing. Removing the `●`/`○` armed glyph
  (`main.rs:131`) also removes the one glyph-only state the sidebar had.
- Screen reader labels, hints, traits: `crates/spectre-app/Cargo.toml` builds eframe with
  `default-features = false` and only `default_fonts` and `glow`, so **no accessibility
  feature is enabled in this workspace today** and R4-4 claims no screen-reader support.
  Decision 17 gates that at beta with a scoped audit at R4; R4-4's obligation is to not
  foreclose it, which it satisfies by using standard widgets with text content and by
  giving every track command a keyboard-reachable button rather than a drag gesture.
- Custom actions for complex interactions: reorder is the only compound interaction, and
  it is deliberately decomposed into two single-activation commands so that no custom
  action is required.
- Text scaling / dynamic type: the sidebar is resizable and the strip row is horizontal;
  at large egui zoom the track name truncates before any state word or action button is
  clipped. Manual check in §5.4.
- Focus order and keyboard navigability: rows are created top-to-bottom in model order, so
  egui's creation-order focus traversal matches reading order. The selected row's action
  cluster is created immediately after that row, so `Move up` / `Move down` /
  `Delete track` are reached directly after the track they act on.

---

## 4. Implementation Specification

### 4.1 Architecture placement

| Path | Change | Thread |
|---|---|---|
| `crates/spectre-project/src/track.rs` | **new** — `Track`, `TrackList`, `TrackInstrument`, `TrackError`, `MAX_TRACKS` | app thread only |
| `crates/spectre-project/src/routing.rs` | **new** — `build_track_graph`, `track_device_factory`, `TrackPathNodes`, `RoutingError` | app thread only |
| `crates/spectre-project/src/command.rs` | modified — five new `CommandKind` variants for undoable track edits | app thread only |
| `crates/spectre-project/src/lib.rs` | modified — `pub mod track; pub mod routing;` and re-exports | app thread only |
| `crates/spectre-project/Cargo.toml` | modified — add `spectre-dsp`, `spectre-graph` path deps | build |
| `crates/spectre-dsp/src/mix.rs` | **new** — `SumBus`, the summing device | **render thread** |
| `crates/spectre-dsp/src/lib.rs` | modified — `mod mix; pub use mix::SumBus;` | build |
| `crates/spectre-graph/src/lib.rs` | modified — `MAX_FLAT_INPUTS` raised, with the bound named and documented | app thread (compile) + render thread (fixed array width) |
| `crates/spectre-offline/src/lib.rs` | modified — `render_track_list` entrypoint reusing the existing FNV-1a walk | app thread |
| `crates/spectre-app/src/lib.rs` | modified — `AppModel` owns a `TrackList`; `TrackView` deleted | app thread only |
| `crates/spectre-app/src/main.rs` | modified — sidebar, inspector, Mix lens, rebuild notice | app thread only |
| `crates/spectre-app/src/engine.rs` | modified (**file created by R4-1, not yet implemented**) — a track-list plan builder beside R4-1's fixture builder | app thread only |

**The dependency direction stays acyclic.** Today `spectre-project` depends on
`spectre-core`, `serde`, and `serde_json` only (`crates/spectre-project/Cargo.toml`).
Adding `spectre-dsp` and `spectre-graph` gives `core ← dsp ← graph ← project ← {app,
offline}`; `spectre-graph` depends on `core` and `dsp` and on nothing else, and
`spectre-dsp` depends on `core` alone, so no cycle is created. This placement is what
answers R4-1 §8 Q3's unplaced question — both `spectre-app` and `spectre-offline` already
depend on `spectre-project`, so one builder can serve the live engine and the offline
harness without a dev-dependency cycle. Whether a dedicated `spectre-session` crate would
be a better home is §8 Q7.

**What is on a callback-reachable path, and what is not.** Exactly one new thing runs in
the callback: `SumBus::process`. Everything else in this slice — the track model, the
commands, the graph builder, compilation, parameter publication — is app-thread code that
may allocate, and none of it is reachable from `RenderBridge::render`
(`crates/spectre-audio/src/bridge.rs:162–208`), which calls only
`CompiledPlan::process` and its own private helpers.

`SumBus::process` is covered by the RT-001 guard that already exists:
`crates/spectre-graph/tests/plan_alloc.rs` installs a counting global allocator
(`plan_alloc.rs:22–39`) and asserts `CompiledPlan::process` allocates nothing
(`plan_alloc.rs:45`); because a `SumBus` node is an ordinary plan step, extending that
test's fixture to include one puts the new device under the existing guard rather than a
new one (§5.1 test 12). `crates/spectre-audio/tests/rt_guard.rs` covers the same plan
through a real backend callback. Note the structural lock scan in that file reads
`src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, and `src/null.rs` — **it does not scan
`spectre-dsp` or `spectre-graph` at all**, so it says nothing about `SumBus` and this spec
does not claim otherwise. The allocation guards are the evidence for the new device; the
lock scan is not.

### 4.2 Data model

All new types carry Jeff's header block in their files. Every field below is private with
accessors, matching the containment pattern `DeviceParameterSnapshot` already uses
(`crates/spectre-dsp/src/parameter.rs:80–126`).

```rust
// crates/spectre-project/src/track.rs
//
// Author: Jeff
// Date: 2026-08-15
// Description: Ordered project track model with stable identity and mixer state
// Notes: App-thread only, never callback-reachable. Levels are carried by the accepted
//   device descriptors rather than a new fader law, so no numeric range is invented here.

// Maximum tracks a v1 project may sum into the master bus.
// Rationale row in docs/01-requirements/requirements-ledger.md (PROD-003, decision 16).
// A bound is structurally required, not merely prudent: PlanStep carries its input
// channel map as a fixed [usize; MAX_FLAT_INPUTS] array (crates/spectre-graph/src/lib.rs:337)
// and CompiledPlan::process builds a fixed [&[f32]; MAX_FLAT_INPUTS] on the stack
// (lib.rs:497), so an unbounded track count would require a heap collection on the
// render path, which RT-001 forbids. The specific value is Spectre's own cost
// arithmetic, not a reference product's: see §4.7 and §8 Q1.
pub const MAX_TRACKS: usize = 16;

// The one instrument kind a v1 track may host. R4-6 replaces the variant; the slot stays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackInstrument {
    Pulse,
}

// App-thread track failure; every variant leaves the model unmutated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackError {
    BlankName,
    TrackLimit { limit: usize },
    UnknownTrack(ObjectId),
    DuplicateId(ObjectId),
    IndexOutOfRange { index: usize, len: usize },
}

// One project track: identity, name, one instrument slot, and mixer state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track { /* private */ }

impl Track {
    // Create a validated track; the caller owns ID allocation from the project IdGen
    pub fn new(id: ObjectId, name: &str, instrument: TrackInstrument) -> Result<Self, TrackError>;
    pub fn id(&self) -> ObjectId;
    pub fn name(&self) -> &str;
    pub fn instrument(&self) -> TrackInstrument;
    // Instrument level, clamped by PULSE_PARAMETERS[0] (crates/spectre-dsp/src/source.rs:38-45)
    pub fn instrument_level(&self) -> f32;
    // Track fader position, clamped by GAIN_PARAMETERS[0] (crates/spectre-dsp/src/effect.rs:19-20)
    pub fn level(&self) -> f32;
    pub fn is_muted(&self) -> bool;
    pub fn is_soloed(&self) -> bool;
    pub fn set_name(&mut self, name: &str) -> Result<(), TrackError>;
    pub fn set_level(&mut self, level: f32);
    pub fn set_instrument_level(&mut self, level: f32);
    pub fn set_muted(&mut self, muted: bool);
    pub fn set_soloed(&mut self, soloed: bool);
}

// Ordered track collection plus the master bus level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackList { /* private */ }

impl TrackList {
    // Empty list; master level is GAIN_PARAMETERS[0].default() — no new constant
    pub fn new() -> Self;
    pub fn tracks(&self) -> &[Track];
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn master_level(&self) -> f32;
    pub fn set_master_level(&mut self, level: f32);
    pub fn index_of(&self, id: ObjectId) -> Option<usize>;
    pub fn get(&self, id: ObjectId) -> Option<&Track>;
    pub fn get_mut(&mut self, id: ObjectId) -> Option<&mut Track>;
    // Append; refuses past MAX_TRACKS and refuses a duplicate ObjectId (CORE-001)
    pub fn push(&mut self, track: Track) -> Result<(), TrackError>;
    pub fn insert(&mut self, index: usize, track: Track) -> Result<(), TrackError>;
    pub fn remove(&mut self, id: ObjectId) -> Result<Track, TrackError>;
    // Move one track to an absolute index, preserving identity and every field
    pub fn reorder(&mut self, id: ObjectId, to_index: usize) -> Result<usize, TrackError>;
    pub fn any_soloed(&self) -> bool;
    // level x mute x solo, clamped through GAIN_PARAMETERS[0]; None for an unknown track
    pub fn effective_gain(&self, id: ObjectId) -> Option<f32>;
    // Advances on every edit that changes the compiled graph's shape: push, insert,
    // remove, reorder, and instrument change. Level, mute, solo, and rename do not
    // advance it, because they reach a live plan over the parameter lane instead.
    // Session-local: #[serde(skip)], so it is not persisted and R4-7 inherits nothing.
    pub fn structure_revision(&self) -> u64;
}
```

```rust
// crates/spectre-project/src/routing.rs

// One compiled gain node and the instance ID of its single automatable parameter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GainNode {
    pub node: NodeId,
    pub gain_parameter: ObjectId,
}

// One compiled instrument node and the instance ID of its level parameter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstrumentNode {
    pub node: NodeId,
    pub level_parameter: ObjectId,
}

// Node identities for one built track graph; index i corresponds to TrackList::tracks()[i]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackPathNodes {
    pub instruments: Vec<InstrumentNode>,
    pub track_gains: Vec<GainNode>,
    pub sum: NodeId,
    pub master: GainNode,
}

// App-thread routing failure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingError {
    TooManyTracks { count: usize, limit: usize },
    Graph(spectre_graph::GraphError),
    Device(&'static str),
}
```

```rust
// crates/spectre-dsp/src/mix.rs
//
// Author: Jeff
// Date: 2026-08-15
// Description: Stereo summing bus with a declared number of stereo input buses
// Notes: Accumulates in f64 per decision 6 and rounds once to f32 at store. Bus order is
//   track order and is part of the contract, because float addition is not associative.

// Maximum stereo input buses one summing device may declare.
// Rationale row in docs/01-requirements/requirements-ledger.md; equals MAX_TRACKS and is
// the same bound expressed in the DSP layer. Kept as a separate const so a layout error
// is caught at device construction with a device-level message, before graph validation.
pub const MAX_SUM_BUSES: usize = 16;

// Stereo summing bus; zero buses is legal and renders exact silence
#[derive(Debug, Clone)]
pub struct SumBus { /* private: buses: usize */ }

impl SumBus {
    // Reject a bus count beyond MAX_SUM_BUSES; zero is accepted
    pub fn new(buses: usize) -> Result<Self, &'static str>;
    pub fn buses(&self) -> usize;
}

impl AudioProcessor for SumBus { /* io(), process() */ }
```

**Database migrations: N/A for this slice.** R4-4 defines the types with `serde` derives
so R4-7 can persist them, but it does **not** add them to `ProjectDoc`
(`crates/spectre-project/src/lib.rs:29–38`) and does not touch `SCHEMA_VERSION`
(`lib.rs:13`). Persisting the track list, and the schema-version decision that comes with
it, are R4-7's. CORE-001's reorder evidence is therefore only half-discharged here: this
slice proves identity survives an in-memory reorder (§5.1 test 4); the ledger's
requirement of evidence "on the first persisted object collection"
(`docs/01-requirements/requirements-ledger.md:46`) closes at R4-7.

### 4.3 API contracts

**New — `crates/spectre-dsp/src/mix.rs`, the summing device.**

```rust
impl AudioProcessor for SumBus {
    fn io(&self) -> DeviceIo {
        DeviceIo {
            class: DeviceClass::Effect,
            audio_inputs: self.buses * 2,
            audio_outputs: 2,
            accepts_notes: false,
        }
    }

    fn process(
        &mut self,
        context: &ProcessContext<'_>,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
    ) -> Result<(), ProcessError>;
}
```

Behavior, stated exactly:

- `validate_buffers` runs first, exactly as `Gain` and `Saturator` do
  (`crates/spectre-dsp/src/effect.rs:61`, `:118`), so a layout or length mismatch is a
  recoverable `ProcessError` and never a panic. `accepts_notes: false` means
  `validate_buffers` also rejects any event delivered to this node
  (`crates/spectre-dsp/src/io.rs:194–196`).
- For each output channel `c ∈ {0,1}` and frame `f`: accumulate
  `inputs[bus * 2 + c][f]` over `bus ∈ 0..buses` into an `f64`, treating any non-finite
  input sample as `0.0` — the same boundary containment `Gain` applies at
  `effect.rs:65–69` and `Saturator` at `effect.rs:122–126`. Store the accumulator to
  `outputs[c][f]` with one `as f32` conversion.
- The `f64` accumulator is decision 6's documented case ("f32 buffers, f64 for
  accumulators/coefficients where numerically warranted, documented per module"). It
  buys one thing and this spec claims only that thing: the intermediate rounding of a
  16-term `f32` sum no longer compounds, because there is one rounding, at the store.
  **It does not make the sum order-independent** — `f64` addition is not associative
  either. Determinism comes from the order being fixed: bus index equals track index,
  because `compile` flattens input buses in bus order (`crates/spectre-graph/src/lib.rs:298–308`)
  and the builder connects track `i` to bus `i`. Test 8 pins it.
- Every output sample is written on every call, satisfying `dsp-device-io.md:27`.
- With `buses == 0`, every output sample is `0.0`. An empty project is exact silence by
  construction, not an error state — which converges with `OBS-PP-ARCH-001` (Phase Plant
  requires at least one output module to produce sound) and with the "unconnected input
  reads zero" convention in `OBS-VCV-VOLT-005`.
- **No limiter, no clip, no normalization.** Summing N tracks can exceed ±1.0 and R4-4
  lets it, which is the position `OBS-AB12-MIX-002` records for a 32-bit float engine:
  over-0 dB is tolerated internally and clipping matters at the physical output. Spectre's
  `f32` buffers occupy the analogous position. This is stated rather than implied, and
  §5.4's manual protocol warns about it.

**RT-003 containment for the new node.** `SumBus` contains non-finite input at its own
boundary as above; it is *not* the only guard. `CompiledPlan::process` runs
`contain_channel` on every node's output before that output can reach a downstream node
(`crates/spectre-graph/src/lib.rs:512–522`), so a `SumBus` that ever emitted a NaN or
infinity would be silenced whole and counted in `ContainmentStats::contaminated_nodes`
(`lib.rs:389–397`) before the master gain saw it. Denormals in the sum flush to signed
zero with sign preserved (`lib.rs:401–414`). Injection tests are named in §5.1 tests
10 and 11.

**Changed — `crates/spectre-graph/src/lib.rs`, the flattened-input bound.**

```rust
// V1 flattened-channel bounds derived from the accepted device layouts
const MAX_FLAT_INPUTS: usize = 32;   // was 4
const FLAT_OUTPUTS: usize = 2;       // unchanged
```

This is the one change R4-4 makes to an existing contract surface, and it is a widening
only: `add_node` still requires an even input-channel count, still requires exactly two
output channels, and still rejects anything above the bound as
`GraphError::InvalidLayout` (`lib.rs:138–143`). Every other graph rule is untouched —
one connection per input bus (`lib.rs:171–180`), every included input bus fed
(`lib.rs:227–240`), Kahn ordering with implicit-cycle rejection (`lib.rs:242–273`),
dedicated output buffers per node (`lib.rs:275–281`). No existing test asserts the old
value: `grep -rn "MAX_FLAT\|InvalidLayout" crates/*/tests/` returns nothing, and
`graph_plan.rs:85–130` exercises out-of-range buses against `Gain`, which has one input
bus regardless of the constant. §5.1 test 13 adds the coverage that is currently missing.

**New — `crates/spectre-project/src/routing.rs`, app-thread graph construction.**

```rust
// Build the editable graph for a track list. Allocates; app thread only.
// Node identities are allocated from `ids` in a fixed order — for each track in list
// order: instrument, then track gain — then the sum bus, then the master gain. That
// order is the contract: it is what makes a rebuild from the same list with the same
// seed produce the same node IDs, which is what test 7 asserts.
pub fn build_track_graph(
    tracks: &TrackList,
    ids: &mut IdGen,
) -> Result<(EditableGraph, TrackPathNodes), RoutingError>;

// Construct the processor for one node of a graph built by build_track_graph.
// Values are read from `tracks` at construction, so the returned plan carries the model's
// levels; live changes travel on the parameter lane instead (§4.4).
pub fn track_device_factory<'a>(
    tracks: &'a TrackList,
    nodes: &'a TrackPathNodes,
) -> impl FnMut(NodeId) -> Result<Box<dyn AudioProcessor>, &'static str> + 'a;

impl TrackPathNodes {
    // The instrument node for one track index, for RenderBridge's single note node
    pub fn note_node(&self, index: usize) -> Option<NodeId>;
    // (device ObjectId, parameter ObjectId) pairs, in track order then master.
    // Returned as plain IDs rather than spectre_audio::control::ParameterTarget so that
    // spectre-project never depends on the audio crate; spectre-app maps them.
    pub fn parameter_targets(&self) -> Vec<(ObjectId, ObjectId)>;
}
```

The graph `build_track_graph` produces, for a list of `n` tracks:

```text
track[0].instrument ──► track[0].gain ──► sum.bus 0 ┐
track[1].instrument ──► track[1].gain ──► sum.bus 1 ├─► sum ──► master gain  (output node)
      …                       …                  …  ┘
```

- Each instrument node declares `INSTRUMENT_IO` (`crates/spectre-dsp/src/source.rs:20–25`):
  zero audio inputs, one stereo output, note input accepted.
- Each track gain node declares `EFFECT_IO` (`crates/spectre-dsp/src/effect.rs:12–17`) and
  is constructed as `Gain::new(tracks.effective_gain(id))`. Mute and solo are folded into
  the gain rather than given their own node, so they are addressable by the existing
  latest-wins parameter lane the moment R4-2 lands.
- The sum node declares `SumBus::new(n)`.
- The master node declares `EFFECT_IO` and is `Gain::new(tracks.master_level())`.
- Compilation is the existing `EditableGraph::compile(master.node, max_frames, &mut factory)`
  (`crates/spectre-graph/src/lib.rs:196–201`). **No new compilation path exists.**

`effective_gain` is defined once, on `TrackList`, and used by the builder, the offline
harness, and the app:

```text
effective_gain(t) = clamp( t.level
                         * (if t.muted { 0.0 } else { 1.0 })
                         * (if any_soloed && !t.soloed { 0.0 } else { 1.0 }) )
```

clamped through `GAIN_PARAMETERS[0]` (`crates/spectre-dsp/src/effect.rs:19–20`,
range `0.0..=2.0`, default `1.0`). No new numeric range is introduced: the fader's range
*is* the accepted gain descriptor's range, and the "unity" position *is* that
descriptor's default. Note the consequence for the existing UI: the inspector slider's
current `0.0..=1.0` hard-coding (`crates/spectre-app/src/main.rs:181`) is replaced by the
descriptor's range, so the slider stops contradicting the DSP (`dsp-device-io.md:60`).

**New — `crates/spectre-offline/src/lib.rs`, deterministic track rendering.**

```rust
// Render a track list through the compiled plan and report the same FNV-1a hash shape
// render_plan already produces (crates/spectre-offline/src/lib.rs:270-278)
pub fn render_track_list(
    sample_rate: f64,
    frames: usize,
    tracks: &TrackList,
    seed: u64,
    note_track: Option<ObjectId>,
    events: &[NoteEvent],
) -> Result<RenderReport, String>;
```

It builds the graph with `build_track_graph`, compiles with the existing `compile`, drives
one quantum with `PlanNoteInput` addressed to the note track's instrument node, and hashes
`plan.last_output()` with the **existing** walk, not a new one. `RenderReport`
(`crates/spectre-offline/src/lib.rs:28–34`) is unchanged.

**Changed — `crates/spectre-app/src/engine.rs` (created by R4-1, not yet implemented).**

```rust
// Build the render-side halves from a track list rather than the fixed fixture snapshot.
// Sits beside R4-1's build_engine_parts; neither is a second render path — both produce
// a CompiledPlan executed by the one RenderBridge.
pub fn build_track_engine_parts(
    tracks: &TrackList,
    seed: u64,
    note_track: Option<ObjectId>,
    config: spectre_audio::StreamConfig,
) -> Result<EngineParts, EngineUnavailable>;
```

`EngineUnavailable` (R4-1 §4.2) gains one variant,
`Routing(spectre_project::RoutingError)`. `EngineParts` gains one field,
`parameter_targets: Vec<spectre_audio::control::ParameterTarget>`, built by mapping
`TrackPathNodes::parameter_targets()` into `ParameterTarget { device, parameter }`
(`crates/spectre-audio/src/control.rs:22–26`); the control channel is constructed over
that fixed set (`control.rs:195–213`), which is what makes a track's gain addressable
later.

**Changed — `crates/spectre-project/src/command.rs`, undoable track edits.** `CommandKind`
today has exactly one variant, `SetProjectName` (`command.rs:30–33`). R4-4 adds five:
`AddTrack`, `RemoveTrack`, `RenameTrack`, `ReorderTrack`, `SetTrackMix`. Each `apply`
returns its own inverse, exactly as `SetProjectName` does (`command.rs:52–67`), so
`Transaction::execute`'s rollback (`command.rs:92–109`) and `EditHistory`'s bounded
undo/redo (`command.rs:113–182`) work unchanged. `RemoveTrack`'s inverse is
`insert(index, track)` carrying the removed `Track` whole, which is what preserves the
`ObjectId` across undo (CORE-001). **`ProjectCommand::apply` takes `&mut ProjectDoc`**
(`command.rs:52`), so wiring track commands through it requires the track list to live on
`ProjectDoc` — which is R4-7's schema change. R4-4 therefore lands the command variants
against a `TrackList` seam and states plainly that they are not reachable from
`EditHistory` until R4-7 puts the list on `ProjectDoc`; §8 Q8 asks whether to invert that
order.

Auth, permissions, pagination, rate limiting: **N/A — an in-process desktop feature with
no network and no multi-user surface.**

### 4.4 State management

| State | Owner | Thread | Lifetime |
|---|---|---|---|
| Track list, order, names, levels, mute, solo, master level | `TrackList` inside `AppModel` | app | process (persisted at R4-7) |
| `structure_revision` | `TrackList` | app | session only (`#[serde(skip)]`) |
| Selection (selected track, selected device), lens | `AppModel` | app | process |
| Compiled plan containing the track graph | `RenderBridge`, moved into the callback closure | render | until the stream drops |
| Effective per-track gain in flight | RT-002 latest-wins parameter lane | app writes, render drains | one block |
| Retired plan after a rebuild | reclaim lane (`RetiredState = Box<dyn Send>`, `control.rs:192`) | render hands over, app drops | per frame |
| The revision the running engine was built from | `SpectrePrototype` (the `eframe::App` implementor), beside R4-1's `Option<LiveEngine>` | app | until rebuild |

**The binding rule, and the one genuinely new correctness hazard.** There are now two
descriptions of the track set: the model, and the plan the render thread is executing.
They diverge the instant a structural edit lands, and the rule that keeps that honest is:

> **Mixer state travels; structure does not.** A change to a track's level, mute, solo, or
> the master level publishes `effective_gain` on the parameter lane and never rebuilds the
> plan. A change to the *set* or *order* of tracks, or to a track's instrument, advances
> `structure_revision`, and the app compares that revision against the one the engine was
> built from on every frame. While they differ, the transport bar states it (§3.6 E5) and
> **the app does not pretend the edit is audible**. A rebuild is an explicit user action.

Three consequences follow, and each is stated rather than assumed.

1. **The compiled plan is never mutated.** GRAPH-001 keeps `EditableGraph` app-thread and
   `CompiledPlan` immutable and execution-only — `CompiledPlan` "exposes no node or edge
   mutation API by design" (`crates/spectre-graph/src/lib.rs:416–417`). R4-4 adds no
   mutator and needs none: structural change produces a *new* plan on the app thread.
2. **Recompilation is never proposed per parameter change.** Decision 22 rejects that
   option on its face because compilation allocates
   (`docs/01-requirements/decision-gates.md`, row 22), and criterion 1E restates it.
   Level, mute, and solo are parameter changes and go down the parameter lane. Only
   add/remove/reorder/instrument-change recompile, and those are structural edits, not
   knob turns.
3. **The rebuild is a stream restart in R4-4, not a hot swap.** The app builds and compiles
   the new plan on its own thread, then stops the stream, installs a new `RenderBridge`
   over the new plan, and starts again. This is the conservative option and it costs an
   audible gap. A glitch-free hot swap is feasible with the pieces that already exist —
   the app boxes the new plan, a bounded lane delivers `Box<CompiledPlan>`, the render
   thread swaps by `std::mem::replace` and hands the old box back on the existing reclaim
   lane as `RetiredState = Box<dyn Send>` (`control.rs:192, 314–324`), with no allocation
   or destructor on the render thread because the coercion is a pointer-metadata change —
   but it needs a fourth lane in `spectre_audio::control` and a decision about what happens
   to notes in flight across the swap. **R4-4 does not implement it and does not claim it.**
   §8 Q3.

**One instrument receives notes.** `RenderBridge` holds exactly one `note_node`
(`crates/spectre-audio/src/bridge.rs:122–123, 133–139`) and builds exactly one
`PlanNoteInput` from it (`bridge.rs:188–191`). R4-4 does not change that. The note node is
the instrument of the track named by `note_track`, and every other track's instrument
receives no events and renders exact silence. **Per-track note routing is R4-5's work**
and this spec adds no partial version of it.

Local vs. server-synced state: **N/A — Spectre has no server; cloud services are a vision
non-goal.**

Offline / draft persistence: **N/A for this slice.** The types are serde-ready and nothing
is written to disk. R4-7 owns the schema change, the version decision, and the atomic-save
path.

### 4.5 Dependencies

- **New third-party crates: none.** `spectre-project` gains `spectre-dsp` and
  `spectre-graph`, both existing workspace members with no external dependencies of their
  own beyond `serde` (transitively, via `spectre-core`).
- **New assets or resources: none.** No fonts, images, samples, presets, or data files.
- **Infrastructure changes: none.** No database, CDN, or third-party service exists
  anywhere in this workspace.
- **New features/flags: none.** R4-4 adds no cargo feature.

### 4.6 Platform-specific considerations

- **Decision 1 makes macOS and Linux co-first-class, and R4-4 is platform-neutral by
  construction.** Nothing in this slice names a platform, a driver, an OS API, or a
  backend. The track model, the summing device, the graph builder, and the offline
  entrypoint are portable Rust over `f32`/`f64` arithmetic and workspace types.
- **The Linux consequence is real anyway, and it is a widening of decision 23's debt.**
  R4-4 makes the plan larger — up to 34 nodes at the ceiling versus today's three — and
  the only callback-headroom measurement Spectre owns is macOS, three nodes, 0.990 worst
  case (`docs/06-plans/current-milestone.md:113`). No Linux audio device has ever been
  opened. **This spec authorizes no Linux claim**, and it adds a specific one to R4-3's
  list: the Linux qualification run should be repeated against a multi-track project, not
  only the three-node fixture, or the Linux row will be qualified on a chain smaller than
  the alpha ships. §8 Q9.
- **Renderer/engine concerns:** egui/eframe is pinned at `0.32.3`
  (`crates/spectre-app/Cargo.toml`) under decision 8. R4-4 adds no custom-painted widget
  and no new egui capability; the sidebar rows, toggles, sliders, and buttons are all
  widgets `main.rs` already uses.
- **Version compatibility:** no OS version, driver version, or browser is involved.
- **Feature flags / gradual rollout:** none. The track model replaces `TrackView` outright
  rather than shipping beside it, because two track types in one binary is exactly the
  forked state criterion 2B fails.

### 4.7 Performance budget

Every figure is computed from source, not estimated from another product.

- **Memory — the plan channel pool.** `compile` allocates
  `channel_count × max_frames` `f32` (`crates/spectre-graph/src/lib.rs:275–281, 324`),
  where `channel_count` is two per included node. A project of `n` tracks compiles
  `2n + 2` nodes (instrument + gain per track, plus sum and master), so the pool is
  `(4n + 4) × max_frames × 4 B`. At `max_frames = 512`: one track is 16,384 B; 16 tracks
  is 139,264 B (≈136 KB). This is the dominant new allocation and it happens once, on the
  app thread, at build.
- **Memory — the per-step input map.** `PlanStep` carries `[usize; MAX_FLAT_INPUTS]`
  (`lib.rs:337`). Raising the constant from 4 to 32 grows `PlanStep` from ~56 B to ~288 B.
  At 34 steps that is ~9.8 KB of `steps`, allocated once at compile. `CompiledPlan::process`
  also builds a `[&[f32]; MAX_FLAT_INPUTS]` on the stack per step (`lib.rs:497`), growing
  from 64 B to 512 B — a stack frame, not an allocation. **This arithmetic is the reason
  the ceiling is 16 rather than unbounded, and the reason it is not, say, 512:** at 512
  tracks the per-step array alone would be 8 KB and the stack array 8 KB per step, which
  is a different design (a heap-backed input map, which the render path cannot have) rather
  than a bigger number. The choice between 16 and, say, 64 is a cost/evidence trade with no
  Spectre measurement behind it, and it is Jeff's — §8 Q1.
- **CPU / render time.** `SumBus::process` is `2 × frames × buses` `f64` adds plus
  `2 × frames` stores; at 16 tracks and 256 frames that is 8,192 adds per block. The larger
  cost is structural: 34 plan steps instead of 3, each running a full device. **No budget
  threshold is asserted**, because Spectre has exactly one headroom measurement and it was
  taken on a three-node chain (decision 16, PROD-003). Producing the multi-track
  measurement is named as R4-9 work in §7.4.
- **UI cost.** One pass over `TrackList::tracks()` per panel per frame at the existing
  250 ms repaint cadence (`main.rs:443`), bounded by `MAX_TRACKS`. `effective_gain` is
  three multiplies and a clamp. Negligible.
- **Network payload:** N/A — no network I/O exists in this feature or this workspace.
- **Storage:** N/A for this slice — nothing is written to disk. R4-7 owns the on-disk cost
  of the persisted track list.
- **Startup time:** unchanged. The default project starts with the single track the
  prototype already creates, and no plan is built until an engine starts.

---

## 5. Test Specification

The workspace gate is exactly:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

Feature-scoped commands, each naming a real target:

```sh
cargo test -p spectre-project --test track_model
cargo test -p spectre-project --test track_routing
cargo test -p spectre-dsp --test devices
cargo test -p spectre-graph --test containment
cargo test -p spectre-graph --test graph_plan
cargo test -p spectre-graph --test plan_alloc
cargo test -p spectre-offline --test track_render
cargo test -p spectre-app --test app_model
cargo test -p spectre-app --test smoke_cli
cargo test -p spectre-audio --test rt_guard
./spectre --smoke-test
```

`track_model.rs`, `track_routing.rs`, and `track_render.rs` are created by this slice
(§7.2); every other file already exists.

### 5.1 Unit tests

Each test names the constructors and accessors it needs, all of which are declared in §4.2
and §4.3.

**In `crates/spectre-project/tests/track_model.rs` (new):**

1. **`a_track_keeps_its_identity_and_fields_across_a_reorder`**
   Setup: `TrackList::new()`; three `Track::new(ids.next_id(), name, TrackInstrument::Pulse)?`
   pushed with `push`; set distinct `set_level`, `set_muted`, `set_soloed` on each.
   Action: `list.reorder(second_id, 0)?`. Assert `list.index_of(second_id) == Some(0)`, and
   that `list.get(second_id)` returns a track whose `id()`, `name()`, `level()`,
   `is_muted()`, `is_soloed()`, and `instrument()` all equal the pre-move values, and that
   the other two tracks are unchanged and in their original relative order. Edge case:
   CORE-001 identity across reorder — the half of that requirement this slice can
   discharge. Fails if reorder is implemented as remove-then-recreate.

2. **`the_track_ceiling_refuses_the_overflowing_track_without_mutating_the_list`**
   Push `MAX_TRACKS` tracks; assert the next `push` returns
   `Err(TrackError::TrackLimit { limit: MAX_TRACKS })` and that `list.len() == MAX_TRACKS`
   and `list.tracks()` is unchanged (compare a clone taken before). Edge case: E2, and the
   fail-closed posture. Fails if the bound is advisory.

3. **`blank_names_and_unknown_ids_are_refused_without_mutation`**
   Assert `Track::new(id, "   ", Pulse)` is `Err(TrackError::BlankName)`;
   `list.set_name` via `get_mut` with `"\t"` is `Err(BlankName)` and leaves `name()`
   unchanged; `list.remove(unrelated_id)` is `Err(TrackError::UnknownTrack(_))` with
   `len()` unchanged; `list.reorder(id, list.len() + 5)` is
   `Err(TrackError::IndexOutOfRange { .. })` with the order unchanged. Also assert
   `push` of a second track carrying an already-present `ObjectId` is
   `Err(TrackError::DuplicateId(_))` — the project-scoped uniqueness CORE-001 requires.
   Edge cases: E1, E3, E4.

4. **`effective_gain_composes_level_mute_and_solo_in_that_order`**
   Three tracks at levels `1.0`, `0.5`, `2.0`. Assert: with nothing muted or soloed,
   `effective_gain` equals each `level()`. Mute track 2 → its gain is `0.0`, others
   unchanged. Un-mute, solo track 3 → track 3 keeps `2.0`, tracks 1 and 2 are `0.0`,
   `any_soloed()` is `true`. Solo track 1 as well → both soloed tracks keep their levels
   (solo is additive). Mute track 3 while it is soloed → `0.0`, proving mute dominates
   solo. Edge case: the §3.2 solo semantics and the §2 confusion story. Fails on any
   ordering error in the composition.

5. **`structure_revision_advances_only_for_structural_edits`**
   Record `structure_revision()`; call `set_level`, `set_muted`, `set_soloed`,
   `set_master_level`, and `set_name` — assert the revision is unchanged after each.
   Then `push`, `reorder`, and `remove` — assert it strictly increases after each.
   Edge case: §4.4's binding rule. This is the assertion that fails if a mixer edit is
   mistakenly made structural (which would rebuild the plan per knob turn — the thing
   decision 22 rejects) or if a structural edit is mistakenly made silent (which would
   leave the engine running a plan that no longer matches the model).

**In `crates/spectre-project/tests/track_routing.rs` (new):**

6. **`the_built_graph_compiles_and_its_shape_matches_the_track_count`**
   Setup: a 3-track list; `let (graph, nodes) = build_track_graph(&list, &mut ids)?`;
   `let mut factory = track_device_factory(&list, &nodes);`
   `let plan = graph.compile(nodes.master.node, 256, &mut factory)?`.
   Assert `plan.step_count() == 2 * 3 + 2` (`CompiledPlan::step_count`,
   `crates/spectre-graph/src/lib.rs:445–447`), `nodes.instruments.len() == 3`,
   `nodes.track_gains.len() == 3`. Edge case: a builder that silently drops a track would
   still compile; the step count is what catches it.

7. **`rebuilding_from_the_same_list_and_seed_produces_the_same_node_ids`**
   Build twice from `IdGen::new(SEED)` and assert the two `TrackPathNodes` are equal
   (`PartialEq` is derived). Edge case: the fixed allocation order §4.3 declares. Fails if
   the builder iterates a `HashMap`.

8. **`track_order_is_bus_order`**
   Build from a 2-track list where track 0's instrument level is `0.0` (silent) and
   track 1's is nonzero; compile; render one quantum with notes addressed to
   `nodes.note_node(1).unwrap()`; capture `plan.last_output()`. Then reorder the list so
   the sounding track is index 0, rebuild, render with notes addressed to
   `nodes.note_node(0).unwrap()`, and assert the two outputs are sample-for-sample equal.
   Then, separately, build a list where **both** tracks sound at different levels, render,
   reorder, rebuild, render, and assert the outputs are equal **or**, if they differ, that
   they differ only within one f32 ULP — documenting the non-associativity §4.3 admits
   rather than asserting an equality the arithmetic does not guarantee. Edge case: the
   determinism claim in §4.3, stated at the precision it actually holds.

9. **`an_empty_track_list_compiles_to_a_plan_that_renders_exact_silence`**
   `build_track_graph(&TrackList::new(), &mut ids)` then compile and
   `plan.process(48_000.0, 64, &[])`. Assert `Ok(())`, and that every sample of
   `plan.last_output().unwrap()` is `0.0` with `to_bits()` equal to `0.0_f32.to_bits()` —
   bit equality, so a `-0.0` would be caught. Edge case: E8 and the zero-bus `SumBus`
   branch. Fails if the builder special-cases the empty list into an error.

**In `crates/spectre-dsp/tests/devices.rs` (existing file, additions):**

10. **`sum_bus_adds_its_buses_and_contains_non_finite_input`**
    Modeled on the existing `saturator_contains_non_finite_input_and_bounds_output`
    (`devices.rs:251–272`). Setup: `SumBus::new(3)?`; three stereo input pairs built with
    the file's existing `output(frames)` helper (`devices.rs:12–15`); one bus carrying
    `f32::NAN`, one carrying `f32::INFINITY`, one carrying finite values. Assert every
    output sample equals the finite bus's value exactly (the poisoned buses contributed
    `0.0`) and that every output sample `is_finite()`. Then assert `SumBus::new(0)?`
    writes all zeros, and `SumBus::new(MAX_SUM_BUSES + 1)` is `Err(_)`. Edge case: RT-003
    boundary containment on the new device, plus the bus-count bound.

11. **`sum_bus_layout_matches_the_declared_bus_count`**
    Extend the existing `device_layouts_match_the_v1_contract` (`devices.rs:186–210`):
    for `buses` in `[0, 1, 2, MAX_SUM_BUSES]`, assert
    `SumBus::new(buses)?.io() == DeviceIo { class: DeviceClass::Effect, audio_inputs: buses * 2, audio_outputs: 2, accepts_notes: false }`.
    Edge case: an `io()` that disagrees with the constructor would be caught at compile by
    `GraphError::IoMismatch` (`crates/spectre-graph/src/lib.rs:291–293`) only at run time;
    this catches it at unit level.

**In `crates/spectre-graph/tests/containment.rs` (existing file, addition):**

12. **`a_contaminated_track_is_silenced_before_it_reaches_the_sum`**
    Reuses the file's existing test-only `PoisonSource` (`containment.rs:44`) and its
    `Poison` helper (`containment.rs:31`). Build a two-bus graph: `PoisonSource` → bus 0,
    a clean `ToneSource` → bus 1, both into a `SumBus::new(2)`, into a master `Gain`.
    Render one quantum. Assert the master output equals the clean source's contribution
    exactly, that `plan.containment().contaminated_nodes` is `1`, and that
    `plan.containment().last_contaminated` names the poison node
    (`ContainmentStats`, `crates/spectre-graph/src/lib.rs:389–397`). Edge case: RT-003's
    node-isolation guarantee across a summing node — the case where one bad track could
    otherwise poison the whole mix. Fails if containment ran after the sum instead of
    before it.

**In `crates/spectre-graph/tests/graph_plan.rs` (existing file, addition):**

13. **`a_layout_beyond_the_flat_input_bound_is_refused`**
    Assert `graph.add_node(id, DeviceIo { class: DeviceClass::Effect, audio_inputs: (MAX_SUM_BUSES + 1) * 2, audio_outputs: 2, accepts_notes: false })`
    returns `Err(GraphError::InvalidLayout(_))`, and that a layout at exactly
    `MAX_SUM_BUSES * 2` inputs is accepted. Edge case: the currently **untested**
    `InvalidLayout` input-count branch (`crates/spectre-graph/src/lib.rs:138–143`). This
    test does not exist today and is the coverage the widening makes necessary.

**In `crates/spectre-graph/tests/plan_alloc.rs` (existing file, modification):**

14. **`plan_process_is_allocation_free`** (existing test, `plan_alloc.rs:45`) — its fixture
    is extended to a two-track graph ending in a `SumBus` and a master `Gain`, so the new
    device runs inside the existing counting-allocator guard (`plan_alloc.rs:22–39`)
    rather than beside it. Assert allocation and deallocation counts are both zero across
    the steady-state quanta the test already drives. Edge case: RT-001 for `SumBus`. This
    test fails if `SumBus::process` ever grows a temporary `Vec`, which is the single most
    likely way this slice could break RT-001.

**In `crates/spectre-app/tests/app_model.rs` (existing file, modifications):**

15. **`adding_and_selecting_tracks_uses_stable_unique_ids`** (existing,
    `app_model.rs:33–41`) and **`blank_track_names_are_rejected_without_mutation`**
    (existing, `app_model.rs:43–49`) are retained with their assertions intact; only the
    element type changes, from `TrackView` to `spectre_project::Track`, and
    `model.tracks().to_vec()` continues to compile because `Track` derives `Clone`.
    A third assertion is added to the first: after `add_track`, `model.tracks().last()`
    has `is_muted() == false`, `is_soloed() == false`, and
    `level() == GAIN_PARAMETERS[0].default()` — a new track starts audible at unity,
    which no test asserts today.

16. **`mix_edits_do_not_stale_the_plan_and_structural_edits_do`**
    Setup: `AppModel::prototype()`. Assert `model.track_structure_revision()` is unchanged
    after `set_track_level`, `set_track_muted`, `set_track_soloed`, and after a rename;
    and strictly increases after `add_track`, `reorder_track`, and `remove_track`. Edge
    case: the app-level projection of test 5 — this is what the transport bar's rebuild
    notice reads, so a wrong answer here is a visible lie.

### 5.2 Integration tests

- **`cargo test -p spectre-offline --test track_render`** (new file). Three assertions,
  each of which fails on a distinct regression, all hashing with the existing FNV-1a walk
  in `crates/spectre-offline/src/lib.rs:271–278` — **no new comparison method**:
  1. *Muting removes exactly that track's contribution.* Render a three-track list with
     track 1 muted; render the same list with track 1 deleted; assert the two
     `RenderReport::hash` values are equal and that `peak > 0.0` on both, so the match is
     not two silent buffers agreeing. This is §1.3's success signal.
  2. *Summing is not a no-op.* Render the same three-track list with nothing muted;
     assert its hash differs from (1) — a builder that silently summed only the note track
     would pass (1) and fail here.
  3. *Repeated renders are bit-identical.* Render the same list three times from the same
     seed; assert all three hashes are equal (1D determinism).
- **`cargo test -p spectre-graph --test containment`, `--test graph_plan`, `--test plan_alloc`**
  must all pass **after** the `MAX_FLAT_INPUTS` change, not merely before it. The widening
  touches a constant that both `PlanStep`'s array width and `process`'s stack array read,
  so running the graph suite post-change is the evidence that the widening was inert for
  existing behavior, and this spec treats it as a required gate rather than a formality.
- **`cargo test -p spectre-audio --test rt_guard`** must continue to pass unchanged. R4-4
  modifies no file that test scans (`src/bridge.rs`, `src/control.rs`, `src/spsc.rs`,
  `src/null.rs`), and its allocation guard over a plan driven through a real backend
  callback covers the new device once a track plan is what the bridge holds. Its structural
  lock scan does **not** cover `spectre-dsp` or `spectre-graph`, so it is not evidence
  about `SumBus` and is not cited as such.
- **`cargo test -p spectre-app --test smoke_cli`.** The smoke line's `tracks=` field
  (`crates/spectre-app/src/main.rs:487–496`) now reports `TrackList::len()`, and the
  existing assertion `tracks=1` (`smoke_cli.rs:18`) holds because the default project still
  starts with one track. Retaining that assertion unchanged is deliberate: it is the
  cheapest possible check that the model swap did not change observable startup state.

### 5.3 UI / E2E tests

**There is no automated GUI-driving harness in this repository** — `crates/spectre-app/tests/`
contains exactly `app_model.rs` and `smoke_cli.rs` — and R4-4 does not add one. Automated
UI scenarios are R4-9's scope. Two things limit what this slice can prove automatically,
and both are stated rather than hidden:

- The sidebar, inspector, and Mix lens live in `crates/spectre-app/src/main.rs`, a binary
  target whose types are unreachable from `crates/spectre-app/tests/`. Everything those
  surfaces render is derived from `AppModel` accessors, so tests 15 and 16 cover the data;
  **the rendering itself is covered only by §5.4's manual protocol.**
- The rebuild-notice behavior (§3.6 E5) is a comparison between two values held in
  `SpectrePrototype`, which is likewise in `main.rs`. Test 16 covers the model half. The
  comparison itself is a manual check until R4-9.

`./spectre --smoke-test` must continue to exit 0 on a machine with no audio device.

### 5.4 Visual / manual verification

Run `./spectre` **with system volume low**: summing several tracks has no limiter (§4.3),
and once R4-2 lands, a mute toggle is an unsmoothed gain step (§8 Q6).

| Configuration | What to check |
|---|---|
| Theme variants | **N/A — the shell hard-codes a single dark palette.** `main.rs:37` sets `dark_mode = true` and every color is a fixed `Color32` constant at `main.rs:9–15`. No light variant exists to check, and introducing one is not this slice's work. |
| Text size extremes | At large egui zoom, a long track name truncates inside the sidebar; the state word (`MUTED` / `SOLO` / `silenced by solo`), the level readout, and the three row action buttons stay fully visible and activatable. |
| Screen size extremes | At the 1060×680 minimum (`main.rs:508`) with the sidebar at its 180 px minimum (`main.rs:119`), and at the 1420×860 default: the Mix lens strip row scrolls or truncates rather than pushing the master strip out of view. |
| Empty vs. populated | Four states must each be produced and read: zero tracks (empty-state copy, master strip only, exact silence), one track, `MAX_TRACKS` tracks with the composer disabled and the ceiling reason visible, and a mixed state with one muted and one soloed track. |
| Color independence | With the display set to grayscale, every mute, solo, and solo-silenced row must still be distinguishable by its text. |
| Honesty check | Toggle mute while sound is playing: the sound must **not** change, and the UI must say why (R4-2). Add a track while sound is playing: the audio must not change and the transport bar must show the rebuild notice. Press `Rebuild engine`: expect an audible gap, then the new track in the mix. |
| Audio behavior | Two tracks with the same instrument level are audibly louder together than either alone; deleting a track removes exactly its contribution; muting every track produces exact silence, not a floor. |

---

## 6. Compliance & Safety Gate

### 6.1 Sensitive data classification

- [x] **No sensitive data involvement.** The only user-supplied data this feature accepts
  is a track name, held in memory and (from R4-7) written into the project file the user
  chose. Nothing is transmitted, logged to disk, or sent anywhere; there is no network path
  in any crate in this workspace.

### 6.2 Asset provenance

- [x] **No third-party assets.** No fonts, images, samples, wavetables, presets, or data
  files are added. No new crate dependency is added, third-party or otherwise. The summing
  device's arithmetic, the track model, the routing builder, and every name in this spec
  are original workspace work.

### 6.3 Language / claims audit

- [ ] Makes claims not supported by evidence — **no.** The benchmark claims in this spec are
  each carried by an `OBS-` ID listed in Appendix A, and the three surfaces where the corpus
  is silent (track-count limits, summing architecture, fader law) are named as gaps rather
  than filled in. The one performance figure quoted — 0.990 worst-case headroom — is
  Spectre's own macOS measurement, cited to `current-milestone.md:113`, and this spec states
  it was taken on a three-node chain rather than a multi-track one.
- [ ] Promises capabilities not yet built — **no.** The spec states in §1.2, §3.2, §4.4, and
  §7.1 that `./spectre` produces no sound today; that R4-1 is spec'd but not implemented;
  that mute, solo, and level will **not** be audible until R4-2 lands decision 22's seam;
  that adding a track requires an explicit engine rebuild with an audible gap; that only one
  instrument receives notes until R4-5; and that the persisted half of CORE-001's reorder
  evidence closes at R4-7, not here.
- [ ] Uses language restricted by domain regulations — **N/A.** No regulated domain is
  involved.

Two user-visible-copy rules are normative. **A track's state word must name the actual
cause**: `MUTED` only when `is_muted()`, `silenced by solo` only when `any_soloed() &&
!is_soloed()`, never one standing in for the other. And **the rebuild notice must be
present whenever the revisions differ** — an app that hides it is claiming an edit is
audible when it is not, which is the fake surface the alpha bar prohibits.

### 6.4 Regulatory alignment

Confirmation against `gauntlet-output/criteria.md` Lens 3:

- **3A milestone fit.** R4-4 is slice 4 of the accepted R4 queue (`docs/status/NEXT.md:26`)
  and the fourth leaf of the confirmed feature tree. The milestone's scope line reads "track
  to master" (`current-milestone.md:12`) and its exit evidence reads "A MIDI clip plays
  through a track into master" (`:83`). This slice delivers the track and the path; the clip
  is R4-5 and the bounce is R4-8. Everything beyond a track-to-master path is named and
  deferred, not smuggled: sends, returns, groups, monitoring, latency compensation, meters,
  and track types are R6 (`rebuild-roadmap.md:31`); recording and arming are R7 (`:32`);
  automation is R9 (`:34`); per-track timeline-vs-launcher authority is R10 (`:35`) and
  PROD-001.
- **3B non-goal respect.** No CLAP/LV2/AU hosting, no plugin-format authoring, no cross-DAW
  preset or project compatibility, no cloud service or content store, no video scoring. The
  `armed` flag is **removed** rather than left as an inert control, which is the opposite of
  implying recording exists.
- **3C deliberately small first devices.** R4-4 adds exactly one device, `SumBus`, and it is
  routing infrastructure rather than an instrument or an effect: it has zero parameters, no
  tone, no character, and no UI beyond the master strip. The existing `PulseInstrument`,
  `Gain`, and `Saturator` are reused unchanged; the track fader is the shipped `Gain`, not a
  new one. Nothing here grows toward the R11 flagship (decision 15).
- **3D originality.** No reference product's code, numbers, layout, or naming is
  transcribed. `SumBus`, `TrackList`, `TrackPathNodes`, and `effective_gain` are original
  names. The two numeric bounds carry Spectre-derived rationale rows (§4.2, §7.2) and are
  routed to Jeff in §8 Q1 precisely because a number without evidence should not be settled
  by an agent.
- **3E platform commitment.** The slice is platform-neutral by construction (§4.6) and it
  makes the undischarged Linux debt *worse* rather than pretending otherwise: it says so and
  adds a concrete requirement to R4-3's run in §8 Q9. No Linux claim is made.
- **3F accessibility trajectory.** No icon-only control, no color-only state, no custom
  widget, no drag-only interaction; the glyph-only `armed` indicator is removed. The known
  eframe accessibility-feature gap is recorded (§3.7) for decision 17's R4 audit rather than
  hidden.

---

## 7. Gap Analysis vs. Current State

### 7.1 What exists today

Verified by reading the files at commit `2e005e5`. Each row is checkable in one `Read`.
The manifest's run-history entry for 2026-08-12 states that the discarded spec claimed
`AppModel::add_track()`, `Track::arm()`, `Track::mute()`, and a track list sidebar "all
described as partially existing. None exist." **That summary is partly wrong today and
this spec does not repeat it.** `AppModel::add_track` and a track list sidebar do exist;
`Track::arm` and `Track::mute` never have, because there is no `Track` type. What is
genuinely absent is the thing the phrase "track model" means: any connection between those
rows and the audio path. The partition below states each claim at the precision the source
supports.

**Implemented — the app's track *presentation*, wired to nothing:**

| Element | Path | What it actually is |
|---|---|---|
| `TrackView` | `crates/spectre-app/src/lib.rs:37–45` | A struct with six **public** fields: `id: ObjectId`, `name: String`, `muted`, `solo`, `armed`, `level: f32`. **No methods, no invariants, no validation, no `Track` type anywhere in the workspace.** |
| `AppModel::add_track` | `crates/spectre-app/src/lib.rs:405–422` | Exists. Trims the name, rejects blank, allocates an `ObjectId` from the model's `IdGen`, pushes a `TrackView` with `muted/solo/armed = false` and a hard-coded `level: 0.72`, and selects it. **No bound on the track count. No graph, plan, or device consequence of any kind.** |
| `AppModel::tracks / selected_track_id / select_track / selected_track / selected_track_mut` | `crates/spectre-app/src/lib.rs:289–311` | Ordinary list and selection accessors over `Vec<TrackView>`. |
| Default project state | `crates/spectre-app/src/lib.rs:218–262` | Exactly one track, named `"Pulse"`, `level: 0.78`, selected. |
| Track list sidebar | `crates/spectre-app/src/main.rs:115–160` | Exists. Renders one `selectable_label` per track prefixed by a `●`/`○` glyph chosen from `track.armed` (`:131`), plus a name field and `+` button calling `add_track` (`:140–152`). |
| Inspector mute/solo/arm/level | `crates/spectre-app/src/main.rs:174–186` | Three `toggle_value` calls writing `track.muted`, `track.solo`, `track.armed` directly (`:177–179`) and a `Slider` over `track.level` hard-coded to `0.0..=1.0` (`:181`). Beneath them, the **literal string** `"MIDI clip  →  Native synth  →  Master"` (`:184`) and a permanently disabled `+ Add device` button (`:185`). |
| Mix lens strips | `crates/spectre-app/src/main.rs:458–478` | One frame per track showing the name, a `ProgressBar` driven by `track.level`, and the literal `"Master route"` (`:474`). |
| Smoke line and its assertion | `crates/spectre-app/src/main.rs:487–496`; `crates/spectre-app/tests/smoke_cli.rs:18` | `tracks={}` prints `model.tracks().len()`; the test asserts the literal `tracks=1`. |
| Track tests | `crates/spectre-app/tests/app_model.rs:33–49` | Two tests: unique IDs across `add_track`, and blank-name rejection without mutation. |

**The load-bearing fact about all of the above:** `muted` and `solo` are **written and never
read** — `grep -rn "\.muted\|\.solo\b" crates` finds the two writes at `main.rs:177–178`
and no reader. `armed` is read once, for a glyph (`main.rs:131`). `level` is read once, for
a progress bar (`main.rs:469–473`). None of them reaches `spectre-dsp`, `spectre-graph`, or
`spectre-audio`, because `crates/spectre-app` has no run-time path to any of them: its
`[dependencies]` are `eframe`, `spectre-core`, `spectre-dsp`, `spectre-project`, `serde`,
`serde_json` — **no `spectre-graph`, no `spectre-audio`**.

**Implemented — the graph and DSP this slice builds on:**

- The GRAPH-001 split: `EditableGraph` (app-thread, `crates/spectre-graph/src/lib.rs:119–125`)
  and `CompiledPlan` (immutable, execution-only, "exposes no node or edge mutation API by
  design", `lib.rs:416–427`). Validated compilation with implicit-cycle rejection
  (`lib.rs:242–273`), a preallocated channel pool (`lib.rs:275–281, 324`), and RT-003
  containment inside `process` (`lib.rs:512–522`).
- **The three graph rules that make a summing device mandatory:** exactly one connection per
  input bus, `GraphError::InputBusOccupied` (`lib.rs:171–180`); every included input bus must
  be fed, `GraphError::MissingInput` (`lib.rs:227–240`); at most `MAX_FLAT_INPUTS = 4`
  flattened input channels — two stereo buses — per node (`lib.rs:16, 138–143`).
- Four native devices: `ToneSource` and `PulseInstrument`
  (`crates/spectre-dsp/src/source.rs:14–25` for their layouts), `Gain` and `Saturator`
  (`crates/spectre-dsp/src/effect.rs:12–17`). `AudioProcessor: Send` at
  `crates/spectre-dsp/src/io.rs:163`.
- The offline harness and its FNV-1a hash walk
  (`crates/spectre-offline/src/lib.rs:204–285`, hash at `:271–278`).
- `spectre-audio`'s backend seam, RT-002 split-lane transport (`control.rs`), callback bridge
  (`bridge.rs`), MIDI ingress, and telemetry — all tested, **and nothing in `./spectre` uses
  any of it.**

**Prototyped — the app shell.** `crates/spectre-app/src/main.rs`'s own header calls it an
"interaction prototype" and states "audio and persistence wiring remain out of scope."

**Planned — spec'd, not implemented:**

- **R4-1 `live-audio-wiring`.** `gauntlet-output/specs/R4-1-live-audio-wiring.md` passed
  blind review at 2.950 (`gauntlet-output/manifest.md:44`), and the manifest's own
  known-gaps line records "R4-1 has passed its spec gate but **no implementation exists**"
  (`manifest.md:19`). `crates/spectre-app/src/engine.rs` does not exist; the crate is two
  files, `lib.rs` and `main.rs`. **`./spectre` produces no sound of any kind today**:
  `AppModel::toggle_play` (`lib.rs:268–275`) mutates an in-memory `Transport` and returns,
  and the transport bar prints the hard-coded string `ENGINE OFFLINE` (`main.rs:79`).
- **R4-2 `runtime-parameter-seam`** and **R4-7 `project-persistence`** are in
  `spec-in-progress` (`manifest.md:45, 50`); neither has a spec file or an implementation.

**Gated — accepted, deliberately not implemented:**

- **Decision 22, runtime parameter seam.** Design accepted in
  `docs/03-architecture/dsp-device-io.md` §"Runtime parameter seam" (lines 81–96);
  implementation is R4-2. Today the bridge drains the parameter lane into a closure that
  discards the value and counts it as `parameters_pending`
  (`crates/spectre-audio/src/bridge.rs:178–186`). **Consequence for this spec: after R4-4
  ships, a mute, solo, or level change does not alter live audio.**
- **Decision 23, Linux device qualification.** The milestone's table row reads "not run"
  (`docs/06-plans/current-milestone.md:114`).
- **Decision 13, undo architecture** — SD adopted, GATE at R5 exit. The command
  infrastructure exists (`crates/spectre-project/src/command.rs`) with exactly one command
  kind, `SetProjectName` (`command.rs:30–33`).
- **Decision 17, accessibility** — scoped audit at R4, gated before beta.
- **GRAPH-002** — only implicit-cycle rejection exists; explicit priced feedback edges are
  gated at decision row 7 before R11 (`requirements-ledger.md:56`).
- **CORE-001 reorder evidence** — explicitly gated on the first persisted object collection
  (`requirements-ledger.md:46`), which R4-7 creates.

**Absent — the gap R4-4 closes:**

- **No `Track` type, and therefore no `Track::arm()` or `Track::mute()`, anywhere in the
  workspace.** `grep -rn "struct Track\b\|fn arm\|fn mute" crates` returns nothing.
  `TrackView`'s booleans are plain public fields mutated directly by egui toggles.
- **No track concept outside `crates/spectre-app/`.** `grep -rn track crates --include='*.rs'`
  hits only `spectre-app`'s `src/lib.rs`, `src/main.rs`, `tests/app_model.rs`, and
  `tests/smoke_cli.rs`.
- **No track in the project document.** `ProjectDoc` carries `id`, `name`, `tempo_map`,
  `transport`, and a flattened `unknown` map — nothing else
  (`crates/spectre-project/src/lib.rs:29–38`).
- **No summing device, no mixer node, no master bus.** `spectre-dsp` exports exactly
  `Gain`, `Saturator`, `PulseInstrument`, `ToneSource` (`crates/spectre-dsp/src/lib.rs:11–19`).
- **No way for two nodes to reach one destination**, per `InputBusOccupied` above.
- **No per-track note routing.** `RenderBridge` holds a single `note_node`
  (`crates/spectre-audio/src/bridge.rs:122–123`).
- **No test covering the `InvalidLayout` input-count branch.**
  `grep -rn "MAX_FLAT\|InvalidLayout" crates/*/tests/` returns nothing.
- **A contradiction between the accepted architecture contract and the shipped code, which
  this slice inherits.** `docs/03-architecture/dsp-device-io.md:94` states "`Gain` already
  smooths" and `:104` describes it as "stereo linear gain with click-resistant smoothing."
  `crates/spectre-dsp/src/effect.rs:29–74` contains a single `gain: f32` field and a process
  loop that multiplies by it directly — **there is no smoothing state and no ramp**. R4-4
  reuses `Gain` unchanged and does not write DSP to fix this; the consequence is that once
  R4-2 makes track gain live, a mute toggle will click. Routed to §8 Q6, not repaired here.

### 7.2 Delta to spec

**New files**

- `crates/spectre-project/src/track.rs` — `MAX_TRACKS`, `TrackInstrument`, `TrackError`,
  `Track`, `TrackList`.
- `crates/spectre-project/src/routing.rs` — `GainNode`, `InstrumentNode`, `TrackPathNodes`,
  `RoutingError`, `build_track_graph`, `track_device_factory`.
- `crates/spectre-dsp/src/mix.rs` — `MAX_SUM_BUSES`, `SumBus`.
- `crates/spectre-project/tests/track_model.rs` — §5.1 tests 1–5.
- `crates/spectre-project/tests/track_routing.rs` — §5.1 tests 6–9.
- `crates/spectre-offline/tests/track_render.rs` — §5.2's three assertions.

**Modified files**

- `crates/spectre-project/src/lib.rs` — `pub mod track; pub mod routing;` and re-exports.
  `ProjectDoc` is **not** changed; the schema is R4-7's.
- `crates/spectre-project/src/command.rs` — five new `CommandKind` variants and their
  inverses.
- `crates/spectre-project/Cargo.toml` — add `spectre-dsp` and `spectre-graph` path deps.
- `crates/spectre-dsp/src/lib.rs` — `mod mix;` and `pub use mix::{SumBus, MAX_SUM_BUSES};`.
- `crates/spectre-graph/src/lib.rs` — `MAX_FLAT_INPUTS` 4 → 32, with the comment naming the
  bound's owner and its rationale row. No other line changes.
- `crates/spectre-offline/src/lib.rs` — add `render_track_list`, reusing `render_plan`'s
  existing hash walk (`:271–278`) rather than adding a second one.
- `crates/spectre-app/src/lib.rs` — delete `TrackView`; `AppModel` owns a `TrackList`;
  `tracks()` returns `&[Track]`; `add_track` delegates to `TrackList::push` and surfaces
  `TrackError`; new `remove_track`, `reorder_track`, `set_track_level`, `set_track_muted`,
  `set_track_soloed`, `set_master_level`, `track_structure_revision`.
- `crates/spectre-app/src/main.rs` — sidebar rows, state words, the removal of the `armed`
  glyph (`:131`), the row action cluster, the ceiling-disabled `+`, the inspector's model-
  derived signal-path line replacing the literal at `:184`, the descriptor-ranged level
  slider replacing the hard-coded `0.0..=1.0` at `:181`, the Mix master strip, and the
  transport bar's rebuild notice.
- `crates/spectre-app/src/engine.rs` — **R4-1's file**; add `build_track_engine_parts`, the
  `EngineUnavailable::Routing` variant, and `EngineParts::parameter_targets`.
- `crates/spectre-dsp/tests/devices.rs` — §5.1 tests 10–11, including extending
  `device_layouts_match_the_v1_contract` (`:186–210`) to cover the new layout.
- `cratests/spectre-graph/tests/containment.rs` — §5.1 test 12, reusing the file's existing
  `PoisonSource` (`:44`).
- `crates/spectre-graph/tests/graph_plan.rs` — §5.1 test 13.
- `crates/spectre-graph/tests/plan_alloc.rs` — §5.1 test 14, extending the existing fixture.
- `crates/spectre-app/tests/app_model.rs` — §5.1 tests 15–16.
- **`docs/01-requirements/requirements-ledger.md` — two rationale rows, required.** PROD-003
  (`requirements-ledger.md:64`) requires every numeric limit's rationale to be recorded **in
  that ledger**, and decision 16 makes it a standing rule, so §4.2's inline rationale does
  not discharge it. The rows to add, in a new `MIX` family:
  - `MIX-001 | A v1 project MUST NOT exceed MAX_TRACKS = 16 tracks summed into the master
    bus. | Rationale: a compile-time bound is structurally required because PlanStep's
    input map is a fixed [usize; MAX_FLAT_INPUTS] array (spectre-graph/src/lib.rs:337) and
    CompiledPlan::process builds a fixed [&[f32]; MAX_FLAT_INPUTS] on the stack
    (lib.rs:497); a heap collection on the render path would violate RT-001. The value 16
    is Spectre's own cost arithmetic — 139 KB of channel pool at 512 frames, 288 B per
    PlanStep, 512 B of stack per step — and is **not** taken from any reference product;
    the corpus contains no track-count record for any benchmark. Re-open trigger: the first
    multi-track callback-headroom measurement (R4-9). | acceptance evidence: §5.1 test 2 and
    test 13 | proposed`
  - `MIX-002 | The v1 flattened input bound MAX_FLAT_INPUTS MUST equal 2 x MAX_TRACKS. |
    Rationale: it is the same bound expressed in flattened stereo channels; keeping them
    derived rather than independently chosen is what prevents a project the model accepts
    from failing graph validation (RoutingError::TooManyTracks vs GraphError::InvalidLayout,
    §3.6 E6). | acceptance evidence: §5.1 test 13 | proposed`
- `docs/03-architecture/dsp-device-io.md` — add the fifth device layout to the list at
  lines 35–39: *"summing bus: N stereo audio inputs for 0 ≤ N ≤ MAX_SUM_BUSES, one stereo
  audio output, no note input."* **This is an amendment to an accepted contract** and is
  escalated to `gauntlet-output/decisions-needed.md` as §8 Q4 rather than made by assertion
  (AF-1).
- `gauntlet-output/decisions-needed.md` — new entries for §8 Q1, Q4, and Q5.
- `docs/status/STATUS.md`, `docs/status/NEXT.md`, `docs/06-plans/current-milestone.md` —
  updated when the slice lands, per `docs/README.md`'s working rule. Status moves to
  `implemented`, not `verified`, until §5.4's manual protocol passes.

**Deliberately not modified:** `crates/spectre-audio/src/bridge.rs`, `control.rs`,
`spsc.rs`, `midi.rs`, `null.rs`, `cpal_backend.rs`, `lib.rs`; `crates/spectre-core/`
entirely; `crates/spectre-dsp/src/effect.rs`, `source.rs`, `io.rs`, `parameter.rs`;
`crates/spectre-project/src/lib.rs`'s `ProjectDoc` and `SCHEMA_VERSION`. No second render
path is created, no existing DSP is rewritten, and the compiled plan's execution model is
untouched.

**Migrations / schema changes:** none in this slice; R4-7 owns them.
**New third-party dependencies:** none.

### 7.3 Estimated scope

**L.** Justification: roughly 700–900 new lines across four crates plus three new test files
of comparable size. It is above **M** because it introduces a new DSP device on the render
path (which carries its own RT-001 and RT-003 obligations), changes a graph constant that
two fixed-size arrays read, changes the dependency graph of a crate, deletes a public type
(`TrackView`) that a binary target and two tests consume, adds five command variants, and
proposes amendments to two accepted documents. It is below **XL** because it writes exactly
one device whose whole body is an accumulate-and-store loop, changes no schema, adds no
dependency, adds no compilation path, adds no control-transport lane, and leaves the
compiled plan's execution model, the bridge, and all four existing devices untouched.

### 7.4 Blocking dependencies

- **R4-1 (`live-audio-wiring`) blocks the *live* half of this slice, not the model half.**
  `crates/spectre-app/src/engine.rs` does not exist yet, so `build_track_engine_parts`
  cannot be written before R4-1 lands. Everything else — the track model, the commands, the
  routing builder, `SumBus`, the graph widening, the offline entrypoint, the app surfaces,
  and tests 1–16 except the engine wiring — is authorable and implementable today and does
  not depend on any audio device. If sequencing forces it, R4-4 can land in that order and
  connect to the engine when R4-1 does.
- **R4-2 (`runtime-parameter-seam`) blocks mute/solo/level being audible without a rebuild.**
  Not the reverse: R4-4 does not block R4-2. Until R4-2 ships, R4-4 must ship the copy that
  tells the truth about it (§3.2, §6.3).
- **R4-7 (`project-persistence`) blocks the persisted half of CORE-001's reorder evidence**
  and blocks routing the five new command variants through `EditHistory`, because
  `ProjectCommand::apply` takes `&mut ProjectDoc` (`crates/spectre-project/src/command.rs:52`)
  and the track list is not on `ProjectDoc` until R4-7 puts it there. §8 Q8.
- **R4-5 (`midi-clips`) will replace the single-note-node arrangement.** The `note_track`
  parameter is scaffolding standing in for clip playback; when clips exist, every track's
  instrument must be addressable and the parameter must be removed, not accumulated
  alongside.
- **R4-6 (`first-devices`) will replace `TrackInstrument::Pulse`.** The slot is designed to
  take a second variant without changing `Track`'s shape.
- **R4-9 (`e2e-and-qa`) owns the first multi-track headroom measurement**, which is the
  re-open trigger on MIX-001.
- **R4-3 (`linux-device-qualification`) blocks any Linux claim**, and §8 Q9 asks that its run
  use a multi-track project rather than the three-node fixture.
- **External gates:** an output device on the developer's machine for §5.4's manual
  protocol; a Linux box with real audio for R4-3.

---

## 8. Open Questions

- **Q1 — What is the alpha's track ceiling?** This spec proposes `MAX_TRACKS = 16` and shows
  its whole cost basis in §4.7 (139 KB of channel pool at 512 frames, 288 B per `PlanStep`,
  512 B of stack per step). The corpus contains **no** track-count record for any of the five
  benchmarks, so there is nothing to converge with or diverge from, and Spectre has no
  multi-track CPU measurement of its own. The number is therefore a judgement about how much
  preallocation an alpha should reserve, and that is Jeff's, not an agent's. 32 costs roughly
  twice the pool and 576 B per `PlanStep`; 8 halves it. — blocks §4.2, `requirements-ledger.md`
  MIX-001. **Escalated to `decisions-needed.md`.**
- **Q2 — Should each track own a device chain in R4?** This spec gives a track exactly one
  instrument and one gain, and leaves the existing flat `AppModel::devices` list
  (`crates/spectre-app/src/lib.rs:313–315`) driving Build and Shape untouched, so that
  R4-1's and R4-8's fixture path keeps working. The cost is that Build shows a device list
  that belongs to no track, which is a seam a user can see. Re-parenting the device browser
  onto tracks is R6 in the roadmap, but doing it later means changing Build twice. — blocks
  §3.1, §4.3.
- **Q3 — Should a structural edit hot-swap the plan instead of restarting the stream?** §4.4
  sketches the mechanism and shows it is RT-001-clean with the pieces that already exist —
  the reclaim lane already carries `Box<dyn Send>` (`crates/spectre-audio/src/control.rs:192`)
  and a `Box<CompiledPlan>` → `Box<dyn Send>` coercion allocates nothing — but it needs a
  fourth lane in `spectre_audio::control` and a policy for notes in flight across the swap.
  R4-4 deliberately does not build it, because a restart is honest and a half-designed swap
  is not. Should this be its own slice before R4-5? — blocks §3.2, §4.4.
- **Q4 — Does the accepted device-layout list gain a fifth entry?**
  `docs/03-architecture/dsp-device-io.md:35–39` enumerates exactly four v1 layouts and
  `SumBus` is none of them. This spec proposes adding *"summing bus: N stereo audio inputs
  for 0 ≤ N ≤ MAX_SUM_BUSES, one stereo audio output, no note input"* and flags it as an
  amendment rather than asserting it (AF-1). The alternative — declaring `SumBus` a
  sidechain effect and chaining a tree of two-input summers — needs a silent source to feed
  unpaired buses (because `compile` rejects an unfed input bus, `spectre-graph/src/lib.rs:227–240`)
  and puts `n` extra nodes in the plan. — blocks §4.3, §7.2. **Escalated to `decisions-needed.md`.**
- **Q5 — Is widening `MAX_FLAT_INPUTS` from 4 to 32 acceptable as a graph change?** It is a
  pure widening — every validation rule keeps its shape and no existing test asserts the old
  value — but it does mean the graph will now accept, say, a sixteen-bus effect that no
  accepted layout describes. The narrower alternative is a per-`DeviceClass` bound, which is
  more machinery for the same outcome. — blocks §4.3, §7.2. **Escalated to
  `decisions-needed.md`.**
- **Q6 — Who fixes `Gain`'s missing smoothing, and when?** `docs/03-architecture/dsp-device-io.md:94`
  and `:104` both describe `Gain` as smoothing; `crates/spectre-dsp/src/effect.rs:29–74`
  does not smooth. R4-4 reuses `Gain` as the track fader without writing DSP, so nothing
  clicks *yet* — values are baked at construction. The moment R4-2 makes them live, a mute
  is a 1 → 0 step and will click. Three options: fix `Gain` in R4-2 (where the seam that
  exposes the problem lands), fix it in R4-6 (device work), or correct the contract text to
  match the code and accept clicks in the alpha. — blocks §3.2, §8 Q3's audibility, R4-2.
- **Q7 — Is `spectre-project` the right home for the routing builder?** §4.1 puts it there
  because both `spectre-app` and `spectre-offline` already depend on `spectre-project`, which
  is what answers R4-1 §8 Q3's "placement is awkward". The cost is that the persistence crate
  now pulls in `spectre-dsp` and `spectre-graph`. A dedicated `spectre-session` crate between
  project and app would keep persistence lean at the cost of a sixth crate. — blocks §4.1.
- **Q8 — Do track commands go through `EditHistory` in R4-4 or R4-7?** `ProjectCommand::apply`
  takes `&mut ProjectDoc` (`crates/spectre-project/src/command.rs:52`), so undoable track
  edits need the track list on `ProjectDoc`, which is R4-7's schema change. R4-4 can either
  land the five variants against a `TrackList` seam now and rewire them at R4-7, or land the
  model now and the commands at R4-7. The first gets undo working sooner; the second touches
  `command.rs` once. — blocks §4.3, §7.4.
- **Q9 — Should R4-3's Linux qualification run against a multi-track project?** The macOS
  record (`docs/06-plans/current-milestone.md:113`) was taken on the three-node fixture. After
  R4-4 the alpha's plan is up to 34 nodes. Qualifying Linux on the small chain would produce
  a result that does not describe what ships. Adding a multi-track case to the drill is
  cheap, but it changes R4-3's accepted protocol, which is that spec's to define. — blocks
  §4.6, R4-3.
- **Q10 — Should solo be additive or exclusive?** This spec makes it additive: any number of
  tracks may be soloed. The only citable evidence on the point is `OBS-AB12-MIX-003` (Live
  makes solo and arm exclusive by default, with modifier and preference overrides), which is
  one benchmark rather than a convergence, and it describes a *default with an override*
  rather than a fixed behavior. No second benchmark in the corpus records solo semantics at
  all. Additive is the simpler model and the one that composes with `effective_gain` without a
  preference; exclusive-by-default matches the one product Spectre can cite. — blocks §3.2,
  §4.3.

---

## Appendix A — Benchmark evidence used, and where it does not exist

Cited, from the accepted corpus in `docs/02-reference-research/`:

- **`OBS-AB12-ROUTE-003`** (Live 12, §17.2.1): track device chains are always stereo even
  with mono input. Spectre **converges** — every track path in this spec is stereo end to
  end, which is already forced by `FLAT_OUTPUTS = 2`
  (`crates/spectre-graph/src/lib.rs:16`) and `CHANNELS_PER_BUS = 2` (`:12`). The same record
  notes Live attenuates 6 dB when summing stereo to mono; Spectre has no mono path, so the
  number is not adopted and no attenuation constant is introduced.
- **`OBS-AB12-MIX-002`** (Live 12, §18.1.1): a 32-bit float engine tolerates over-0 dB
  between tracks without clipping, and clipping matters only at physical outputs. Spectre
  **converges**: `SumBus` applies no limiter, no clip, and no normalization, and §5.4 warns
  about it rather than hiding it behind an invisible guard.
- **`OBS-AB12-MIX-003`** (Live 12, §18.3): solo and arm are exclusive by default with
  modifier and preference overrides. Spectre **diverges** in R4-4 — solo is additive — and
  §8 Q10 puts the choice in front of Jeff rather than settling it here. Arm is absent
  entirely, because recording is R7.
- **`OBS-AB12-MIX-004`** (Live 12, §18.3) and **`OBS-AB12-MIX-005`** (§18.4): group tracks
  with their own mixer controls, and return sends with pre/post tap points. Both are real
  and both are **deliberately deferred to R6** (`docs/06-plans/rebuild-roadmap.md:31`).
  Cited to show the deferral is informed rather than accidental.
- **`OBS-AB12-MIX-001`** (Live 12, §18.1): track meters show peak and RMS. R4-4 ships **no
  meters**, and §3.3 says so explicitly rather than dressing a fader-position bar as one.
  Metering is R6.
- **`OBS-AB12-MIX-009`** (Live 12, §18.9): per-track performance-impact indicators. Spectre
  **diverges** for the same reason R4-1 recorded: there is no track-level DSP attribution in
  the plan, and inventing one would be a fake surface.
- **`OBS-AB12-SES-005`** (Live 12, §7.5) and **`OBS-BW53-LAUNCH-001`** (Bitwig, corroborating
  rather than benchmark): per-track mutual exclusion between timeline and launcher playback,
  with an explicit "return authority" affordance. This is PROD-001
  (`docs/01-requirements/requirements-ledger.md:62`), gated at R10. R4-4's obligation is only
  not to foreclose it: a track has exactly one playback source in R4, and nothing in `Track`
  assumes it will stay that way.
- **`OBS-PP-ARCH-002`** (Phase Plant): generators auto-route top-to-bottom, **each mixing
  onto the signal from above** — implicit summing at every stage. **`OBS-BW53-GRID-002`**
  (Bitwig, corroborating): an out port fans out to unlimited in ports, **an in port accepts
  exactly one cable**, and unconnected in ports read zero — explicit summing only. The two
  researched systems **diverge**, so there is no convergent pattern to follow. Spectre keeps
  the explicit model, which is already its graph's rule (`GraphError::InputBusOccupied`,
  `crates/spectre-graph/src/lib.rs:171–180`), and adds a visible summing device rather than
  making buses implicitly mix. The reason is stated rather than assumed: implicit bus mixing
  would mean the plan carries a summing step no node accounts for, which is harder to
  reconcile with GRAPH-001's "the plan contains exactly the ancestors of the output node".
- **`OBS-PP-ARCH-001`** (Phase Plant): at least one output module is required to produce
  sound. Spectre **converges** — `compile` takes an explicit output node
  (`crates/spectre-graph/src/lib.rs:196–201`) and an empty track list compiles to a zero-bus
  `SumBus` and a master gain producing exact silence, so "nothing to hear" is a construction
  rather than an error state.
- **`OBS-VCV-VOLT-005`** (VCV Rack 2): missing channels on an under-provisioned polyphonic
  input read 0 V. Cited only for the narrow convention that an absent input reads zero, which
  is what the zero-bus `SumBus` branch does.
- **`OBS-VCV-VOLT-006`** (VCV Rack 2): modules should output 0 on NaN/infinity detection.
  Already RT-003's recorded provenance (`requirements-ledger.md:30`); `SumBus` inherits it at
  its own boundary (§4.3) and through `CompiledPlan`'s containment
  (`crates/spectre-graph/src/lib.rs:512–522`).
- **`OBS-PP-FX-001`** (Phase Plant): post-generator effect lanes carry per-lane mute/solo,
  gain, mix, and a send-to destination of "next lane or master". Cited as the corpus's only
  record placing mute, solo, and level together on a routing lane with an explicit master
  destination — the shape R4-4 gives a track. Spectre takes the grouping and **not** the
  per-lane mix control or the send, which are R6.

**Named gaps — evidence that does not exist and was not invented:**

- **No benchmark in the corpus records a maximum track count, a channel-count limit, or any
  capacity bound.** The Ableton corpus's 85 records span exactly nine categories — ARR, AUTO,
  CLIP, LAUNCH, MIX, REC, ROUTE, SES, WARP — and contains no capacity record;
  `grep -in "maximum\|limit" ableton-live-observations.md` returns one hit, and it is
  `OBS-AB12-LAUNCH-007` stating a track may contain *any number* of follow-action groups.
  `MAX_TRACKS` is therefore Spectre's own arithmetic (§4.7, §8 Q1) and is recorded as a
  research need, not sourced.
- **No benchmark record describes a summing-bus architecture, a mixer's internal accumulation
  order, or a fader law (dB curve, taper, or unity position).** Spectre's fader is the
  accepted `GAIN_PARAMETERS[0]` linear descriptor
  (`crates/spectre-dsp/src/effect.rs:19–20`) precisely so that no curve is invented; a dB
  taper is a future decision with its own rationale row, not something this spec assumes.
- **Logic Pro: zero citable behavioral observations.** `docs/02-reference-research/logic-pro.md`
  is inventory-only. This spec asserts nothing about Logic Pro's track model, mixer, summing,
  solo semantics, or capacity, and its silence here is a gap in the corpus rather than a gap
  in the analysis. D-R1 in `gauntlet-output/decisions-needed.md` records the decision of
  whether to commission a Logic pass before R4-4 reaches implementation, and names R4-4
  specifically as one of the two features that would benefit.
- **Serum 2: two citable records** (`OBS-SR2-CPU-001`, `OBS-SR2-KB-001`), and its dossier is
  `blocked-source-gap`. Neither concerns tracks, mixing, or routing, so **neither is used in
  this spec**. The ~250-record extraction in `serum-2-observations.md` is quarantined under
  D-R2 and is cited by nothing here.
- **No benchmark record describes what a DAW does when a structural edit lands while audio is
  running** — whether it hot-swaps, restarts, or defers. §4.4's restart-based answer and §8
  Q3's hot-swap alternative are both Spectre's own reasoning from its own realtime contract,
  and are recorded as a research need rather than dressed in a citation.

---

**End of spec.**
