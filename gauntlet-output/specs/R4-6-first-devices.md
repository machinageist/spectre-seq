<!--
Author: Jeff
Date: 2026-08-16
Description: R4-6 spec — one small original instrument (Filament) and one small original effect (Gloam) as the alpha's voice, sized by decision 15
Notes: The scope guard is the feature. Decision 15 makes these devices deliberately small, and
  `product-implications.md` §"Prohibited conclusions" forbids asserting a synthesis architecture,
  a modulation limit, or a final native-device list at the current evidence level — so this spec
  designs two devices and refuses to describe a device family. Three facts shaped it and are
  stated rather than smoothed: the shipped `Gain` has no smoothing state despite an accepted
  document saying it does (D-R3, open, not resolved here); `AudioProcessor` has no parameter
  setter today, so both devices depend on R4-2 landing first; and `./spectre` produces no sound
  of any kind, so every claim below is about offline and test-harness evidence unless it names
  R4-1 as a dependency.
  Iteration 2 (remediation 1) — four changes, all inside §5, plus the two §7.2 lines they force.
  Nothing in §0–§4, §6, §7.1, or §8 changed; no numeric bound was added, moved, or restated.
  (1) §5.3's `opening_a_new_device_focuses_shape` called `select_device`, which exists nowhere in
  `crates/` — repo-wide grep, zero hits. It now calls `AppModel::open_device_in_shape`
  (`crates/spectre-app/src/lib.rs:338`), whose `Result<(), &'static str>` return the test must
  handle, so the assertion order is now `Ok(())` first and lens/selection second.
  (2) §5.2's `existing_fixture_hash_is_unchanged` had no baseline in `harness.rs` and therefore
  could not fail; it is withdrawn, and the three hand-wired gates that already guard the
  peak/hash walk from outside `spectre-offline` are named with their lines instead. A golden
  vector is argued for and declined here, because §7.2 item 5's helper is private and an
  integration test cannot reach it; the pin belongs to R4-8, where the fold becomes public.
  (3) §5.1 test 6's unargued "within 1 ULP" over a 256-sample recursion is gone — deleted, not
  derived: at `depth = 0.0` the coefficient is bit-exactly the stored `damp_a`, so the assertion
  is now bit-equality against a reference that replicates the same `f32` arithmetic. One fewer
  number in the spec, not one more.
  (4) §5.1 test 7's poison block is now specified as entirely non-finite, which is the condition
  its state-equality assertion silently depended on.
  Every line number added above was re-opened at commit `2e005e5` before it was written.
-->

# Spec: First Devices — Filament and Gloam

**Feature ID:** `R4-6` (`first-devices`)
**Parent feature:** `R4` Credible Alpha (root)
**Spec author agent:** gauntlet spec agent, R4-6 leaf
**Date:** 2026-08-16
**Iteration:** 2 (remediation 1)

- **Status:** proposed
- **Last verified:** 2026-08-16 (source read at commit `2e005e5`, branch `rename/geist-to-spectre`)
- **Scope:** two new native devices in `spectre-dsp` — `Filament` (note-driven instrument) and `Gloam` (stereo insert effect) — their descriptors, their `AudioProcessor` implementations including R4-2's parameter setter, their numeric-bound rationale rows, their test evidence, and their appearance in the existing app device list
- **Decision authority:** Jeff
- **Upstream sources:** `docs/01-requirements/decision-gates.md` rows 1, 15, 16, 17, 22 (`:25`, `:39`, `:40`, `:41`, `:47`); `docs/01-requirements/requirements-ledger.md` RT-001/002/003 (`:28`–`:30`), CORE-002 (`:47`), GRAPH-001 (`:55`), PROD-002 (`:63`), PROD-003 (`:64`); `docs/03-architecture/dsp-device-io.md` (the contract both devices must satisfy); `docs/00-product/vision.md`; `docs/06-plans/current-milestone.md:84`; `docs/status/NEXT.md:28` (slice 6); `gauntlet-output/specs/R4-2-runtime-parameter-seam.md` (the trait method both devices implement); `gauntlet-output/specs/R4-1-live-audio-wiring.md`
- **Downstream dependents:** R4-4 (track model — a track hosts these devices), R4-5 (MIDI clips — the clip drives `Filament`), R4-7 (save/reload — these descriptors are what a project persists), R4-8 (offline bounce — live/offline equivalence is asserted over a chain containing these devices), R4-9 (e2e fixture and manual QA protocol)
- **Supersedes:** none. It does **not** supersede `ToneSource`, `PulseInstrument`, `Gain`, or `Saturator`; all four stay exactly as they are, because they are the fixture the existing hash, containment, and RT-001 evidence is built on
- **Superseded by:** none
- **Open decisions:** §8 Q1–Q9. Q1 routes the shared click-on-parameter-change question to Jeff and is logged against the already-open **D-R3** in `gauntlet-output/decisions-needed.md`, entry **D-R3**; this spec states how `Filament` and `Gloam` behave and does **not** resolve D-R3 for `Gain` by assertion. Q3 (a new `DEV` requirements-ledger family) is a proposed change to an accepted document and is routed, not asserted
- **Known gaps:** (a) the corpus has no citable record about how any benchmark implements a small first-party device internally — `serum-2.md` is `blocked-source-gap` with exactly two behavioral records, and `logic-pro.md` is `draft` / `inventory-only` (`docs/02-reference-research/logic-pro.md:10–11`) with zero, so no Logic Pro claim appears anywhere below; (b) neither device is band-limited, and the corpus contains no anti-aliasing record for any benchmark, so §4.7 states the limitation rather than a target; (c) no listening evidence exists for either device because neither exists, so §1.3's success signal is deliberately a measurable one and not a taste claim

This spec is subordinate to the conflict precedence in `docs/README.md:23–35`. Where it would
change accepted direction — a new ledger family, a decision on smoothing — it says so and
routes to §8 rather than choosing for Jeff.

---

## 0. What this spec deliberately does not do

This section is first because it is the feature. Decision 15
(`docs/01-requirements/decision-gates.md:39`) reads: *"R4 ships a deliberately small original
synth; R11 designs the flagship and modular identity from accepted requirements without
prototype-code reuse."* `docs/03-architecture/dsp-device-io.md:107` says the same thing about
the devices that already exist: *"These devices prove the contract. They do not define the
eventual flagship instrument or effect catalog."*

`docs/02-reference-research/workflow-field-study/product-implications.md:90–104` lists what the
current corpus cannot justify. Four items on that list are exactly what a device spec wants to
assert, and this spec asserts none of them:

| Prohibited conclusion (`:94`–`:100`) | What this spec does instead |
|---|---|
| a final native-device list | Names two devices for R4 and states no catalog, no naming scheme, no device families, and no count. §8 Q2. |
| a synthesis architecture or modulation limit | `Filament` has an internal structure because code must have one. That structure is **this device's implementation**, not Spectre's synthesis architecture, and this spec proposes no modulation source, no modulation routing, and therefore no modulation limit. §8 Q4. |
| a default shortcut map | §3.4 adds no binding and ratifies none. |
| a monitoring-latency threshold / gesture or time budget | §4.7 gives a per-sample operation count and no time target. |

Decision 15's smallness is also a *cost* boundary. `OBS-SR2-CPU-001`
(`docs/02-reference-research/synth-modular-observations.md:54`) is one of only two citable Serum 2
records, and it says per-voice duplication is the cost center and that a shared post-FX stage
applied once is the recommended alternative to per-voice work. **The specific voice counts in
that record are Serum's and are not adopted here** (AF-4, decision 16 at
`decision-gates.md:40`). What Spectre takes from it is the shape of the argument: one voice and
one shared insert effect is the cheapest structure that still makes a sound, and it is the
structure R4 ships.

---

## 1. Purpose

### 1.1 One-sentence job

Give the alpha a voice of its own: a musician plays notes into `Filament`, shapes them with
`Gloam`, and hears Spectre's own sound rather than a test tone.

### 1.2 Why it matters

`docs/06-plans/current-milestone.md:84` makes this an R4 exit row in one line: *"One small
original synth and one original effect ship as the alpha's voice."* `docs/status/NEXT.md:28`
is slice 6: *"Ship one small original synth and one original effect as the alpha's voice, per
decision 15's deliberately-small scope."*

What exists today is a fixture, not a voice. `PulseInstrument`
(`crates/spectre-dsp/src/source.rs:112–215`) is a monophonic oscillator with **one** parameter,
`level` (`:39–46`), and a four-way `Waveform` enum chosen at construction (`:103–109`) with no
setter of any kind — its shapes are the naive triangle, saw, and square at `:143–157`. It has no
envelope, so a note begins and ends as a hard amplitude step (`:202–209`: the sample is the
oscillator value while a note is active and exactly `0.0` otherwise). `Gain` is a single `f32`
(`crates/spectre-dsp/src/effect.rs:29–31`) and `Saturator` is a `tanh` soft clip with a dry/wet
blend (`:107–133`). That set proves the process contract, which is what it was built for
(`docs/03-architecture/dsp-device-io.md:98–107`), and it is not something anyone would play.

`docs/00-product/vision.md:30` names the eventual bar — *"a serious native synth and effect set
with visible, assignable modulation"* — and `:48` sets the alpha bar much lower and on purpose:
*"one track, MIDI clip, native synth + effect, transport, save/reload, offline bounce — honest
telemetry, no fake surfaces."* R4-6 delivers the second sentence and explicitly not the first.

**Stated plainly and kept stated until it is false: `./spectre` produces no sound of any kind
today.** `docs/status/NEXT.md:19` records it as a known gap — *"`./spectre` still does not use
`spectre-audio`, so nothing launchable makes sound"* — and R4-1, the slice that changes it, has
passed its spec gate at 2.950 with zero lines of implementation. Everything R4-6 can prove on
its own is offline and in tests; every audible claim below is conditional on R4-1 and R4-2
landing first (§7.4).

### 1.3 Success signal

**Automated, and it is the one that gates the slice:** an offline render of
`Filament → Gloam` through the existing compiled plan produces a nonzero peak, is bit-identical
across two runs of the same inputs, reports `contaminated_nodes == 0`, and returns to **exactly**
`0.0` — not a denormal — within 64 render quanta after the last note-off. That is §5.2's
`voice_chain_renders_deterministically` plus §5.1's silence and containment tests, and each
would fail if the corresponding behavior regressed.

**Observable, once R4-1 and R4-2 exist:** with the engine running, playing a note through
`Filament` is audible, and dragging `Gloam`'s `depth` slider from `0.00` to `1.00` audibly
changes the sound while the note sustains, with `params pending 0` in R4-2's transport counters.

---

## 2. User Stories

> As an electronic musician opening Spectre for the first time, I want a native instrument that
> responds to note velocity and does not click at the start and end of every note, so that the
> first thing I hear is a musical sound rather than a test signal.

> As a sound designer, I want one control on the instrument that changes its harmonic character
> continuously rather than switching between fixed waveform names, so that I can find a sound by
> ear instead of by picking from a menu.

> As a musician shaping a part, I want an effect whose behavior follows how hard I am playing, so
> that a static loop does not sound static.

> As a musician who has set the instrument's release to its shortest value, I want to know that
> the note still ends on a ramp and not on a step, so that a percussive setting does not turn
> into a click I have to fix later. (§4.2's `rise_ms`/`fall_ms` minimum is chosen for exactly
> this; §4.3 states what it does and does not guarantee, and §8 Q1 routes the wider question.)

