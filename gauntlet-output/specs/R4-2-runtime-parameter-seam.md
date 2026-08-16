<!--
Author: Jeff
Date: 2026-08-15
Description: R4-2 spec — implement decision 22's callback-safe AudioProcessor parameter setter and connect the RT-002 parameter lane the bridge already drains
Notes: Implements the accepted "Runtime parameter seam" contract; it does not redesign it. Option (b),
  recompiling a plan per change, is rejected by decision 22 and is not proposed anywhere here. Two
  facts drove the shape of this spec and are stated rather than smoothed over: the shipped `Gain` has
  no smoothing state despite the architecture contract saying it does, and this slice modifies two of
  the four RT-scanned modules, so its RT-001 argument is made on the content of those edits and on a
  post-edit `rt_guard` run, never on an untouched-set claim. `./spectre` makes no sound today and
  R4-1, which changes that, is spec'd but not implemented.
-->

# Spec: Runtime Parameter Seam

**Feature ID:** `R4-2` (`runtime-parameter-seam`)
**Parent feature:** `R4` Credible Alpha (root)
**Spec author agent:** gauntlet spec agent, R4-2 leaf
**Date:** 2026-08-15
**Iteration:** 1

- **Status:** proposed
- **Last verified:** 2026-08-15 (source read at commit `2e005e5`, branch `rename/geist-to-spectre`)
- **Scope:** the callback-safe parameter setter on `AudioProcessor`, its routing through `CompiledPlan` and `RenderBridge`, and the app-thread publication path from a Shape slider into the RT-002 parameter lane
- **Decision authority:** Jeff
- **Upstream sources:** `docs/03-architecture/dsp-device-io.md` §"Runtime parameter seam" (the design authority for this slice), `docs/01-requirements/decision-gates.md` rows 15, 16, 17, 21, 22, 23, `docs/01-requirements/requirements-ledger.md` (RT-001/002/003, CORE-002, GRAPH-001, PROD-002, PROD-003), `docs/00-product/vision.md`, `docs/06-plans/current-milestone.md` §"Inherited debt" item 2, `docs/status/NEXT.md` slice 2, `gauntlet-output/specs/R4-1-live-audio-wiring.md`
- **Downstream dependents:** R4-4 (track model — every added device registers parameter targets through this seam), R4-6 (first devices — a new device must implement the new trait method), R4-8 (offline bounce — live/offline equivalence is asserted against a parameter set this slice makes mutable), R4-9 (e2e and QA), and R9 sample-accurate automation, which extends this seam rather than replacing it
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** §8 Q1–Q8. Q1 (the `Gain` smoothing discrepancy) is a conflict between an accepted architecture document and the shipped code and is logged for `gauntlet-output/decisions-needed.md` as **D-R3**; this spec does not resolve it by assertion.
- **Known gaps:** no benchmark in the accepted corpus has a citable observation about a DAW's internal control→render parameter transport, its block-boundary application granularity, or its failure behavior when a parameter cannot reach a live processor. The corpus documents what users *see* when they move an automated control, not how the value crosses to the audio thread. Every transport-level decision here is Spectre's own and is recorded as a research need in Appendix A and §8 Q7.

This spec is subordinate to the conflict precedence in `docs/README.md`. It **implements**
the accepted contract in `docs/03-architecture/dsp-device-io.md` §"Runtime parameter
seam"; it does not amend it. Where the shipped code and an accepted document disagree
(§7.1, "Contract-versus-code discrepancy"), the spec says so explicitly and routes the
question to §8 rather than choosing for Jeff.

---

## 1. Purpose

### 1.1 One-sentence job

When a musician drags a Shape slider while `./spectre` is making sound, the sound
changes — on the next block, through the same immutable compiled plan that is already
playing, with nothing allocated, locked, or recompiled on the audio thread.

### 1.2 Why it matters

This is the one gap that makes Spectre's device surface a picture rather than an
instrument. Every other piece of the path already exists and is tested: the app clamps
and stores the edit (`crates/spectre-app/src/lib.rs:385–403`), the RT-002 parameter lane
carries it latest-wins with a version counter (`crates/spectre-audio/src/control.rs:66–145`),
and the bridge drains it once per block in the right position — after transport and notes,
before `plan.process` (`crates/spectre-audio/src/bridge.rs:175–195`). The value then
reaches a closure that throws it away:

```rust
// crates/spectre-audio/src/bridge.rs:178-186, verbatim
        // Parameter changes are observed but cannot reach live processors yet: the accepted
        // AudioProcessor contract has no runtime parameter seam. Counting them keeps the gap
        // visible instead of silently discarding edits.
        let pending = self.control.drain_parameters(|_, _| {});
        if pending > 0 {
            self.telemetry
                .parameters_pending
                .fetch_add(pending as u64, Ordering::Relaxed);
        }
```

`|_, _| {}` is the whole feature. The reason it is empty is `AudioProcessor`, which has
exactly two methods — `io` and `process` (`crates/spectre-dsp/src/io.rs:163–172`) — and
no way to change a value on a processor that is already inside a compiled plan. Decision
22 recorded that as a discovered blocker at R3 slice 4, accepted option (a) on
2026-08-09, and assigned implementation to R4.

**Stated plainly, and it must stay stated until it is false: `./spectre` produces no
sound of any kind today.** Pressing Play mutates an in-memory `Transport`
(`crates/spectre-app/src/lib.rs:268–275`) and nothing else; the transport bar prints the
hard-coded string `ENGINE OFFLINE` (`crates/spectre-app/src/main.rs:79`);
`crates/spectre-app/Cargo.toml`'s `[dependencies]` are exactly `eframe`, `spectre-core`,
`spectre-dsp`, `spectre-project`, `serde`, `serde_json` — no `spectre-audio`. R4-1 is the
slice that changes that, and R4-1 has **passed its spec gate (2.950) and has zero lines of
implementation** (`gauntlet-output/manifest.md`, R4-2's own row records "Impl iter 0" for
every feature). R4-2 is therefore a spec written against a live path that does not exist
yet, and §7.4 makes R4-1 a hard blocking dependency rather than a footnote.

`docs/06-plans/current-milestone.md` §"Inherited debt" item 2 states the consequence
directly: *"A playable alpha needs this closed."* The R4 exit evidence row is *"Decision
22's parameter seam is implemented, so a UI edit changes live audio."*

### 1.3 Success signal

With `./spectre` running and audible, dragging the Shape `Gain` slider from `1.00` to
`0.20` audibly attenuates the output within one block period, and the transport bar's
parameter counters read `applied ≥ 1` with `pending 0`. The automated form of that signal
is §5.1 test 4: pump a block, assert a nonzero peak; publish a new gain value; pump again;
assert the second block's peak is lower by the ratio the two gain values imply, that
`BridgeTelemetry::parameters_applied()` advanced, and that
`BridgeTelemetry::parameters_pending()` is still `0`. The `pending == 0` assertion is the
one that encodes the acceptance criterion, and it fails today by construction.

---

## 2. User Stories

> As an electronic musician shaping a sound, I want to hear a parameter change while the
> transport is running, so that I am designing by ear instead of by numbers and a reload.

> As that same musician, I want a fast slider drag to be delivered as the value I ended on
> rather than as a queue of stale values that arrive late, so that the sound tracks my hand
> and never lags behind it.

> As a musician whose edit could not reach the engine — the device is not in the live plan,
> or the engine is not running — I want the app to keep the edit in the project and tell me
> plainly that live audio did not receive it, so that a silent divergence between what I see
> and what I hear is impossible.

> As a musician who has just moved a gain control on a sounding voice, I want to know
> whether Spectre will click, so that I am not surprised mid-take by an artifact the tool
> knew about. (§3.2 and §5.4 tell the truth here rather than promising smoothness this slice
> does not implement — see §8 Q1.)

> As a keyboard-only or screen-reader user, I want the parameter counters and any
> "not applied to live audio" state to be text, focusable, and legible without color, so
> that decision 17's beta accessibility bar is not foreclosed by this slice.

> As Jeff running CI on a headless container, I want the whole parameter path — publish,
> route, apply, containment, refusal — to be provable against the null backend with no
> device present, so that the gate is deterministic and a false pass is impossible.

> As the maintainer of R4-6, I want a new device to be *forced* to answer the parameter
> seam rather than silently ignoring edits, so that the first original synth cannot ship
> with dead controls.

---

## 3. UX Specification

### 3.1 Screen / view inventory

R4-2 introduces **no new screen, modal, sheet, popover, drawer, or panel**. It modifies
two regions of the single-window shell and changes the behavior behind one existing
control.

