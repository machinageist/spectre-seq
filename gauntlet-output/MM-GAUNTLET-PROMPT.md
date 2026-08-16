<!--
Author: Jeff
Date: 2026-08-15
Description: Self-contained gauntlet prompt for the Spectre mixing/mastering suite
Notes: Paste as the entry prompt for an MM gauntlet run; Phase 0 is already complete, so a run starts at Phase 1
-->

# Spectre Mixing & Mastering Suite — Gauntlet Prompt

- **Status:** proposed
- **Last verified:** 2026-08-15
- **Scope:** the complete instruction set for an agent running the mixing/mastering spec gauntlet
- **Decision authority:** Jeff
- **Upstream sources:** `criteria-mixing-mastering.md`, `templates/UNIVERSAL-GAUNTLET.md`, `templates/SPEC-TEMPLATE.md`, `templates/SCORECARD-TEMPLATE.md`
- **Downstream dependents:** `specs-mm/`, `spec-scorecards-mm/`, `manifest-mm.md`
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** D-MM1 through D-MM4 in `decisions-needed.md` — **D-MM2 and D-MM3 gate dispatch**
- **Known gaps:** no mixing/mastering implementation exists, so every spec's §7.1 starts from absence

---

## Read this first: what makes this gauntlet different

This is not the R4 gauntlet with different feature names. Three things change.

1. **The grading standard is `criteria-mixing-mastering.md`, not `criteria.md`.** It has different lenses, different weights, and three additional auto-fails.
2. **Three whole categories of number do not exist in the benchmark corpus** — latency figures, time constants, and loudness-standard conformance. The single most likely way a spec fails here is by stating one anyway. Two of the three new auto-fails exist for exactly this.
3. **The work is not approved.** D-MM2 — whether Spectre builds this suite at all — is open. Specs authored here are proposals under examination, not scheduled work. No spec may read as though the decision is made.

## Preconditions — check before dispatching anything

**Stop and ask Jeff if any of these is false.**

1. `gauntlet-output/criteria-mixing-mastering.md` exists and its status is `accepted`. It is `proposed` as of 2026-08-15 and **D-MM1 (lens weighting) is open**. Grading against a proposed standard produces scores nobody agreed to.
2. **D-MM2 is resolved** — Jeff has said whether the suite is being built. If it is deferred, a gauntlet run is still legitimate as a *design exercise*, but say so in the manifest and do not write specs that imply scheduled work.
3. **D-MM3 is resolved** — the suite's roadmap placement. Criterion 3A grades "does the spec state where this lands", and it cannot be graded against an unmade decision.
4. The R4 loop is not mid-remediation on a feature this would compete with. As of 2026-08-15 R4-1 has passed at 2.950 and batch 2 is next; running both loops concurrently is Jeff's call, not the agent's.

## Phase 0 — already complete, do not redo

Unlike a fresh gauntlet, Phase 0 is done. **Do not re-run the criteria interview and do not re-derive the feature tree from source.**

- **Criteria:** `criteria-mixing-mastering.md`, version 1.
- **Benchmark set:** FabFilter and iZotope Ozone 12. Locked. Do not add benchmarks, and in particular do not add a benchmark you have no ledger records for.
- **Feature tree:** the component tree in `criteria-mixing-mastering.md` §"The suite's component tree" — Tier 0 `MM-0.1`–`MM-0.7`, Tier 1 `MM-1`–`MM-7`, Tier 2 `MM-8`–`MM-11`.
- **Research corpus:** `docs/02-reference-research/fabfilter.md`, `fabfilter-observations.md` (199 records), `ozone.md`, `ozone-observations.md` (179 records), `source-ledger.json`.

**Do not re-interview the benchmarks.** The dossiers exist and are `in-review`. Re-deriving competitor behavior from recollection is the failure mode that produced the discarded 2026-08-12 run.

## Dispatch order

Tier 0 before Tier 1 — this is criterion 3C, and it is the tree's whole point. A limiter spec written before the metering core is specified will invent a private meter, and that scores 1 or lower.

