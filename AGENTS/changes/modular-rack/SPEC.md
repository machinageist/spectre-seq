<!--
Author: Jeff
Date: 2026-07-03
Description: Change-level spec for the modular rack environment inside Geist
Notes: Behavioral target is docs/modular_rack_spec.md (clean-room, from the VCV manual)
-->

# Modular Rack — Change Spec

## Desired outcome

Geist's Build lens becomes a playable modular environment: the user opens the
node graph, adds generator/processor/utility nodes from the browser, patches
cables by dragging port-to-port, tweaks knobs with the standard modifier
ladder, and hears the result live through the existing engine — with
VCV-compatible signal conventions so patch knowledge transfers.

## Scope

- Signal conventions (voltages, 1 V/oct, gates/triggers, NaN flush) as shared
  helpers adopted by `geist-modular` and any node exposed to the rack.
- Polyphonic channel semantics on modular cables (1–16 channels, broadcast /
  index / zero-fill rule, mono-summing rule).
- Rack-facing node registry: existing `geist-modular` utilities plus the
  minimum playable set (VCO, LFO, envelope, VCA, filter — reusing
  `geist-dsp`/`geist-synth` primitives, not re-implementing).
- Bridge nodes: transport clock, MIDI→CV (v/oct, gate, velocity, retrigger;
  poly allocation modes), rack audio out into the track's device chain.
- Patching UX in `views/node_graph.rs`: drag-to-connect, one-cable inputs,
  fan-out outputs, cable delete, node add via browser, node delete, param
  drag ladder / double-click reset / numeric entry.
- Rack patch persistence inside the project file; module presets later.

This change is the detailed execution of `PRODUCTION_PLAN.md` milestone P6
(modular identity), extended with the VCV-derived signal/poly contract.
P6.1 engine adoption — decided by Jeff 2026-07-03: migrate the live engine
onto `geist-graph`'s compiled process plan + swap; the fixed-track engine
becomes one prebuilt graph. Applies at task 10.

## Non-goals (this change)

- No hard typed-port blocking — decided by Jeff 2026-07-03: any output
  patches to any input (spec §2.1, §14.1); validation is feedback only
  (port colors, warn tint on unusual pairings per P6.3 /
  `ui_interaction_model.md`), the connection is never refused.
- No hosted-plugin modules in the rack.
- No preset browser/factory-preset packaging (follow-up change).
- No panel-graphics system; nodes stay semantic blocks in the current theme.

## Acceptance criteria

1. A patch of [MIDI→CV] → [VCO] → [VCA] ← [envelope ← gate] → [rack out]
   plays polyphonically from the QWERTY/MIDI input path, audible in the
   running app.
2. Signal helpers round-trip the spec numbers: 0 V = C4 = 261.6256 Hz on
   v/oct; gates 10 V; triggers 1 ms; Schmitt 0.1/1.0 V; LFO 0 V = 2 Hz.
3. Channel-count propagation follows the broadcast/index/zero-fill rule and
   is unit-tested for M=1, M≥N, 1<M<N, and the 0-channel unpatched case.
4. All patching interactions work with mouse only; Delete respects focus
   gating like the rest of the app.
5. Rack patches survive project save/load.
6. RT safety: no allocation in the audio callback (existing allocator guard
   stays green); full workspace tests and clippy pass.

## Constraints

- Follow `docs/realtime_rules.md`; Geist rules win over VCV conventions on
  any conflict.
- Small validated slices, one commit per slice, per working-context law.
- Reuse `geist-dsp` primitives; nothing speculative beyond the playable set.
