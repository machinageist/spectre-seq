<!--
Author: Jeff
Date: 2026-07-11
Description: Atomic clean-room observations from Phase Plant docs, VCV Rack voltage standards, and Serum 2 public sources
Notes: Observed public behavior only; version limitations recorded per source
-->

# Synth and Modular References — Atomic Observations

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** Phase Plant official docs (unversioned), VCV Rack voltage standards (Rack 2 manual), Serum 2 public support material
- **Decision authority:** Jeff
- **Upstream sources:** `SRC-PHASE-PLANT-LIVE-DOCS`; `SRC-VCV-RACK2-VOLTAGE-STANDARDS`; `SRC-SERUM2-SUPPORT-CPU-ARTICLE`
- **Downstream dependents:** flagship-synth spec, modular-routing spec, DSP contracts, requirements ledger
- **Supersedes:** none; legacy `docs/specs/geist-modular-synth-spec.md` remains historical evidence
- **Superseded by:** none
- **Open decisions:** none
- **Known gaps:** Phase Plant docs expose no product/manual version; Serum 2 still has no complete public manual (gap unchanged); extraction used assisted summarization with transcription risk until spot-verified

## Phase Plant (official Kilohearts docs, unversioned, accessed 2026-07-11)

### Architecture

- `OBS-PP-ARCH-001`: The generator area holds up to 32 modules of five generator types (analog oscillator, granular, noise, sample player, wavetable) plus in-stack effects (distortion, filter, non-linear filter), utility modules (mix, aux), and required output modules (envelope output, curve output). At least one output module is required to produce sound.
- `OBS-PP-ARCH-002`: Generators auto-route top-to-bottom, each mixing onto the signal from above; a group header breaks automatic routing, and audio between groups requires explicit audio-rate modulation. The aux module adds one sample of latency.
- `OBS-PP-ARCH-003`: Audio-rate modulation between generator modules is assigned by a hover-plus-drag gesture and its semantics depend on target parameter: Phase → classic FM (no tuning drift), Harmonic/Shift → linear FM, Level → ring modulation, Pitch → exponential FM (documented as hard to control).
- `OBS-PP-ARCH-004`: Shared generator parameters: Level 0–200%, Pitch (semitones/cents interval), Harmonic multiplier (×0 disables keytracking), Shift in Hz (can push frequency below zero), Phase Offset in degrees, Phase Randomness in ±degrees applied per note (per unison voice when unison is on).
- `OBS-PP-ARCH-005`: Wavetables are 256 frames × 2048 samples (524,288 samples total; WAV/FLAC import) with a Frame selector and a pre-modulation bandlimit filter for heavy phase modulation.
- `OBS-PP-ARCH-006`: The sample player offers loop modes infinite / sustain / ping-pong / reverse with loop start, length, and boundary crossfade; root tuning is set by eye against zoomed waveform cycles or by ear.
- `OBS-PP-ARCH-007`: The granular generator exposes grain length (keytracked optionally), grain envelope (attack/decay/curvature), spawn modes free-rate (Hz) / synced-rate / density, root pitch, phase alignment, warm start (spawn all grains at voice start), per-grain randomization of position/timing/pitch/level/pan/reverse-probability, and a chord picker with range and up/down/up-down/random patterns.

### Unison and voices

- `OBS-PP-UNI-001`: Two unison scopes exist: per-oscillator unison (cheap, internal duplicates mixed at the oscillator) and global unison (parallel voices across the whole generator stack plus polyphonic effect lanes).
- `OBS-PP-UNI-002`: Unison modes: Hard (same phase), Smooth (random phases), Synthetic (evenly spread phases), Frequency Stack, Pitch Stack, Shepard (octave stack with distance-based volume), Chords — with mode-dependent fourth parameter (Bias/Range/Center/Balance) plus common voice count, detune, stereo spread, and blend.
- `OBS-PP-UNI-003`: Voice settings include glide (time, Always/Legato), polyphony limit (1 = mono with Retrig/Legato trigger modes), master pitch, and bend range; bend range is saved with the project, not the preset. Voice recycling steals the oldest and quietest voice.

### Effect lanes

- `OBS-PP-FX-001`: Three post-generator effect lanes route serially or in parallel with per-lane poly toggle (poly must be contiguous from the left), mute/solo, gain, mix 0–100%, and send-to (next lane or master).

## VCV Rack (Rack 2 voltage standards, accessed 2026-07-11)

- `OBS-VCV-VOLT-001`: Audio outputs are nominally ±5 V (10 Vpp) pre-bandlimiting; the virtual power rails are ±12 V with protection-diode headroom to about ±11.7 V; modules should avoid hard clipping and let downstream stages attenuate.
- `OBS-VCV-VOLT-002`: CV conventions: unipolar 0–10 V, bipolar ±5 V. dBFS full scale is defined as ±10 V; 0 dBVU = −18 dBFS.
- `OBS-VCV-VOLT-003`: Triggers are 10 V, 1 ms pulses; gates hold 10 V while active; Schmitt-trigger thresholds are ≤0.1 V low and 1–2 V high; modules with CLOCK and RESET inputs should ignore CLOCK for 1 ms after RESET.
- `OBS-VCV-VOLT-004`: Pitch is 1 V/oct (f = f₀·2^V) with baselines C4 = 261.6256 Hz for oscillators and 2 Hz (120 BPM) for LFOs/clocks.
- `OBS-VCV-VOLT-005`: Polyphony is up to 16 channels per cable; monophonic inputs are copied to all engines; missing channels on under-provisioned polyphonic inputs read 0 V.
- `OBS-VCV-VOLT-006`: Each cable imposes a one-sample delay; modules should output 0 on NaN/infinity detection.

## Serum 2 (public support material; complete manual still unavailable publicly)

- `OBS-SR2-CPU-001` (support article 51): Official guidance treats per-oscillator unison beyond ~3–7 voices as unnecessary; high unison counts "increase CPU usage significantly" and introduce phasing; the recommended alternative is one chorus on the FX bus, applied once rather than per voice.
- `OBS-SR2-KB-001` (support category): The public Serum 2 KB comprises six articles (authorization, upgrade FAQ, CPU guidelines, preset previews, sample-to-wavetable conversion, automatic sample root-note mapping). No complete public user manual is exposed; the source gap recorded in the ledger stands.

## Cross-cutting patterns worth carrying into requirements work

1. Three distinct modular signal contracts are now on record: VCV's voltage-typed mono cables with 16-channel polyphony, Bitwig's semantically typed always-stereo 4×-rate signals, and Phase Plant's implicit top-down mix bus with audio-rate modulation as the only cross-group routing. Geist's modular contract should be an explicit original decision benchmarked against all three.
2. Unison is consistently two-scoped (per-oscillator cheap vs. per-stack expensive) and both Kilohearts and Xfer treat per-voice effects as the cost center — favoring shared post-FX over per-voice duplication.
3. Voice lifetime/stealing policies are documented product behavior (Phase Plant: oldest+quietest; Bitwig: distributed lifetime predicate) and belong in Geist's device-model contract, not in implementation folklore.
4. One-sample-latency edges (VCV cables, Phase Plant aux) are the standard way modular systems price feedback; both document it plainly.
5. Pitch representations differ meaningfully (1 V/oct vs. ±0.1/octave around middle C vs. semitone offsets) — Geist must pick one canonical internal pitch type with explicit conversions.
