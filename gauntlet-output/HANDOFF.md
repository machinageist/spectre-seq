<!--
Author: Jeff
Date: 2026-08-15
Description: Resume contract — the next executable action for an agent with no conversation history
Notes: Every agent updates this before stopping, including after failures
-->

# Spectre Gauntlet Handoff

- **Status:** accepted
- **Last verified:** 2026-08-15
- **Scope:** current gauntlet position and the exact next action, for the R4 loop and the mixing/mastering loop
- **Decision authority:** Jeff
- **Upstream sources:** `manifest.md`, `criteria.md`, `feature-tree.md`, `criteria-mixing-mastering.md`
- **Downstream dependents:** the next session, whoever runs it
- **Supersedes:** the earlier 2026-08-15 handoff
- **Superseded by:** none
- **Open decisions:** D-R1, D-R2, D-MM1–D-MM4 in `decisions-needed.md`
- **Known gaps:** four specs are written and **none of the four has been verified**; that is the whole of the next action

**Current state:** three spec-passes — R4-1 **2.950**, R4-2 **3.000**, R4-7 **3.000**.
R4-3 and R4-4 are **spec-review, iteration 1, awaiting a verifier**. R4-5, R4-6, R4-8, R4-9
unstarted.

> **Blocked on a usage cap until 23:10 America/Los_Angeles on 2026-08-15.** The cap has
> fired three times today. Re-dispatch after the reset.

**Exact next action, in order:**

1. **Dispatch two blind verifiers — R4-3 and R4-4 — at concurrency 2.** Both must be
   **fresh agents**, and the reason is a conflict of interest that must not be papered over:
   - **R4-3** — the session that would otherwise grade it *read the author's full findings
     report* before the author was killed. That is knowledge of the author's reasoning, which
     the blind-review rule exists to exclude.
   - **R4-4** — the same session **briefed that author with a factually false claim** (that
     `AppModel::add_track` does not exist; see below). The spec's most notable feature is
     that it refused that claim in writing and was right. Grading a spec whose headline
     virtue is catching your own error is not a blind review, and the bias runs toward
     leniency.

   Each verifier gets: the spec, `criteria.md`, `templates/SCORECARD-TEMPLATE.md`, and the
   repository. **No verifier may receive any part of its author's reasoning or report.**
   - `specs/R4-3-linux-device-qualification.md` → `spec-scorecards/R4-3-linux-device-qualification-scorecard.md`
   - `specs/R4-4-track-model.md` → `spec-scorecards/R4-4-track-model-scorecard.md`
2. **Optionally re-verify R4-2 and R4-7's sampled sections.** Both passed at 3.000 on
   exhaustive evidence-integrity checking plus **sampled** quality grading; each scorecard
   marks which criteria rest on a sample and says a fuller read may correct it. Not a gate.
3. **Remediate whatever fails**, max 3 rounds each, then escalate to `gap-reports/`. A fresh
   verifier each round — never the author, never the previous verifier.
4. **Then batch 3** — R4-5 (midi-clips), R4-6 (first-devices), R4-8 (offline-bounce) at
   concurrency 3. **Then R4-9 last**, which depends on every other leaf.
5. **D-R2, D-MM2, D-MM3 still need Jeff**; **D-R3 blocks R4-2's implementation**; **R4-7's
   Q2 must be answered before R4-7 lands.**

> **An accepted document was found wrong, and the finding matters more than the specs.**
> `criteria.md`'s AF-2 illustration and `manifest.md`'s 2026-08-12 entry both stated that
> `AppModel::add_track()` and a track list sidebar were invented by the discarded spec and
> that "none exist." **Both exist**, and have since commit `6c397d9` — `add_track` at
> `crates/spectre-app/src/lib.rs:405–422`, the sidebar at
> `crates/spectre-app/src/main.rs:115–160`, with two covering tests at
> `crates/spectre-app/tests/app_model.rs:33–49`. Only `Track::arm`/`Track::mute` were
> genuinely invented, so the discarded spec's disposition stands on AF-5 and corrupted
> encoding regardless. Both documents now carry the correction **in place**, deliberately
> unedited-away, because an accepted document misstating the codebase inside the rule that
> forbids exactly that is the best available argument for why AF-2 requires opening the file.

**The verifier's central obligation, restated because it is what this loop is for:** open
every source path the spec cites and check the claim against the file. R4-1 failed its
first review on exactly one false sentence about the codebase while scoring 2.750 and
clearing every numeric threshold. Recompute any arithmetic a spec performs.

