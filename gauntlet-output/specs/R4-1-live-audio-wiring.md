<!--
Author: Jeff
Date: 2026-08-15
Description: R4-1 spec — wire ./spectre to the qualified audio backend so Play produces sound through the existing compiled plan
Notes: No new render path and no new DSP. Iteration 2 corrects the rt_guard scan list (null.rs is
  scanned, midi.rs is not) and rebuilds the RT-001 argument on what the null.rs edit actually contains,
  replaces the invalid note-ordering guarantee with the counted fail-closed behavior, and makes tests 3
  and 9 compilable. Today ./spectre makes no sound at all.
-->

# Spec: Live Audio Wiring

**Feature ID:** `R4-1` (`live-audio-wiring`)
**Parent feature:** `R4` Credible Alpha (root)
**Spec author agent:** gauntlet spec agent, R4-1 leaf
**Date:** 2026-08-15
**Iteration:** 2 (remediation 1)

- **Status:** proposed
- **Last verified:** 2026-08-15 (source read at commit `dae16bb`, branch `rename/geist-to-spectre`)
- **Scope:** connecting `crates/spectre-app` to `crates/spectre-audio` so the launchable binary opens a real output stream and renders the existing compiled plan
- **Decision authority:** Jeff
- **Upstream sources:** `docs/00-product/vision.md`, `docs/01-requirements/requirements-ledger.md` (RT-001/002/003, GRAPH-001), `docs/01-requirements/decision-gates.md` (rows 1, 8, 16, 19, 20, 21, 22, 23), `docs/03-architecture/dsp-device-io.md`, `docs/06-plans/current-milestone.md` §"Wiring gap", `docs/status/NEXT.md` slice 1
- **Downstream dependents:** R4-2 (runtime parameter seam), R4-3 (Linux qualification), R4-5 (MIDI clips), R4-8 (offline bounce), R4-9 (e2e and QA)
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** §8 Q1–Q9
- **Known gaps:** no benchmark in the accepted corpus has a citable observation about audio-device selection, driver configuration, or engine start/failure UX; recorded as a research need in Appendix A ("Named gaps") and in §8

This spec is subordinate to the conflict precedence in `docs/README.md`. It proposes
behavior; it does not amend an accepted requirement, decision row, or architecture
contract. Where it touches accepted material it says so and routes the question to §8.

---

## 1. Purpose

### 1.1 One-sentence job

When a musician launches `./spectre` and presses Play, they hear Spectre's own signal
chain through their audio interface — the same computation the offline harness already
renders — instead of watching a transport button change shape in silence.

### 1.2 Why it matters

R3 built a complete live shell that nothing launches. `spectre-audio` has a backend
seam, a control transport, a callback bridge, MIDI ingress, and health telemetry, all
tested; `crates/spectre-app/Cargo.toml` does not depend on `spectre-audio` at all, so
none of it runs in the product. `docs/06-plans/current-milestone.md` §"Wiring gap"
names this the milestone's honest headline, and `docs/status/STATUS.md` records it in
its own known-gaps line: *"`./spectre` does not use the audio crate, so nothing
launchable makes sound."*

**Stated plainly: `./spectre` currently produces no sound of any kind.** Pressing Play
mutates an in-memory `Transport` (`crates/spectre-app/src/lib.rs:268`) and nothing
else. The transport bar prints a hard-coded string `ENGINE OFFLINE`
(`crates/spectre-app/src/main.rs:79`), which is accurate today and must become
evidence-backed the moment an engine exists.

Every later R4 slice is unobservable until this lands. A track model (R4-4), a MIDI
clip (R4-5), and an original synth (R4-6) are all silent artifacts without it, and
R4-8's bounce has no live path to be compared against.

### 1.3 Success signal

On a machine with a working default output device, `./spectre` opens a stream, the
transport bar reports a nonzero and advancing `blocks_rendered` count, and pressing
Play produces audible output whose one-block interleaved FNV-1a hash equals
`spectre_offline::render_app_snapshot` for the same app snapshot and the same fixture
events. The equality is asserted by an automated test against the null backend
(`cargo test -p spectre-app --test live_engine`); the audibility is confirmed by the
manual protocol in §5.4 and by the hardware drill's telemetry line.

---

## 2. User Stories

> As an electronic musician launching Spectre for the first time, I want the app to
> open my default output device by itself and tell me which device and format it got,
> so that I can trust the transport button before I invest a session in the tool.

> As that same musician, I want Play to produce sound and Stop to produce exact
> silence, so that the transport control means what every other DAW has taught me it
> means.

> As a musician on a machine whose default device is busy, missing, or locked to a
> format Spectre cannot open, I want the app to keep running, tell me exactly what the
> driver refused, and offer a retry, so that a device problem costs me a retry instead
> of a crash and a lost session.

> As a musician who cares whether the engine is keeping up, I want the app to show real
> callback headroom, xrun count, and containment counters read from the render thread,
> so that "the engine is fine" is a measurement and not a decoration.

> As a keyboard-only or screen-reader user, I want every new engine control and readout
> to be a focusable, text-labelled element whose state is legible without color, so
> that decision 17's beta accessibility bar is not foreclosed by this slice.

> As Jeff running CI on a headless Linux container with no audio device, I want
> `./spectre --smoke-test` and the workspace suite to pass without ever opening a
> device, so that the gate stays deterministic and a false pass is impossible.

> As the maintainer reviewing R4-2, I want the live plan built from the same validated
> app snapshot the offline harness consumes, so that the parameter seam has one set of
> values to reason about rather than two.

---

## 3. UX Specification

### 3.1 Screen / view inventory

R4-1 introduces **no new screen, modal, sheet, popover, or drawer**. It modifies three
existing regions of the single-window shell built in `crates/spectre-app/src/main.rs`.

