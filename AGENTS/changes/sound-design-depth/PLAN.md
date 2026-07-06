<!--
Author: Jeff
Date: 2026-07-03
Description: Atomic implementation plan for the sound-design-depth change (P7)
Notes: Executes AGENTS/changes/sound-design-depth/SPEC.md against
       docs/specs/serum-2-clean-room-spec.md + docs/specs/geist-modular-synth-spec.md;
       ordered, one commit each, engine substrate before UX before persistence
-->

# Sound-Design Depth — Implementation Plan

Verification shorthand: `T <crate>` = `cargo test -p <crate>`, `W` = full
`cargo test --workspace` + clippy, `R` = launch app and exercise by hand.

## Progress

- **M1.1 DONE** (`aa909c1`): typed `ModSource`/`ModTarget` identities replace
  opaque `usize` indices on `ModRoute`; per-route `enabled` flag; `resolve()`
  takes fixed typed arrays. Engine maps `LfoDestination`→`ModTarget`. `T geist-synth`
  + `T geist-daw` green.
- **M3.7 (engine side) DONE** (`eed9723`): `SynthNode` now surfaces filter A mode,
  filter B (cutoff/res/mode), and series/parallel routing — the `FilterStack`
  DSP already had the two SVFs; only the node facade was single-filter. DSP tests
  prove B darkens in series and parallel keeps more energy than series. `T geist-synth`
  green (45 unit). **TODO M3.8:** wire filter-B + routing through `Patch`/
  `EngineCommand`/`session.rs` persistence + a rack Filter page (app surface,
  uncontested by the modular-rack/stacksynth sessions).
- **Coordination:** parallel sessions own `geist-modular` (rack) and
  `geist-stacksynth` (new crate). This lane stays in `geist-synth` + the app's
  synth patch surface (`engine.rs` Track/Patch, `control.rs` synth commands,
  `session.rs` TrackState, `studio.rs` synth rack slots) to avoid collisions.

## M1 — Modulation identity substrate (engine)

1. **`geist-synth::mod_matrix` identity routes** — `ModSource`/`ModTarget`
   enums with stable discriminants; `ModRoute { source, target, depth, polarity,
   enabled }`; bounded `[ModRoute; MAX_ROUTES]`; `resolve()` keeps the current
   allocation-free sum-then-clamp shape; map existing LFO→cutoff/pitch/FM
   routes onto IDs (serum spec §6.2; subsumes .hermes Task 3). Tests: multi-route
   summing, bipolar/unipolar, disabled/missing skip, clamp at boundary.
   → verify: `T geist-synth`.
2. **Modulator pools** — envelopes 2–4 (reuse the existing envelope type,
   DAHDSR params via `SynthParam` descriptors) and an LFO pool (shape enum
   sine/tri/saw/square/S&H, rate Hz or beat-sync, retrig modes free/retrig/
   envelope; serum §6.3–§6.4) allocated at voice-pool build, ticked at block
   rate like the current single LFO. Tests: pool sizes fixed, retrig behavior,
   sync rate math vs tempo. → verify: `T geist-synth`.
3. **Macros as sources/targets** — 8 macro values on the patch state exposed
   as `ModSource::Macro(n)` and targetable (serum §6.5); wire the existing
   rack macro knobs through. Tests: macro→route→param end-to-end.
   → verify: `T geist-synth`, `T geist-daw`.

## M2 — Source slots (engine)

4. **Slot refactor** — `OscStack` becomes a fixed `[SourceSlot; 3]` + sub +
   noise; `SourceSlot` enum starts with `Disabled` and `Wavetable(WavetableOsc
   state)` preserving A/B behavior bit-for-bit as slots 0/1 (serum §3.1/§10.1,
   phase §2.1; subsumes .hermes Task 2). Per-slot level/pan/coarse/fine/unison.
   Tests: default patch regression, disabled-slot silence, per-slot tuning.
   → verify: `T geist-synth`.
5. **Sub + noise sources** — sub: shape set (sine/tri/square), octave offset,
   optional bypass-filter direct-out (serum §3.7); noise: white/pink via
   `geist-dsp`, looped vs one-shot, pitch/level/pan, stereo width (serum §3.8).
   Tests: sub octave math, noise one-shot termination, direct-out routing flag.
   → verify: `T geist-synth`, `T geist-dsp`.
6. **Slot params + commands** — extend `SynthParam` descriptors for slots
   C/sub/noise; new `EngineCommand`s follow the realtime checklist; rack
   Oscillator slot UI grows slot selector + enable. Mirror diff stays silent
   on frame one. → verify: `T geist-daw`, `T geist-ui`, allocator guard.

## M3 — Dual filter and routing (engine + UX)

