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
- **Open decisions:** criteria and tree sign-off, both blocking Phase 1
- **Known gaps:** no spec has been authored under version 1 criteria

**Run:** 2 (run 1 discarded — see run history)
**Criteria version:** 1 (2026-08-14, `proposed`)
**Status:** Phase 0 artifacts drafted; **blocked on Jeff's sign-off** of `criteria.md`
and `feature-tree.md`. No spec may be dispatched before both are `accepted`.
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
| R4-1 | live-audio-wiring | pending | — | — | — | 0 | 0 | author spec after sign-off |
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
completion extension the older local copy lacks. Awaiting sign-off.
