<!--
Author: Jeff
Date: 2026-06-30
Description: VST3-only host boundary notes for Geist DAW
Notes: External plugin hosting only; native devices are not plugins
-->

# VST Hosting

## Decision

Geist DAW supports third-party VST3 hosting as the only external plugin standard in the active architecture. Native Geist synths, effects, MIDI tools, modulators, and utility nodes are internal devices and are not exported as VST, CLAP, AU, LV2, or standalone plugin binaries.

## Boundary

All VST3-specific code lives in `crates/geist-vst-host`.

Responsibilities:

- Discover `.vst3` bundles in platform-standard locations and user-specified paths.
- Build and refresh a plugin metadata cache off the audio thread.
- Load plugin modules through a narrow dynamic-library/COM boundary.
- Expose plugin classes and parameters as internal device descriptors.
- Wrap plugin instances as internal graph/device nodes.
- Save and restore opaque plugin state blobs through `geist-project`.
- Report latency and bus layouts to the graph compiler.
- Host editor windows on the UI/main thread where supported.

## Realtime rules

VST scanning, filesystem traversal, metadata cache writes, plugin instantiation, editor creation, and project serialization never run in the audio callback. The callback may call a prepared VST process adapter only after the plugin is instantiated, activated, and connected to preallocated audio/event buffers.

## Native-device separation

Native crates (`geist-synth`, `geist-fx`, `geist-modular`, future `geist-midi-tools`) must not depend on `geist-vst-host`. They depend only on internal traits/data structures such as `geist-core`, `geist-graph`, and `geist-dsp`.

## Naming guardrail

Use "plugin" for third-party hosted VSTs only. Use "device", "internal device", "device chain", or "rack" for Geist synths, effects, MIDI tools, modulators, and modular utilities. Historical filenames may keep old names for link stability, but their contents must state this rule.

## Historical formats

`geist-clap-host` and `geist-lv2-host` remain in the repository as excluded historical scaffolds after the 2026-06-30 architecture alignment. They are not active workspace members and should not receive feature work unless Jeff explicitly changes the plugin policy.