| Region | Navigation path | New or modified | Layout pattern |
|---|---|---|---|
| Transport bar (`transport()`, main.rs:50–84) | always visible, top of window, 62 px fixed | modified | full-width top panel, horizontal centered row |
| Build lens footer copy (`build_devices()`, main.rs:328–333) | lens selector → Build (or the prototype's existing `2` binding) | modified (text only) | central panel body text |
| Shape lens header copy (`shape_devices()`, main.rs:337–342) | lens selector → Shape | modified (text only) | central panel body text |

Nothing is added to Arrange or Mix. The transport bar is chosen for engine state
because it is the only region visible from all four lenses, which keeps engine state
one global fact rather than a per-lens fork (vision.md, "One project, linked lenses").

### 3.2 Interaction flows

**Primary flow — launch to sound.**

1. The user runs `./spectre`. The launcher execs `cargo run --locked --quiet -p
   spectre-app` (`spectre`, line 9).
2. The window opens. Engine state is `NotStarted`; the transport bar reads
   `Engine — starting`.
3. On the **first** `eframe::App::update` call, and exactly once, the app attempts
   engine start: enumerate the default device, read its sample rate, validate a stereo
   `StreamConfig`, build the plan from the app's device-parameter snapshot, open the
   stream, start it. (Rationale for doing this after the first frame rather than in
   `main()` is in §4.7.)
4. On success the bar reads `Engine cpal · <device name> · <rate> Hz · 256 fr`, with a
   live `blocks` counter, `headroom` percentage, and `xruns` count beside it. **The
   running label is derived from stream state AND a nonzero block count**: a stream
   that opened but has never called back reads `Engine cpal · <device> · opened, no
   callbacks yet`, never `running`.
5. Nothing is audible yet. The fixture instrument is note-driven and holds no note, so
   the chain renders exact silence — the same silence `spectre_offline::render_silence`
   already proves for this chain. This is a property of the material, not a mute gate.
6. The user presses Play. The app calls `start_audition`, which sends
   `TransportCommand::Play` on the transport lane and then one held `NoteEventKind::On`
   on the note lane, in that order. **The UI transport flips if and only if the transport
   send returned `Ok`** — see §4.4's binding rule for why the note send cannot gate it.
7. Within one block period the bridge drains both lanes, the `PulseInstrument` opens,
   and sound reaches the device.
8. The user presses Stop. The app calls `stop_audition`, which sends
   `NoteEventKind::AllNotesOff { channel: None }` and then `TransportCommand::Stop`. The
   bridge collects notes after applying transport (`bridge.rs:175–176`), so the release is
   delivered on the same block; the instrument clears its active note
   (`crates/spectre-dsp/src/source.rs:194`) and the chain returns to exact silence.
   If step 6 and step 8 land inside the *same* block period, the two note events are
   keyed out of order and the block is refused into exact silence with `plan_errors`
   incremented — §4.3's note-ordering paragraph and §3.6 E10.

**Branch — the engine cannot start.** Any `BackendError` from enumeration, rate query,
config validation, or open leaves the engine in `Failed(error)`. The bar reads
`Engine unavailable — <BackendError Display>` and shows a `Retry engine` button. The
rest of the app is fully usable; no dialog is raised and no state is lost.

**Branch — the build has no backend.** With `spectre-audio`'s `cpal-backend` feature
disabled, the app never selects a backend and reads `Engine unavailable — built without
an audio backend`. It does **not** silently fall back to `NullBackend`: a null stream
that renders into a discarded buffer would be a fake "running" surface.

**Branch — the driver hands an oversized block.** Already handled in shipped code:
`RenderBridge::render` fills exact silence and increments
`frame_capacity_rejections` (`bridge.rs:167–173`). R4-1 surfaces that counter; it adds
no new behavior.

**Sound, haptic, and animation cues.** No new animation, no haptics, no UI sound. The
only new sensory output is the audio itself. One honest warning belongs in the copy and
in the QA protocol: `PulseInstrument` has no amplitude envelope (`source.rs:202–209`),
so note-on and note-off are hard gates and **will click**. An envelope is R4-6's device
work, not R4-1's, and this spec does not add one.

### 3.3 Layout descriptions

**Transport bar, leading → trailing** (existing content unchanged unless noted):

1. `SPECTRE` wordmark — unchanged.
2. Play / Stop button — unchanged appearance; its handler gains the send-before-mutate
   order from §3.2 step 6.
3. Record button — **modified**: rendered disabled with the hover reason "Recording
   arrives at R5." It is currently an enabled button that does nothing
   (`main.rs:72–73`); once the engine is real, an enabled Record button is a fake
   surface under the vision's release bar.
4. Separator.
5. `120.00 BPM`, `4 / 4`, `001 · 01 · 000` — these are hard-coded literals
   (`main.rs:75–77`), not model-derived. **Modified**: the position literal becomes `—`
   and all three carry a hover reason "Not driven by the engine yet; arrives with
   clips (R4-5)." A live engine beside a frozen bar counter is exactly the fake surface
   `vision.md`'s alpha bar prohibits. Whether to wire real position now is §8 Q5.

**Right-to-left cluster (trailing edge), reading order left→right as displayed:**

6. `Engine <state>` — single text label, described in §3.6. Data source:
   `LiveEngine::state()`.
7. `blocks <n> · headroom <p>% · xruns <n>` — data source: `EngineHealth`, an owned
   struct read from `BridgeTelemetry`'s atomics on the app thread each frame.
8. `contained <n> · refused <n> · plan errors <n>` — data source:
   `BridgeTelemetry::contaminated_nodes`, `frame_capacity_rejections`, and `plan_errors`.
   Each is shown only when it is nonzero, because a permanently-zero counter is noise;
   when nonzero it must never be hidden. `plan errors` is in this cluster because §4.3's
   same-block Play→Stop refusal is counted there and would otherwise be a silent failure.
9. `Retry engine` button — present only in the `Failed` state.

**Empty states.** Before the first update, `Engine — starting`. That string is
transient by one frame and must not be reachable as a resting state; if the start
attempt is skipped (smoke mode), the state is `NotStarted { reason }` and the reason is
displayed verbatim.

**Build lens footer** (`main.rs:330`) currently reads "DSP is active in deterministic
offline rendering; live audio routing is not connected yet." Replace with: "This chain
is the live signal path. Parameter edits reach live audio at R4-2; until then they
apply to offline rendering and to the next engine start."

**Shape lens header** gains one line: "Edits apply to offline rendering and to the next
engine start. Live parameter application arrives with R4-2 (decision 22)." This is
required, not cosmetic: without it, moving a slider while hearing sound implies a
connection that does not exist.

### 3.4 Input & gestures

- Play / Stop: pointer click on the existing button. The prototype also binds `Space`
  and `1`–`4` (`main.rs:425–437`). Those bindings are pre-existing prototype
  scaffolding. **This spec neither extends, ratifies, nor documents a default shortcut
  map**, because `docs/02-reference-research/workflow-field-study/product-implications.md`
  §"Prohibited conclusions at current evidence level" lists a default shortcut map as
  unsupported by the corpus. The command system, its context scoping, and its
  remappability are a later slice.
- `Retry engine`: pointer click, and keyboard activation via egui's standard focus and
  Enter/Space handling. No dedicated binding is assigned, for the reason above.
- Specialized input (stylus, controller, voice, camera): N/A — R4-1 adds no such
  surface. Live MIDI input is out of scope: `spectre_audio::midi::MidiIngress` exists
  and is tested, but nothing in the app feeds it, and adding a MIDI device path is
  R4-5's work.
- Responsive behavior: the transport bar is a fixed-height 62 px panel on a window with
  a 1060 px minimum width (`main.rs:508`). The new cluster is text; at the minimum
  width it must truncate the device name rather than push the Play button off-screen.
  Truncation applies to the device name only — never to the state word or a counter.

### 3.5 Transitions & animation

- Navigation transitions: none. No view is added or removed.
- In-view state change: the engine label and counters change value on the existing
  250 ms repaint cadence (`main.rs:443`). No fade, no easing, no motion is introduced —
  the numbers simply update. No new timing constant is introduced.
- Reduced motion: because R4-1 introduces no animation, there is nothing to suppress; a
  reduced-motion setting has no effect on this feature. That is the complete answer, not
  a deferral.

### 3.6 Error states

Presentation is **inline, persistent, in the transport bar** for every case below. That
choice is deliberate: engine availability is a persistent condition, not an event, so a
toast (which disappears) or a modal (which blocks the workspace and would fire before
the user has done anything) would both misrepresent it. The transport bar is the one
region visible from every lens, which keeps a single condition in a single place.
Recoverable per-action failures reuse the existing `feedback_status` line in the
inspector (`main.rs:210`), the same channel `set_device_parameter_from_ui` already uses.

| # | Trigger | Presentation | Recovery path | Data loss |
|---|---|---|---|---|
| E1 | Built without `cpal-backend` | `Engine unavailable — built without an audio backend` | rebuild with the default feature set | no |
| E2 | `BackendError::NoDefaultDevice` | `Engine unavailable — no default output device` + `Retry engine` | connect/select a device in the OS, press Retry | no |
| E3 | `BackendError::EnumerationFailed(s)` | `Engine unavailable — device enumeration failed: <s>` + Retry | as above | no |
| E4 | `UnsupportedSampleRate` / `UnsupportedChannels` / `UnsupportedBufferSize` from `StreamConfig::validate` | `Engine unavailable — <Display>` + Retry | change the OS device format, press Retry | no |
| E5 | `BackendError::OpenFailed(s)` — the host refused the fixed buffer size or the stereo format | `Engine unavailable — stream open failed: <s>` + Retry | change device/format, press Retry; see §8 Q4 for the ALSA case | no |
| E6 | `GraphError` while compiling the live plan (would indicate a broken snapshot) | `Engine unavailable — plan build failed: <Display>` + Retry | none in-app; this is a defect, and the message must say so rather than implying user error | no |
| E7 | `AuditionError::Transport(..)` — the transport lane refused the command (`ControlError::TransportLaneFull`, `control.rs:36`) | inspector status line: "Transport command refused: <Display>. Press again." **UI transport state does not change, and the note send is not attempted.** | press the button again | no |
| E7b | `AuditionError::Note(..)` — the transport command was queued but the note lane refused (`ControlError::NoteLaneFull`, `control.rs:35`) | inspector status line: "Audition note refused: <Display>. Transport applied." **UI transport state changes**, because the queued command cannot be recalled from a wait-free lane | none needed for transport; press Play again to retry the note | no |
| E8 | Device disappears mid-run (unplug) | counters stop advancing and `stream errors <n>` appears (nonzero `AudioStream::stream_errors`) | press Stop, then `Retry engine` to reopen | no |
| E9 | Driver delivers a block larger than the plan's capacity | `refused <n>` counter becomes visible; audio is exact silence for those blocks, never stale | already fail-closed in shipped code; Retry re-opens at the current device format | no (audio only) |
| E10 | Play and Stop land in the same block period, so the note-on and its `AllNotesOff` are keyed out of order (§4.3) | `plan errors <n>` counter becomes visible; that one block is exact silence and both note events are discarded together, leaving no stuck note | none required — the transport commands still applied and the next block renders normally; press Play again | no (one block of audio) |

Two rules bind all eleven rows. First, **no error is reported from the audio thread**:
every string above is formatted on the app thread from a value the render side
published into an atomic or returned from an app-thread call (RT-001). Second, **no
threshold-derived alarm state is specified** — there is no "engine unhealthy" badge
computed from headroom or from a stalled block counter, because Spectre has exactly one
hardware measurement (macOS, 2026-08-09, worst headroom 0.990) and one data point
cannot justify a threshold (decision 16, PROD-003).

### 3.7 Accessibility

- Every new element is a text label or a standard egui button; there is no icon-only
  control, no color-only state, and no custom-painted widget in this feature. The engine
  state is carried by the words `starting` / `opened, no callbacks yet` / `running` /
  `unavailable`, so a monochrome or color-blind reading loses nothing.
- Screen reader labels, hints, traits: `crates/spectre-app/Cargo.toml` builds eframe
  with `default-features = false` and only `default_fonts` and `glow`, so **no
  accessibility feature is enabled in this workspace today** and R4-1 does not claim
  screen-reader support. Decision 17 gates that at beta with a scoped audit at R4;
  R4-1's obligation is to not foreclose it, which it satisfies by using standard
  widgets with text content rather than painted glyphs.
- Custom actions for complex interactions: N/A — no compound gesture is added.
- Text scaling / dynamic type: the transport panel is a fixed 62 px height
  (`main.rs:50–53`). At large egui zoom the new cluster must wrap or truncate the device
  name inside the panel rather than clip the Play button; this is a manual check in
  §5.4, not an automated one.
- Focus order and keyboard navigability: egui's focus order follows widget creation
  order, and the new cluster is created inside a `right_to_left` layout
  (`main.rs:78`), so creation order runs opposite to visual order. The `Retry engine`
  button must therefore be created so that it is reached in a position matching its
  reading order; if that proves impossible without restructuring the layout, the
  mismatch is recorded for decision 17's R4 audit rather than papered over.

---

## 4. Implementation Specification

### 4.1 Architecture placement

| Path | Change | Thread |
|---|---|---|
| `crates/spectre-app/src/engine.rs` | **new** — app-thread engine host: plan construction, control transport, stream ownership, telemetry read-out | app thread only |
| `crates/spectre-app/src/lib.rs` | add `pub mod engine;`; no change to `AppModel` semantics | app thread only |
| `crates/spectre-app/src/main.rs` | own `Option<LiveEngine>`; transport-button handler; status rendering; copy changes | app thread only |
| `crates/spectre-app/Cargo.toml` | add `spectre-audio`, `spectre-graph` deps; add default-on `live-audio` feature forwarding `spectre-audio/cpal-backend`; add `spectre-offline` dev-dep | build |
| `crates/spectre-audio/src/lib.rs` | add one method to `AudioBackend`, one to `AudioStream` | app thread only |
| `crates/spectre-audio/src/null.rs` | implement both new methods | app thread only |
| `crates/spectre-audio/src/cpal_backend.rs` | implement both new methods; move `error_count`'s body behind the trait method | app thread only |

**The RT-001 structural argument, stated against the actual scan.**
`crates/spectre-audio/tests/rt_guard.rs` — a test file, not a `src` module — declares
`RT_MODULES` at lines 293–298 with exactly four entries, read verbatim at lines 294–297:
`src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, `src/null.rs`. **`midi.rs` is not
scanned; `null.rs` is.** The forbidden set is `FORBIDDEN` at lines 299–307, seven
needles read verbatim at lines 300–306: `Mutex`, `RwLock`, `Condvar`, `thread::sleep`,
`println!`, `eprintln!`, `dbg!`. The test asserts by substring that no scanned module's
text contains any of them (`rt_guard.rs:313–318`).

R4-1 therefore **cannot** argue from an untouched scanned set, because the table above
modifies one member of it — `crates/spectre-audio/src/null.rs` gains the two new trait
methods. The argument is made on the content of that edit instead:

- Three of the four scanned modules — `bridge.rs`, `control.rs`, `spsc.rs` — are not
  modified at all, so their scan result is unchanged by construction.
- The fourth, `null.rs`, gains exactly two app-thread method bodies, and **both return a
  constant**: `NullBackend::default_sample_rate` returns `48_000` and
  `NullStream::stream_errors` returns `0` (§4.3). Neither body allocates, blocks, logs,
  or names any of the seven forbidden needles, so the scan's result for `null.rs` is
  unchanged as well. The relaxed atomic load that `stream_errors` exists to expose lives
  in `cpal_backend.rs:166`, which the scan does not cover and which R4-1 does not add —
  it is the body of the existing inherent `error_count` (`cpal_backend.rs:163–168`),
  moved behind the trait method.
- Because this is no longer an untouched-module argument, it must be re-proved rather
  than assumed: §5.2 requires `cargo test -p spectre-audio --test rt_guard` to pass
  **after** the `null.rs` edit, and that pass is the evidence, not this paragraph.

The callback body the app installs is `move |mut block| bridge.render(&mut block)` —
substantively the closure already exercised by
`crates/spectre-audio/tests/lifecycle_health.rs:269` under real CoreAudio hardware and by
`crates/spectre-audio/tests/rt_guard.rs`'s null-stream guard test. (That line reads
`Box::new(move |mut block: RenderBlock| bridge.render(&mut block)),`; the explicit type
annotation is the only difference, so "identical closure" is a claim about behavior, not
about characters.) No new code is placed on a callback-reachable path, and §5.1 test 11
re-runs the allocation guard over the app's own wiring rather than trusting the argument
above.

The engine lives in the **library** crate rather than in `main.rs` for one concrete
reason: `main.rs` is a binary target and its types are unreachable from
`crates/spectre-app/tests/`. `AppModel` stays renderer-neutral and audio-free; it gains
no audio dependency and no new field.

### 4.2 Data model

New types, all in `crates/spectre-app/src/engine.rs`. **None of these exist today.**

```rust
// Author: Jeff
// Date: 2026-08-15
// Description: App-thread host for the live audio engine
// Notes: Every method here runs on the app thread and may allocate. Nothing in this file
//   is reachable from the audio callback; the only value that crosses is the RenderBridge
//   moved into the render closure at open.

// Numeric bounds this slice introduces, each with its own rationale (decision 16, PROD-003)

// Requested driver block size. Rationale: 256 frames is the only block size for which
// Spectre has its own measured hardware evidence — the macOS qualification recorded in
// docs/06-plans/current-milestone.md rendered 173 callbacks at 256/48 kHz with 0 xruns and
// 0.990 worst-case headroom. Reusing it means the first live run of ./spectre is compared
// against a baseline Spectre measured, not a number taken from another product. Re-open when
// R4-3's Linux qualification produces a second measurement.
pub const ENGINE_BUFFER_FRAMES: usize = 256;

// Plan capacity margin over the requested block. Rationale: the seam asks cpal for
// BufferSize::Fixed (crates/spectre-audio/src/cpal_backend.rs:129) but no Spectre evidence
// proves every host honors it, and the macOS record does not include a frame-capacity
// rejection count, so a larger-than-requested block is unproven-absent rather than known-absent.
// The margin's entire cost is 6,144 additional preallocated bytes (6 plan channels x 256
// frames x 4 B); any block beyond the margin still lands in the already-implemented counted
// silence refusal. Not derived from any reference product.
pub const ENGINE_PLAN_FRAME_MARGIN: usize = 2;

// Why the engine is not running, phrased for direct display
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineUnavailable {
    // Compiled without a real backend; the app declines to fake one with NullBackend
    NoBackendCompiled,
    // Headless/smoke invocation deliberately skipped the start attempt
    NotAttempted,
    Backend(spectre_audio::BackendError),
    // The live plan could not be compiled from the app snapshot; this is a defect, not user error
    Plan(String),
    Control(spectre_audio::control::ControlError),
    // The app snapshot was not the complete canonical fixture
    Snapshot(crate::DeviceParameterSnapshotError),
}

// What the transport bar renders; derived, never stored as a claim
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineState {
    Unavailable(EngineUnavailable),
    // Stream opened and started, but the driver has not called back yet
    Opened { backend: &'static str, device: String, sample_rate: u32, frames: usize },
    // Opened AND blocks_rendered > 0; the only state allowed to read as "running"
    Running { backend: &'static str, device: String, sample_rate: u32, frames: usize },
}

// Owned copy of the render thread's counters, read once per frame
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineHealth {
    pub blocks_rendered: u64,
    pub xruns: u64,
    pub worst_headroom: f32,
    pub last_headroom: f32,
    pub plan_errors: u64,
    pub frame_capacity_rejections: u64,
    pub notes_deferred: u64,
    pub contaminated_nodes: u64,
    pub stream_errors: u64,
}

// Render-side halves built together, before any device is touched
pub struct EngineParts {
    pub bridge: spectre_audio::bridge::RenderBridge,
    pub sender: spectre_audio::control::ControlSender,
    pub telemetry: std::sync::Arc<spectre_audio::bridge::BridgeTelemetry>,
    pub config: spectre_audio::StreamConfig,
    // Frames the plan was compiled for. RenderBridge's plan field is private and its public
    // surface is only new/telemetry/transport/render (bridge.rs:133/152/157/162), so plan
    // capacity is unreadable through the bridge; this slice adds no accessor to bridge.rs and
    // carries the value build_engine_parts passed to EditableGraph::compile instead
    pub plan_max_frames: usize,
}

// Which half of a two-send audition sequence was refused; the caller needs the distinction
// because a queued transport command cannot be recalled from a wait-free lane
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditionError {
    // The transport send was refused, so the render transport will not change and the UI
    // must not either. Dominant: returned even if the note send was also refused
    Transport(spectre_audio::control::ControlError),
    // The transport command was queued; only the note send was refused, so the UI change stands
    Note(spectre_audio::control::ControlError),
}

// App-thread owner of the live stream; not Send, because AudioStream is not Send.
// Generic over the stream with dyn AudioStream as the default: the app holds the erased
// LiveEngine<dyn AudioStream> that open_default returns, while a test can hold a concrete
// LiveEngine<NullStream> and still reach NullStream::pump, which is inherent to NullStream
// (null.rs:104) and absent from the AudioStream trait (spectre-audio/src/lib.rs:181–196)
pub struct LiveEngine<S: ?Sized = dyn spectre_audio::AudioStream> {
    sender: spectre_audio::control::ControlSender,
    telemetry: std::sync::Arc<spectre_audio::bridge::BridgeTelemetry>,
    backend_name: &'static str,
    device_name: String,
    config: spectre_audio::StreamConfig,
    // Sequence counter for outgoing note events; the ordering key's tie-break
    next_sequence: u64,
    // Fixed identity of the single audition voice
    voice_id: u32,
    // Box<S> is itself Sized even when S is not, so field order is unconstrained here
    stream: Box<S>,
}
```

`AudioStream` is object-safe — every method takes `&self` or `&mut self`, none is generic,
none returns `Self` (`crates/spectre-audio/src/lib.rs:181–196`) — so `dyn AudioStream`
satisfies the `S: AudioStream` bound the `impl` block carries, and
`Box<NullStream> → Box<dyn AudioStream>` coerces. The generic parameter changes no
behavior and adds no method to the audio seam; it exists so §5.1 test 9 can compile.

Database migrations: **N/A — R4-1 persists nothing.** The project envelope
(`spectre-project`) is untouched; CORE-004's filesystem implementation is R4-7.

### 4.3 API contracts

**New — `crates/spectre-audio/src/lib.rs`, two trait methods.** Both are app-thread
surfaces on traits whose doc comments already state that every method is app-thread-only
(`lib.rs:198`). Both may allocate. Neither is reachable from a callback.

```rust
// On trait AudioBackend:
// Report the sample rate the device is currently configured for, before any open attempt
fn default_sample_rate(&self, device: &DeviceId) -> Result<u32, BackendError>;

// On trait AudioStream:
// Count driver-thread stream errors recorded since open
fn stream_errors(&self) -> u64;
```

- `default_sample_rate` exists because the seam today offers **no way to learn a
  device's format**: `DeviceInfo` carries only `id`, `name`, `max_output_channels`, and
  `is_default` (`lib.rs:47–53`). Without it the app must guess a rate, and a guess of
  48 000 fails on any device locked to 44 100. `CpalBackend` implements it from
  `cpal::Device::default_output_config`. `NullBackend` returns `48_000`; rationale for
  that constant: the synthetic device has no hardware rate, and 48 000 is the value
  every existing `spectre-audio` test already asserts against
  (`bridge_plan.rs:19`, `lifecycle_health.rs:22`, `rt_guard.rs:24`), so the null default
  keeps deterministic tests on the number they already use.
  Errors: `UnknownDevice`, `EnumerationFailed`, `NoDefaultDevice`.
- `stream_errors` exists because `CpalStream::error_count`
  (`cpal_backend.rs:163–168`) is an inherent method on the concrete type and is
  therefore **unreachable through `Box<dyn AudioStream>`**, which is what the app holds.
  The cpal error callback already does the RT-safe thing — a single
  `fetch_add` with no formatting (`cpal_backend.rs:139–141`) — so this is a relaxed
  atomic load on the app thread. It is a required method, not a defaulted one, so a
  future backend must answer rather than silently reporting zero. `NullStream` returns
  `0`. No error case; no auth; no pagination.

**New — `crates/spectre-app/src/engine.rs`, app-thread functions.**

```rust
// Build the render-side halves from a validated app snapshot; touches no device
pub fn build_engine_parts(
    snapshot: &[spectre_dsp::DeviceParameterSnapshot],
    config: spectre_audio::StreamConfig,
) -> Result<EngineParts, EngineUnavailable>;

// Open and start the default output device, consuming the parts' bridge
pub fn open_default(
    backend: &dyn spectre_audio::AudioBackend,
    snapshot: &[spectre_dsp::DeviceParameterSnapshot],
) -> Result<LiveEngine, EngineUnavailable>;

impl<S: spectre_audio::AudioStream + ?Sized> LiveEngine<S> {
    // Adopt an already-open, already-started stream and the app-thread halves of its parts.
    // The bridge is not passed: the caller must already have moved it into the render closure
    // at open, which is what erases it from the app thread. Boxing is the caller's, so a test
    // may pass Box<NullStream> and keep the concrete type.
    pub fn from_open_stream(
        stream: Box<S>,
        sender: spectre_audio::control::ControlSender,
        telemetry: std::sync::Arc<spectre_audio::bridge::BridgeTelemetry>,
        backend_name: &'static str,
        device_name: String,
        config: spectre_audio::StreamConfig,
    ) -> Self;
    // Borrow the stream for callers that must drive it explicitly. The app never calls this;
    // it exists so a test holding LiveEngine<NullStream> can pump between state() reads
    pub fn stream_mut(&mut self) -> &mut S;
    pub fn state(&self) -> EngineState;
    pub fn health(&self) -> EngineHealth;
    pub fn config(&self) -> spectre_audio::StreamConfig;
    // Queue a transport command; Err leaves the caller's UI state unchanged
    pub fn send_transport(&mut self, command: spectre_core::TransportCommand)
        -> Result<(), spectre_audio::control::ControlError>;
    // Queue one note event, stamped with frame_offset 0 and the next sequence
    pub fn send_note(&mut self, kind: spectre_dsp::NoteEventKind)
        -> Result<(), spectre_audio::control::ControlError>;
    // Play the audition voice: Play command, then one held note-on. Transport first, and
    // the note is not attempted if the transport send is refused
    pub fn start_audition(&mut self) -> Result<(), AuditionError>;
    // Stop it: all-notes-off, then Stop. Note first, so the release lands on the same block
    pub fn stop_audition(&mut self) -> Result<(), AuditionError>;
    // Drop retired render state on the app thread; called once per UI frame
    pub fn reclaim(&mut self) -> usize;
    pub fn close(&mut self) -> Result<(), spectre_audio::BackendError>;
}
```

Splitting `build_engine_parts` from `open_default` is what makes the feature testable
without hardware: the test builds the parts, destructures them, moves `bridge` into the
render closure, and opens a concrete `NullStream` via `NullBackend::open_null_output`
(`null.rs:38`) so it can call `pump()` (`null.rs:104`) and `last_block()`
(`null.rs:118`) — the same seam every R3 test uses. `from_open_stream` then wraps that
concrete stream without erasing it, which is what lets §5.1 test 9 read `state()` across
a pump.

**Note ordering — what `send_note` does and does not guarantee.**

`send_note` stamps `frame_offset: 0` and a monotonically increasing `sequence` so the
caller never assembles the ordering key by hand. That is a convenience, **not** an
ordering guarantee, and iteration 1 of this spec wrongly claimed it was one.

`ProcessContext::new` (`crates/spectre-dsp/src/io.rs:91–101`) requires the key
`(event.frame_offset, event.kind.rank(), event.sequence)` — the tuple is built at
`io.rs:96` — to be **strictly increasing** across a batch, rejecting anything else with
`ProcessError::UnsortedEvents` (`io.rs:97–99`). `rank` returns 0 for `Off`/`AllNotesOff`
and 1 for `On` (`io.rs:152–157`). With every event stamped at `frame_offset: 0`, emission
order and key order therefore disagree in exactly one case: **a Play and a Stop that land
in the same block.** Those events are keyed `(0, 1, N)` then `(0, 0, N+1)` — strictly
decreasing — and the batch is refused. The reverse pairing (Stop then Play in one block)
keys `(0, 0, N)` then `(0, 1, N+1)` and is accepted, as is any single event.

What actually happens on the refusal, read out of the shipped code rather than assumed:
`CompiledPlan::process` wraps the error as `PlanError::Process` and returns before the
note node's processor runs (`crates/spectre-graph/src/lib.rs:487–492`);
`RenderBridge::render` fills exact silence and increments `plan_errors`
(`bridge.rs:192–200`); the next block's `collect_notes` clears the scratch
(`bridge.rs:258`), so both events are discarded together. Because the discarded pair is
one note-on and its own release, the instrument's state is unchanged and **no note is
left stuck** — the audible consequence is that a Play cancelled by a Stop inside one
block period produces no sound, counted once in `plan_errors`. The transport commands are
unaffected: `apply_transport` runs before `collect_notes` (`bridge.rs:175–176`), so the
render transport still receives Play then Stop.

Exposure window: both presses must land inside one block period — 5.33 ms at 256 frames /
48 kHz. The outcome is fail-closed (exact silence, counted, never stale audio) and it is
surfaced rather than hidden: §3.6 E10 and the §3.3 item 8 counter cluster. This spec makes
no claim that the app cannot construct an unsorted batch, because it can. Eliminating the
case would mean stamping a nonzero `frame_offset` on the second event of a same-block
pair, which is a behavior change to the send path and is routed to §8 Q9 rather than
asserted here.

Auth, permissions, pagination, rate limiting: **N/A — this is an in-process desktop
feature with no network or multi-user surface.**

### 4.4 State management

| State | Owner | Thread | Lifetime |
|---|---|---|---|
| Lens, selection, tracks, device parameter values | `AppModel` (`lib.rs:205`) | app | process |
| UI transport truth | `AppModel.transport` | app | process |
| Backend, stream, control sender, telemetry handle | `LiveEngine` | app | until `close()`/drop |
| Compiled plan, note scratch, render transport | `RenderBridge`, moved into the callback closure | render | until the stream drops |
| Retired render state | reclaim lane, drained by `ControlSender::reclaim` | app drains | per frame |

`LiveEngine` is owned by `SpectrePrototype` (the `eframe::App` implementor) as an
`Option`, not by `AppModel`. That keeps `AppModel` renderer-neutral and audio-free, and
it keeps the non-`Send` stream on the UI thread where it was created.

**Two transports now exist** and this is the one genuinely new correctness hazard.
`AppModel.transport` is authoritative for the UI; `RenderBridge.transport`
(`bridge.rs:127`) is derived from commands and is unreadable from the app once the
bridge moves into the closure. They diverge if a command is refused. The binding rule:

> The app sends first and mutates second. `toggle_play` becomes: call
> `LiveEngine::start_audition` / `stop_audition`, and apply the transport change to
> `AppModel` **only if the transport send was accepted**. Concretely:
>
> - `Ok(())` — both sends accepted; the UI flips.
> - `Err(AuditionError::Transport(..))` — no transport command was queued, so the UI is
>   unchanged and E7 is surfaced. In `start_audition` the note send is not attempted at
>   all; in `stop_audition` the release may already have been queued, which is harmless —
>   a release with no matching attack is a no-op at the instrument
>   (`crates/spectre-dsp/src/source.rs:194`).
> - `Err(AuditionError::Note(..))` — the transport command **was** queued and the render
>   thread will apply it, so the UI flips anyway and E7b is surfaced. Gating the UI on
>   the note send here would be the worse error: it would leave the UI reading "stopped"
>   while the render transport is playing, which is the divergence this rule exists to
>   prevent. A queued command cannot be recalled — `spsc.rs`'s ring has no un-push — so
>   the honest UI is the one that matches what the render thread will see.
>
> `stop_audition` sends the note first and then the Stop command, and it sends Stop even
> if the note was refused: a refused release plus a stopped transport is recoverable,
> a running transport the UI thinks is stopped is not. If both of its sends are refused
> it returns `Transport(..)`, because the unchanged UI is the consequence that matters.
>
> When no engine exists, `AppModel` mutates exactly as it does today, so the prototype
> keeps working with no device.

Local vs. server-synced state: **N/A — Spectre has no server, and cloud services are a
vision non-goal.**

Offline / draft persistence: **N/A for this slice.** Engine state is deliberately not
persisted; a chosen device, a remembered format, and a restore-on-launch policy are all
project-persistence questions that R4-7 owns and that need a decision row before
anything is written to disk.

### 4.5 Dependencies

- **New crate dependencies:** none outside the workspace. `spectre-app` gains
  `spectre-audio` and `spectre-graph`, both already workspace members;
  `spectre-audio` already depends on `cpal 0.15.3`, adopted and licensed under
  decision 19.
- **New dev-dependency:** `spectre-offline` on `spectre-app`, for the equivalence test.
  `spectre-offline` already dev-depends on `spectre-app`, so this creates a
  dev-dependency cycle. Cargo permits cycles that run entirely through
  dev-dependencies, because lib targets never build against them — but this must be
  **verified at implementation** with `cargo metadata`; if Cargo refuses, host the
  equivalence test in `crates/spectre-offline/tests/` instead, which already has the
  `spectre-app` edge.
- **New feature:** `spectre-app/live-audio`, default-on, forwarding
  `spectre-audio/cpal-backend`. Mirrors the existing default-on posture in
  `crates/spectre-audio/Cargo.toml` so the real backend is always type-checked and
  linted.
- **New assets or resources:** none. No fonts, images, samples, or presets.
- **Infrastructure:** none. CI already installs `libasound2-dev`
  (`docs/status/NEXT.md` closed-R3 item 2), which is what the Linux build needs.

### 4.6 Platform-specific considerations

- **Decision 1 makes macOS and Linux co-first-class, and R4-1 ships one code path for
  both.** The backend is selected through the `AudioBackend` trait; nothing in
  `engine.rs` names cpal, CoreAudio, or ALSA.
- **macOS:** qualified. `docs/06-plans/current-milestone.md`'s hardware table records
  cpal/CoreAudio with an M-Audio AIR 192|6, 173 blocks, 0 xruns, 0.990 worst headroom,
  0 plan errors, 0 contaminated nodes, on 2026-08-09.
- **Linux:** **not qualified, and R4-1 authorizes no Linux claim.** Decision 23 carries
  the debt into R4 and R4-3 discharges it. What is established is a build-and-link
  result only: on 2026-08-09 the workspace compiled against ALSA in a Linux aarch64
  container and all 34 non-hardware `spectre-audio` tests passed, and the drill failed
  closed on a machine with no device. No Linux audio device has ever been opened.
  Concretely, the risks R4-1 exposes on ALSA are (a) `BufferSize::Fixed(256)` may be
  refused, and (b) the default rate may be 44 100 — which is why
  `default_sample_rate` exists rather than a hard-coded 48 000. Both surface as E5 and
  are §8 Q4.
- **Channel count:** v1 is stereo-only. `OUTPUT_CHANNELS: u16 = 2`
  (`crates/spectre-audio/src/lib.rs:22`) and the graph's buses are stereo
  (`crates/spectre-graph/src/lib.rs:12`). A device that cannot present two channels
  fails to open and reports E4/E5. Mono and multichannel devices are out of R4 scope.
- **Version compatibility:** eframe is pinned at `0.32.3`, and the non-`Send` question is
  settled rather than deferred. `Box<dyn AudioStream>` is not `Send`
  (`crates/spectre-audio/src/lib.rs:181` declares the trait with no `Send` bound, and
  `CpalStream` is documented thread-affine at `cpal_backend.rs:154`), so `LiveEngine` is
  not `Send`. That is fine here: eframe 0.32.3 declares `pub trait App` with no `Send`
  supertrait (`eframe-0.32.3/src/epi.rs:137`) and `AppCreator<'app>` boxes
  `Box<dyn 'app + App>` with no `Send` bound (`epi.rs:48–49`), so a non-`Send`
  `LiveEngine` can be an ordinary field of the `eframe::App` implementor. No fallback
  owner is needed. (This remains a `cargo check` item only in the ordinary sense that
  every declared signature is.)
- **Feature flags / gradual rollout:** the `live-audio` feature is the rollout control.
  Disabling it produces E1 — an app that runs and says why it is silent.

### 4.7 Performance budget

All figures are computed from the source, not estimated from another product.

- **Memory (steady state, added):**
  - Plan channel pool: the fixture has three nodes with two output channels each, so
    `compile` allocates 6 × `max_frames` f32 (`spectre-graph/src/lib.rs:324`). At
    `max_frames = 512` that is 12,288 B; the margin over a 256-frame block costs
    6,144 B of it.
  - Note scratch: `DEFAULT_NOTE_SCRATCH = 256` events (`bridge.rs:20`), on the order of
    8 KB at `size_of::<NoteEvent>()`.
  - Control lanes: `DEFAULT_NOTE_CAPACITY = 1_024` and
    `DEFAULT_TRANSPORT_CAPACITY = 64` (`control.rs:17–18`), on the order of 32 KB and
    1 KB respectively; the reclaim lane is 32 slots.
  - Total added resident memory is well under 64 KB, dominated by the note lane. No
    per-frame allocation is added on either thread; `EngineHealth` is `Copy` and is
    built on the stack.
- **CPU / render time:** the render cost is unchanged, because the plan and the bridge
  are unchanged. The one measurement Spectre owns is 0.990 worst-case headroom at
  256 frames / 48 kHz on the qualified macOS device — roughly 1% of the block budget
  for this three-node chain. No second measurement exists, so no budget threshold is
  asserted (decision 16).
- **UI cost:** one `EngineHealth` read per frame — nine relaxed atomic loads at the
  existing 250 ms repaint cadence. Negligible and lock-free.
- **Network payload:** N/A — no network I/O exists anywhere in this feature.
- **Storage (client + server):** N/A — nothing is written to disk.
- **Startup time:** one device enumeration plus one stream open, both synchronous on the
  UI thread. They run on the **first `update()` call rather than inside `main()`**, so a
  slow or hanging driver cannot prevent the window from appearing; the user sees a
  window that says `starting` instead of a hung process. The cost is that one UI frame
  may be long. Moving the open off-thread is not available: `AudioStream` is not `Send`
  (§4.6), so the stream cannot be constructed on a worker and moved back. §8 Q2.

---

## 5. Test Specification

Every command below is real and names a real target. All of them run today except
`cargo test -p spectre-app --test live_engine`, whose test file this slice creates
(§7.2); it runs from the moment that file lands. The workspace gate is exactly:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

Feature-scoped commands:

```sh
cargo test -p spectre-app --test live_engine
cargo test -p spectre-app --test smoke_cli
cargo test -p spectre-audio --test rt_guard
cargo test -p spectre-audio --test bridge_plan
./spectre --smoke-test
# hardware, run explicitly; macOS re-run confirms no regression, Linux is R4-3's debt
cargo test -p spectre-audio --test lifecycle_health -- --ignored --nocapture
```

### 5.1 Unit tests

New file `crates/spectre-app/tests/live_engine.rs` unless noted.

**Which tests hold a `LiveEngine` and which hold bare parts.** The distinction is load
bearing, because `NullStream::pump` is inherent to the concrete type (`null.rs:104`) and
is not on the `AudioStream` trait (`spectre-audio/src/lib.rs:181–196`). Tests 1, 2, 3, and
6 use `build_engine_parts` output alone and open no stream at all. Tests 4, 5, 10, and 11
destructure the parts, move `bridge` into the render closure, open a concrete `NullStream`
via `NullBackend::open_null_output` (`null.rs:38`), and drive it directly, reading counters
from the `parts.telemetry` handle they kept. Tests 7, 9, and 14 hold a
`LiveEngine<NullStream>` built with `from_open_stream` over that same concrete stream,
because each asserts on a `LiveEngine` method; 9 and 14 pump through `stream_mut()`.
Test 8 needs neither — it only calls `open_default` against a failing backend double. No
test needs `LiveEngine<dyn AudioStream>`; the app is its only consumer.

1. **`parts_build_from_the_prototype_snapshot_without_touching_a_device`**
   Setup: `AppModel::prototype().device_parameter_snapshot()?`, a validated
   `StreamConfig::stereo(48_000, ENGINE_BUFFER_FRAMES)`. Assert `build_engine_parts`
   returns `Ok` and no backend method was called (the function takes no backend, which
   is the structural proof). Edge case: construction order — the plan must exist before
   a device is opened, so an open failure cannot leave a half-built engine.

2. **`an_incomplete_snapshot_is_refused_before_any_plan_is_built`**
   Setup: a snapshot with three of the four canonical entries. Assert
   `Err(EngineUnavailable::Snapshot(..))`. Edge case: the fail-closed posture the
   offline path already uses is preserved on the live path.

3. **`the_plan_reserves_more_frames_than_the_requested_block`**
   Assert `parts.plan_max_frames == ENGINE_BUFFER_FRAMES * ENGINE_PLAN_FRAME_MARGIN`.
   The assertion reads `EngineParts`'s own field (§4.2), **not** the bridge: `RenderBridge`
   keeps `plan` private and exposes only `new`, `telemetry`, `transport`, and `render`
   (`bridge.rs:133/152/157/162`), and this slice adds no accessor to `bridge.rs`. The
   field's value is the `max_frames` argument `build_engine_parts` passed to
   `EditableGraph::compile` (`spectre-graph/src/lib.rs:196–201`), so the assertion still
   fails if a future edit drops the margin. Edge case: the oversized-block hazard in
   §4.2's rationale.

4. **`play_produces_nonzero_output_and_stop_returns_exact_silence`** — the core test.
   Setup: build parts, open a `NullStream` via `NullBackend::open_null_output`, start.
   Pump once with no note: assert `last_block()` is all zeros. Call the Play sequence,
   pump: assert the block's peak is `> 0.0`. Call the Stop sequence, pump twice: assert
   the second block is exactly zero in every sample. Edge case: this is the assertion
   that would fail if Play were wired to nothing, if the note lane were dropped, or if
   `AllNotesOff` stopped reaching the instrument.

5. **`the_live_plan_matches_the_offline_render_of_the_same_app_snapshot`**
   Setup: same snapshot on both sides; send `spectre_offline::fixture_events(256)`
   through the control sender; pump one 256-frame block. Hash the interleaved output
   with the identical FNV-1a walk already used by
   `crates/spectre-offline/src/lib.rs`'s `render_plan` and
   `crates/spectre-audio/tests/bridge_plan.rs`'s `hash_interleaved` (offset basis
   `0xcbf2_9ce4_8422_2325`, prime `0x0000_0100_0000_01b3`), copied verbatim as those two
   already are — **no new comparison method is introduced**. Assert the hash equals
   `spectre_offline::render_app_snapshot(48_000.0, 256, &snapshot)?.hash`, and assert
   that report's `peak > 0.0` so a match cannot be two silent buffers agreeing. Edge
   case: 1D determinism, and the guarantee that R4-1 introduced no second render path.

6. **`repeated_engine_builds_render_identically`**
   Build and pump the same sequence three times; assert all three hashes are equal.
   Edge case: hidden nondeterminism in plan construction from the snapshot.

7. **`a_refused_transport_send_leaves_the_ui_transport_unchanged`**
   Setup: hold a `LiveEngine<NullStream>` and call `send_transport` past
   `DEFAULT_TRANSPORT_CAPACITY` (64, `control.rs:18`) without pumping, so the lane is
   full, then call `start_audition()`. Assert
   `Err(AuditionError::Transport(ControlError::TransportLaneFull))` — the `Transport`
   variant specifically, since that is what §4.4 makes the UI gate — and that
   `AppModel::is_playing()` is unchanged. Edge case: E7, the two-transport divergence
   hazard.

8. **`engine_open_failure_reports_the_backend_error_and_leaves_the_model_intact`**
   Setup: a test double implementing `AudioBackend` that returns
   `BackendError::NoDefaultDevice`. Assert `EngineUnavailable::Backend(NoDefaultDevice)`
   and that lens, selected track, and selected device are unchanged. Edge case: the
   loop-first rule that an engine event never disturbs selection context.

9. **`opened_but_silent_never_reports_running`**
   Build the parts, move `bridge` into the render closure, open a concrete `NullStream`
   via `NullBackend::open_null_output` (`null.rs:38`), `start()` it, and wrap it with
   `LiveEngine::from_open_stream(Box::new(stream), parts.sender, parts.telemetry, …)`,
   giving a `LiveEngine<NullStream>`. Before any pump, assert `engine.state()` is
   `EngineState::Opened`. Then `engine.stream_mut().pump()?` once and assert it is
   `Running`. Edge case: the anti-fake-surface rule from §3.2 step 4. This test fails if
   `state()` is derived from `AudioStream::state()` alone — which is exactly why the
   engine is generic over its stream (§4.2): an erased `Box<dyn AudioStream>` could not be
   pumped here and the assertion could not be written.

10. **`health_mirrors_the_render_thread_counters`**
    Pump 8 blocks; assert `blocks_rendered == 8`, `xruns == 0`,
    `worst_headroom` finite and `<= last_headroom`, `plan_errors == 0`,
    `frame_capacity_rejections == 0`, `contaminated_nodes == 0`. Edge case: telemetry
    plumbed to the wrong field, or a counter read as a constant.

11. **`the_engine_callback_allocates_nothing`** (RT-001 guard, in the same file)
    Install a guarding global allocator with the thread-local RT-section pattern from
    `crates/spectre-audio/tests/rt_guard.rs:38–64` — it must be re-declared because a
    `#[global_allocator]` is per-test-binary — with a positive control that
    deliberately allocates inside a section and asserts the guard fires. Wrap
    `stream.pump()` for 16 blocks and assert zero violations. Edge case: this is the
    test that would fail if app wiring smuggled an allocating closure onto the callback
    path, which is the single most likely way R4-1 could break RT-001.

12. **`AudioStream::stream_errors` default and cpal paths** (in
    `crates/spectre-audio/tests/backend_seam.rs`): assert `NullStream::stream_errors()`
    is 0 after pumps, and that the trait method compiles behind `Box<dyn AudioStream>`.
    Edge case: the accessor being unreachable through the trait object, which is the
    defect it exists to fix.

13. **`default_sample_rate` for the null backend** (same file): assert `48_000` for the
    synthetic device and `Err(UnknownDevice)` for any other key. Edge case: silent
    fallback to a default on an unknown device.

14. **`play_then_stop_inside_one_block_is_refused_into_counted_silence`** (back in
    `live_engine.rs`) — pins §4.3's note-ordering behavior so it cannot drift into either
    a silent regression or a false guarantee. Hold a `LiveEngine<NullStream>` as test 9
    does. Call `start_audition()` and then `stop_audition()` **with no pump between
    them**, so both note events sit in the lane for one block. Pump once and assert three
    things: the rendered block is exactly zero in every sample; `health().plan_errors == 1`;
    and `health().blocks_rendered` did **not** advance for that block, because
    `RenderBridge::render` returns before its increment on the error path
    (`bridge.rs:192–200` vs `:204–206`). Then pump a second time with no further sends and
    assert `plan_errors` is still 1 and the block is still exactly zero — the discarded
    pair left no stuck note. Edge case: this test fails both if the ordering hazard is
    silently swallowed without counting and if a future change makes the batch succeed
    while the spec still describes it as refused.

### 5.2 Integration tests

- **`cargo test -p spectre-app --test live_engine`** — tests 1–11 and 14 above are already
  integration-level: they drive the real `CompiledPlan` through a real `AudioStream`
  implementation, not a mock of the render path.
- **`cargo test -p spectre-audio --test rt_guard`** must continue to pass, and this is a
  **required gate rather than a formality**, because R4-1 modifies one of the four
  modules that test scans. `rt_modules_contain_no_blocking_primitives`
  (`crates/spectre-audio/tests/rt_guard.rs:289–320`) reads `src/bridge.rs`,
  `src/control.rs`, `src/spsc.rs`, and `src/null.rs` and fails if any contains `Mutex`,
  `RwLock`, `Condvar`, `thread::sleep`, `println!`, `eprintln!`, or `dbg!`. The two
  methods R4-1 adds to `null.rs` return constants and name none of the seven, so the
  test must pass **after** the edit — running it post-edit is the evidence that §4.1's
  argument holds. `plan_process_is_rt_clean_through_the_null_stream`
  (`rt_guard.rs:258–287`), which wraps `NullStream::pump` in the allocation guard, must
  also pass unchanged, since `null.rs` is the module R4-1 touches.
- **`cargo test -p spectre-audio --test bridge_plan`** must continue to pass unchanged,
  proving the bridge's behavior was not altered to make the app work.
- **Hardware drill:** `cargo test -p spectre-audio --test lifecycle_health --
  --ignored --nocapture` re-run on macOS. R4-1 additionally requires the drill's
  printed line to include `frame_capacity_rejections`, because the existing record
  (`current-milestone.md` hardware table) does not carry that counter, so
  larger-than-requested blocks are currently unproven-absent rather than known-absent.
  The Linux run is R4-3's, and until it happens **no Linux result may be recorded in
  the qualification table**.

### 5.3 UI / E2E tests

**There is no automated GUI-driving harness in this repository** — `crates/spectre-app/tests/`
contains exactly `app_model.rs` and `smoke_cli.rs` — and R4-1 does not add one.
Automated UI scenarios are R4-9's scope (`e2e-and-qa`). What R4-1 does provide at the
end-to-end level:

- **`cargo test -p spectre-app --test smoke_cli`**, extended: the smoke line gains an
  `engine=` field and the test asserts `engine=not-started`. This assertion fails if
  anyone wires engine startup into the headless path, which is exactly the regression
  that would break CI on a device-less container. Existing assertions
  (`lens=Arrange`, `tracks=1`, `selected_device=Pulse(pulse)`) stay.
- **`./spectre --smoke-test`** must exit 0 on a machine with no audio device.

### 5.4 Visual / manual verification

Run `./spectre` **with system volume low** — the audition voice is a held saw with no
amplitude envelope and it will click at note edges.

| Configuration | What to check |
|---|---|
| Theme variants | **N/A — the shell hard-codes a single dark palette** (`main.rs:37` sets `dark_mode = true` and every color is a fixed `Color32` constant at `main.rs:9–15`). No light variant exists to check; introducing one is not this slice's work. |
| Text size extremes | At large egui zoom the transport bar's fixed 62 px height must not clip the Play button; the device name truncates, the state word and counters do not. |
| Screen size extremes | At the 1060×680 minimum (`main.rs:508`) and at 1420×860 default: the engine cluster stays inside the panel. |
| Empty vs. populated | Four resting states must each be produced and read: `Unavailable` (unplug/disable the default device before launch), `Opened` (observable only if the driver is slow to call back), `Running`, and the smoke-mode `NotAttempted`. |
| Audio behavior | Silence at launch; sound within one block of Play; exact silence after Stop; no sound at all when the engine is `Unavailable`. |
| Honesty check | With sound playing, move a Shape slider: the sound must not change, and the Shape copy must say why (R4-2). With sound playing, confirm the position readout shows `—` rather than a frozen `001 · 01 · 000`. |
| Failure recovery | Unplug the interface mid-playback: the app must not crash; `stream errors` appears or block counters stall; `Retry engine` recovers on a reconnected device. |

---

## 6. Compliance & Safety Gate

### 6.1 Sensitive data classification

- [x] **No sensitive data involvement.** The feature reads an OS-reported output device
  name and renders it in the UI. Nothing is transmitted, logged to disk, or persisted;
  there is no network path anywhere in `spectre-app` or `spectre-audio`. The device name
  lives in memory for the lifetime of the engine and is dropped with it.

### 6.2 Asset provenance

- [x] **No third-party assets.** No fonts, images, samples, wavetables, presets, or data
  files are added. The only third-party *code* involved is `cpal 0.15.3`, already a
  dependency of `spectre-audio` and already dispositioned by decision 19, which records
  its dual MIT/Apache licensing as matching this workspace. R4-1 adds no new crate.
  All DSP, graph, transport, and UI code involved is original workspace code.

### 6.3 Language / claims audit

- [ ] Makes claims not supported by evidence — **no.** The two claims this spec makes
  about live behavior are (a) macOS hardware qualification, cited to the milestone's
  hardware table, and (b) that no Linux device has ever been opened, cited to decision
  23. The spec explicitly refuses to describe Linux as working.
- [ ] Promises capabilities not yet built — **no.** The spec states in §1.2 and §7.1
  that `./spectre` currently makes no sound, that live parameter edits will still not
  work after this slice (R4-2), that the transport does not gate the render path, that
  the tempo and position readouts are literals, and that the audition voice clicks. It
  also states, in §4.3 and §3.6 E10, the one ordering case the send path does **not**
  make impossible — a Play and a Stop inside one block period, refused into counted
  silence — rather than claiming a guarantee it does not have.
- [ ] Uses language restricted by domain regulations — **N/A.** No regulated domain
  (medical, financial, legal) is involved.

One user-visible-copy rule is normative: **the word "running" may only appear when
`blocks_rendered > 0`.** A stream that opened and never called back is not running, and
the UI must not say it is.

### 6.4 Regulatory alignment

Confirmation against `gauntlet-output/criteria.md` Lens 3:

- **3A milestone fit.** R4-1 is slice 1 of the accepted R4 queue (`docs/status/NEXT.md`)
  and the first leaf of the confirmed feature tree. It adds no track model, no clip, no
  device, and no persistence; each of those is named and left to its own leaf.
- **3B non-goal respect.** No CLAP/LV2/AU hosting, no plugin-format authoring, no
  cross-DAW preset or project compatibility, no cloud service or content store, no video
  scoring. Recording is explicitly *removed* from the surface (the Record button becomes
  disabled) rather than implied.
- **3C deliberately small first devices.** No device is added or modified. The existing
  `PulseInstrument`/`Gain`/`Saturator` fixture is used exactly as compiled today, and
  the spec explicitly declines to give Pulse an amplitude envelope even though that
  would make the audition prettier — that is R4-6's decision under decision 15.
- **3D originality.** No reference product's code, numbers, layout, or naming is
  transcribed. Both new constants carry Spectre-derived rationale rows (§4.2). The one
  numeric value adopted from prior art is Spectre's own prior measurement.
- **3E platform commitment.** One code path for macOS and Linux; the Linux gap is stated
  three times (§4.6, §7.1, §7.4) and no Linux claim is made. `default_sample_rate`
  exists specifically to avoid baking a macOS-shaped assumption into the app.
- **3F accessibility trajectory.** No icon-only control, no color-only state, no custom
  widget; the known eframe accessibility-feature gap and the right-to-left focus-order
  hazard are both recorded for decision 17's R4 audit rather than hidden.

---

## 7. Gap Analysis vs. Current State

### 7.1 What exists today

Verified by reading the files at commit `dae16bb`. Each row is checkable in one `Read`.

**Implemented and verified — `spectre-audio` (nothing in the app uses any of it):**

| Element | Path | Note |
|---|---|---|
| `AudioBackend`, `AudioStream`, `StreamConfig`, `RenderBlock`, `RenderCallback`, `BackendError`, `StreamState`, `DeviceInfo`, `DeviceId` | `crates/spectre-audio/src/lib.rs:31–216` | trait seam per decision 19; `OUTPUT_CHANNELS = 2` at line 22; bounds at lines 25–28 |
| `NullBackend`, `NullStream` with explicit `pump`, `last_block`, `blocks_rendered` | `crates/spectre-audio/src/null.rs:18–176` (`NullBackend` at :18–19, `open_null_output` at :38, `NullStream` at :80, `pump` at :104, `last_block` at :118) | deterministic, hardware-free |
| `CpalBackend`, `CpalStream` behind default-on `cpal-backend` | `crates/spectre-audio/src/cpal_backend.rs`; feature in `crates/spectre-audio/Cargo.toml` | only file naming cpal types; `BufferSize::Fixed` at line 129; error callback counts only, lines 139–141 |
| RT-002 split-lane transport: `control_channel`, `ControlSender`, `ControlReceiver`, latest-wins `ParameterWriter`/`ParameterReader`, strict-FIFO note/transport lanes, reclaim lane | `crates/spectre-audio/src/control.rs` | decision 21; `retire` hands values back rather than dropping them, lines 314–324 |
| Bounded wait-free SPSC ring | `crates/spectre-audio/src/spsc.rs` | `push` returns the rejected value |
| `RenderBridge` and `BridgeTelemetry` | `crates/spectre-audio/src/bridge.rs` | drives the existing plan; telemetry at lines 24–116 |
| Timestamped MIDI ingress | `crates/spectre-audio/src/midi.rs` | reuses `NoteEventKind::rank` rather than restating it |
| RT-001 allocation guard, positive control, structural lock scan, `Send` assertions | `crates/spectre-audio/tests/rt_guard.rs` | `Send` asserted for `CompiledPlan`, `RenderBridge`, both control halves, lines 30–36; module scan list `RT_MODULES` at lines 293–298, whose four entries at 294–297 are `src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, `src/null.rs` — **`midi.rs` is not scanned**; forbidden-primitive list `FORBIDDEN` at 299–307 |
| Bridge↔offline hash equivalence, oversized-block refusal, transport ordering, note deferral | `crates/spectre-audio/tests/bridge_plan.rs` | |
| Lifecycle drill: the deterministic half (`crates/spectre-audio/tests/lifecycle_health.rs:68–240`, six tests) and the `#[ignore]`d `hardware_lifecycle_drill` (`:245–295`) | `crates/spectre-audio/tests/lifecycle_health.rs` | the hardware drill is `#[cfg(feature = "cpal-backend")]` as well as `#[ignore]`, `:246–247` |

**Implemented — graph and DSP:**

- `EditableGraph` / `CompiledPlan` GRAPH-001 split, validated compilation, implicit-cycle
  rejection, preallocated channel pool, RT-003 containment inside `process`:
  `crates/spectre-graph/src/lib.rs` (`contain_channel` at 401–414; `process` at
  456–533; `last_output` at 536).
- `PulseInstrument` (monophonic, note-driven, handles `AllNotesOff` at
  `crates/spectre-dsp/src/source.rs:194`, **no amplitude envelope** — lines 202–209),
  `Gain`, `Saturator`, `ToneSource`; `AudioProcessor: Send` at
  `crates/spectre-dsp/src/io.rs:163`; ordering key `rank` at `io.rs:152`.
- Offline harness: `fixture_events`, `render_vertical_slice`, `render_app_snapshot`,
  `render_silence` in `crates/spectre-offline/src/lib.rs:178–334`.

**Prototyped — the app shell.** `crates/spectre-app/src/main.rs`'s own header calls it
an "interaction prototype" and states "audio and persistence wiring remain out of
scope." `AppModel` (`crates/spectre-app/src/lib.rs:205`) owns lens, tracks, devices,
selection, an in-memory `Transport`, and the validated four-entry
`device_parameter_snapshot` (lines 348–383).

**Absent — this is the gap R4-1 closes:**

- **`spectre-app` does not depend on `spectre-audio` or `spectre-graph`.**
  `crates/spectre-app/Cargo.toml`'s `[dependencies]` are exactly `eframe`,
  `spectre-core`, `spectre-dsp`, `spectre-project`, `serde`, `serde_json`.
- **`./spectre` produces no sound.** The launcher runs `cargo run -p spectre-app`
  (`spectre`, line 9). `AppModel::toggle_play` (`lib.rs:268–275`) applies a
  `TransportCommand` to an in-memory `Transport` and returns. There is no audio thread,
  no stream, no bridge, and no plan anywhere in the app.
- The transport bar's `ENGINE OFFLINE` (`main.rs:79`) and `CPU —` (`main.rs:80`) are
  hard-coded strings. `120.00 BPM`, `4 / 4`, and `001 · 01 · 000` (`main.rs:75–77`) are
  hard-coded literals not derived from any model.
- The Build lens states "live audio routing is not connected yet" (`main.rs:330`) —
  currently true.
- **No transport gate exists in the render path.** `RenderBridge::render`
  (`bridge.rs:162–208`) executes the plan on every block regardless of
  `self.transport`; transport commands are applied (line 175) and readable via
  `transport()` (line 157) but do not gate execution.
- **No way to query a device's format through the seam.** `DeviceInfo` carries no
  sample rate or buffer range (`lib.rs:47–53`), and `AudioBackend` has no config-query
  method (`lib.rs:199–216`).
- **No way to read stream errors through the seam.** `error_count` is inherent to
  `CpalStream` (`cpal_backend.rs:163–168`), not on the `AudioStream` trait.
- No `engine` module, `LiveEngine`, `EngineHealth`, or any audio-facing type in
  `spectre-app`. The crate is two files: `lib.rs` and `main.rs`.

**Gated (accepted, deliberately not implemented):**

- **Decision 22 — runtime parameter seam.** Design accepted in
  `docs/03-architecture/dsp-device-io.md` §"Runtime parameter seam"; implementation is
  R4-2. The bridge counts observed changes as `parameters_pending`
  (`bridge.rs:181–186`) with a callback that discards the value. **Consequence for this
  spec: after R4-1 ships, moving a Shape slider will still not change live audio.**
- **Decision 23 — Linux device qualification.** Table row reads "not run".
- Decision 17 — accessibility audit scoped at R4, gated before beta.
- GRAPH-002 — only implicit-cycle rejection exists; priced feedback edges are gated at
  decision row 7 before R11.

### 7.2 Delta to spec

**New files**

- `crates/spectre-app/src/engine.rs` — engine host, constants, `EngineState`,
  `EngineHealth`, `EngineUnavailable`, `AuditionError`, `EngineParts`,
  `build_engine_parts`, `open_default`, `LiveEngine` with `from_open_stream`.
- `crates/spectre-app/tests/live_engine.rs` — tests 1–11 and 14 of §5.1, including a
  re-declared guarding global allocator with its positive control.

**Modified files**

- `crates/spectre-app/src/lib.rs` — add `pub mod engine;`. No `AppModel` field or
  method-semantics change.
- `crates/spectre-app/src/main.rs` — `Option<LiveEngine>` field; first-frame start
  attempt; send-before-mutate in the Play/Stop handler; engine status and health
  cluster; `Retry engine` button; Record button disabled with a reason; position literal
  → `—` with hover reasons on the tempo/meter/position group; Build and Shape copy
  updates; `engine=` field in the smoke line.
- `crates/spectre-app/Cargo.toml` — `spectre-audio`, `spectre-graph` deps;
  `live-audio` default feature; `spectre-offline` dev-dep.
- `crates/spectre-audio/src/lib.rs` — `AudioBackend::default_sample_rate`,
  `AudioStream::stream_errors`.
- `crates/spectre-audio/src/null.rs` — implement both.
- `crates/spectre-audio/src/cpal_backend.rs` — implement both; `error_count`'s body
  moves behind `stream_errors`.
- `crates/spectre-audio/tests/backend_seam.rs` — tests 12–13 of §5.1.
- `crates/spectre-app/tests/smoke_cli.rs` — assert `engine=not-started`.
- `crates/spectre-audio/tests/lifecycle_health.rs` — add
  `frame_capacity_rejections` to the drill's printed line and assert it is 0.
- `docs/01-requirements/requirements-ledger.md` — one rationale row each for
  `ENGINE_BUFFER_FRAMES` and `ENGINE_PLAN_FRAME_MARGIN`, carrying the §4.2 text.
  PROD-003 (`requirements-ledger.md:64`) requires every numeric limit's rationale to be
  recorded **in that ledger**, and decision 16 makes it a standing rule, so §4.2's
  rationale alone does not discharge it. Iteration 1 omitted this file and would have
  left both rows unwritten.
- `docs/status/STATUS.md`, `docs/status/NEXT.md`,
  `docs/06-plans/current-milestone.md` — update the wiring-gap and known-gaps claims
  when the slice lands, per `docs/README.md`'s working rule. Status must move to
  `implemented`, not `verified`, until the manual protocol and a hardware re-run pass.

**Deliberately not modified:** `crates/spectre-audio/src/bridge.rs`, `control.rs`,
`spsc.rs`, `midi.rs`; all of `spectre-graph`, `spectre-dsp`, `spectre-core`,
`spectre-project`, `spectre-offline`. No second render path is created and no DSP is
written.

**On the RT-scanned set specifically.** `rt_guard.rs`'s `RT_MODULES` (lines 293–298)
scans `src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, and `src/null.rs` — not
`midi.rs`. Three of those four are in the not-modified list above; the fourth,
`src/null.rs`, **is modified** by this slice, and the modified-files list says so. The
RT-001 argument therefore rests on the content of that edit and on a post-edit
`rt_guard` run (§4.1, §5.2), not on the scanned set being untouched. `midi.rs` is left
alone for a different and weaker reason: nothing in R4-1 feeds `MidiIngress`, so there
is no cause to touch it.

**Migrations / schema changes:** none. **New third-party dependencies:** none.

### 7.3 Estimated scope

**M.** Justification: roughly 250–350 new lines in `spectre-app` plus a test file of
similar size; two small app-thread trait methods with two implementations each; edits to
three existing test files, three status/plan documents, and the requirements ledger. It
is above **S** because it crosses three crates, adds a public method to two accepted
traits, and changes the transport button's mutation order (a correctness change, not a
cosmetic one). It is below **L** because it writes no DSP, adds no dependency, changes no
schema, adds no code to a callback-reachable path — the two `null.rs` methods it does add
are app-thread-only constant returns, and the render closure itself is unchanged (§4.1) —
and reuses the plan, bridge, transport, and telemetry exactly as they were qualified in
R3.

### 7.4 Blocking dependencies

- **Nothing blocks authorship or macOS implementation.** Every component R4-1 composes
  is implemented and tested.
- **R4-3 (`linux-device-qualification`) blocks any Linux claim**, not this work.
  Decision 23 is explicit: no Linux support claim is authorized until
  `cargo test -p spectre-audio --test lifecycle_health -- --ignored --nocapture` runs on
  real Linux hardware. R4-1 must not add a Linux row to the qualification table.
- **R4-2 (`runtime-parameter-seam`) is blocked *by* R4-1's observability**, not the
  reverse — but R4-1 must ship the Shape copy that tells the truth about it in the
  meantime.
- **R4-5 (`midi-clips`) will replace the audition voice.** The held note this slice
  sends on Play is scaffolding standing in for clip playback; when clips exist, Play
  must play a clip and the audition voice must be removed, not accumulated alongside it.
- **External gates:** an output device on the developer's machine for manual
  verification, and a Linux box with real audio for R4-3.

---

## 8. Open Questions

- **Q1 — Should a stopped transport stop plan execution?** This spec keeps
  `RenderBridge::render` unchanged, so the plan runs every block and silence at Stop
  comes from note release, not from a gate. Gating in the bridge would make Stop
  unconditionally silent (stronger fail-closed behavior) but would cut reverb tails,
  foreclose the monitor-while-stopped behavior Live documents at
  `OBS-AB12-ROUTE-001` (Auto monitors while armed and not playing), and require editing
  eight existing R3 tests that render with a default (stopped) transport. Recommend
  deferring the gate to R4-5, when clips give it a meaning. — blocks §3.2, §4.4, R4-5.
- **Q2 — Should the device open move off the UI thread?** It cannot today:
  `AudioStream` has no `Send` bound and `CpalStream` is thread-affine, so a stream built
  on a worker cannot be moved back. Adding `Send` to `AudioStream` would be a change to
  an accepted seam and needs a decision row, not a spec assertion. — blocks §4.7.
- **Q3 — Should the fixture chain get one shared constructor?** It is now assembled in
  four places: `spectre-offline`'s `render_plan`, `bridge_plan.rs`,
  `lifecycle_health.rs`, and (with this spec) `engine.rs`. These are four builders of
  the *same* plan, not four render paths, and test 5 pins them together — but a shared
  builder would be less fragile. Placement is awkward: `spectre-offline` and
  `spectre-audio` each dev-depend on the other's neighbors. — blocks §7.2.
- **Q4 — `BufferSize::Fixed` on ALSA.** `cpal_backend.rs:129` always requests a fixed
  size. If ALSA hosts refuse it, R4-1 fails closed with E5 on Linux and the platform is
  silent until the seam also supports `BufferSize::Default`. This is unknowable without
  the hardware R4-3 needs. — blocks R4-3, §4.6.
- **Q5 — Tempo and position readout.** This spec blanks the position to `—`. The
  alternative is publishing the render thread's transport position into a telemetry
  atomic and displaying it, which is small but is the first piece of R4-5's playhead
  work. Which does Jeff want in the alpha? — blocks §3.3.
- **Q6 — Sub-feature candidate.** The two seam additions (`default_sample_rate`,
  `stream_errors`) are separable from the app wiring and could be their own slice
  ahead of it, keeping R4-1's diff confined to `spectre-app`. This spec bundles them
  because R4-1 cannot open a real device correctly without the first. — blocks §4.3.
- **Q7 — Device selection.** R4-1 opens the default device only, with no picker. A
  picker needs enumeration UI, a persisted choice, and a decision about what happens
  when the saved device is missing — all of which touch R4-7. Is default-only acceptable
  for the alpha? — blocks §3.1.
- **Q8 — Audition level and safety.** The audition reuses the fixture's values (Pulse
  level 0.3 × velocity 0.8, through Gain 0.7 and Saturator drive 2.5 / mix 0.35) because
  identical values are what make test 5's hash equality possible. Changing the level for
  hearing safety would break that equality and would need a separate rationale row.
  There is no output limiter anywhere in the chain, and none is proposed here. — blocks
  §5.4.
- **Q9 — Should `send_note` make a same-block out-of-order pair impossible?** §4.3
  documents the one case where the app's emission order disagrees with the plan's
  ordering key: a Play and a Stop inside one ~5.33 ms block, which
  `ProcessContext::new` refuses (`io.rs:97–99`) into a counted block of silence. This
  spec accepts and counts that rather than preventing it. The alternative is for
  `LiveEngine` to remember the last rank it emitted in the current block and stamp
  `frame_offset: 1` on any event whose rank would not increase — cheap, but it is a
  behavior change to the send path, it makes the audition's timing depend on hidden
  state, and it would need its own rationale row for the frame-offset constant under
  PROD-003. Deferring it is defensible because the failure is fail-closed, counted,
  displayed, and leaves no stuck note; accepting it permanently is not obviously right.
  Jeff's call. — blocks §4.3, §3.6 E10, §5.1 test 14.

---

## Appendix A — Benchmark evidence used, and where it does not exist

Cited, from the accepted corpus:

- `OBS-PP-ARCH-001` (Phase Plant): at least one output module is required to produce
  sound. Spectre converges — `EditableGraph::compile` takes an explicit output node and
  the plan contains exactly its ancestors, so "no terminal output" is silence by
  construction rather than an error state to be handled.
- `OBS-VCV-VOLT-006` (VCV Rack 2): modules should output 0 on NaN/infinity. Already
  RT-003's recorded provenance; R4-1 inherits `CompiledPlan`'s containment unchanged and
  surfaces its counters instead of hiding them.
- `OBS-AB12-ROUTE-001` (Live 12, §17.1): monitor modes In/Auto/Off, with Auto monitoring
  while armed and not playing clips — i.e. audio flows while the transport is stopped.
  Spectre follows the implication by keeping the stream open across transport changes
  rather than opening on Play and closing on Stop.
- `OBS-AB12-MIX-009` (Live 12, §18.9) and `OBS-SR2-CPU-001` (Serum 2 support article
  51): two of the five benchmarks have citable records treating CPU cost as a
  user-visible concern. Spectre **diverges** on the presentation: it publishes per-block
  callback headroom, xruns, and containment counts from the render thread rather than a
  per-track performance meter, because R4-1 has no track-level DSP attribution and
  inventing one would be a fake surface.
- `OBS-AB12-MIX-002` (Live 12, §18.1.1): a 32-bit float engine tolerates over-0 dB
  internally, and clipping matters at physical outputs. Spectre's f32 buffers plus
  RT-003 containment occupy the analogous position, and R4-1 adds **no** output limiter,
  so a hot chain clips at the device. Stated rather than implied.
- `ableton-live-observations.md` §"Cross-cutting patterns worth carrying into
  requirements work", item 1: context-scoped binding reuse is pervasive and deliberate
  in Live. This is why R4-1 assigns no binding to `Retry engine` and does not ratify the
  prototype's existing `Space`/`1`–`4` scaffolding — the command model precedes the map,
  and AF-5 forbids fixing the map at this evidence level.

**Named gaps — evidence that does not exist and was not invented:**

- **No benchmark in the corpus has a citable observation about audio-device selection,
  driver/host configuration, sample-rate or buffer-size choice, or engine start and
  failure UX.** The Ableton corpus's 85 records span ARR, SES, CLIP, WARP, LAUNCH,
  ROUTE, MIX, REC, and AUTO; there is no preferences or audio-device category. Every
  device-open, format-negotiation, and failure-surface decision in this spec is
  therefore Spectre's own, recorded as a research need rather than dressed in a citation.
- **Logic Pro: zero citable behavioral observations.** `logic-pro.md` is inventory-only.
  This spec asserts nothing about Logic Pro's engine, device handling, or CPU display.
- **Serum 2: two citable records** (`OBS-SR2-CPU-001`, `OBS-SR2-KB-001`), and its
  dossier is `blocked-source-gap`. Only the CPU record is used, only for the narrow
  claim that CPU cost is user-visible.
- **No benchmark record describes exposing per-block callback headroom or NaN
  containment counts to the user.** Spectre's decision to surface them is not claimed as
  superior on evidence; it follows from the vision's "Rust-native engine with a
  published realtime contract" and the alpha bar's "honest telemetry, no fake surfaces."

---

**End of spec.**
