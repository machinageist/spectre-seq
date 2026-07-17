<!--
Author: Jeff
Date: 2026-07-12
Description: Immediate Geist DAW work queue
Notes: Only active-milestone slices belong here
-->

# Next

- **Status:** accepted
- **Last verified:** 2026-07-16
- **Scope:** immediately actionable R1 and R2-intake slices
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
3. Complete the R1 exit disposition for CORE-001 reorder/migration scope and CORE-004 atomic-save API design; mark only fully evidenced requirements verified.
4. Implement the R2 editable-graph/compiled-plan split around the accepted DSP I/O contract.
5. Move the existing Pulse → Gain → Saturator fixture onto the compiled plan and add silence/impulse/hash gates.
6. Publish device parameter snapshots from the app model to the offline plan before live audio work.
7. Convert feedback copied from `./geist` into narrow, testable interaction slices without implying live audio capability.

## Parallel research

- Obtain visible-session Bitwig Studio evidence.
- Obtain visible-session Ableton Live evidence to corroborate interview self-report.
- Inventory the REAPER guide table of contents.
- Keep the four action-sequence observations separate from the two thematic self-reports.

## Standing constraints

- No frequency or priority claims from the current workflow corpus.
- Every admitted workflow record updates all linked research artifacts in the same slice.
- Callback-reachable code remains allocation-free, lock-free, bounded, and free of I/O and panics.