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
- **Open decisions:** audio backend selection blocks slice 2 and later
- **Known gaps:** later milestones are intentionally not decomposed here

## Next slices

1. Add the audio-backend decision-gates row and ratify it. Record candidates, reversibility, license posture, and the macOS/Linux co-first-class constraint from decision 1. No code. This unblocks every later slice and is the only slice that can start today without a pending choice.
2. Stand up the chosen backend behind a trait seam with an explicit null implementation, and prove device enumeration and open/close off the audio thread. No plan execution yet.
3. Define the RT-002 control→render channel: bounded, wait-free, stated overflow policy, off-thread reclamation of retired state. Land it with concurrency tests before anything writes to it from the app thread.
4. Bridge the callback to the existing `CompiledPlan` for a fixed fixture, driving the same plan the offline harness renders. Prove callback output matches an offline render of identical input.
5. Wrap every callback-reachable path in allocation and lock guards in CI (RT-001), extending the counting-allocator approach already used by the plan-execution tests.
6. Accept RT-003 and land denormal flush plus NaN/Inf containment with per-node-type injection fixtures: isolate the offending node, emit silence, surface a diagnostic off-thread.
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
