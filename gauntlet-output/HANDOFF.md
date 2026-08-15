<!--
Author: Jeff
Date: 2026-08-14
Description: Resume contract — the next executable action for an agent with no conversation history
Notes: Every agent updates this before stopping, including after failures
-->

# Spectre Gauntlet Handoff

- **Status:** accepted
- **Last verified:** 2026-08-14
- **Scope:** current gauntlet position and the exact next action
- **Decision authority:** Jeff
- **Upstream sources:** `manifest.md`, `criteria.md`, `feature-tree.md`
- **Downstream dependents:** the next session, whoever runs it
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** criteria and tree sign-off
- **Known gaps:** none at the handoff layer

**Current feature:** R4-1 — live-audio-wiring
**Current state:** `spec-remediation-1`, owed. The spec is unmodified at iteration 1.
**Current owner role:** remediation agent (not yet run to completion)

> **Blocked on a usage cap until 05:30 America/Los_Angeles on 2026-08-15.** Two agents
> were terminated mid-run. Neither left a partial edit in the spec; both leave work
> owed. Re-dispatch after the reset.

**Exact next action, in order:**

1. **Re-dispatch R4-1 remediation iteration 1.** The spec still contains all three
   Priority 1 defects. The scorecard at
   `spec-scorecards/R4-1-live-audio-wiring-scorecard.md` has the findings; the first
   one is confirmed against source — `crates/spectre-audio/tests/rt_guard.rs:294–297`
   scans `src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, `src/null.rs`, not the
   `midi.rs` set the spec names, and the spec's own §4.1 table modifies `null.rs`.
2. **Fresh blind re-verification** by an agent that has not seen the remediation.
   Maximum 3 remediation rounds, then escalate to a gap report.
3. **Then batch 2** — R4-2, R4-4, R4-7 at concurrency 3, per `feature-tree.md`.
4. **Separately, D-R2 needs Jeff** before anything in
   `docs/02-reference-research/serum-2-observations.md` may be cited.

**Last completed action:** restarted the gauntlet from Phase 0. Fixed `.gitignore` so
`gauntlet-output/` is tracked while `gauntlet-active/` and `gauntlet-universal/` stay
ignored; deleted the corrupted `specs/track-management.md`; seeded `templates/` from
the model-agnostic GeistScope set with paths rewritten to `gauntlet-output/`; wrote
`criteria.md` (4 lenses, 6 auto-fails), `feature-tree.md` (9 R4 features), and
`manifest.md`.

**Last verified revision/worktree state:** branch `rename/geist-to-spectre` at
`dae16bb`, identical to `origin/rename/geist-to-spectre`. All `origin` refs match
local; nothing to pull, no conflicts. Changes are `.gitignore` (modified) and
`gauntlet-output/` (new).

**Commands run and outcomes:**
- `git ls-remote origin` — every ref matches local; the pull was a no-op.
- `git check-ignore -v gauntlet-output/` — exit 1, not ignored, as intended.
- `git check-ignore -v gauntlet-universal/GAUNTLET.md gauntlet-active/GAUNTLET.md` —
  both matched, rules on `.gitignore:20` and `.gitignore:19`.
- `diff -rq gauntlet-active gauntlet-universal` — no differences; byte-identical copies.
- `cargo fmt --all -- --check` — clean.
- `cargo clippy --locked --workspace --all-targets -- -D warnings` — clean, no warnings.
- `cargo test --locked --workspace` — all green, 0 failures. One test ignored:
  `hardware_lifecycle_drill`, the hardware-gated drill that is R4-3. This change
  touches no Rust, so the gate is a baseline, not evidence about the scaffolding.

**Files changed:** `.gitignore` (deduped, 6 malformed lines → 2 entries plus a section
comment). Added `gauntlet-output/{criteria,feature-tree,manifest,HANDOFF,decisions-needed,README}.md`
and `gauntlet-output/templates/{UNIVERSAL-GAUNTLET,CRITERIA-TEMPLATE,SPEC-TEMPLATE,SCORECARD-TEMPLATE}.md`.
Deleted `gauntlet-output/specs/track-management.md`. No crate source touched; nothing
under `docs/` modified.

**Artifacts produced:** the six control documents and four templates above.

**Known failures/blockers:** none technical. **R4-3 is hardware-blocked** — decision
23's Linux device drill needs a Linux host with a real ALSA device. Its spec is
authorable now; only execution is blocked.

**Decision required:** yes, and it blocks Phase 1.

> Jeff must sign off on `criteria.md` and `feature-tree.md`. Both are `proposed`.
> The open question in `criteria.md` is the lens weighting — currently Realtime &
> Correctness 35%, DAW Workflow Depth 25%, Product Identity & Scope Discipline 20%,
> Truthfulness & Evidence 20%. Nothing else in Phase 0 is open.

**Exact next action:** get Jeff's sign-off on both files, flip their status from
`proposed` to `accepted`, update their **Last verified** dates, then dispatch the
R4-1 spec agent — alone, ahead of the concurrency-3 batches, because live-audio wiring
is what makes every later slice observable.

**Exact next command:** none pending; the next step is a conversation with Jeff, not a
command. After sign-off, the gate to re-run before dispatch is:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

**Do not redo:**
- The pull. `origin` and local are identical; there is nothing to fetch or resolve.
- Recovering `specs/track-management.md`. It was corrupt *and* factually wrong; the
  manifest's run history is the record. Its subject is now R4-4, starting fresh.
- Discovering the feature tree from source. It is transcribed from the accepted R4
  slice queue in `docs/status/NEXT.md` on purpose — inventing it from source is what
  broke run 1.
- The benchmark interview. `docs/02-reference-research/` already holds accepted
  research on nine reference products; re-interviewing invites unsourced claims.
- Re-copying templates from `gauntlet-active/` or `gauntlet-universal/`. Those are the
  older Claude-Code-specific version. `templates/` is authoritative and tracked.
