<!--
Author: Jeff
Date: 2026-07-31
Description: Living dependency roadmap from the current Geist prototype to the product contract.
Notes: Update this roadmap whenever an accepted product decision changes dependencies or scope.
-->

# Geist Product Roadmap

## Planning authority

`docs/product/PRODUCT_VISION.md` defines the product. This roadmap defines dependency order. Detailed work still receives a narrow `docs/changes/<slice>/SPEC.md` and `PLAN.md` before implementation.

`INITIAL_PLAN.md` and `PROPOSED_FILE_TREE.md` remain architectural references. Their completed foundations are retained, but their phase labels and estimates do not override implementation evidence or this roadmap.

## Current implementation audit

### Foundations that are materially useful

- Realtime rules and app-thread/audio-thread separation are explicit.
- The graph has deterministic topology, validated port metadata, preallocated audio buffers, and a zero-allocation block executor.
- Core DSP includes usable oscillator, filter, dynamics/effect, FFT, resampling, and analysis primitives.
- A basic synth voice pool, wavetable oscillator stack, filter stack, and internal modulation matrix exist.
- Utility modular nodes implement useful audio/CV-style math, logic, timing, and sample/hold behavior.
- Basic first-party delay, reverb, chorus, saturation, and EQ graph nodes exist.
- The app can produce audio, schedule notes, show a studio UI, save/load prototype sessions, record input, package media, preserve offline clips, and play audio clips.
- Canonical 960-PPQ musical time, shared nonzero track/clip identity, and a structural arrangement entity shell are implemented and independently reviewed.
- Typed global UI selection and initial arrangement interactions exist.

### Gaps that change roadmap order

1. The graph's typed ports are metadata only for most domains. The executor routes all edges through `f32` sample buffers, while notes and parameters are global slices delivered to every node. Per-port event routing, control buffers, polyphonic lanes, and typed rate conversion are not implemented.
2. Graph `NodeId` is currently a runtime zero-capable handle. Durable device, node, parameter, port, scene, asset, note, mapping, and automation-target identity is incomplete.
3. The app's session and project models still use floating beats, raw integers, vector asset indices, and parallel legacy structures. Canonical project authority has not migrated.
4. The structural `ClipEntity` models arrangement placement only. It does not model launcher slots, scenes, source/instance boundaries, MPE expression, tuning, warp state, or durable automation targets.
5. There is no launcher scheduler, scene model, launch quantization model, per-track arrangement override, or Back to Arrangement state.
6. The current synth is a useful two-oscillator prototype, not the accepted bounded flexible-generator/lane architecture.
7. The current modular crate is a utility-node collection, not a complete patchable polyphonic rack instrument.
8. The first-party effect bundle is an integration foundation, not yet a professional suite. Several plugin wrappers and parameter surfaces remain scaffolds.
9. Audio capture exists, but full Off/Auto/In monitoring, compensated routing, launcher-slot recording, overdub policy, and feedback prevention are incomplete.
10. Several UI widgets, plugin-host paths, backend paths, packaging tools, and architecture documents remain pseudocode scaffolds.
11. At least four arrangement representations remain live across UI, app, audio engine, legacy timeline, and additive canonical arrangement paths.
12. The running fixed three-track engine bypasses the compiled graph, automation, modular, and plugin-host authorities.
13. Queue saturation is commonly ignored while app mirrors advance, so UI/project and audio state can silently diverge.
14. Replacing the final audio-asset `Arc` may deallocate media on the callback; asset retirement lacks an off-thread reclamation path.
15. Playback scheduling is block-accurate in key app paths, and input capture/output playback use unsynchronized streams without a clock-domain or compensation contract.
16. Persistence advertises more state than the app round-trips and still uses zero IDs, vector asset indices, and source-coordinate semantics that conflict with canonical contracts.

## Corrected dependency sequence

### Milestone 0 — Product contracts and migration map

Status: in progress.

