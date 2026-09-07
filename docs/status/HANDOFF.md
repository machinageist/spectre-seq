<!--
Author: Jeff
Date: 2026-09-07
Description: Session handoff — where the 2026-09-06/07 session left Spectre and what to do next
Notes: Written because the session ended on a usage limit rather than at a natural boundary
-->

# Handoff — 2026-09-07

- **Status:** accepted
- **Last verified:** 2026-09-07
- **Scope:** state at the end of the 2026-09-06/07 session, and how to resume
- **Decision authority:** Jeff
- **Upstream sources:** `STATUS.md`, `NEXT.md`, this session's 27 commits
- **Downstream dependents:** the next implementation session
- **Supersedes:** nothing
- **Superseded by:** none
- **Open decisions:** decision 8 (UI stack), decision 24 (note routing, proposed)
- **Known gaps:** no operator has heard any of this; see §3

## 1. Branch and gate

Branch `spectre/mvp-engine-foundations`, 27 commits ahead of `main`, working tree clean at
`3d342d4`. **Not merged.** The branch name is stale — it started as `m0/r4-operator-gate` and the
work outgrew both names.

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

**603 passed / 0 failed / 3 ignored.** The three ignored are hardware and operator drills and must
not be reclassified. Session start was 487.

## 2. What changed, in one paragraph

The session began from "the engine is excellent and the product cannot make music." It can now:
add a track, choose an instrument, chain effects, set a tempo, create a clip, write a chord, loop
bars, hear every edit without restarting the stream, save and recover, and undo all of it.
`NEXT.md` §"Where the product stands" is the authoritative list; `STATUS.md` carries the
per-change record. **Do not re-derive either from this file.**

## 3. The single most valuable next action

**Run `docs/05-quality/r4-qa-protocol.md`.** `./spectre` launches bare — `pipewire-alsa` is
installed and no `ALSA_CONFIG_PATH` override is needed. Thirteen manual rows have read `NOT RUN`
since 2026-08-28, and nothing in this session was confirmed by ear.

This is not box-ticking. The protocol's row 3 — "a slider drag reaches live audio" — describes a
defect this session found by *reading code*: on a Filament track the Lean slider was wired to the
level and the Level slider to nothing. Three of the session's defects were in rows the unrun
protocol already covers.

## 4. Three things that need a decision, not more code

1. **Decision 8 — UI stack.** Accepted only "for the R4 shell", reversibility rated Low once beta
   ships. Gates the piano-roll canvas, the session grid, and a drawn mixer. Jeff said "custom
   render everything" in the 2026-09-06 interview; the decision row has not been rewritten, and
   `docs/06-plans/` has no renderer milestone. **Nothing should be drawn on the current shell
   beyond what is already there without settling this.**
2. **Decision 24 — note routing.** `docs/03-architecture/note-routing.md`, status `proposed`, with
   four open sub-decisions in §6. MIDI effects are unrepresentable until it is ratified. Do not
   implement against it while it is proposed.
3. **Clip launching.** Genuinely unstarted. `PROD-001` is `proposed` and there is no scene or slot
   type anywhere in the workspace. This is a milestone's modelling, not a wiring slice.

## 5. Recorded and deliberately unfixed

- **No MIDI input backend.** `midir` is not a dependency; `MidiIngress` converts timestamped
  messages nothing produces. A MIDI keyboard cannot play Spectre, which the vision's audience
  statement assumes.
- **Meter would not persist if edited.** `spectre_app::project::project_envelope` never writes
  `model.meter_map`; it is read from the project's unknown map and written back only if it arrived
  there. Invisible today because everything agrees on 4/4. Silent data loss the moment meter
  becomes editable. Fix is a typed field plus a schema 4 migration.
- **R5 slice 8**, missing-media diagnostics, blocked on a media reference type that arrives with
  the drum sampler.
- **34 requirement-ledger rows are still `proposed`** — implemented in code, never ratified.

## 6. How to find work, if the above is blocked

Two audits produced most of this session's value and both are repeatable:

**Unreachable API.** Scan for `pub fn` with no call site outside its own definition. Six
musician-facing features were built, tested, and connected to nothing: the schedule lane, the clip
commands, voice routing, the loop region, instrument choice, and the effect chain's surface. Run
it against `crates/*/src/` only — a caller in `tests/` does not make a feature reachable.

**Miswired API — the harder class.** Code that runs successfully and does the wrong thing quietly.
No unreachability scan finds it. Four instances this session: the crossed Lean/Level wire, stored
device values that never applied on open, a schedule lane with no destination, and `has_clips`
following baked notes rather than placements. The way to find these is to ask "can a musician
actually do this end to end?" and follow the answer down to the metal.

**Falsify every central claim.** Three of this session's tests could not fail when first written,
and the mutation pass caught all three, not review.

## 7. Unresolved request, NOT Spectre

The session ended mid-way through a request that does not belong to this repository:

> "the ability to edit individual calendar event by double clicking on the event in the gui card
> and opening an editing card for the event… look for other features 'one layer down' like this
> one to add to the other modules in the suite as well."

Spectre has no calendar, events, cards, or module suite. A grep across Jeff's repositories for
`calendar` hit **SomaTrace, geistscope, mg-server, dotfiles, and mg-coreforge**, so the target was
not determined. **Ask Jeff which project before starting.** The phrasing — "gui card", "modules in
the suite" — most suggests the Quickshell desktop shell in `~/dotfiles` (which has bar modules and
panels, and a `hyprshell` skill) or the SomaTrace clinical app, but that is a guess and should not
be acted on as one.
