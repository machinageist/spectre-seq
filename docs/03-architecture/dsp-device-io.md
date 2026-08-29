<!--
Author: Jeff
Date: 2026-07-12
Description: Version-one DSP device input, output, event, and realtime contract
Notes: This contract precedes instrument/effect UI and remains narrower than graph routing
-->

# DSP Device I/O Contract

- **Status:** accepted for R2/R4 implementation
- **Last verified:** 2026-08-09
- **Scope:** native source, instrument, and audio-effect process boundaries
- **Decision authority:** Jeff
- **Upstream sources:** realtime rules, GRAPH-001..002, decision gate 6
- **Downstream dependents:** `spectre-dsp`, offline graph, live callback, device UI
- **Supersedes:** removed prototype device APIs
- **Superseded by:** none
- **Open decisions:** control-rate modulation and multichannel layouts after the stereo vertical slice
- **Known gaps:** live MIDI ingress, latency compensation, and live callback publication are later contracts

## Processing representation

- Audio samples are planar `f32` buffers.
- One process call covers `1..=max_frames` frames at one finite positive sample rate.
- Channel slices and event storage are allocated by the compiled plan before callback execution.
- Process methods borrow all input, output, and event storage. They do not own or resize buffers.
- Every output channel has exactly `frames` writable samples and MUST be fully written on every call.
- Input and output buffers do not alias in the safe API. In-place processing is not part of v1.
- Non-finite parameter or signal values are contained at the device boundary and MUST NOT propagate.

## Device layouts

Each device declares one immutable `DeviceIo` layout before plan compilation:

- source: zero audio inputs, one stereo audio output;
- instrument: zero audio inputs, one bounded note-event input, one stereo audio output;
- insert effect: one stereo audio input and one stereo audio output;
- sidechain effect: one stereo main input, one stereo sidechain input, and one stereo output;
- analyzer/sink and event output buses are deferred until required by a vertical slice.

Bus identity is semantic rather than positional at the editable-graph layer. The compiled v1 process call flattens buses into stable planar channel order. A layout mismatch fails before processing; it is not repaired in the callback.

## Events

- V1 event input carries note-on, note-off, and bounded all-notes-off.
- Note events carry a nonzero note ID, MIDI channel `0..=15`, note `0..=127`, and normalized finite velocity so overlapping notes can be terminated precisely.
- Each event has a frame offset in `[0, frames)` and a stable sequence number.
- Events MUST arrive sorted by `(frame_offset, semantic_rank, sequence)`, with note-off/all-notes-off before note-on at the same frame.
- Event slices are bounded and borrowed; a device never queues or allocates callback events.
- One device event bus admits at most 1,024 events per quantum. Upstream overflow is rejected before the current process seam and becomes fixed telemetry when the compiled graph lands.
- A note number is MIDI-compatible `0..=127`; velocity is normalized finite `f32` in `[0, 1]`.
- Events outside the block, duplicated in deterministic order, over capacity, or out of order fail block validation before device processing.

## Parameters

- Every user-visible device and parameter has a stable nonzero project-instance `ObjectId` allocated from the project-scoped `IdGen`; all such IDs are unique within the project. A parameter's instance ID is distinct from its static `DeviceParameterKey`, which identifies one parameter slot on a device type.
- R4 controls update plain values on the app thread; compiled snapshots contain callback-ready `f32` values.
- V1 applies one parameter value per render quantum. Sample-accurate automation arrives with R9.
- Parameter smoothing is owned by the device when discontinuities can click or destabilize processing.
- UI labels, units, ranges, defaults, and disabled reasons derive from backend descriptors; UI code MUST NOT redefine DSP ranges.

### `f32` numeric policy

- `DspParameter` performs the core linear mapping in `f64` and publishes `f32`. Both directions are finite and monotone for finite inputs, and normalized `0`/`1` map bit-exactly to the plain minimum/maximum and back.
- Normalized-to-plain publication rounds the exact linear result to nearest `f32`; its absolute plain-domain error is therefore at most one half of the enclosing `f32` quantization interval. This is the authoritative plain-domain bound and does not claim nonexistent extra precision.
- For the R2 fixture evidence distribution—normalized `0`, the next value above `0`, quarter anchors, the next value below `1`, `1`, and 10,000 values from the fixed `0x1234_5678` LCG stream per descriptor—the normalized → plain → normalized round trip is bounded by `NORMALIZED_ROUND_TRIP_MAX_ULPS = 8192`. Tests apply the benchmark's descriptor order first, then cover every other native descriptor. The blind benchmark's worst observed fixture value was 3,005 ULP for `saturator.drive`.
- The offset `saturator.drive` range `[1, 24]` is ill-conditioned in normalized ULPs near zero: rounding the plain `f32` near `1` and then subtracting that offset magnifies normalized ULP distance. Normalized inputs sufficiently close to an endpoint may quantize to that exact plain endpoint; they map back to exact normalized `0` or `1` and are governed by the plain-domain quantization bound rather than a universal normalized-ULP claim. Nextafter endpoint probes pin this behavior explicitly.

### Signed zero and subnormals

- Finite in-range signed zero is preserved bit-exactly. When a descriptor minimum is canonical `+0`, a candidate below it, including the negative minimum subnormal, clamps to that canonical `+0`.
- Finite in-range parameter subnormals are preserved bit-exactly through the app setter and `DeviceParameterSnapshot` constructor/getter. Non-finite candidates map to the descriptor default.
- This parameter-control policy does not define DSP signal-path denormal handling. Signal FTZ/DAZ or equivalent remains deferred to RT-003.

## Realtime contract

