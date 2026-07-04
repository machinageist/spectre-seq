# Clean-Room Spec Codebase Improvement Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Convert the Ableton Live, Bitwig Studio, VCV Rack, Serum 2, and Phase Plant clean-room specs into small verified improvements to Geist's Rust DAW model, engine, project schema, and UI.

**Architecture:** Keep Geist's native-device/VST-host split. First implement common data contracts (parameters, clip launch, modulation, assets, containers), then wire app/session/project/UI behavior in small slices, then deepen DSP/source families. Do not clone vendor UI, names, presets, content, algorithms, or file formats.

**Tech Stack:** Rust workspace (`crates/*`, `app/geist-daw`), egui UI shell, `geist-core`, `geist-graph`, `geist-timeline`, `geist-automation`, `geist-project`, `geist-synth`, `geist-modular`, `geist-ui`.

---

## Source specs now in repo

- `docs/specs/ableton-live-clean-room-spec.md` — 626 lines; Live manual behavior plus source-bound exclusions.
- `docs/modular_rack_spec.md` — 1,263 lines; feature-by-feature VCV Rack manual coverage.
- `docs/specs/serum-2-clean-room-spec.md` — 1,228 lines; public Serum 2 product/PDF/support coverage.
- `docs/specs/geist-modular-synth-spec.md` — 733 lines; Phase Plant/Kilohearts generator/modulation/lane/asset coverage.
- `docs/specs/bitwig-studio-clean-room-spec.md` — 713 lines; Bitwig user-guide feature coverage.
- `docs/specs/clean-room-spec-completeness-audit.md` — final readiness gate.
- `docs/specs/clean-room-spec-audit-metrics.json` — deterministic audit metrics.

## Spec completeness gate

Do not start code implementation from these specs until each source family has an exhaustive feature-by-feature matrix, not only a behavioral summary. A spec is ready for implementation planning only when it includes:

- A provenance list of every public manual/source page used.
- A manual/table-of-contents coverage matrix with each relevant chapter or feature named.
- Per-feature behavioral semantics, not just product-level summary.
- Data model implications: project/session fields, asset references, parameter identity, routing identity, preset/state boundaries.
- Realtime implications: callback safety, scheduling, bounded buffers/pools, graph edge rates, offline render constraints.
- UI/command implications: undoable command paths, view state vs project truth, interaction constraints, validation messages.
- Geist mapping notes that name target crates/files where known.
- Explicit non-goals for vendor UI, assets, presets, algorithms, private formats, screenshots, and source code.
- Gap entries with concrete reasons and source limitations. Avoid vague `partial` labels; say exactly which public information is unavailable or intentionally out of scope.

Current known first-pass insufficiencies to resolve before coding:

- Resolved 2026-07-03: VCV Rack, Serum 2, Phase Plant, and Bitwig specs were expanded after the first pass.
- Resolved 2026-07-03: Ableton `partial` coverage labels were replaced with source-bound coverage and an added feature inventory for MPE, audio-to-MIDI, stems, grooves, tuning, comping, browser/file management, and device/rack details.
- Remaining source-bound exclusions are accepted for planning: exact vendor algorithms, exact private schemas, factory content, screenshots/UI assets, and undocumented defaults.

## Spec-to-implementation traceability

| Implementation area | Primary spec sources |
|---|---|
| Session launcher, scenes, follow/launch settings | Ableton §§3, 6, 16A; Bitwig §§7, 22 |
| Arrangement clips, audio/note events, comping | Ableton §§4, 5, 16A; Bitwig §§6, 11, 12 |
| Audio warp/stretch metadata and render/bounce | Ableton §§7, 14, 15; Bitwig §§6, 15; Serum/Phase asset sections for sample handling |
| Automation vs clip envelopes vs modulators | Ableton §§12–13; Bitwig §§10, 17; Phase Plant modulation sections; Serum modulation sections |
| Native synth source slots and asset-backed oscillators | Serum oscillator/source matrix; Phase Plant generator stack/source sections |
| Modular/grid patching and voltage/channel policy | VCV Rack voltage/polyphony/core sections; Bitwig Grid §§18; Phase Plant audio-rate modulation/Aux sections |
| Nested device containers, racks, macros, zones | Ableton device/rack §§11, 16A; Bitwig advanced device concepts §17; Phase/Serum macro/mod sections |
| Browser, asset map, missing files, packages/content | Ableton browser/file sections; Bitwig browser/dashboard sections; Serum/Phase preset/asset sections |
| Controller/remotes/MPE/expression | Ableton MPE/controller sections; Bitwig controller/note-expression sections; Serum/Phase performance sources |
| Analysis scopes/meters | Bitwig device-description scope; VCV module meters/CPU; existing Geist meter paths |

## Current code context

