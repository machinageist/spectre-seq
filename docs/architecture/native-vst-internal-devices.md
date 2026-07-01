<!--
Author: Jeff
Date: 2026-06-30
Description: Documentation trail for the native-device/VST-only architecture pivot
Notes: Use this to maintain coherence across sessions and models
-->

# Native Devices + VST-Only Host Boundary

## Why this exists

Jeff provided a stronger product/architecture constraint: this DAW is a native Rust DAW with custom DSP, internal devices, and optional third-party VST hosting. First-party synths, effects, MIDI tools, modulators, and utility nodes must not be built as VST, CLAP, AU, LV2, or standalone plugin binaries.

This document records where the repo is after applying that constraint so future agents do not reintroduce plugin-suite architecture by accident.

## Repository findings on 2026-06-30

- The codebase already had a substantial Rust DAW vertical slice: core, graph, backend, DSP, timeline, automation, project IO, UI, app runtime, synth, effects, modular nodes, and VST host scaffolding.
- `cargo test --workspace` passed before the architecture move.
- `docs/adr/001-clap-over-vst.md` already chose VST3-only hosting, but `INITIAL_PLAN.md`, `PROPOSED_FILE_TREE.md`, project-local skills, and the physical `plugins/` tree still carried old CLAP/LV2/first-party plugin-suite assumptions.
- Native device crates lived under `plugins/` and included dormant CLAP export source files. That contradicted the new hard rule even though those files were not in the module tree.
- `geist-clap-host` and `geist-lv2-host` were still active workspace members via the `crates/*` glob.

## Changes made in this slice

- Moved internal native device crates:
  - `plugins/geist-synth` -> `crates/geist-synth`
  - `plugins/geist-fx` -> `crates/geist-fx`
  - `plugins/geist-modular` -> `crates/geist-modular`
- Removed dormant first-party CLAP export files:
  - `crates/geist-synth/src/clap_plugin.rs`
  - `crates/geist-fx/src/delay/clap_plugin.rs`
  - `crates/geist-fx/src/reverb/clap_plugin.rs`
  - `crates/geist-modular/src/clap_plugins.rs`
- Updated dependency paths in the moved crates and app manifest.
- Removed `plugins/*` from the workspace.
- Excluded `crates/geist-clap-host` and `crates/geist-lv2-host` from active workspace builds.
- Rewrote plan/tree docs to reflect native internal devices and VST3-only external hosting.
- Updated the DSP project skill language away from first-party plugin exports.

## Current active crate roles

- `geist-synth`: internal synth device crate.
- `geist-fx`: internal effects device crate.
- `geist-modular`: internal modular utility device crate.
- `geist-vst-host`: VST3 host adapter crate.
- `geist-clap-host` / `geist-lv2-host`: excluded historical scaffolds. Do not build new features there.

## Do not regress

Do not:

- Add `plugins/*` back to the workspace.
- Add CLAP/VST/AU/LV2 export modules to first-party device crates.
- Rename internal devices as "plugins" in architecture docs unless specifically discussing third-party hosted VSTs.
- Let `geist-vst-host` types leak into native synth/fx/modular crates.
- Put plugin scanning or project saving anywhere near the audio callback.

## Next coherent slices

1. Keep sweeping any remaining user-facing native-device labels to "device chain/rack" where they do not refer to third-party VSTs.
2. Delete or archive `crates/geist-clap-host` and `crates/geist-lv2-host` after Jeff confirms no historical code needs to be kept.
3. Implement `AudioDevice` for concrete native devices and the VST wrapper as descriptors/states are formalized.
4. Expand `geist-vst-host` scan/cache/state integration without letting VST shape native devices.

Completed follow-up:

- `geist-core` now has explicit time newtypes: `SampleTime`, `BeatTime`, `Seconds`, `PpqTick`, `BarBeat`.
- `geist-core` now has `DeviceId`, `DeviceKind`, `DeviceDescriptor`, and `DeviceState`.
- `geist-graph` now exposes an internal `AudioDevice` surface above the realtime `AudioNode` trait.

## Validation trail

- Pre-change: `cargo test --workspace` passed.
- Post-change validation included for the completed architecture/core/graph follow-ups:
  - `cargo check --workspace`
  - `cargo test --workspace`
  - `cargo test -p geist-core`
  - `cargo test -p geist-graph`
  - targeted `cargo check -p geist-vst-host -p geist-synth -p geist-fx -p geist-daw`

When iterating on this boundary, rerun the touched crate tests plus `cargo check --workspace`.