> As a musician who has just driven the effect with a loud passage, I want the tail to end in
> real silence rather than in an inaudible residue that costs CPU forever, so that leaving a
> project open does not slowly get more expensive. (§4.3's state flush; §5.1 test 8.)

> As a keyboard-only or screen-reader user, I want both devices' controls to be the same
> descriptor-driven rows every other device uses, with names, units, ranges, and defaults read
> from the backend, so that nothing about the new devices is a special case my tools cannot reach.

> As Jeff reviewing this at R4 exit, I want every number in both devices to have a rationale row
> I can audit in `requirements-ledger.md`, so that PROD-003 is discharged by record rather than by
> a spec paragraph nobody can grep later.

---

## 3. UX Specification

### 3.1 Screen / view inventory

R4-6 introduces **no new screen, modal, sheet, popover, drawer, or panel.** Both devices appear
through the device surfaces that already exist and already read their content from backend
descriptors.

| Region | Navigation path | New or modified | Layout pattern |
|---|---|---|---|
| Build lens device list | lens selector → Build | **modified** — two entries appended to `AppModel::prototype()`'s device vector (`crates/spectre-app/src/lib.rs:228–250`) | vertical list of device frames with read-only parameter values |
| Shape lens parameter rows | Build → open a device → Shape | **modified by data only** — `Filament` contributes four rows and `Gloam` three; no widget type, layout rule, or interaction changes | scroll area, one horizontal row per parameter |
| Arrange, Mix | — | unchanged | — |

Nothing is added to the transport bar. No device-specific panel, no oscilloscope, no custom
graphic is proposed: a waveform display would be a new surface with its own repaint cost and its
own accessibility obligation, and R4 has no evidence that it is needed. §8 Q6.

The reason both devices ride the existing rows rather than getting bespoke UI is
`docs/03-architecture/dsp-device-io.md:60`: *"UI labels, units, ranges, defaults, and disabled
reasons derive from backend descriptors; UI code MUST NOT redefine DSP ranges."* Adding a device
is therefore a `spectre-dsp` change plus two lines of app data, and that is the whole UI delta.

### 3.2 Interaction flows

**Primary flow — hearing the new voice (requires R4-1 and R4-2; §7.4).**

1. The user opens Build and sees five device frames instead of three.
2. The user opens `Filament` from Build, which sets `selected_device` and switches to Shape
   (`crates/spectre-app/src/lib.rs:342–343`, the existing focus drill-in).
3. Shape draws four rows — Lean, Rise, Fall, Level — each an `egui::Slider` bounded by
   `descriptor.minimum()..=descriptor.maximum()` with the descriptor's unit, exactly as the three
   existing devices are drawn today.
4. A note arrives (R4-5's clip, or R4-1's MIDI ingress). `Filament` starts a voice: phase resets
   to `0`, the envelope begins rising toward `1.0`.
5. The user drags Lean. R4-2's path publishes the clamped value into the latest-wins parameter
   lane; the bridge applies it once, before `process`, on the next block; the harmonic character
   changes without the pitch, the phase, or the envelope moving.
6. The user opens `Gloam` and raises Depth. The damping corner now follows the signal's own
   magnitude, so louder passages open up and quiet ones sit dark.

**Branch — a second note arrives while one is sounding.** `Filament` is monophonic. The new
note-on replaces the active note and resets phase, which is precisely what `PulseInstrument`
already does (`crates/spectre-dsp/src/source.rs:181–187`). A note-off whose `id` does not match
the active note is ignored (`:188–193`, `:198`), so a released older note cannot silence a newer
one — the note-ID matching the event contract requires
(`docs/03-architecture/dsp-device-io.md:46`) is what makes that correct rather than lucky.
`AllNotesOff` clears the voice, and the envelope then falls at `fall_ms` rather than cutting.

**Branch — the release is still falling when a new note arrives.** The envelope reverses toward
`1.0` from wherever it is. It never jumps, so a fast repeated note cannot produce a step.

**Branch — a parameter changes mid-note.** `Filament`'s `level` and `Gloam`'s three parameters
are applied at the block boundary, once, before `process` — the accepted granularity
(`docs/03-architecture/dsp-device-io.md:58`, `:93`). **A large, fast `level` move will therefore
step at a block boundary and can be audible as a click**, exactly as it would on `Gain`. This
spec does not add per-parameter smoothing to either device (§4.3, "Where smoothing is and is not"),
does not claim either device is click-free under parameter motion, and routes the shared question
to §8 Q1 against the open **D-R3**. `OBS-AB12-MIX-008`
(`docs/02-reference-research/ableton-live-observations.md:105`) is the one citable benchmark
record on this surface: a benchmark documents changing a millisecond-scale track delay during a
performance as a clicks-and-pops risk and warns against it, which establishes that click risk
from a live parameter move is a real product concern in a shipping DAW — not that any particular
smoothing scheme is the answer.

**Sound, haptic, and animation cues.** None are added. The only new perceptible behavior is
audio, and §4.3 describes exactly what it is.

### 3.3 Layout descriptions

**Build lens — two appended frames, top → bottom in device order:**

| Field | `Filament` | `Gloam` |
|---|---|---|
| key | `filament` | `gloam` |
| display name | `Filament` | `Gloam` |
| subtitle | `Instrument · stereo out · note input` | `Effect · stereo in/out` |
| parameter rows | 4 | 3 |

The subtitle strings are the **existing** strings used for the instrument and effect classes
(`crates/spectre-app/src/lib.rs:233`, `:240`, `:247`), reused rather than reinvented so the list
stays one vocabulary.

**Shape lens — `Filament`, in descriptor order:**

1. **Lean** — `lean`, unit `Linear`, `0.00 … 1.00`, default `0.50`. Empty state: N/A, a device
   always has all of its parameters.
2. **Rise** — `rise_ms`, unit `Milliseconds`, `1.0 … 1000.0`, default `31.6`.
3. **Fall** — `fall_ms`, unit `Milliseconds`, `1.0 … 1000.0`, default `31.6`.
4. **Level** — `level`, unit `Percent`, `0.00 … 1.00`, default `0.20`.

**Shape lens — `Gloam`, in descriptor order:**

1. **Damp** — `damp_hz`, unit `Hertz`, `20.0 … 20000.0`, default `632.5`.
2. **Depth** — `depth`, unit `Percent`, `0.00 … 1.00`, default `0.00`.
3. **Track** — `track_ms`, unit `Milliseconds`, `1.0 … 1000.0`, default `31.6`.

Data source for every row is `DeviceControl.parameters`
(`crates/spectre-app/src/lib.rs:57–63`), populated by `DeviceControl::from_descriptors` from the
`const` descriptor table in `spectre-dsp`. No value in the table above is written in app code.

**Modulation visibility (PROD-002, `requirements-ledger.md:63`).** Neither device introduces
automation or modulation, and neither is a modulation source or destination beyond the plain
parameter seam. Every value shown in Shape is therefore a base value and nothing else writes to
it, so there is no automated/overridden distinction to render and no restore action to offer.
The provenance record for PROD-002 is `OBS-AB12-AUTO-004`
(`docs/02-reference-research/ableton-live-observations.md:127`), where a benchmark dims an LED on
an automated control that has been manually overridden and offers an explicit
re-enable command. R4-6's obligation is to *not foreclose* that, which it satisfies by holding
exactly one value per parameter in exactly one place and by making the device setter the only way
a value enters the DSP state — so an automation or modulation layer added later has a single
insertion point rather than a second store to reconcile. Inventing a tri-state display now, with
no modulation source in the product, would be a fake surface.

### 3.4 Input & gestures

- **Slider drag / click and `Reset`**: unchanged from the existing Shape rows; both devices use
  the same widgets with no new interaction.
- **Note input**: `Filament` declares `accepts_notes: true` and consumes the bounded, sorted
  `NoteEvent` slice the plan already validates (`crates/spectre-dsp/src/io.rs:77–107`). It adds
  no second event path.
- **Keyboard shortcuts**: **none are added and none are ratified.** A default shortcut map is a
  prohibited conclusion at the current evidence level
  (`docs/02-reference-research/workflow-field-study/product-implications.md:96`). When a command
  system exists it must be context-scoped, searchable, and remappable per
  `docs/00-product/vision.md:41`; the map comes after the model.
- **Specialized input** (stylus, controller, voice, camera, MIDI CC): N/A — R4-6 adds no such
  surface. MIDI parameter control would need its own decision row.
- **Responsive behavior**: unchanged. Seven new rows use the existing row geometry; no new
  minimum width is introduced.

### 3.5 Transitions & animation

- Navigation transitions: none. No view is added or removed.
- In-view state change: two more frames in Build and up to four more rows in Shape, drawn on the
  existing repaint cadence. No fade, no easing, no new timing constant.
- Audio-domain behavior is not animation: `Filament`'s amplitude envelope is DSP, and its two
  time constants are user parameters, not motion design.
- Reduced motion: R4-6 introduces no animation, so a reduced-motion setting has no effect on this
  feature. That is the complete answer, not a deferral.

### 3.6 Error states

Both devices are constructed on the app thread from validated descriptors and applied to on the
render thread. Every failure below is either a compile-time impossibility, an app-thread `Err`,
or a counted render-side refusal. **No error string is formatted on the audio thread.**

| # | Trigger | Presentation | Recovery path | Data loss |
|---|---|---|---|---|
| E1 | A descriptor table is written with `max <= min` or an out-of-range default | **Compile-time panic in `const` evaluation.** `parameter()` is a `const fn` that panics on an invalid `ParamSpec` (`crates/spectre-dsp/src/parameter.rs:140–143`) and `ParamSpec::new` rejects `max <= min` at `crates/spectre-core/src/param.rs:77–79`. Both tables are `const`, so the build fails; nothing reaches a callback. | fix the table | no |
| E2 | `Filament::new` or `Gloam::new` receives an out-of-range or non-finite value | `Err(&'static str)`, matching the existing constructors' shape (`crates/spectre-dsp/src/source.rs:57–63`, `effect.rs:34–39`). The app surfaces it on the inspector status line the existing device-open path already uses (`crates/spectre-app/src/lib.rs:457`). | correct the value | no |
| E3 | A parameter value arrives at `set_parameter` for a key the device does not have | `Err(ParameterError::UnknownKey(key))` — recoverable, never a panic, per the accepted seam (`docs/03-architecture/dsp-device-io.md:92`). R4-2 counts it into `parameters_pending`. | none in-app; it is a route-table defect | no |
| E4 | A non-finite value somehow reaches `set_parameter` | Contained without a branch: both setters route through `DspParameter::clamp`, and `ParamSpec::clamp` maps non-finite input to the descriptor default (`crates/spectre-core/src/param.rs:119–124`). The DSP state can never hold a non-finite parameter. | none needed | no |
| E5 | A non-finite **audio sample** reaches `Gloam`'s input | Contained at the device boundary before the filter state is updated (§4.3), so the recursive state cannot latch. This is required by `docs/03-architecture/dsp-device-io.md:29` and matters more for `Gloam` than for the existing stateless effects — see §4.3. | none needed | no |
| E6 | A device nonetheless emits NaN or infinity | RT-003 silences that node's entire output for the quantum and records it (`crates/spectre-graph/src/lib.rs:510–521`), surfacing as `contaminated_nodes`. Silence, never noise. | none needed | no |
| E7 | Buffer or event layout does not match the declared `DeviceIo` | `validate_buffers` returns `ProcessError` before any device arithmetic (`crates/spectre-dsp/src/io.rs:175–198`), which the plan maps to `PlanError::Process`. Both new devices call it first, exactly as all four existing devices do. | none in-app; it is a plan-construction defect | no |
| E8 | `Gloam`'s recursive state decays into denormal range | Flushed to signed zero at the end of the quantum using the same predicate the plan uses (`crates/spectre-graph/src/lib.rs:407–410`). Bounded exposure: at most one render quantum. | none needed | no |

Three rules bind the table. **Fail closed:** every refusal produces silence or the previous
value, never stale audio and never noise. **Counted, not dropped:** every render-side refusal
lands in an existing counter rather than a log. **No new error vocabulary:** R4-6 adds no
`ProcessError` variant and no `PlanError` variant; both devices fit the existing failure model,
which is itself evidence that the contract is right rather than that the devices are special.

### 3.7 Accessibility

- Both devices are drawn entirely by the existing descriptor-driven rows. No icon-only control,
  no color-only state, no custom-painted widget, and no drag-only gesture is introduced, so
  nothing here forecloses decision 17's beta bar (`docs/01-requirements/decision-gates.md:41`).
- Screen reader labels, hints, traits: `crates/spectre-app` builds eframe with no accessibility
  feature enabled today, so **R4-6 claims no screen-reader support.** Its obligation is
  non-foreclosure: every one of the seven new controls carries a text name, a unit, a range, and
  a default that come from the backend descriptor, which is precisely the material an
  accessibility layer needs and is exactly why `docs/03-architecture/dsp-device-io.md:117–118`
  requires descriptors before device UI.
- Color-independent state: neither device communicates anything by color. `Filament`'s state is
  audible and its controls are numeric.
- Custom actions: N/A — no compound gesture is added.
- Text scaling: seven additional rows use the existing row geometry; the only new risk is vertical
  length in the Shape scroll area, which already scrolls. Manual check in §5.4.
- Focus order: appended. Two new device frames come last in Build; within Shape, rows follow
  descriptor order, which is creation order. Nothing is reordered.
- **A naming obligation this creates:** `Lean`, `Damp`, `Depth`, and `Track` are short labels
  that a screen reader will read literally. They are unambiguous within a device but not
  self-describing out of context, so §5.4's manual check includes reading each row's hover text —
  which already names range and default — and §8 Q7 asks whether descriptors should carry a
  longer description field before the decision 17 audit.

---

## 4. Implementation Specification

### 4.1 Architecture placement

| Path | Change | Thread |
|---|---|---|
| `crates/spectre-dsp/src/filament.rs` | **new** — `Filament`, `FILAMENT_PARAMETERS` | render (`process` and `set_parameter` are callback-reachable) |
| `crates/spectre-dsp/src/gloam.rs` | **new** — `Gloam`, `GLOAM_PARAMETERS` | render |
| `crates/spectre-dsp/src/lib.rs` | **modify** — two `mod` lines and two `pub use` lines | build |
| `crates/spectre-app/src/lib.rs` | **modify** — two `DeviceControl::from_descriptors` entries in `prototype()` | app |
| `crates/spectre-offline/src/lib.rs` | **modify** — one new public render entry point for the two-device chain, plus extraction of the existing peak/hash walk into a private helper so there is exactly one definition of it | app (offline) |
| `crates/spectre-dsp/tests/devices.rs` | **modify** — device contract and injection tests | test |
| `crates/spectre-graph/tests/containment.rs` | **modify** — one sibling to the existing healthy-device test | test |
| `crates/spectre-offline/tests/harness.rs` | **modify** — determinism and silence tests for the new chain | test |
| `crates/spectre-audio/tests/rt_guard.rs` | **modify** — one allocation-guarded plan render over the new devices | test |
| `crates/spectre-app/tests/app_model.rs` | **modify** — device-list and snapshot-arity tests | test |
| `docs/01-requirements/requirements-ledger.md` | **modify** — the `DEV` rationale rows in §7.2, routed as a proposal (§8 Q3) | docs |
| `docs/03-architecture/dsp-device-io.md` | **modify** — extend the "Initial native devices" list; routed as a proposal (§8 Q2) | docs |

Two new modules rather than appending to `source.rs` and `effect.rs`: each device is a distinct
state machine with its own descriptor table, and both existing files declare a single purpose in
their headers (`source.rs:3`, `effect.rs:3`). The `parameter()` constructor is `pub(crate)`
(`crates/spectre-dsp/src/parameter.rs:128`), so a new module inside `spectre-dsp` can build
descriptors with it; a module outside the crate could not. That is a checked fact, not an
assumption.

**The RT-001 argument, made against the actual guard rather than a claim about it.**

`crates/spectre-audio/tests/rt_guard.rs` carries two different guarantees and R4-6 is covered by
only one of them:

- The **structural lock scan** (`rt_guard.rs:289–320`) reads four module paths declared at
  `:293–298` — `src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, `src/null.rs` — and joins them
  to `env!("CARGO_MANIFEST_DIR")` at `:309`, which is `spectre-audio`'s own manifest directory.
  **It cannot reach `crates/spectre-dsp/src/` at all.** No device that has ever shipped is
  covered by it, and R4-6's two are not either. Saying so is the honest position; §8 Q8 asks
  whether the scan should be widened, and this spec does not widen it unilaterally.
- The **allocation guard** is the stronger of the two, and the file's own header says so
  (`rt_guard.rs:8–9`). A thread-local flag marks an RT section and the global allocator
  attributes any allocation inside one to the exact call, with a deliberately-allocating positive
  control at `:138–155` proving the probe works.
  `plan_process_is_rt_clean_through_the_null_stream` (`:259`) drives a compiled plan through a
  real backend callback inside that guard, using the `Pulse → Gain → Saturator` fixture built at
  `:76–117`. **R4-6 adds a sibling test that builds a `Filament → Gloam` plan and drives it the
  same way** (§5.2), which is the actual RT-001 evidence for these devices.

**Why the new work is allocation-free, path by path.** Both `process` bodies own no heap
storage: state is fixed-size scalars in the device struct, all buffers are borrowed
(`docs/03-architecture/dsp-device-io.md:26`), and the loops index existing slices. Both call
`validate_buffers` first, which allocates nothing (`crates/spectre-dsp/src/io.rs:175–198`).
Both `set_parameter` bodies compare a `&'static str`-backed key and call `DspParameter::clamp`,
which is `self.spec.clamp(f64::from(value)).plain() as f32`
(`crates/spectre-dsp/src/parameter.rs:49–51`) over a `const` descriptor. There is no `Vec`, no
`Box`, no `String`, no formatting, no logging, and no lock in either module.

**No panic on any callback-reachable path.** The three places a panic could hide are all
closed by construction and each is checked below rather than asserted: indexing is bounded by
`context.frames()` after `validate_buffers` has proven every buffer is exactly that long
(`io.rs:187–193`); division by zero is unreachable in `Filament`'s phase warp and impossible in
both time-constant computations (§4.3); and `f64::clamp`, which panics only when `min > max`,
sits behind a `const` `ParamSpec` that a build cannot produce with `min > max` (E1). No
`unwrap`, `expect`, `assert`, or slice-range operation appears in either device.

**GRAPH-001 (`requirements-ledger.md:55`) is unaffected.** Both devices are `AudioProcessor`
implementations that a plan owns; neither touches graph structure, and `CompiledPlan` still
exposes no node or edge mutation API (`crates/spectre-graph/src/lib.rs:416–417`). Nothing is
recompiled per parameter change; decision 22's rejected option (b) is not proposed here in any
form.

### 4.2 Data model

**New — `crates/spectre-dsp/src/filament.rs`.**

```rust
// Author: Jeff
// Date: 2026-08-16
// Description: Filament — monophonic phase-warped voice with a linear amplitude contour
// Notes: One voice, one oscillator, no modulation. Phase accumulates in f64; every other value
//   is f32. Nothing here allocates, locks, formats, or panics.

const FILAMENT_IO: DeviceIo = DeviceIo {
    class: DeviceClass::Instrument,
    audio_inputs: 0,
    audio_outputs: 2,
    accepts_notes: true,
};

pub const FILAMENT_PARAMETERS: [DspParameter; 4] = [
    parameter("lean", "Lean", ParamUnit::Linear, 0.0, 1.0, 0.5),
    parameter("rise_ms", "Rise", ParamUnit::Milliseconds, 1.0, 1_000.0, 31.6),
    parameter("fall_ms", "Fall", ParamUnit::Milliseconds, 1.0, 1_000.0, 31.6),
    parameter("level", "Level", ParamUnit::Percent, 0.0, 1.0, 0.2),
];

// Monophonic note-driven voice: one warped oscillator behind one linear amplitude contour
#[derive(Debug, Clone)]
pub struct Filament {
    lean: f32,
    rise_ms: f32,
    fall_ms: f32,
    level: f32,
    // Normalized oscillator phase in [0, 1); f64 for the same reason ToneSource uses it
    phase: f64,
    // Active note identity and number, matching the note-ID contract the event slice carries
    active: Option<(u32, u8)>,
    velocity: f32,
    // Amplitude contour in [0, 1]; reaches its target exactly, so silence is exact
    contour: f32,
}
```

`FILAMENT_IO` is the same declaration `PulseInstrument` uses
(`crates/spectre-dsp/src/source.rs:20–25`), because the instrument layout is fixed by the
contract (`docs/03-architecture/dsp-device-io.md:36`) and is not this device's to choose.

**New — `crates/spectre-dsp/src/gloam.rs`.**

```rust
// Author: Jeff
// Date: 2026-08-16
// Description: Gloam — stereo one-pole damping whose corner opens with the signal's own level
// Notes: Recursive state, so non-finite input is contained before the state update rather than
//   after it; a latched NaN would poison every later block. Coefficients are computed once per
//   quantum, never per sample.

const GLOAM_IO: DeviceIo = DeviceIo {
    class: DeviceClass::Effect,
    audio_inputs: 2,
    audio_outputs: 2,
    accepts_notes: false,
};

pub const GLOAM_PARAMETERS: [DspParameter; 3] = [
    parameter("damp_hz", "Damp", ParamUnit::Hertz, 20.0, 20_000.0, 632.5),
    parameter("depth", "Depth", ParamUnit::Percent, 0.0, 1.0, 0.0),
    parameter("track_ms", "Track", ParamUnit::Milliseconds, 1.0, 1_000.0, 31.6),
];

// Stereo level-tracked damping; two scalars of state per channel and nothing else
#[derive(Debug, Clone)]
pub struct Gloam {
    damp_hz: f32,
    depth: f32,
    track_ms: f32,
    // Per-channel magnitude follower output in [0, max |x|]
    follower: [f32; 2],
    // Per-channel one-pole state; a convex combination of past inputs, so |state| <= max |x|
    damped: [f32; 2],
}
```

`GLOAM_IO` is byte-for-byte the existing `EFFECT_IO`
(`crates/spectre-dsp/src/effect.rs:12–17`); the constant is restated in the new module rather
than made `pub(crate)` and shared, because R4-6 should not refactor `effect.rs` to add a device.
§8 Q9 asks whether the four layout constants should be hoisted once a fifth and sixth device
exist.

**Every numeric bound above, with its own rationale.** These are the rows §7.2 schedules into
`docs/01-requirements/requirements-ledger.md`, discharging PROD-003 (`:64`) and decision 16
(`decision-gates.md:40`). **None is taken from any reference product**, and where a bound is
chosen rather than derived, the row says so and carries a re-open trigger.

| ID | Bound | Value | Rationale |
|---|---|---|---|
| DEV-001 | `filament.lean` range | `0.0 … 1.0` | The control **is** a normalized phase position — the breakpoint of the phase map in §4.3. Spectre's existing oscillators already normalize phase to `[0, 1)` via `.fract()` (`crates/spectre-dsp/src/source.rs:96`, `:205`), so the range is the domain itself and not a chosen envelope. Both endpoints are reachable and both are proven safe in §4.3. |
| DEV-002 | `filament.lean` default | `0.5` | **Derived, not chosen.** `0.5` is the unique value at which the phase map is the identity and the oscillator is an exact sine (§4.3 gives the algebra; §5.1 test 2 asserts it). The default is the device's neutral point by construction. |
| DEV-003 | `filament.rise_ms` / `filament.fall_ms` range | `1.0 … 1000.0` ms | **Minimum 1.0 ms:** the shortest contour Spectre will let a user set must still be a ramp at every sample rate the engine can open. `MIN_SAMPLE_RATE` is `8_000` (`crates/spectre-audio/src/lib.rs:25`), and `1 ms × 8 000 Hz = 8` sample periods, so the contour is at least an eight-step ramp everywhere and the device cannot produce a full-scale amplitude step of its own. **Maximum 1000.0 ms:** two beats at 120 BPM, which is the tempo Spectre's own default project uses (`crates/spectre-offline/src/lib.rs:157`, `TempoMap::constant(120.0)`); a contour longer than two beats at the project default would outlast the musical gesture it shapes. **Re-open trigger:** the first real project whose tempo makes 1000 ms shorter than one beat — that is any tempo below 120 BPM for a two-beat contour — or the first request for a pad-length release. |
| DEV-004 | `filament.rise_ms` / `filament.fall_ms` default | `31.6` ms | **Derived by a stated rule:** the geometric midpoint of the parameter's own declared range, rounded to one decimal place. `sqrt(1.0 × 1000.0) = 31.6227766…` → `31.6`. The rule is used for every strictly-positive range in both devices so that no default in either device is a taste number. |
| DEV-005 | `filament.level` range and default | `0.0 … 1.0`, default `0.2` | Range is the normalized amplitude domain. The default is **Spectre's own existing instrument default**, `PULSE_PARAMETERS[0]`'s `0.2` (`crates/spectre-dsp/src/source.rs:39–46`), reused so that swapping the alpha's instrument does not change its resting loudness. The geometric-midpoint rule degenerates to `0` on a range whose minimum is `0`, and a silent instrument default would be a fake surface, so this row states its own basis explicitly. |
| DEV-006 | `gloam.damp_hz` range | `20.0 … 20000.0` Hz | **Spectre's own already-accepted audible band.** `TONE_PARAMETERS[0]` declares `20 … 20 000 Hz` for the frequency of a native source (`crates/spectre-dsp/src/source.rs:28–35`). Declaring a second, different audible band for a filter corner would create two definitions of the same thing; this row reuses the one that exists. |
| DEV-007 | `gloam.damp_hz` default | `632.5` Hz | **Derived by the DEV-004 rule:** the geometric midpoint of `20 … 20 000`, rounded to one decimal. `20 × 20 000 = 400 000`; `sqrt(400 000) = 632.4555…` → `632.5`. A one-pole corner has no more principled center than the geometric center of its own declared band, and a derived number is auditable where a taste number is not. |
| DEV-008 | `gloam.depth` range and default | `0.0 … 1.0`, default `0.0` | Range is the normalized fraction of the available corner travel. **Default `0.0` is deliberate and is not the midpoint rule:** at `0.0` the device is a plain static one-pole with a closed-form response, which is (a) the least-surprise resting state — the effect does nothing the user did not ask for — and (b) the reference state §5.1 test 6 asserts against a hand-computed expectation. The level-tracking behavior becomes attributable to a deliberate move rather than to the device's default. |
| DEV-009 | `gloam.track_ms` range and default | `1.0 … 1000.0` ms, default `31.6` | Same range and the same derived default as DEV-003/DEV-004, and for the same two reasons: 1 ms is at least eight sample periods at `MIN_SAMPLE_RATE` so the follower is a filter rather than a bare rectifier, and 1000 ms is two beats at the project default tempo. Sharing the envelope with `filament.rise_ms` is intentional — one derivation, three parameters. |
| DEV-010 | `Filament` voice count | `1` | **One voice.** Polyphony requires a voice count *and* a voice-stealing policy, and both are numeric/behavioral limits Spectre has no evidence for. `OBS-PP-UNI-003` (`docs/02-reference-research/synth-modular-observations.md:37`) records that a benchmark treats polyphony limit and voice recycling — oldest and quietest — as documented product behavior, which is the evidence that this is a product decision rather than an implementation detail. `OBS-SR2-CPU-001` (`:54`) is the one citable record on per-voice cost and it favors shared post-FX over per-voice duplication; **its numbers are not adopted.** One voice also matches the shipped instrument (`crates/spectre-dsp/src/source.rs:117`, a single `Option<(u32, u8)>`). **Re-open trigger:** R4-5's MIDI clips producing overlapping notes that a user expects to hear together. §8 Q4. |
| DEV-011 | `Gloam` filter order | `1` (one pole per channel) | One pole is the highest order whose stability needs no argument beyond "the coefficient is in `[0, 1]`" — it is a convex combination of the current input and the previous output, so it cannot ring, cannot self-oscillate, and cannot exceed its input's peak, whatever the parameters do (§4.3 proves the coefficient bound). A second order would introduce resonance, and resonance introduces a Q limit, which is a numeric bound with nothing behind it. **Re-open trigger:** the first accepted requirement that needs a resonant filter. |
| DEV-012 | `Gloam` denormal-state flush threshold | `f32::MIN_POSITIVE` | **Not a new number.** It is the same predicate and the same constant the accepted RT-003 implementation already uses on plan output (`crates/spectre-graph/src/lib.rs:407–410`), applied to this device's internal state. A second threshold would be a second definition of denormal. |

Arithmetic in DEV-004, DEV-007, and DEV-003 is checkable in one line each and is stated so it
can be checked: `sqrt(1000) = 31.6227766…`; `20 × 20 000 = 400 000` and `sqrt(400 000) =
632.4555…`; `0.001 s × 8 000 Hz = 8` samples.

**Migrations: N/A — R4-6 persists nothing.** Device parameter persistence is R4-7's; these
descriptors are what it will persist, but this slice adds no serialized field and no schema
change.

### 4.3 API contracts

**Both devices implement `AudioProcessor` as R4-2 extends it.** The trait has exactly two methods
today — `io` and `process` (`crates/spectre-dsp/src/io.rs:163–172`) — and R4-2 adds a required
third, `set_parameter(&mut self, key: DeviceParameterKey, value: f32) -> Result<(), ParameterError>`,
per the accepted seam at `docs/03-architecture/dsp-device-io.md:87–94`. **R4-6 is a hard consumer
of that method, not an optional one**, and R4-2's own §2 says so: a new device must be forced to
answer the seam rather than silently ignore edits. If R4-2 has not landed, both devices compile
with `io` and `process` only and every control is dead; §7.4 makes this blocking.

```rust
impl Filament {
    // Validate against the device's own descriptors, exactly as the shipped constructors do
    pub fn new(lean: f32, rise_ms: f32, fall_ms: f32, level: f32) -> Result<Self, &'static str>;

    pub fn parameters(&self) -> &'static [DspParameter];
}

impl Gloam {
    pub fn new(damp_hz: f32, depth: f32, track_ms: f32) -> Result<Self, &'static str>;

    pub fn parameters(&self) -> &'static [DspParameter];
}
```

Each `new` validates through `FILAMENT_PARAMETERS[i].validate(..)` / `GLOAM_PARAMETERS[i].validate(..)`
and maps the error to a `&'static str`, which is the shape both shipped constructors already use
(`crates/spectre-dsp/src/source.rs:57–63`, `crates/spectre-dsp/src/effect.rs:34–39`) — a
`&'static str` rather than an owned `String` so construction never allocates a message.

**`Filament::process`, stated as behavior.**

Per quantum, computed once, before the sample loop:

```text
rise_step = 1.0 / max(1.0, rise_ms * 0.001 * sample_rate)
fall_step = 1.0 / max(1.0, fall_ms * 0.001 * sample_rate)
```

The `max(1.0, …)` is what makes both totals: the denominator is never zero, never negative, and
never subnormal, so neither step is `inf` or `NaN` for any finite positive sample rate — and
`ProcessContext::new` has already rejected a non-finite or non-positive rate
(`crates/spectre-dsp/src/io.rs:82–84`). A step of `1.0` means the contour reaches its target in
one sample, which is the degenerate case and is still finite.

Per frame, in this order:

1. Drain events at this frame offset with the same `while` structure `PulseInstrument` uses
   (`crates/spectre-dsp/src/source.rs:176–201`): `On` sets `active` and `velocity` and resets
   `phase` to `0.0`; `Off` clears the voice only when the ID matches; `AllNotesOff` clears it
   unconditionally. Velocity is already validated finite and in `[0, 1]` by the event contract
   (`crates/spectre-dsp/src/io.rs:137–142`), so the device does not re-police it.
2. Move the contour one step toward its target — `1.0` if a note is active, `0.0` otherwise —
   clamping at the target so it is reached **exactly**: `contour = (contour + rise_step).min(1.0)`
   when rising, `contour = (contour - fall_step).max(0.0)` when falling.
3. If `active.is_none()` and `contour == 0.0`, write exact `0.0` to both channels and leave the
   phase where it is. This is what makes silence exact silence
   (`docs/03-architecture/dsp-device-io.md:112`) rather than an exponential residue that would
   generate denormals forever.
4. Otherwise write `warped(phase, lean) * contour * velocity * level` to both channels, then
   advance `phase` by `frequency / sample_rate` and take `.fract()`, where `frequency` is the
   equal-tempered mapping the shipped instrument already uses
   (`crates/spectre-dsp/src/source.rs:204`). Mono content on both channels: the effect layout is
   stereo by contract, and R4 has no stereo-image requirement to satisfy.

**The phase warp — the device's one original idea, and the whole of it.**

```text
warped(phase, lean) = sin( TAU * map(phase, lean) )

map(phase, lean) = 0.5 * phase / lean                     when phase <  lean
                 = 0.5 + 0.5 * (phase - lean) / (1 - lean) when phase >= lean
```

`map` is a two-segment piecewise-linear reparameterization of the phase ramp that always sends
`[0, 1)` onto `[0, 1)` and always sends `lean` to `0.5`. Moving `lean` away from `0.5` compresses
one half-cycle of the sine and stretches the other, which raises harmonic content continuously
without switching shapes. That is why the user-facing control is a continuous **Lean** rather
than a waveform menu, and it is why this device is not `PulseInstrument` with more parameters.

Three properties, each checkable:

- **`lean = 0.5` is the identity.** First branch: `0.5 * phase / 0.5 = phase`. Second branch:
  `0.5 + 0.5 * (phase - 0.5) / 0.5 = 0.5 + (phase - 0.5) = phase`. Both multiplications and
  divisions are by `0.5`, a power of two and therefore exact in binary floating point, and
  `phase - 0.5` for `phase ∈ [0.5, 1)` is exact by Sterbenz's lemma, so the identity is
  **bit-exact**, not approximate. §5.1 test 2 asserts equality with `(phase * TAU).sin()`.
- **Neither degenerate denominator is reachable.** `phase ∈ [0, 1)` because it comes from
  `.fract()` of a non-negative value. At `lean = 0.0` the guard `phase < 0.0` is false for every
  such phase, so only the second branch runs and its denominator is `1 - 0 = 1`. At `lean = 1.0`
  the guard `phase < 1.0` is true for every such phase, so only the first branch runs and its
  denominator is `1`. The division that would be by zero is in the branch that endpoint never
  takes. **The strict `<` and the `else` are load-bearing and must be written exactly as above**;
  §5.1 test 3 pins both endpoints against output that is finite and bounded.
- **The attack begins at a zero crossing.** A note-on sets `phase = 0`, and `map(0, lean)` is
  `0` for every `lean > 0` and `0.5` for `lean = 0`; `sin(0)` and `sin(TAU × 0.5)` are both zero
  crossings. Combined with the contour starting at `0`, `Filament`'s note-on cannot produce an
  amplitude step regardless of settings. §5.1 test 4.

**Aliasing, stated rather than glossed.** `warped` is not band-limited. For `lean` away from
`0.5` it produces harmonics above Nyquist that fold back, exactly as `PulseInstrument`'s naive
saw and square already do (`crates/spectre-dsp/src/source.rs:147–155`). **This spec claims no
anti-aliasing and no spectral quality of any kind.** No benchmark in the accepted corpus has a
citable observation about oscillator anti-aliasing, so there is no evidence to grade a target
against; §8 Q5 routes band-limiting to Jeff as an explicit pre-beta question rather than
smuggling it into an R4 device.

**`Gloam::process`, stated as behavior.**

Per quantum, computed once, before the sample loop, in `f64` and stored as `f32`:

```text
damp_a  = 1 - exp(-TAU * damp_hz / sample_rate)
track_k = 1 - exp(-1 / (track_ms * 0.001 * sample_rate))
```

Both are total. `damp_hz ≥ 20` and `sample_rate > 0` are both guaranteed, so the exponent is
finite and non-positive and `exp` of it lies in `(0, 1]`, giving `damp_a ∈ [0, 1)`. If the ratio
is enormous, `exp` underflows to `0` and `damp_a` becomes exactly `1` — transparent, still
stable. For `track_k`, if the denominator underflows to zero then `-1/0 = -inf` and
`exp(-inf) = 0`, giving `track_k = 1` — instantaneous tracking, still bounded. **Neither
expression can produce `NaN` for any input the descriptors and the process context allow**, and
that is the complete argument. Two `exp` calls per quantum, not per sample; `exp` is a libm call
that does not allocate, lock, or panic, and per-sample transcendentals are already shipped
precedent on this path (`sin` at `crates/spectre-dsp/src/source.rs:93`, `tanh` at
`crates/spectre-dsp/src/effect.rs:127`).

Per frame, per channel, in this order — **the order is the correctness argument**:

1. `x = if input.is_finite() { input } else { 0.0 }`. Containment happens **before** the state
   update. This is the difference between `Gloam` and the two shipped effects: `Gain`
   (`crates/spectre-dsp/src/effect.rs:65–69`) and `Saturator` (`:122–126`) are stateless, so a
   non-finite input corrupts exactly one output sample even if containment were missed. `Gloam`
   is recursive, so a single `NaN` reaching `follower` or `damped` would latch and every
   subsequent block would be poisoned forever. The plan's RT-003 containment
   (`crates/spectre-graph/src/lib.rs:510–521`) does not help here, because it runs on a node's
   output *after* that node has already updated its own state.
2. `follower += track_k * (|x| - follower)`, with `track_k ∈ (0, 1]`, so `follower` stays a
   convex combination of past magnitudes and therefore in `[0, max |x|]`.
3. `opening = clamp(depth * follower, 0.0, 1.0)`. **The clamp is mandatory, not defensive
   tidiness.** `follower` can exceed `1.0` whenever the input does, and without the clamp the
   next line would produce a coefficient greater than `1`, which is the one way this filter could
   be made unstable.
4. `a = damp_a + (1 - damp_a) * opening`, so `a ∈ [damp_a, 1] ⊆ [0, 1]` for every reachable
   parameter and signal value. At `opening = 1` the coefficient is exactly `1` and the filter is
   exactly transparent (`damped += 1 × (x - damped)` gives `damped = x`), which is the intended
   fully-open endpoint and needs no special case.
5. `damped += a * (x - damped)`; write `damped` to the output.

**Two consequences worth stating because they are guarantees, not hopes.** Because `a ∈ [0, 1]`,
`damped` is always a convex combination of past inputs, so `|output| ≤ max |input|`: `Gloam`
cannot increase peak level, cannot ring, and cannot clip a downstream buffer. And because
`Filament`'s output is bounded by `level ≤ 1.0`, the pair `Filament → Gloam` has an output
magnitude bounded by `1.0` by construction. `OBS-AB12-MIX-002`
(`docs/02-reference-research/ableton-live-observations.md:99`) records a benchmark relying on a
32-bit float engine to tolerate over-0 dB internally and treating clipping as a physical-output
concern; Spectre's engine is likewise `f32` throughout, and this pair does not exercise that
tolerance because it stays inside unity by construction.

**Denormal state, at the end of each quantum.** After the sample loop, each of the four state
scalars is flushed using the same predicate as the accepted plan-level flush
(`crates/spectre-graph/src/lib.rs:407–410`): if the value is nonzero and its magnitude is below
`f32::MIN_POSITIVE`, it becomes signed zero. This is necessary because the plan's flush operates
on **output buffers** and never sees a device's internal state, so `Gloam`'s exponentially
decaying tail would otherwise sit in denormal range indefinitely after a signal stops, at
denormal arithmetic cost, on the callback path, forever. Once flushed, the recursion produces
exact `0.0` from then on. Doing it per quantum rather than per sample bounds the exposure to one
render quantum and keeps the inner loop to two adds and two multiplies per sample per channel;
that bound is stated rather than dismissed.

**Where smoothing is and is not.** `docs/03-architecture/dsp-device-io.md:59` says *"Parameter
smoothing is owned by the device when discontinuities can click or destabilize processing,"* and
`:94` says *"Smoothing stays the device's concern. `Gain` already smooths."* **The shipped `Gain`
does not smooth** — it is a single `f32` field (`crates/spectre-dsp/src/effect.rs:29–31`)
multiplied directly per sample (`:62–71`), with no target, no coefficient, and no ramp. That
contradiction is logged as **D-R3** in `gauntlet-output/decisions-needed.md`, it is open, and
**R4-6 does not resolve it.** What R4-6 states about its own two devices:

- `Filament` has no per-parameter smoothing. Its **amplitude** contour is a user-controlled ramp
  whose minimum is chosen so the device cannot step (DEV-003), which answers the click question
  for the note envelope — the loudest discontinuity an instrument has — and answers nothing else.
  A large, fast `level` move mid-note steps at the block boundary and can click.
- `Gloam` has no per-parameter smoothing either, but its `damp_hz` and `track_ms` reach the
  signal only through coefficients recomputed once per quantum and applied inside a one-pole
  recursion, so a coefficient change moves the output continuously rather than scaling it — it
  cannot produce a jump discontinuity in the output the way a gain change can. `depth` behaves
  the same way. This is a property of where those parameters sit in the topology, **not** a claim
  that `Gloam` is click-free.

Adding true parameter smoothing to either device would require a smoothing time constant, which
is another numeric bound needing a rationale row, and it would change rendered output for the
same inputs, which is the same evidence consequence R4-2 declined to take on. §8 Q1 routes the
shared question — should Spectre define one smoothing policy for all devices, or leave it
per-device? — to Jeff against D-R3.

**Auth, permissions, pagination, rate limiting: N/A** — in-process desktop DSP with no network
and no multi-user surface.

### 4.4 State management

| State | Owner | Thread | Lifetime |
|---|---|---|---|
| Parameter value of record | `AppModel.devices[..].parameters[..].value` (`crates/spectre-app/src/lib.rs:57–63`) | app | process |
| Clamping and range authority | `FILAMENT_PARAMETERS` / `GLOAM_PARAMETERS`, `const`, in `spectre-dsp` | both sides use the same tables | static |
| Applied parameter value | the device's own fields inside the compiled plan | render | until the plan is dropped |
| Voice state (`phase`, `active`, `velocity`, `contour`) | `Filament` | render | until the plan is dropped |
| Filter state (`follower`, `damped`) | `Gloam` | render | until the plan is dropped |

**No new state container, no new store, and no second source of truth.** Both devices are owned
by `CompiledPlan` as `Box<dyn AudioProcessor>` exactly as the four existing devices are. Neither
holds a reference to app state, a channel, a clock, or a global.

**RT-002 disposition (criterion 1B).** R4-6 introduces **no new control↔render traffic** and no
new lane. Parameter values reach both devices through the existing latest-wins parameter lane
that decision 21 accepted (`docs/01-requirements/decision-gates.md:45`) and that R4-2 connects;
note events reach `Filament` through the existing bounded FIFO note path, whose per-quantum
capacity is already fixed at `MAX_NOTE_EVENTS_PER_BLOCK = 1_024`
(`crates/spectre-dsp/src/io.rs:52`). Overflow behavior, counting, and off-thread reclamation are
unchanged and are not this slice's to redefine. **Retired state:** neither device allocates, so
neither produces anything to reclaim; when a plan is replaced, the whole plan travels back to the
app thread on the existing reclaim lane and the devices are dropped there.

**Offline / draft persistence: N/A for this slice.** R4-7 owns persistence; §7.2 adds no
serialized field.

**One coupling this slice must not break, and the reason it does not.**
`AppModel::device_parameter_snapshot` (`crates/spectre-app/src/lib.rs:348–383`) iterates a
**hardcoded four-entry `fixture` array** at `:351–356` — `pulse.level`, `gain.gain`,
`saturator.drive`, `saturator.mix` — and looks each one up in `self.devices`. It does **not**
iterate `self.devices`. `spectre-offline`'s `DeviceValues::from_snapshot` refuses any snapshot
whose length is not exactly four (`crates/spectre-offline/src/lib.rs:45–47`). Appending two
devices to `prototype()` therefore leaves the snapshot at exactly four entries and the offline
fixture contract untouched — but only for as long as that array stays hardcoded. §5.3's
`snapshot_arity_is_unchanged_by_new_devices` pins it so a later change cannot break the offline
harness silently.

### 4.5 Dependencies

- **New packages / libraries / frameworks: none.** Both devices use `core`/`std` floating-point
  math only. `spectre-dsp` gains no dependency; its existing imports are `spectre_core` and
  `std::f64::consts::TAU` (`crates/spectre-dsp/src/source.rs:11–12`).
- **New assets or resources: none.** No wavetable, no sample, no impulse response, no lookup
  table. That is a deliberate consequence of decision 15 and it is also what keeps §6.2 empty:
  a device with no asset has no provenance question.
- **Infrastructure: none.** No database, CDN, or third-party service.
- **Workspace graph:** unchanged. `spectre-audio` already uses `spectre_dsp` in its tests
  (`crates/spectre-audio/tests/rt_guard.rs:19`), so §5.2's guard test needs no manifest change.

### 4.6 Platform-specific considerations

- **macOS and Linux are co-first-class (decision 1, `docs/01-requirements/decision-gates.md:25`),
  and both devices are platform-neutral by construction.** There is no `cfg`, no intrinsic, no
  CPU control-register manipulation, and no `unsafe` in either module. The denormal flush is the
  same software FTZ-equivalent the project already chose over control-register manipulation
  precisely because it is portable across both targets
  (`docs/06-plans/current-milestone.md:59`).
- **What that does and does not discharge.** Decision 23's Linux debt is about opening a real
  ALSA device (`decision-gates.md:49`), and R4-6 opens no device. Both devices are provable on
  either platform with no audio hardware at all, because every test in §5 runs offline or against
  the null backend. **This spec authorizes no Linux support claim**, and R4-3 owns the debt.
- **Version compatibility:** none introduced. No OS API, no driver behavior, no GPU path.
- **Feature flags / gradual rollout:** none. Adding a device behind a feature flag would create a
  build in which the alpha has no voice, which is worse than not shipping it.
- **Floating-point determinism across platforms.** Both devices use `f32` arithmetic with `f64`
  phase and coefficient accumulation, which is what the contract permits
  (`docs/03-architecture/dsp-device-io.md:79`). `sin` and `exp` are libm calls, and **libm
  results are not guaranteed bit-identical across platforms or libm versions.** Determinism as
  R4-6 claims it is therefore *within a build on a machine* — identical input, identical output,
  which is what `docs/03-architecture/dsp-device-io.md:113` asks for and what §5.2 asserts. A
  checked-in golden hash that must match on both macOS and Linux would be a stronger claim than
  the evidence supports; §5.2 asserts run-to-run equality rather than a literal hash constant,
  and §8 Q5 records the cross-platform question. The existing devices share this property —
  `ToneSource` calls `sin` (`crates/spectre-dsp/src/source.rs:93`) and `Saturator` calls `tanh`
  (`crates/spectre-dsp/src/effect.rs:119`, `:127`) — so this is a pre-existing property of the
  project's determinism story that R4-6 names rather than introduces.

### 4.7 Performance budget

Stated as operation counts, because Spectre has exactly one hardware measurement
(`docs/06-plans/current-milestone.md:113`: macOS, 2026-08-09, worst-case headroom `0.990`) and a
time budget is a prohibited conclusion at this evidence level
(`docs/02-reference-research/workflow-field-study/product-implications.md:97`). **No time
target, no CPU-percentage target, and no headroom threshold is asserted anywhere in this spec.**

- **Memory.** `Filament` holds four `f32`, one `f64`, one `Option<(u32, u8)>`, and one `f32`;
  `Gloam` holds three `f32` and two `[f32; 2]`. Both are well under one cache line and neither
  allocates at any point in its life. No buffer, no delay line, no table.
- **Per-sample CPU, `Filament`:** one branch on voice state, one add/min-or-max for the contour,
  one comparison and one divide plus two multiply-adds for the phase map, one `sin`, three
  multiplies, one add, one `fract`. **One transcendental per sample.** That is the same order as
  the shipped `Saturator`, which calls `tanh` once per sample per channel
  (`crates/spectre-dsp/src/effect.rs:127`) — twice per frame — while `Filament` computes one
  sample and writes it to both channels.
- **Per-sample CPU, `Gloam`:** per channel, one `is_finite` branch, one `abs`, one clamp, and
  four multiply-adds. **No transcendental in the inner loop**; the two `exp` calls are per
  quantum, so at R4-1's requested 256-frame quantum they amortize to under 0.4% of the per-sample
  work.
- **Per-quantum overhead:** two `max` computations and two divides (`Filament`), two `exp` calls
  (`Gloam`), and a four-scalar denormal flush (`Gloam`).
- **Network payload, storage, startup:** N/A / zero. No I/O of any kind; two more device frames
  at app construction.
- **What is not budgeted, and why that is the honest answer.** Nothing here has been measured,
  because neither device exists. §5.2's allocation-guard test proves the *structural* property
  RT-001 requires; the first real cost number for these devices will come from R4-9's end-to-end
  fixture or a hardware run, and until then the correct statement is an operation count.

---

## 5. Test Specification

Every test below compiles against the API declared in §4.2 and §4.3 and would fail if the
behavior it names regressed. Commands are the real ones.

**The workspace gate, run in full for this slice:**

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

**Targeted commands:**

```sh
cargo test -p spectre-dsp   --test devices
cargo test -p spectre-graph --test containment
cargo test -p spectre-offline --test harness
cargo test -p spectre-audio --test rt_guard
cargo test -p spectre-app   --test app_model
```

### 5.1 Unit tests — `crates/spectre-dsp/tests/devices.rs`

| # | Name | Setup | Assertion | Edge case covered |
|---|---|---|---|---|
| 1 | `new_device_layouts_match_the_v1_contract` (extend the existing test at `devices.rs:186–187`) | `Filament::new`/`Gloam::new` at defaults | `Filament::io()` is `{Instrument, 0, 2, notes: true}`; `Gloam::io()` is `{Effect, 2, 2, notes: false}` | layout drift from `docs/03-architecture/dsp-device-io.md:36–37` |
| 2 | `filament_is_an_exact_sine_at_the_default_lean` | `lean = 0.5`, `level = 1.0`, `rise_ms = 1.0`, note-on velocity `1.0` at frame 0, 4 096 frames at 48 kHz | every output sample equals `(phase * TAU).sin() as f32 * contour` computed independently in the test from the same note frequency — bit-equality, per the exactness argument in §4.3 | the derived default DEV-002; silent drift in the phase map |
| 3 | `filament_stays_finite_at_both_lean_endpoints` | `lean = 0.0` and `lean = 1.0`, held note, 4 096 frames | every sample `is_finite()` and `abs() <= level`; no panic | the two degenerate denominators in the phase map |
| 4 | `filament_attacks_from_a_zero_crossing` | any `lean` in `{0.0, 0.5, 1.0}`, note-on at frame 0 | `output[0].abs() <= 1e-6` for every case | a note-on step |
| 5 | `filament_release_reaches_exact_silence` | `fall_ms = 1.0`, note-on then note-off, render `ceil(fall_ms * rate / 1000) + 2` frames past the release | every sample after the release ramp is **bit-exactly** `0.0` and `is_sign_positive()` | an exponential residue that would never reach zero and would generate denormals forever |
| 6 | `gloam_at_zero_depth_is_a_plain_one_pole` | `depth = 0.0`, `damp_hz = 632.5`, unit step input, 256 frames at 48 kHz. The reference is built to be the **same arithmetic**, not merely the same formula: `a` is computed in `f64` as `1.0 - (-TAU * f64::from(damp_hz) / sample_rate).exp()` and cast with `as f32`, exactly as §4.3 specifies, and the recursion runs in `f32` in the same expression form, `y += a * (x - y)` | every output sample is **bit-exactly** equal to the reference. **No tolerance** — see the note below the table | DEV-008's reference state; coefficient drift |
| 7 | `gloam_contains_non_finite_input_without_latching` | the poison block is composed **entirely** of non-finite samples: every sample of every channel is one of `NAN`, `INFINITY`, `NEG_INFINITY`, cycled, and **not one sample in it is finite**. Feed that block to instance A; then feed one clean block to A and the **same** clean block to a fresh instance B | (i) every sample of A's poison-block output is bit-exactly `0.0`; (ii) every sample of A's and B's clean-block outputs `is_finite()`; (iii) A's clean-block output equals B's clean-block output sample for sample | the recursive-latch failure §4.3 step 1 exists to prevent |
| 8 | `gloam_state_reaches_exact_zero_after_silence` | drive with a unit step for one block, then feed exact silence for at most 64 blocks of 256 frames | within that bound, a whole block of output is bit-exactly `0.0`; assert the bound is not hit | denormal state persisting forever. At the default coefficient `a ≈ 0.0794` the per-sample decay factor is `≈ 0.9206`, so reaching `f32::MIN_POSITIVE ≈ 1.18e-38` from `1.0` needs `≈ 87 / 0.0827 ≈ 1 052` samples — about 4.1 quanta — and 64 quanta is ~15× that headroom |
| 9 | `gloam_never_exceeds_its_input_peak` | full-scale square, sine, and impulse inputs across `depth ∈ {0.0, 0.5, 1.0}` and `damp_hz` at both endpoints | `output.abs().max() <= input.abs().max()` in every combination | the convex-combination bound in §4.3; a coefficient escaping `[0, 1]` |
| 10 | `new_devices_reject_out_of_range_construction` | `Filament::new` and `Gloam::new` with values below minimum, above maximum, `NAN`, and `INFINITY` | each returns `Err` | `docs/03-architecture/dsp-device-io.md:115` (finite under extreme valid values) and the constructor contract |
| 11 | `new_device_setters_clamp_and_refuse_unknown_keys` | `set_parameter` with in-range, out-of-range, and non-finite values, and with a key from the *other* device | in-range applies; out-of-range clamps to the bound; non-finite becomes the descriptor default (`crates/spectre-core/src/param.rs:119–124`); an unknown key returns `Err(ParameterError::UnknownKey(..))` and leaves state untouched | R4-2's seam contract at `docs/03-architecture/dsp-device-io.md:91–92` |
| 12 | `new_device_descriptors_meet_the_normalized_round_trip_policy` (extend the existing test at `devices.rs:69–70`) | add both tables to `native_parameters()` at `devices.rs:37` | all seven descriptors satisfy `NORMALIZED_ROUND_TRIP_MAX_ULPS` (`crates/spectre-dsp/src/parameter.rs:9`) | the offset `damp_hz` range is the ill-conditioned shape `docs/03-architecture/dsp-device-io.md:67` warns about; this is where that shows up |
| 13 | `filament_contour_minimum_is_at_least_eight_sample_periods` | read `FILAMENT_PARAMETERS[1].minimum()` and `[2].minimum()` | both are `>= 1.0`, and `minimum() * 0.001 * 8_000.0 >= 8.0` | DEV-003's rationale. **This pins the descriptor, not the DSP** — it fails if someone lowers the minimum and silently breaks the no-step guarantee. The `8_000` literal mirrors `spectre_audio::MIN_SAMPLE_RATE` (`crates/spectre-audio/src/lib.rs:25`), which `spectre-dsp` cannot import, and the test comment must say so |

**Why test 6 asserts bit-equality and carries no tolerance.** The earlier form of this row allowed
"within 1 ULP per sample". That is a numeric bound with nothing behind it: error accumulates across
a 256-sample recursion, so a single-ULP bound over a whole block is an assertion rather than a
derivation, and PROD-003 (`docs/01-requirements/requirements-ledger.md:64`) is exactly the rule
against carrying one. The tolerance is **removed rather than derived**, because the stronger claim
is available. At `depth = 0.0`, §4.3 step 3 gives `opening = clamp(0.0 * follower, 0.0, 1.0)`,
which is `+0.0` for every finite `follower`, so step 4 gives `a = damp_a + (1 - damp_a) * 0.0`,
which is `damp_a` bit-exactly: the per-sample coefficient **is** the stored `f32`, with no
arithmetic standing between them. The test therefore computes `damp_a` by the same `f64`
expression and the same `as f32` cast, then runs the same `f32` recursion in the same expression
order, so both sides evaluate the identical sequence of IEEE-754 operations on identical inputs.
Rust emits no fast-math flags, so neither side may contract `y + a * (x - y)` into a fused
multiply-add the other does not, and the one libm call (`exp`) is one call on one input inside one
process, so both sides read the same bits out of it. **This is within-a-build determinism, which
is the only kind §4.6 claims** — it says nothing about macOS versus Linux and does not need to.
The unit-step input is chosen so no state on this path is ever denormal, which keeps the
end-of-quantum flush out of the comparison.

**Why test 7's poison block must be entirely non-finite.** Assertion (iii) is a statement about
*state*, and it holds only if both instances enter the clean block from the same state. A poison
block carrying even one finite nonzero sample would legitimately move A's `follower` and `damped`
away from zero and not B's, and the test would then fail against a device behaving exactly as §4.3
specifies — a false alarm, which is worse than no test. With every sample non-finite, §4.3 step 1
maps all of them to `0.0`; both recursions are then driven from zero by zero, `follower += track_k
* (0.0 - 0.0)` and `damped += a * (0.0 - 0.0)`, so A's state at the end of the poison block is
bit-identical to B's initial state. That is what makes (i) and (iii) provable rather than hopeful.
**The word "entirely" is load-bearing and must be written into the test's setup comment**, the
same way the strict `<` and the `else` are load-bearing in §4.3's phase map.

### 5.2 Integration tests

**`crates/spectre-graph/tests/containment.rs`** — sibling to
`healthy_native_devices_report_no_containment_activity` (`:327–328`):

- `filament_and_gloam_report_no_containment_activity` — compile a `Filament → Gloam` plan, render
  a held note over several quanta, assert `contaminated_nodes == 0`, `last_contaminated.is_none()`,
  and a nonzero peak. The nonzero-peak assertion is what stops two silent buffers from agreeing.

Note what is **not** testable here and why: the plan contains each node's output before it can
reach a downstream device (`crates/spectre-graph/src/lib.rs:510–521`), so an upstream poison
device can never deliver a `NaN` to `Gloam`'s input inside a plan. `Gloam`'s own boundary
containment is defense in depth and is therefore tested at the device level (§5.1 test 7), which
is the same reason the poison devices in that file are test-only
(`crates/spectre-graph/tests/containment.rs:4–6`).

**`crates/spectre-offline/tests/harness.rs`:**

- `voice_chain_renders_deterministically` — call the new `render_voice_chain(48_000.0, 4_096)`
  twice, assert the two `RenderReport`s are equal including `hash`, and assert `peak > 0.0`. The
  hash is the **existing** FNV-1a walk (`crates/spectre-offline/src/lib.rs:271–278`), reused via
  the helper §7.2 extracts; no second walk is introduced, per criterion 1D.
- `voice_chain_renders_exact_silence_without_events` — same chain with no events; assert
  `peak == 0.0`, mirroring `plan_renders_exact_silence_without_events` (`harness.rs:49–50`).

**`existing_fixture_hash_is_unchanged` is withdrawn, and no test replaces it.** The earlier form
asserted that `render_vertical_slice` "still returns the same report as before this slice", which
names no value: `crates/spectre-offline/tests/harness.rs` holds no golden constant, so the test
could only have compared the implementation against itself and **could not fail**. Criterion 1G
scores an unfailable test at zero, and the honest fix is to delete it rather than to invent a
baseline for it.

The guard it was reaching for **already exists, three times, and each of the three computes its
reference outside `spectre-offline` so the extraction cannot hide inside it:**

- `plan_render_matches_hand_wired_chain` (`crates/spectre-offline/tests/harness.rs:57–58`)
  hand-wires `PulseInstrument → Gain → Saturator` in the test file, folds peak and FNV-1a over the
  result with its own inline loop (`:97–105`), and asserts `render_vertical_slice`'s `hash` and
  `peak` equal it (`:108–109`).
- `app_defaults_match_backend_authoritative_default_render` (`:220–221`) asserts
  `render_app_snapshot` at the descriptor defaults equals `hand_wired_report` (`:112–170`, its own
  inline fold at `:155–163`).
- `every_app_parameter_maps_exactly_to_the_compiled_plan` (`:172–173`) asserts the same equality
  across four parameter moves.

If §7.2 item 5's extraction changed the peak/hash walk by one bit, **all three fail**, because all
three references are written in the test crate and none of them calls the helper. That is a
stronger regression guard than a self-comparison, and it costs this slice nothing.

**What R4-6 deliberately does not add here, and why.** The pin the extracted helper would most
benefit from is a checked-in golden vector — a literal `[f32]` array and a literal `u64`, no
render, no device, and therefore no libm, which is the one form of golden hash §4.6's argument
does not reach, since §4.6 refuses golden *render* output because `sin`/`exp`/`tanh` route to
platform libm. R4-6 does not add one, for a reason that is structural rather than aesthetic:
§7.2 item 5 extracts the walk as a **private** helper, and `tests/harness.rs` is an integration
test that links `spectre-offline` as an external consumer, so it cannot reach a private item.
Making the helper `pub` solely so a test could call it would be a public API change this slice
does not otherwise need. R4-8 already makes that fold public and declares exactly this vector and
constant for it (`gauntlet-output/specs/R4-8-offline-bounce.md`, test `the_shared_fold_matches_its_checked_in_golden_vector`); the pin belongs there,
with the public surface, and duplicating it here would create the second definition of one value
that DEV-006 and DEV-012 exist to avoid.

**`crates/spectre-audio/tests/rt_guard.rs`** — sibling to
`plan_process_is_rt_clean_through_the_null_stream` (`:259`):

- `voice_chain_process_is_rt_clean_through_the_null_stream` — build a `Filament → Gloam` plan,
  drive it through the null backend inside the existing RT section
  (`rt_guard.rs:67`), assert **zero** allocations recorded. This is R4-6's RT-001 evidence, and
  the positive control at `:138–155` is what makes a passing result mean something rather than
  being a broken probe reporting success.

### 5.3 UI / E2E tests — `crates/spectre-app/tests/app_model.rs`

- `prototype_exposes_the_two_new_devices` — `AppModel::prototype().devices()` contains entries
  keyed `filament` and `gloam`, with 4 and 3 parameters respectively, and every parameter's
  initial value equals its descriptor default.
- `snapshot_arity_is_unchanged_by_new_devices` — `device_parameter_snapshot()` returns exactly
  four entries. This is the §4.4 coupling made into a gate: it fails the moment someone changes
  `device_parameter_snapshot` to iterate `self.devices`, which would break
  `render_app_snapshot` at `crates/spectre-offline/src/lib.rs:45–47`.
- `opening_a_new_device_focuses_shape` — `AppModel::open_device_in_shape` on `filament`'s
  instance ID, exercising the existing drill-in (`crates/spectre-app/src/lib.rs:342–343`) against
  a device that did not exist when it was written. **The method is fallible and the test must
  handle it:** it is declared
  `pub fn open_device_in_shape(&mut self, id: ObjectId) -> Result<(), &'static str>` at
  `crates/spectre-app/src/lib.rs:338` and returns `Err("unknown device")` at `:340` for any id no
  device in `self.devices` carries. So the assertion order is
  `assert_eq!(model.open_device_in_shape(filament_id), Ok(()))` **first** — otherwise a wrong id
  would leave the lens and the selection untouched and the two assertions below would report a
  focus failure instead of the lookup failure that actually happened — then
  `model.lens() == Lens::Shape` and `model.selected_device_id() == Some(filament_id)` (`:324`),
  which are the two fields `:342–343` writes. The instance id comes from `model.devices()` by
  `key == "filament"`, the way the existing sibling
  `open_device_in_shape_atomically_focuses_existing_device_and_preserves_track`
  (`crates/spectre-app/tests/app_model.rs:206`) does it for `saturator`; this test is that test's
  shape, pointed at a new device.

**No automated UI-rendering test is proposed.** The app has no UI test harness today
(`crates/spectre-app/tests/` contains `app_model.rs` and `smoke_cli.rs` only), and standing one
up to check two appended list rows would be a larger change than the feature. §5.4 covers the
rendering by hand.

### 5.4 Visual / manual verification

Both devices produce audio; the manual protocol is therefore mostly listening, and it cannot run
until R4-1 and R4-2 land. Stated honestly: **as of this spec, none of §5.4 can be executed**,
because `./spectre` makes no sound. R4-9 owns the written QA protocol; these are the R4-6 rows
it should carry.

| # | Check | Expected |
|---|---|---|
| M1 | Build lens with five devices | Two new frames render in list order with correct names and subtitles; no clipping or overlap |
| M2 | Shape lens, `Filament`, at default egui zoom and at the largest zoom the shell supports | All four rows readable; slider and `Reset` reachable; the header sentence wraps rather than clipping the slider |
| M3 | Shape lens, `Gloam` | All three rows readable; hover text names range and default for each |
| M4 | Monochrome / color-blind reading | Nothing about either device's state is carried by color |
| M5 | Hold a note, sweep `Lean` from 0.00 to 1.00 | The timbre changes continuously; the pitch does not move; no dropout |
| M6 | Hold a note, sweep `Level` fast from 1.00 to 0.00 | **A click or zipper artifact is possible and is not a defect against this spec** (§4.3, §8 Q1). Record what is heard for D-R3. |
| M7 | Set `Fall` to 1.0 ms, release a note | The release is short but is a ramp, not a click |
| M8 | Raise `Gloam`'s `Depth` to 1.00 while playing a dynamic phrase | Louder notes are brighter; quiet notes are darker; no ringing, no self-oscillation |
| M9 | Stop playing and leave the project open for a minute | Output is silent; the transport bar's health telemetry shows no drift in xruns or worst-case headroom |

Theme variants, screen-size extremes, and empty states: **N/A beyond M1–M4** — R4-6 adds no view,
no theme-sensitive surface, and no empty state of its own; the Shape empty state
(`SHAPE_EMPTY_MESSAGE`, `crates/spectre-app/src/lib.rs:92`) is unchanged and is reached by
selecting no device, not by these devices.

---

## 6. Compliance & Safety Gate

### 6.1 Sensitive data classification

- [x] **No sensitive data involvement.** Both devices process audio samples and note events in
      memory. Nothing is stored, transmitted, logged, or displayed beyond parameter values the
      user set. No network, no filesystem, no telemetry beyond the counters that already exist.

### 6.2 Asset provenance

- [x] **No third-party assets.** No wavetable, sample, impulse response, lookup table, font,
      image, or dataset. Both devices are closed-form arithmetic over `core`/`std` math. No code
      is ported, adapted, or transcribed from any product, plugin, tutorial, or paper; the phase
      map in §4.3 and the level-tracked coefficient in §4.3 are written out in full above so the
      claim is checkable rather than assertable.

### 6.3 Language / claims audit

- [ ] Makes claims not supported by evidence — **no, and the checks are named.** No listening
      claim, no CPU claim, no latency claim, no anti-aliasing claim, no cross-platform bit-equality
      claim, no Linux claim. Aliasing is stated as a limitation (§4.3); libm cross-platform
      variance is stated (§4.6); the absence of measurement is stated (§4.7).
- [ ] Promises capabilities not yet built — **no.** §1.2 and §5.4 state that `./spectre` produces
      no sound and that R4-1 and R4-2 must land first. Every audible outcome in §3.2 and §5.4 is
      marked conditional.
- [ ] Uses language restricted by domain regulations — **no.** No health, financial, safety, or
      accessibility-conformance claim appears; §3.7 explicitly claims **no** screen-reader support.

**AF-6 self-audit.** The words "should pass", "expected to", "seamless", "instantly",
"effortless", "professional-grade", and "production-ready" appear nowhere about this feature's
state. Where a benefit is stated (§1.1, §2), it is stated as a job the feature is *for*, not as a
capability the repository has.

### 6.4 Regulatory alignment — `criteria.md` Lens 3, criterion by criterion

- **3A Milestone fit.** R4 as scoped is *"one track, MIDI clip, native synth + effect, transport,
  save/reload, offline bounce"* (`docs/00-product/vision.md:48`;
  `docs/06-plans/current-milestone.md:12`). This is the "native synth + effect" item exactly, and
  the R4 exit row is `current-milestone.md:84`. Everything adjacent is deferred by name: track
  hosting to R4-4, clip playback to R4-5, persistence to R4-7, bounce equivalence to R4-8, QA
  protocol to R4-9.