7. **Filter 2 + routing modes** — second `FilterStack` instance; series vs
   parallel enum; per-source route target (filter1/filter2/direct) replacing
   the implicit all-through-filter path (serum §4.1–§4.2). Tests: series ==
   f2(f1(x)) within epsilon, parallel sums, direct bypasses both.
   → verify: `T geist-synth`.
8. **Filter UX** — rack Filter slot gains filter-2 page + routing selector;
   persisted; existing single-filter saves load as filter1-series-default.
   → verify: `T geist-daw`, `T geist-ui`, `R`.

## M4 — Wavetable editor first slice (P7.1)

9. **Frame ops in `geist-synth`** — frame list on the wavetable type: add/
   remove/reorder, import single-cycle WAV (resample to table size off-thread),
   normalize / DC-remove / smooth ops as pure fns with tests (serum §3.2 scope,
   original algorithms). → verify: `T geist-synth`.
10. **Editor view** — `views/wavetable_editor.rs`: frame list, waveform
    display, op buttons, import via file dialog off the audio thread; table
    swap through the existing immutable-snapshot path. **[JEFF QA]**
    → verify: `T geist-ui` + `R`.
11. **WT-position interpolation** — smooth frame interpolation for the WT POS
    param (serum §3.2 "smooth interpolation"); modulatable via M1 routes.
    Tests: midpoint blend, position clamp. → verify: `T geist-synth`.

## M5 — MIDI tools (P7.2)

12. **`geist-midi-tools` crate** — scale lock (key/scale table, nearest-note
    quantize) and arpeggiator (rate/octaves/direction up-down-updown-random/
    gate; serum §7.3 subset) as pure event-buffer transforms with
    sample-accurate offsets. Property tests on offset ordering.
    → verify: `T geist-midi-tools`.
13. **Track wiring** — insert as optional per-track MIDI devices ahead of the
    instrument; commands + rack slots + persistence. → verify: `T geist-daw`,
    allocator guard, `R`.

## M6 — Dynamics (P7.3)

14. **`geist-dsp::compressor`** — feed-forward peak/RMS detector, threshold/
    ratio/attack/release/makeup, limiter mode (ratio=inf, lookahead-free first
    pass), GR metering value. Step/sine fixture tests. → verify: `T geist-dsp`.
15. **FX-chain wiring** — new `FxKind::Compressor` through the duplicate-
    capable chain (same pattern as EQ/Saturator, commit `21db198`); mixer GR
    readback optional label. → verify: `T geist-daw`, `T geist-ui`, `R`.

## M7 — Gate

16. **End-to-end gate** — SPEC acceptance 1–9 walked by hand; `W`; headless +
    release smoke gates; HANDOFF iteration log; PRODUCTION_PLAN P7 status
    update; mark .hermes Tasks 2/3 subsumed. → verify: `W` + `R`.

## Relationship to geist-modular-synth-plan.md (Jeff to arbitrate)

The parallel Phase-Plant plan (`docs/specs/geist-modular-synth-plan.md`, D1)
recommends a new `geist-stacksynth` crate so the generator-stack rebuild never
destabilizes the shipping synth. This plan is the other lane: Serum-style
depth ON the shipping `geist-synth`. Both stand if D1(b) is confirmed, with
one coordination rule: M1's `ModSource`/`ModTarget`/route types land in
`geist-core` or `geist-automation` (not synth-private) so stacksynth P6 reuses
them — one modulation identity system, two consumers. If Jeff instead picks
D1(a) (evolve in place), stacksynth P0–P11 absorbs this plan's M1–M3 and this
change shrinks to M4–M6 (editor, midi-tools, dynamics).

## Decisions taken (flag to Jeff if wrong)

- Source slot capacity fixed at 3 primary + sub + noise (serum §1.2 shape)
  rather than Phase-Plant-style unbounded stacks — bounded voice cost, no
  hot-path allocation; phase spec §2 informs slot semantics only.
- Slot families beyond wavetable (sample/multisample/granular/spectral) are a
  separate asset-pipeline change; the slot enum leaves room.
- LFO pool sized 4 (not Serum's 10) until UI demand exists; pool size is a
  compile-time constant so growth is cheap.
- Compressor before limiter-only variant; both live in one module with a mode
  flag (P7.3 "mix-bus table stakes").

## Task→agent map

M1–M2 `geist-audio-dsp-agent`; M3 tasks 7 → audio-dsp, 8 → `geist-ui-project-agent`;
M4 tasks 9/11 → audio-dsp, 10 → ui-project; M5 task 12 → audio-dsp,
13 → ui-project; M6 task 14 → audio-dsp, 15 → ui-project;
M7 → `geist-reviewer` before the milestone commit.