- `crates/geist-core/src/port.rs` has strict same-type/same-channel connection validation. This is good for DAW safety but too rigid for VCV/Grid-style internal patch surfaces.
- `crates/geist-synth/src/engine/params.rs` is still a scaffold. Rich Serum/Phase Plant-style source slots need typed parameter descriptors first.
- `crates/geist-synth/src/engine/osc_stack.rs` is currently hard-coded to two wavetable oscillators plus FM/unison. Serum/Phase Plant specs point toward source slots.
- `crates/geist-synth/src/engine/mod_matrix.rs` has fixed opaque index routes. Bitwig/Phase Plant specs need stable source/target identities, signed depth, source polarity, and base/automation/modulation separation.
- `app/geist-daw/src/engine.rs` already has session clip launching and quantization. Ableton/Bitwig specs need per-clip launch settings, stop-button state, follow actions, and scene tempo/time-signature metadata.
- `app/geist-daw/src/session.rs` persists many app/device values through fixed parameter ID ranges. Specs point toward typed nested device/rack/modulator state.
- `crates/geist-timeline/src/clip.rs` has audio/MIDI/automation clip foundations. Ableton/Bitwig specs need clip launch settings, audio event metadata, warp/stretch metadata, and clip envelopes.
- `crates/geist-ui/src/model.rs` already has mixer, rack, timeline, session, graph, browser-style surfaces. It should consume richer descriptors rather than fixed slot constants in `app/geist-daw/src/studio.rs`.

## Implementation order

### Progress

- 2026-07-03: Task 1 started and completed for `geist-synth`: added typed native synth parameter descriptors, stable IDs/order, default/range/unit metadata, and descriptor tests. Validation: `cargo test -p geist-synth --test params_descriptors`, `cargo test -p geist-synth params`, `cargo check -p geist-synth`, and `cargo test -p geist-synth` passed.

### Task 1: Add typed native parameter descriptors

**Objective:** Replace scaffolded synth param definitions with a reusable internal parameter descriptor model suitable for synth, FX, modulators, and automation.

**Files:**
- Modify: `crates/geist-synth/src/engine/params.rs`
- Consider shared type in: `crates/geist-core/src/params.rs`
- Test: `crates/geist-synth/src/engine/params.rs`

**Steps:**
1. Inspect `crates/geist-core/src/params.rs` and neighboring device descriptors.
2. Add typed destination identifiers for synth parameters without vendor names.
3. Include normalized/default/range/unit/taper metadata.
4. Add tests for default values, clamping, and stable descriptor count/order.
5. Run: `cargo test -p geist-synth params` and `cargo check -p geist-synth`.

### Task 2: Refactor synth oscillators into bounded source slots

**Objective:** Prepare Serum/Phase Plant-style multi-source patches without implementing every source family at once.

**Files:**
- Modify: `crates/geist-synth/src/engine/osc_stack.rs`
- Modify: `crates/geist-synth/src/engine/voice.rs`
- Test: existing synth engine tests plus new source-slot tests.

**Steps:**
1. Introduce a fixed-capacity source slot enum with initial variants for existing wavetable A/B behavior and disabled slots.
2. Preserve existing rendered output behavior as much as possible.
3. Keep render loops allocation-free.
4. Add tests proving disabled slots are silent, two default slots still produce sound, and per-slot tuning is deterministic.
5. Run: `cargo test -p geist-synth`.

### Task 3: Make modulation routes identity-based

**Objective:** Move from opaque source/destination indices to stable source/target IDs that can back Bitwig/Phase Plant/Serum-style modulation UI.

**Files:**
- Modify: `crates/geist-synth/src/engine/mod_matrix.rs`
- Modify after preflight: `crates/geist-automation/src/route.rs`, `crates/geist-automation/src/matrix.rs`
- Test: synth and automation route tests.

**Steps:**
1. Define route identity types and polarity/depth semantics.
2. Keep a bounded fixed route array for realtime resolution.
3. Preserve fast `resolve()` with no allocation.
4. Add tests for multiple routes to one destination, bipolar/unipolar behavior, missing source/target skip, and clamp policy at parameter application boundary.
5. Run: `cargo test -p geist-synth mod_matrix` and `cargo test -p geist-automation`.

### Task 4: Add clip launch settings and stop-button state

**Objective:** Close the first Ableton/Bitwig session-launch gap already adjacent to existing code.

**Files:**
- Modify: `app/geist-daw/src/engine.rs`
- Modify: `app/geist-daw/src/session.rs`
- Modify: `crates/geist-ui/src/model.rs`
- Test: app tests touching session launching and session persistence.

**Steps:**
1. Inspect existing `SessionSlot`, `SessionClips`, and session roundtrip tests.
2. Add per-slot launch quantization override, stop-button enabled flag, and trigger mode enum with only the modes Geist can support now.
3. Persist those fields in `StudioSession`/`TrackSession` without breaking old saves.
4. Expose the state in the UI model but keep UI controls minimal.
5. Add tests for scene launch not stopping a track when stop button is disabled.
6. Run targeted app tests, then `cargo test -p geist-daw` if available for the app package.

### Task 5: Add follow-action data model without playback behavior

**Objective:** Create the stable data shape before implementing follow-action scheduling.

**Files:**
- Modify: `crates/geist-timeline/src/clip.rs` or a new `crates/geist-timeline/src/launch.rs`
- Modify: `app/geist-daw/src/session.rs`
- Test: timeline/session serialization tests.