| Batch | Features | Concurrency | Rationale |
|---|---|---|---|
| 1 | `MM-0.3` latency declaration and compensation | 1, alone | Nothing else in the tier is correct without it, it has near-zero benchmark evidence so it must be derived from Spectre's own design, and it is the most likely place to discover the suite is bigger than it looks. Same reasoning that put R4-1 alone ahead of its batches. |
| 2 | `MM-0.1` metering core, `MM-0.4` I/O and gain staging, `MM-0.7` preset and state contract | 3 | Independent of each other; all three are contracts rather than DSP. |
| 3 | `MM-0.2` analyzer core, `MM-0.5` oversampling seam, `MM-0.6` crossover infrastructure | 3 | All three depend on `MM-0.3` being settled. |
| 4 | `MM-1` equalizer, `MM-5` de-esser | 2 | The richest-evidence device and the smallest useful device. Deliberately not the whole of Tier 1 — see the stop rule below. |
| — | **STOP.** | | Report to Jeff before any further batch. |

**The stop rule.** After batch 4, halt and write the summary. Four Tier-0 contracts plus two devices is enough to know whether the suite is coherent and whether the criteria are grading it usefully. Specifying all eleven remaining components before anyone has read one is how a gauntlet produces volume instead of judgment.

## Phase 1 — spec generation

For each feature, dispatch **one author agent**. Give it exactly this context and no more:

1. `gauntlet-output/criteria-mixing-mastering.md` — the binding standard, read in full.
2. `gauntlet-output/templates/SPEC-TEMPLATE.md` — every section gets filled.
3. `docs/README.md`, `docs/status/STATUS.md`, `docs/status/NEXT.md` — authority and verified state.
4. `docs/02-reference-research/fabfilter.md` and `ozone.md` — the dossiers, including their acceptance blockers.
5. `docs/02-reference-research/fabfilter-observations.md` and `ozone-observations.md` — the record sets, cited by ID.
6. The crate source. **The author reads it; it does not trust `STATUS.md` for §7.1.**

**Output:** `gauntlet-output/specs-mm/{feature-id}.md`.

### Standing instructions to every author agent

