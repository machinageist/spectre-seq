<!--
Author: Jeff
Date: 2026-07-11
Description: Next small rebuild slices and current blockers for the Geist DAW rebuild
Notes: Only the active milestone's slices belong here; the roadmap lives in docs/06-plans
-->

# Next

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** immediately actionable slices and blocking decisions
- **Decision authority:** Jeff
- **Upstream sources:** `STATUS.md`, `../06-plans/current-milestone.md`, `../02-reference-research/workflow-field-study/source-index.md`
- **Downstream dependents:** work sessions resuming the rebuild
- **Supersedes:** continuation sections of ad hoc handoff files (`docs/archive/handoffs/sol-handoff.md`)
- **Superseded by:** none
- **Open decisions:** listed under Blockers
- **Known gaps:** slices beyond the active milestone are intentionally not enumerated

## Next slices, in order

1. ~~Extract and admit 1–2 Ableton Live workflow observations~~ — done 2026-07-11 (`WF-ABLETON-LUSTWERK-005`, `WF-ABLETON-TSURUTA-006`, interview self-reports at `low` confidence).
2. Obtain visible-session Bitwig Studio evidence: the two official artist interviews were inspected 2026-07-11 and yielded preference/friction statements without admissible action sequences; the `/latest/` guide's own version identity is resolved as 5.3, but 5.3 must still be verified against an official release notice.
3. Obtain visible-session Ableton Live video evidence to corroborate the interview self-reports at gesture level; then continue alternating products until each major DAW has complete-task plus friction evidence. Do not add more FL Studio sources until other products catch up.
4. Promote at least one manual from `inventory-only` to a section-level coverage matrix (Ableton Live 12 is the strongest candidate: versioned URL, 42-chapter inventory already recorded).
5. Draft `docs/01-requirements/decision-gates.md` from the open decisions already recorded across the audits.
6. Draft `docs/00-product/` North Star documents once field evidence supports priority claims.
7. Draft requirements ledger and traceability skeleton; then architecture contracts; then rebuild roadmap; then R0/R1.

## Blockers requiring Jeff

None currently block the slices above. Decisions that will block later slices (collect evidence now, decide at the gate):

- active-workspace migration posture (in-place vs. namespaced);
- project encoding and bundle layout;
- UI stack confirmation for the rebuild;
- VST3 bindings and license posture (SDK tag v3.8.0_build_66 identified, submodule/license audit incomplete);
- platform matrix and first-class platform order;
- relationship between stacksynth, modular rack, and the flagship synth.

## Standing constraints

- Keep the modular-rack/application-code lane separate from rebuild documentation commits.
- Preserve the five pre-existing tracked modifications uncommitted until attributed.
- No frequency/priority claims from the current 4-observation corpus.
- Every admitted workflow observation MUST update all five research artifacts in the same slice.
