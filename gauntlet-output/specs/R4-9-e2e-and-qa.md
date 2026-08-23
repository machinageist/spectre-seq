<!--
Author: Jeff
Date: 2026-08-21
Description: R4-9 spec — the end-to-end fixture and the written manual QA protocol that R4's exit requires, defined over the union of the other eight leaves
Notes: This spec tests work that does not exist. All eight sibling leaves are spec'd and none is
  implemented; `./spectre` produces no sound of any kind today. R4-9 therefore specifies evidence,
  not behavior, and every assertion below is conditional on the slice that produces the thing it
  asserts. Two structural facts shaped it. First, there is no automated GUI-driving harness in this
  repository and R4-9 does not build one — `crates/spectre-app/tests/` contains exactly
  `app_model.rs` and `smoke_cli.rs`, and `main.rs:499-518` shows the only headless path is an early
  return before `eframe::run_native`. Second, R4's exit is a ten-row conjunction and one row —
  Linux device qualification — is blocked on hardware that has never existed in this project, which
  blocks R4-9's own Linux column just as hard. That conjunction is §8 Q1 and it is routed to Jeff,
  not answered here.
-->

# Spec: End-to-End Fixture and Manual QA Protocol

**Feature ID:** `R4-9` (`e2e-and-qa`)
**Parent feature:** `R4` Credible Alpha (root)
**Spec author agent:** gauntlet spec agent, R4-9 leaf
**Date:** 2026-08-21
**Iteration:** 1

- **Status:** proposed
- **Last verified:** 2026-08-21 (every source path below opened on branch `rename/geist-to-spectre`, working tree at commit `b5af060`; no file under `crates/` is modified in the working tree, so every code citation is both the tree and the commit)
- **Scope:** one automated end-to-end fixture that exercises the union of R4-1…R4-8 below the egui widget layer, and one written manual QA protocol with a per-run record format for everything the fixture structurally cannot reach. Out of scope: any change to the eight sibling features' designs, any new DSP, any GUI-automation framework, and the execution of the protocol on Linux
- **Decision authority:** Jeff
- **Upstream sources:** `docs/06-plans/current-milestone.md` §"Exit evidence" (`:79–90`) — the list this feature exists to make checkable; `docs/README.md` §"Conflict precedence" (`:23–35`) and §"Status vocabulary" (`:52–62`); `docs/01-requirements/requirements-ledger.md` RT-001 `:28`, RT-002 `:29`, RT-003 `:30`, TIME-002 `:37`, TIME-005 `:40`, CORE-001 `:46`, CORE-003 `:48`, CORE-004 `:49`, GRAPH-001 `:55`, PROD-003 `:64`; `docs/01-requirements/decision-gates.md` rows 1 `:25`, 8 `:32`, 13 `:37`, 15 `:39`, 16 `:40`, 17 `:41`, 19 `:43`, 20 `:44`, 21 `:45`, 22 `:47`, 23 `:49`; `docs/01-requirements/traceability.md`; `docs/status/STATUS.md`; `docs/status/NEXT.md` slice 9 (`:31`); all eight sibling specs in `gauntlet-output/specs/`; their scorecards in `gauntlet-output/spec-scorecards/`
- **Downstream dependents:** `docs/06-plans/current-milestone.md` §"Exit evidence" row 7 (the fixture and protocol are its output artifacts), `docs/01-requirements/traceability.md` (R4's exit requires traceability to match implementation), `docs/status/STATUS.md`, and the R4 root umbrella spec if one is written
- **Supersedes:** none. There is no prior end-to-end or QA document in this repository; `docs/05-quality/` does not exist (`docs/README.md:50` — quality directories "are created only when they contain grounded contracts")
- **Superseded by:** none
- **Open decisions:** §8 Q1–Q12. **Q1 is the one that must be answered before this feature can produce a verdict at all**: what R4's exit means while the Linux qualification row is open. Q2 (D-R3), Q3 (R4-7's Q2 on a `verified` requirement), and Q4 (schema version) are sibling decisions this spec routes rather than assumes
- **Known gaps:** (a) the accepted research corpus contains **no citable observation** about how any of the five benchmark products tests itself, structures an internal QA protocol, defines an acceptance fixture, or records a release-qualification run. QA methodology is a surface with zero benchmark evidence and it is named as a gap in §Appendix A rather than filled in from recollection; (b) **the Linux half of this protocol is hardware-blocked by exactly the missing hardware that blocks R4-3**, and more severely, because R4-3 needs one command on that host while R4-9's manual half needs an operator working through a rendered UI with an audio interface attached; (c) **nothing this spec asserts can be run today** — all eight sibling features are spec'd and none is implemented, so §5's new tests describe files that do not exist against APIs that do not exist; §5 marks every item accordingly; (d) the fixture's byte content cannot be authored until R4-4's track schema, R4-5's clip schema, R4-6's device descriptors, and R4-7's envelope version exist, so §4.2 specifies its **content** exactly and its **encoding** conditionally.

This spec is subordinate to the conflict precedence in `docs/README.md:23–35`. It proposes
evidence artifacts. It does not amend an accepted requirement, decision row, or architecture
contract, and it does not amend any sibling spec. Where it proposes new normative rows it says
so and routes them to §8 rather than asserting them.

**Standing constraint, stated once and binding on every section below.** As of 2026-08-21
`./spectre` produces no sound of any kind. `crates/spectre-app/Cargo.toml:12–18` lists
`eframe`, `spectre-core`, `spectre-dsp`, `spectre-project`, `serde`, and `serde_json`, and
does not list `spectre-audio`, so no audio backend, control transport, or callback bridge is
reachable from the launchable binary (`docs/status/STATUS.md` §"Repository state" records the
same fact in its own words). R4-1 is the slice that changes that and it is spec'd, not built.
Every sentence below that describes audible output, a track, a clip, a saved file, or a bounce
is describing a slice that has not landed. No section claims otherwise.

---

## 1. Purpose

### 1.1 One-sentence job

Give R4 a single command and a single written procedure that together answer the question
"is the credible alpha actually credible?" with recorded evidence instead of eight separate
slice-level assertions and a feeling.

### 1.2 Why it matters

R4's exit evidence (`docs/06-plans/current-milestone.md:79–90`) is a ten-row conjunction.
Eight of those rows are owned by sibling features and each sibling proves its own row in
isolation: R4-1 proves the engine opens and hashes equal to the offline render, R4-2 proves a
parameter reaches a live processor, R4-4 proves three tracks sum, R4-5 proves a clip schedules
notes without a second event path, R4-6 proves two devices render deterministically, R4-7
proves a save round-trips, R4-8 proves a bounce equals the live path. Every one of those is a
test of one seam.

**A vertical slice is not the conjunction of eight seams tested separately.** The failure mode
this feature exists to catch is the one where each seam passes its own test and the composition
does not work — a clip that schedules correctly against R4-5's scheduler but whose note IDs
collide with the live ingress's IDs once both are running, a project that reloads with correct
identities but whose reloaded device values render a different hash, a bounce that matches the
live path on the fixture chain but not on the three-track chain the user actually has.
`docs/06-plans/current-milestone.md:87` asks for exactly one artifact against that risk: *"An
end-to-end fixture plus a written manual QA protocol both pass."*

The second half of that sentence matters as much as the first. The repository's automated
evidence stops at a hard boundary. `crates/spectre-app/tests/` contains exactly two files,
`app_model.rs` and `smoke_cli.rs`; `app_model.rs` tests `AppModel` as a pure data type with no
renderer (its own header comment at `:4` says "Pins behavior independently of egui rendering"),
and `smoke_cli.rs` runs the binary with `--smoke-test` and greps four substrings out of one
line of stdout (`:14–19`). Nothing in this repository has ever driven a rendered widget, and
nothing in it has ever listened to audio. R4-1's §5.3 states the same boundary and defers UI
automation here by name. Someone has to write down what a human checks, when, and what they
record — otherwise "the alpha works" is an assertion with no evidence class behind it, which
`docs/status/STATUS.md`'s own header prohibits.

### 1.3 Success signal

Two signals, deliberately separated because they have different evidence classes and a single
combined word would hide which half held.

**Automated:** `cargo test --locked -p spectre-offline --test e2e_alpha` passes, and its
central assertion is that one checked-in project file, loaded from disk, rendered offline and
driven through `RenderBridge` at the same block size on the same host, produces the **same**
FNV-1a hash on a render whose peak is nonzero — and that the same file saved, reloaded, and
rendered again produces that same hash a third time. The nonzero-peak condition is part of the
assertion because two silent buffers agree; that precedent is already set by the bridge
equivalence test recorded at `docs/06-plans/current-milestone.md:49`.

**Manual and recorded:** one completed run record in `docs/05-quality/r4-qa-records.md`,
carrying a per-platform outcome word, the environment fields §4.2 defines, and an explicit
verdict for every row of `docs/06-plans/current-milestone.md:79–90` including the rows that
did **not** run. A record with a blank Linux column is a valid record; a record that omits the
Linux column is not.

Neither signal exists today and neither can be produced today. §7.1 states why.

---

## 2. User Stories

> As **Jeff**, before I mark R4 exited, I want one command and one checklist that together
> touch every exit row, so that "credible alpha" is a recorded result rather than eight
> separate memories of things that worked once.

> As **Jeff**, when the end-to-end fixture fails, I want the failure to name which sibling
> feature's seam broke, so that I am debugging one slice instead of bisecting a milestone.

> As **the implementer of a later R4 slice**, I want to run the end-to-end fixture before I
> claim my slice landed, so that I find out I broke the composition on my own change rather
> than at milestone exit.

> As **a blind reviewer of this project's claims**, I want every QA run to leave a dated record
> naming the host, the commit, the outcome, and what was *not* checked, so that a claim of
> "R4 passed" can be traced to a specific machine on a specific day instead of to a document.

> As **a musician using assistive technology**, I want the manual protocol to include a
> keyboard-only pass and a screen-reader-labels pass whose results are recorded as findings,
> so that decision 17's beta gate (`docs/01-requirements/decision-gates.md:41`) arrives at R5
> with data rather than with a fresh audit from zero.

> As **Jeff, holding a macOS machine and no qualified Linux machine**, I want the protocol to
> refuse to aggregate a one-platform result into a milestone verdict, so that decision 1's
> co-first-class commitment (`docs/01-requirements/decision-gates.md:25`) cannot be discharged
> by silence.

> As **the operator running the protocol on a laptop with no audio interface attached**, I want
> the protocol to tell me which rows I cannot legitimately produce, so that I record
> `NOT RUN` for those rows instead of guessing at them from the built-in speakers.

---

## 3. UX Specification

### 3.1 Screen / view inventory

R4-9 introduces **no application screen, panel, modal, popover, drawer, or widget**. This is a
substantive design decision, not an omission, and §3.2 and §7.2 both follow from it: every
surface R4-9 touches is a terminal transcript or a Markdown document under version control.

| Surface | New / modified | Navigation path | Layout pattern |
|---|---|---|---|
| `cargo test` transcript for `e2e_alpha` | New test target; the runner is unmodified | `cargo test --locked -p spectre-offline --test e2e_alpha` | libtest stdout; a failing assertion carries the sibling feature ID in its message (§4.3) |
| `docs/05-quality/r4-qa-protocol.md` | **New file, and the first file in a new `docs/05-quality/` directory** (`docs/README.md:50` authorizes its creation only when it holds a grounded contract; this is that contract) | Read in an editor or on a code host | Markdown: numbered procedure, one manual-check table, one outcome-vocabulary table |
| `docs/05-quality/r4-qa-records.md` | New file | Same | Markdown: append-only, newest run last, one numbered block per run |
| `docs/06-plans/current-milestone.md` §"Exit evidence" | Modified — each of the ten rows gains a verdict marker written by the protocol, not by a slice author | Same | Existing bullet list |

**Why no application surface.** A QA affordance inside `./spectre` — a "run diagnostics"
button, a self-test panel — would be a new product surface that no exit row asks for, that no
accepted document authorizes, and that would itself need testing. `docs/00-product/vision.md:48`
sets R4's bar as "honest telemetry, no fake surfaces"; a diagnostics panel over eight
unimplemented features is the definition of a fake surface. R4-1 already specifies the honest
telemetry that belongs in the product (its engine cluster), and this feature reads that
telemetry rather than adding a second display of it.

### 3.2 Interaction flows

**Flow A — the automated fixture (primary).**

1. Operator runs the workspace gate first, unchanged and in order:
   `cargo fmt --all -- --check`; `cargo clippy --locked --workspace --all-targets -- -D warnings`;
   `cargo test --locked --workspace`. This is the gate named at `docs/status/STATUS.md`
   §"Validation" and enforced by CI at `.github/workflows/ci.yml:40–47`.
2. `cargo test --locked --workspace` runs `e2e_alpha` as part of the workspace suite; the
   operator may also run it alone as
   `cargo test --locked -p spectre-offline --test e2e_alpha -- --nocapture` to read its
   printed report line.
3. The test loads `crates/spectre-offline/tests/fixtures/r4-alpha.json` from disk, renders it
   offline, drives the identical schedule through `RenderBridge` against `NullBackend`,
   compares hashes, saves and reloads the project, renders a third time, and asserts the
   telemetry counters §5.2 lists.
