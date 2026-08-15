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
- **Open decisions:** none blocking; D-R1 is a research need, not a gate
- **Known gaps:** none

## How this file is used

A spec records its own uncertainty in §8. A question escalates here when it cannot be
resolved inside one feature — when it changes accepted authority, spans features, or
needs Jeff's product judgment. Auto-fail rule AF-1 requires this route: a spec may not
contradict an accepted decision by assertion, only by proposing supersession here.

Entries are never deleted. Resolved ones move to the resolved section with the outcome.

## Open

### D-R1 — Benchmark evidence gaps for Logic Pro and Serum 2

**Raised:** 2026-08-14, at Phase 1 dispatch.
**Blocks:** nothing. Recorded so specs name the gap instead of filling it in.

Jeff set the AAA benchmark set to Ableton Live, Logic Pro, Serum 2, Phase Plant, and
VCV Rack 2. The citable evidence behind those five is uneven:

| Benchmark | `OBS-` records | Dossier research state |
|---|---|---|
| Ableton Live 12 | 85 | inventory-only |
| Phase Plant | 11 | inventory-only |
| VCV Rack 2 | 6 | inventory-only |
| Serum 2 | 2 | **blocked-source-gap** |
| Logic Pro | **0** | inventory-only |

Serum 2's dossier states it "cannot currently be marked source-complete or accepted
for product planning." Logic Pro has no behavioral observations at all.

**Consequence for this run:** lens 2 criterion 2G requires specs to cite what exists
and name what does not. A spec that says "no citable Logic Pro evidence for this
surface" is correct and scores well; one that invents Logic behavior fails AF-6.

**Decision for Jeff, not blocking:** whether to commission Logic Pro and Serum 2
observation passes before the R4 UI-facing features (R4-4 track model, R4-6 first
devices) reach implementation. Until then those two benchmarks contribute little to
grading and the effective bench is Ableton, Phase Plant, and VCV.

### D-R2 — Provenance of the Serum 2 user guide, and whether its extraction can be accepted

**Raised:** 2026-08-15, after the Serum 2 research pass was cut off by a usage cap.
**Blocks:** promotion of any record in `docs/02-reference-research/serum-2-observations.md`.
Blocks nothing in the R4 gauntlet.

A research pass produced ~250 atomic Serum 2 observations, the large majority sourced
from what it describes as an official **354-page Serum 2 User Guide, manual 1.0.3,
documenting product 2.0.18**. If that guide is publicly authorized, it closes
`GAP-SERUM2-0001` — the blocker that has held the Serum 2 dossier at
`blocked-source-gap` since 2026-07-11 — and materially upgrades the weakest benchmark
in Jeff's AAA set.

**The problem.** The run was terminated before it wrote any source records. The ledger
has no URL, publisher, access date, source class, or access limitation for that guide
or for the three practitioner sources. So the file's central evidentiary claim rests on
provenance that was never recorded.

This is not a formality. `methodology.md` §"Research boundary" permits **publicly
authorized sources only** and explicitly forbids leaked or private manuals.
`serum-2.md` still carries the open decision *"whether a legitimately available
installed/customer guide can be reviewed without redistribution,"* and
`GAP-SERUM2-0004` records that authenticated-customer documentation was never
assessed. Whether this guide sits inside or outside that boundary **is** the open
question, and the extraction cannot answer it about itself.

**Decision for Jeff:**

1. Where did the guide come from — an official public endpoint, an authenticated
   customer download, or a third-party upload? Only the first is unambiguously inside
   the methodology; the second is the open decision; the third is outside.
2. If inside: authorize a completion pass to write the ledger records, fix the
   self-reported count errors, and run the second-pass contradiction review the
   review gates require. `GAP-SERUM2-0001` and `-0002` may then close.
3. If outside or unresolved: the file is deleted rather than quarantined, and the
   dossier stays `blocked-source-gap`.

**Recommendation:** do not promote anything from the file until (1) is answered. The
work is preserved under a quarantine banner and cited by nothing. Note the run also
disclosed that roughly half the guide's chapters remain `unreviewed`, so even the
favorable path is a partial coverage matrix, not a source-complete dossier.

## Resolved

### D-G1 — Lens weighting for `criteria.md` — **accepted as proposed, 2026-08-14**

Four lenses rather than the template's three, on the reasoning that a realtime audio
engine has a hard-constraint dimension no UI or competitive lens covers. Weights:
Realtime & Correctness 35%, DAW Workflow Depth 25%, Product Identity & Scope
Discipline 20%, Truthfulness & Evidence 20%. Jeff authorized the run without
amending them; `feature-tree.md`'s nine features were confirmed in the same breath.
