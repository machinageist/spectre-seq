<!--
Author: Jeff
Date: 2026-07-17
Description: The single active Geist DAW milestone
Notes: Exactly one milestone is active; the roadmap owns ordering
-->

# Current Milestone — R2 Offline Graph

- **Status:** accepted
- **Last verified:** 2026-08-06
- **Scope:** editable graph → validated compiled plan → deterministic offline render
- **Decision authority:** Jeff
- **Upstream sources:** `rebuild-roadmap.md`, GRAPH-001..002, DSP device I/O contract
- **Downstream dependents:** `../status/NEXT.md`, implementation slices
- **Supersedes:** the R0/R1 foundation milestone, exited 2026-07-17
- **Open decisions:** input-bus summing semantics and explicit feedback pricing at their intake
- **Known gaps:** the offline app-snapshot seam, fixture migration, and the four render gates are closed; later live-audio work remains R3 scope

## R0/R1 exit record

R0/R1 exited 2026-07-17 with all exit evidence passing: formatting, strict Clippy, and the full workspace test suite; tempo/time/transport/event/ID/persistence property coverage; the deterministic offline harness; and traceability matching implementation. CORE-004's atomic-save design is accepted with implementation at R4; CORE-001's reorder/migration evidence is explicitly gated to R4/R5.

## Current evidence

`geist-graph` implements the GRAPH-001 split: an app-thread `EditableGraph` with stereo-bus semantics and single-feed inputs, and an immutable `CompiledPlan` built through validated compilation (ancestor inclusion, missing-input rejection, implicit-cycle diagnostics, factory layout verification, deterministic ordering, preallocated planar buffers). Plan execution is allocation-free and lock-free with take/restore buffer handoff. Seven behavioral tests cover editing, compilation, and execution.

The offline Pulse → Gain → Saturator fixture renders through the compiled plan, bit-identical to a hand-wired chain, with the silence, impulse, allocation, and deterministic-hash gates passing on that path.

Slice 6 closes the app-path integration gap at the smallest offline seam. The renderer-neutral `DeviceParameterSnapshot` DTO lives in `geist-dsp`, keeps its fields private, exposes project-instance and static-key getters, and clamps through a supplied canonical `DspParameter`. `AppModel` allocates distinct nonzero device and parameter `ObjectId`s, exposes device structure read-only, routes edits through descriptor-clamped identity setters, and emits exactly the fixed four-parameter fixture by stable identities with explicit invariant errors. `render_app_snapshot` accepts exactly one each of `pulse.level`, `gain.gain`, `saturator.drive`, and `saturator.mix` in any order; it rejects empty, partial, duplicate, aliased, unknown, mismatched, non-finite, out-of-range, and non-canonical input before constructing processors for the immutable `CompiledPlan`. Targeted tests prove exact hand-wired equivalence for four distinct app edits, discriminate saturator drive from mix, preserve deterministic identical snapshots and project-instance IDs, match backend defaults, pin the documented numeric boundary policy, and fail closed on malformed fixture snapshots. `CompiledPlan::process` is unchanged; no live callback, audio backend, or automation path is implied.

No copied `./geist` user-feedback artifact was available for Slice 7, so an explicit blind code-level fallback selected Device Focus Drill-In. The slice adds only renderer-neutral interaction focus around that seam. `AppModel` starts with Pulse selected by stable device `ObjectId`; one fail-closed `open_device_in_shape` operation validates membership before changing both focus and lens. Every Build card exposes a labeled action and selected styling, while Shape renders only the selected-device presentation with the existing descriptor-owned controls and setter. Lens changes, parameter edits, and complete offline snapshots preserve their prior behavior, feedback/smoke output names the focus, and invalid identity leaves lens, track/device selection, structure, and values untouched. Recoverable UI focus or parameter mismatches produce actionable feedback rather than panic. Selection is transient app-thread state: no graph mutation, persistence, callback transport, or live audio behavior is added.

## Requirements in scope

GRAPH-001 (implemented with offline app-path integration evidence), GRAPH-002 intake, RT-001 as workspace policy.

## Non-goals

Live audio I/O, callback bridge, MIDI ingress, latency compensation, VST3, recording, and buffer-reuse optimization.

## Exit evidence

- ~~The offline fixture renders through the compiled plan, not a hand-wired chain~~ — done 2026-07-17.
- ~~Silence, impulse, allocation, and deterministic-hash tests pass on the plan path~~ — done 2026-07-17.
- ~~App-model device parameters publish to the offline plan~~ — done 2026-08-06 with owned typed snapshots, fail-closed identity matching, backend defaults, and compiled-plan render evidence.
- ~~`cargo fmt`, strict Clippy, and the full workspace suite stay green~~ — passed 2026-08-06 with 155/155 tests, the selected-device app smoke test, and the offline self-test.
- ~~Traceability and status match the implementation~~ — updated 2026-08-06 for the bounded Device Focus Drill-In; this does not declare broad R2 product readiness.
