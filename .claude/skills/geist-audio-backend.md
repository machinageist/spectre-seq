---
name: geist-audio-backend
description: "Load when implementing or reviewing `crates/geist-audio-backend`, device enumeration, stream negotiation, cpal/PipeWire/JACK wrappers, xrun reporting, callback wiring, or backend tests."
---

<!--
Author: Jeff
Date: 2026-05-27
Description: Audio backend implementation guide
Notes: Use for cpal, PipeWire, JACK, stream config, device selection, and callback integration
-->

# Geist Audio Backend

## Responsibility

`geist-audio-backend` hides platform I/O behind a small trait and drives the compiled graph from the audio callback.

## Boundaries

- Backend code owns device discovery and stream lifecycle.
- Engine code owns graph/timeline semantics.
- Callback code receives already-compiled state and bounded event inputs.
- Platform-specific errors are mapped into explicit backend errors.

## Implementation order

1. Define `AudioBackend` trait.
2. Define `AudioDevice`, `DeviceInfo`, `StreamConfig`, `AudioStream`, `XrunCounter`.
3. Implement fake/null backend for deterministic tests.
4. Implement cpal backend as default.
5. Gate PipeWire and JACK behind features.
6. Add stream start/stop lifecycle tests.
7. Add roundtrip/xrun validation harness where feasible.

## Callback rules

- Do not discover devices in callback.
- Do not resize buffers in callback.
- Do not log xruns from callback directly; increment counters and report elsewhere.
- Convert host buffer layout once at boundary.
- Fail stream creation before callback starts, not during processing.

## Validation

- `cargo check -p geist-audio-backend`.
- Unit tests for config validation and fake backend lifecycle.
- Manual device test remains opt-in; CI must not require physical audio devices.