- **3B Non-goal respect** (`docs/00-product/vision.md:53–59`;
  `docs/06-plans/current-milestone.md:77`). No CLAP/LV2/AU hosting and **no plugin-format
  authoring of these devices** — both are plain Rust types inside `spectre-dsp` reachable only
  through `AudioProcessor`. No cross-DAW preset or project compatibility: R4-6 defines no preset
  format at all. No cloud service, no content store, no video scoring. **No flagship synth** —
  see 3C. No automation and no modulation, which `current-milestone.md:77` also lists as R5+.
- **3C Deliberately small first devices** (decision 15). Seven parameters between two devices.
  One voice, one oscillator, one pole. No unison, no filter bank, no LFO, no modulation matrix,
  no effect lanes, no wavetable, no sample player, no granular engine, no preset browser, no
  visualizer. §0 is the audit trail: the four prohibited conclusions a device spec would reach for
  are named and refused, and the four questions they raise are routed to §8 rather than answered.
  The devices grow at R11, not here.
- **3D Originality** (`docs/00-product/vision.md:42`). The names `Filament` and `Gloam` appear
  nowhere in `docs/` or `crates/` and are not any benchmark's device, style, or mode name. The
  parameter vocabulary — Lean, Rise, Fall, Damp, Depth, Track — is Spectre's own; where a word is
  reused it is reused from **Spectre's** existing tables (`level`, from
  `crates/spectre-dsp/src/source.rs:36`, `:39–46`), not from a product. The DSP is written out in
  full in §4.3 and is not transcribed from anything. Every numeric bound has a rationale row in
  §4.2 derived from this repository's own values or stated as a chosen envelope with a re-open
  trigger — never from a reference product (AF-4, decision 16).