4. **Branch — hash mismatch.** The assertion message names which pair disagreed
   (offline↔live, or pre-save↔post-reload) and prints both hashes and both peaks. It does not
   print the sample data; §4.7 explains why.
5. **Branch — nonzero `plan_errors`, `contaminated_nodes`, `parameters_pending`,
   `notes_deferred`, or `frame_capacity_rejections`.** Each is a separate assertion with its
   own message naming the sibling feature whose seam it belongs to, so the transcript
   identifies the slice without bisection.
6. **Branch — the fixture file is missing or fails semantic validation.** The test fails with
   the decode error verbatim. It does **not** fall back to a generated in-memory project;
   a fixture that silently regenerates itself proves nothing about the file on disk.

**Flow B — the manual protocol.**

1. Operator opens `docs/05-quality/r4-qa-protocol.md` and starts a new record block in
   `docs/05-quality/r4-qa-records.md`, filling the environment fields **before** running
   anything, so the record cannot be back-filled to match a result.
2. Operator runs Flow A and transcribes its outcome into the record.
3. Operator runs `./spectre` and works the manual-check table in §5.4 top to bottom, writing
   `PASS`, `FAIL`, `NOT RUN`, or `BLOCKED` against every row, with a note for anything that is
   not `PASS`.
4. **Branch — a row cannot be produced on this host** (no audio interface, no second output
   device, no Linux machine): the row is recorded `NOT RUN` with the reason. It is never
   recorded `PASS` by inference from a related row.
5. Operator applies §5.5's outcome rules **per platform** and writes one outcome word per
   platform. There is no combined word (§5.6).
6. **Branch — any row is `FAIL`.** The record is committed as-is, the corresponding row in
   `docs/06-plans/current-milestone.md` §"Exit evidence" stays open, and the finding is raised
   against the owning sibling feature. A failing record is never deleted or amended in place;
   a re-run appends a new block.

**Cues.** No haptic, sound, or animation cues are introduced. The only sound involved is the
audio under test, and §5.4 rule 1 requires it to be produced at low system volume for the
reason R4-1 §5.4 gives.

### 3.3 Layout descriptions

**`docs/05-quality/r4-qa-protocol.md`** — component hierarchy, top to bottom:

1. Jeff's header block and the metadata block `docs/README.md:64–66` requires.
2. A standing-constraint paragraph naming what is unimplemented at the time of writing.
3. §Preconditions — the environment fields to fill before running.
4. §Procedure — Flow B as a numbered list.
5. §Manual checks — the single table reproduced from §5.4 of this spec.
6. §Outcome rules — the table reproduced from §5.5.
7. §What a PASS authorizes — the list reproduced from §5.6.

**`docs/05-quality/r4-qa-records.md`** — one numbered block per run. Fields, **reusing
R4-3 §3.3's E1–E10 environment schema rather than inventing a second one**, as R4-3's own
downstream-dependents line asks:

```
[Q{n}] R4 QA run, {ISO date} — platform: macOS | Linux — outcome: PASS | FAIL | INCONCLUSIVE | REFUSED
  E1  — distribution/OS and release
  E2  — kernel or Darwin release and machine architecture
  E3  — audio subsystem version (ALSA: /proc/asound/version; macOS: CoreAudio, OS build)
  E4  — sound server mediating the device, if any, and how that was determined
  E5  — hardware: interface make/model and connection type (onboard / USB / PCI / Thunderbolt)
  E6  — device key exactly as the app reports it
  E7  — Spectre commit SHA, rustc version, cargo features in effect
  E8  — requested geometry and granted geometry, or "granted: not recorded" with the reason
  E9  — workspace-gate result immediately before the run, verbatim summary line
  E10 — verbatim panic message, driver error string, or "none"
  Q-A — outcome of Flow A (the automated fixture), verbatim libtest summary
  Q-B — the §5.4 manual-check table with one verdict per row
  Q-C — one line per exit row of docs/06-plans/current-milestone.md:79-90, with its verdict
  Q-D — what was NOT checked on this run, and why
```

**Data sources.** Every cell comes from the transcript, from a named shell command on the host,
or from the operator's direct observation of the running application. **No cell is sourced from
`docs/status/STATUS.md`, from a sibling spec, or from a previous run record.** That rule exists
because this run is the first evidence in the project whose subject is a composition rather than
a seam, and a record that quotes documentation is a document reviewing itself.

**Empty state.** Before the first run, `r4-qa-records.md` contains its header block and the
single line `No run has been performed. R4's exit row 7 is open.` That is the correct empty
state and must not be replaced with a placeholder row that looks like a measurement — the same
rule R4-3 §3.3 applies to the milestone's `| Linux | not run |` row
(`docs/06-plans/current-milestone.md:114`).

### 3.4 Input & gestures

- **Keyboard / click:** Flow A is one shell command. Flow B drives the existing `./spectre`
  surfaces that R4-1, R4-2, R4-4, R4-5, R4-6, R4-7, and R4-8 specify; R4-9 adds no control of
  its own and therefore no new interaction.
- **Specialized input:** N/A — no stylus, controller, voice, or camera surface exists anywhere
  in this repository.
- **Keyboard shortcuts:** **none are defined and none may be.** AF-5 and
  `docs/02-reference-research/workflow-field-study/product-implications.md:96` prohibit fixing
  a default shortcut map at the current evidence level. The protocol's keyboard-only pass
  (§5.4 row 12) records *whether every command was reachable from the keyboard*, which is
  decision 17's question, and deliberately records **no** key assignment, because writing the
  keys down in an accepted quality document would fix a default map by the back door.
- **No gesture count and no time budget.** The protocol does not state how many actions a row
  takes or how long a run should take, and §5.5 contains no duration threshold. Same source:
  `product-implications.md:97` prohibits a gesture-count or time budget at this evidence level.
  This is a deliberate omission and the protocol says so in its own text, so a future operator
  does not add one thinking it was overlooked.
- **Responsive behavior:** the transcript wraps at 80 columns; the records file is Markdown read
  in an editor or on a code host. No responsive work is required.

### 3.5 Transitions & animation

N/A — the surfaces are a terminal transcript and two Markdown documents. There is no navigation
transition and no in-view state change, so there is no reduced-motion alternative to specify.
This is a real N/A and not a deferral: adding animation here would mean inventing a surface
§3.1 deliberately declines to build. The reduced-motion behavior of the *application*'s own
transitions is R4-1's and R4-4's to specify; §5.4 row 11 records whether the operator observed
any motion that has no reduced-motion path, as a finding rather than as a gate.

### 3.6 Error states

| # | Trigger | Presentation | Recovery | Data loss risk |
|---|---|---|---|---|
| E-1 | Fixture file missing or unreadable | libtest failure naming the path | Restore the file; the test never regenerates it (§3.2 A6) | None |
| E-2 | Fixture fails semantic validation after decode | libtest failure carrying the decode error verbatim | Fix the fixture or the schema; if the schema moved, §8 Q4 governs | None |
| E-3 | Offline↔live hash mismatch on the same host | libtest failure printing both hashes and both peaks | A composition defect. Route to the owning sibling; do not adjust the fixture to make it pass | None |
| E-4 | Pre-save↔post-reload hash mismatch | libtest failure naming R4-7 | Persistence changed what the project computes — the exact failure R4-7's §1.3(c) exists to catch | None in the test; **in the product this class of defect is silent data corruption**, which is why it is asserted here as well as in R4-7 |
| E-5 | `parameters_pending > 0` after the run | libtest failure naming R4-2 | R4-2's acceptance criterion is `pending == 0`; a nonzero value means the seam is not consuming the lane | None |
| E-6 | `contaminated_nodes > 0` or `plan_errors > 0` | libtest failure naming RT-003 / the plan | A DSP defect in R4-6's devices or R4-4's summing. Never "expected on this host" | None |
| E-7 | `frame_capacity_rejections > 0` against `NullBackend` | libtest failure | The fixture drove a block wider than the plan's capacity — a harness bug, since the null backend's block geometry is chosen by the test. Distinct from the live-driver case R4-3 §3.6 E-9 covers | None |
| E-8 | Flow A passes and Flow B's audibility row fails | **No automated signal at all** | The operator records `FAIL` on that row. This is the highest-value error state in the feature: it is exactly the case where every hash agrees and the user hears nothing | None |
| E-9 | The host has no audio interface, or no second output device to unplug | No signal; the operator must notice | Record the affected rows `NOT RUN` with the reason (§3.2 B4). A built-in speaker is recorded as such in E5, and rows that depend on an interface stay `NOT RUN` | None, but the record must not imply coverage it lacks |
| E-10 | No Linux host with a real ALSA device is available | No signal | Record the whole Linux platform column `NOT RUN`, cite decision 23 (`decision-gates.md:49`), and **do not** write a combined outcome word (§5.6). §8 Q1 is the governing question | None, but this is the state the feature is in today and expects to be in at first run |
| E-11 | A sibling feature is unimplemented at run time | Flow A fails to compile, or Flow B has rows with no surface to check | Record `BLOCKED` with the sibling's feature ID. **This is the state of every row today** (§7.1) | None |

### 3.7 Accessibility

The protocol's own surfaces are plain Markdown and a terminal transcript: no color carries
meaning, no state is communicated by color alone, and text scales with the reader's editor or
terminal. Focus order and keyboard navigability are trivially satisfied because there are no
interactive elements.

The substantive accessibility content of this feature is what it **records about the
application**. Decision 17 (`docs/01-requirements/decision-gates.md:41`) makes keyboard-complete
operation and screen-reader labels a beta gate with "a scoped audit at R4"; R4-9 is the only
R4 feature with a written procedure, so the scoped audit belongs here. §5.4 rows 12 and 13
require the operator to attempt every step of Flow B (a) keyboard-only and (b) with the
platform screen reader active (VoiceOver on macOS, Orca on Linux), and to record which steps
were impossible and which controls announced no label.

Three constraints on that audit, each stated so the record cannot overclaim:

1. **It records findings, not a verdict.** Decision 17 gates beta, not R4. A run with
   unlabeled controls is not an R4 `FAIL`; it is an R4 record with accessibility findings,
   which is what "scoped audit" means at this milestone.
2. **`eframe`'s accessibility feature is not enabled today.** `crates/spectre-app/Cargo.toml:13`
   declares `eframe = { version = "0.32.3", default-features = false, features =
   ["default_fonts", "glow"] }` — an explicit `default-features = false` with two features named,
   neither of which is an accessibility feature. R4-7's §8 Q5 already asks whether to enable it
   at R4. Until Jeff answers, the screen-reader pass is expected to find little to announce, and
   the record must say that the cause is a build configuration rather than a labeling defect.
3. **It designs nothing away.** Per criterion 3F this feature must not foreclose the beta gate.
   Recording per-step keyboard reachability is the input that makes the beta work scopable;
   recording a shortcut map would violate AF-5, and §3.4 says why it does not.

---

## 4. Implementation Specification

### 4.1 Architecture placement

Two artifacts, in two places, for two reasons.

**The automated fixture lives in `crates/spectre-offline/tests/e2e_alpha.rs`, with its project
file at `crates/spectre-offline/tests/fixtures/r4-alpha.json`.**

The placement is forced by the dependency graph, not chosen for taste. A single test binary
that walks the whole slice must be able to name `spectre_app` (the model the user edits),
`spectre_project` (save/reload), `spectre_offline` (the offline render and the FNV walk),
`spectre_graph` and `spectre_dsp` (the plan and the devices), and `spectre_audio` (the bridge).
Today:

- `crates/spectre-offline/Cargo.toml:12–18` declares `spectre-core`, `spectre-dsp`,
  `spectre-graph`, `spectre-project`, `serde`, and `serde_json` as dependencies, and `:20–21`
  declares `spectre-app` as a **dev-dependency**. So `spectre-offline`'s test targets can
  already name five of the six.
- The missing one is `spectre-audio`. `crates/spectre-audio/Cargo.toml:23–24` declares
  `spectre-offline` as *its* dev-dependency, so adding `spectre-audio` to `spectre-offline`'s
  dev-dependencies creates a **mutual dev-dependency cycle**.
- **That cycle is legal and was verified rather than assumed.** A two-crate probe workspace was
  built outside this repository in which crate `a` dev-depends on `b` and `b` dev-depends on
  `a`, each with an integration test calling the other; `cargo metadata` resolved it and
  `cargo test --workspace` compiled and ran both tests. Cargo requires the *normal* dependency
  graph to be acyclic and permits cycles through dev-dependencies. No file in this repository
  was modified to establish that. If Jeff prefers not to rely on it, §8 Q5 states the
  alternative — a test-only workspace member `crates/spectre-e2e` that nothing depends on —
  and why this spec does not choose it by default.

`crates/spectre-app/tests/` is the wrong home for the same reason R4-1 gives: `spectre-app`
does not depend on `spectre-offline`, and the FNV walk and the fixture render live there.
`crates/spectre-audio/tests/` is the wrong home because `spectre-audio` does not depend on
`spectre-app`, and the fixture must start from the model the user actually edits.

