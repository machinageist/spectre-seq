<!--
Author: Jeff
Date: 2026-06-30
Description: Current implementation plan for the Rust-first native DAW architecture
Notes: Supersedes CLAP/LV2/plugin-suite language; internal devices are not plugin binaries
-->

# Rust-First Native DAW Implementation Plan

## Product posture

Working title remains `geist-daw` until Jeff chooses a product name. The product is a Rust-first DAW for electronic music production, live performance, modular routing, sound design, and generative MIDI composition.

The DAW owns its audio graph, DSP, sequencing, project model, automation/modulation, native devices, and UI/controller state. Third-party plugin support is VST3 hosting only. First-party synths, effects, MIDI tools, modulators, and utility nodes are internal DAW devices and must never be exported as VST, CLAP, AU, LV2, or standalone plugin binaries.

## Non-negotiable architecture

- Native devices live in workspace crates under `crates/`, not `plugins/`.
- `geist-synth`, `geist-fx`, and `geist-modular` are internal device crates.
- `geist-vst-host` is the only active external plugin-format crate.
- `geist-clap-host` and `geist-lv2-host` are excluded from the workspace and remain historical shelved scaffolds only until deleted.
- VST3 devices are wrapped as internal graph nodes/device implementations. Native devices do not depend on VST abstractions.
- Audio callback code consumes precompiled state and bounded queues only: no heap allocation, blocking locks, file/network I/O, logging, UI calls, plugin scanning, dynamic graph mutation, or panics across the boundary.

## Workspace layout

```text
Cargo.toml
app/geist-daw/              # app shell, controller, runtime wiring
crates/geist-core/          # IDs, time, ports, params, events, process context
crates/geist-audio-backend/ # audio I/O trait and cpal callback bridge
crates/geist-graph/         # editable graph, topology, compiled process plan, swap
crates/geist-dsp/           # pure DSP primitives and tests
crates/geist-timeline/      # transport, tempo map, clips, tracks, scheduling
crates/geist-automation/    # automation lanes and modulation resolution
crates/geist-project/       # schema, serialization, assets, autosave
crates/geist-config/        # workflow profiles, templates, keybindings
crates/geist-ui/            # UI state, commands, renderer/view/widget models
crates/geist-synth/         # flagship internal synth device
crates/geist-fx/            # internal native audio effects devices
crates/geist-modular/       # internal utility/modular routing devices
crates/geist-vst-host/      # VST3-only host adapter boundary
docs/                       # architecture, ADRs, plan trail, validation notes
```

## Dependency policy

Allowed dependencies are isolated by layer: native OS audio/windowing, GPU/UI backend, serialization, math/FFT utilities, file dialogs, platform bindings, VST3 hosting bindings, and testing utilities. External DAW engines, synth/effects engines, modular synthesis engines, plugin-wrapper frameworks for first-party devices, and UI frameworks that own realtime architecture are out of scope unless Jeff explicitly approves them.

## Internal device model

The graph-facing abstraction is `geist-graph::AudioNode` today. The long-term device abstraction should generalize this into a richer internal `AudioDevice` surface with prepare/process/parameter descriptors/latency/state/load-state. Native devices and VST wrappers both adapt into that internal surface, but only `geist-vst-host` knows VST3 details.

Native device crates:

- `geist-synth`: polyphonic wavetable/subtractive instrument, voice allocator, wavetable oscillators, envelopes, filters, parameter surface, graph node.
- `geist-fx`: utility/gain-style app path plus delay, reverb, chorus, saturator, EQ, distortion/phaser/flanger DSP integration as internal effects.
- `geist-modular`: internal utility nodes for CV/control/gate-style routing math.
- Future `geist-midi-tools`: internal MIDI processors such as scale lock, arpeggiator, chord/probability/generative tools.

## VST hosting boundary

`geist-vst-host` owns plugin discovery, bundle resolution, module loading, host app callbacks, class descriptors, future scan cache, instantiation, state blobs, parameter mapping, processing adapter, latency reporting, and editor-window integration. Other crates may depend on the common internal device/graph traits, not on VST3 COM bindings or lifecycle types.

VST scanning and cache updates run off the audio thread. VST process calls may run on the audio thread only after instances are prepared and wrapped in a graph/device adapter that obeys the same callback contract as native devices.

## Phase 0 — repository inspection and architecture alignment

Status: complete for this slice.

