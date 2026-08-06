<!--
Author: Jeff
Date: 2026-07-12
Description: Immediate Geist DAW work queue
Notes: Only active-milestone slices belong here
-->

# Next

- **Status:** accepted
- **Last verified:** 2026-08-06
- **Scope:** immediately actionable R2 slices; R1 rows retained as exit record
- **Decision authority:** Jeff
- **Upstream sources:** `STATUS.md`, `../06-plans/current-milestone.md`
- **Downstream dependents:** implementation sessions
- **Supersedes:** removed prototype continuation plans
- **Superseded by:** none
- **Open decisions:** none; remaining R1 work is evidence disposition
- **Known gaps:** later milestones are intentionally not decomposed here

## Next slices

1. ~~Close the accepted JSON project-codec decision~~ — verified 2026-07-16 with a checked-in canonical fixture, exact rewrite/round trip, schema and semantic rejection, and unknown-field preservation.
2. ~~Close the accepted fixed-point `BeatTicks` decision~~ — verified 2026-07-16 at 960 PPQ with checked overflow and transparent signed-integer serialization evidence.
3. ~~Complete the R1 exit disposition for CORE-001 reorder/migration scope and CORE-004 atomic-save API design~~ — dispositioned 2026-07-17: CORE-004 design accepted via the persistence contract; CORE-001 stays implemented with reorder evidence gated to R4 and migration evidence to R5.
4. ~~Implement the R2 editable-graph/compiled-plan split around the accepted DSP I/O contract~~ — landed 2026-07-17 as `geist-graph` (EditableGraph/CompiledPlan, validated compilation, allocation-free execution, seven behavioral tests, graph-compilation contract).
5. ~~Move the existing Pulse → Gain → Saturator fixture onto the compiled plan and add silence/impulse/hash gates~~ — landed 2026-07-17: fixture renders through the plan bit-identically to the hand-wired chain; silence, impulse, allocation, and hash gates pass.
6. ~~Transfer device parameter snapshots from the app model to the offline plan before live audio work~~ — completed 2026-08-06: the renderer-neutral DTO lives in `geist-dsp` with private fields, getters, and canonical clamping; the app emits exactly the four canonical fixture identities; offline rendering accepts a complete order-independent set and rejects incomplete, duplicate, unknown, mismatched, or non-canonical input before plan construction.
7. ~~Choose the next narrow, testable interaction slice without implying live audio, broad R3, or automation capability~~ — completed 2026-08-06 via an explicit blind code-level fallback because no copied `./geist` user-feedback artifact was available. Device Focus Drill-In adds stable app-thread device selection, atomic Build → Shape focus, selected-only descriptor-backed Shape controls, selection-aware feedback/smoke output, and no live or persisted control path.

## Parallel research

- Obtain visible-session Bitwig Studio evidence.
- Obtain visible-session Ableton Live evidence to corroborate interview self-report.
- Inventory the REAPER guide table of contents.
- Keep the four action-sequence observations separate from the two thematic self-reports.

## Standing constraints

- No frequency or priority claims from the current workflow corpus.
- Every admitted workflow record updates all linked research artifacts in the same slice.
- Callback-reachable code remains allocation-free, lock-free, bounded, and free of I/O and panics.