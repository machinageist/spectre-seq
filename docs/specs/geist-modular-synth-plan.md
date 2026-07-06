<!--
Author: Jeff
Date: 2026-07-03
Description: Phased implementation plan for the Geist modular synth specced in geist-modular-synth-spec.md
Notes: Companion to the clean-room spec; each phase is a small validated slice with explicit verification; decision points are Jeff's
-->

# Geist Modular Synth — Implementation Plan

Companion to `geist-modular-synth-spec.md` (the "spec"). Spec section numbers are referenced as `§N`. Phases are labeled S0–S11 (synth slices) to avoid colliding with `PRODUCTION_PLAN.md` milestones P1–P8.

## Ground truth at planning time

- Current internal synth (`crates/geist-synth`): fixed-topology voices — `OscStack` (2-op FM, PolyBLEP/wavetable) → `FilterStack` (SVF) with amp/filter ADSR, `VoicePool` (no-realloc cap), index-based `ModMatrix` resolved per block. This is a classic subtractive/FM synth, not a generator stack.
- `crates/geist-dsp` already provides: polyblep/sine/wavetable/noise oscillators, ADSR/AHDSR/follower envelopes, SVF/biquad/ladder/comb filters, LFO/stepseq, distortion/chorus/phaser/flanger/delay/reverb/eq/saturator FX.
- `crates/geist-modular` provides control-rate utility nodes (logic, math, sample_hold, timing).
- `crates/geist-automation` has curve/evaluator/matrix/route primitives.
- RT contract: allocator-guarded audio callback (no alloc/lock/IO); commands are `Copy` over a bounded rtrb ring.

## Strategy decisions (Jeff to confirm; recommendations marked)

| Decision | Options | Recommendation |
|---|---|---|
| D1 Where the device lives | (a) evolve `geist-synth` in place; (b) new crate, keep current synth shipping | **(b) new crate `geist-stacksynth`** (working name). The generator-stack voice model is architecturally different; evolving in place would destabilize the working M3 synth. Reuse `geist-dsp` primitives from both. |
| D2 Compatibility scope | Behavioral parity per spec vs Geist-flavored subset | **Full behavioral parity for engine semantics** (§2–§10); editors and content (§11–§14) staged, some deferred. Compatibility limits (32 gens / 32 modulators / 3 lanes / 8 macros / 64 automation slots) enforced from day one per §15.2. |
| D3 Effect-lane content | New effect framework vs host existing `FxKind` pool | **Host existing geist-dsp/fx effects in lanes**; the §8.3 Snapin matrix is a long-term catalog, not a phase gate. |
| D4 Wavetable/sample editors | Build per §12–§13 vs defer | **Defer editors** (S10+); import + playback first. Curve editor MVP comes earlier because Curve Output and Curve/Remap modulators depend on it. |

## Phase plan

Each phase lands as one or more commits on a feature branch, workspace-green (`cargo clippy --workspace` + `cargo test --workspace`) before the next begins. Per-phase verification is listed as `verify:`.

### S0 — Patch schema and validation (data only, no DSP)

1. New crate `geist-stacksynth` with `schema` module: `Patch { groups, lanes, modulators, macros, voice_settings, unison }`, stable IDs, explicit ordering arrays, schema version (§1.1, §15.2).
2. Module enums for all §2.4 module kinds with per-kind parameter structs using spec semantics (common Level/Pitch/Harmonic/Shift/Phase block per §2.3).
3. Route types: audio-rate edge (source tap, target param, depth, enabled — §7.3) and control-rate route (source slot, target, amount, curvature, bounds, output-range mode — §10.1).
4. Validator: 32/32/3/8 limits; every generator module inside a group; missing-input warnings for processors at group top; poly-lane prefix rule; cycle detection with Aux-delay exemption; errors name the offending edge (§2.2, §7.3, §8.1).
   - verify: `cargo test -p geist-stacksynth` — schema round-trip (serde), one test per validator rule, limit-boundary tests.

### S1 — Voice graph compile

1. Compiler: ordered modules per group → per-voice render plan (implicit top-to-bottom sum inside a group, group boundary breaks flow — §2.2). Output = flat processing list with fixed buffer slots, no allocation at render time.
2. Aux explicit-input edges get a mandatory one-sample delay node (§4.5); cycle legality is decided here.
3. Render-plan swap follows the existing engine's graph-swap discipline (build off-thread, swap via ring).
   - verify: unit tests — group isolation (signal never crosses groups implicitly), aux latency exactly 1 sample, deterministic compile of a fixture patch.

