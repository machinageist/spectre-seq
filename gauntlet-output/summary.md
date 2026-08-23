<!--
Author: Jeff
Date: 2026-08-22
Description: Phase 4 report for gauntlet run 2 — all nine R4 features spec'd, reviewed, and passed
Notes: Grades proposals only. Not one line of R4 implementation exists; no score here is evidence about code.
-->

# Gauntlet Summary — Spectre R4

- **Status:** accepted
- **Last verified:** 2026-08-22
- **Scope:** results of run 2, Phase 1 through Phase 4, for the nine R4 features
- **Decision authority:** Jeff
- **Upstream sources:** `manifest.md`, `criteria.md`, `feature-tree.md`, every file under `specs/` and `spec-scorecards/`
- **Downstream dependents:** `HANDOFF.md`, the implementation decision
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** D-R1, D-R2, D-R3, **D-R4**, D-MM1–D-MM4 in `decisions-needed.md`; R4-7's Q2
- **Known gaps:** **no implementation exists**; every score below grades a proposal

## The one-line result

**Nine features spec'd, nine passed, zero escalated, mean composite 2.898.** Two failed their
first review and both passed after one remediation round. No feature needed a second.

## Score distribution

| Feature | Score | Iters | Note |
|---|---|---|---|
| R4-2 runtime-parameter-seam | **3.000** | 1 | perfect on evidence integrity; surfaced D-R3 |
| R4-7 project-persistence | **3.000** | 1 | exposed a false claim in `criteria.md` itself |
| R4-3 linux-device-qualification | **2.967** | 2 | 2.633 → 2.967; hardware-blocked |
| R4-1 live-audio-wiring | **2.950** | 2 | 2.750 → 2.950; the run's first FAIL |
| R4-6 first-devices | **2.864** | 1 | tightest AF-4/AF-5 exposure, cleanly handled |
| R4-8 offline-bounce | **2.848** | 1 (+1 remediation) | caught a 48× arithmetic error |
| R4-9 e2e-and-qa | **2.845** | 1 | raised D-R4 |
| R4-4 track-model | **2.810** | 1 | refused a false claim in an accepted document |
| R4-5 midi-clips | **2.798** | 1 | found a lane capacity that is not what it says |

Lens 4 (Truthfulness & Evidence) carried the run: no feature scored below 2.667 on it, and
three scored 3.000. Lens 1 (Realtime & Correctness) was the weakest at a 2.79 mean — every
sub-3.000 score in the run traces to a test-specification or control-lane defect, not to a
design error.

## What actually failed, and why it matters

**Both first-round failures were evidence defects, not design defects.**

- **R4-1** cleared every numeric threshold at 2.750 and failed anyway, on a single false
  sentence: it named `midi.rs` as one of four RT-scanned modules when the scan reads `null.rs`
  — and its own change list modified `null.rs`. The spec's central RT-001 safety argument rested
  on the error.
- **R4-3** cleared every threshold at 2.633 and failed on three: a §7.1 that misdescribed which
  tests construct a backend, a constant (90) contradicting its own derivation (⌊93.75⌋ − 4 = 89),
  and an unaccounted code path that made its own predicted ALSA risk indistinguishable from a
  dead driver.

Neither would have been caught by reading the spec. Both required opening the file.

## Common failure patterns

Ranked by frequency across all reviews:

1. **Arithmetic that contradicts its own derivation** — three occurrences: R4-3's 90-vs-89,
   R4-8's 337,500-vs-16,200,000 (a **48×** error headed for `requirements-ledger.md` verbatim),
   R4-9's "seven of ten rows" against a table showing eight. The R4-9 author caught its own.
2. **Tests that cannot fire** — four: R4-1's tests 3 and 9 (non-compiling), R4-5's I-10 (asserts
   an overflow that cannot occur), R4-6's `select_device` (a method that does not exist),
   R4-8's test 1 (compares an implementation against itself after its own refactor).
3. **Off-by-one line citations** — pervasive and low-severity. Every review found some; none
   was load-bearing.
4. **Claims about a file's role rather than its content** — R4-1's scan list, R4-3's backend
   attribution. Both passed a casual read and failed an actual one.

**The pattern that did not appear:** not one spec invented a benchmark claim. Every `OBS-` ID
cited across nine specs resolved and said what was claimed. R4-8 went further — it *found*
`OBS-SR2-GLOB-008`, the one convenient export record, and refused it on D-R2's quarantine.

## The finding that outranks the specs

**An accepted document was misstating the codebase inside the rule that forbids exactly that.**

`criteria.md`'s AF-2 illustration and `manifest.md`'s run-1 entry both recorded that the
discarded 2026-08-12 spec invented `AppModel::add_track()` and a track list sidebar — "None
exist." Both exist, and have since commit `6c397d9`: `add_track` at
`crates/spectre-app/src/lib.rs:405–422`, the sidebar at `main.rs:115–160`, with two covering
tests at `tests/app_model.rs:33–49`. Only `Track::arm`/`Track::mute` were genuinely invented,
so the discarded spec's disposition stands on AF-5 and corrupted encoding regardless.

It surfaced because the R4-7 author cited `lib.rs:405` accurately, and was independently
confirmed by the R4-4 author, who **was briefed with the false claim, checked it anyway, and
refused it in writing.** Both documents now carry the correction in place, unedited-away.

## Escalated features

**None.** Zero features reached the three-round remediation limit or a gap report.

## What every passing spec still owes

A spec pass here means *the proposal is sound and honestly evidenced* — never *ready to build
unchanged*. Every one of the nine carries at least one must-fix-before-implementation item:

