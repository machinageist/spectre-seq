<!--
Author: Jeff
Date: 2026-08-14
Description: Per-feature gauntlet state tracker and dated run history
Notes: The manifest and HANDOFF.md together define the next executable action
-->

# Gauntlet Manifest — Spectre

- **Status:** accepted
- **Last verified:** 2026-08-15
- **Scope:** state of every feature in `feature-tree.md` across the spec and implementation stages
- **Decision authority:** Jeff
- **Upstream sources:** `criteria.md`, `feature-tree.md`
- **Downstream dependents:** `HANDOFF.md`, dispatch decisions
- **Supersedes:** the unrecorded 2026-08-12 run
- **Superseded by:** none
- **Open decisions:** D-R1 and D-R2 in `decisions-needed.md`; neither blocks the R4 loop
- **Known gaps:** eight features remain unspec'd; R4-1 has passed its spec gate but no implementation exists

**Run:** 2 (run 1 discarded — see run history)
**Criteria version:** 1 (2026-08-14, `accepted`)
**Status:** Phase 1 dispatching. Criteria and tree accepted; benchmark set locked to
Ableton Live, Logic Pro, Serum 2, Phase Plant, and VCV Rack 2.
**Batch policy:** concurrency 3, per `feature-tree.md` §"Dispatch order".

## State machine

```text
pending → spec-in-progress → spec-review → spec-remediation-{1,2,3}
        → spec-pass | escalated
        → implementation-in-progress → implementation-review
        → implementation-remediation-{1,2,3}
        → slice-pass | escalated → integration-review → complete
```

A spec pass alone never proves implementation. Only a verifier advances a feature;
an author never verifies its own artifact.

## Features