**Last completed action:** four specs authored, the loudness-standards research pass
completed, and the standards findings integrated into the mixing/mastering criteria.

**Last verified revision/worktree state:** branch `rename/geist-to-spectre`. Nothing is
committed by this session; all work is in the working tree.

**Commands run and outcomes:**
- `cargo fmt --all -- --check` — clean.
- `cargo clippy --locked --workspace --all-targets -- -D warnings` — clean, exit 0.
- `cargo test --locked --workspace` — **230 passed, 1 ignored**, 0 failures. The ignored
  test is `hardware_lifecycle_drill`, which is R4-3. Matches `STATUS.md` exactly.
  **No Rust has been touched**, so the gate is a baseline, not evidence about any change.
- `source-ledger.json` integrity check against HEAD — 62 records, 0 duplicates, **0
  pre-existing records changed, 0 lost**. Two agents wrote to this file in the same
  session; the check confirms neither clobbered the other.

**Files changed since the last handoff:**

*R4 loop*
- `specs/R4-2-runtime-parameter-seam.md`, `specs/R4-3-linux-device-qualification.md`,
  `specs/R4-4-track-model.md`, `specs/R4-7-project-persistence.md` — **new**, iteration 1,
  all eight template sections filled.
- `manifest.md` — four features to spec-review; two cap events and the loudness pass
  recorded in run history.

*Mixing/mastering*
- `docs/02-reference-research/loudness-standards-observations.md` — **new**, 95 records.
- `docs/02-reference-research/source-ledger.json` — 59 → **62**; `SRC-ITU-BS1770-5` moved
  to `claims-extracted`, plus `SRC-EBU-R128`, `SRC-EBU-TECH3341`, `SRC-EBU-TECH3342`.
- `docs/02-reference-research/ozone.md` — standards boundary section rewritten, source
  matrix updated, `GAP-OZONE-0043` **closed** with two successor gaps.
- `docs/02-reference-research/external-reference-register.md` — standards rows updated;
  a new row for ITU-R BS.1771 / Report BS.2217, which are cited by the retrieved documents
  and still unread.
- `gauntlet-output/criteria-mixing-mastering.md` — **MM-AF-5 narrowed**, evidence inventory
  gained a standards row, criterion 1A gained the RT collision.
- `gauntlet-output/MM-GAUNTLET-PROMPT.md`, `gauntlet-output/decisions-needed.md` — D-MM4
  rewritten to a product question.

**Known failures/blockers:**
- **Usage caps are the dominant failure mode of this loop**, not technical error. Three
  terminations today. **Every agent brief must instruct the agent to write its artifact to
  disk before polishing** — the 13:15 dispatch lost five agents' work entirely, the 18:16
  dispatch lost none, and the brief change is the only difference.
- **R4-3 is hardware-blocked** — decision 23's Linux drill needs a Linux host with a real
  ALSA device. Its spec is written; only execution is blocked.
- **The MM gauntlet is decision-blocked** on D-MM2 and D-MM3, by design.

**Decision required:** yes, but nothing blocks the R4 loop.

> Verification can dispatch the moment the cap resets. Jeff is needed for D-R2 (Serum 2
> guide provenance), D-MM2/D-MM3 (whether the mastering suite is built, and where), and
> D-MM4, which is now a **product** question rather than a research gap: BS.1770-5 supplies
> K-weighting coefficients for 48 kHz only, so any other sample rate is an original Spectre
> engineering decision, and correct Integrated Loudness collides with RT-001/RT-002 such
> that a bounded approximation would be a declared deviation rather than conformance.

**Exact next command:** re-run the gate, then dispatch the four verifiers.

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

**Do not redo:**
- **The four specs.** They are complete and unverified. Verify them; do not rewrite them.
- **The R4-1 blind re-verification.** Complete and recorded at 2.950.
- **The loudness-standards reading pass.** Four documents read in full, 95 records. The
  remaining loudness gaps are `GAP-LOUDNESS-*` and are real source gaps, not unread pages.
- **The FabFilter and Ozone extraction.** 378 records with ledger backing.
- **The MM criteria interview.** Phase 0 for that loop is complete.
- **Discovering either feature tree from source.** The R4 tree is transcribed from the
  accepted slice queue in `docs/status/NEXT.md`; the MM tree lives in
  `criteria-mixing-mastering.md`. Inventing a tree from source is what broke run 1.
- **Re-copying templates from `gauntlet-active/` or `gauntlet-universal/`.** `templates/`
  is authoritative and tracked.