- **3E Platform commitment** (decision 1). §4.6: no `cfg`, no intrinsic, no `unsafe`, portable
  denormal handling, and all evidence in §5 runs with no audio hardware on either platform. This
  spec authorizes no Linux support claim and does not touch decision 23's debt.
- **3F Accessibility trajectory** (decision 17). §3.7: nothing is foreclosed; every control is a
  named, ranged, defaulted descriptor row; no color-only or icon-only state exists; the one real
  risk — terse labels — is named and routed to §8 Q7 ahead of the R4 audit.

---

## 7. Gap Analysis vs. Current State

### 7.1 What exists today

Every claim below was checked by opening the file at the cited line.

**Implemented — the four native devices.**

- `ToneSource` (`crates/spectre-dsp/src/source.rs:50–100`): a deterministic stereo sine with two
  parameters, `frequency_hz` (`20 … 20 000`, default `440`) and `level` (`0 … 1`, default `0.25`),
  declared at `:27–37`. Phase in `f64`, advanced with `.fract()` at `:96`. **No inherent
  setters** — `impl ToneSource` at `:56–74` contains `new` and `parameters` only.
- `PulseInstrument` (`:112–215`): monophonic, note-driven, **one** parameter — `level`
  (`0 … 1`, default `0.2`) at `:39–46`. Its waveform is a four-variant enum chosen at construction
  (`:103–109`) with a getter (`:139–141`) and no setter. Its shapes are naive: triangle at `:147`,
  saw at `:148`, square at `:149–155`. There is **no amplitude envelope**: `:202–209` emits the
  oscillator value while a note is active and exactly `0.0` otherwise, so every note begins and
  ends on an amplitude step. Note handling at `:176–201` matches note-off by ID (`:188–193`),
  ignores non-matching offs (`:198`), and honors `AllNotesOff` (`:194–197`).