- **Your §7.1 starts from absence.** Spectre has no mixing/mastering device, no analyzer, no oversampling, no crossover, no latency declaration, and no loudness measurement of any kind. Metering today is `BridgeTelemetry`'s counters and nothing more. Verify this yourself rather than restating it, and if you find otherwise, that is a finding worth surfacing.
- **Cite every benchmark claim by `OBS-` ID.** If a surface has no record, write "no citable evidence for {surface}; recorded as a research need." That scores 3. Inventing it fails.
- **You may not state a latency figure taken from a benchmark**, because none exists in the corpus. Derive yours from your own design, in samples, and show the derivation.
- **Loudness: cite the record, never memory.** As of 2026-08-15 the standards ARE read — ITU-R BS.1770-5 Annexes 1–2, EBU R 128, Tech 3341, Tech 3342 — giving 95 records in `docs/02-reference-research/loudness-standards-observations.md`. The algorithm, K-weighting coefficients, gating rules, true-peak method, and LRA definition are citable. **Two things remain absolutely prohibited under MM-AF-5:** any conformance claim ("conforms to", "implements", "EBU Mode" — the test-signal sets were never retrieved, `GAP-LOUDNESS-0008`), and any streaming or mastering delivery target (none exists in any of the four documents, `GAP-LOUDNESS-0003`; the only citable target is R 128's *broadcast* −23.0 LUFS / −1 dBTP). Read MM-AF-5 in full before writing a single loudness sentence — it names two further traps, on oversampling factor and on sample rate.
- **You may not describe your design as matching, modeling, or emulating any vendor's**, and you may not name a device after one.
- **Every numeric bound you introduce gets a rationale row scheduled into `docs/01-requirements/requirements-ledger.md` in your §7.2 modified-files list.** Arguing it in prose is not enough; PROD-003 requires the row.
- **Route what you cannot decide to §8**, and escalate anything that spans features to `decisions-needed.md` as a `D-MM*` entry. Do not resolve a product question by asserting it.

## Phase 2 — blind verification

Dispatch a **separate** agent that has not seen the spec being written and has no context from the author's run. An author never verifies its own artifact.

Give it: the finished spec, `criteria-mixing-mastering.md`, `templates/SCORECARD-TEMPLATE.md`, and the repository. Nothing about the author's reasoning.

**Output:** `gauntlet-output/spec-scorecards-mm/{feature-id}-scorecard.md`, and on a second or later iteration, `{feature-id}-scorecard-iter{N}.md` — one file per iteration that produced a verdict, so a failure's record survives its fix.

### Standing instructions to every verifier

- **Open every source path the spec cites and check the claim against the file.** This is the feasibility rule; it is not satisfied by reading §7.1.
- **Open every `OBS-` ID the spec cites and confirm the record says what the spec says it says.** This is new relative to the R4 gauntlet and it is where a mastering spec is most likely to drift — the corpus is large enough that a plausible-sounding record ID is easy to assume and easy to check.
- **Hunt specifically for the three absent number classes.** Any latency figure, any attack/release/hold range, and any loudness threshold or target in the spec is a fabrication unless it is derived in-spec or ledger-backed. Search the spec for `ms`, `LUFS`, `dBTP`, `samples`, and `latency` and check each hit.
- **A test that cannot fail scores 0**, and for any metering surface, at least one test must assert a reported number against a synthesized signal with a known answer.
- **State your review's scope honestly** in the scorecard summary — what you opened in full, and what you sampled.
- **A verdict is PASS or FAIL against `criteria-mixing-mastering.md`'s four binding conditions.** A high composite does not survive an auto-fail; that is the point of the auto-fails.

## Phase 3 — remediation loop

Maximum **3** rounds per feature, then escalate to `gauntlet-output/gap-reports/`.

1. Read the scorecard's remediation brief.
2. Dispatch a remediation agent with the spec, the scorecard, and the criteria. It fixes Priority 1 items and as many Priority 2 items as it can defend.
3. Dispatch a **fresh** blind verifier — not the one that failed it, and not the remediator.
4. Increment the iteration in the spec header and record what changed in the header Notes.

**Do not let a remediation loop become goalpost-moving.** A verifier that finds a genuinely new defect on iteration 2 should record it at the priority it deserves — a new *blocking* finding on a late iteration needs to clear a high bar, and a new non-blocking one belongs in Priority 2 or 3 with the spec still passing.

## Phase 4 — final report

Write `gauntlet-output/summary-mm.md`:

- score distribution across the batches run;
- **which auto-fails actually fired**, and whether MM-AF-5, MM-AF-6, and MM-AF-7 earned their place — if none of the three ever fires, either the authors are unusually disciplined or the rules are unenforceable, and both are worth knowing;
- common failure patterns;
- escalated features and their gap reports;
- **whether the component tree survived contact.** If four Tier-0 contracts and two devices did not decompose the way the tree predicts, say so — the tree is proposed, and a gauntlet that cannot revise its own tree is a rubber stamp;
- every `D-MM*` entry the run raised;
- a recommendation to Jeff on whether to continue past the stop rule.

## Run-state files

This loop keeps its own state, parallel to the R4 loop's and never mixed with it:

```text
gauntlet-output/
├── criteria-mixing-mastering.md    # the binding standard (this loop's)
├── MM-GAUNTLET-PROMPT.md           # this file
├── manifest-mm.md                  # per-feature state and dated run history
├── HANDOFF-mm.md                   # the next executable action, updated before stopping
├── summary-mm.md                   # Phase 4
├── specs-mm/
├── spec-scorecards-mm/
└── gap-reports/                    # shared with the R4 loop; prefix MM entries
```

`manifest-mm.md` and `HANDOFF-mm.md` are created by the first run. Every agent updates the handoff before stopping, **including after a failure** — the R4 loop's usage-cap terminations on 2026-08-15 are the reason that rule exists, and the runs that left a handoff lost nothing.

## The state machine

```text
pending → spec-in-progress → spec-review → spec-remediation-{1,2,3}
        → spec-pass | escalated
```

Implementation stages are deliberately absent. **This gauntlet produces specs only.** D-MM2 is open, so no mixing/mastering code is authorized, and a gauntlet that slid from specifying into building would be deciding a product question by momentum.