Findings:

- The workspace already builds and has substantial native Rust implementations across core, graph, audio backend, DSP, timeline, automation, project, UI, app, synth, effects, modular utilities, and VST host scaffolding.
- The old plan still described CLAP/LV2 hosting and first-party plugin exports even though ADR 001 had already chosen VST3-only hosting.
- Native device crates physically lived under `plugins/`, with dormant CLAP export files. That contradicted the new hard rule.
- `cargo test --workspace` was green before this architecture slice.

Actions:

- Move native device crates from `plugins/` to `crates/`.
- Remove dormant first-party CLAP export source files.
- Exclude CLAP/LV2 host crates from active workspace builds.
- Rewrite this plan and the proposed file tree around VST3-only external hosting and internal native devices.
- Add/refresh documentation trail in `HANDOFF.md` and `docs/architecture/native-vst-internal-devices.md`.

## Phase 1 — workspace and core types

Status: implemented in the existing codebase and retained.

Implemented surface includes workspace membership, core IDs, ports, config, process context, MIDI/note/parameter events, parameter ranges, transport snapshots, beat/sample conversions, project schema, track/clip models, serialization, and tests.

Completed in the 2026-06-30 continuation slice:

- Added explicit public time newtypes in `crates/geist-core/src/time.rs`: `SampleTime`, `BeatTime`, `Seconds`, `PpqTick`, and `BarBeat`.
- Exported the time types through `geist-core::prelude`.
- Added unit tests for sample/seconds, beat/sample, PPQ, bar/beat/tick, and invalid conversion inputs.
- Added internal device model primitives in `crates/geist-core/src/devices.rs`: `DeviceKind`, `DeviceDescriptor`, and `DeviceState`.
- Added `DeviceId` to `crates/geist-core/src/ids.rs` and exported the device model through `geist-core::prelude`.
- Added tests proving native vs hosted origin stays hidden behind the common descriptor/state envelope.
- Added `geist-graph::node::AudioDevice` as the common internal device surface above `AudioNode`, with descriptor, parameter, latency, state, and load-state hooks.
- Re-exported `AudioDevice` through `geist-graph::prelude` and tested the default latency/state contract.

Next refinement:

- Implement `AudioDevice` for concrete native devices and the VST wrapper as their descriptors/states are formalized.

## Phase 2 — audio engine skeleton

Status: implemented through app/audio-backend/graph seams.

Implemented surface includes audio backend trait, cpal backend, block bridge, stream config, xrun counter, process context, transport snapshots, event buffers, silence/rolling render behavior in app tests, and realtime command queues.

Next refinements:

- Formalize `crates/geist-audio-engine/` only if app-level engine code needs extraction from `app/geist-daw`.
- Add stronger debug-only audio-thread guardrails for allocation/logging/locks where feasible.

## Phase 3 — graph engine

Status: implemented and tested.

Implemented surface includes editable graph, node and port IDs, typed port descriptors, connection validation, topological ordering, feedback detection, one-block delay policy, compiled process plan, executor, graph swap, and graph tests.

Completed in the 2026-06-30 continuation slice:

- Added the common `AudioDevice` trait while keeping `AudioNode` as the minimal realtime process trait.

Next refinements:

- Extend port taxonomy toward MIDI/events, control scalar/vector, gate, trigger, pitch, clock, transport, sidechain, and MPE expression without hard-coding stereo assumptions.
- Fold latency accounting into compilation.

## Phase 4 — sequencing

Status: implemented for current vertical slice.

Implemented surface includes tempo map, transport, playhead, tracks, MIDI clips/patterns, clip placements, half-open sample windows, arrangement scheduling, session launch behavior in app tests, and undo/redo command primitives.

Next refinements:

- Add explicit scene/follow-action models.
- Expand arrangement regions and automation/modulation lanes in the UI-facing model.
- Add MIDI tool crate and sample-accurate MIDI transformation buffers.

## Phase 5 — first native synth

Status: implemented as an internal device crate.

Implemented surface includes `crates/geist-synth`, voice allocator, oscillator stack, wavetable/subtractive voice path, filter stack, ADSR/AHDSR support through DSP primitives, MIDI note handling, parameter macros, unison/FM/pitch controls, and offline/sample-accurate tests.

Hard rule: `geist-synth` is an internal DAW instrument only. It has no CLAP/VST/AU/LV2 export module.