- `Gain` (`crates/spectre-dsp/src/effect.rs:29–74`): **a single `f32` field** (`:29–31`) with an
  instantaneous clamped setter (`:45–47`) and **no smoothing state of any kind**; the process loop
  at `:62–71` multiplies by it directly and maps non-finite input to `0.0` at `:65–69`. One
  parameter, `gain` (`0 … 2`, default `1`), at `:19–20`.
- `Saturator` (`:78–133`): `tanh` soft clip normalized by `drive.tanh()` (`:119`) with a dry/wet
  blend (`:128`) and input containment at `:122–126`. Two parameters, `drive` (`1 … 24`, default
  `1`) and `mix` (`0 … 1`, default `1`), at `:22–25`.

**Implemented — the contract these devices satisfy.** `AudioProcessor`
(`crates/spectre-dsp/src/io.rs:163–172`) has exactly two methods, `io` and `process`, plus a
`Send` bound. `DeviceIo` (`:15–21`) declares class, input count, output count, and note
acceptance. `ProcessContext::new` (`:77–107`) validates sample rate (`:82–84`), frame count
(`:85–87`), event capacity against `MAX_NOTE_EVENTS_PER_BLOCK = 1_024` (`:52`, `:88–90`), and
event validity and ordering (`:91–101`). `validate_buffers` (`:175–198`) checks channel counts
and buffer lengths and rejects events on a device that does not accept them (`:194–196`).