**Steps:**
1. Add a minimal `FollowAction` enum and `LaunchSettings` struct.
2. Attach launch settings to launcher clips, not arrangement-only clips.
3. Persist defaults so existing projects load unchanged.
4. Add tests for default/no-action behavior and roundtrip.
5. Run: `cargo test -p geist-timeline` and app session tests.

### Task 6: Add clip-envelope model

**Objective:** Support Ableton-style clip-relative envelopes and Bitwig-style event/clip expression without overloading arrangement automation.

**Files:**
- Modify: `crates/geist-timeline/src/clip.rs`
- Modify: `crates/geist-automation/src/lane.rs` or add a reusable curve container.
- Test: timeline/automation tests.

**Steps:**
1. Extract reusable breakpoint curve storage if needed.
2. Add clip-relative envelope container with optional independent loop length.
3. Keep base parameter, arrangement automation, and clip modulation conceptually separate.
4. Add tests for clip-relative evaluation and unlinked loop wrap.
5. Run: `cargo test -p geist-timeline` and `cargo test -p geist-automation`.

### Task 7: Add audio clip warp/stretch metadata only

**Objective:** Prepare for Ableton/Bitwig audio clip behavior without claiming proprietary warp parity.

**Files:**
- Modify: `crates/geist-timeline/src/clip.rs`
- Modify: `crates/geist-project/src/schema.rs`
- Test: timeline/project roundtrip tests.

**Steps:**
1. Add original tempo, warp enabled flag, warp marker list, stretch mode enum, gain, pitch, fades, and reverse metadata.
2. Do not implement DSP stretching in this task.
3. Add project migration/default tests.
4. Run: `cargo test -p geist-timeline` and `cargo test -p geist-project`.

### Task 8: Add modular patch compatibility policy

**Objective:** Let Geist support both typed DAW ports and VCV/Grid-style flexible patching without weakening normal graph validation.

**Files:**
- Modify: `crates/geist-core/src/port.rs`
- Modify: `crates/geist-graph/src/edge.rs` if validation owns policy there.
- Test: port/graph routing tests.

**Steps:**
1. Add a connection policy enum: strict typed DAW graph vs modular patch surface.
2. In modular policy, allow compatible voltage-like audio/CV/gate/control connections with explicit channel adaptation rules.
3. Keep MIDI/note/event ports typed unless a specific adapter exists.
4. Add tests proving strict mode behavior remains unchanged.
5. Add tests for modular mono/poly adaptation policy.
6. Run: `cargo test -p geist-core port` and `cargo test -p geist-graph`.

### Task 9: Introduce graph subgraph/container metadata

**Objective:** Prepare Bitwig Grid and Phase Plant generator groups/effect lanes as internal subgraphs.

**Files:**
- Modify: `crates/geist-graph/src/graph.rs`
- Modify: `crates/geist-graph/src/node.rs`
- Modify: `crates/geist-project/src/schema.rs`
- Test: graph/project tests.

**Steps:**
1. Add container/group IDs and parent-child metadata on editable graph nodes.
2. Keep compiled process list flat for audio-thread simplicity.
3. Add tests that grouping does not change deterministic process order.
4. Add project roundtrip for group metadata.
5. Run: `cargo test -p geist-graph` and `cargo test -p geist-project`.

### Task 10: Move app fixed slot constants toward descriptors

**Objective:** Reduce the gap between clean-room device specs and the current fixed rack in `studio.rs`.

**Files:**
- Modify: `app/geist-daw/src/studio.rs`
- Modify: `crates/geist-ui/src/model.rs`
- Modify: native device descriptor surfaces as needed.
- Test: UI/app snapshot and rack tests.

**Steps:**
1. Identify fixed slot constants in `studio.rs` (`SLOT_OSC`, `SLOT_LFO`, etc.).
2. Add a descriptor-driven construction path for rack slots while preserving current default rack.
3. Keep the mirror diff silent on frame one.
4. Add tests for descriptor-to-rack model conversion and no-grow command traffic if existing test harness supports it.
5. Run app/UI targeted tests.

## Validation baseline

Run after documentation-only spec work:

```bash
git status --short
find docs -name '*clean-room-spec.md' -o -name 'modular_rack_spec.md'
```

Run before any code implementation claim:

```bash
cargo test -p geist-core
cargo test -p geist-graph
cargo test -p geist-automation
cargo test -p geist-timeline
cargo test -p geist-project
cargo test -p geist-synth
cargo test -p geist-ui
cargo test --workspace
```

Use narrower commands first while iterating; use full workspace before handoff.

## Risks and open questions

- Public Serum 2 docs are less exhaustive than the other manuals. Do not overfit or claim exact parity.
- VCV's flexible voltage patching conflicts with Geist's current strict port type checks. Solve with explicit policy/context rather than making every graph edge permissive.
- Audio warping is large DSP work. Store metadata first; implement original stretch modes later.
- Modulators can explode UI and engine complexity. Implement identity/base/depth semantics first, then add source families.
- Scene tempo/time-signature launch affects transport and timeline conversion. Add data shape before behavior.
- Project schema migration must accompany any persistent field added to `geist-project` or app session files.