## Phase 6 — first native effects

Status: implemented as internal device crates/app chain.

Implemented surface includes `crates/geist-fx`, delay, reverb, chorus, saturator, EQ nodes, DSP distortion/phaser/flanger primitives, app-level FX chain processing, bypass behavior, duplicate instance addressing, finite/silence tests, and project/session persistence for current chain state.

Hard rule: native effects are internal DAW devices only. They are not plugin binaries.

## Phase 7 — minimal app UI

Status: implemented beyond the original placeholder level.

Implemented surface includes eframe/egui shell, transport controls, arrangement/session/piano roll/mixer/browser/device rack surfaces, workflow profiles, UI commands, widgets, project save/load seams, and app tests for interaction models.

Next refinements:

- Keep device-chain language reserved for native/internal devices and plugin language reserved for third-party hosted VSTs.
- Keep UI as command/snapshot driven; no direct mutation of audio-thread state.

## Phase 8 — VST host boundary

Status: active VST3 scaffolding exists; headless-safe tests pass.

Implemented surface includes `crates/geist-vst-host`, VST3 bundle paths, scanner, descriptors, host app identity, module loading failure paths, instance error surfaces, and a `VstPluginNode` boundary shell.

Next refinements:

- Add scan/cache schema.
- Add plugin state storage integration with `geist-project` opaque blobs.
- Add parameter mapping and latency reporting.
- Validate against real `.vst3` binaries outside headless CI.

## Phase 9 — modular routing UI

Status: partially implemented as UI graph view and internal modular utility nodes.

Implemented surface includes node graph view models, port/cable geometry helpers, validation-oriented graph model, and internal `geist-modular` nodes.

Next refinements:

- Connect graph edits to project command model.
- Surface validation messages in UI.
- Add advanced patching panel without making cable spaghetti the default view.

## First vertical slice status

Current repo satisfies the first meaningful vertical slice at code/test level:

- Workspace builds and tests.
- App crate exists and launches through eframe/cpal code paths.
- Project/session save and load are covered by tests.
- Tracks, clips, transport, MIDI scheduling, synth rendering, native effects, and graph data are implemented and tested.
- VST hosting remains optional and is not required for the first slice.

## Risk register

1. Existing uncommitted B5 UI/app work predates this architecture slice; do not overwrite it casually.
2. `geist-clap-host` and `geist-lv2-host` still exist as excluded historical crates; delete in a separate cleanup once Jeff confirms no archaeology is needed.
3. Historical filenames such as `docs/plugin_sdk.md` and `docs/adr/001-clap-over-vst.md` remain for archaeology/link stability; their content must state the current VST3-only, internal-device policy.
4. VST3 process/editor integration cannot be fully proven headless; it needs real plugin fixtures and OS-window QA.
5. Audio-thread no-allocation enforcement is tested indirectly today; stronger guardrails are still needed.
6. Time newtypes are present; tempo/meter edge-case expansion remains open.
7. Moving native device crates changes file paths; downstream scripts/docs may need path updates.

## Validation plan

- `cargo test --workspace` after every cross-crate slice.
- `cargo check --workspace` after workspace membership/path edits.
- Targeted tests:
  - `cargo test -p geist-core`
  - `cargo test -p geist-graph`
  - `cargo test -p geist-dsp`
  - `cargo test -p geist-synth`
  - `cargo test -p geist-fx`
  - `cargo test -p geist-timeline`
  - `cargo test -p geist-project`
  - `cargo test -p geist-vst-host`
- `cargo clippy --workspace` before committing.
- Manual app smoke: `cargo run -p geist-daw --release` when a GUI/audio device is available.

## Files changed for this architecture slice

- `Cargo.toml`
- `app/geist-daw/Cargo.toml`
- `crates/geist-synth/**` moved from `plugins/geist-synth/**`
- `crates/geist-fx/**` moved from `plugins/geist-fx/**`
- `crates/geist-modular/**` moved from `plugins/geist-modular/**`
- Removed first-party CLAP export files from moved native device crates
- `INITIAL_PLAN.md`
- `PROPOSED_FILE_TREE.md`
- `docs/architecture.md`
- `docs/architecture/native-vst-internal-devices.md`
- `docs/vst_hosting.md`
- `docs/plugin_sdk.md`
- `.claude/skills/geist-dsp-and-plugins.md`
- `HANDOFF.md`