- Maintain the living product vision and roadmap.
- Inventory every legacy/canonical state owner and define which model becomes authoritative.
- Define compatibility windows and deletion criteria for duplicate float-beat, sample-placement, asset-index, and runtime-ID paths.
- Reconcile ADR 002 with the implemented mutable-executor queue swap, replace scaffold ADR 003 with the actual project-package decision, and update architecture/realtime/plugin documentation to distinguish implemented behavior from planned contracts.
- Revise `PROPOSED_FILE_TREE.md` only after authority boundaries are accepted; do not let it silently override implementation evidence.
- Establish a real CI matrix and explicit stable/nightly policy before release claims.
- Split broad work into independently reviewable change specs.

Exit gate: every next milestone has explicit owners, identity domains, persistence boundaries, and realtime projection boundaries.

### Milestone 1 — Canonical domain vocabulary and durable identity

- Define durable nonzero identities for assets, notes, scenes, devices/nodes, parameters or parameter keys, routes, controller mappings, automation targets, and other persisted entities.
- Distinguish durable project identity from compiled/runtime handles where their lifecycles differ.
- Put normalized values, curve/interpolation vocabulary, typed immutable asset references, musical/sample coordinate types, and rejection errors in dependency-low domain crates.
- Remove the proposed timeline-to-automation dependency for shared curve vocabulary.
- Define non-cloneable domain owners that enforce allocation, imported-ID observation, uniqueness, exhaustion, and atomic batch reservation.

Exit gate: persisted references never depend on vector position or ephemeral graph handles, and higher-level crates do not own shared domain vocabulary.

### Milestone 2 — Canonical `ProjectDocument` authority and transactions

- Define one app-thread-owned document containing tracks/order, arrangement, launcher, graph/device/routing state, assets, conductor/transport state, automation/modulation, mappings, and persistence metadata.
- Name one final arrangement aggregate; do not leave legacy `Timeline`, additive `Arrangement`, UI `TimelineModel`, and audio-thread arrangement as peer authorities.
- Make UI emit typed intents/commands and persistence serialize the document rather than renderer-facing mirrors.
- Define atomic load/build validation across IDs, assets, clips, graph targets, and unresolved placeholders before mutating the live project.
- Define one project-level history for all durable edits, dirty-state ownership, atomic import/recording commits, and exact accepted/rejected results; transient transport and live performance remain outside history.
- Preserve unresolved assets, devices, plugins, modules, mappings, and targets losslessly.
- Version and migrate project persistence instead of promising saveability through legacy vector indices.
- Define a project-directory package with canonical manifest, managed media/recordings, renders/freezes, autosaves/backups, and disposable cache; add Collect All and Save, atomic manual save, migration, dirty-state, and startup recovery workflows.
- Allow explicit pre-v1 prototype breaks where necessary, with diagnostics and fixtures; define stable format v1 as the start of supported forward migration and prohibit silent partial loads or downgrade writes.

Exit gate: one document owns durable DAW truth; UI, persistence, and audio publication are projections with parity tests.

### Milestone 3 — Typed realtime render and publication contract

- Replace metadata-only graph typing with compiled routes for audio, CV, gate, note/event, MIDI, parameter, and meter domains.
- Route events and parameter changes only to compiled destinations with sample offsets and stable note identity.
- Add control-rate buffers, bounded timestamped event queues, deterministic fan-in, explicit rate adapters, and polyphonic-lane split/merge/reduce/broadcast.
- Support mono and stereo audio buses first with explicit upmix/downmix adapters; keep bus descriptors extensible to declared multichannel layouts without implicit conversion.
- Require explicit visible domain-appropriate delay/feedback elements for every cycle; remove hidden automatic one-block cycle conversion from the production contract and include declared latency in compensation.
- Publish immutable/movable render generations with sequence/version acknowledgement; queue saturation must not silently advance app mirrors.
- Retire graph executors, plugin instances, and replaced audio assets off the callback.
- Define sample-accurate transport/tempo slices and scheduler boundaries rather than block-start approximation.
- Keep compilation, allocation, deallocation, locks, logging, and I/O off the callback; enforce this with allocator guards, overload tests, graph-swap stress, and callback benchmarks.
- Establish reproducible performance fixtures around the 48 kHz/128-frame mainstream eight-core baseline and 64-frame live stress mode, including graph size, active tracks, voice count, device/modulation density, recording, graph swaps, and plugin teardown.