**Implemented — parameter descriptors.** `DspParameter` (`crates/spectre-dsp/src/parameter.rs:31–35`)
wraps a `spectre_core::ParamSpec`; `validate` (`:42–46`) rejects out-of-range and non-finite
input, `clamp` (`:49–51`) contains it, and `minimum`/`maximum`/`default` are `pub const fn`
(`:66–76`). `parameter()` is a `pub(crate) const fn` (`:128–145`) that **panics at const
evaluation** on an invalid spec (`:140–143`). `ParamSpec::new` is itself `const` and rejects
`max <= min` (`crates/spectre-core/src/param.rs:68–89`, specifically `:77–79`); `ParamSpec::clamp`
maps non-finite input to the descriptor default (`:119–124`).

**Implemented — RT-003 containment.** `contain_channel`
(`crates/spectre-graph/src/lib.rs:399–414`) flushes denormals to signed zero with the predicate
at `:407–410` and reports contamination; the plan applies it to every node's output before that
output can reach a downstream device (`:510–521`), silencing the whole node and recording it. It
operates on **output buffers only** and never sees a device's internal state.

**Implemented — the offline harness.** `render_plan`
(`crates/spectre-offline/src/lib.rs:204–285`) compiles the fixed `PulseInstrument → Gain →
Saturator` chain (`:215–258`) and computes peak and an FNV-1a hash over the planar output
(`:269–284`, seed at `:271`, prime at `:276`). `render_vertical_slice` (`:288–303`),
`render_silence` (`:319–334`), and `render_app_snapshot` (`:306–316`) are its three entry points.
`DeviceValues::from_snapshot` refuses any snapshot whose length is not exactly four (`:45–47`).

