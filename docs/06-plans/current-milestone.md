<!--
Author: Jeff
Date: 2026-08-09
Description: The single active Spectre milestone
Notes: Exactly one milestone is active; the roadmap owns ordering
-->

# Current Milestone — R3 Live Shell

- **Status:** accepted
- **Last verified:** 2026-08-09
- **Scope:** qualified audio backend, callback bridge over the existing compiled plan, MIDI timestamps, health telemetry
- **Decision authority:** Jeff
- **Upstream sources:** `rebuild-roadmap.md`, RT-001..003, `../03-architecture/graph-compilation.md`
- **Downstream dependents:** `../status/NEXT.md`, implementation slices
- **Supersedes:** the R2 offline-graph milestone, exited 2026-08-09
- **Open decisions:** audio backend selection, RT-002 structure and overflow policy, RT-003 acceptance
- **Known gaps:** no audio backend, callback bridge, or MIDI ingress exists; every R3 exit row is open

## R0/R1 exit record

R0/R1 exited 2026-07-17 with all exit evidence passing: formatting, strict Clippy, and the full workspace test suite; tempo/time/transport/event/ID/persistence property coverage; the deterministic offline harness; and traceability matching implementation. CORE-004's atomic-save design is accepted with implementation at R4; CORE-001's reorder/migration evidence is explicitly gated to R4/R5.

## R2 exit record

R2 exited 2026-08-09 against its stated roadmap exit gate. `spectre-graph` implements the GRAPH-001 split: an app-thread `EditableGraph` with stereo-bus semantics and single-feed inputs, and an immutable `CompiledPlan` built through validated compilation (ancestor inclusion, missing-input rejection, implicit-cycle diagnostics, factory layout verification, deterministic ordering, preallocated planar buffers). Plan execution is allocation-free and lock-free with take/restore buffer handoff.

The offline Pulse → Gain → Saturator fixture renders through the compiled plan, bit-identical to a hand-wired chain, with the silence, impulse, allocation, and deterministic-hash gates passing on that path. The renderer-neutral `DeviceParameterSnapshot` DTO carries app-model parameter edits into offline rendering, and `render_app_snapshot` rejects empty, partial, duplicate, aliased, unknown, mismatched, non-finite, out-of-range, and non-canonical input before constructing processors. Device Focus Drill-In added stable app-thread device selection and fail-closed Build → Shape focus without graph mutation, persistence, or live audio.

GRAPH-002 did **not** close at R2 and is not claimed. Only implicit-cycle rejection exists; explicit one-quantum-priced feedback edges remain unimplemented and stay gated at decision row 7 before the R11 modular surface. R2 exited on its four render gates, not on full GRAPH-002 satisfaction.

## Current evidence

None yet for R3. This milestone opens with no implementation. The inherited asset R3 builds on is the immutable `CompiledPlan` with its measured allocation-free steady-state execution; the callback bridge must drive that plan rather than introduce a second render path.

## Requirements in scope

RT-001 (no allocation, deallocation, blocking locks, I/O, logging, or panics across the callback boundary), RT-002 (bounded wait-free control↔render communication with defined overflow policy and off-thread reclamation), RT-003 (denormal flush and NaN/Inf containment with node isolation, emitting silence rather than noise).

RT-001..003 are currently traced as workspace policy with no implementation. RT-003 remains `proposed` in the requirements ledger and needs acceptance during this milestone.

## Open decisions at intake

These block R3 exit, not R3 start:

1. **Audio backend selection.** No decision-gates row covers it. Choosing between a cross-platform crate and direct per-platform backends needs its own row with reversibility and a license note, consistent with decision 1 (macOS and Linux co-first-class).
2. **RT-002 concrete structure and overflow policy.** The requirement names bounded wait-free structures and off-thread reclamation, but no structure is chosen and the overflow behavior must be defined rather than discovered.
3. **RT-003 acceptance.** The requirement is still proposed; its per-node-type injection fixtures define what containment means.

## Non-goals

VST3 hosting, recording, arrangement editing, piano roll, automation and modulation, latency compensation beyond health telemetry, mixer routing, and UI build-out beyond what makes the callback bridge observable.

## Exit evidence

- Audio backend selected, recorded as a decision-gates row, and qualified on at least one macOS and one Linux device.
- The callback bridge drives the existing `CompiledPlan` with no second render path.
- Allocation and lock guards wrap every callback-reachable path in CI (RT-001).
- Control↔render transfer uses a bounded wait-free structure with tested overflow policy and off-thread reclamation (RT-002).
- Denormal flush and NaN/Inf containment pass per-node-type injection fixtures, isolating the offending node and emitting silence (RT-003).
- A device lifecycle drill covers start, stop, device change, sample-rate change, and device loss without panic or audio-thread blocking.
- MIDI events carry timestamps through to the plan with defined ordering at equal timestamps.
- Health telemetry reports xruns and callback headroom off the audio thread.
- `cargo fmt`, strict Clippy, and the full workspace suite stay green.
- Traceability and status match the implementation.
