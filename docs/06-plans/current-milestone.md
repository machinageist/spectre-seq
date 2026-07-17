<!--
Author: Jeff
Date: 2026-07-17
Description: The single active Geist DAW milestone
Notes: Exactly one milestone is active; the roadmap owns ordering
-->

# Current Milestone — R2 Offline Graph

- **Status:** accepted
- **Last verified:** 2026-07-17
- **Scope:** editable graph → validated compiled plan → deterministic offline render
- **Decision authority:** Jeff
- **Upstream sources:** `rebuild-roadmap.md`, GRAPH-001..002, DSP device I/O contract
- **Downstream dependents:** `../status/NEXT.md`, implementation slices
- **Supersedes:** the R0/R1 foundation milestone, exited 2026-07-17
- **Open decisions:** input-bus summing semantics and explicit feedback pricing at their intake
- **Known gaps:** app parameter-snapshot publication; fixture migration and the four render gates closed 2026-07-17

## R0/R1 exit record

R0/R1 exited 2026-07-17 with all exit evidence passing: formatting, strict Clippy, and the full workspace test suite; tempo/time/transport/event/ID/persistence property coverage; the deterministic offline harness; and traceability matching implementation. CORE-004's atomic-save design is accepted with implementation at R4; CORE-001's reorder/migration evidence is explicitly gated to R4/R5.

## Current evidence

`geist-graph` implements the GRAPH-001 split: an app-thread `EditableGraph` with stereo-bus semantics and single-feed inputs, and an immutable `CompiledPlan` built through validated compilation (ancestor inclusion, missing-input rejection, implicit-cycle diagnostics, factory layout verification, deterministic ordering, preallocated planar buffers). Plan execution is allocation-free and lock-free with take/restore buffer handoff. Seven behavioral tests cover editing, compilation, and execution.

The offline Pulse → Gain → Saturator fixture renders through the compiled plan, bit-identical to a hand-wired chain, with the silence, impulse, allocation, and deterministic-hash gates passing on that path.

## Requirements in scope

GRAPH-001 (implemented; app-path integration evidence outstanding), GRAPH-002 intake, RT-001 as workspace policy.

## Non-goals

Live audio I/O, callback bridge, MIDI ingress, latency compensation, VST3, recording, and buffer-reuse optimization.

## Exit evidence

- ~~The offline fixture renders through the compiled plan, not a hand-wired chain~~ — done 2026-07-17.
- ~~Silence, impulse, allocation, and deterministic-hash tests pass on the plan path~~ — done 2026-07-17.
- App-model device parameters publish to the offline plan (NEXT slice 6).
- `cargo fmt`, strict Clippy, and the full workspace suite stay green.
- Traceability and status match the implementation.
