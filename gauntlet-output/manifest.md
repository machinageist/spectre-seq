<!--
Author: Jeff
Date: 2026-08-14
Description: Per-feature gauntlet state tracker and dated run history
Notes: The manifest and HANDOFF.md together define the next executable action
-->

# Gauntlet Manifest — Spectre

- **Status:** accepted
- **Last verified:** 2026-08-14
- **Scope:** state of every feature in `feature-tree.md` across the spec and implementation stages
- **Decision authority:** Jeff
- **Upstream sources:** `criteria.md`, `feature-tree.md`
- **Downstream dependents:** `HANDOFF.md`, dispatch decisions
- **Supersedes:** the unrecorded 2026-08-12 run
- **Superseded by:** none
- **Open decisions:** D-R1 and D-R2 in `decisions-needed.md`; neither blocks the R4 loop
- **Known gaps:** R4-1 remediation 1 was cut off by a usage cap before editing; eight features remain unspec'd

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
| R4-1 | live-audio-wiring | spec-remediation-1 | `specs/R4-1-live-audio-wiring.md` | `spec-scorecards/R4-1-live-audio-wiring-scorecard.md` | 2.750 — **FAIL (AF-2)** | 1 | 0 | **re-dispatch remediation after 05:30 cap reset**; spec is still iteration 1, unedited |
| R4-2 | runtime-parameter-seam | pending | — | — | — | 0 | 0 | author spec after sign-off |
| R4-3 | linux-device-qualification | pending · **hardware-blocked** | — | — | — | 0 | 0 | spec authorable now; execution needs a Linux box |
| R4-4 | track-model | pending | — | — | — | 0 | 0 | author spec after sign-off |
| R4-5 | midi-clips | pending | — | — | — | 0 | 0 | author spec after sign-off |
| R4-6 | first-devices | pending | — | — | — | 0 | 0 | author spec after sign-off |
| R4-7 | project-persistence | pending | — | — | — | 0 | 0 | author spec after sign-off |
| R4-8 | offline-bounce | pending | — | — | — | 0 | 0 | author spec after sign-off |
| R4-9 | e2e-and-qa | pending | — | — | — | 0 | 0 | spec last; depends on all leaves |

Nine features, zero spec'd, zero passed, zero escalated.

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
2. **Invented APIs** — `AppModel::add_track()`, `Track::arm()`, `Track::mute()`, and a
   track list sidebar, all described as partially existing. None exist. **AF-2.**
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

**2026-08-15 00:19 — R4-1 spec agent terminated on a session cap** ("resets 12:30am
America/Los_Angeles") before writing anything. `specs/` was unchanged, so there was no
partial artifact to detect or discard. This is the same hard-cap failure mode recorded
in the mg-server manifest for 2026-08-07 and 2026-08-08; the difference is that those
runs lost completed work, and this one lost none. Re-dispatched at 00:31 after reset.
