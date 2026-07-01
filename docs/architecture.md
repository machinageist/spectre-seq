<!--
Author: Jeff
Date: 2026-06-30
Description: High-level architecture for the Rust-first native-device Geist DAW
Notes: Current architecture, not aspiration; VST is a host boundary only
-->

# Architecture

## Summary

Geist DAW is a Rust-first native DAW. It owns the project model, time model, graph, DSP, sequencing, automation/modulation, native devices, and UI/controller flow. Third-party plugins are optional VST3 devices hosted behind `geist-vst-host`; first-party devices are not plugin binaries.

## Layers

1. `geist-core`: stable IDs, config, ports, event types, parameter descriptors, process context, transport snapshots.
2. `geist-graph`: editable routing graph, typed connections, topology, compiled process list, graph swap, graph node trait.
3. `geist-audio-backend`: platform audio I/O and callback bridge. It does not own DAW logic.
4. `geist-dsp`: pure Rust signal-processing primitives. No I/O, no host/plugin assumptions.
5. `geist-timeline`: tempo map, transport, tracks, clips, pattern scheduling, playhead.
6. `geist-automation`: automation curves and modulation matrix resolution.
7. `geist-project`: versioned schema, serialization, migration, asset map, autosave.
8. `geist-synth`, `geist-fx`, `geist-modular`: first-party internal native devices.
9. `geist-vst-host`: VST3 discovery/loading/wrapping boundary.
10. `geist-ui` and `app/geist-daw`: command/snapshot UI and app runtime wiring.

## Realtime contract

The audio thread consumes prepared state. It does not allocate, lock, block on I/O, scan plugins, save projects, call UI, mutate graph topology, or wait on async work. Graph and project mutations happen on the app/controller side and publish bounded commands or immutable compiled state to the render side.

## Native device contract

Native devices are internal DAW modules. They expose graph/device behavior to the DAW directly through Rust traits and app state, not through plugin ABIs. Device code may have pure DSP engines and graph wrappers, but no CLAP/VST/AU/LV2 export layers.

Current internal devices:

- `crates/geist-synth`: flagship synth.
- `crates/geist-fx`: effect devices.
- `crates/geist-modular`: modular utility devices.

## VST contract

VST3 is the only supported external plugin format. `crates/geist-vst-host` adapts VST3 plugins into internal graph/device nodes. No other crate should know VST3 COM details. VST scanning, metadata cache updates, editor-window setup, and plugin instantiation happen off the audio callback.

## Documentation trail

- `INITIAL_PLAN.md`: phase status and next architectural slices.
- `PROPOSED_FILE_TREE.md`: intended workspace layout.
- `docs/architecture/native-vst-internal-devices.md`: details of the 2026-06-30 architecture pivot.
- `HANDOFF.md`: cross-session working-tree status and validation notes.