| Feature | Owed before code |
|---|---|
| R4-1 | §4.4's binding rule has no automated coverage (lives in `main.rs`, untestable) |
| R4-2 | **D-R3 blocks it** — an accepted doc says `Gain` smooths; the shipped `Gain` does not |
| R4-3 | **hardware-blocked**; needs a Linux host with a real ALSA device |
| R4-4 | line-citation fixes; a §7.2 path typo |
| R4-5 | `SCHEDULE_LANE_CAPACITY` is 3 not 2; I-10 can never pass |
| R4-6 | §5.3 calls `select_device`, which does not exist |
| R4-7 | **Q2 blocks it** — a schema bump may disturb a `verified` CORE-003 acceptance test |
| R4-8 | a third verbatim copy of the fixture chain, unaddressed by §7.2 |
| R4-9 | §5.4's fifteen manual rows describe surfaces that do not exist yet |

## Remediation phase — completed 2026-08-23

After all nine passed, every must-fix-before-implementation item was worked. **Four remediation
rounds ran; all four closed their items and none was skipped.** Specs now at iteration 2 or 3:
R4-4 (1,817 lines), R4-5 (1,599), R4-6 (1,396), R4-8 (2,200).

**The finding of the phase: remediations caught reviewer errors at a steady rate — four rounds,
four sets of overturned findings, right every time.**

| Round | What it refused, and was right about |
|---|---|
| R4-6 | Two "off-by-one corrections" that were **themselves off by one**; the spec's citations were already exact |
| R4-5 | A prescribed fix that was **mathematically unworkable** — `MIN_CAPACITY = 2` means every argument 0–3 yields the same ring |
| R4-4 | Two wrong pointers, **plus a fabricated quotation** the review had passed |
| R4-8 | A defect the review **undercounted** — a third *and fourth* copy, which is what made "accept it" untenable |

In every case the remediator verified against source rather than complying. **The blind review is
necessary but not sufficient; the remediation stage is functioning as a second independent
verification pass**, and the run's evidence quality depends on it doing so. Briefs now state
explicitly that reviewers are fallible and that a finding believed wrong must be reported, not
applied.

**Three defects surfaced only in remediation, none by review:**

1. **A fabricated quotation.** R4-4's §7.1 quoted `manifest.md:19` as text that reproduces at no
   commit; at `2e005e5` that line said something else entirely and `:44` recorded R4-1 as
   **2.750 FAIL**, not a pass. This is the AF-2 failure class aimed at a document instead of
   code, and it survived a blind review.
2. **A hidden ungoverned constant.** R4-8's §3.2 UI string asserted a 48 kHz sample rate with no
   constant, no rationale, and no ledger row — while no shipping source under `crates/*/src/`
   defines a sample rate at all. Now `BOUNCE_FALLBACK_SAMPLE_RATE` with a scheduled row; §7.2
   went from two rows to three.
3. **An overclaimed citation with wide blast radius.** A 256-frame figure was attributed to the
   macOS qualification table, which **has no block-size and no sample-rate column**. It sat in
   three sections and would have entered `requirements-ledger.md` verbatim.

**Two rules were generalized rather than patched**, and both are worth carrying forward:

- *De-duplicate the specimen, never the last independent instrument.* Two independently written
  hash walks agreeing over one fixture is verification; two independently maintained fixtures is
  drift that fires the live/offline alarm for reasons unrelated to the engine.
- *Do not cite a mutable tracker for durable facts.* R4-4 now carries **no** `manifest.md` line
  citation; sibling status is evidenced from source, because the manifest is rewritten every time
  any sibling advances.

Two remediations also **removed** numbers rather than deriving them — R4-6 deleted an unargued
1-ULP tolerance by proving bit-exactness, and withdrew a test rather than invent a baseline to
save it. Deleting an unrationalized bound is the cleanest way to satisfy PROD-003.

**Process result: the body-first rule held under test.** Two agents were killed mid-edit during
this phase. Both left honest partial states — bodies edited, terminators intact, headers still
reading the prior iteration — and both resumed cleanly. The rule was written after one incident
where the reverse order left a spec asserting an unapplied fix; it has now paid for itself twice.

## Recommended next steps

1. **Answer D-R4 before anything else.** R4's exit is a ten-row conjunction whose own text says
   "None is optional", and the Linux row cannot close on hardware that does not exist. Option
   (b) — exiting on macOS evidence again — would make R4 the **second consecutive milestone** to
   defer decision 1's co-first-class commitment at the point it could have been discharged. That
   deserves a decision row saying so, not a default.
2. **Answer D-R3 and R4-7's Q2.** Each blocks a specific implementation.
3. **Then implement in dependency order** — R4-1 first, since it makes every later slice
   observable, exactly as the tree says.
4. **Do not treat these scores as implementation evidence.** The state machine's implementation
   stages are untouched for all nine features. `./spectre` still produces no sound.

## Process notes worth keeping

- **Usage caps were the dominant failure mode**, not technical error: sixteen agent
  terminations across eight days. The single change that mattered was instructing every agent to
  **write its artifact to disk before polishing**. Before that instruction, a five-agent dispatch
  lost everything; after it, a four-agent dispatch lost nothing.
- **One remediation was lost mid-edit** and left R4-8 asserting a fix it had not applied —
  a document lying about its own contents. The fix: remediations now edit **body first, header
  last**, because the reverse order is what manufactured the lie.
- **Two stub scorecards** left by killed agents turned out to carry real findings, including
  R4-5's lane-capacity defect. Deleting them as noise would have thrown that away.
- **A reviewer caught itself** about to record an execution it had not performed. Its scorecard
  now carries a provenance correction and independently established ground truth.