| Region | Navigation path | New or modified | Layout pattern |
|---|---|---|---|
| Shape lens parameter rows (`shape_devices()`, `crates/spectre-app/src/main.rs:336–398`) | lens selector → Shape | modified — behavior and header copy; the widgets themselves are unchanged | scroll area, one horizontal row per parameter |
| Transport-bar counter cluster (added by R4-1 at `main.rs:78–81`'s right-to-left layout) | always visible from all four lenses | modified — two counters added to R4-1's cluster | full-width top panel, trailing text run |
| Inspector status line (`main.rs:210`, driven by `feedback_status`) | always visible | modified — reused, not restructured, for per-edit refusals | side panel body text |

Nothing is added to Arrange, Build, or Mix. Build already shows each device's parameter
values as read-only text (`main.rs:297–310`); those values keep tracking the model, which
is now the same value the engine has, so Build needs no change of its own.

The counters live in the transport bar for the reason R4-1 established and this spec
inherits: it is the only region visible from all four lenses, which keeps engine state one
global fact rather than a per-lens fork (`vision.md`, "One project, linked lenses").

### 3.2 Interaction flows

**Primary flow — a Shape edit reaches live audio.**

1. The engine is running (R4-1's `EngineState::Running`), so blocks are being rendered and
   the transport bar shows a nonzero `blocks` count.
2. The user drags a slider in Shape. egui reports a changed value; the existing code
   collects it into `edits` (`main.rs:380–382`) and applies it after the scroll area closes
   (`main.rs:389–397`).
3. The app calls `apply_parameter_edit` (§4.3), which:
   a. applies the edit to `AppModel`, where it is clamped by the parameter's own descriptor
      (`crates/spectre-app/src/lib.rs:401`, `crates/spectre-dsp/src/parameter.rs:49–51`);
   b. reads back the **stored, clamped** value together with the parameter's stable
      `(device_instance_id, parameter_instance_id)` pair;
   c. publishes that value to the RT-002 latest-wins slot for that target.
   The order is binding: **the model is the source of the value, and only a value the model
   accepted is ever published.** Publishing the raw widget value instead would let live
   audio and offline rendering hold different numbers for the same parameter, which is
   precisely the divergence R4-8's bounce equivalence must not have to reconcile.
4. `ParameterWriter::set` stores the bits and then bumps the version
   (`control.rs:99–100`). It never blocks and never fails for a registered target.
5. On the next block the bridge drains the lane before executing the plan
   (`bridge.rs:181`, unchanged position), resolves the target to `(NodeId,
   DeviceParameterKey)` through the preallocated route table, and calls
   `CompiledPlan::set_parameter`, which calls the processor's new
   `AudioProcessor::set_parameter`.
6. The block renders with the new value. **The whole block sees one coherent parameter
   set**, because application happens once, before `process`, exactly as the accepted
   contract requires.
7. The transport bar's `params applied` counter advances on the next UI frame.

**Latency, stated as a fact rather than a target.** The value takes effect on the first
block that begins after the store lands. At R4-1's requested 256 frames and 48 kHz that is
under 5.33 ms plus one UI frame of the app's existing 250 ms repaint cadence
(`main.rs:443`) — but the repaint cadence affects only when the *counter* updates, not
when the *sound* changes, because the store happens in the input handler, not in the
repaint. **No latency threshold is asserted anywhere in this spec.** A monitoring-latency
threshold is one of the conclusions `docs/02-reference-research/workflow-field-study/product-implications.md`
§"Prohibited conclusions at current evidence level" forbids at this evidence level, and
Spectre has exactly one hardware measurement (macOS, 2026-08-09, worst-case headroom
0.990).

**Branch — a fast drag.** egui can report a changed value on every frame of a drag. Each
one overwrites the same latest-wins slot; the render thread applies whatever is there when
it drains. This is decision 21's accepted policy and it is already proven: `control_channel.rs:45`
`parameter_sweep_coalesces_to_one_latest_value` covers a 5,000-write sweep collapsing to
one application. R4-2 adds no throttle, no debounce, and no rate limit, because the lane's
own policy already makes them unnecessary and any of them would be a numeric bound with no
evidence behind it.

**Branch — the engine is not running.** `apply_parameter_edit` applies the edit to the
model and skips publication. The Shape header states the current condition (§3.3) and the
transport bar already reads `Engine unavailable — …` from R4-1. No error is raised for the
ordinary case of editing with no engine: that is the prototype's normal state today and
must stay usable.

**Branch — the target is not in the live plan.** R4's fixture registers exactly four
targets (§4.4). A device or parameter outside that set has no slot, so
`ParameterWriter::set_target` returns `ControlError::UnknownTarget`
(`control.rs:105–110`). The app surfaces it on the inspector status line and **does not**
roll back the model edit — the edit is valid for offline rendering and for the next engine
start; only live delivery failed. §3.6 E2.

**Branch — the render thread cannot route or apply the value.** Counted, never silent:
`parameters_pending` increments and the block still renders. §3.6 E4/E5 and §4.3's failure
table.

**Sound, haptic, and animation cues.** No new animation, no haptics, no UI sound. One
honest warning belongs in the copy and in the QA protocol: **a live parameter change is a
step at a block boundary, and the shipped `Gain` applies its value with no smoothing at
all** (`crates/spectre-dsp/src/effect.rs:29–31` is a single `gain: f32` field;
`:62–71` multiplies by it directly). A large, fast gain move on a sounding voice will
therefore produce a step discontinuity that can be audible as a click or zipper noise. The
accepted architecture contract says `Gain` already smooths; it does not. That discrepancy
is §7.1's "Contract-versus-code discrepancy" and §8 Q1, and this spec does **not** add
smoothing, because doing so changes rendered output and would break the bit-exact
live/offline hash equivalence the whole engine currently rests on (§4.3, "Why smoothing is
not in this slice").

### 3.3 Layout descriptions

**Shape lens, top → bottom (existing structure unchanged unless noted):**

1. Header line. Currently *"Controls derive directly from backend descriptors and preserve
   DSP ranges."* (`main.rs:337–342`). **Modified** — it gains one state-dependent second
   line, driven by `EngineState` and by whether the selected device is in the live plan:
   - engine running and device routed: *"Edits reach live audio on the next block."*
   - engine running, device not in the live plan: *"Edits apply to this project and to
     offline rendering. This device is not in the live signal path, so live audio will not
     change."*
   - engine not running: *"Edits apply to this project and to offline rendering. Start the
     engine to hear them."*
   This line is required, not decorative. R4-1 ships a Shape header that says live
   parameter application *does not exist*; leaving that string in place after R4-2 lands
   would be a false surface in the opposite direction.
2. Device frame — unchanged (`main.rs:348–356`).
3. One row per parameter — **unchanged widgets**: label, `egui::Slider` bounded by
   `descriptor.minimum()..=descriptor.maximum()` with the descriptor's unit and a hover
   text naming range and default (`main.rs:358–378`), and a `Reset` small button that sets
   the descriptor default (`main.rs:376–378`). Data source is `AppModel`'s
   `DeviceControl.parameters` (`crates/spectre-app/src/lib.rs:57–63`), which is unchanged.
   R4-2 changes **what happens after** the value changes, not the control.
4. Empty state — unchanged: `SHAPE_EMPTY_MESSAGE` (`crates/spectre-app/src/lib.rs:92`),
   *"No device selected. Open a device from Build to shape it."*

**Transport bar, trailing cluster (R4-1 owns this cluster; R4-2 adds to it):**

- `params applied <n>` — shown once nonzero. Data source: `EngineHealth.parameters_applied`,
  read from `BridgeTelemetry` on the app thread each frame.
- `params pending <n>` — shown **only** when nonzero, and when nonzero it must never be
  hidden. Data source: `EngineHealth.parameters_pending`. This is a defect counter: in a
  correctly wired build it stays at zero for the life of the process, which is the
  acceptance criterion made visible.

Both follow R4-1's established rule that a permanently-zero counter is noise while a
nonzero one is never suppressed.

**Modulation and automation state.** PROD-002 requires automated parameters to expose
distinct visible states for automated versus manually-overridden with an explicit restore
action. **R4-2 introduces no automation and no modulation**, so there is no automated state
to distinguish and no override to restore — every value in Shape is a base value and
nothing else writes to it. The `Reset` button is a return-to-default, not a
restore-automation action, and the copy must not imply otherwise. This is the correct
answer for this slice rather than a deferral: designing a base/automation/modulation
tri-state display before any modulation source exists would be inventing a surface. What
R4-2 *must* do is not foreclose it, which it does by keeping one value per parameter in one
place (`ParameterControl.value`) and by making the live path a publication of that value
rather than a second store — see §4.4.

### 3.4 Input & gestures

- **Slider drag / click**: pointer, and egui's standard keyboard interaction on a focused
  slider (arrow keys, and typing a value where egui supports it). Unchanged from today.
- **`Reset` button**: pointer click, and keyboard activation via egui's standard focus and
  Enter/Space handling. Its behavior is unchanged; it now also publishes, because it goes
  through the same `edits` path (`main.rs:376–382`).
- **Keyboard shortcuts**: **none are added, ratified, or documented.** The prototype binds
  `Space` and `1`–`4` (`main.rs:425–437`); those are pre-existing scaffolding that this
  spec neither extends nor blesses. `docs/02-reference-research/workflow-field-study/product-implications.md`
  §"Prohibited conclusions at current evidence level" lists *a default shortcut map* as
  unsupported by the current corpus, so fixing one here would be inventing evidence. When a
  command system exists it must be context-scoped and remappable, per `vision.md`'s
  keyboard-first pillar; the map comes after the model, not before it.
- **Specialized input** (stylus, controller, voice, camera, MIDI CC): N/A — R4-2 adds no
  such surface. MIDI parameter control is explicitly out of scope: `spectre_audio::midi`
  handles note messages only, and nothing in the app feeds it. A MIDI-learn path would need
  its own decision row.
- **Responsive behavior**: the Shape rows already reserve a 520 px minimum width and a
  360 × 26 px slider (`main.rs:362`, `:370`). R4-2 adds no width. The transport-bar
  counters are text and must truncate the device name before any counter, per R4-1's rule.

### 3.5 Transitions & animation

- Navigation transitions: none. No view is added or removed.
- In-view state change: the two new counters and the Shape header line change value on the
  existing 250 ms repaint cadence (`main.rs:443`). No fade, no easing, no motion, no new
  timing constant.
- Audio-domain "transition": the parameter change itself is a step at a block boundary and
  is deliberately not ramped in this slice (§3.2, §8 Q1). Calling that an animation would be
  a category error, but it is the one perceptible discontinuity R4-2 introduces and it is
  named here so it is not discovered by ear.
- Reduced motion: R4-2 introduces no animation, so a reduced-motion setting has no effect
  on this feature. That is the complete answer, not a deferral.

### 3.6 Error states

Presentation follows R4-1's split, deliberately: **persistent conditions go to the
transport bar; per-action failures go to the inspector status line** (`main.rs:210`), the
same channel `set_device_parameter_from_ui` already uses for a rejected edit
(`crates/spectre-app/src/lib.rs:462–474`). No modal and no toast is introduced: a modal
would block the workspace mid-drag, and a toast would disappear before a user finishes the
gesture that caused it.

| # | Trigger | Presentation | Recovery path | Data loss |
|---|---|---|---|---|
| E1 | Edit made while no engine exists (the normal state today) | Shape header line: "Edits apply to this project and to offline rendering. Start the engine to hear them." No error. | none needed | no |
| E2 | `ControlError::UnknownTarget` — the edited parameter has no registered lane slot (device outside the live fixture) | inspector: "Live audio did not receive `<device>.<parameter>`: `<Display>`. The edit is saved and applies to offline rendering." Shape header shows the not-in-live-path line. | none in-app for R4; the live plan's device set grows at R4-4/R4-6 | no — the model keeps the edit |
| E3 | `ControlError::TargetIndexOutOfRange` — an index-addressed publish out of range | inspector: same shape as E2. Indicates a defect in route construction, and the copy says so rather than implying user error. | none in-app; it is a bug | no |
| E4 | Render thread receives a target with **no route entry** | `params pending` becomes visible and increments. Audio is unaffected: the block renders with the previous value. | none in-app; the counter is the diagnostic | no (audio unchanged, not stale-and-hidden) |
| E5 | Route resolves but the processor refuses the key (`ParameterError::UnknownKey`) | `params pending` increments, same as E4. This is a route table that disagrees with a device's descriptor set — a construction defect. | none in-app; §5.1 test 8 exists to make it impossible to ship | no |
| E6 | A non-finite value somehow reaches a processor | Contained twice. The device setter maps non-finite to the descriptor default (`crates/spectre-core/src/param.rs:120–122`), so the value never enters the DSP state; and if a device were written that skipped that, RT-003 containment silences that node's whole output and counts it (`crates/spectre-graph/src/lib.rs:401–414`, `:512–521`), surfacing as R4-1's `contained <n>` counter. Never noise. | none needed; both layers are automatic | no |
| E7 | `AppModel::set_device_parameter` refuses the edit (unknown device or parameter key) | Existing behavior, unchanged: inspector shows "Could not update `<device>.<parameter>`: `<error>`. Reopen the device from Build and try again." (`crates/spectre-app/src/lib.rs:469–472`). **Nothing is published**, because publication is downstream of the model accepting the edit. | reopen the device from Build | no |
| E8 | `control_channel` refuses construction because the target set exceeds `MAX_PARAMETER_TARGETS` (§4.2) | Engine start fails with R4-1's `EngineUnavailable::Control(..)`; the transport bar reads `Engine unavailable — <Display>` and the rest of the app keeps working. | none in-app for R4; the cap is far above R4's four targets and hitting it is a defect | no |

Three rules bind every row. **No error string is formatted on the audio thread** — every
message above is built on the app thread from a value the render side published into an
atomic or from an app-thread call's `Err` (RT-001, and `ControlError`'s own doc comment at
`control.rs:28` says it is "never constructed on the render path"). **A failed live
delivery never rolls back a model edit**, because the project value and the live value are
not the same fact and pretending they are would lose user work. **No threshold-derived
alarm is specified** — there is no "parameter path unhealthy" badge computed from a pending
count, because any such threshold would be a numeric bound with nothing behind it
(decision 16, PROD-003).

### 3.7 Accessibility

- Every element R4-2 touches is an existing egui slider, an existing small button, or a
  text label. No icon-only control, no color-only state, no custom-painted widget is
  introduced. Engine and parameter state is carried by words and numbers — `applied`,
  `pending`, and the Shape header sentence — so a monochrome or color-blind reading loses
  nothing.
- Screen reader labels, hints, traits: `crates/spectre-app/Cargo.toml` builds eframe with
  `default-features = false` and only `default_fonts` and `glow`, so **no accessibility
  feature is enabled in this workspace today** and R4-2 claims no screen-reader support.
  Decision 17 gates that at beta with a scoped audit at R4. R4-2's obligation is not to
  foreclose it, which it satisfies by adding only standard widgets with text content, and
  by making the "did this reach live audio?" answer a sentence rather than an indicator
  color.
- Custom actions for complex interactions: N/A — no compound gesture is added. The slider
  keeps egui's built-in keyboard interaction.
- Text scaling / dynamic type: the Shape row's 520 px minimum width and 360 px slider are
  fixed (`main.rs:362`, `:370`); at large egui zoom the header sentence must wrap rather
  than clip the slider. Manual check in §5.4.
- Focus order and keyboard navigability: unchanged. R4-2 adds no interactive element to
  Shape, so the existing focus order — device frame, then each parameter's slider and
  `Reset` in creation order — is preserved exactly. The two transport-bar counters are
  labels and take no focus. Any focus-order defect inherited from R4-1's right-to-left
  cluster (`main.rs:78`) stays R4-1's to record for decision 17's R4 audit.

---

## 4. Implementation Specification

### 4.1 Architecture placement

| Path | Change | Thread |
|---|---|---|
| `crates/spectre-dsp/src/io.rs` | **modify** — add `ParameterError` and the required `AudioProcessor::set_parameter` method | render (the method is callback-reachable) |
| `crates/spectre-dsp/src/source.rs` | **modify** — implement `set_parameter` for `ToneSource` and `PulseInstrument` | render |
| `crates/spectre-dsp/src/effect.rs` | **modify** — implement `set_parameter` for `Gain` and `Saturator`, reusing the existing inherent setters' bodies | render |
| `crates/spectre-dsp/src/lib.rs` | **modify** — re-export `ParameterError` | build |
| `crates/spectre-graph/src/lib.rs` | **modify** — add `CompiledPlan::set_parameter` and two `PlanError` variants | render |
| `crates/spectre-audio/src/route.rs` | **new** — `ParameterRoute` and the sorted route table; render-side lookup is a binary search | render |
| `crates/spectre-audio/src/control.rs` | **modify** — add `MAX_PARAMETER_TARGETS` and `ControlError::TooManyTargets`, checked in `control_channel` | app thread (the check is in the constructor) |
| `crates/spectre-audio/src/bridge.rs` | **modify** — carry the route table, apply parameters instead of discarding them, add `parameters_applied` | render |
| `crates/spectre-audio/src/lib.rs` | **modify** — `pub mod route;` | build |
| `crates/spectre-app/src/lib.rs` | **modify** — add `ParameterEdit` and `AppModel::edit_device_parameter`; re-express `set_device_parameter` in terms of it | app thread |
| `crates/spectre-app/src/engine.rs` | **modify (R4-1's file)** — register the real target set, build the route table, add `LiveEngine::send_parameter` and `apply_parameter_edit` | app thread |
| `crates/spectre-app/src/main.rs` | **modify** — Shape edit loop calls `apply_parameter_edit`; Shape header line; two counters | app thread |

**The RT-001 structural argument, stated against the actual scan — and this slice cannot
make the easy version of it.**

`crates/spectre-audio/tests/rt_guard.rs` — a test file, not a `src` module — declares
`RT_MODULES` at lines 293–298, whose four entries at lines 294–297 are, read verbatim:
`src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, `src/null.rs`. **`midi.rs` is not
scanned.** The forbidden set is `FORBIDDEN` at lines 299–307, seven needles at 300–306:
`Mutex`, `RwLock`, `Condvar`, `thread::sleep`, `println!`, `eprintln!`, `dbg!`. The test
asserts by substring that no scanned module's text contains any of them
(`rt_guard.rs:310–319`).

**R4-2's own change list modifies two of those four modules** — `src/bridge.rs` and
`src/control.rs` — so no untouched-set argument is available. The argument is made on the
content of the edits, and it is re-proved by running the test, not by this paragraph:

- `src/spsc.rs` and `src/null.rs` are not modified at all, so their scan result is
  unchanged by construction.
- `src/control.rs` gains one `pub const`, one `ControlError` variant with a `Display` arm,
  and a length check at the top of `control_channel` — which is an app-thread constructor
  that already allocates (`control.rs:205–217`). None of that names any of the seven
  needles, and none of it is on a callback-reachable path.
- `src/bridge.rs` gains one `Box<[ParameterRoute]>` field, one constructor, one
  `AtomicU64` counter with an accessor, and a replacement for the closure body at
  `bridge.rs:181`. The replacement performs a binary search over a preallocated boxed slice
  and calls two methods; it allocates nothing, names none of the seven needles, and adds no
  new I/O.
- `src/route.rs` is **new and callback-reachable**, so R4-2 **adds it to `RT_MODULES`**,
  changing the array to `[&str; 5]`. Extending the scan is part of this slice's change
  list (§7.2), not an optional extra: a callback-reachable module outside the scan is a
  hole in the RT-001 evidence.
- Because none of this is an untouched-module argument, §5.2 makes
  `cargo test -p spectre-audio --test rt_guard` a **required post-edit gate**, and that
  pass is the evidence.

**Why the new work is allocation-free, path by path.** The render-side sequence added by
this slice is: one `slice::binary_search_by` over `Box<[ParameterRoute]>` (no allocation,
bounded by `log2` of the route count, returns `Result` rather than panicking); one linear
scan of `CompiledPlan.steps` for the node, the same scan `process` already performs for
note inputs at `crates/spectre-graph/src/lib.rs:466–470`; one dynamic dispatch to
`set_parameter`; and inside the device, one or two `&'static str` comparisons plus
`DspParameter::clamp`. `clamp` is `self.spec.clamp(f64::from(value)).plain() as f32`
(`crates/spectre-dsp/src/parameter.rs:49–51`), and `ParamSpec::clamp`
(`crates/spectre-core/src/param.rs:119–124`) returns the descriptor default for a
non-finite input and otherwise calls `f64::clamp`. `f64::clamp` panics only when
`min > max`; `ParamSpec::new` rejects `max <= min` (`param.rs:77–79`) and is a `const fn`,
and the four descriptor tables are `const` (`crates/spectre-dsp/src/source.rs:27`, `:39`;
`crates/spectre-dsp/src/effect.rs:19`, `:22`), so an ill-ordered spec is a **compile-time**
failure, not a callback-time panic. That is the complete no-panic argument for the setter.

`AppModel` stays renderer-neutral and audio-free: the type it gains (`ParameterEdit`)
carries two `spectre_core::ObjectId` values and an `f32` and names nothing from
`spectre-audio`. R4-1's invariant is preserved, not weakened.

### 4.2 Data model

**New — `crates/spectre-dsp/src/io.rs`.**

```rust
// Callback-safe parameter-application failure. Carries the refused key so the app thread can
// build a diagnostic; constructing it copies a &'static str pointer and never allocates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterError {
    // The device has no parameter under this key. Recoverable, never a panic.
    UnknownKey(DeviceParameterKey),
}
```

`io.rs` gains `use crate::parameter::DeviceParameterKey;`. There is no module cycle:
`parameter.rs` imports only from `spectre_core` (`crates/spectre-dsp/src/parameter.rs:6`).

**New — `crates/spectre-audio/src/route.rs`.**

```rust
// Author: Jeff
// Date: 2026-08-15
// Description: Render-side map from an RT-002 parameter target to a live plan node and key
// Notes: Built once on the app thread and never mutated. Lookup is a binary search over a
//   preallocated boxed slice: no allocation, no locks, no panic, bounded by log2 of the entry
//   count. This module is callback-reachable and is scanned by rt_guard's RT_MODULES.

// One resolved route from lane identity to plan addressing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParameterRoute {
    pub target: crate::control::ParameterTarget,
    pub node: spectre_graph::NodeId,
    pub key: spectre_dsp::DeviceParameterKey,
}

// Immutable sorted routing table; construction is app-thread and may allocate
#[derive(Debug, Default)]
pub struct ParameterRoutes {
    // Sorted by `target`, which derives Ord (control.rs:22), so lookup is a binary search
    entries: Box<[ParameterRoute]>,
}

impl ParameterRoutes {
    // Sort and freeze a route set; duplicate targets are refused rather than shadowed
    pub fn new(mut entries: Vec<ParameterRoute>) -> Result<Self, RouteError>;

    // An empty table: every incoming target is unrouted and counted
    pub fn empty() -> Self;

    // Resolve one target; callback-safe, allocation-free, bounded
    pub fn resolve(
        &self,
        target: crate::control::ParameterTarget,
    ) -> Option<(spectre_graph::NodeId, spectre_dsp::DeviceParameterKey)>;

    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}

// App-thread route-construction failure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteError {
    DuplicateTarget(crate::control::ParameterTarget),
}
```

`ParameterTarget` already derives `PartialOrd, Ord` (`crates/spectre-audio/src/control.rs:22`),
so sorting and binary search need no new ordering definition. `ParameterRoutes::empty()`
builds `Vec::new().into_boxed_slice()`, which is a dangling-pointer boxed slice and
performs no allocation; it is constructed on the app thread regardless.

**New — `crates/spectre-audio/src/control.rs`, one bound.**

```rust
// Largest parameter-target set a control channel will register.
//
// Rationale (decision 16, PROD-003 — this bound is Spectre's own and is not taken from any
// reference product). It exists because `ParameterReader::drain` walks *every* registered slot
// on *every* block (control.rs:127), so without a cap the per-block cost of the callback path
// grows without limit as a project grows, which RT-001's "bounded" clause does not permit.
// The value is deliberately generous rather than tuned, because Spectre has no measurement that
// would justify a tuned value and a generous bound still discharges the obligation that the
// number be bounded at all. What *is* computable from source is the memory: one target costs a
// 16-byte ParameterTarget (two ObjectId(u64), control.rs:22-26), an 8-byte ParameterSlot (two
// AtomicU32, control.rs:67-70), a 4-byte `seen` entry (control.rs:87), and a 40-byte
// ParameterRoute, so 1_024 targets reserve 69,632 B — negligible against the plan's own channel
// pool. R4's complete accepted device scope registers four. Re-open when a real project
// approaches the cap, or when a hardware run produces the first drain-cost measurement.
pub const MAX_PARAMETER_TARGETS: usize = 1_024;
```

and one `ControlError` variant, with a `Display` arm matching the existing style
(`control.rs:39–62`):

```rust
    // Registered target set exceeds the callback path's bounded-work budget
    TooManyTargets(usize),
```

**New — `crates/spectre-app/src/lib.rs`.**

```rust
// The canonical result of one accepted Shape edit: what the model stored, and which stable
// identities name it. Public fields, matching ParameterControl (lib.rs:49-53), because this is
// an app-thread value rather than a published cross-crate DTO like DeviceParameterSnapshot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParameterEdit {
    pub device_instance_id: spectre_core::ObjectId,
    pub parameter_instance_id: spectre_core::ObjectId,
    // Already clamped by the parameter's own descriptor; this is what offline rendering would use
    pub value: f32,
}
```

**Modified — `crates/spectre-audio/src/bridge.rs`.** `RenderBridge`
(`bridge.rs:119–129`) gains one field:

```rust
    // Built on the app thread before the stream opens; immutable for the bridge's life, so no
    // retired route table is ever handed to the reclaim lane
    routes: crate::route::ParameterRoutes,
```

`BridgeTelemetry` (`bridge.rs:24–39`) gains one counter, `parameters_applied: AtomicU64`,
with an accessor beside the existing ones. `parameters_pending`'s **field and accessor are
kept** — the acceptance criterion is stated in terms of it — but its meaning narrows from
"observed but not yet applicable to a live plan" (`bridge.rs:82`) to "observed but not
delivered to a processor", and its doc comment changes to say so. In a correctly wired
build it stays zero for the life of the process.

**Modified — `crates/spectre-graph/src/lib.rs`.** `PlanError` (`lib.rs:352–362`) gains two
variants with `Display` arms in the existing style (`lib.rs:364–383`):

```rust
    // A parameter application named a node that is not in the plan
    UnknownParameterNode(NodeId),
    // The node's processor refused the key
    Parameter { node: NodeId, error: spectre_dsp::ParameterError },
```

**Database migrations: N/A — R4-2 persists nothing.** The project envelope
(`spectre-project`) is untouched; device parameter persistence is R4-7's, and this slice
adds no field that would need one.

### 4.3 API contracts

**New — `crates/spectre-dsp/src/io.rs`, one trait method.**

```rust
pub trait AudioProcessor: Send {
    fn io(&self) -> DeviceIo;

    // Apply one already-validated parameter value to a live processor.
    //
    // Inherits the full realtime contract of `process`: no allocation, no locks, no I/O, no
    // string formatting, no logging, no panic. Implementations perform assignment and at most
    // bounded arithmetic. Values arriving here are already canonical — the app thread clamped
    // them against this parameter's own descriptor before publication — so an implementation
    // applies, it does not police. Unknown keys are refused, never panicked on.
    //
    // Required rather than defaulted, deliberately: a device added later must answer the seam
    // instead of silently ignoring every edit made to it.
    fn set_parameter(
        &mut self,
        key: DeviceParameterKey,
        value: f32,
    ) -> Result<(), ParameterError>;

    fn process(
        &mut self,
        context: &ProcessContext<'_>,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
    ) -> Result<(), ProcessError>;
}
```

This is the accepted shape, point for point, from `docs/03-architecture/dsp-device-io.md`
§"Runtime parameter seam": one callback-safe method, addressed by the same static
`DeviceParameterKey` the descriptors and the offline snapshot DTO already use
(`crates/spectre-dsp/src/parameter.rs:13`, `:115–117`); full realtime contract inherited;
validation and clamping on the app thread; unknown keys refused as a recoverable error;
application a separate step from `process`, executed once per block before it; smoothing
left to the device.

**Four device implementations.** Each compares the incoming key against its own `const`
descriptor table so the setter cannot drift from the descriptors, and each reuses the
clamp the constructor already applies. `Gain` and `Saturator` reuse the bodies of their
existing inherent setters (`crates/spectre-dsp/src/effect.rs:45–47`, `:98–104`), which
today have **no callers anywhere in the workspace** (verified: the only occurrences of
`set_gain`, `set_drive`, and `set_mix` in `crates/` are their own declarations). This slice
is what finally reaches them.

```rust
// Gain (crates/spectre-dsp/src/effect.rs)
fn set_parameter(&mut self, key: DeviceParameterKey, value: f32) -> Result<(), ParameterError> {
    if key == GAIN_PARAMETERS[0].key {
        self.set_gain(value);   // clamps via GAIN_PARAMETERS[0].clamp — effect.rs:46
        return Ok(());
    }
    Err(ParameterError::UnknownKey(key))
}

// Saturator: drive → set_drive, mix → set_mix, else UnknownKey
// PulseInstrument: "level" → self.level = PULSE_PARAMETERS[0].clamp(value), else UnknownKey
// ToneSource: "frequency_hz" → TONE_PARAMETERS[0].clamp, "level" → TONE_PARAMETERS[1].clamp
```

`PulseInstrument` and `ToneSource` have no inherent setters today
(`crates/spectre-dsp/src/source.rs:121–141`, `:56–74`), so theirs are written here. Note
that changing `ToneSource::frequency` mid-stream does **not** reset `phase`
(`source.rs:52`, `:96`), so a frequency change is phase-continuous — correct behavior, and
worth stating because the alternative would click.

**New — `crates/spectre-graph/src/lib.rs`, one plan method.**

```rust
impl CompiledPlan {
    // Apply one parameter value to one live node's processor. Callback-safe: the node lookup is
    // the same bounded linear scan `process` already performs over `steps` (lib.rs:466-470), and
    // the processor's own setter carries the realtime contract.
    pub fn set_parameter(
        &mut self,
        node: NodeId,
        key: spectre_dsp::DeviceParameterKey,
        value: f32,
    ) -> Result<(), PlanError>;
}
```

Implementation shape: `self.steps.iter().position(|step| step.node == node)` →
`PlanError::UnknownParameterNode(node)` on `None`; otherwise
`self.processors[index].set_parameter(key, value)` mapped to
`PlanError::Parameter { node, error }`. `steps` and `processors` are built in the same
order in `compile` (`crates/spectre-graph/src/lib.rs:283–318`, pushed together at `:310`
and `:317`), so the index is valid for both — the same invariant `process` relies on when
it zips them at `:482`. **GRAPH-001 is unaffected**: this mutates a processor's internal
value, it does not add node or edge mutation, and `CompiledPlan` still exposes no graph
editing API. Nothing is recompiled; decision 22's option (b) is not proposed here in any
form.

**New — `crates/spectre-audio/src/bridge.rs`, one constructor.**

```rust
impl RenderBridge {
    // Build a bridge with a live parameter route table
    pub fn with_parameter_routes(
        plan: CompiledPlan,
        control: ControlReceiver,
        note_node: NodeId,
        sample_rate: f64,
        note_scratch: usize,
        routes: crate::route::ParameterRoutes,
    ) -> Self;
}
```

`RenderBridge::new` (`bridge.rs:133–149`) is **kept and delegates** with
`ParameterRoutes::empty()`. That is a deliberate choice for a small diff: `new`'s existing
five-argument form is called by `crates/spectre-audio/tests/bridge_plan.rs:33`,
`crates/spectre-audio/tests/rt_guard.rs:165`, `:193`, and `:264`, and none of those tests
needs a route table. They compile unchanged, which keeps the R3 evidence intact rather than
rewriting it to accommodate this slice.

**The render-side parameter step**, replacing `bridge.rs:178–186`:

```rust
        // Parameter application runs once per block, before `process`, so the whole block sees
        // one coherent parameter set (dsp-device-io.md, "Runtime parameter seam").
        // Borrows are split by field before the call, so the closure captures `plan`, `routes`,
        // and the counters rather than `self`, and `drain_parameters` gets `&mut self.control`.
        let plan = &mut self.plan;
        let routes = &self.routes;
        let mut applied = 0_u64;
        let mut unapplied = 0_u64;
        self.control.drain_parameters(|target, value| {
            match routes.resolve(target) {
                Some((node, key)) if plan.set_parameter(node, key, value).is_ok() => applied += 1,
                _ => unapplied += 1,
            }
        });
        if applied > 0 {
            self.telemetry.parameters_applied.fetch_add(applied, Ordering::Relaxed);
        }
        if unapplied > 0 {
            self.telemetry.parameters_pending.fetch_add(unapplied, Ordering::Relaxed);
        }
```

The counters accumulate as plain locals inside the closure and are published once per block
as two relaxed adds, which is the same pattern `CompiledPlan` already uses for containment
stats — "plain integers so the render path stays free of atomics and the owner publishes
them off-thread at its own cadence" (`crates/spectre-graph/src/lib.rs:387–388`).

**New — `crates/spectre-app/src/lib.rs`, one model method.**

```rust
impl AppModel {
    // Apply a Shape edit and report the canonical stored value with its stable identities.
    // This is the single clamping site; `set_device_parameter` becomes
    // `self.edit_device_parameter(..).map(|_| ())`, so no existing caller or test changes and
    // there is exactly one place a value can be clamped.
    pub fn edit_device_parameter(
        &mut self,
        device_key: &str,
        parameter_key: &str,
        value: f32,
    ) -> Result<ParameterEdit, &'static str>;
}
```

**New — `crates/spectre-app/src/engine.rs`** (R4-1 creates this file; R4-2 extends it).

```rust
// Build the lane's target set and the render-side route table from the same validated snapshot
// the plan is built from. No new app-side identity plumbing is needed: DeviceParameterSnapshot
// already carries device_instance_id, device_key, parameter_instance_id, and parameter_key
// (crates/spectre-dsp/src/parameter.rs:81-126), which is exactly the join.
pub fn build_parameter_wiring(
    snapshot: &[spectre_dsp::DeviceParameterSnapshot],
    nodes: &FixtureNodes,
) -> Result<(Vec<spectre_audio::control::ParameterTarget>,
             spectre_audio::route::ParameterRoutes), EngineUnavailable>;

// The three plan nodes build_engine_parts allocates, exposed so routes can be built against them
pub struct FixtureNodes {
    pub pulse: spectre_graph::NodeId,
    pub gain: spectre_graph::NodeId,
    pub saturator: spectre_graph::NodeId,
}

impl<S: spectre_audio::AudioStream + ?Sized> LiveEngine<S> {
    // Publish one already-clamped value to its latest-wins slot. Takes &self, not &mut self:
    // ParameterWriter::set is an atomic store and needs no exclusivity (control.rs:92).
    pub fn send_parameter(
        &self,
        target: spectre_audio::control::ParameterTarget,
        value: f32,
    ) -> Result<(), spectre_audio::control::ControlError>;
}

// Apply a Shape edit to the model and publish the accepted value to the live lane.
// Extracted as a free function rather than left inline in main.rs so it can be tested against a
// real AppModel — main.rs is a binary target and its types are unreachable from
// crates/spectre-app/tests/. This is the same lesson R4-1's iteration-2 review recorded about
// its own binding rule.
pub fn apply_parameter_edit(
    model: &mut AppModel,
    engine: Option<&LiveEngine>,
    device_key: &str,
    parameter_key: &str,
    value: f32,
    feedback_status: &mut String,
);
```

`build_engine_parts` (R4-1 §4.3) changes in exactly one way: it calls `control_channel`
with the snapshot-derived target set instead of an empty one, and it stores the route table
into the bridge via `with_parameter_routes`. `EngineParts` gains
`pub targets: Box<[ParameterTarget]>` so the app can address slots by identity without
rebuilding them.

**Failure behavior of the whole path, in one table.**

| Stage | Failure | Result | Counted where |
|---|---|---|---|
| model | unknown device/parameter key | `Err(&'static str)`, nothing published | inspector status line (E7) |
| publish | target not registered | `Err(ControlError::UnknownTarget)` | inspector status line (E2) |
| publish | index out of range | `Err(ControlError::TargetIndexOutOfRange)` | inspector status line (E3) |
| construct | target set over the cap | `Err(ControlError::TooManyTargets)` at engine start | transport bar (E8) |
| route | no entry for the target | value dropped, block renders with the previous value | `parameters_pending` (E4) |
| apply | node absent from the plan | `PlanError::UnknownParameterNode` | `parameters_pending` (E4) |
| apply | processor refuses the key | `PlanError::Parameter` / `ParameterError::UnknownKey` | `parameters_pending` (E5) |
| value | non-finite reaches a device | mapped to the descriptor default at the setter; if a device skipped that, RT-003 silences the node | `contained` (E6) |

Every one of them is fail-closed: the previous value keeps rendering, nothing goes stale
and hidden, nothing panics, and nothing is logged on the audio thread.

**Why smoothing is not in this slice.** The accepted contract says "Smoothing stays the
device's concern. `Gain` already smooths." The shipped `Gain` does not: it is a single
`gain: f32` (`crates/spectre-dsp/src/effect.rs:29–31`) multiplied directly per sample
(`:62–71`), with no target, no coefficient, and no per-sample ramp. Adding smoothing here
would change rendered output for the same inputs, which would break the bit-exact
live/offline hash equality that `crates/spectre-audio/tests/bridge_plan.rs` asserts against
`spectre_offline::render_vertical_slice`, and would require its own numeric bound (a
smoothing time constant) with a rationale row. That is a real device decision with real
evidence consequences, it belongs to R4-6 under decision 15's deliberately-small scope, and
it is not this slice's to make by assertion. §7.1 records the discrepancy, §3.2 and §5.4
make the resulting click visible, and §8 Q1 routes it to Jeff with D-R3 logged for
`gauntlet-output/decisions-needed.md`.

**Auth, permissions, pagination, rate limiting: N/A** — this is an in-process desktop
feature with no network and no multi-user surface.

### 4.4 State management

| State | Owner | Thread | Lifetime |
|---|---|---|---|
| Parameter value of record | `AppModel.devices[..].parameters[..].value` (`crates/spectre-app/src/lib.rs:49–63`) | app | process |
| Clamping authority | `DspParameter` / `ParamSpec` descriptors, `const` per device type | both (same code, both sides) | static |
| In-flight value | one `AtomicU32` latest-wins slot + version per target (`control.rs:66–70`) | published app, consumed render | until drained |
| Applied value | the processor's own field inside `CompiledPlan` | render | until the stream closes |
| Route table | `ParameterRoutes` inside `RenderBridge` | built app, read render | until the stream closes |
| Counters | `BridgeTelemetry` atomics | written render, read app | until the stream closes |

**There is exactly one value of record, and it is the model's.** The render side holds a
*copy* that the model published; it never originates a value and never sends one back. That
asymmetry is what keeps the two lenses (Shape's slider and the sounding chain) views over
one model rather than two stores that must be reconciled — `vision.md`'s "One project,
linked lenses" applied to a value rather than to a view. It is also what makes R4-8's
bounce equivalence tractable: an offline render built from
`AppModel::device_parameter_snapshot` (`crates/spectre-app/src/lib.rs:348–383`) and a live
render that applied published values from that same model are, by construction, the same
numbers.

**The binding rule for publication:**

> Publish only what the model accepted, and publish the model's stored value — never the
> widget's raw value.
>
> - `Ok(edit)` — the model clamped and stored it; publish `edit.value` to
>   `(edit.device_instance_id, edit.parameter_instance_id)`.
> - `Err(_)` from the model — publish nothing and surface E7. A value the project rejected
>   must never reach the audio thread.
> - `Err(_)` from the publish — the model edit **stands**; surface E2/E3. The project value
>   and the live value are different facts, and rolling back a valid project edit because a
>   lane refused it would lose user work to a transport failure.
> - No engine — publish nothing, no error, Shape header explains.

**Reclamation.** R4-2 adds **no traffic to the reclaim lane**. The route table is built once
before the stream opens and is immutable for the bridge's life; there is no old table to
retire. Applying a parameter overwrites an `f32` inside a processor and frees nothing.
`ControlSender::reclaim` (`control.rs:274–281`) stays exactly as R4-1 drives it, once per
UI frame. This is worth stating precisely because RT-002's off-thread-reclamation clause is
the part a parameter feature would most plausibly violate — and the reason it does not
violate it is that option (a) mutates in place while option (b), which decision 22 rejects,
would have retired a whole plan per knob turn.

**Local vs. server-synced state: N/A** — Spectre has no server; cloud services are a
`vision.md` non-goal.

**Offline / draft persistence: N/A for this slice.** Parameter values already live in
`AppModel` and are already published to offline rendering as an owned snapshot. Writing them
to disk is CORE-004's implementation at R4-7, and R4-2 adds no field that changes what R4-7
must persist.

### 4.5 Dependencies

- **New crate dependencies: none.** Every crate involved is already a workspace member and
  the edges already exist: `spectre-graph` already depends on `spectre-dsp`
  (`crates/spectre-graph/src/lib.rs:9`), `spectre-audio` already depends on both
  (`crates/spectre-audio/src/bridge.rs:13–14`), and R4-1 adds `spectre-audio` and
  `spectre-graph` to `spectre-app`. R4-2 adds no edge that R4-1 has not already added.
- **New assets or resources:** none. No fonts, images, samples, wavetables, or presets.
- **Infrastructure:** none. No CI change; the workspace gate is unchanged.
- **New feature flags:** none. R4-2 rides R4-1's `live-audio` feature; with it disabled the
  app has no engine, `apply_parameter_edit` publishes nothing, and Shape's header says so.

### 4.6 Platform-specific considerations

- **Decision 1 makes macOS and Linux co-first-class, and R4-2 is platform-neutral by
  construction.** Nothing in this slice names an OS, a driver, a host API, or a
  platform-conditional path. The setter is arithmetic and assignment; the route table is a
  sorted slice; the lane is the existing portable atomics. There is no `#[cfg]` anywhere in
  the change list.
- **The Linux risk this slice carries is inherited, not created.** Decision 23 leaves Linux
  device qualification undischarged and R4-3 owns discharging it. R4-2 **authorizes no
  Linux claim**: no Linux audio device has ever been opened in this project, so "a Shape
  edit changes live audio on Linux" is unproven until R4-3 runs and someone repeats §5.4's
  manual protocol there. What R4-2 *can* say is that its deterministic evidence — every
  test in §5.1 — runs against the null backend and therefore passes identically on both
  platforms in CI, which is a portability result and not a device result. §5.4's manual
  protocol is explicitly listed as owed on **both** platforms.
- **Denormal behavior is identical on both targets**, because RT-003's flush is the
  software FTZ-equivalent in `contain_channel` (`crates/spectre-graph/src/lib.rs:401–414`)
  rather than a CPU control-register manipulation — chosen for exactly this portability
  reason at R3 slice 6. A parameter driven toward zero (gain → 0.0, mix → 0.0) produces
  denormal-range products; they flush the same way on x86-64 and aarch64, on macOS and
  Linux. §5.1 test 11 pins it.
- **Version compatibility:** edition 2021 workspace-wide (`Cargo.toml:19`). The borrow
  split in §4.3's render step binds `&mut self.plan` and `&self.routes` to locals *before*
  the `self.control.drain_parameters(..)` call, so it relies on ordinary disjoint field
  borrowing rather than on edition-2021 closure capture rules. No toolchain feature is
  required.
- **Feature flags / gradual rollout:** none of its own. The seam is either implemented or
  it is not; a half-wired parameter path that silently drops edits is precisely the state
  this slice exists to end.

### 4.7 Performance budget

All figures are computed from source, not estimated from another product.

- **Memory (steady state, added).**
  - Route table: `ParameterRoute` is a 16-byte `ParameterTarget` + an 8-byte `NodeId`
    (`spectre-graph/src/lib.rs:20`, wrapping `ObjectId(u64)`) + a 16-byte
    `DeviceParameterKey` (a `&'static str`, `spectre-dsp/src/parameter.rs:13`) = 40 B.
    R4's four targets cost **160 B**.
  - Lane slots for four targets: 4 × (16 B target + 8 B slot + 4 B `seen`) = **112 B**.
    These replace the empty target set R4-1 registers, so the delta is the whole cost.
  - Two `AtomicU64` counters on `BridgeTelemetry`: **16 B**.
  - Total added resident memory for R4 as scoped: **under 300 B.** At the
    `MAX_PARAMETER_TARGETS` cap it would be 69,632 B (§4.2's rationale), which is the number
    the cap is chosen against.
  - **No per-block and no per-edit allocation is added on either thread.** `ParameterEdit`
    is `Copy` and built on the stack; the closure in §4.3 captures references only.
- **CPU / render time.** Added per block: the existing `drain_parameters` walk over
  registered slots, which is 4 relaxed loads today instead of 0; and, for each *changed*
  target only, one binary search over ≤ 4 entries, one linear scan over 3 plan steps, one
  virtual call, and ≤ 2 short `&str` comparisons plus one `f64` clamp. In the common
  steady state — no knob moving — the added cost is exactly four relaxed atomic loads that
  compare equal and return. **No budget threshold is asserted**: Spectre owns one hardware
  measurement (macOS, 2026-08-09, worst-case headroom 0.990 for the three-node chain at
  256 frames / 48 kHz) and one data point cannot justify a threshold (decision 16,
  PROD-003). The hardware re-run in §5.2 is what produces the first evidence that this
  slice did not move the number.
- **UI cost:** two additional relaxed atomic loads per frame, folded into R4-1's existing
  once-per-frame `EngineHealth` read at the 250 ms repaint cadence (`main.rs:443`).
  Negligible and lock-free.
- **Network payload:** N/A — no network I/O exists anywhere in this feature.
- **Storage (client + server):** N/A — nothing is written to disk.
- **Startup time:** one additional app-thread pass over the four-entry snapshot to build
  targets and routes, plus one sort of four elements, inside R4-1's existing engine-start
  path. Unmeasurable against the device open it sits beside.

---

## 5. Test Specification

Every command below is real and names a real target. All of them run today except
`cargo test -p spectre-app --test live_engine`, whose file **R4-1** creates and which R4-2
extends; it runs from the moment R4-1's file lands. The workspace gate is exactly:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

Feature-scoped commands:

```sh
cargo test -p spectre-dsp --test devices
cargo test -p spectre-graph --test graph_plan
cargo test -p spectre-audio --test control_channel
cargo test -p spectre-audio --test bridge_plan
cargo test -p spectre-audio --test rt_guard
cargo test -p spectre-app --test app_model
cargo test -p spectre-app --test live_engine
# hardware, run explicitly; macOS re-run confirms no regression, Linux is R4-3's debt
cargo test -p spectre-audio --test lifecycle_health -- --ignored --nocapture
```

### 5.1 Unit tests

**Which tests hold which handles.** Tests 1–3 and 8 are pure and open no stream. Tests 4–7
and 9–11 build the fixture plan and a `RenderBridge` directly and call `render` against a
`RenderBlock`, exactly as `crates/spectre-audio/tests/bridge_plan.rs` does at `:96–101` —
no device and no `LiveEngine` needed. Tests 12–14 hold a `LiveEngine<NullStream>` built by
R4-1's `from_open_stream` over a concrete `NullStream` from
`NullBackend::open_null_output` (`crates/spectre-audio/src/null.rs:38`) and pump through
`stream_mut()`, which is why R4-1 made `LiveEngine` generic over its stream. Test 15 needs
only an `AppModel`.

**In `crates/spectre-dsp/tests/devices.rs`:**

1. **`each_device_applies_its_own_keys_and_refuses_every_other`**
   Setup: construct `ToneSource::new(440.0, 0.25)`, `PulseInstrument::new(Waveform::Saw, 0.2)`,
   `Gain::new(1.0)`, `Saturator::new(1.0, 1.0)`. For each device, call `set_parameter` with
   every key in its own descriptor table and assert `Ok(())`; then call it with a key from a
   *different* device's table and assert
   `Err(ParameterError::UnknownKey(k))` carrying that key. Edge case: a device silently
   accepting a foreign key, which would let a route-table mistake apply the wrong value to
   the wrong parameter and never be noticed.

2. **`a_set_parameter_value_out_of_range_is_clamped_not_refused`**
   Assert `Gain::set_parameter("gain", 99.0)` returns `Ok` and that the next rendered block
   equals the block rendered at the descriptor maximum (`GAIN_PARAMETERS[0].maximum()`,
   `2.0`). Repeat for a value below the minimum, and for `f32::NAN`, which must land on the
   descriptor **default** (`crates/spectre-core/src/param.rs:120–122`), not on the minimum.
   Edge case: the containment contract in `dsp-device-io.md` §"Signed zero and subnormals"
   — "Non-finite candidates map to the descriptor default" — proved at the new seam rather
   than assumed from the constructor path.

3. **`applying_a_parameter_does_not_disturb_running_dsp_state`**
   Render 64 frames from `ToneSource` at 440 Hz, call
   `set_parameter("frequency_hz", 880.0)`, render 64 more, and assert the output has no
   discontinuity at the seam beyond what the frequency change itself implies — concretely,
   that `phase` was not reset, by asserting the first sample of the second block is not
   `0.0` when the last sample of the first block was not near a zero crossing. Edge case: a
   setter that reconstructs the device instead of assigning to it, which would reset phase
   and click on every edit.

**In `crates/spectre-graph/tests/graph_plan.rs`:**

4. **`the_plan_routes_a_parameter_to_the_named_node_only`**
   Setup: compile the Pulse → Gain → Saturator fixture. Call
   `plan.set_parameter(gain_node, GAIN_PARAMETERS[0].key, 0.0)` and render; assert the
   output is exactly zero in every sample. Then call it with `1.0` and render; assert a
   nonzero peak. Then call `plan.set_parameter(pulse_node, GAIN_PARAMETERS[0].key, 0.5)`
   and assert `Err(PlanError::Parameter { .. })` — Pulse has no `gain` key — and that
   rendering afterwards is unchanged. Edge case: cross-node leakage, and the refusal path
   leaving the plan intact.

5. **`an_unknown_node_is_refused_without_touching_any_processor`**
   Call `set_parameter` with a `NodeId` never added to the graph; assert
   `Err(PlanError::UnknownParameterNode(..))` and that a subsequent render hashes
   identically to a render taken before the call. Edge case: a stale route surviving a plan
   rebuild.

**In `crates/spectre-audio/tests/bridge_plan.rs`:**

6. **`a_published_parameter_changes_the_next_block_and_only_the_next_block`** — the core
   test. Setup: build the fixture plan and a `ControlSender`/`ControlReceiver` over the
   four fixture targets; build the bridge with `with_parameter_routes`. Send
   `fixture_events(FRAMES)` and render one block; record the peak. Publish
   `gain = 0.25` through `sender.parameters().set_target(..)`; render again; assert the
   second block's peak is strictly lower, that
   `bridge.telemetry().parameters_applied() == 1`, and that
   **`bridge.telemetry().parameters_pending() == 0`**. Render a third block with no further
   publication and assert `parameters_applied()` is still 1 — a latest-wins slot must not
   re-apply an unchanged value, which `ParameterReader::drain`'s version check
   (`control.rs:129–131`) is what guarantees. Edge case: this is the assertion that fails
   today, fails if the closure at `bridge.rs:181` is left discarding, fails if routing is
   wrong, and fails if the value is applied every block instead of on change.

7. **`a_sweep_applies_once_with_the_last_value`**
   Publish 5,000 gain values ending at `0.10` with no render between them; render one
   block; assert `parameters_applied() == 1` and that the block matches a reference block
   rendered with `Gain::new(0.10)`. Edge case: decision 21's latest-wins policy holding
   end-to-end rather than only inside `control.rs`, where
   `control_channel.rs:45` already proves it in isolation.

8. **`the_route_table_covers_exactly_the_registered_targets`**
   Build targets and routes from `AppModel::prototype().device_parameter_snapshot()?`.
   Assert `routes.len() == targets.len() == 4`, that every target resolves, and that a
   `ParameterTarget` built from two fresh `IdGen` values resolves to `None`. Then assert
   `ParameterRoutes::new` returns `Err(RouteError::DuplicateTarget(..))` for a table
   containing the same target twice. Edge case: a route table that drifts out of step with
   the lane's target set, which is the defect E5 exists to report and this test exists to
   prevent shipping.

9. **`an_unrouted_target_is_counted_and_the_block_still_renders`**
   Build the bridge with `ParameterRoutes::empty()` while the lane has registered targets.
   Publish a value, render, and assert the block is bit-identical to the block rendered
   before the publication, that `parameters_pending() == 1`, and that
   `parameters_applied() == 0`. Edge case: the fail-closed rule — an undeliverable edit must
   leave audio unchanged and be counted, never leave audio stale-and-silently-wrong.

10. **`applying_parameters_never_allocates`** (RT-001 guard). This test belongs in
    `crates/spectre-audio/tests/rt_guard.rs`, not here, and is listed as test 13 below.

11. **`driving_gain_to_zero_flushes_denormals_rather_than_producing_noise`**
    Publish `gain = 0.0` while a note is sounding, render, and assert every output sample is
    exactly `0.0` (signed zero permitted) and that
    `bridge.telemetry().contaminated_nodes() == 0` while `denormals_flushed()` is a finite
    count. Then publish `saturator.mix = 0.0` and assert the chain still produces finite
    output. Edge case: RT-003 at the new seam — a parameter path is the most direct way to
    drive a chain into the denormal range at runtime, and this asserts the existing
    containment covers it rather than assuming it does.

**In `crates/spectre-audio/tests/control_channel.rs`:**

12. **`a_target_set_over_the_cap_is_refused_at_construction`**
    Call `control_channel` with `MAX_PARAMETER_TARGETS + 1` distinct targets and assert
    `Err(ControlError::TooManyTargets(n))` with the offending count. Assert the cap value
    itself constructs successfully. Edge case: the bound existing in the constant but not in
    the check, which would leave the callback path's work unbounded while looking bounded.

**In `crates/spectre-audio/tests/rt_guard.rs`:**

13. **`parameter_application_is_rt_clean`**
    Extend the existing guarded pattern. `rt_guard.rs:157–187`'s `bridge_render_is_rt_clean`
    **already publishes a parameter every block** (`:175–177`) — against a bridge whose
    route table is empty, so today it proves only that discarding is clean. Add a sibling
    test that builds the bridge with a real route table over the fixture's targets, warms up
    outside the guard, then for eight blocks publishes a new value for every target and
    wraps `bridge.render(..)` in `rt_section`, asserting zero violations. The existing
    positive control (`rt_guard.rs:138–154`) makes the zero meaningful. Edge case: this is
    the test that fails if the route table is rebuilt per block, if the setter formats a
    string, or if a device's implementation allocates — the three most likely ways this
    slice could break RT-001.

14. **`rt_modules_contain_no_blocking_primitives`** (existing test, `rt_guard.rs:290–320`)
    **must be updated**, not merely re-run: `RT_MODULES` becomes `[&str; 5]` with
    `"src/route.rs"` added. Edge case: a callback-reachable module sitting outside the scan.

**In `crates/spectre-app/tests/app_model.rs`:**

15. **`an_accepted_edit_reports_the_clamped_value_and_its_identities`**
    Call `model.edit_device_parameter("gain", "gain", 99.0)` and assert the returned
    `ParameterEdit.value` equals `GAIN_PARAMETERS[0].maximum()`, that its two `ObjectId`s
    equal the ones `device_parameter_snapshot` reports for that parameter
    (`crates/spectre-app/src/lib.rs:348–383`), and that
    `model.set_device_parameter("gain", "gain", 99.0)` still returns `Ok(())` unchanged.
    Then assert `edit_device_parameter("gain", "nope", 0.5)` returns `Err("unknown parameter")`
    and that the stored value did not move. Edge case: the publish path taking the raw widget
    value instead of the clamped stored one, which would let live audio and offline rendering
    hold different numbers for the same parameter.

**In `crates/spectre-app/tests/live_engine.rs`** (R4-1's file):

16. **`a_shape_edit_reaches_live_audio_through_the_real_app_path`** — the end-to-end
    assertion, and the one that encodes the acceptance criterion.
    Setup: `AppModel::prototype()`, R4-1's parts built from its snapshot, a concrete
    `NullStream` started, wrapped as `LiveEngine<NullStream>`. Pump once with a held note and
    record the block's peak from `NullStream::last_block()`
    (`crates/spectre-audio/src/null.rs:118`). Call
    `apply_parameter_edit(&mut model, Some(&engine), "gain", "gain", 0.25, &mut status)`.
    Pump again. Assert: the second block's peak is strictly lower; `model`'s stored gain is
    `0.25`; `engine.health().parameters_applied == 1`; and
    **`engine.health().parameters_pending == 0`**. Edge case: every seam in the chain at
    once — model, publish, lane, route, plan, processor — and the one number
    `docs/status/NEXT.md` slice 2 names as the acceptance criterion.

17. **`an_edit_with_no_engine_is_kept_and_reported_honestly`**
    Call `apply_parameter_edit(&mut model, None, "gain", "gain", 0.25, &mut status)`. Assert
    the model stored `0.25` and that `status` is unchanged (no error — this is the normal
    no-engine case, not a failure). Then call it with an unknown parameter key and assert
    the model is unchanged and `status` contains the existing "Could not update" wording
    (`crates/spectre-app/src/lib.rs:469–472`). Edge case: an engine-less edit being treated
    as an error, or a model-rejected edit being published anyway.

18. **`an_unregistered_target_reports_a_live_delivery_failure_without_rolling_back`**
    Build a `LiveEngine` whose lane registered only the Pulse target, then edit
    `saturator.mix`. Assert the model stored the new value, that `status` names the
    delivery failure, and that `health().parameters_applied` did not advance. Edge case:
    §4.4's binding rule — a lane refusal must never roll back a valid project edit.

### 5.2 Integration tests

- **`cargo test -p spectre-audio --test bridge_plan`** — tests 6, 7, 8, 9, and 11 above are
  integration-level by construction: they drive the real `CompiledPlan` through the real
  bridge with the real control transport, not a mock. The file's **existing**
  `bridge_output_matches_the_offline_render_of_identical_input` (`bridge_plan.rs:94–124`)
  must continue to pass **unchanged**, which is the load-bearing regression gate for this
  slice: it proves that adding a parameter seam did not perturb the render path when no
  parameter is published, so live and offline are still one computation.
- **`cargo test -p spectre-audio --test rt_guard`** is a **required post-edit gate, not a
  formality**, because R4-2 modifies two of the four modules that test scans and adds a
  fifth. Both existing guard tests must pass after the edits:
  `bridge_render_is_rt_clean` (`:157–187`) and
  `plan_process_is_rt_clean_through_the_null_stream` (`:259–287`), plus new test 13.
  Running them post-edit is the evidence that §4.1's argument holds; §4.1 is not itself the
  evidence.
- **`cargo test -p spectre-audio --test control_channel`** must pass unchanged apart from
  the added test 12, proving the cap did not disturb the lane's existing policy evidence —
  in particular `parameter_sweep_coalesces_to_one_latest_value` (`:45`),
  `drain_applies_only_changed_targets` (`:67`), and
  `parameter_values_survive_bit_exactly` (`:86`).
- **`cargo test --locked --workspace`** — the seven `impl AudioProcessor` sites
  (`crates/spectre-dsp/src/source.rs:76`, `:160`; `crates/spectre-dsp/src/effect.rs:50`,
  `:107`; `crates/spectre-graph/tests/graph_plan.rs:259`;
  `crates/spectre-graph/tests/containment.rs:48`, `:79`) all gain the method, so a missing
  implementation is a compile error, not a silent gap. That is the point of making it
  required.
- **Hardware drill:** `cargo test -p spectre-audio --test lifecycle_health -- --ignored
  --nocapture` re-run on macOS to confirm no headroom regression. **No Linux row may be
  recorded** until R4-3 runs (decision 23).

### 5.3 UI / E2E tests

**There is no automated GUI-driving harness in this repository** — `crates/spectre-app/tests/`
contains `app_model.rs` and `smoke_cli.rs`, and R4-1 adds `live_engine.rs`. R4-2 adds none.
Automated UI scenarios are R4-9's scope. What R4-2 provides at the end-to-end level:

- Tests 16–18 exercise the complete real path from an `AppModel` edit to a rendered block,
  through `apply_parameter_edit` — the same function `main.rs` calls. Only the egui widget
  layer is unexercised, and that layer contributes nothing but the value.
- The Shape header's three states are **not** automatically tested, because the string
  selection lives in `main.rs`, which is a binary target and unreachable from
  `crates/spectre-app/tests/`. That is a stated limit, not a hidden one: §5.4 makes all
  three a manual gate, and R4-9 owns making it automatic. If the string selection is worth
  a regression gate before then, the fix is to move it into `engine.rs` as a pure function
  of `(EngineState, bool_is_routed)` and test it there — priced in §8 Q6 rather than
  smuggled in here.
- `./spectre --smoke-test` must still exit 0 with no device, and R4-1's `engine=not-started`
  assertion in `smoke_cli.rs` must be unaffected: R4-2 adds nothing to the headless path.

### 5.4 Visual / manual verification

Run `./spectre` **with system volume low.** The audition voice is a held saw with no
amplitude envelope (`crates/spectre-dsp/src/source.rs:202–209`) and it will click at note
edges; R4-2 additionally makes a large gain move a step at a block boundary (§3.2, §8 Q1).

| Configuration | What to check |
|---|---|
| Theme variants | **N/A — the shell hard-codes a single dark palette** (`crates/spectre-app/src/main.rs:37` sets `dark_mode = true`; every color is a fixed `Color32` constant at `:9–15`). No light variant exists to check, and R4-2 does not introduce one. |
| Text size extremes | At large egui zoom, the Shape header's new second line wraps rather than clipping the 360 px slider (`main.rs:370`); the transport-bar counters do not push the Play button off-screen. |
| Screen size extremes | At the 1060 × 680 minimum (`main.rs:508`) and the 1420 × 860 default, the Shape row's 520 px minimum width (`main.rs:362`) still fits and the counters stay inside the transport panel. |
| Empty vs. populated | Shape with no device selected still shows `SHAPE_EMPTY_MESSAGE` unchanged; the `params applied` counter is absent until the first edit and never disappears afterwards; `params pending` is absent throughout a healthy session. |
| Audio behavior | With sound playing: dragging Gain to 0 silences it and back to 1 restores it; dragging Saturator drive audibly changes timbre; dragging Pulse level changes loudness. Each within one block. Stop still produces exact silence. |
| Honesty check | With the engine **not** running, a Shape edit must produce no sound and the header must say why. With the engine running but the transport stopped, an edit must still change the (silent) chain's state so that pressing Play uses the new value. The `Reset` button must behave identically to dragging to the default. |
| Click audit | Move Gain quickly from 2.0 to 0.0 on a sounding note and listen for the step discontinuity §3.2 predicts. **Record what you hear in §8 Q1** — this manual observation is the evidence Jeff needs to decide whether R4-6 must smooth, and Spectre has no measurement of it today. |
| Platform | The whole table is owed on macOS **and** on Linux. Until R4-3 discharges decision 23, only the macOS column may be recorded; the Linux column stays empty rather than assumed. |

---

## 6. Compliance & Safety Gate

### 6.1 Sensitive data classification

- [x] **No sensitive data involvement.** The feature moves `f32` device parameter values
  between an in-memory app model and an in-memory audio thread. Nothing is transmitted,
  logged to disk, or persisted; there is no network path anywhere in `spectre-app`,
  `spectre-audio`, `spectre-graph`, or `spectre-dsp`. No user identity, credential, file
  path, or device name is introduced or handled by this slice.

### 6.2 Asset provenance

- [x] **No third-party assets.** No fonts, images, samples, wavetables, presets, or data
  files are added. No new crate is added; the only third-party code anywhere near this path
  is `cpal`, already a `spectre-audio` dependency dispositioned by decision 19. All DSP,
  graph, transport, and UI code involved is original workspace code.

### 6.3 Language / claims audit

- [ ] Makes claims not supported by evidence — **no.** The only behavioral claim about
  hardware is the macOS qualification (`docs/06-plans/current-milestone.md` hardware table)
  and it is cited, not restated. The spec asserts nothing about Linux beyond a build result,
  nothing about latency in milliseconds as a target, and nothing about a benchmark product's
  internal parameter transport, which no record in the corpus describes (Appendix A).
- [ ] Promises capabilities not yet built — **no.** The spec states in §1.2 and §7.1 that
  `./spectre` makes no sound today, that R4-1 is spec'd but has zero implementation, that
  this slice adds no automation or modulation, that it adds no smoothing and will therefore
  step at block boundaries, and that the accepted architecture contract's claim about
  `Gain` smoothing does not match the shipped code.
- [ ] Uses language restricted by domain regulations — **N/A.** No regulated domain
  (medical, financial, legal) is involved.

Two user-visible-copy rules are normative. **The Shape header must never say edits reach
live audio when the engine is not running or the device is not in the live plan** — the
three-state line in §3.3 exists for exactly that reason. And **`params pending` must never
be suppressed when nonzero**, because it is the counter that says the acceptance criterion
is not being met.

### 6.4 Regulatory alignment

Confirmation against `gauntlet-output/criteria.md` Lens 3:

- **3A milestone fit.** R4-2 is verbatim slice 2 of `docs/status/NEXT.md` — "Implement
  decision 22's runtime parameter seam on `AudioProcessor` per the accepted contract, then
  connect the RT-002 parameter lane the bridge already drains" — and the second exit-evidence
  row of `docs/06-plans/current-milestone.md`. It adds no track, no clip, no device, no
  persistence, and no bounce; each is named and left to its own leaf in §7.4.
- **3B non-goal respect.** No CLAP/LV2/AU hosting, no plugin-format authoring, no cross-DAW
  preset or project compatibility, no cloud service or content store, no video scoring.
  Automation and modulation are R5+ per `current-milestone.md` §Non-goals and are explicitly
  excluded in §3.3.
- **3C deliberately small first devices.** No device is added, and no device is grown. The
  four existing devices gain one method each that assigns to fields they already have. The
  spec explicitly **declines** to add smoothing to `Gain` — the one change that would grow a
  device — and routes it to R4-6 under decision 15 with the evidence cost stated (§4.3).
- **3D originality.** No reference product's code, numbers, layout, or naming is
  transcribed. The one numeric bound introduced, `MAX_PARAMETER_TARGETS`, carries a
  Spectre-derived rationale computed from this codebase's own struct sizes and gets its own
  ledger row (§7.2). AF-4 does not trigger.
- **3E platform commitment.** The slice contains no platform-conditional code at all
  (§4.6); every test runs on the null backend and is therefore identical on both targets.
  No Linux claim is made, and §5.4's manual protocol is owed on both platforms with only
  the macOS column recordable until R4-3.
- **3F accessibility trajectory.** No icon-only control, no color-only state, no custom
  widget; delivery state is a sentence and two named counters. The known eframe
  accessibility-feature gap is recorded, not hidden, and no existing focus order changes.

---

## 7. Gap Analysis vs. Current State

### 7.1 What exists today

Verified by reading the files at commit `2e005e5`. Each row is checkable in one `Read`.

**Implemented and verified — the parameter transport, with nothing consuming it:**

| Element | Path | Note |
|---|---|---|
| RT-002 latest-wins parameter lane: `ParameterTarget`, `ParameterSlot`, `ParameterLane`, `ParameterWriter`, `ParameterReader` | `crates/spectre-audio/src/control.rs:22–145` | one `AtomicU32` value + `AtomicU32` version per target (`:67–70`); `set` stores bits then bumps version (`:99–100`); `drain` reads version before bits and skips unchanged slots (`:127–137`) |
| Lane depths and the whole-transport halves | `crates/spectre-audio/src/control.rs:17–19`, `:173–189`, `:195–239` | 1,024 / 64 / 32; `control_channel` refuses duplicate targets (`:200–204`) but has **no cap on target count** |
| Bounded wait-free SPSC ring | `crates/spectre-audio/src/spsc.rs` | `push` returns the rejected value (`:80–93`) |
| Off-thread reclamation | `crates/spectre-audio/src/control.rs:274–281`, `:314–324` | `retire` hands a value back on overflow rather than dropping it on the render thread |
| Parameter-lane behavioral evidence | `crates/spectre-audio/tests/control_channel.rs:45`, `:67`, `:86`, `:107`, `:132` | 5,000-write sweep coalescing to one application; changed-only drain; bit-exact signed-zero/subnormal/NaN delivery; target validation; duplicate refusal |

**Implemented — the render path, with the parameter step stubbed:**

- `RenderBridge` (`crates/spectre-audio/src/bridge.rs:119–290`) drains the lane once per
  block, in the correct position — after `apply_transport` and `collect_notes` (`:175–176`),
  before `plan.process` (`:192–195`) — and passes a **discarding closure**, counting the
  result into `parameters_pending` (`:178–186`). The counter's accessor is at `:82–85` and
  its field at `:29`.
- `CompiledPlan` (`crates/spectre-graph/src/lib.rs:418–545`) holds
  `processors: Vec<Box<dyn AudioProcessor>>` (`:420`) parallel to `steps: Vec<PlanStep>`
  (`:419`), both pushed in the same order by `compile` (`:310`, `:317`) and zipped by
  `process` (`:482`). It exposes `max_frames`, `step_count`, `containment`, `process`, and
  `last_output` — **and no parameter method of any kind.**
- RT-003 containment inside `process` (`:401–414`, `:512–521`) flushes denormals to signed
  zero and silences a node whose output is non-finite.

**Implemented — the DSP seam, without the method this slice adds:**

- `AudioProcessor` has exactly two methods, `io` and `process`
  (`crates/spectre-dsp/src/io.rs:163–172`), with the `Send` bound added at R3 slice 5.
  **There is no `set_parameter`, no parameter method, and no `ParameterError` type
  anywhere in `spectre-dsp`.**
- `Gain::set_gain` (`crates/spectre-dsp/src/effect.rs:45–47`), `Saturator::set_drive`
  (`:98–100`), and `Saturator::set_mix` (`:102–104`) exist as **inherent** methods that
  clamp via their descriptors. They are unreachable through `Box<dyn AudioProcessor>`,
  which is what a plan holds, and **nothing in the workspace calls any of them** — the only
  occurrences of those three names under `crates/` are their own declarations.
- `PulseInstrument` (`crates/spectre-dsp/src/source.rs:113–158`) and `ToneSource`
  (`:50–74`) have **no setters at all**; their values are constructor arguments only.
- `DspParameter::clamp` (`crates/spectre-dsp/src/parameter.rs:49–51`) and
  `ParamSpec::clamp` (`crates/spectre-core/src/param.rs:119–124`) already provide the
  callback-safe containment the setter needs: non-finite → descriptor default, otherwise
  `f64::clamp`.
- `DeviceParameterSnapshot` (`crates/spectre-dsp/src/parameter.rs:81–126`) already carries
  `device_instance_id`, `device_key`, `parameter_instance_id`, `parameter_key`, and a
  clamped `value` — exactly the join this slice's route table needs.

**Prototyped — the app shell.** `crates/spectre-app/src/main.rs`'s own header calls it an
"interaction prototype" and states "audio and persistence wiring remain out of scope."
Shape renders descriptor-backed sliders for the selected device (`main.rs:336–398`),
collects changed values into `edits` (`:380–382`), and applies them after the scroll area
closes (`:389–397`) through `set_device_parameter_from_ui`
(`crates/spectre-app/src/lib.rs:462–474`) into `AppModel::set_device_parameter`
(`:385–403`), which clamps and stores. `AppModel::device_parameter_snapshot` (`:348–383`)
publishes the fixed four-entry fixture — `pulse.level`, `gain.gain`, `saturator.drive`,
`saturator.mix`.

**Absent — this is the gap R4-2 closes:**

- **No callback-safe way to change a parameter on a live processor.** The trait has two
  methods; the plan has no parameter method; the bridge's parameter closure is `|_, _| {}`.
- **No mapping from a lane `ParameterTarget` to a plan node and key.** The lane speaks in
  project-instance `ObjectId` pairs (`control.rs:22–26`); the plan speaks in `NodeId` and
  the devices speak in `DeviceParameterKey`. Nothing joins them today.
- **No cap on the registered target count**, so `ParameterReader::drain`'s per-block walk
  over every slot (`control.rs:127`) is unbounded in principle.
- **No app-side publication path.** `spectre-app` does not depend on `spectre-audio`
  (`crates/spectre-app/Cargo.toml` `[dependencies]`: `eframe`, `spectre-core`,
  `spectre-dsp`, `spectre-project`, `serde`, `serde_json`), so no code in the app can reach
  a `ControlSender`.
- **`./spectre` produces no sound.** `AppModel::toggle_play` (`crates/spectre-app/src/lib.rs:268–275`)
  applies a `TransportCommand` to an in-memory `Transport` and returns. The transport bar's
  `ENGINE OFFLINE` (`main.rs:79`) and `CPU —` (`:80`) are hard-coded strings.

**Gated (accepted, deliberately not implemented):**

- **Decision 22 — this slice.** Accepted design-only on 2026-08-09, option (a), specified
  in `docs/03-architecture/dsp-device-io.md` §"Runtime parameter seam", implementation
  assigned to R4 following the CORE-004 precedent. Option (b) is rejected on its face.
- **R4-1 — live audio wiring.** `gauntlet-output/manifest.md` records R4-1 as
  **spec-pass, score 2.950, spec iteration 2, implementation iteration 0.** The spec exists;
  `crates/spectre-app/src/engine.rs` does not, and neither does
  `crates/spectre-app/tests/live_engine.rs`. R4-2 is written against R4-1's declared API and
  §7.4 makes that dependency hard.
- **Decision 23 — Linux device qualification.** Table row reads "not run".
- **Decision 17 — accessibility audit** scoped at R4, gated before beta.
- **PROD-002 — automated/overridden parameter states with a restore action.** Proposed,
  evidence gated at R9. R4-2 introduces no automation, so there is nothing to display.

**Contract-versus-code discrepancy, flagged rather than resolved (AF-1 handling).**

`docs/03-architecture/dsp-device-io.md` states twice that `Gain` smooths: §"Initial native
devices" describes it as "stereo linear gain with click-resistant smoothing", and
§"Runtime parameter seam" says "`Gain` already smooths; devices whose parameters would
click MUST smooth internally rather than requiring the caller to ramp." **The shipped
`Gain` has no smoothing state**: the struct is a single `gain: f32`
(`crates/spectre-dsp/src/effect.rs:29–31`) and `process` multiplies by it directly with no
target, coefficient, or ramp (`:62–71`). `Gain::set_gain` (`:45–47`) assigns immediately.

This spec does not resolve the discrepancy, because resolving it either way is a decision
with real consequences: adding smoothing changes rendered output and breaks the bit-exact
offline/live hash equality the engine's central evidence rests on; deleting the sentence
amends an accepted architecture contract. Both are Jeff's. The spec's obligations here are
to (a) not claim smoothing exists, (b) not silently rely on it, (c) make the resulting
click visible in copy, in §5.4's click audit, and in the QA record, and (d) log it. It is
§8 Q1 and is filed for `gauntlet-output/decisions-needed.md` as **D-R3**.

### 7.2 Delta to spec

**New files**

- `crates/spectre-audio/src/route.rs` — `ParameterRoute`, `ParameterRoutes`, `RouteError`;
  sorted table, binary-search resolve, `empty()`.

**Modified files**

- `crates/spectre-dsp/src/io.rs` — add `ParameterError`; add the required
  `AudioProcessor::set_parameter`; `use crate::parameter::DeviceParameterKey`.
- `crates/spectre-dsp/src/lib.rs` — re-export `ParameterError` from the `io` module list.
- `crates/spectre-dsp/src/source.rs` — `set_parameter` for `ToneSource` and
  `PulseInstrument`.
- `crates/spectre-dsp/src/effect.rs` — `set_parameter` for `Gain` and `Saturator`, reusing
  the existing inherent setters.
- `crates/spectre-graph/src/lib.rs` — `CompiledPlan::set_parameter`; `PlanError::UnknownParameterNode`
  and `PlanError::Parameter` with `Display` arms.
- `crates/spectre-graph/tests/graph_plan.rs` — `ImpulseSource` (`:259`) implements the new
  method; add tests 4 and 5.
- `crates/spectre-graph/tests/containment.rs` — `PoisonSource` (`:48`) and `PoisonEffect`
  (`:79`) implement the new method.
- `crates/spectre-audio/src/lib.rs` — `pub mod route;`.
- `crates/spectre-audio/src/control.rs` — `MAX_PARAMETER_TARGETS`;
  `ControlError::TooManyTargets` with its `Display` arm; the length check in
  `control_channel`.
- `crates/spectre-audio/src/bridge.rs` — `routes` field; `with_parameter_routes`
  constructor with `new` delegating; `parameters_applied` counter and accessor;
  `parameters_pending`'s doc comment narrowed; the parameter step at `:178–186` replaced.
- `crates/spectre-audio/tests/rt_guard.rs` — **`RT_MODULES` becomes `[&str; 5]` with
  `"src/route.rs"` added** (`:293–298`); add test 13.
- `crates/spectre-audio/tests/bridge_plan.rs` — tests 6, 7, 8, 9, 11.
- `crates/spectre-audio/tests/control_channel.rs` — test 12.
- `crates/spectre-dsp/tests/devices.rs` — tests 1, 2, 3.
- `crates/spectre-app/src/lib.rs` — `ParameterEdit`; `AppModel::edit_device_parameter`;
  `set_device_parameter` re-expressed in terms of it.
- `crates/spectre-app/src/engine.rs` **(R4-1's file)** — register the snapshot-derived
  target set; `FixtureNodes`; `build_parameter_wiring`; `EngineParts.targets`;
  `LiveEngine::send_parameter`; `apply_parameter_edit`; `EngineHealth.parameters_applied`.
- `crates/spectre-app/src/main.rs` — Shape edit loop calls `apply_parameter_edit`; Shape
  header's three-state second line; `params applied` / `params pending` in R4-1's counter
  cluster.
- `crates/spectre-app/tests/app_model.rs` — test 15.
- `crates/spectre-app/tests/live_engine.rs` **(R4-1's file)** — tests 16, 17, 18.
- **`docs/01-requirements/requirements-ledger.md`** — one rationale row for
  `MAX_PARAMETER_TARGETS`, carrying §4.2's text. PROD-003
  (`docs/01-requirements/requirements-ledger.md:64`) requires every numeric limit's
  rationale to be recorded **in that ledger**, and decision 16 makes it a standing rule, so
  §4.2's prose alone does not discharge it. The row also updates RT-002's acceptance
  evidence line to name the end-to-end application evidence, since RT-002's current
  evidence covers the transport only.
- `docs/03-architecture/dsp-device-io.md` — mark the "Runtime parameter seam" section
  implemented, record the concrete method signature that landed, and — **pending Q1** —
  correct or substantiate the two `Gain` smoothing sentences. Not to be edited before Q1 is
  answered.
- `gauntlet-output/decisions-needed.md` — add **D-R3**, the `Gain` smoothing discrepancy
  (§7.1, §8 Q1).
- `docs/status/STATUS.md`, `docs/status/NEXT.md`, `docs/06-plans/current-milestone.md` —
  update the known-gaps and inherited-debt claims when the slice lands, per
  `docs/README.md`'s working rule. Status moves to `implemented`, **not** `verified`, until
  §5.4's manual protocol and a hardware re-run pass.

**Deliberately not modified:** `crates/spectre-audio/src/spsc.rs`,
`crates/spectre-audio/src/null.rs`, `crates/spectre-audio/src/midi.rs`,
`crates/spectre-audio/src/cpal_backend.rs`; all of `spectre-core`, `spectre-project`, and
`spectre-offline`. No second render path is created, no DSP algorithm is changed, and no
device grows a new parameter.

**On the RT-scanned set specifically.** `rt_guard.rs`'s `RT_MODULES` (lines 293–298) scans
`src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, and `src/null.rs` — **not** `midi.rs`.
Two of those four, `bridge.rs` and `control.rs`, **are modified by this slice**, and the
modified-files list above says so. The RT-001 argument therefore rests on the content of
those edits and on a post-edit `rt_guard` run (§4.1, §5.2), never on the scanned set being
untouched. `spsc.rs` and `null.rs` are untouched. The new `route.rs` is callback-reachable
and is **added to the scan** rather than left outside it.

**Migrations / schema changes:** none. **New third-party dependencies:** none.

### 7.3 Estimated scope

**M.** Justification: roughly 150–220 new lines of implementation spread thinly across five
crates — one trait method with four small implementations, one plan method, one small new
module, one bridge closure body, one model method, one app-thread function — plus roughly
400 lines of test across six files. It is above **S** because it adds a **required** method
to an accepted trait, which is a breaking change to every `AudioProcessor` implementation in
the workspace (seven sites, four shipping and three test-only), and because it changes two
of the four RT-scanned modules and extends the scan itself. It is below **L** because it
writes no DSP algorithm, adds no dependency, changes no schema, allocates nothing new on
either thread, adds no platform-conditional code, and reuses the transport, plan, bridge,
and telemetry exactly as R3 qualified them.

### 7.4 Blocking dependencies

- **R4-1 (`live-audio-wiring`) hard-blocks the app-thread half of this slice.** R4-2's
  §4.3 declares functions in `crates/spectre-app/src/engine.rs`, a file R4-1 creates and
  which does not exist; R4-2's tests 16–18 live in `crates/spectre-app/tests/live_engine.rs`,
  also R4-1's; and `spectre-app` cannot reach a `ControlSender` at all until R4-1 adds the
  `spectre-audio` dependency. R4-1 is spec-pass with zero implementation
  (`gauntlet-output/manifest.md`), so **this is a real ordering constraint, not a
  formality.**
- **The DSP, graph, and audio halves are not blocked.** Everything from
  `AudioProcessor::set_parameter` through `RenderBridge`'s route table and tests 1–14 can be
  implemented and proved today against the existing crates and the null backend, with no
  device and no app wiring. If R4-1 slips, this slice can land in that order — and doing so
  would let R4-1's own `build_engine_parts` register real targets from the start instead of
  an empty set.
- **R4-3 (`linux-device-qualification`) blocks any Linux claim**, not this work. No Linux
  row may be added to the qualification table by R4-2.
- **R4-4 (`track-model`) and R4-6 (`first-devices`) depend on this slice**, not the reverse.
  Every device R4-6 adds must implement the new required method — that is the point of
  making it required — and every device R4-4 puts in the live plan must register its targets
  and routes through §4.3's construction path.
- **R4-8 (`offline-bounce`) depends on §4.4's single-value-of-record rule.** A bounce that
  must reconcile two independently-mutated parameter stores would be a much harder feature;
  this slice's asymmetry is what keeps it easy.
- **§8 Q1 blocks one documentation edit**, not the implementation: the `dsp-device-io.md`
  smoothing sentences must not be touched until Jeff decides.
- **External gates:** an output device for §5.4's manual protocol on macOS, and a Linux box
  with real audio before any Linux row is recorded.

---

## 8. Open Questions

- **Q1 — The accepted contract says `Gain` smooths; the shipped `Gain` does not (§7.1).**
  Three resolutions, each with a cost. (a) Add smoothing to `Gain` in this slice: changes
  rendered output for identical inputs, breaks the bit-exact live/offline hash equality in
  `crates/spectre-audio/tests/bridge_plan.rs` and every offline fixture hash, and needs a
  smoothing-time-constant rationale row under PROD-003 — a large change dressed as a small
  one. (b) Assign smoothing to R4-6 with the rest of the device work under decision 15 and
  correct the two sentences in `dsp-device-io.md` to describe the code as it is: cheap, and
  what this spec assumes as the default. (c) Leave the contract as the target and accept
  that it currently describes intent rather than code. §5.4's click audit is designed to
  give Jeff the ear evidence before he chooses. Logged as **D-R3**. — blocks §4.3, §7.2's
  `dsp-device-io.md` row, R4-6.
- **Q2 — Should a parameter change be applied when it arrives, or only when the transport is
  playing?** This spec applies it on every block regardless of transport, which means an
  edit made while stopped is already in effect when Play is pressed. That matches the
  bridge's existing behavior — `RenderBridge::render` executes the plan every block
  regardless of `self.transport` (`bridge.rs:162–208`), the property R4-1's Q1 also
  surfaced — and it is what makes silent sound-design possible. The alternative would make
  the sounding state depend on when the edit was made, which seems clearly worse, but it is
  a behavior choice and it is Jeff's. — blocks §3.2.
- **Q3 — Should `MAX_PARAMETER_TARGETS` live in `control.rs` or in the app's engine layer?**
  Putting it in `control.rs` means modifying an RT-scanned module and makes the bound
  binding for every caller, which is why this spec chose it. Putting it in `engine.rs`
  would leave the scanned set less disturbed but would leave the transport itself
  uncapped, so a future caller could reintroduce the unbounded drain. — blocks §4.2, §7.2.
- **Q4 — Should the render side resolve routes by lane index instead of by target?**
  `ParameterReader::drain` hands the closure a `ParameterTarget` (`control.rs:135`); it
  already knows the slot `index` and could hand that over too, making resolution `O(1)`
  array indexing instead of an `O(log n)` binary search. That is strictly better on the
  callback path but changes `drain_parameters`'s public closure signature, which touches an
  accepted, tested module and three existing tests. The binary search is chosen here because
  it needs no change to the accepted transport; the index variant is a clean follow-up if
  the cost ever shows up in a measurement. — blocks §4.2, §4.7.
- **Q5 — Should the app publish on every widget change, or only on drag release?** This
  spec publishes on every change, relying on latest-wins coalescing, which gives continuous
  audible feedback during a drag. The alternative — publish on release — would give a
  single jump and needs no lane at all, and it is what a smoothing-free `Gain` might argue
  for, since fewer steps means fewer clicks. Interacts with Q1. — blocks §3.2.
- **Q6 — Should the Shape header's three-state string move out of `main.rs`?** As specified
  it lives in the binary target and is manual-only until R4-9 (§5.3). Moving it into
  `engine.rs` as a pure function of `(EngineState, is_routed)` would make it a regression
  gate for one small function's worth of cost. This is the same trade R4-1's iteration-2
  review recorded for its own binding rule, and the same answer probably applies — but it is
  scope R4-2 did not ask for. — blocks §5.3.
- **Q7 — Is a benchmark research need worth opening here?** No record in the accepted corpus
  describes any product's internal control→render parameter transport, its application
  granularity, or its behavior when a value cannot reach a live processor (Appendix A). The
  gap is real, but it may be unfillable from public documentation, which describes what
  users see rather than how the value crosses the thread boundary. Worth a ledger entry as a
  known-unfillable research need, or worth leaving as a stated gap? — blocks Appendix A.
- **Q8 — Should `parameters_pending` be renamed now that its meaning has narrowed?** The
  acceptance criterion in `docs/status/NEXT.md` is stated in terms of that exact name, so
  this spec keeps it. Once the criterion is met and recorded, `parameters_undelivered` would
  describe the counter more accurately. Renaming it later costs one accessor and its
  callers; renaming it now would make the acceptance criterion read against a name that no
  longer exists. — blocks §4.2.

---

## Appendix A — Benchmark evidence used, and where it does not exist

Cited, from the accepted corpus (Ableton Live 12 `OBS-AB12-`, Phase Plant `OBS-PP-`,
VCV Rack 2 `OBS-VCV-VOLT-`, Serum 2's two records, with Bitwig `OBS-BW53-` as corroborating
evidence outside Jeff's benchmark set):

- **`OBS-VCV-VOLT-006`** (VCV Rack 2): each cable imposes a one-sample delay; modules should
  output 0 on NaN/infinity detection. This is RT-003's recorded provenance in
  `requirements-ledger.md`. Spectre **converges**, and R4-2 inherits it rather than
  reimplementing it: a parameter driven to a pathological value is contained by the device's
  own clamp first and by `contain_channel` second, and §5.1 test 11 asserts the chain stays
  finite when a parameter is driven to zero.
- **`OBS-AB12-AUTO-004`** (Live 12, §25.4) and **`OBS-BW53-AUTO-002`** (Bitwig 5.3): two
  researched products converge on the same override/restore semantics — manually changing an
  automated control overrides its automation with a visible indicator change, and an explicit
  restore action returns control. This convergence is already carried into **PROD-002**.
  Spectre **defers rather than diverges**: R4-2 introduces no automation, so there is no
  automated state to override and no restore action to offer. §3.3 says so explicitly and
  §4.4 preserves the one-value-per-parameter model that PROD-002 will later need to
  decompose into base + automation + modulation. Claiming a modulation-visibility design
  here would be inventing a surface for a source that does not exist.
- **`OBS-AB12-AUTO-003`** (Live 12, §25.2.1): recording-gesture semantics differ by input —
  mouse edits behave as Touch (punch out on release), MIDI controller input behaves as
  Latch. This is the closest the corpus comes to describing *when* a parameter edit takes
  effect, and it describes automation recording, not live application. Spectre's Q5 (publish
  on change vs. on release) is the analogous question and the record is **not** sufficient to
  settle it, which is why Q5 is a question rather than a citation.
- **`OBS-PP-ARCH-003`** (Phase Plant): audio-rate modulation semantics depend on the target
  parameter — Phase gives classic FM, Level gives ring modulation, Pitch gives exponential
  FM described as hard to control. Spectre **converges on the underlying principle** that a
  parameter's identity determines what changing it means, which is why this slice addresses
  parameters by their stable `DeviceParameterKey` and lets each device decide what applying
  that key does, rather than routing a positional index the way a plugin ABI would.
- **`OBS-PP-ARCH-004`** (Phase Plant): shared generator parameters carry declared ranges —
  Level 0–200%, Harmonic multiplier, Shift in Hz, Phase Offset in degrees. Spectre
  **converges on descriptor-declared ranges** — CORE-002 and `DspParameter` already require
  it — and **diverges on the numbers**: Spectre's ranges are its own
  (`crates/spectre-dsp/src/effect.rs:19–25`, `crates/spectre-dsp/src/source.rs:27–46`), and
  decision 16 forbids importing any of Phase Plant's. Not one number in this spec comes from
  that record.
- **`OBS-SR2-CPU-001`** (Serum 2, support article 51): official guidance treats high
  per-oscillator unison counts as a significant CPU cost. Used here only for the narrow claim
  that per-parameter CPU cost is a user-visible concern in shipping products, which is why
  §4.7 computes this slice's added per-block cost from source rather than waving at it.
  Serum 2 has exactly two citable records and this is one of them.

**Named gaps — evidence that does not exist and was not invented:**

- **No benchmark in the corpus has a citable observation about a DAW's or synth's internal
  control→render parameter transport.** The corpus documents what a user sees and does — LED
  states, restore buttons, gesture semantics, envelope editing — not whether the value
  crosses to the audio thread through a lock-free slot, a queue, or a message, nor at what
  granularity it is applied, nor what happens when it cannot be delivered. Every
  transport-level decision in this spec — latest-wins per target, block-boundary
  application, counted undelivered edits, a sorted route table — is Spectre's own, follows
  from decisions 21 and 22 and from RT-001/002, and is recorded as a research need in §8 Q7
  rather than dressed in a citation.
- **Logic Pro: zero citable behavioral observations.** `logic-pro.md` is inventory-only.
  This spec asserts nothing about Logic Pro's parameter handling, control surface, or engine.
- **Serum 2: exactly two citable records** (`OBS-SR2-CPU-001`, `OBS-SR2-KB-001`), and its
  dossier is `blocked-source-gap`. Only the CPU record is used, only for the narrow claim
  above. No Serum 2 parameter, modulation, or voice behavior is asserted anywhere in this
  spec.
- **No benchmark record describes surfacing a count of *undelivered* parameter edits to the
  user.** Spectre's `params pending` counter is not claimed as superior on evidence; it
  follows from `vision.md`'s "Rust-native engine with a published realtime contract" and the
  alpha release bar's "honest telemetry, no fake surfaces". A parameter path that silently
  drops edits is the failure mode this counter exists to make impossible to hide, and that
  reasoning is Spectre's, not a benchmark's.

---

**End of spec.**