**Implemented — the app's device surface.** `AppModel::prototype`
(`crates/spectre-app/src/lib.rs:218–262`) builds exactly three `DeviceControl`s — `pulse`,
`gain`, `saturator` — at `:228–250`, each from its `const` descriptor table.
`device_parameter_snapshot` (`:348–383`) iterates a **hardcoded four-entry array** at `:351–356`,
not the device vector.

**Implemented — the RT-001 guards.** `crates/spectre-audio/tests/rt_guard.rs` carries an
allocator-attributing guard with a positive control (`:138–155`) and drives a plan through a real
backend callback inside it (`:259`). Its structural lock scan (`:289–320`) reads four
`spectre-audio` module paths (`:293–298`) joined to that crate's own manifest directory (`:309`),
so **it does not and cannot cover `crates/spectre-dsp/src/`.**

**Absent.**

- `Filament`, `Gloam`, `FILAMENT_PARAMETERS`, `GLOAM_PARAMETERS` — the strings appear nowhere in
  `docs/` or `crates/`.
- Any amplitude envelope, any filter, any per-parameter smoothing, and any modulation source on
  any shipped device.
- `AudioProcessor::set_parameter` — the trait has two methods (`crates/spectre-dsp/src/io.rs:163–172`).
  R4-2 is spec'd and passed at 3.000; it is **not implemented**.
- Any in-crate `#[cfg(test)]` module in `spectre-dsp`, `spectre-graph`, `spectre-audio`, or
  `spectre-offline` — those crates test through `tests/` only, and R4-6 follows that convention.
- Sound from `./spectre`, which does not depend on `spectre-audio` at all.

**Gated.** Live audibility of both devices is gated on R4-1; live parameter control of both is
gated on R4-2; Linux qualification is gated on R4-3 and decision 23.

**Contract-versus-code discrepancy, carried not resolved.**
`docs/03-architecture/dsp-device-io.md:94` states *"`Gain` already smooths"* and `:104` describes
`Gain` as *"stereo linear gain with click-resistant smoothing."* The shipped `Gain` has no
smoothing state (`crates/spectre-dsp/src/effect.rs:29–31`). This is **D-R3**
(`gauntlet-output/decisions-needed.md`, entry **D-R3**), raised by R4-2 and confirmed at blind verification.
It is open. R4-6 does not resolve it, does not add smoothing to `Gain`, and does not restate the
document's claim as though it were true; §4.3 states what the two **new** devices do, and §8 Q1
routes the shared question.

### 7.2 Delta to spec

**New files**

1. `crates/spectre-dsp/src/filament.rs` — `Filament`, `FILAMENT_PARAMETERS`, `FILAMENT_IO`.
2. `crates/spectre-dsp/src/gloam.rs` — `Gloam`, `GLOAM_PARAMETERS`, `GLOAM_IO`.

Both carry Jeff's header block and follow the existing device modules' comment discipline.

**Modified files**

3. `crates/spectre-dsp/src/lib.rs` — two `mod` lines beside the four at `:6–9`; two `pub use`
   lines exporting `Filament`, `FILAMENT_PARAMETERS`, `Gloam`, `GLOAM_PARAMETERS`.
4. `crates/spectre-app/src/lib.rs` — two `DeviceControl::from_descriptors` entries appended in
   `prototype()` (`:228–250`), reusing the existing class subtitles at `:233` and `:240`.
   **`device_parameter_snapshot` is not touched**, which is what keeps the offline fixture at four
   entries (§4.4).
