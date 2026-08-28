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
- **Known gaps:** the R4 QA protocol has never been run; the live bridge addresses one instrument; R4-3 needs Linux hardware

**Current state: the spec loop is finished and the implementation is eight of nine.**
All nine R4 specs hold a verdict. R4-1, R4-2, R4-4, R4-5, R4-6, R4-7, R4-8, and R4-9 are
implemented and committed; **R4-3 is hardware-blocked** and is the only feature with no code.
437 workspace tests pass with `cargo fmt`, strict Clippy, and `cargo test` all clean.

Commits, newest first: `11bbbdb` R4-9, `a7aff87` R4-7, `136a227` R4-8, `deab408` site,
`d8c2c87` + `487bc49` R4-5, `1960a8b` + `0d2a03c` R4-6/R4-4.

**Three things are open and none of them is a spec problem.**

1. **The R4 QA protocol has never been run.** `docs/05-quality/r4-qa-protocol.md` is written;
   `docs/05-quality/r4-qa-records.md` is at its empty state. R4's end-to-end exit row is half
   closed — the fixture passes, no operator has worked the manual rows. This needs a human at
   the machine, not an agent.
2. **The live bridge addresses one instrument.** `RenderBridge` carries one `note_node`, so a
   three-track project plays one track live while the offline path plays all three. Found by
   R4-9's composed fixture, which is what a vertical slice is for.
   `the_bridge_can_only_deliver_notes_to_one_instrument` pins it and fails the day it is lifted.
3. **R4-3 needs a Linux box.** One command; it only needs the hardware.

**Exact next action, in order:**

1. **Jeff answers D-R4** — what finishing R4 means while the Linux row is unclosable here.
   R4's exit is a ten-row conjunction whose own text says "None is optional", and the Linux row
   needs hardware that has never existed in this environment. The dispositions are in
   `decisions-needed.md`. Exiting on macOS evidence alone would make R4 the second milestone to
   do so, which is the option to avoid by default.
2. **Lift the one-instrument limit in `RenderBridge`**, or record it as accepted scope for the
   alpha. Today R4's "a MIDI clip plays through a track into master" row is closed offline and
   open in the product. Whichever way it goes, `the_bridge_can_only_deliver_notes_to_one_instrument`
   and `current-milestone.md`'s marker on that row must move together.
3. **Run the QA protocol on macOS** and write block `Q1` into `r4-qa-records.md`. Per-platform
   outcome word; no aggregate. A blank Linux column is valid; an omitted one is not.
4. **Run R4-3's drill on Linux hardware** when a box is available:
   `cargo test -p spectre-audio --test lifecycle_health -- --ignored --nocapture`.
5. **Decide the citation-drift rule** the manifest records: specs pin lines that are true at one
   commit, and this run implemented past those commits. Either freeze the gauntlet while
   implementation runs, or cite by symbol and quoted literal. R4-7's iteration-2 reviewer
   recommends the latter as its "single most important fix".