| ID | Name | State | Spec | Scorecard | Score | Spec iter | Impl iter | Next action |
|---|---|---|---|---|---|---|---|---|
| R4-1 | live-audio-wiring | **spec-pass** | `specs/R4-1-live-audio-wiring.md` | iter 1: `…-scorecard.md` (2.750 FAIL AF-2) · iter 2: `…-scorecard-iter2.md` (2.950 PASS) | **2.950** | 2 | 0 | ready for implementation; two Priority 2/3 items are optional |
| R4-2 | runtime-parameter-seam | **spec-pass** | `specs/R4-2-runtime-parameter-seam.md` (1,686 lines) | `spec-scorecards/R4-2-runtime-parameter-seam-scorecard.md` | **3.000** | 1 | 0 | passed at iteration 1; **D-R3 blocks implementation** |
| R4-3 | linux-device-qualification | spec-review · **hardware-blocked** | `specs/R4-3-linux-device-qualification.md` (1,145 lines) | pending | pending | 1 | 0 | **awaiting blind verification** — first verifier died on a cap before writing |
| R4-4 | track-model | spec-review | `specs/R4-4-track-model.md` (1,610 lines) | pending | pending | 1 | 0 | **awaiting blind verification**; its §7.1 refused a false claim in `criteria.md` and was right |
| R4-5 | midi-clips | pending | — | — | — | 0 | 0 | batch 3 |
| R4-6 | first-devices | pending | — | — | — | 0 | 0 | batch 3 |
| R4-7 | project-persistence | **spec-pass** | `specs/R4-7-project-persistence.md` (1,452 lines) | `spec-scorecards/R4-7-project-persistence-scorecard.md` | **3.000** | 1 | 0 | passed at iteration 1; **Q2 (CORE-003's verified status) needs Jeff before it lands** |
| R4-8 | offline-bounce | pending | — | — | — | 0 | 0 | batch 3 |
| R4-9 | e2e-and-qa | pending | — | — | — | 0 | 0 | batch 4; spec last, depends on all leaves |

Nine features: one spec'd and passed, four spec'd and awaiting blind verification, four
unstarted, zero escalated.

R4-3 was pulled forward out of batch 4 because `feature-tree.md` states its spec is
authorable at any point — only its *execution* needs Linux hardware — so it carries no
dependency on the batch 2 features and gains nothing from waiting.

**Scorecard naming deviation, recorded deliberately.** `templates/UNIVERSAL-GAUNTLET.md`
names one scorecard per feature (`{feature-id}-scorecard.md`). R4-1's iteration 2 was
written to `{feature-id}-scorecard-iter2.md` instead, because overwriting iteration 1
would destroy the record of what failed and why — which in a run whose whole purpose is
evidence integrity is the wrong trade. Later features follow the same rule: one file per
iteration that produced a verdict.

## Run history

**2026-08-11 15:06 → 22:18.** `gauntlet-universal/` was copied into the repo and then
duplicated as `gauntlet-active/`. The two directories are byte-identical; `diff -rq`
reports no differences. Both are gitignored tooling copies. The tracked, authoritative
templates now live in `templates/`.

**2026-08-12 02:38 — run 1, discarded.** A Hermes session (`20260811_142402_4918b8`,
recorded in the gitignored `resume-commands`) wrote a single spec,
`specs/track-management.md`, and stopped. Phase 0 never ran: there was no
`criteria.md`, no `feature-tree.md`, and no manifest, so the spec was authored against
no graded standard and no confirmed feature.

The file was unusable on three independent counts, each of which the version 1
criteria now catch:

1. **Corrupted encoding.** The body was a JSON-escaped string written to disk —
   literal `\n` sequences instead of newlines from line 9 onward, terminating in a
   stray tool-response fragment (`", "total_lines": 261, "file_size": 7987, …`).
2. **Invented APIs** — `Track::arm()` and `Track::mute()`, described as partially existing.
   Neither exists. **AF-2.**
   > **Corrected 2026-08-15.** This entry originally also named `AppModel::add_track()` and
   > a track list sidebar as invented, and said "None exist." That was wrong.
   > `AppModel::add_track` exists at `crates/spectre-app/src/lib.rs:405–422` and has since
   > commit `6c397d9`; the sidebar exists at `crates/spectre-app/src/main.rs:115–160`; two
   > tests cover `add_track` at `crates/spectre-app/tests/app_model.rs:33–49`. The AF-2
   > finding survives on `Track::arm`/`Track::mute` alone, and the spec's disposition is
   > unchanged — it also failed on AF-5 and on corrupted encoding. See the corrected passage
   > in `criteria.md`, which carried the same error.
3. **A default shortcut map** (`⌘T` / `⌘U` / `⌘M`), which the workflow field study
   explicitly lists among conclusions its evidence cannot support. **AF-5.**

Its subject — track management — is real R4 work, but unstarted: `NEXT.md` slice 4
lists "Introduce the track model" as pending. It is now **R4-4**, and its spec starts
from an empty §7.1.

**Disposition:** deleted 2026-08-14. Nothing was salvageable; the record is this entry.
Git history retains the file.

**2026-08-14 — run 2, Phase 0.** Restarted from Phase 0 with the tree derived from the
accepted R4 slice queue rather than discovered from source. `gauntlet-output/` became
tracked so run state survives a fresh clone; the two tooling copies stay ignored.
Templates were seeded from the model-agnostic GeistScope set, which carries the
completion extension the older local copy lacks.

**2026-08-14 — benchmark set locked.** Jeff named Ableton Live, Logic Pro, Serum 2,
Phase Plant, and VCV Rack 2 as the AAA bench. An audit of citable evidence found it
uneven enough to change the grading: Ableton 85 `OBS-` records, Phase Plant 11, VCV 6,
Serum 2 two, Logic Pro none. Criterion 2G and D-R1 record the consequence — cite what
exists, name what does not. Criteria and tree moved to `accepted`; D-G1 closed with
weights unchanged.

**2026-08-15 00:46 → 01:15 — R4-1 authored, failed blind review, remediation cut off.**
The spec was written (67 KB, every template section filled) and blind-reviewed. It
scored a composite **2.750** and cleared every numeric threshold — all four lenses
above 2.00, no criterion at 0 or 1 — and still **failed on AF-2**.

The finding, independently confirmed against source before remediation was dispatched:
§4.1 and §7.2 claimed `rt_guard.rs` scans `bridge.rs`, `control.rs`, `spsc.rs`, and
`midi.rs`, all untouched. `crates/spectre-audio/tests/rt_guard.rs:294–297` actually
scans `src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, **`src/null.rs`** — and the
spec's own §4.1 table modifies `null.rs` two rows above that sentence. The spec named
a file that is not scanned, missed the one that is, and built its central RT-001 safety
argument on the error. The path was also wrong: the file is under `tests/`, not `src/`.
The reviewer additionally found an ordering guarantee in §4.3 that does not follow from
its premises, and two of thirteen proposed tests that cannot compile.

Everything else held: roughly seventy citations verified to the exact line, both
claimed API absences confirmed genuine, lenses 2 and 3 perfect at 3.00. This failed on
evidence integrity, not quality — which is precisely the failure class the version 1
criteria were written to catch, working on its first use.

**Remediation iteration 1 was terminated by a usage cap** ("resets 5:30am") after
confirming the scan list but before editing. The spec is therefore **unmodified at
iteration 1** and the fix is still owed. No partial edit landed; verified by mtime
(00:46, unchanged) and by the file still reading `Iteration: 1`.

**2026-08-15 ~01:05 — Serum 2 research pass terminated by the same cap**, after
writing `docs/02-reference-research/serum-2-observations.md` (~250 records) but before
writing any source-ledger records. Quarantined under D-R2: its primary source is an
unrecorded 354-page user guide whose authorization status is the very open decision
`serum-2.md` has carried since July. `source-ledger.json` and `serum-2.md` were never
modified and remain clean.

**2026-08-15 ~13:15 — batch 2 dispatch lost five agents to a usage cap; nothing was
written and nothing was lost.** Authors for R4-2, R4-4, R4-7, and R4-3 were dispatched at
concurrency 4, plus a fifth agent on the D-MM4 loudness-standards reading pass. All five
were terminated by the same account-level session cap ("resets 5:50pm America/Los_Angeles")
within minutes of launch.

**Disposition: clean.** Verified after termination — `gauntlet-output/specs/` still
contained only `R4-1-live-audio-wiring.md` and `README.md`; `source-ledger.json` parsed
cleanly at 59 records with zero duplicate `source_id`s and only `SRC-ITU-BS1770-5` in the
standards class; `loudness-standards-observations.md` did not exist. **No partial artifact
landed, so none had to be quarantined or discarded.** This is the fourth cap termination
recorded today (00:19, ~01:05, and the earlier 05:30 reset), and the first where the
briefs were written to expect it — every author was told to land a complete draft before
polishing, precisely because a cap can arrive at any moment.

The loudness agent reported having retrieved all four standards documents in full before
dying at the write step, and one author confirmed a finding worth keeping. That finding was
**re-verified independently rather than trusted**: `crates/spectre-app/Cargo.toml:5`
declares a `spectre-project` dependency, and no file under `crates/spectre-app/src/` or
`crates/spectre-app/tests/` references `spectre_project` at all. The app declares the
persistence crate and never uses it — a clean statement of how far R4-7 is from existing,
and it was handed to the re-dispatched author as established fact.

**Re-dispatched at 18:16 PDT**, after the reset, with the same five briefs.

**2026-08-15 18:29–18:35 — four specs landed; the second cap arrived at the reporting
step, not the writing step.** R4-3 (1,145 lines), R4-2 (1,686), R4-4 (1,610), and R4-7
(1,452) were all written to disk, complete — eight of eight template sections each, Jeff's
header block, `End of spec.` terminator, `Iteration: 1`. The cap ("resets 11:10pm") then
killed three of the four authors mid-report and killed the R4-3 blind verifier before it
wrote anything.

**This is the difference the brief change made.** The 13:15 dispatch lost everything
because agents were killed while still reading. The 18:16 briefs told every author to land
a complete draft before polishing, and three of the three that died had already written
their file. The instruction is cheap and it is now standard for every dispatch in this
loop.

**The loudness-standards pass completed in full** and is recorded under its own entry
below. Its one flagged risk — that re-dumping `source-ledger.json` with
`ensure_ascii=False` had unescaped em-dashes across the concurrently-added FabFilter and
Ozone records — was checked by parsing HEAD and the working tree and comparing records
semantically: 62 records, zero duplicates, **zero pre-existing records changed, none
lost**. The concern was textual noise only.

**2026-08-15 — R4-2 and R4-7 verified and passed at 3.000 each, and the review found an
error in `criteria.md` itself.** Both were graded by a reviewer that did not author them and
never received their authors' reasoning — both authors were killed mid-report by the cap, so
their findings were never transmitted. Evidence integrity was checked exhaustively; quality
lenses were sampled, and both scorecards mark which criteria rest on a sample rather than
implying full coverage.

**R4-2 (3.000)** absorbed both of R4-1's failures at iteration 1: it states the `rt_guard`
scan list correctly, and rather than claiming an untouched scanned set it observes that its
own change list modifies two of the four, argues from edit content, and **adds its new
callback-reachable module to the scan array** — a move R4-1 never had to make. Forty-six
citations were opened; zero false. It also disclosed that the shipped `Gain` has no smoothing
state despite `dsp-device-io.md:94` and `:104` asserting twice that it does, and routed that
to **D-R3** rather than resolving a conflict between an accepted document and shipped code.
Confirmed independently: `Gain` is a single `f32` with an instantaneous clamped setter.

**R4-7 (3.000)** was equally clean on citations and correctly separates designing for
crash-durability from claiming its evidence, which is R5's. Its Q2 is the sharp one: a schema
bump may disturb an acceptance test belonging to CORE-003, a **`verified`** requirement, and
landing the slice without answering it would quietly downgrade a verified requirement.

**The incidental finding outranks both.** R4-7 cited `AppModel::add_track` at
`crates/spectre-app/src/lib.rs:405`, which contradicted this manifest and `criteria.md` —
both of which recorded that method as an invented API in the discarded 2026-08-12 spec, with
the words "None exist." `git log -S` puts `add_track` in commit `6c397d9`, before the
discarded spec and before the criteria were accepted; the track list sidebar and two covering
tests exist as well. **An accepted grading standard was misstating the codebase inside its own
AF-2 illustration.** Both documents were corrected in place rather than silently edited. The
R4-4 author, which had been briefed with the false claim, verified it independently and
refused it in writing — which is the behavior the whole loop is built to produce.

**2026-08-15 — D-MM4's research half discharged.** ITU-R BS.1770-5 (Annexes 1 and 2 in
full), EBU R 128, EBU Tech 3341, and EBU Tech 3342 were retrieved and read, producing 95
records in `docs/02-reference-research/loudness-standards-observations.md` — 70 `OBSERVED`,
12 `SOURCE-GAP`, 13 `SPECTRE-CANDIDATE` — and four `claims-extracted` ledger records.
`SRC-ITU-BS1770-5` legitimately moved off `inventory-only`.

**MM-AF-5 was narrowed as a direct result**, which is the point of the exercise: four of
its six prohibitions lifted, two stand. Conformance claims stand because the EBU
minimum-requirement test-signal sets were never retrieved (`GAP-LOUDNESS-0008`) — reading a
specification is not passing it. Delivery targets stand because no streaming target exists
in any of the four documents (`GAP-LOUDNESS-0003`); the only citable target is R 128's
*broadcast* −23.0 LUFS / −1 dBTP.

Two findings outrank the coverage itself and are now carried into the criteria:
`GAP-LOUDNESS-0001` — BS.1770-5 gives K-weighting coefficients **for 48 kHz only**, making
any other rate an original Spectre engineering decision — and `OBS-T3341-020`, which
records that correct Integrated Loudness requires recomputation from stored per-block
history and therefore **collides head-on with RT-001/RT-002**. The second is now quoted in
criterion 1A as the sharpest available test of callback-path discipline.

Stale cross-references left behind by that pass were repaired in the same session:
`ozone.md` (four sites, plus `GAP-OZONE-0043` closed with two successor gaps),
`criteria-mixing-mastering.md` (MM-AF-5, the evidence inventory, criterion 1A, the header),
`MM-GAUNTLET-PROMPT.md`, and `decisions-needed.md`'s D-MM4 entry.

**2026-08-15 — R4-1 remediation 1 landed and passed blind re-verification at 2.950.**
Iteration 2 fixed all four Priority 1 defects, and each fix was checked against source by
a reviewer with no sight of the remediation. The RT-scan claim now matches
`rt_guard.rs:294–297` exactly at both sites, and — more valuable than the correction —
the RT-001 argument was **rebuilt** rather than patched: it no longer rests on an
untouched scanned set, but on the content of the `null.rs` edit, with a post-edit
`rt_guard` run promoted to a required gate in §5.2. The invalid note-ordering guarantee
was retracted in the spec's own text and replaced with the counted fail-closed behavior
the shipped code actually produces, traced through `graph/lib.rs:487–492` and
`bridge.rs:192–200/258/175–176`, pinned by a new test 14, with the behavior *change* routed
to §8 Q9 instead of asserted. Tests 3 and 9 compile: test 3 reads a new
`EngineParts::plan_max_frames` rather than an accessor on `RenderBridge`, and `LiveEngine`
became generic over its stream so a test can hold `LiveEngine<NullStream>` and pump it.
All four Priority 2 items were fixed too, including the `requirements-ledger.md` rows
PROD-003 compels.

Lens 1 scored 2.857 rather than 3.000 on a **new** finding, recorded as Priority 2 and not
a blocker: §4.4's binding rule — the spec's own "one genuinely new correctness hazard" —
has no automated coverage, because it lives in `main.rs`, which §4.1 correctly establishes
is unreachable from tests. Test 7 gestures at it with an assertion that cannot fail. The
spec is honest about this (§5.3 defers UI automation to R4-9; §5.4 checks it by hand), so
it is a stated limit, not a hidden one.

The re-verification was a delta review with a regression sample, and the scorecard says so
in its own summary: 31 source locations across nine files plus the vendored `eframe 0.32.3`
source were opened for everything the remediation touched, while iteration 1's ~70
already-verified citations were sampled (seven `OBS-` IDs, the `main.rs` literals, the FNV
constants, the offline fixture path) rather than re-read in full.

**2026-08-15 00:19 — R4-1 spec agent terminated on a session cap** ("resets 12:30am
America/Los_Angeles") before writing anything. `specs/` was unchanged, so there was no
partial artifact to detect or discard. This is the same hard-cap failure mode recorded
in the mg-server manifest for 2026-08-07 and 2026-08-08; the difference is that those
runs lost completed work, and this one lost none. Re-dispatched at 00:31 after reset.