5. `crates/spectre-offline/src/lib.rs` — add `pub fn render_voice_chain(sample_rate, frames)`
   building `Filament → Gloam`, and extract the peak/hash walk at `:269–284` into a private
   helper both it and `render_plan` call, so criterion 1D's "the existing FNV-1a walk, not a new
   one" is satisfied structurally rather than by promise. **`render_vertical_slice`,
   `render_silence`, `render_app_snapshot`, and `DeviceValues` are behaviorally unchanged**, and
   the three hand-wired gates that already exist in `crates/spectre-offline/tests/harness.rs`
   (`:57–58`, `:172–173`, `:220–221`) prove it, because each recomputes its own peak and FNV-1a
   reference inside the test crate rather than through the helper (§5.2). This slice adds no
   fourth test for that, and the earlier `existing_fixture_hash_is_unchanged` is withdrawn.
6. `crates/spectre-dsp/tests/devices.rs` — §5.1 tests 1–13; extend `native_parameters()` (`:37`)
   and the layout test (`:186–187`).
7. `crates/spectre-graph/tests/containment.rs` — one sibling test (§5.2).
8. `crates/spectre-offline/tests/harness.rs` — two tests (§5.2).
9. `crates/spectre-audio/tests/rt_guard.rs` — one allocation-guarded plan test (§5.2).
10. `crates/spectre-app/tests/app_model.rs` — three tests (§5.3).
11. **`docs/01-requirements/requirements-ledger.md`** — a new `DEV` family carrying rows
    **DEV-001 … DEV-012** exactly as tabulated in §4.2, each in the ledger's own format
    (`:21–22`: `ID | requirement | provenance | acceptance evidence | status`), each `proposed`
    until Jeff accepts. This is what discharges PROD-003 (`:64`) and decision 16 by record rather
    than by prose, and the ledger's own known-gaps line (`:19`) already anticipates families
    arriving at their milestone intakes. **Routed as a proposal — §8 Q3.**
12. **`docs/03-architecture/dsp-device-io.md`** — extend the "Initial native devices" list
    (`:98–107`) with entries 5 and 6, keeping the closing sentence at `:107` verbatim, since it is
    the sentence that keeps the list from becoming the prohibited native-device catalog. **Routed
    as a proposal — §8 Q2.** This spec does **not** propose correcting `:94` or `:104`; that is
    D-R3's to resolve.
13. `docs/status/STATUS.md`, `docs/status/NEXT.md`, `docs/01-requirements/traceability.md` — updated
    when the slice lands, per `docs/README.md:68–70`. Not updated by this spec.

**Migrations / schema changes:** none. **New dependencies:** none.

### 7.3 Estimated scope

**M.** Two self-contained modules of roughly 120–160 lines each with no dependency beyond
`std` math and the crate's own descriptor helper; two lines of app data; one contained refactor
in `spectre-offline`; about twenty tests across five files. The DSP is closed-form and every
correctness argument in §4.3 is short enough to check by reading.

What keeps it from being S: the numeric-bound work is real. Twelve rationale rows in an accepted
document, each of which must be defensible on its own and two of which (DEV-003, DEV-010) carry
re-open triggers. What keeps it from being L: no new architecture, no new trait, no new lane, no
new error type, no allocation, no asset, and no UI beyond two list entries.

### 7.4 Blocking dependencies

| Dependency | Why | State |
|---|---|---|
| **R4-2 runtime parameter seam** | Both devices must implement `AudioProcessor::set_parameter`, and that method does not exist. Without it both devices compile but every control is dead, which would be a fake surface (4F). **Hard blocker for the setters; not for the DSP.** | spec passed at 3.000; **zero lines implemented** |
| **R4-1 live audio wiring** | Nothing in §3.2's primary flow or §5.4 is observable without it, because `./spectre` produces no sound. **Blocks audibility and the manual protocol; blocks none of §5.1–§5.3**, which run offline. | spec passed at 2.950; **zero lines implemented** |
| D-R3 (`decisions-needed.md`) | Does not block. §4.3 states both devices' behavior without resolving it; §8 Q1 routes it. | open |
| §8 Q3 (the `DEV` ledger family) | Blocks the *record*, not the code. The bounds are defensible as written; where they live is Jeff's call. | proposed |
| R4-4 track model, R4-5 MIDI clips | Do **not** block. Both devices are exercised through the existing plan and the existing note path; a track is where they will eventually live, not what makes them work. | downstream |

---

## 8. Open Questions

- **Q1 — Smoothing policy: one rule for all devices, or per device?** `Filament`'s contour
  minimum (DEV-003) means it cannot step on note attack or release, and `Gloam`'s parameters
  reach the signal only through a recursion, but **neither device smooths a parameter change**,
  and `level` in particular can click on a fast move. The accepted contract says smoothing is the
  device's concern (`docs/03-architecture/dsp-device-io.md:59`) *and* asserts a smoothing on
  `Gain` that does not exist (`:94`, `:104`) — the open **D-R3**. Does Spectre want one
  workspace-wide parameter-smoothing policy with a single rationale row, or per-device answers?
  A workspace policy needs a time constant, which is a numeric bound; a per-device answer needs
  one per device. Either changes rendered output and therefore every hash-equivalence claim, so
  it should be decided once. **Blocks:** §4.3, and D-R3's disposition.
- **Q2 — Where does the native-device list live, and does it stay non-normative?** §7.2 item 12
  proposes appending to `docs/03-architecture/dsp-device-io.md:98–107`. That list is currently a
  contract-proving inventory whose closing sentence (`:107`) explicitly disclaims being a
  catalog. Adding devices to it is safe only while that sentence stays. Should R4-6's devices go
  there, or should the architecture document stop enumerating devices entirely and leave the
  list to status? **Blocks:** §7.2 item 12. **Related to** the prohibited "final native-device
  list" (`product-implications.md:94`).
- **Q3 — A new `DEV` requirements-ledger family?** Twelve rows is the honest cost of PROD-003
  (`requirements-ledger.md:64`) for two devices. Confirm the family name, whether per-parameter
  granularity is right or whether one row per device with a bounds table is better, and the
  status each row carries at R4 exit. **Blocks:** §7.2 item 11.
- **Q4 — When does `Filament` become polyphonic, and by what policy?** DEV-010 ships one voice.
  R4-5's MIDI clips can produce overlapping notes, and a user will expect to hear them. Polyphony
  needs a voice count *and* a stealing policy — `OBS-PP-UNI-003`
  (`synth-modular-observations.md:37`) shows a benchmark treating both as documented product
  behavior. Neither belongs in R4 by decision 15, but the question should be dated rather than
  discovered at R4-5. **Blocks:** nothing in R4-6; informs R4-5 and R11.
- **Q5 — Band-limiting and cross-platform bit-equality: which one matters first?** `Filament`'s
  warp aliases (§4.3), as `PulseInstrument` already does. Separately, `sin`/`exp`/`tanh` are libm
  calls, so bit-identical output across macOS and Linux is **not** guaranteed for any Spectre
  device today, and R4-8's bounce-equivalence claim will run into this before R4-6's aliasing
  does. Should the project (a) add band-limited oscillators before beta, (b) pin a portable math
  implementation so hashes are cross-platform, (c) both, or (d) scope determinism to
  within-a-build and say so in the architecture contract? **Blocks:** §4.6's determinism claim,
  and R4-8's equivalence framing.
- **Q6 — Does any device get a custom view before beta?** §3.1 gives both devices plain
  descriptor rows and no waveform display, no meter, no curve editor. That is right for R4 and
  probably wrong for a synth long-term. When it changes it should change with a decision row,
  because a per-device view is a new surface with its own repaint and accessibility cost.
  **Blocks:** nothing now.
- **Q7 — Should descriptors carry a longer description field?** `Lean`, `Damp`, and `Track` are
  terse labels that read poorly aloud (§3.7). `DspParameter` carries `key`, `name`, and `spec`
  (`crates/spectre-dsp/src/parameter.rs:31–35`) and no description. Adding one touches CORE-002
  (`requirements-ledger.md:47`) and every device. Worth deciding before decision 17's R4 audit
  rather than during it. **Blocks:** §3.7's naming obligation.
- **Q8 — Should the RT-001 structural lock scan reach across crates?** It reads four
  `spectre-audio` modules joined to that crate's manifest directory
  (`crates/spectre-audio/tests/rt_guard.rs:293–298`, `:309`), so no device module has ever been
  covered by it. The allocation guard is stronger and does cover devices, but the gap is real and
  currently invisible. Widen the scan, move it to a workspace-level test, or record the scope
  limit in the guard's own header? **Blocks:** nothing; R4-6 does not widen it unilaterally.
- **Q9 — Should the four `DeviceIo` layout constants be hoisted?** `GLOAM_IO` restates
  `EFFECT_IO` (`crates/spectre-dsp/src/effect.rs:12–17`) exactly, and `FILAMENT_IO` restates
  `INSTRUMENT_IO` (`crates/spectre-dsp/src/source.rs:20–25`) exactly. R4-6 restates rather than
  refactors, to keep the diff surgical. With five and six devices that becomes four duplicated
  constants. Hoist them into `io.rs` as `pub const`s in a later slice? **Blocks:** nothing.

---

## Appendix A — Benchmark evidence, cited or named as absent

The AAA benchmark set is Ableton Live, Logic Pro, Serum 2, Phase Plant, and VCV Rack 2. This is
what the corpus does and does not support for a first-devices spec.

| Claim in this spec | Evidence | Where |
|---|---|---|
| A live parameter move is a real click risk in a shipping DAW | `OBS-AB12-MIX-008` — a benchmark warns against changing millisecond-scale track delays on stage because of clicks and pops | §3.2, §4.3 |
| Automated-vs-overridden parameter state is a convergent pattern Spectre must not foreclose | `OBS-AB12-AUTO-004` — manual change while not recording overrides automation, LED dims, explicit re-enable command; PROD-002's provenance (`requirements-ledger.md:63`) | §3.3 |
| An `f32` engine tolerating internal over-0 dB is normal practice, and Spectre's pair does not exercise it | `OBS-AB12-MIX-002` | §4.3 |
| Polyphony brings a voice-stealing policy that is product behavior, not an implementation detail | `OBS-PP-UNI-003` | DEV-010, §8 Q4 |
| Per-voice duplication is the cost center; shared post-FX applied once is the cheaper structure | `OBS-SR2-CPU-001` — **numbers not adopted** (AF-4) | §0, DEV-010 |
| A device should output zero on NaN/infinity rather than propagate it | `OBS-VCV-VOLT-006`, RT-003's provenance (`requirements-ledger.md:30`) | §4.3, §5.1 test 7 |

**Named gaps — the corpus is silent and this spec does not fill it.**

1. **No citable record for any benchmark on how a first-party device is implemented internally**
   — oscillator topology, envelope shape, filter order, coefficient update rate, or smoothing
   scheme. The Phase Plant records describe an exposed module architecture
   (`synth-modular-observations.md:25–31`), which is a product surface, not an implementation.
   Every DSP choice in §4.3 is Spectre's own and is recorded as a research need.
2. **No citable record for any benchmark on oscillator anti-aliasing.** §4.3 states the
   limitation and §8 Q5 routes it; no target is asserted.
3. **Serum 2: exactly two records exist**, `OBS-SR2-CPU-001` and `OBS-SR2-KB-001`
   (`synth-modular-observations.md:54–55`). The dossier is `blocked-source-gap` and states it
   cannot be accepted for product planning. **No Serum 2 claim beyond those two appears in this
   spec.**
4. **Logic Pro: zero behavioral records.** `docs/02-reference-research/logic-pro.md:10–11` is
   `draft` / `inventory-only`. **No Logic Pro claim appears in this spec.** Naming the hole is the
   correct behavior; filling it would be fabrication.
5. **VCV Rack's signal conventions do not transfer.** `OBS-VCV-VOLT-001` (±5 V nominal audio) and
   `OBS-VCV-VOLT-004` (1 V/oct) describe a voltage-typed modular contract; Spectre's devices use
   normalized `f32` and equal-tempered note numbers, and `docs/00-product/vision.md:39` commits to
   an original signal contract informed by rather than copied from VCV. **2E disposition:**
   Spectre diverges deliberately, and the divergence is already accepted product direction rather
   than this spec's invention.

**Differentiation (2F), without claiming parity as completeness.** What R4-6 does that the
benchmark set is not documented as doing: every numeric bound in both devices is published with
its derivation and a re-open trigger, in a requirements ledger, before the device ships — DEV-002
and DEV-007 are derived rather than chosen, and DEV-003 and DEV-010 carry explicit re-open
conditions. That is a statement about what Spectre publishes, **not** a claim that these devices
sound better than, or are more capable than, anything in the benchmark set. They are smaller than
all of it, on purpose, by decision 15.

---

**End of spec.**
