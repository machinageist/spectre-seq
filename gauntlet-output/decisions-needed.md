<!--
Author: Jeff
Date: 2026-08-14
Description: Open decisions escalated out of specs and scorecards, awaiting Jeff
Notes: Questions that outgrow a spec's section 8 land here; nothing here blocks silently
-->

# Decisions Needed

- **Status:** accepted
- **Last verified:** 2026-08-14
- **Scope:** decisions the gauntlet cannot make for itself
- **Decision authority:** Jeff
- **Upstream sources:** spec §8 entries, scorecard escalations
- **Downstream dependents:** `manifest.md`, affected specs
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** one, D-G1 below
- **Known gaps:** none

## How this file is used

A spec records its own uncertainty in §8. A question escalates here when it cannot be
resolved inside one feature — when it changes accepted authority, spans features, or
needs Jeff's product judgment. Auto-fail rule AF-1 requires this route: a spec may not
contradict an accepted decision by assertion, only by proposing supersession here.

Entries are never deleted. Resolved ones move to the resolved section with the outcome.

## Open

### D-G1 — Lens weighting for `criteria.md`

**Raised:** 2026-08-14, Phase 0.3.
**Blocks:** all of Phase 1. No spec may be dispatched until this closes.

`criteria.md` version 1 proposes four lenses instead of the template's default three,
on the reasoning that a realtime audio engine has a hard-constraint dimension no UI or
competitive lens covers:

| Lens | Proposed weight |
|---|---|
| 1 — Realtime & Correctness | 35% |
| 2 — DAW Workflow Depth | 25% |
| 3 — Product Identity & Scope Discipline | 20% |
| 4 — Truthfulness & Evidence | 20% |

**Options:** accept as proposed; reweight; merge lenses 3 and 4 back into a
three-lens shape; or add a fifth. The system supports two to five.

**Recommendation:** accept as proposed. Lens 1 carries the largest weight because
RT-001/002/003 are the requirements a DAW cannot be wrong about, and lens 4 is
weighted equally with lens 3 because misdescribing current state is precisely what
made run 1's output worthless.

Also pending in the same sign-off, though not itself a judgment call: confirmation
that `feature-tree.md`'s nine features match the accepted R4 queue.

## Resolved

None yet.
