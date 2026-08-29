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
- **Known gaps:** the R4 QA protocol's manual half has never been run; that is the only open R4 exit row

**Current state: all nine features are implemented.** All nine R4 specs hold a verdict and all
nine have code as of 2026-08-28. 450 workspace tests pass with `cargo fmt`, strict Clippy, and
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

**One requirement was found unimplemented while closing the above.** R4-9 §"Devices per track"
specifies one `Filament` instrument and one `Gloam` insert per track, and §Traceability asserts
Flow A renders through `Filament → Gloam`. `build_track_graph` wired instrument → track gain and
constructed no effect at all, so R4-6's effect reached no render and the R4-6 exit row overstated
what shipped. Closed by `1f4314c`. The lesson for the next run is that every one of the nine
scorecards graded a *proposal*; nothing in the loop checked that a shipped feature satisfied the
spec that commissioned it, and a two-line grep would have caught this one.

**Exact next action, in order:**

1. **Run the QA protocol's manual half**, on this host, and write block Q-2 into
   `r4-qa-records.md`. Per-platform outcome word; no aggregate. This is the only open R4 exit
   row. Note `./spectre` needs `pipewire-alsa` installed on this box, or an `ALSA_CONFIG_PATH`
   override — the host ships no `pcm.!default`.
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