### S2 — Sound sources MVP (analog, wavetable, noise)

1. Common generator param block DSP: pitch ratio `2^((semi+cents/100)/12)`, harmonic multiply (x0 kills keytracking), shift in Hz (signed, no zero clamp, negative-frequency phase decrement), phase offset + seeded per-note random phase (§2.3).
2. Analog Oscillator: reuse polyblep; add hard sync (phase reset at base-note period) and pulse width (§3.1).
3. Wavetable Oscillator: 256×2048 import profile (WAV/FLAC), frame position param, pre-phase-mod bandlimit filter (§3.5).
4. Noise Generator: colored (slope white→pink→brown landmarks), stepped/smooth keytracked modes, stereo blend, Stable/Random seed modes with deterministic offline bounce (§3.2).
   - verify: `cargo test -p geist-stacksynth -p geist-dsp`; spectral/behavior tests (sync resets, harmonic x0, negative shift runs backward, stable seed reproducibility); bench: one voice of each source under existing bench harness.

### S3 — In-stack processors and utilities

1. Distortion module: 6 type enum, drive/bias/spread/mix (§4.1) — wrap/extend `geist_dsp::fx::distortion`.
2. Filter module: 7 types, cutoff/Q/gain(shelf-peak)/slope 1x-2x (§4.2) — SVF/biquad reuse.
3. Non-Linear Filter: 5 types incl. allpass, drive, Clean + Geist-original color modes (§4.3, gap table §17.2).
4. Mix (level+invert on group signal) and Aux (explicit input + upstream mix, 1-sample delay) (§4.4–4.5).
   - verify: per-module unit tests; a fixture patch chaining osc→dist→filter→mix renders non-silent and matches golden RMS envelope.

### S4 — Output modules and buses

