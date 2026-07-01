---
name: geist-dsp-and-plugins
description: "Load when implementing or reviewing `geist-dsp`, first-party synth/fx/modular internal devices, oscillator/filter/envelope/LFO/effect engines, parameter definitions, voice allocation, or DSP benchmarks."
---

<!--
Author: Jeff
Date: 2026-06-30
Description: DSP and internal device implementation guide
Notes: Use for geist-dsp, geist-synth, geist-fx, and geist-modular
-->

# Geist DSP And Internal Devices

## Responsibility

DSP code is pure math over slices and small state structs. Native device code wraps DSP engines as internal DAW nodes only.

## DSP rules

- No I/O.
- No heap allocation in process methods.
- No host/plugin assumptions in pure DSP modules.
- State reset is explicit and deterministic.
- Parameters are normalized at boundaries, not scattered through DSP loops.
- SIMD is feature-gated and correctness-matched against scalar implementations.

## Native device layering

Each first-party device keeps two layers:
- `engine/`: pure DSP and parameter math.
- `daw_node.rs`: internal `AudioNode` wrapper.

Do not add VST, CLAP, AU, LV2, or standalone plugin-export wrappers for first-party devices. External plugin hosting is VST3-only and belongs in `crates/geist-vst-host`.

## Implementation order

1. Add math utilities and scalar baselines.
2. Implement oscillator primitives with tests.
3. Implement filters with stability tests.
4. Implement envelopes and LFOs.
5. Implement effects one at a time.
6. Build `crates/geist-synth` voice, pool, osc stack, filter stack, and mod matrix.
7. Build `crates/geist-fx` modules.
8. Build `crates/geist-modular` utility nodes.
9. Add benchmarks only after correctness tests exist.

## Test expectations

- Golden-value tests for simple deterministic DSP.
- Stability tests for filters under extreme params.
- Silence-in/silence-out where expected.
- Parameter clamping tests.
- Voice stealing tests.
- No-allocation tests for process methods when tooling exists.