**The manual protocol lives in a new `docs/05-quality/` directory**, as
`r4-qa-protocol.md` and `r4-qa-records.md`. `docs/README.md:46` assigns the quality class
`docs/05-quality/` and gives it ownership of "deterministic validation and release gates";
`docs/README.md:50` says quality directories "are created only when they contain grounded
contracts", which is the condition this feature satisfies and the reason the directory does
not exist yet (verified: `docs/` contains `00-product`, `01-requirements`,
`02-reference-research`, `03-architecture`, `06-plans`, `status`, and `README.md`, and nothing
else). It does **not** live in `gauntlet-output/`, which is run state for the spec gauntlet and
carries no product authority.

**Nothing else moves.** R4-9 adds no module to any `src/` tree, no type, no trait, and no
device. It is the only R4 leaf that touches no callback-reachable code at all.

### 4.2 Data model

**No Rust types are added or modified.** The data model of this feature is (a) the fixture
project's content and (b) the run-record schema, which §3.3 defines once and which is
deliberately R4-3 §3.3's E1–E10 schema extended by four Q-fields rather than a second
independent schema.

**The fixture project — content, stated exactly.**

| Element | Value | Why this value |
|---|---|---|
| Envelope schema version | whatever R4-7 lands (`SCHEMA_VERSION` today is the constant `spectre_project::SCHEMA_VERSION`, re-exported and used at `crates/spectre-offline/src/lib.rs:14` and `:153`) | R4-7's §8 Q1 and Q2 are open; §8 Q4 routes the consequence rather than picking a number |
| Project name | `R4 Alpha E2E` | Distinguishes it from `Untitled`, the name `default_project` writes (`crates/spectre-offline/src/lib.rs:156`) |
| Tempo | one constant segment at 120 BPM | Reuses `TempoMap::constant(120.0)`, already the default project's tempo (`lib.rs:157`); a tempo *map* with segments belongs to a TIME-003 fixture, which already exists and is `verified` (`requirements-ledger.md:38`) |
| Transport | stopped, position 0 | TIME-005's state machine (`requirements-ledger.md:40`); the fixture asserts render output, not transport transitions |
| Tracks | exactly **three**: `Lead` (unmuted), `Pad` (unmuted), `Ref` (**muted**) | Three is the smallest count that produces R4-4 §1.3's hash triangle — all-unmuted, one-muted, and muted-track-deleted must be comparable — and R4-4's §1.3 states that triangle as its own success signal. Not a limit; fixture content |
| Devices per track | one `Filament` instrument and one `Gloam` insert (R4-6) | R4's exit row `docs/06-plans/current-milestone.md:84` ships one original synth and one original effect; a fixture that used `PulseInstrument → Gain → Saturator` would test R2's chain, not R4's alpha |
| Clip per track | one MIDI clip, notes spelled out below | R4's exit row `:83`: a MIDI clip plays through a track into master |
| Device parameter values | one non-default value per device, listed in the fixture file and echoed in the protocol | A fixture at every default cannot distinguish "values were restored" from "values were re-defaulted" on reload — the exact failure R4-7's round-trip assertion is for |

**The clip note content**, chosen so the fixture exercises the three places R4-5's scorecard
and spec identify as fragile, and nothing more:

1. `Lead`: one note-on at frame 0 of block 0, note-off at the last frame of block 15. The
   note-off sits **on a block boundary** because that is where a scheduler and a live ingress
   with independent sequence counters collide on the accepted ordering key
   `(frame_offset, NoteEventKind::rank, sequence)`, which R4-5's own header block names as
   the whole point of the feature.
2. `Pad`: two notes whose note-off and note-on share **one** frame offset, so the plan's own
   validator has to see a release ordered before an attack at equal timestamp — the TIME-002
   total-ordering rule (`requirements-ledger.md:37`), enforced by `NoteEventKind::rank`, which
   `docs/status/NEXT.md:42` records was made public specifically so producers sort by the key
   block validation enforces.
3. `Ref`: one note, identical in content to `Lead`'s, on the muted track. Identical content is
   what makes "muted contributes nothing" a hash-level claim rather than a level observation.

**The expected output is recorded, not asserted as a constant.** This is the single most
important line in §4 and it is the opposite of the obvious design.

A checked-in golden `u64` hash asserted in CI would fail for a reason that has nothing to do
with Spectre being wrong. R4-6's §8 Q5 states it plainly: `sin`, `exp`, and `tanh` are libm
calls, so bit-identical output across macOS and Linux **is not guaranteed for any Spectre
device today**, and the question of whether to pin a portable math implementation is open and
belongs to Jeff. A cross-platform golden constant asserted in the workspace suite would
therefore encode an unresolved decision as a passing test on one platform and a red CI on the
other. What R4-9 asserts instead is **equality between two computations on the same host in the
same process** — which is what determinism means at this stage — and what it **records** is the
observed hash, peak, frames, and channels per platform per commit, in the run record's Q-A
field. If and when Jeff closes R4-6 Q5 toward portable math, that record becomes the evidence
that promotes to a checked-in golden; until then it is a recorded observation and is labeled
as one. R4-8's §8 Q4 asks the same question from the bounce side and this spec deliberately
gives it the same answer rather than a second one.

**One numeric bound is introduced, and it is derived rather than chosen.**

- **`E2E_TOTAL_BLOCKS = E2E_NOTE_BLOCKS + E2E_TAIL_BLOCKS = 16 + 64 = 80`.**
  - `E2E_NOTE_BLOCKS = 16`: the note span must cross block boundaries in both directions, so
    it needs a note-on strictly inside the first block and a note-off on a later block's edge.
    Sixteen blocks is the smallest power-of-two span that leaves room for the same-frame
    off/on pair in the middle of the span without either event landing in the first or last
    block, where it would be indistinguishable from a boundary artifact.
  - `E2E_TAIL_BLOCKS = 64`: R4-6's §1.3 states its silence criterion as returning to
    **exactly** `0.0` "within 64 render quanta after the last note-off". A tail shorter than
    64 quanta cannot observe the criterion it exists to check, and a longer one adds render
    time without adding an observation. The bound is therefore **inherited from R4-6 and moves
    with it**; §7.2 schedules the ledger row to state the dependency in its own text so the two
    numbers cannot silently diverge.
  - **Arithmetic, stated so it can be checked:** 16 + 64 = **80** blocks. At the existing
    block geometry of 256 frames (`crates/spectre-audio/tests/lifecycle_health.rs:23`,
    `const FRAMES: usize = 256`), 80 × 256 = **20,480 frames**. At the existing requested rate
    of 48,000 Hz (`lifecycle_health.rs:22`, `const SAMPLE_RATE: f64 = 48_000.0`; the same value
    is requested by `StreamConfig::stereo(48_000, FRAMES)` at `:264`), 20,480 ÷ 48,000 =
    **0.426̅ s**, i.e. 426.67 ms of rendered audio. Two stereo `f32` channels at 20,480 frames
    is 20,480 × 2 × 4 = **163,840 bytes** of output per render, and the test performs three
    renders, so the peak transient cost is under 512 KB. §4.7 carries that figure.

**Bounds this spec deliberately does not introduce**, each with the reason:

- **No sample rate or block size of its own.** Both are reused from `lifecycle_health.rs:22–23`
  and cited, so the e2e record is directly comparable to the qualification record R4-3 produces
  on the same host. Restating them here would create two constants that must be kept equal.
- **No pass percentage, no tolerance, and no "at least N of M rows".** Every automated
  assertion is exact equality or exact zero. A tolerance would be a numeric bound with no
  derivation available, which is precisely what AF-4 and PROD-003 (`requirements-ledger.md:64`)
  forbid.
- **No run duration, no gesture count, no latency threshold.** AF-5, and
  `docs/02-reference-research/workflow-field-study/product-implications.md:97` and `:99`.
- **No retention count for run records.** The file is append-only and version-controlled; a
  numeric retention limit would be a bound with no rationale.

No migrations. No schema versioning of the record format itself: the protocol document carries
the metadata block `docs/README.md:64–66` requires and its version is its git history.

### 4.3 API contracts

**No function signature in any `src/` tree changes.** The contract this feature defines is the
mapping from each exit row to the artifact that produces evidence for it, and — as in R4-3
§4.3 — the honest distinction between what is **asserted** and what is merely **observed**.
This is the most important table in the spec.

| R4 exit row (`current-milestone.md`) | Evidence artifact | Class | Owning sibling |
|---|---|---|---|
| `:81` engine opens, sound through the existing plan, no second render path | Flow A hash equality (offline↔live) asserts *the computation*; **audibility is Flow B row 1 and is not automatable** | Asserted + **observed** | R4-1 |
| `:82` a UI edit changes live audio | Flow A asserts `parameters_pending == 0` and a level change across a published edit; **that the user's drag reaches the lane through `main.rs` is Flow B row 3** | Asserted + observed | R4-2 |
| `:83` a MIDI clip plays through a track into master | Flow A asserts nonzero peak on a clip-driven render with `plan_errors == 0` | Asserted | R4-5, R4-4 |
| `:84` one original synth and one original effect | Flow A renders through `Filament → Gloam` and asserts determinism, containment, and return to exact zero | Asserted | R4-6 |
| `:85` atomic save/reload round-trips tracks, clips, device parameters; CORE-001 reorder evidence | Flow A asserts pre-save↔post-reload hash equality and `ObjectId` stability across a reorder | Asserted | R4-7 |
| `:86` offline bounce deterministic and matching the live path | Flow A asserts equality at equal block size only (§4.4 rule 3) | Asserted | R4-8 |
| `:87` **end-to-end fixture and manual QA protocol both pass** | This feature. Flow A is the fixture; Flow B is the protocol | Asserted + observed | R4-9 |
| `:88` Linux device qualification runs | **Not produced by this feature.** R4-3's drill is the only artifact that closes it, and it is hardware-blocked | **Neither** — recorded `NOT RUN` | R4-3 |
| `:89` fmt, strict Clippy, full workspace suite green | The workspace gate, verbatim, recorded in E9 | Asserted (by CI and by the operator) | workspace |
| `:90` traceability and status match the implementation | Flow B row 15: the operator reads `docs/01-requirements/traceability.md` and `docs/status/STATUS.md` against the run's own findings | **Observed only** | R4-9 |

**The consequence, stated plainly, and counted against the table above rather than asserted.**
**Eight** of the ten rows have an assertable core (`:81`–`:87`, `:89`). **Three** of those eight
— `:81`, `:82`, and `:87` — carry an observed half that no assertion covers, because audibility,
a pointer drag through `main.rs`, and the manual protocol itself are not automatable. **One** row,
`:90`, is observation-only. **One** row, `:88`, this feature produces no evidence for at all, and
it cannot be produced on the hardware this project has. A protocol that reported "R4 exit: PASS"
on the strength of `cargo test` exiting 0 would be claiming all ten rows on the evidence of eight
partial ones.

**Assertion-message contract.** Every assertion in `e2e_alpha.rs` carries the owning sibling's
feature ID in its message text — `"R4-7: hash changed across save/reload"`, not
`"assertion failed: left == right"`. The cost is a string literal per assertion; the benefit is
that the transcript of a composition failure names the slice to open. §5.2 lists the assertions
and their messages.

**Auth / permissions.** N/A for Flow A: no service, no credential, no network path exists
anywhere in this workspace. Flow B has one permission surface, and only on Linux — the
operator's user must be able to open the ALSA device. R4-3 §4.3 records the same surface and
the same rule: a permission failure is recorded in E4 and is **not** worked around with
elevated privileges, because a root-only result does not describe how a user runs Spectre.

**Pagination / rate limiting.** N/A — no service and no request stream.

### 4.4 State management

Eight binding rules. Rules 1–4 govern the fixture; 5–8 govern the record.

1. **The fixture file on disk is the input of record.** Flow A reads
   `crates/spectre-offline/tests/fixtures/r4-alpha.json` with `std::fs::read` and decodes it
   through the accepted codec. It never constructs the project in memory as a fallback and
   never rewrites the file. A test that regenerates its own input is testing the generator.
2. **State ownership is unchanged.** `AppModel` stays the app-thread owner, `CompiledPlan`
   stays immutable and render-side per GRAPH-001 (`requirements-ledger.md:55`), and the
   fixture introduces no new store, container, or injection point. R4-9 adds no state.