1. Shared output behavior: gain/pan/out-toggle/send-to {lane1..3, master, sideband}; out-off mutes send but keeps module as mod source (§5.1, §18).
2. Envelope Output using geist-dsp ADSR (§5.2).
3. Curve Output: curve playback with two loop handles and Off/Infinite/Sustain/PingPong/Reverse modes, equal-handles hold (§5.3, §11.3) — reuse `geist-automation::curve` for storage.
4. Sideband bus plumbing as a named summing bus (§16).
   - verify: loop-mode unit tests (each mode's boundary behavior), sustain exits on gate release, bus routing tests.

### S5 — Audio-rate modulation

1. Per-sample parameter input taps on Phase/Harmonic/Shift/Level/Pitch of all sources and drive/cutoff of in-stack processors (§7.1–7.2).
2. Edge data → compiled per-voice sample-rate connections; technique matrix behaviors (PM keeps tuning, harmonic-linear FM pitch-invariant, level = ring mod, pitch = exponential FM) (§7.2).
3. Cross-group routing only via explicit edges; Aux delay is the sanctioned cycle breaker (§7.3).
   - verify: FM sideband presence test (FFT bins), ring-mod product-frequency test, tuning-invariance test for PM vs pitch-mod, cycle-rejection test.

### S6 — Control-rate modulation system

1. Modulator lane: 32 slots, per-voice + global state split, output range modes (uni/bi/inverted), modulatable depth (§10.1).
2. Core modulators first: Envelope (with Seamless), LFO (free/synced, phase, shapes), Curve, Random (voice modes, jitter/smooth/chaos), Note/Velocity/Note Gate/Pitch Wheel/MIDI CC (§10.2).
3. Trigger system: gate semantics, Auto/Never/Always/Legato, modulated-trigger override, sensitivity (§10.3).
4. Processors second slice: Scale, Lower/Upper Limit, Remap, Sample & Hold (§10.2).
5. Deferred within S6: Audio Follower, Pitch Tracker, LFO Table, Pressure/MPE (need bus taps / MPE plumbing).
6. Control-rate block size configurable (~64 samples default) (§10.1).
   - verify: per-modulator unit tests incl. trigger-mode matrix; determinism test for seeded Random across offline renders.

### S7 — Voice management and unison

1. Allocator: oldest+quietest recycling with deterministic tie-break; mono retrig/legato; glide always/legato with last-pitch state (§9.1).
2. Oscillator unison: Hard/Smooth/Synthetic phase policies + voices/detune/spread/blend/bias, post-unison mix as the mod-source tap (§9.2–9.3). Analog/Wavetable first; Sample Player when S9 lands.
3. Creative modes second slice: Frequency Stack, Pitch Stack, Shepard (Center), Chords (Geist-original chord list, Balance) (§9.3, §17.2).
4. Global unison: whole-stack duplication incl. poly lanes (§9.2) — land after S8.
5. Master section: master pitch (semis+cents) as global pre-generator pitch transform, master gain after the lane/master sum, bend range stored in project state and excluded from presets (§1.2, §9.1).
   - verify: allocator determinism test, unison phase-policy tests, CPU bench at 16 voices × 8 unison.

### S8 — Effect lanes

1. Three lanes hosting the existing FxKind pool (D3); per-lane mute/solo/gain/mix/send-to right-or-master (§8.1).
2. Poly prefix rule enforced in schema (S0) and engine merge-point placement (§8.1, §16).
3. Per-effect enable/bypass and collapse per §8.2; per-effect preset layer and randomize deferred.
   - verify: routing topology tests (serial, parallel, poly prefix violations rejected), dry-path capture test for lane Mix.

### S9 — Sample-based sources

1. Sample asset pipeline: WAV/AIFF (+FLAC) import, root/offset/loop sidecar metadata (no silent file mutation — §3.3), project-relative refs and missing-asset diagnostics (§15.2).
2. Sample Player: loop modes Infinite/Sustain/PingPong/Reverse, start/length/x-fade, root tuning (§3.3).
3. Granular Generator: grain scheduler (free/synced/density), grain envelope with curvature, randomization set, align-phases, warm start, chord picking (§3.4).
   - verify: loop-boundary click test (x-fade), sustain-release continuation test, density≈`rate·length` concurrency test, RT-safety: no alloc in grain spawn (allocator guard test).

### S10 — Macros, automation, presets

1. Eight macros as renameable control-rate sources with fan-out; rename reflected to host automation (§1.2, §14.3).
2. 64 assignable automation slots for dynamic params (§14.3).
3. Preset payload per §14.2 checklist; Bend Range excluded from presets, stored in project (§1.2); browser reuses the existing geist-ui browser with favorites/search/tags staged.
   - verify: preset round-trip covering every §14.2 bullet; automation-slot assign/clear tests.

### S11 — UI

1. Generator stack panel in the device detail view (add/move/copy modules, groups, missing-input warnings, per-module output scopes driven by engine DSP at a fixed frequency per §2.1) — original design per §15.4.
2. Modulator lane UI + route gesture (source → target with depth drag), route list per target (§10.1).
3. Curve/LFO shape editor MVP: control-point tool, grid/snap, loop selector for curve type (§11.2–11.4). Free-draw/stepped tools second slice.
4. Deferred: wavetable editor (§12), sample editor (§13) — import-only until then.
   - verify: `cargo run -p geist-daw --release` visual QA by Jeff (native egui not screenshotable headless); state round-trip tests for UI-model structs.

## Sequencing and integration

- S0→S5 are strictly ordered (each depends on the previous). S6 and S7 can interleave after S5. S8 after S7.1. S9 anytime after S2. S10–S11 last.
- The device mounts through the same `daw_node`/rack-slot mechanism as `geist-synth` so the studio shell needs no structural change until S11.
- Every phase: `cargo clippy --workspace && cargo test --workspace` green + allocator-guard suite green before commit; RT-critical phases (S1, S5, S7, S9) also run the alloc-guard test under a rendered fixture patch.
- Relative to `PRODUCTION_PLAN.md`: this plan is a work package inside production milestone P7 (sound-design depth, "device growth per founding list"). Start after production P6 lands; its graph-adoption decision is resolved (engine moves onto geist-graph's compiled plan), so the device should ultimately mount as a graph node, with the `daw_node` rack slot as the interim mount. Production P7.1's wavetable editor must operate on the same wavetable type S2 imports so the editor and the oscillator share assets.

## Out of scope (per spec §17)

- Vendor content, preset/binary compatibility, vendor UI cloning, exact vendor DSP algorithms.
- Wavetable/sample editor tool suites until post-S11 product call.
