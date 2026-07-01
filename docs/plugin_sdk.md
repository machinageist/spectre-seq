<!--
Author: Jeff
Date: 2026-06-30
Description: Native Geist device/node development guide
Notes: Historical filename retained; this is not a plugin-export SDK
-->

# Native Device Development Guide

## Scope

This document describes how to build native Geist devices and graph nodes. It is not a guide for exporting first-party devices as plugins. Native synths, effects, MIDI tools, modulators, and modular utilities are first-class internal DAW devices.

## Layering

A native device should keep concerns separated:

1. Pure engine/DSP state: deterministic math over slices/events; no UI, filesystem, host ABI, or dynamic allocation in process methods.
2. Internal graph/device wrapper: adapts the engine to `geist-graph::AudioNode` or the future richer `AudioDevice` abstraction.
3. App/project state: serializable descriptors, parameters, presets, and automation/modulation mappings.
4. UI/editor state: command/snapshot model only; no direct audio-thread mutation.

There is no VST/CLAP/AU/LV2 export layer for first-party devices.

## Current native device crates

- `crates/geist-synth`: flagship synth.
- `crates/geist-fx`: internal effects devices.
- `crates/geist-modular`: internal utility/modulation/routing devices.

## Process requirements

- `prepare` allocates or sizes owned state outside the callback.
- `process` uses borrowed buffers/events and bounded loops.
- Parameters are smoothed/clamped at the device boundary or in explicit DSP helpers.
- Device state is serializable through project/device state types, not plugin state blobs.
- Latency is reported explicitly when a device introduces delay.

## Testing requirements

Add tests beside behavior:

- Parameter clamping and smoothing.
- Silence-in/silence-out where expected.
- Deterministic oscillator/event output.
- Filter/effect stability and finite output.
- Sample-accurate MIDI/event offsets.
- No capacity growth in realtime paths where existing helpers can observe it.
