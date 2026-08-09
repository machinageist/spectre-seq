<!--
Author: Jeff
Date: 2026-08-09
Description: Immediate Spectre work queue
Notes: Only active-milestone slices belong here
-->

# Next

- **Status:** accepted
- **Last verified:** 2026-08-09
- **Scope:** immediately actionable R3 slices; the closed R2 queue is retained as an exit record
- **Decision authority:** Jeff
- **Upstream sources:** `STATUS.md`, `../06-plans/current-milestone.md`
- **Downstream dependents:** implementation sessions
- **Supersedes:** the R2 slice queue, closed 2026-08-09
- **Superseded by:** none
- **Open decisions:** RT-003 acceptance, closed by slice 6; the backend decision that blocked slices 2+ was ratified 2026-08-09
- **Known gaps:** later milestones are intentionally not decomposed here

## Next slices

1. ~~Add the audio-backend decision-gates row and ratify it~~ — closed 2026-08-09 as decision rows 19 (cpal behind an `AudioBackend` trait seam, with a standing re-open trigger on RT-001 violations), 20 (ALSA baseline, JACK behind a feature), and 21 (split-lane RT-002 overflow policy). Jeff confirmed macOS and Linux qualification hardware, so the two-platform exit row stays closable.
2. ~~Stand up cpal behind the `AudioBackend` trait seam with an explicit null implementation~~ — landed 2026-08-09 as `spectre-audio`. `AudioBackend`/`AudioStream` traits, validated `StreamConfig`, `NullBackend` with a deterministic pump, and `CpalBackend` behind a default-on `cpal-backend` feature. 15 tests cover enumeration, config bounds, fail-closed lifecycle transitions, exact block geometry, off-thread rendering, and an allocation-free pump. CI now installs `libasound2-dev`.
3. ~~Define the RT-002 control→render channel per decision 21~~ — landed 2026-08-09 as `spectre_audio::{spsc, control}`. A bounded wait-free SPSC ring returns rejected values instead of dropping them, so a full lane never runs a destructor on the audio thread. Parameters use one `AtomicU32` latest-wins slot per target with a version counter; notes and transport are strict FIFO with counted overflow; retired state travels back to the app thread on a reclaim lane. 19 tests, including a 5,000-write sweep coalescing to one application, bit-exact signed-zero/subnormal/NaN delivery, cross-thread FIFO over 100,000 items, teardown that drops elements straddling the ring wrap, and allocation-free send/drain.
4. ~~Bridge the callback to the existing `CompiledPlan` for a fixed fixture~~ — landed 2026-08-09 as `spectre_audio::bridge::RenderBridge`. It drains transport and notes, executes the existing immutable plan, and interleaves the result; there is no second render path. The equivalence test rebuilds the offline fixture exactly and hashes the bridge's interleaved output with the same FNV-1a walk the offline harness uses, and the hashes match on an audible (nonzero-peak) render. Also covers oversized-block refusal into silence, transport ordering, note deferral without loss, and an allocation-free render. **Surfaced a blocker:** the accepted `AudioProcessor` contract has no runtime parameter seam, so live parameter edits cannot reach processors; the bridge counts them as `parameters_pending` rather than discarding them. See decision row 22.
5. ~~Wrap every callback-reachable path in allocation and lock guards in CI (RT-001)~~ — landed 2026-08-09 as `rt_guard.rs`. A thread-local RT-section flag makes the global allocator attribute violations to the exact call rather than inferring from totals, and a positive control proves the guard actually fires. Covers bridge render, both bridge refusal paths, control drain, retire, and the plan driven through a real backend callback. Lock-freedom is enforced structurally by scanning the RT modules for blocking primitives, labeled as the weaker guarantee it is. **Found and fixed a blocker:** `AudioProcessor` had no `Send` bound, so `CompiledPlan` was not `Send` and the bridge could not move to an audio thread at all. A compile-time assertion now pins it.
6. ~~Accept RT-003 and land denormal flush plus NaN/Inf containment~~ — RT-003 accepted 2026-08-09 and implemented in `CompiledPlan::process`. Containment runs on each node's output before it can reach any downstream device: denormals flush to signed zero (software FTZ-equivalent, portable across the macOS/Linux targets), and any NaN or infinity silences that whole node and records it. Stats accumulate as plain integers on the render path and the bridge republishes them into atomics for off-thread reading. 7 injection tests using test-only poisoning source and effect devices, since the shipping devices contain non-finite values at their own boundary.
7. Run the device lifecycle drill: start, stop, device change, sample-rate change, and device loss, with no panic and no audio-thread blocking.
8. Carry MIDI timestamps into the plan with defined ordering at equal timestamps, reusing the existing bounded event-ordering contract.
9. Report xrun counts and callback headroom through off-thread health telemetry.

## Closed R2 queue

1. ~~Close the accepted JSON project-codec decision~~ — verified 2026-07-16 with a checked-in canonical fixture, exact rewrite/round trip, schema and semantic rejection, and unknown-field preservation.
2. ~~Close the accepted fixed-point `BeatTicks` decision~~ — verified 2026-07-16 at 960 PPQ with checked overflow and transparent signed-integer serialization evidence.
3. ~~Complete the R1 exit disposition for CORE-001 reorder/migration scope and CORE-004 atomic-save API design~~ — dispositioned 2026-07-17: CORE-004 design accepted via the persistence contract; CORE-001 stays implemented with reorder evidence gated to R4 and migration evidence to R5.
4. ~~Implement the R2 editable-graph/compiled-plan split around the accepted DSP I/O contract~~ — landed 2026-07-17 as `spectre-graph` (EditableGraph/CompiledPlan, validated compilation, allocation-free execution, seven behavioral tests, graph-compilation contract).
5. ~~Move the existing Pulse → Gain → Saturator fixture onto the compiled plan and add silence/impulse/hash gates~~ — landed 2026-07-17: fixture renders through the plan bit-identically to the hand-wired chain; silence, impulse, allocation, and hash gates pass.
6. ~~Transfer device parameter snapshots from the app model to the offline plan before live audio work~~ — completed 2026-08-06: the renderer-neutral DTO lives in `spectre-dsp` with private fields, getters, and canonical clamping; the app emits exactly the four canonical fixture identities; offline rendering accepts a complete order-independent set and rejects incomplete, duplicate, unknown, mismatched, or non-canonical input before plan construction.
7. ~~Choose the next narrow, testable interaction slice without implying live audio, broad R3, or automation capability~~ — completed 2026-08-06 via an explicit blind code-level fallback because no copied `./spectre` user-feedback artifact was available. Device Focus Drill-In adds stable app-thread device selection, atomic Build → Shape focus, selected-only descriptor-backed Shape controls, selection-aware feedback/smoke output, and no live or persisted control path.

## Parallel research

- Obtain visible-session Bitwig Studio evidence.
- Obtain visible-session Ableton Live evidence to corroborate interview self-report.
- Inventory the REAPER guide table of contents.
- Keep the four action-sequence observations separate from the two thematic self-reports.

## Standing constraints

- No frequency or priority claims from the current workflow corpus.
- Every admitted workflow record updates all linked research artifacts in the same slice.
- Callback-reachable code remains allocation-free, lock-free, bounded, and free of I/O and panics.