3. **Live↔offline comparison happens at equal block size, and only at equal block size.** This
   is not a simplification; it is R4-8's load-bearing finding, and this spec adopts it rather
   than re-deriving it: RT-003 containment silences a whole render quantum
   (`crates/spectre-graph/src/lib.rs`, cited by R4-8's header block at `:517–518`), so on a
   contaminated render two different quantum geometries silence different amounts of audio and
   the hashes legitimately differ. Comparing 256-frame offline quanta against 256-frame driver
   blocks keeps the comparison well-defined; comparing across geometries would make a passing
   result depend on the render being clean, which is the thing under test.
4. **Flow A runs against `NullBackend`, and the spec says so rather than implying a driver.**
   `NullBackend` drives its callback through an explicit `pump` rather than a timer
   (`docs/06-plans/current-milestone.md:37`), so block counts are exact and there are no
   sleeps and no races. Flow A therefore proves the *composition*; it proves nothing about a
   real driver. Everything about real drivers belongs to R4-3's drill and to Flow B.
5. **The record is append-only.** A re-run appends a new numbered block. A failing block is
   never edited to reflect a later success, and a later success never removes the earlier
   failure. `docs/status/STATUS.md`'s header rule — claims link to live evidence — has no force
   if the evidence file is rewritten to agree with the claim.
6. **Environment fields are filled before the run, verdicts after.** §3.2 B1. A record whose
   environment was reconstructed after seeing the result cannot distinguish "this host passed"
   from "this host was described in whatever way made the pass make sense".
7. **Per-platform outcomes never merge.** One outcome word per platform column, and no
   combined word anywhere in the record or in the milestone. §5.6 states what this authorizes.
8. **A row the operator could not produce is `NOT RUN`, never `PASS`.** Including — especially
   including — rows whose adjacent rows passed. Inference between rows is the mechanism by
   which a partial run becomes a full claim.

### 4.5 Dependencies

- **New packages / libraries / frameworks:** **one line in one manifest** —
  `spectre-audio = { path = "../spectre-audio" }` added to
  `crates/spectre-offline/Cargo.toml` under `[dev-dependencies]`, alongside the existing
  `spectre-app` entry at `:20–21`. No third-party crate is added anywhere. In particular
  **no UI-automation framework, no screenshot-diff library, no headless-display harness, and
  no audio-capture crate** is introduced; §5.3 justifies each absence.
- **New assets or resources:** one checked-in JSON fixture,
  `crates/spectre-offline/tests/fixtures/r4-alpha.json`. It is the second project fixture in
  the repository; the first is `crates/spectre-project/tests/fixtures/r1-canonical.json`
  (verified: those are the only two `.json` files under `crates/`). It is original content
  authored for Spectre — original note data, original device names, original parameter values —
  and contains no third-party material.
- **Infrastructure changes:** none required. CI already runs the full workspace suite
  (`.github/workflows/ci.yml:46–47`), so `e2e_alpha` runs in CI the day it lands, on
  `ubuntu-latest`, against `NullBackend`, with no audio device — which is exactly the
  configuration Flow A is designed for. **No CI change is proposed**: adding a macOS runner or
  a virtual display would be new infrastructure serving a GUI harness this spec does not build,
  and adding an audio device to CI is not possible in the hosted runner.

### 4.6 Platform-specific considerations

**Flow A is platform-independent by construction and must stay that way.** It uses
`NullBackend`, asserts only same-host equalities, and checks in no cross-platform constant
(§4.2). It therefore produces the same verdict on macOS and on Linux, and its passing on
`ubuntu-latest` in CI is a real result rather than a partial one.

**Flow B is not, and this is where decision 1 lands.** Decision 1 (`decision-gates.md:25`)
makes macOS and Linux co-first-class. Decision 23 (`:49`) narrowed **R3's exit** to macOS
qualification alone, explicitly did not amend decision 1, and states that "no Linux support
claim is authorized until the drill runs on real Linux hardware". The milestone carries that
as inherited debt item 1 (`current-milestone.md:22`) with the exact command that discharges it.

The situation R4-9 inherits is worse than R4-3's, and the difference is worth stating precisely
because it is easy to assume the two are the same block:

- R4-3 needs **one command on a Linux host with a real ALSA device**:
  `cargo test -p spectre-audio --test lifecycle_health -- --ignored --nocapture`.
- R4-9's Flow B needs **an operator at that same host**, with an audio interface attached,
  working through a rendered egui window, listening, unplugging a device mid-playback, running
  a screen reader, and saving and reopening files.

So the Linux column of R4-9's record is blocked by a strict superset of what blocks R4-3, and
it cannot be discharged by borrowing R4-3's result: a passing lifecycle drill establishes that
cpal's ALSA backend opened a device from a test binary, and says nothing about `./spectre`,
which does not use `spectre-audio` at all today. R4-3 §5.6 item 9 makes exactly that point
about its own PASS.

**The build-versus-device distinction is already established and must not be blurred.** On
2026-08-09 the workspace compiled and linked against ALSA in a Linux aarch64 container and all
34 non-hardware `spectre-audio` tests passed, and the drill failed closed on a machine with no
device rather than reporting a false pass (`current-milestone.md:116`). That is a build and
portability result. **No Linux audio device has ever been opened** (`current-milestone.md:127`;
`docs/status/STATUS.md` known gaps). Flow B's Linux column inherits that state exactly.

**macOS specifics for Flow B.** The one hardware qualification on record is macOS: cpal opened
an M-Audio AIR 192|6 through CoreAudio for 173 driver callbacks with 0 xruns, worst-case
headroom 0.990 (`current-milestone.md:113`). Flow B on macOS is therefore runnable the moment
R4-1 lands. Its record must still name the interface in E5, because one interface is not a
platform — R4-3 §5.6 item 2 gives the reasoning and it applies unchanged here.

**Version compatibility.** `rust-toolchain.toml` pins the `stable` channel with no version
number, so the record's E7 field must capture the actual `rustc --version` at run time; two
runs on "stable" are not necessarily two runs on the same compiler, and with libm-backed device
math (R4-6 §8 Q5) that is not a pedantic distinction.

**Feature flags.** `crates/spectre-audio/Cargo.toml:12–15` makes `cpal-backend` a default-on
feature. Flow A does not require it — `NullBackend` is unconditional — but a run with
`--no-default-features` is a different configuration and E7 records the features in effect.

### 4.7 Performance budget

- **Memory.** Flow A's dominant allocation is three planar stereo renders of 20,480 frames:
  20,480 × 2 channels × 4 bytes = **163,840 bytes** each, so under **512 KB** across the three,
  plus the compiled plan's own per-block channel pool, which R4-8 §4.7 derives as 6,144 B at
  256 frames and which is unchanged here. All of it is app-thread allocation in a test binary;
  none of it is on a callback-reachable path.
- **CPU / render time.** 80 blocks per render × 3 renders = **240 block renders** of a
  six-device chain (three tracks × two devices) plus summing. The existing macOS qualification
  observed worst-case headroom 0.990 on a three-device chain at the same block geometry
  (`current-milestone.md:113`), i.e. roughly 1% of the block's time budget consumed; this
  fixture's chain is larger and its render is off the callback entirely, so the wall-clock cost
  is bounded by ordinary test-suite noise. **No time budget is asserted** — see §3.4 and AF-5 —
  and the test contains no timing assertion, because a timing assertion in a shared CI runner
  is a flake generator, not a performance gate.
- **Network payload.** N/A — no network path exists anywhere in this workspace.
- **Storage.** The fixture JSON is a three-track, three-clip project in the inspectable JSON
  envelope decision 3 chose (`decision-gates.md:27`); on the order of a few kilobytes, and it
  is checked in once. Flow A writes one temporary project file during the save/reload assertion
  and removes it. Run records grow by roughly one screen of Markdown per run.
- **Startup time.** Unaffected. R4-9 adds nothing to `./spectre`; §3.1 adds no surface and
  §4.1 adds no module to any `src/` tree.
- **What the per-block hash log costs.** R4-9 does **not** enable R4-8's per-block hash log for
  the e2e fixture. R4-8's remediation established that the log is 8 bytes per block and that its
  cost at a 24-hour ceiling is 16,200,000 blocks × 8 B = 129.6 MB. At R4-9's 80 blocks the log
  would be 640 bytes — trivially cheap — but it is still left off, because its only consumer is
  the live/offline comparison and enabling it here would couple this fixture to a default whose
  disposition is R4-8's open question, not R4-9's.


### 4.8 Realtime disposition — RT-001, RT-002, RT-003, GRAPH-001

*(Added beyond the template's seven subsections. The realtime contract is the workspace's
heaviest standing policy and a feature that touches none of it should say so explicitly and
checkably rather than by silence.)*

**RT-001 (`requirements-ledger.md:28`) — callback-path discipline.** R4-9 adds **no
callback-reachable code**. §7.2's change list contains no file under any `src/` tree: four new
files (two test-tree files, two documents), one manifest line, and edits to five documents.
The structural lock scan at `crates/spectre-audio/tests/rt_guard.rs:289–320` reads exactly four
modules — `src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, `src/null.rs` (`:293–298`) — and
R4-9 modifies none of them. **This spec can therefore make the untouched-set argument honestly,
which is precisely the argument R4-1's iteration 1 made falsely and R4-2 correctly declined to
make**: R4-1 named a module that is not scanned and missed one that is, and R4-2's change list
modified two of the four so it argued from edit content instead. R4-9's change list is checkable
against the scan array in one read and intersects it in zero files. The allocation guard's
positive control and the guarded paths are likewise unchanged.

Flow A's own test code allocates freely — it decodes JSON, builds graphs, and collects hashes —
and that is not an RT-001 concern: it runs on the test thread, and `NullBackend` renders
through an explicit `pump` call rather than a driver thread (`current-milestone.md:37`), so no
allocation in the harness is ever inside a callback. The protocol document states this so a
future reader does not "optimize" the fixture for realtime safety it does not need.

**RT-002 (`:29`) — control↔render communication.** R4-9 introduces no new lane, no new message
type, and no new overflow policy. One assertion, §5.2 I-8, **uses** the existing parameter lane:
it publishes through R4-2's app-thread API and observes the effect after the next `pump`. That
lane is decision 21's latest-wins slot per `(device, parameter)` target with a version counter
(`decision-gates.md:45`), so its overflow behavior is coalescing rather than rejection and no
counted-overflow path is exercised. Retired-state reclamation is unchanged and is not on any
path this feature adds. I-8 asserts `parameters_pending == 0`, which is R4-2's acceptance
criterion and which fails today by construction (`crates/spectre-audio/src/bridge.rs:181–186`).

**RT-003 (`:30`) — numerical containment.** R4-9 adds **no DSP node**, so it owes no injection
fixture; the per-node-type injection evidence stays in `crates/spectre-graph/tests/containment.rs`
and belongs to R4-6 for its two new devices. What R4-9 adds is composition-level containment
evidence: I-9 asserts `contaminated_nodes == 0` across the whole run
(`crates/spectre-audio/src/bridge.rs:88`) and I-10 asserts the chain returns to exact zero of
either sign, which fails on a residual denormal while passing the `-0.0` that the software
FTZ-equivalent flush legitimately produces (`crates/spectre-graph/src/lib.rs:407–410`, called
from `:513–514`).

**RT-003's second half is deliberately not asserted, and the reason is worth stating.**
`denormals_flushed` (`bridge.rs:93`) is a counter this fixture could read, but a nonzero value
is not a defect — flushing is the requirement working — and a zero value on a clean render
proves nothing. Asserting either direction would be a test that cannot fail in a useful way
(criterion 1G), so the counter is **recorded** in the run record and asserted nowhere. R4-3
§5.6 item 11 records the same counter as unmeasured in its own drill; R4-9 measures it and
declines to threshold it, which is a different and stronger position than not looking.

**GRAPH-001 (`:55`) — graph/plan split.** Unchanged and unchallenged. The fixture compiles once
per render and executes an immutable `CompiledPlan`; **no plan recompilation per parameter
change is proposed anywhere in this spec**, which decision 22 (`decision-gates.md:47`) rejects
on its face because compilation allocates. I-8 changes a parameter through the RT-002 lane
precisely so that it does not recompile.

**Fail-closed behavior (criterion 1F).** Three places, each explicit rather than incidental:
Flow A never regenerates a missing fixture and fails with the path instead (§4.4 rule 1); every
assertion is exact equality or exact zero with no tolerance (§4.2); and a row the operator could
not produce is `NOT RUN`, never `PASS` (§4.4 rule 8). The feature's own failure mode — an
unproduceable row silently becoming a claim — is closed by vocabulary rather than by code, which
is the only mechanism available to a protocol.

---

## 5. Test Specification

**Runnability, stated first and without hedging.** Of everything named in this section:

- **Runnable today, verified by running it on 2026-08-21:** `cargo fmt --all -- --check`
  (exit 0); `cargo run --locked --quiet -p spectre-offline -- --self-test` (prints the
  `OfflineReport` JSON — `schema_version 1`, `project_name "Untitled"`, one tempo segment,
  transport at sample 0); `./spectre --smoke-test` (prints
  `Spectre prototype ready lens=Arrange tracks=1 transport=stopped selected_device=Pulse(pulse)`).
  The other two gate commands, `cargo clippy --locked --workspace --all-targets -- -D warnings`
  and `cargo test --locked --workspace`, are the workspace gate recorded at
  `docs/status/STATUS.md` §"Validation" and enforced in CI at `.github/workflows/ci.yml:43–47`;
  they were **not** re-run for this spec and no current result is claimed for them. The last
  recorded result is 230/230 with one ignored hardware drill on 2026-08-09
  (`docs/01-requirements/traceability.md:53`), which is a dated record, not a present-tense
  claim.
- **Not runnable yet, and why:** **every test in §5.1, §5.2, and §5.3 below.** They name a file
  (`crates/spectre-offline/tests/e2e_alpha.rs`) that does not exist, a fixture
  (`…/fixtures/r4-alpha.json`) that does not exist, and APIs — a track list, a clip, `Filament`,
  `Gloam`, `save_project_atomic`, `LiveEngine` — that do not exist. All eight sibling features
  are spec'd and **none is implemented**. This section specifies what will be runnable; it does
  not describe a suite that runs. §7.1 states the same thing about the codebase.
- **Not runnable on this project's hardware at all:** §5.4's Linux column, for the reason §4.6
  gives.

### 5.1 Unit tests

R4-9 introduces **no unit-testable logic of its own** — it adds no function, type, or module to
any `src/` tree (§4.1). Writing unit tests here would mean inventing a unit to test, which is
the shape of a test that cannot fail (criterion 1G).

What belongs at the unit level instead is **fixture hygiene**: three small tests, in
`e2e_alpha.rs`, whose subject is the checked-in file rather than the render. Each fails if the
fixture rots.

| # | Name | Setup | Assertion | Edge case covered |
|---|---|---|---|---|
| U-1 | `fixture_decodes_and_validates` | Read `fixtures/r4-alpha.json` from disk | Decode succeeds and semantic validation passes; the error is surfaced verbatim if not | A fixture that survives a schema change syntactically but not semantically |
| U-2 | `fixture_content_is_exactly_what_the_protocol_documents` | Decode the fixture | Exactly 3 tracks named `Lead`, `Pad`, `Ref`; exactly one of them muted; exactly one clip per track; every device parameter differs from its descriptor default | The fixture silently drifting from the protocol document that describes it to the operator |
| U-3 | `fixture_object_ids_are_unique_and_nonzero` | Decode the fixture | Every `ObjectId` in the file is nonzero and no two are equal | CORE-001 (`requirements-ledger.md:46`); the existing codec already rejects zero IDs at decode, so this asserts the *fixture author* did not create a collision the codec would accept |

**Not runnable yet:** all three depend on R4-4's and R4-5's schema and on the fixture file.

### 5.2 Integration tests — `crates/spectre-offline/tests/e2e_alpha.rs`

This is Flow A. One test file, one fixture, and the assertions below. Every message carries the
owning sibling's ID per §4.3.

| # | Name | Setup | Assertion (message prefix) | Would fail if |
|---|---|---|---|---|
| I-1 | `fixture_renders_with_a_nonzero_peak` | Decode fixture; build the graph; render `E2E_TOTAL_BLOCKS` blocks of 256 frames at 48 kHz | `peak > 0.0` (`"R4-6/R4-5: the fixture rendered silence"`) | The clip scheduled no notes, the instrument produced nothing, or a mute rule silenced everything. **This assertion is what makes every hash equality below meaningful** |
| I-2 | `two_renders_of_the_fixture_are_bit_identical` | Render twice in one process | Both `RenderReport.hash` values equal, and both peaks equal | Any nondeterminism in the plan, the scheduler, or device state — GRAPH-001's determinism claim at the composition level |
| I-3 | `live_bridge_matches_the_offline_render_at_equal_block_size` | Drive the identical schedule through `RenderBridge` over `NullBackend` at 256-frame blocks; hash the interleaved output with the walk `bridge_plan.rs:80–92` already uses | Hashes equal (`"R4-1/R4-8: live and offline diverged"`) | A second render path appeared, or the bridge and the offline harness disagree about note delivery. §4.4 rule 3 fixes the block size |
| I-4 | `save_reload_render_produces_the_same_hash` | Render; `save_project_atomic` to a temp path; reload; render again | Third hash equals the first (`"R4-7: persistence changed what the project computes"`) | Reload lost a device value, a clip note, or a track order. R4-7 §1.3(c) states the same assertion at its own level; here it runs on the composed project |
| I-5 | `track_ids_survive_reorder_across_a_save` | Reorder the three tracks; save; reload | Every `ObjectId` unchanged and the new order exact (`"R4-7/CORE-001"`) | CORE-001's reorder evidence, which `requirements-ledger.md:46` gates explicitly on the first persisted collection |
| I-6 | `muting_a_track_changes_the_hash` | Render all-unmuted; render with `Ref` muted | Hashes differ, both peaks nonzero | A mute that reaches no signal path — the exact state `TrackView`'s booleans are in today (§7.1) |
| I-7 | `a_muted_track_hashes_equal_to_its_absence` | Render with `Ref` muted; render the project with `Ref` deleted | Hashes equal (`"R4-4: a muted track contributed signal"`) | **Flagged:** this asserts R4-4's claim that a muted track contributes exact zero. R4-9 does not independently establish it; if R4-4 lands additive solo semantics or a non-zero mute floor this assertion moves with R4-4, and §8 Q6 routes it |
| I-8 | `a_published_parameter_edit_changes_the_next_block` | Render one block; publish a `Gloam` parameter change on the RT-002 lane; render the next block | The second block differs, `parameters_applied` advanced, and **`parameters_pending == 0`** (`"R4-2: the parameter lane is not being consumed"`) | R4-2's acceptance criterion. `parameters_pending` is incremented today at `crates/spectre-audio/src/bridge.rs:182–186` because the drained value goes into a closure that discards it (`:181`), so this assertion fails by construction until R4-2 lands |
| I-9 | `telemetry_is_clean_across_the_whole_run` | After I-1 | `plan_errors == 0`, `contaminated_nodes == 0`, `notes_deferred == 0`, `frame_capacity_rejections == 0`, each a separate assertion with its own message | RT-003 containment fired (`bridge.rs:88`), the plan refused a block (`:68`), the note scratch overflowed (`:78`), or a block exceeded the plan's capacity (`:73`). Four counters, four failure causes, four distinguishable messages — the ambiguity R4-3 §3.6 E-9 documents for the live drill does not arise here because the test chooses the block size |
| I-10 | `the_chain_returns_to_exact_zero_after_the_last_note_off` | Render the tail blocks | Every sample in the final block is exactly zero, asserted as `sample == 0.0` **and** `sample.abs() < f32::MIN_POSITIVE` so a residual denormal fails while a legitimately flushed **negative** zero passes | R4-6 §1.3's silence criterion at the composition level. The signed-zero allowance is not a loophole: `contain_channel` flushes a denormal to `-0.0` when the sign is negative (`crates/spectre-graph/src/lib.rs:407–410`), so a bit-pattern equality against `+0.0` alone would fail a correctly-flushed render. `E2E_TAIL_BLOCKS = 64` exists to make this observable (§4.2) |
| I-11 | `the_bounce_of_the_fixture_matches_the_live_path` | Run R4-8's bounce over the same fixture at the same block size | Streaming hash equals I-3's live hash | R4's exit row `current-milestone.md:86`. Depends on R4-8's `bounce_equivalence` API |
| I-12 | `the_report_line_is_printed_for_the_record` | End of run | Prints one line: `e2e frames={} channels={} peak={} hash={:#018x} blocks={}` | Not an assertion. It exists so the operator can transcribe Q-A into the record without re-deriving anything (§3.3). Marked explicitly as a print, not a check, so nobody mistakes it for coverage — the trap R4-3 §4.3 found in the hardware drill |

**Not runnable yet:** all twelve. Every one names an API that no slice has built.

**One test deliberately absent.** There is no assertion comparing this fixture's hash to a
checked-in constant. §4.2 gives the reason: cross-platform bit-equality is not established for
any Spectre device (R4-6 §8 Q5), so such a constant would encode an open decision as a CI
result. The observed hash is recorded per platform in Q-A instead.

### 5.3 UI / E2E tests

**There is no automated GUI-driving harness in this repository, and R4-9 does not build one.**
This is the decision R4-1 §5.3 deferred here, and it is answered rather than deferred again.

**The facts it rests on, each checkable in one read:**

1. `crates/spectre-app/tests/` contains exactly two files, `app_model.rs` and `smoke_cli.rs`.
2. `app_model.rs` tests `AppModel` as a plain data type; its header note at `:4` reads "Pins
   behavior independently of egui rendering", and none of its tests constructs a renderer.
3. `smoke_cli.rs` is a **process** test: it runs `CARGO_BIN_EXE_spectre-app` with
   `--smoke-test` (`:10–13`) and asserts four substrings of one stdout line (`:16–19`). It
   never opens a window.
4. `crates/spectre-app/src/main.rs:499–503` shows why that is the only headless path:
   `main` checks for `--smoke-test`, calls `smoke_test()` (`:480–497`, which prints one line
   from `AppModel::prototype()`), and **returns before** `eframe::run_native` at `:511–518`.
   Every widget in the file is inside the `eframe` event loop, which owns the window and needs
   a windowing system.
5. CI is a single `ubuntu-latest` job (`.github/workflows/ci.yml:22–23`) whose own header
   comment at `:6–7` states that this is "a deliberate minimum, not a platform coverage claim".
   It installs xcb and xkbcommon **dev** packages (`:30–32`) so `eframe` *links*; it starts no
   display server and runs no GUI.

**Why building one is the wrong call for R4:**

- It would need three pieces of new infrastructure at once — a third-party UI-automation
  dependency, a virtual display in CI, and a macOS runner that does not exist — none of which
  any exit row asks for. `current-milestone.md:87` asks for "an end-to-end fixture plus a
  written manual QA protocol", which is a fixture and a document.
- The two things a GUI harness would actually verify — that the user *hears* the audio, and
  that the window *looks* right — are exactly the two things a GUI harness cannot verify. A
  pixel-diff proves a widget was drawn; it does not prove the drawn widget reports true state.
  `docs/00-product/vision.md:48`'s bar is "honest telemetry, no fake surfaces", and a screenshot
  of a fake surface passes a screenshot test.
- The surface under test is unbuilt. Automating a UI that eight unimplemented specs describe
  would produce a harness whose assertions are guesses about widget identity, and the first
  slice to land would rewrite it.

**What R4-9 provides at the UI/E2E level instead:**

- **`cargo test -p spectre-app --test smoke_cli`, extended by one assertion.** R4-1 §5.3 already
  specifies adding an `engine=` field to the smoke line and asserting `engine=not-started`.
  R4-9 adds nothing to that line and duplicates none of it; it **records** the smoke line
  verbatim in the run record's Q-A field, so the record shows which build was exercised.
- **`./spectre --smoke-test` must exit 0 on a machine with no audio device.** Verified runnable
  today (see the runnability note above); it is R4-1's assertion and R4-9 re-runs it as a
  precondition of Flow B, because an operator who cannot get a clean smoke line should stop
  before recording UI observations.
- **Everything else at this level is Flow B.** §5.4 is not a supplement to automated UI
  coverage; for the rendered surface it is the *entirety* of the coverage, and the protocol
  document says so in its own text so that no future reader assumes a harness exists.

**The one uncovered item this leaves, named rather than hidden.** R4-1's own iteration-2
scorecard recorded that its §4.4 binding rule — the spec's single new correctness hazard — has
no automated coverage because it lives in `main.rs`, which is unreachable from tests, and that
its test 7 gestures at it with an assertion that cannot fail. R4-9 does not fix that: no
harness R4-9 declines to build could have covered it either, because the rule is about what
`main.rs` does with an engine handle. It is Flow B row 3's subject and it is listed in the
record's Q-D field ("what was not checked") on every run where the operator does not reach it.

### 5.4 Visual / manual verification

This is Flow B: the manual protocol R4's exit row `current-milestone.md:87` requires. It is
reproduced verbatim into `docs/05-quality/r4-qa-protocol.md` as that document's §Manual checks.

**Standing instructions.** Run `./spectre` **with system volume low** — R4-1 §5.4 gives the
reason: the audition voice is a held saw with no amplitude envelope and it clicks at note
edges. Record a verdict for **every** row, including rows you cannot produce. `NOT RUN` is a
valid verdict; a blank is not.

| # | Row | What the operator does and records | Verdict class |
|---|---|---|---|
| 1 | **Audibility** | Press Play. Sound is audible. Record **what was heard** — pitch, character, whether it matched the fixture's note content — not that "audio worked". Record whether the onset felt immediate or delayed as an operator judgement; **no latency threshold is defined anywhere in this spec** (§3.4) | The single row that no test in §5.2 can produce |
| 2 | **Silence at rest and after Stop** | No sound before Play. After Stop, **exact** silence, not a fading tail or a low hum | |
| 3 | **A slider drag reaches live audio** | With sound playing, drag a `Gloam` parameter. The sound changes, and the transport counters read `params pending 0`. **This row covers R4-1 §4.4's binding rule**, which lives in `main.rs` and has no automated coverage (§5.3) | |
| 4 | **Transport honesty** | Position readout advances while playing and matches what is heard; it does not show a frozen or fabricated position when the engine is not running | `vision.md:48` "no fake surfaces" |
| 5 | **Engine-unavailable state** | Launch with the default output device disabled or unplugged. The app opens, states that the engine is unavailable, and produces no sound. It does not crash and does not claim to be playing | |
| 6 | **Device loss mid-playback** | Unplug the interface while playing. No crash; the engine state and/or counters reflect the loss; reconnecting and retrying recovers | `NOT RUN` if the host has no detachable interface (E-9) |
| 7 | **Save / quit / reopen** | Save the fixture project to a new path, quit, relaunch, open it. Same three tracks, same order, same names, same device values on screen | R4-7 |
| 8 | **Save over an existing file** | Save onto an existing project file. The previous file is either fully replaced or fully intact — never a half-written file | CORE-004 (`requirements-ledger.md:49`); crash-injection evidence is R5's, not this row's |
| 9 | **Bounce the fixture and listen to it** | Run the bounce; open the written file in any player. It sounds like what was heard live, and its duration matches the requested span | R4-8. "Sounds like" is a recorded judgement, not a measurement, and the record says so |
| 10 | **Screen size extremes** | At the 1060×680 minimum and the 1420×860 default (`crates/spectre-app/src/main.rs:507–508`), every panel stays inside the window and no control is clipped | |
| 11 | **Motion** | Note any animated transition that has no reduced-motion path. Recorded as a finding, not a gate (§3.5) | |
| 12 | **Keyboard-only pass** | Attempt every step of rows 1–9 without a pointer. Record **which steps were impossible**. Record **no key assignments** (§3.4, AF-5) | Decision 17 audit input |
| 13 | **Screen-reader pass** | Repeat rows 1–9 with VoiceOver (macOS) or Orca (Linux). Record which controls announced no label. Note that `eframe`'s accessibility feature is not enabled (`crates/spectre-app/Cargo.toml:13`), so an empty result is a build-configuration finding, not a labeling audit | Decision 17 audit input |
| 14 | **Theme variants** | **N/A — the shell hard-codes a single dark palette.** R4-1 §5.4 records this with its source lines; there is no light variant to check and introducing one is not this feature's work | Structural N/A |
| 15 | **Traceability and status** | Read `docs/01-requirements/traceability.md` and `docs/status/STATUS.md` against this run's findings. Record every claim in them that this run contradicts | R4 exit row `:90`; observation-only (§4.3) |

**Two things the operator must not do**, stated because both are natural and both destroy the
record's value:

- **Do not use audibility to infer a passing hash, or a passing hash to infer audibility.**
  They are independent evidence classes. The failure mode where all hashes agree and nothing is
  heard is error state E-8, and it is the highest-value case this protocol exists to catch.
- **Do not re-run a failing row until it passes and record only that.** Record the failure,
  then record the re-run as a separate numbered block (§4.4 rule 5). R4-3 §3.6 E-4 applies the
  same rule to a refused geometry and for the same reason.

### 5.5 Outcome evaluation rules

*(Added beyond the template's four subsections because the record format is half of this
feature's deliverable and the outcome vocabulary has nowhere else to live. The vocabulary is
R4-3 §5.5's, reused rather than reinvented, so that a reader of the milestone meets one set of
words.)*

Applied **per platform**, never across platforms (§4.4 rule 7).

| Outcome | Definition | Effect on R4's exit |
|---|---|---|
| `PASS` | Flow A passes in full, **and** every §5.4 row is `PASS` or a structural `N/A` (row 14) | Exit row `:87` is satisfied **for this platform only**, subject to §5.6 |
| `FAIL` | Flow A fails any assertion, **or** any §5.4 row is `FAIL` | Row `:87` stays open. The record is written and kept; the finding is routed to the owning sibling |
| `INCONCLUSIVE` | Flow A passes but one or more §5.4 rows are `NOT RUN` for host reasons (E-9), or the workspace gate did not run clean immediately before (E9) | Row `:87` stays open. Nothing adverse was learned |
| `BLOCKED` | One or more sibling features are unimplemented, so the rows that depend on them have no surface to exercise | Row `:87` stays open. **This is the outcome of a run performed today**, and §7.1 says why |
| `REFUSED` | The host's audio device declined the requested geometry, so the engine never opened | Row `:87` stays open. This is a finding about the seam, not about the operator; R4-3 §5.5 uses the same word for the same condition |

**Two rules about the words themselves.**

1. **A green `cargo test` is necessary and not sufficient.** This is R4-3's central insight
   applied one level up: its §4.3 showed that the hardware drill prints `xruns` and
   `worst_headroom` and asserts neither, so a run can emit `xruns=41` and libtest still reports
   `ok`. R4-9's equivalent is broader — §4.3's table shows three exit rows whose observed
   half no assertion covers, one that is observation-only, and one for which this feature
   produces no evidence at all. The operator evaluates
   these rules against the record, not against the exit code, in **both** directions: a run may
   also be `BLOCKED` while every test that exists passes.
2. **There is no aggregate word.** §5.6.

### 5.6 What an R4-9 PASS authorizes, and what it does not

A `PASS` on one platform authorizes exactly one sentence, of this shape and no broader:

> On {date}, on {OS} {release} with {audio subsystem}, at commit {SHA}, Spectre's end-to-end
> fixture rendered, round-tripped, and matched the live path in {N} automated assertions, and
> an operator worked the {M} manual rows of the R4 QA protocol on {interface}, with the
> results recorded in `docs/05-quality/r4-qa-records.md` block {Q-n}.

**It does not authorize** — and the record's Q-D field states each of these that applies:

1. **"R4 has exited."** R4's exit is a ten-row conjunction (`current-milestone.md:79–90`).
   This feature produces evidence for one of those rows and contributes to seven others; it
   produces **none** for row `:88`, Linux device qualification. §8 Q1 is the governing question
   and it belongs to Jeff.
2. **"Spectre works on Linux."** No Linux audio device has ever been opened
   (`current-milestone.md:127`). A macOS-only `PASS` says nothing about Linux, and decision 23
   (`decision-gates.md:49`) authorizes no Linux support claim until R4-3's drill runs on real
   Linux hardware. A **combined** verdict across a run macOS column and an empty Linux column
   would be exactly that unauthorized claim, which is why §4.4 rule 7 forbids one.
3. **Any other machine, interface, or driver configuration.** One host, one interface, one
   configuration. R4-3 §5.6 items 2–4 give the reasoning and it transfers unchanged.
4. **Crash durability or recovery.** CORE-004's crash-injection evidence is R5's by the
   accepted persistence contract's own milestone assignment; row 8 checks atomic replacement in
   the ordinary path only, and R4-7 §1.3(b) owns the fault-injection half.
5. **Long-run stability.** The fixture renders 426.67 ms of audio and Flow B is a single
   session. Nothing about xruns, drift, or leaks over minutes or hours is measured.
6. **Cross-platform bit-equality of the audio.** §4.2: no cross-platform golden is asserted,
   because R4-6 §8 Q5 leaves the libm question open. Two `PASS` records on two platforms
   establish that each host is self-consistent, **not** that they computed the same samples.
7. **Accessibility.** Rows 12 and 13 record findings toward decision 17's beta gate. A `PASS`
   with unlabeled controls is a `PASS` with recorded accessibility findings, and the record must
   not be read as an accessibility conformance result.
8. **Performance.** No timing assertion exists (§4.7) and no latency or CPU threshold is
   defined anywhere in this spec (AF-5). Headroom figures transcribed from telemetry are
   observations.
9. **That the eight sibling features are correct.** R4-9 tests their **composition**. Each
   sibling's own evidence is its own; a passing e2e run over a subtly wrong device is a passing
   e2e run.
10. **That the protocol itself is complete.** It was written before any of the surfaces it
    checks existed. Every row will need re-reading against the real surface when the slices
    land, and §8 Q11 asks who does that and when.

---

## 6. Compliance & Safety Gate

### 6.1 Sensitive data classification

- [x] **No sensitive data involvement.** Flow A reads one checked-in JSON fixture, writes one
  temporary project file, and removes it. Flow B records an OS name and release, a kernel or
  Darwin release, an audio-subsystem version, an interface make and model, a device key as
  reported by the OS, a commit SHA, and a toolchain version. No user content, credential,
  telephone number, location, or personal identifier is collected. There is no network path
  anywhere in this workspace: `Cargo.lock` contains no HTTP or async-runtime crate (verified
  2026-08-21 — no `reqwest`, `hyper`, `ureq`, or `tokio` package entry), so nothing is
  transmitted anywhere by any part of this feature.
- [ ] Handles sensitive data — no.
- [x] **Uses synthetic/test data only.** The fixture project is authored test content: invented
  track names, invented note data, invented parameter values.

One caution worth writing into the protocol document rather than leaving implicit: an
interface make and model plus an OS build in a public repository is low-sensitivity but not
zero-sensitivity information about Jeff's machines. The record deliberately captures the
*device key as the OS reports it* and the interface model, because R4-3 §3.3 establishes that
the key is the strongest available evidence about what was actually opened — but it captures
no hostname, no username, and no serial number, and the protocol says so explicitly so an
operator does not paste a fuller `system_profiler` or `aplay -l` dump than the fields ask for.

### 6.2 Asset provenance

- [x] **No third-party assets.** No fonts, images, samples, wavetables, presets, or data files
  are added. The one new resource is `crates/spectre-offline/tests/fixtures/r4-alpha.json`,
  original content authored for Spectre.
- [ ] Uses third-party assets — no.

The only third-party **code** this feature touches is code already in the workspace and already
dispositioned: `cpal 0.15.3` (`crates/spectre-audio/Cargo.toml:18`), whose dual MIT/Apache
licensing decision 19 records as matching this workspace (`decision-gates.md:43`), and
`eframe 0.32.3` (`crates/spectre-app/Cargo.toml:13`), adopted under decision 8
(`decision-gates.md:32`). **R4-9 adds no crate**, and §4.5 states which categories of crate it
deliberately declines to add.

### 6.3 Language / claims audit

- [ ] Makes claims not supported by evidence — **no, and this is the section this feature is
  most at risk in.** A QA protocol's whole output is claims. The controls are structural rather
  than editorial: §5.6 enumerates what a `PASS` does not authorize; §4.4 rule 8 forbids
  inferring one row from another; §4.4 rule 7 forbids an aggregate verdict; §3.3 forbids
  sourcing any record cell from documentation; and §5.5 adds `BLOCKED` to the vocabulary
  specifically so that a run performed before the slices land has an honest word available
  instead of being forced toward `PASS` or `FAIL`.
- [ ] Promises capabilities not yet built — **no**, and the reverse discipline is applied
  throughout. The header block, the standing constraint, §5's runnability note, §5.5's
  `BLOCKED` outcome, and §7.1 all state that all eight sibling features are spec'd and none is
  implemented, and that `./spectre` produces no sound. No sentence in this document is written
  in the present tense about an unbuilt surface.
- [ ] Uses language restricted by domain regulations — **no.** No medical, financial, safety,
  or loudness-conformance claim appears. In particular no loudness or delivery-target claim is
  made anywhere: MM-AF-5's surviving prohibitions cover conformance claims and delivery targets,
  and row 9's "sounds like what was heard live" is recorded explicitly as an operator judgement
  rather than a measurement.

### 6.4 Regulatory alignment — `criteria.md` Lens 3, criterion by criterion

- **3A Milestone fit.** The feature *is* an R4 exit row (`current-milestone.md:87`). It adds
  no capability: no device, no DSP, no UI surface (§3.1), no crate (§4.5). The one thing it
  could have over-reached on — a GUI-automation harness — is declined in §5.3 with its reasons.
- **3B Non-goal respect.** No CLAP/LV2/AU hosting, no plugin-format authoring, no cross-DAW
  preset or project compatibility, no cloud service or content store, no video scoring
  (`vision.md:53–59`). The fixture is Spectre's own format, exercised by Spectre's own codec;
  nothing in this feature reads or writes any other product's data.
- **3C Deliberately small first devices.** R4-9 renders `Filament` and `Gloam` exactly as R4-6
  ships them and asks for no capability either does not have. Decision 15 (`decision-gates.md:39`)
  is respected by omission: this spec proposes no parameter, no voice count, and no feature for
  either device.
- **3D Originality.** The fixture is original content; the record schema is R4-3's, which is
  Spectre's own; the FNV walk is the existing one at `crates/spectre-offline/src/lib.rs:271–278`
  rather than a new comparison. No numeric limit is taken from any reference product — the one
  bound this spec introduces is derived in §4.2 from R4-6's own criterion and from the existing
  block geometry.
- **3E Platform commitment.** This is the criterion the feature engages most directly and the
  one it must not paper over. §4.6 states that Flow A is platform-independent and Flow B's Linux
  column is blocked by a strict superset of what blocks R4-3; §4.4 rule 7 and §5.6 item 2 forbid
  a combined verdict; §8 Q1 routes the consequence to Jeff rather than assuming it. The Linux
  path is named, not assumed, and no Linux claim appears anywhere in this document.
- **3F Accessibility trajectory.** §3.7 and §5.4 rows 12–13 make the decision-17 scoped audit a
  recorded part of every run, while §3.4 refuses to fix a shortcut map. The protocol records
  reachability and labels; it designs nothing that forecloses keyboard-complete operation,
  because it designs no surface at all.

---

## 7. Gap Analysis vs. Current State

### 7.1 What exists today

**Stated first and without qualification: nothing this feature tests has been built.** All
eight sibling features — R4-1 through R4-8 — are **spec'd and not implemented**. `./spectre`
produces no sound of any kind. There is no track model in the audio path, no clip, no
`Filament`, no `Gloam`, no filesystem save, and no bounce. A run of this protocol performed
today would record `BLOCKED` on every row that depends on a sibling, which is most of them.

| Thing | State | Evidence |
|---|---|---|
| `docs/05-quality/` directory | **absent** | `docs/` contains `00-product/`, `01-requirements/`, `02-reference-research/`, `03-architecture/`, `06-plans/`, `status/`, and `README.md`. `docs/README.md:50` says quality directories are created only when they hold grounded contracts |
| Any end-to-end fixture or QA protocol document | **absent** | No file in `docs/` or `crates/` names one |
| `crates/spectre-offline/tests/e2e_alpha.rs` | **absent** | `crates/spectre-offline/tests/` contains exactly `harness.rs` |
| `crates/spectre-offline/tests/fixtures/` | **absent** | The only checked-in project fixture in the workspace is `crates/spectre-project/tests/fixtures/r1-canonical.json` |
| Automated GUI-driving harness | **absent** | `crates/spectre-app/tests/` contains exactly `app_model.rs` and `smoke_cli.rs` (§5.3) |
| `./spectre` producing sound | **absent** | `crates/spectre-app/Cargo.toml:12–18` does not depend on `spectre-audio`. `docs/status/STATUS.md` §"Repository state": the prototype "has no audio thread, and Play changes the model's transport state without producing sound" |
| Track model in the signal path | **absent** as a signal concept; **implemented** as a presentation row | `AppModel::add_track` exists at `crates/spectre-app/src/lib.rs:405–422` and is covered by tests at `crates/spectre-app/tests/app_model.rs:33–49`; a track sidebar exists at `crates/spectre-app/src/main.rs:115–160`. R4-4 §7.1 establishes that `TrackView`'s `muted`/`solo` booleans reach no signal path. **Both halves of this row matter**: `criteria.md`'s own corrected passage (`:100–123`) records that a discarded spec got this wrong in both directions |
| MIDI clips | **absent** | R4-5 is spec'd; nothing named a clip exists in any `src/` tree |
| `Filament`, `Gloam` | **absent** | R4-6 is spec'd. `spectre-dsp` ships `ToneSource`, `PulseInstrument`, `Gain`, and `Saturator`, which R4-6 explicitly does not supersede |
| Filesystem save / open | **absent** | `crates/spectre-app/Cargo.toml:16` declares `spectre-project` and no file under `crates/spectre-app/src/` or `crates/spectre-app/tests/` references `spectre_project`; the app declares the persistence crate and never uses it |
| Offline bounce | **absent** | `crates/spectre-offline` renders single quanta only: `render_vertical_slice` (`src/lib.rs:288–303`), `render_app_snapshot` (`:306–316`), `render_silence` (`:319–334`), each calling `render_plan` (`:204–285`) once. There is no multi-quantum render and no file writer |
| Runtime parameter application | **absent, and counted** | `crates/spectre-audio/src/bridge.rs:181` drains the parameter lane into `\|_, _\| {}` and `:182–186` increments `parameters_pending`. The comment at `:178–180` states why |
| Offline deterministic self-test | **implemented and runnable** | `cargo run --locked -p spectre-offline -- --self-test` (`.github/workflows/ci.yml:49–50`); run 2026-08-21, prints the `OfflineReport`. It **inspects** the default project (`src/lib.rs:166–175`) and renders nothing |
| Process smoke test | **implemented and runnable** | `crates/spectre-app/tests/smoke_cli.rs`; `./spectre --smoke-test` run 2026-08-21, printed `Spectre prototype ready lens=Arrange tracks=1 transport=stopped selected_device=Pulse(pulse)` |
| The FNV-1a walk the fixture reuses | **implemented** | `crates/spectre-offline/src/lib.rs:271–278`, offset basis `0xcbf2_9ce4_8422_2325` at `:271` and prime `0x0000_0100_0000_01b3` at `:276`; the interleaved variant is `crates/spectre-audio/tests/bridge_plan.rs:80–92` |
| Bridge telemetry the record transcribes | **implemented** | `BridgeTelemetry`'s eleven counters at `crates/spectre-audio/src/bridge.rs:24–39`, accessors at `:63`–`:113`; `worst_headroom_bits` starts at infinity (`:55–56`) so the first block establishes the real minimum |
| RT-001 structural scan | **implemented** | `crates/spectre-audio/tests/rt_guard.rs:289–320`; the scanned set at `:293–298` is `src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, `src/null.rs` — **four modules, and `midi.rs` is not among them** |
| Linux device qualification | **gated, never run** | `docs/06-plans/current-milestone.md:114` records `\| Linux \| not run \|`; `:127` records "No Linux audio device has ever been opened"; decision 23 (`decision-gates.md:49`) carries the debt to R4 |
| macOS device qualification | **verified once, from a test binary** | `current-milestone.md:113`: cpal/CoreAudio, M-Audio AIR 192\|6, 173 callbacks, 0 xruns, worst headroom 0.990. Driven by `hardware_lifecycle_drill` (`crates/spectre-audio/tests/lifecycle_health.rs:245–295`, `#[ignore]` at `:246`), never from `./spectre` |
| Workspace gate | **implemented** | `.github/workflows/ci.yml:40–47`; `cargo fmt --all -- --check` re-run clean 2026-08-21. Last recorded full result: 230/230 with one ignored hardware drill, 2026-08-09 (`docs/01-requirements/traceability.md:53`) |

**Sibling spec state, as of 2026-08-21.** All eight have been written and all eight have passed
blind verification: R4-1 (2.950, iteration 2), R4-2 (3.000), R4-3 (2.967, iteration 2,
**hardware-blocked**), R4-4 (2.810), R4-5 (2.798), R4-6 (2.864), R4-7 (3.000), R4-8 (2.848,
remediation 1 applied). **A spec pass is not an implementation**: `gauntlet-output/manifest.md`'s own state machine separates
`spec-pass` from `implementation-in-progress`, and every one of the eight sits at implementation
iteration 0. This spec cites sibling specs as design authority for what R4-9 must exercise, and
cites **source paths** for every claim about what exists.

**Two sibling defects are still owed and are load-bearing for this feature**, per the
scorecards rather than the specs' self-assessment:

- **R4-4's §7.1 carries a factual correction that has not landed.** Its scorecard's Priority 1
  item 1 records that the `grep -rn track crates --include='*.rs'` transcript printed in R4-4
  §1.2 and §7.1 does not reproduce: it also returns `crates/spectre-project/src/lib.rs:162` and
  `:170`, both the string `future_track_kind` inside an unknown-field-preservation test. The
  **conclusion** — no track concept outside `crates/spectre-app/` — survives. R4-9's fixture
  depends on R4-4's track model, not on that transcript, so the correction does not change this
  spec; it is recorded here so a reviewer of R4-9 does not inherit the error.
- **R4-8's remediation is applied but its Priority 1 item 5 is a design choice R4-9 depends on.**
  The `bridge_plan.rs:80–92` walk stays an independent implementation rather than delegating to
  a shared one. §5.2 I-3 hashes the live side with that function precisely because it is
  independent; if it later delegates, I-3 becomes a comparison of one implementation against
  itself and must be re-argued.

### 7.2 Delta to spec

**New files**

1. `crates/spectre-offline/tests/e2e_alpha.rs` — Flow A: U-1…U-3 and I-1…I-12.
2. `crates/spectre-offline/tests/fixtures/r4-alpha.json` — the fixture project, content per
   §4.2. **Authored last**, after R4-4, R4-5, R4-6, and R4-7 land, because its encoding depends
   on their schemas (§8 Q4).
3. `docs/05-quality/r4-qa-protocol.md` — Flow B, layout per §3.3. New directory
   `docs/05-quality/`, authorized by `docs/README.md:50`.
4. `docs/05-quality/r4-qa-records.md` — the append-only record file, empty state per §3.3.

**Modified files**

5. `crates/spectre-offline/Cargo.toml` — one line under `[dev-dependencies]`:
   `spectre-audio = { path = "../spectre-audio" }`, alongside the existing `spectre-app` entry
   at `:20–21`. §4.1 records the mutual dev-dependency cycle this creates and how it was
   verified.
6. **`docs/01-requirements/requirements-ledger.md`** — **PROD-003 (`:64`) compels this and it
   is not optional.** One numeric row plus three normative rows in the `QUAL` family R4-3
   proposes. R4-3 proposes `QUAL-001`, `QUAL-002`, and `QUAL-003`, so R4-9 continues at
   `QUAL-004`; if R4-3's rows land under different IDs these renumber, and §8 Q7 asks Jeff who
   owns the family's numbering:
   - **`QUAL-004`** *(the only numeric row)* — The R4 end-to-end fixture MUST render at least
     **80** blocks of 256 frames, composed as 16 note blocks plus 64 tail blocks.
     *Rationale, recorded in the row itself:* the tail term is **inherited from R4-6's
     return-to-silence criterion of 64 render quanta after the last note-off** and MUST move
     with it if that criterion changes; the note term is the smallest power-of-two span that
     places a same-frame note-off/note-on pair away from the first and last blocks. 16 + 64 = 80;
     at 256 frames that is 20,480 frames, and at 48,000 Hz, 426.67 ms. The block size and sample
     rate are **not** new bounds — they are reused from
     `crates/spectre-audio/tests/lifecycle_health.rs:22–23` so the e2e record is directly
     comparable to R4-3's qualification record on the same host.
   - **`QUAL-005`** *(normative)* — Live↔offline equivalence MUST be asserted at equal block
     size. *Rationale:* RT-003 containment silences a whole render quantum
     (`crates/spectre-graph/src/lib.rs:517–518`), so unequal geometries legitimately differ on
     a contaminated render.
   - **`QUAL-006`** *(normative)* — The e2e fixture MUST NOT assert a checked-in cross-platform
     golden hash while cross-platform bit-equality is undecided; the observed hash is recorded
     per platform instead. *Rationale:* R4-6 §8 Q5 — device math routes through libm.
   - **`QUAL-007`** *(normative)* — A QA run record MUST carry a per-platform outcome word and
     MUST NOT carry an aggregate one. *Rationale:* decision 1 (`decision-gates.md:25`) and
     decision 23 (`:49`).
   Only `QUAL-004` is a numeric bound; the other three are normative acceptance rules proposed
   for the same family, and Jeff may decline them without affecting PROD-003 compliance.
7. `docs/06-plans/current-milestone.md` — §"Exit evidence" (`:79–90`) gains a verdict marker per
   row, written by the protocol. **No exit row's text is changed and no row is removed.**
    The marker follows the precedent the same document already sets for R3's ten rows, which are
    struck through with a closing date and a one-line result (`current-milestone.md:94–107`);
    recording a verdict against a row is evidence, not amendment. If Jeff answers §8 Q1 by
    narrowing R4's exit, that narrowing is a decision-gates row in the shape of decision 23,
    not an edit to this list by a spec.
8. `docs/01-requirements/traceability.md` — one row for R4-9's artifacts once they exist, and
   updates to whatever rows the first real run contradicts (exit row `:90`).
9. `docs/status/STATUS.md` and `docs/status/NEXT.md` — updated when slice 9 lands, per the
   working rule at `docs/README.md:68–70`.
10. `docs/README.md` — the class table at `:45–46` already names `05-quality/`; no edit is
    needed, and none is proposed. Recorded here so a reader does not assume one was missed.

**Migrations / schema changes:** none owned by this feature. The fixture's envelope version
follows R4-7 (§8 Q4).

**New dependencies:** none. One path dev-dependency between two existing workspace members
(item 5).

### 7.3 Estimated scope

**M**, with an unusual shape: the writing is small and the *sequencing* is the cost.

- Flow A is one test file of roughly 15 assertions over APIs other slices build, plus a fixture
  file. Neither involves new logic. That is **S**.
- Flow B is two documents, one of which is a table this spec already contains. That is **S**.
- What makes it **M** is that R4-9 is the only feature that cannot be written to completion
  before the features it tests exist. The fixture's encoding depends on four sibling schemas;
  the manual rows describe surfaces none of which have been built; and the run that produces the
  evidence needs an operator, an interface, and — for one platform — a machine this project does
  not have. §8 Q11 asks how that re-reading pass is scheduled.

It is emphatically **not L**: no GUI harness (§5.3), no new crate, no new dependency, no CI
change, no new render path, and nothing on a callback-reachable path.

### 7.4 Blocking dependencies

| Blocker | What it blocks | State |
|---|---|---|
| **R4-1** live-audio-wiring | Flow B rows 1–6; I-3's live half | spec-pass 2.950, **not implemented** |
| **R4-2** runtime-parameter-seam | I-8; Flow B row 3 | spec-pass 3.000, **not implemented**; **its implementation is additionally blocked by D-R3** (`gauntlet-output/decisions-needed.md:102`) |
| **R4-3** linux-device-qualification | Nothing in Flow A. **The entire Linux column of Flow B**, indirectly — see §4.6 | spec-pass 2.967, **hardware-blocked**, never run |
| **R4-4** track-model | The fixture's three tracks; I-6, I-7 | spec-pass 2.810, **not implemented** |
| **R4-5** midi-clips | The fixture's clips; I-1 | spec-pass 2.798, **not implemented** |
| **R4-6** first-devices | The fixture's devices; I-10; `QUAL-004`'s tail term | spec-pass 2.864, **not implemented** |
| **R4-7** project-persistence | I-4, I-5; Flow B rows 7–8; the fixture's envelope version | spec-pass 3.000, **not implemented**; **its landing is gated on its own Q2**, which asks whether a schema bump disturbs an acceptance test of CORE-003 — a `verified` requirement (`requirements-ledger.md:48`) |
| **R4-8** offline-bounce | I-11; Flow B row 9 | spec-pass 2.848, remediation 1 applied, **not implemented** |
| **A Linux host with a real ALSA device** | Flow B's Linux column entirely | Has never existed in this project |
| **An audio interface for the operator** | Flow B rows 1–3, 5–6, 9 at full fidelity | macOS: available (`current-milestone.md:113`). Linux: unavailable |
| **§8 Q1** (what R4's exit means with row `:88` open) | The *meaning* of any verdict this feature produces | Open, and it is Jeff's |

**Sequencing consequence.** R4-9 is the last leaf to implement as well as the last to spec.
Flow B's document can be drafted as soon as R4-1 and R4-4 land, because most of its rows describe
surfaces those two introduce; Flow A cannot compile until all eight are in. A partial Flow A that
tests only the landed subset is possible and is **not** proposed here, because a fixture that
changes shape every slice is a fixture nobody can compare across runs — §8 Q9 asks whether Jeff
wants one anyway as an interim signal.

---

## 8. Open Questions

- **Q1 — What does R4's exit mean while the Linux qualification row is open?** *The sharpest
  question in this spec, and the one that decides whether any verdict R4-9 produces means
  anything.* R4's exit evidence is a ten-row conjunction (`docs/06-plans/current-milestone.md:79–90`)
  and its §"Inherited debt" (`:20–27`) says at `:22` of the four carried obligations: *"None is optional
  and none should be rediscovered later."* Row `:88` — Linux device qualification — is closable
  only by running R4-3's drill on a Linux host with a real ALSA device, which has never existed
  in this project, and decision 23 (`decision-gates.md:49`) authorizes no Linux support claim
  until it does. Meanwhile R4-9's own Linux column is blocked by a strict superset of that
  (§4.6). Three dispositions, and this spec takes none of them:
  **(a) Strict conjunction** — R4 does not exit until the Linux drill runs. Consistent with
  decision 1 and with the milestone's own "none is optional", and it means R4 can be nine-tenths
  done and still open on a hardware purchase.
  **(b) A second scoped narrowing** — R4 exits on macOS alone, recorded as a new decision row in
  the shape of decision 23. That would be the **second consecutive milestone** to narrow decision
  1's co-first-class commitment, and decision 1 would then have been deferred at every milestone
  that could have discharged it. If this is the answer it should be a decision row with that
  consequence written into it, not an assumption inside a spec.
  **(c) Split the exit** — nine rows close, the milestone stays formally open on row `:88`, and a
  named state (`R4-exit-pending-Linux`) is added to the status vocabulary so `STATUS.md` can say
  it in one word.
  Until Jeff answers, §4.4 rule 7 and §5.6 item 2 hold: R4-9 records per-platform outcomes and
  writes no aggregate word, which is the only behavior consistent with all three dispositions.
  **Blocks:** the meaning of §5.5's outcome table and every use of this feature's result.

- **Q2 — D-R3: does `Gain` owe smoothing, and does that change what Flow B row 3 listens for?**
  `gauntlet-output/decisions-needed.md:102` D-R3 records that `docs/03-architecture/dsp-device-io.md`
  asserts twice that `Gain` smooths while the shipped `Gain` is a single `f32` with an
  instantaneous clamped setter, and it blocks R4-2's implementation. R4-9 does not resolve it and
  must not: Flow B row 3 asks the operator to record *whether the sound changed*, deliberately
  **not** whether it changed without a click, because "no click" is only a defensible check once
  Jeff has decided which of D-R3's three options governs. If option 2 or 3 lands, row 3 gains a
  click check and `QUAL-004`'s note span may need to be long enough to hear a ramp.
  **Blocks:** §5.4 row 3's wording.

- **Q3 — R4-7's Q2: a `verified` requirement's acceptance test.** R4-7's Q2 asks whether
  retargeting `canonical_fixture_rewrite_is_byte_stable` is acceptable when it is acceptance
  evidence for CORE-003, which is `verified` (`requirements-ledger.md:48`). This matters to R4-9
  twice over: the e2e fixture is encoded in whatever envelope R4-7 lands, and Flow B row 15 asks
  the operator to check that traceability matches implementation — which would have to record a
  `verified` requirement quietly losing its evidence. R4-9 routes it and asserts nothing.
  **Blocks:** §4.2's schema-version row and §5.4 row 15's expected findings.

- **Q4 — Which schema version does the checked-in fixture carry, and who re-authors it when the
  schema moves?** Following from Q3. A fixture pinned to schema 1 becomes a migration test the
  moment R4-7 bumps; a fixture that floats with the current version tests nothing about
  compatibility. R4-9's default is that the fixture carries the **current** version and that
  compatibility fixtures are R5's, per decision 14 (`decision-gates.md:38`) putting migrations
  and recovery there — but this is Jeff's to confirm, because it decides whether an R4 fixture
  becomes an R5 migration input. **Blocks:** §4.2's first row, §7.2 item 2.

- **Q5 — Mutual dev-dependency cycle, or a test-only crate?** §4.1 places Flow A in
  `crates/spectre-offline/tests/` and adds `spectre-audio` to that crate's dev-dependencies,
  creating a mutual dev-dependency cycle with `crates/spectre-audio/Cargo.toml:23–24`. The cycle
  is legal — verified with a two-crate probe workspace outside this repository, not assumed — and
  it costs one manifest line. The alternative is a new test-only workspace member
  `crates/spectre-e2e` that depends on all seven crates and that nothing depends on: no cycle to
  reason about, at the cost of an eighth crate whose only content is tests, and with the downside
  that an e2e failure no longer blocks `cargo test -p spectre-offline`. This spec chooses the
  cycle because it adds nothing to `Cargo.toml:8–16`; Jeff may prefer the explicit crate.
  **Blocks:** §4.1, §7.2 item 5.

- **Q6 — Is "a muted track hashes equal to its absence" (I-7) a correct assertion?** It asserts
  R4-4's claim that a muted track contributes exact zero. R4-9 does not independently establish
  it, and it is false under at least two plausible R4-4 outcomes: a mute implemented as a very
  low gain rather than a hard zero, or additive solo semantics that make "muted" depend on the
  solo set (R4-4's §8 Q10 leaves solo additive-versus-exclusive open, and `OBS-AB12-MIX-003`
  records that Live's solo and arm are exclusive by default with overrides). If R4-4 lands
  either, I-7 is deleted or re-stated. **Blocks:** §5.2 I-7 only.

- **Q7 — Who owns the `QUAL` requirement family's numbering?** R4-3 proposes `QUAL-001`…`QUAL-003`
  and R4-9 continues at `QUAL-004`…`QUAL-007`. Two specs allocating IDs in one family without a
  registry is how collisions happen, and R4-3's rows are not landed either. Either Jeff assigns
  the numbers at merge time, or the family gets an owner. **Blocks:** §7.2 item 6's row IDs, not
  their content.

- **Q8 — Should the run record live in `docs/05-quality/` or alongside the milestone?** R4-3 puts
  its qualification record **inside** `docs/06-plans/current-milestone.md` as a table plus a
  numbered environment block, because that table already exists there. R4-9 puts its records in a
  separate `docs/05-quality/r4-qa-records.md` because a full run record is a screen of Markdown
  and would swamp the milestone document. The cost is two record locations with one schema. The
  alternative is to move R4-3's record into `05-quality/` too and leave the milestone with a
  pointer. **Blocks:** §3.1, §7.2 items 3–4.

- **Q9 — Does Jeff want a partial Flow A before all eight siblings land?** §7.4 argues against a
  fixture whose shape changes every slice, because runs stop being comparable. The counter-argument
  is real: the first slice to land is R4-1, and a two-assertion e2e that only checks "the engine
  opens and the hash matches offline" would catch composition breakage months earlier than a
  complete fixture. If yes, the interim fixture needs its own name so it is never mistaken for the
  R4 exit artifact. **Blocks:** nothing today; it changes sequencing only.

- **Q10 — Who is the operator, and is a second one required?** This spec names Jeff, because Jeff
  is the sole decision authority in every metadata block in this repository and no second person
  exists in the project record. That is honest but it means the author of a slice is also the
  verifier of its manual evidence — the exact separation `gauntlet-output/manifest.md`'s state
  machine enforces for specs ("an author never verifies its own artifact") and which cannot be
  enforced here with one person. Options: accept it and record it as a known limitation of every
  run; or require that the manual protocol be run against a build the operator did not just
  write, with the commit SHA in E7 as the check. **Blocks:** §3.2 B, and the credibility of every
  record.

- **Q11 — When does the protocol get re-read against the real surfaces?** Every §5.4 row was
  written before the surface it checks existed. Some rows will be wrong — naming a control that
  ends up elsewhere, or missing a state a slice introduces. This spec proposes that the protocol
  document is re-read and amended as part of the **last** sibling slice to land rather than as a
  separate pass, so it cannot be forgotten; but that puts a documentation edit inside someone
  else's slice. **Blocks:** nothing now; it is a process question that decides whether the
  protocol rots (§5.6 item 10).

- **Q12 — Is a benchmark research need worth opening for QA methodology?** The accepted corpus has
  **no** record about how any of the five benchmark products tests itself, defines an acceptance
  fixture, or records a qualification run — Appendix A names the hole. Unlike the loudness gap,
  which reading a standard closed, this one is mostly not public: internal QA practice is not
  documented in user manuals, which is what the corpus is built from. The honest answer may be
  that this surface is permanently uncitable and Spectre's protocol is simply Spectre's own.
  Recording that conclusion is worth more than an open research task that cannot be discharged.
  **Blocks:** nothing.

---

## Appendix A — Benchmark evidence used, and where it does not exist

**The primary finding of this appendix is a hole, and naming it is the correct behavior under
criterion 2G rather than a shortfall.**

**No citable observation exists, for any of the five benchmarks, about:** how the product is
tested; what its acceptance fixture is; how it defines a release-qualification run; what it
records about a tested host; what its internal QA protocol contains; or how it decides that a
build is shippable. The corpus is built from user-facing manuals and support material
(`docs/02-reference-research/methodology.md`), and a manual documents what the product does for
the user, not how its makers verify it. Per criterion 2G, the correct output here is
"no citable evidence for any benchmark on this surface; recorded as a research need", and §8 Q12
records the further judgement that this particular hole may be permanently unfillable from public
sources.

The evidence inventory the criteria fix (verified 2026-08-14) is unchanged and constrains this
appendix: Ableton Live 12 has 85 `OBS-AB12-` records; Phase Plant 11; VCV Rack 2 six; **Serum 2
exactly two**, `OBS-SR2-CPU-001` and `OBS-SR2-KB-001`, both confirmed present at
`docs/02-reference-research/synth-modular-observations.md:54–55`; **Logic Pro zero**. **No Logic
Pro claim appears anywhere in this spec**, and no Serum 2 claim beyond those two records appears
anywhere in it. The wider Serum 2 extraction is quarantined under D-R2 and is not used.

**The three records this spec does use**, each for a user-observable behavior the fixture or the
protocol has to account for:

1. **`OBS-AB12-MIX-002`** (Live 12 manual §18.1.1): the 32-bit floating-point engine tolerates
   over-0 dB levels between tracks without clipping, and clipping matters only at physical
   outputs, the main output, or file export. Used in §5.4 row 9's framing: the peak that matters
   for a bounce is the one at the written file, not one measured between tracks. Spectre converges
   with this — its plan is `f32` end to end — so no divergence rationale is owed (criterion 2E).
2. **`OBS-AB12-MIX-009`** (§18.9): Live surfaces a per-track performance-impact indicator.
   Corroborating evidence that surfacing render cost to the user is a convergent pattern, which is
   why §3.3's record transcribes Spectre's headroom telemetry rather than discarding it.
   **No threshold is derived from it**: AF-5 and `product-implications.md:99` prohibit a
   monitoring-latency threshold, and §4.7 states no timing gate.
3. **`OBS-AB12-MIX-003`** (§18.1): Live's solo and arm are exclusive by default, with modifier or
   preference overrides. Cited in §8 Q6 only, as the reason a muted-versus-deleted hash equality
   depends on a solo semantic R4-4 has not settled. **This is a citation about a benchmark, not a
   recommendation**: R4-4 owns Spectre's solo semantics and §8 Q6 routes rather than argues.

**Records deliberately not used.** `OBS-VCV-VOLT-006` (modules output 0 on NaN/infinity) is the
cited precedent for RT-003 in `requirements-ledger.md:30` and is genuinely adjacent to §5.2 I-9's
containment assertion — but RT-003 is already accepted and implemented, and re-citing its
provenance here would be decoration rather than evidence, so I-9 cites the requirement and the
counter accessor instead. `OBS-SR2-CPU-001` is about unison voice counts and CPU and has no
bearing on a QA protocol; it is named here only to make the two-record Serum 2 budget visible and
demonstrably unspent.

**Differentiation (criterion 2F).** What Spectre does here that the benchmark set is not
documented as doing — stated as a claim about Spectre, not as a deficiency claim about products
whose internal practice the corpus cannot see: Spectre's QA record names **what was not checked**
in a required field (Q-D), refuses to aggregate a two-platform result into one word (§4.4 rule 7),
and publishes the list of things a passing run does not authorize (§5.6) inside the same document
that records the pass. That is the vision's "honest telemetry, no fake surfaces"
(`docs/00-product/vision.md:48`) applied to the project's own evidence rather than only to the
product's UI. It is not a parity claim, and the corpus supports no comparison either way.

---

**End of spec.**
