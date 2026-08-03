---
name: spectre-dsp-and-plugins
description: "Load when implementing or reviewing `spectre-dsp`, first-party synth/fx/modular plugins, oscillator/filter/envelope/LFO/effect engines, parameter definitions, voice allocation, or DSP benchmarks."
---

<!--
Author: Jeff
Date: 2026-05-27
Description: DSP and plugin suite implementation guide
Notes: Use for spectre-dsp, spectre-synth, spectre-fx, and spectre-modular
-->

# Spectre DSP And Plugins

## Responsibility

DSP code is pure math over slices and small state structs. Plugin code wraps DSP engines as DAW nodes and optional CLAP exports.

## DSP rules

- No I/O.
- No heap allocation in process methods.
- No host/plugin assumptions in pure DSP modules.
- State reset is explicit and deterministic.
- Parameters are normalized at boundaries, not scattered through DSP loops.
- SIMD is feature-gated and correctness-matched against scalar implementations.

## Plugin layering

Each first-party plugin keeps three layers:
- `engine/`: pure DSP and parameter math.
- `daw_node.rs`: internal `AudioNode` wrapper.
- `clap_plugin.rs`: standalone CLAP ABI wrapper.

No DSP duplication between DAW node and CLAP wrapper.

## Implementation order

1. Add math utilities and scalar baselines.
2. Implement oscillator primitives with tests.
3. Implement filters with stability tests.
4. Implement envelopes and LFOs.
5. Implement effects one at a time.
6. Build `spectre-synth` voice, pool, osc stack, filter stack, and mod matrix.
7. Build `spectre-fx` modules.
8. Build `spectre-modular` utility nodes.
9. Add benchmarks only after correctness tests exist.

## Test expectations

- Golden-value tests for simple deterministic DSP.
- Stability tests for filters under extreme params.
- Silence-in/silence-out where expected.
- Parameter clamping tests.
- Voice stealing tests.
- No-allocation tests for process methods when tooling exists.