`AudioProcessor` requires `Send`. A compiled plan is built on the app thread and moved to the audio thread, so a device that cannot cross threads could never run in a callback. This was added 2026-08-09 at R3 slice 5, when wiring the bridge into a real backend callback failed to compile: `CompiledPlan` stores `Box<dyn AudioProcessor>` and was therefore not `Send`. All four v1 devices satisfy the bound without change. A compile-time assertion pins `CompiledPlan`, the bridge, and both control halves as `Send`.

`process` MUST NOT allocate, lock, block, perform I/O, format strings, log, serialize, inspect UI state, or panic for valid compiled-plan input. Recoverable layout/event errors are detected by validation outside the hot loop. DSP arithmetic uses `f32`; phase, coefficient, or time accumulators MAY use `f64` where documented.

## Runtime parameter seam

Accepted 2026-08-09 as decision 22, design-only. **Implemented 2026-08-24 in R4 slice 2**, following the CORE-004 precedent where an API is accepted at one milestone and implemented at a later one. R3 did not implement this, and R3's exit did not depend on it.

The method that landed, on `AudioProcessor` in `crates/spectre-dsp/src/io.rs`:

```rust
fn set_parameter(
    &mut self,
    key: DeviceParameterKey,
    value: f32,
) -> Result<(), ParameterError>;
```

It is required rather than defaulted, so a device added later must answer the seam instead of silently ignoring every edit made to it. `CompiledPlan::set_parameter` addresses one node's processor; `spectre_audio::route::ParameterRoutes` maps an RT-002 target to a node and key with a binary search over a frozen boxed slice; `RenderBridge` applies the drained lane once per block before `process`. Failures are counted into `parameters_pending` and never logged on the audio thread.

> **D-R3 closed 2026-08-28 by implementing this document rather than amending it.** `Gain` had no target and no ramp from R4-2 until then, so the smoothing clause below was false of the code. R4-2 gave two reasons for leaving it that way and both turned out to be testable and false. It does **not** change rendered output for identical inputs: `Gain::new` sets target equal to the current value, so a render with no parameter change takes the delta-is-zero path and is bit-identical to the unsmoothed device — the live/offline hash equality in `crates/spectre-audio/tests/bridge_plan.rs` still passes untouched. It needs **no numeric bound of its own**: the ramp spans exactly one block, which is the boundary this document already says parameters are applied on, so the length is `context.frames()` rather than a new constant requiring a rationale row.

The problem this closes: v1 devices take their parameters as constructor arguments, so a compiled plan's processors are fixed once built. The RT-002 parameter lane can deliver a latest-wins value per `(device, parameter)` target to the render thread, but no processor can accept it. Recompiling a plan per change is rejected because compilation allocates and cannot run on the audio thread.

Accepted shape:

- `AudioProcessor` gains one callback-safe method that applies a single already-validated parameter value to a live processor, addressed by the same static parameter key the descriptors and offline snapshot DTO already use.
- The method inherits the full realtime contract above: no allocation, no locks, no I/O, no formatting, no panics. It performs assignment and at most bounded arithmetic.
- Validation and clamping happen on the app thread, against the existing `DspParameter` descriptors, before the value enters the RT-002 lane. The render thread applies values, it does not police them; a value that reaches a processor is already canonical.
- Unknown keys are refused as a recoverable error, not a panic, keeping the fail-closed posture the offline snapshot path already uses.
- Applying parameters is a separate step from `process`, executed once per block before it, so a block sees one coherent parameter set rather than values changing mid-render.
- Smoothing stays the device's concern. `Gain` already smooths; devices whose parameters would click MUST smooth internally rather than requiring the caller to ramp.

What this deliberately does not decide: sample-accurate parameter automation within a block. R3 and R4 apply parameters at block boundaries. Sample-accurate automation is PROD-002 at R9 and will extend this seam rather than replace it.

## Initial native devices

The first original vertical slice contains:

1. `ToneSource`: deterministic stereo sine source for graph and render fixtures.
2. `PulseInstrument`: monophonic note-driven oscillator with level and waveform controls.
3. `Gain`: stereo linear gain with click-resistant smoothing.
4. `Saturator`: stereo soft saturation with drive and wet/dry mix.

The alpha's own voice, added at R4 slice 6 under decision 15's deliberately-small scope:

5. `Filament`: monophonic phase-warped voice behind a linear amplitude contour, with lean, rise, fall, and level controls.
6. `Gloam`: stereo one-pole damping whose corner opens with the signal's own level, with damp, depth, and track controls.

Both are original clean-room DSP. Their numeric bounds are DEV-001..DEV-013 in the requirements ledger, each with its own rationale and none derived from a reference product.

These devices prove the contract. They do not define the eventual flagship instrument or effect catalog.

## Acceptance checklist

- Layout and block validation reject mismatched channels, lengths, rates, and event order.
- Silence remains finite silence through every effect.
- Identical initial state and input produce bit-identical offline output. **Block size is part of that state, not part of the input** — clarified 2026-08-28. RT-003 containment is scoped to the quantum, so a *contaminated* render differs across block sizes; `a_contaminated_render_is_not_identical_across_block_sizes` proves it and `a_contaminated_render_is_bit_identical_to_itself_at_the_same_block_size` proves the clause itself still holds, contamination included. This resolves the item R4-8 routed as a narrowing: the clause never covered two different quanta, so nothing is narrowed. A bounce must therefore state its block size rather than choose one silently.
- Instrument note events begin and end on their declared frame offsets.
- Device output remains finite under extreme valid parameter values.
- Process loops allocate no storage and use no locks or I/O.
- Backend descriptors exist before any device UI is implemented.
- UI controls consume descriptors and preserve backend defaults and ranges.
