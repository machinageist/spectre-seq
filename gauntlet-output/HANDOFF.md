<!--
Author: Jeff
Date: 2026-08-15
Description: Resume contract — the next executable action for an agent with no conversation history
Notes: Every agent updates this before stopping, including after failures
-->

# Spectre Gauntlet Handoff

- **Status:** accepted
- **Last verified:** 2026-08-28
- **Scope:** current gauntlet position and the exact next action, for the R4 loop and the mixing/mastering loop
- **Decision authority:** Jeff
- **Upstream sources:** `manifest.md`, `criteria.md`, `feature-tree.md`, `criteria-mixing-mastering.md`
- **Downstream dependents:** the next session, whoever runs it
- **Supersedes:** the earlier 2026-08-15 handoff
- **Open decisions:** D-R1, D-R2, D-R3, D-R4, D-MM1–D-MM4 in `decisions-needed.md`
- **Known gaps:** the R4 QA protocol's manual half has never been run; the two open R4 exit rows both reduce to it

**Current state: all nine features are implemented, and eight of R4's ten exit rows are closed.**
All nine R4 specs hold a verdict and all nine have code as of 2026-08-28. 460 workspace tests pass with `cargo fmt`, strict Clippy, and
`cargo test` all clean, **on Linux** — which had never once been true before this date.

Commits, newest first: `1f4314c` track insert slot, `af654e5` alpha headroom, `ec465a0` CI +
R4 exit rows, `43a2403` R4-9 protocol and golden-hash removal, `09b6d6d` R4-3, `6f37c98` Linux
build, then `11bbbdb` R4-9, `a7aff87` R4-7, `136a227` R4-8, `deab408` site, `d8c2c87` +
`487bc49` R4-5, `1960a8b` + `0d2a03c` R4-6/R4-4.

**The three things this handoff listed as open are closed, and how they closed matters.**

1. ~~**The live bridge addresses one instrument.**~~ Lifted 2026-08-28. `RenderBridge` carries a
   fixed `Box<[ClipVoice]>` and `the_live_bridge_plays_every_instrument_track` replaces the test
   that pinned the limit.
2. ~~**R4-3 needs a Linux box.**~~ Discharged 2026-08-28. **This handoff said the hardware "has
   never existed in this environment"; the development host is Arch Linux with three audio
   cards and always was.** What was missing was anything that ran against one: the workspace did
   not compile on Linux at all — `spectre-app` named neither `x11` nor `wayland` with `eframe`'s
   default features off — and CI had been failing on exactly that since 2026-08-06 while
   watching only `main`, which no R4 slice ever touched. It was also not "one command": the
   drill first exposed a real `find_device` defect on ALSA.
3. **The R4 QA protocol's manual half still has not been run.** Its automated half has, on
   Linux, and `r4-qa-records.md` carries block Q-1 at `INCONCLUSIVE` with thirteen rows `NOT
   RUN`. This still needs a human at the machine, not an agent.

**Four requirements were found unimplemented or misclaimed while closing the above**, all by
checking a spec or an exit row against the path the product actually takes:

1. **R4-9 §"Devices per track"** — one `Filament` and one `Gloam` insert per track. The router
   wired instrument → gain and constructed no effect at all, so R4-6's effect reached no render.
2. **R4-2's parameter seam** — closed against `build_engine_parts`, whose targets derive from the
   model's own snapshot. `main.rs` opens `open_track_engine`, whose targets are graph ObjectIds,
   so no Shape edit reached live audio in the product. Fixed by resolving an edit's destination
   by role on the selected track; R4-4 §8 Q2's device-browser re-parenting stays R6 work.
3. **R4-5 §3.1's five surfaces** — clip lane, inspector, note list, track row, transport readout.
   None were drawn; Arrange showed a hardcoded rectangle matching no project data, and the whole
   clip API had zero callers outside tests.
4. **R4-1's driver evidence** — the cited drill opens `open_default`, which the binary never
   calls, and no drill asserted the render carried any signal at all.

Every one had a passing test and a verified scorecard. The lesson for the next run is that all
nine scorecards graded a **proposal**; nothing in the loop checked that a shipped feature
satisfied the spec that commissioned it, and in each case a two-line grep would have found it.
`decisions-needed.md` D-R5 carries the process question.

**Exact next action, in order:**

1. **Run the QA protocol's manual half**, on this host, and write block Q-2 into
   `r4-qa-records.md`. Per-platform outcome word; no aggregate. **This is the only work left on
   R4's exit: eight of ten rows are closed, and both open rows reduce to this one activity.**
   Rows 1 and 7 need an operator to confirm that signal reaching the driver reaches the speakers.
   `./spectre` needs `pipewire-alsa` installed on this box or an `ALSA_CONFIG_PATH` override —
   §Procedure step 4 carries both.
2. **Jeff answers D-R3** — the accepted device contract says `Gain` smooths and the shipped
   `Gain` does not. R4-2 declined to change either side by assertion, and it is still open.
3. **Jeff rules on the `dsp-device-io.md` determinism clause**, which asserts bit-identical
   offline output unconditionally while RT-003 containment is scoped to the quantum. R4-8
   recorded the narrowing rather than editing an accepted contract.
4. **Decide whether the alpha's headroom is acceptable.** The plan the product runs takes about
   60% of its callback budget at the median on this host (0.404 worst-case headroom, worst
   sample 0.165), against the 0.834 the three-node qualification chain implied. 0 xruns
   throughout, so it is not unsafe here — but every headroom figure before 2026-08-28 described
   a workload the alpha does not have.
5. **Decide the citation-drift rule** the manifest records: specs pin lines that are true at one
   commit, and this run implemented past those commits. Either freeze the gauntlet while
   implementation runs, or cite by symbol and quoted literal. R4-7's iteration-2 reviewer
   recommends the latter as its "single most important fix".

