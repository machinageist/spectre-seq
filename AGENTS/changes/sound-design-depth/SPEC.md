<!--
Author: Jeff
Date: 2026-07-03
Description: Change-level spec for PRODUCTION_PLAN.md P7 — sound-design depth
Notes: Behavioral targets are docs/specs/serum-2-clean-room-spec.md and
       docs/specs/geist-modular-synth-spec.md (clean-room, from public manuals);
       subsumes .hermes improvement-plan Tasks 2 and 3 for geist-synth
-->

# Sound-Design Depth — Change Spec

## Desired outcome

Geist Synth stops being a 2-osc scaffold and becomes a spec-grounded hybrid
instrument: bounded source slots (wavetable A/B/C + sub + noise), an
identity-based modulation matrix feeding a fixed pool of envelopes, LFOs, and
macros, dual filters with series/parallel routing, a first-slice wavetable
editor, scale-lock + arpeggiator MIDI devices on the track path, and a
compressor/limiter for the mix bus — the "reasons to leave Ableton" layer.

## Scope

- Source slots in `geist-synth` (serum spec §3.1/§10.1, phase spec §2.1–§2.3):
  fixed-capacity slot array, per-slot enable/tuning/level/pan/unison,
  disabled-slot silence; wavetable engine stays the first family.
- Sub oscillator (serum §3.7: shape set, octave, direct-out option) and noise
  source (serum §3.8: looped/one-shot, pitch, stereo) as cheap dedicated slots.
- Identity-based mod matrix (serum §6.2, bitwig/phase modulation sections):
  stable `ModSource`/`ModTarget` IDs replacing opaque indices, signed depth,
  uni/bipolar, per-route enable, bounded route array, allocation-free resolve.
- Modulator pools (serum §6.3–§6.5): envelopes 1–4 (DAHDSR), LFO pool with
  shape/rate/sync/retrig-envelope modes, 8 macros as sources and targets.
- Dual filter with series/parallel routing and per-source filter targeting
  (serum §4.1–§4.2); reuse `geist-dsp` filter models, original algorithms only.
- Wavetable editor first slice (P7.1): frame list, waveform display, import
  single-cycle WAV, normalize/DC-remove/smooth ops, smooth WT-position
  interpolation; operates on `geist-synth` wavetable types.
- `geist-midi-tools` crate (P7.2): scale lock + arpeggiator (serum §7.3
  behavior subset: rate/octave-range/direction/gate) as internal devices with
  sample-accurate event offsets.
- Compressor/limiter DSP in `geist-dsp` + FX-chain wiring (P7.3).
- Session/project persistence for every new field; old saves load unchanged.

## Non-goals (this change)

- No sample, multisample/SFZ, granular, or spectral source engines — the
  asset-pipeline work (serum §3.3–§3.6, phase §3.3–§3.4) is its own change.
- No warp/transform slots beyond the existing FM path (serum §3.2 dual warp
  deferred); no chaos LFOs, LFO drawing tools, or LFOs 7–10.
- No FX bus racks/splitters (serum §5); the existing per-track chain stands.
- No preset browser/database work (serum §8).
- No vendor UI art, names-as-branding, presets, wavetables, file formats, or
  inferred algorithms — clean-room rules per both specs' non-goal sections.

## Acceptance criteria

1. Default patch after the refactor is audibly/test-identical to today's
   two-osc behavior (existing synth tests stay green unmodified where pinned).
2. A slot-disabled source renders exact silence; enabling sub + noise on top
   of A/B changes output; per-slot tuning is deterministic. Unit-tested.
3. Mod routes address sources/targets by stable IDs; multiple routes to one
   target sum before clamp at the parameter boundary; bipolar/unipolar and
   missing-ID-skip behavior unit-tested.
4. Envelope pool (4) and LFO pool sizes are fixed at voice-alloc time; no
   allocation in render (allocator guard stays green).
5. Filter 2 with series and parallel modes passes routing tests; a source
   routed direct-to-main bypasses both filters.
6. Wavetable editor: import a single-cycle WAV into a frame, run
   normalize/DC/smooth, hear the table play at the set WT position. GUI QA.
7. Scale lock quantizes out-of-scale input notes; arp emits sample-accurate
   offsets within the block. Event-offset unit tests.
8. Compressor: threshold/ratio/attack/release with GR readback, limiter mode;
   DSP unit tests over step/sine fixtures.
9. Full workspace tests + clippy green; realtime-rules checklist for every
   new `EngineCommand`; all new state survives save/load round-trip.

## Constraints

- Follow `docs/realtime_rules.md`; fixed capacities, no hot-path allocation.
- Small validated slices, one commit each, per working-context law.
- PRODUCTION_PLAN.md sequencing: P6 lands before the modulation-overlay parts
  of this change touch the graph surface; synth-internal matrix work is
  independent and may proceed.
- Cite the relevant spec section in each task/commit; no vendor cloning.