Exit gate: audio, note, control, MIDI, and polyphonic routes execute independently; rejected/overloaded publication reconciles explicitly; callback work is bounded and allocation/deallocation-free.

### Milestone 4 — Hybrid track and device-chain authority

- Define the graph-backed hybrid track aggregate and typed pipeline stages.
- Make Audio, Instrument, Group, Return, and other track choices templates over that aggregate.
- Define device insertion, removal, reorder, bypass, nesting, sends, returns, sidechains, and explicit graph expansion.
- Prove ordered chain and graph views round-trip to one routing truth.
- Add durable parameter metadata, normalized/native mapping, modulation-rate declarations, remote-control pages, and macro targets.

Exit gate: one project track can route note devices into an instrument, audio effects, sends, meters, and an expanded graph with deterministic save/load.

### Milestone 5 — Canonical clip, scene, and content aggregates

This milestone replaces the currently paused broad B2 slice with smaller prerequisites.

- Separate arrangement placement from launcher-slot ownership while retaining stable `ClipId` semantics.
- Add stable `SceneId`, ordered scenes, track/scene slots, empty-slot semantics, and clip/scene launch settings.
- Define whether common content records are needed without introducing implicit shared mutable editing. Arrangement capture remains independent.
- Add non-destructive audio region state over stable managed `AssetId`: source range, gain, fades, reverse state, warp markers, tempo interpretation, and stretch mode.
- Add structured MIDI notes with stable `NoteId`, checked musical coordinates, normalized velocity, MPE/per-note expression, and tuning-compatible pitch semantics.
- Add automation only after durable target identity exists. Preserve unresolved target descriptors and curves.
- Model arrangement automation in project time and clip-local automation in clip-relative time as separate owners; defer their evaluation precedence with realtime modulation to a dedicated parameter-control specification.
- Add a project conductor aggregate for tempo automation, time-signature markers, metronome/count-in state, and reusable groove templates.
- Make aggregate-owned indexes enforce clip, note, slot, scene, asset, and target invariants atomically.

Exit gate: audio, MIDI, launcher, arrangement, and unresolved content can be removed/restored exactly and persisted without legacy indices or runtime IDs.

### Milestone 6 — Typed reversible editing

- Prove command-history failure semantics first.
- Implement create, delete, move, cross-track move, launcher-slot move/copy, split, duplicate, right resize, left trim, fades, note edits, expression edits, and automation edits in narrow slices.
- Allocate all identities transactionally at the enforcing aggregate.
- Preserve exact state through undo/redo; rejected commands do not disturb history or allocation.
- Conduct dedicated gesture interviews before routing UI interactions.

Exit gate: canonical editing is complete enough for one end-to-end MIDI and audio production loop without mutating legacy authority.

### Milestone 7 — Arrangement and launcher scheduler

- Implement global and per-clip/per-scene launch quantization, Immediate mode, clip and scene launch, track stop, and deterministic transition scheduling.
- Implement per-track launcher override and Back to Arrangement.
- Record launcher performance into independent arrangement entities.
- Define looping, start offsets, legato/retrigger, follow actions, stop buttons, and automation interaction in explicit sub-specs.
- Publish bounded immutable playback snapshots or commands to the engine.

Exit gate: a multitrack launcher performance can be recorded into, replayed from, and returned to the arrangement sample-accurately.

### Milestone 8 — Live input and recording

- Add explicit input routing and Off/Auto/In monitoring.
- Define unified input/output clocking or explicit resampling and drift correction for separate devices/streams.
- Add latency measurement/compensation, direct/software-monitor policy, feedback prevention, arm/exclusive-arm policy, count-in, punch, and metronome routing.
- Record audio and MIDI into arrangement and launcher slots.
- Define overdub semantics per clip kind.
- Preserve native source sample rate and sample-domain source coordinates; report xruns/dropouts and recover partial takes atomically.
- Keep project-managed media portable with exact hash/size recovery.
- Add take lanes and comping only after the first complete recording workflow.

Exit gate: live audio and MIDI can pass through devices, monitor safely, and record into either production surface with compensated placement.

### Milestone 9 — Core editing UI and production workflow

- Complete arrangement, launcher, audio editor, piano roll, expression editor, automation editor, mixer, browser, inspector, device chain, and graph expansion surfaces.
- Make launcher and arrangement composable tiled panes that can be focused independently or shown together.
- Preserve one typed global selection authority and command-based canonical mutation.
- Disable or visibly explain aspirational actions until their commands and acknowledged state exist; UI controls must not advertise false transport, routing, recording, or scene behavior.
- Add keyboard/controller workflows, MIDI learn, remote pages, macros, templates, presets, search, tagging, and recoverable diagnostics.
- Add non-destructive simple warp playback before advanced spectral algorithms.

Exit gate: the core DAW loop is usable without debug/demo models or direct legacy mutation.

### Milestone 10 — Professional first-party suite and supporting instruments

- Define one first-party device contract covering parameters, state/versioning, bypass, wet/dry, presets, latency/tail, buses/sidechains, channel policy, tempo sync, smoothing, meters, quality modes, and realtime bounds.
- Finish stable parameter surfaces and preset support for existing effects.
- Deliver utility, EQ, compressor, gate/expander, limiter, saturation/clipping, delay, reverb, chorus, flanger, phaser, and analysis devices.
- Deliver initial first-class MIDI devices: arpeggiator, chord, scale, transpose, velocity, note length, note echo, probability/randomization, expression mapping, filtering/routing, and monitor.
- Add a sampler and drum rack/drum instrument before selecting later FM/additive, granular, or other supporting instruments.
- Add advanced creative, spectral, multiband, pitch, mastering, and restoration tiers separately.

Exit gate: a complete production and mix can be built with supported first-party devices alone.

### Milestone 11 — Flagship hybrid synthesizer

- Replace the fixed prototype architecture with bounded add/reorder generator and signal lanes.
- Add wavetable, sample/granular, virtual-analog, noise, and utility generators.
- Add serial lane processing, parallel mixing, sends, cross-lane routing, per-voice/global modulation, macros, MPE, tuning, preset migration, and professional UI.
- Reuse shared Geist DSP and device protocols rather than forking private infrastructure.

Exit gate: the instrument supports deep modern electronic sound design while remaining faster to operate than the free-form rack.

### Milestone 12 — Flagship modular instrument

- Add durable rack/module/port/cable identity, module registry, patch persistence, reusable subpatches, and module discovery.
- Build on the typed multi-rate graph and polyphonic-lane contract.
- Let each rack patch declare stable typed external ports so it can occupy instrument, audio-effect, note/event-device, or modulation roles.
- Add audio, CV, gate, note/event, parameter, utility, generator, processor, sequencing, logic, and I/O modules.
- Add explicit voice-domain adapters, feedback/latency indication, macros, scopes, debugging, and a production-quality patch editor.

Exit gate: complex polyphonic patches save/load exactly and run within bounded realtime budgets.

### Milestone 13 — Interoperability, rendering, and release engineering

- Complete VST3 hosting before broadening formats, following accepted ADR 001; require state, presets, automation, latency, note expression, GUI embedding, missing-plugin preservation, diagnostics, and crash recovery.
- Add plugin scanning, isolation/recovery policy, latency compensation, automation, state, preset, and missing-plugin handling.
- Add offline render, stems, freeze, flatten, bounce, resample, dither, loudness targets, and export verification.
- Complete backend, packaging, benchmarks, crash recovery, autosave, compatibility migration, accessibility, and release gates.

## Immediate work order

1. Finish independent review of the product contract and roadmap.
2. Specify the final app-thread `ProjectDocument`, arrangement authority, transaction boundary, and acknowledged realtime publication protocol.
3. Extract canonical low-level vocabulary and durable identity in dependency-safe slices.
4. Specify and implement the typed multi-rate realtime graph/publication path in narrow slices.
5. Replace the paused canonical clip B2 unit with dependency-safe content, scene/launcher, warp, MPE/tuning, and durable-target sub-specifications.
6. Return to canonical clip commands only when the correct aggregate can enforce and persist every affected identity atomically.

No UI gesture routing, broad persistence migration, flagship-synth rewrite, or modular-rack implementation begins without its dedicated interview and change specification.
