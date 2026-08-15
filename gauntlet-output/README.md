<!--
Author: Jeff
Date: 2026-08-14
Description: Entry point for any agent resuming the Spectre spec gauntlet
Notes: Read this first, then HANDOFF.md; the repository files are authoritative, not any vendor harness
-->

# Spectre Spec Gauntlet

- **Status:** accepted
- **Last verified:** 2026-08-14
- **Scope:** how to resume this pipeline without conversation history
- **Decision authority:** Jeff
- **Upstream sources:** `templates/UNIVERSAL-GAUNTLET.md`
- **Downstream dependents:** every agent that runs the gauntlet
- **Supersedes:** the untracked 2026-08-12 run
- **Superseded by:** none
- **Open decisions:** see `decisions-needed.md`
- **Known gaps:** none

## What this is

A multi-agent pipeline that writes a product spec for every feature in the active
milestone, has a **fresh, blind** agent grade each spec against `criteria.md`, and
loops failures through remediation up to three times before escalating.

An author never verifies its own artifact. Only a verifier advances a feature.

## Read this in order

1. **`HANDOFF.md`** — the exact next action. Start here on every resume.
2. **`manifest.md`** — per-feature state and the dated run history.
3. **`criteria.md`** — the binding grading standard.
4. **`feature-tree.md`** — what gets spec'd, in what order.
5. **`templates/UNIVERSAL-GAUNTLET.md`** — the full protocol.
6. **`decisions-needed.md`** — what is waiting on Jeff.

Then independently check repository state and re-run the last focused gate rather than
trusting the handoff's account of it.

## Authority

This directory is **subordinate** to `../docs/`. The conflict precedence in
`../docs/README.md` governs, and specs never silently redefine accepted requirements.
Where the gauntlet wants to change accepted direction, it says so in
`decisions-needed.md`. Auto-fail rule AF-1 enforces this.

Read `../CLAUDE.md` before doing repository work: it sets the load order, the
non-negotiables, and Jeff's header-block requirement, which applies to every file here.

## Layout

```text
gauntlet-output/          # tracked — durable run state, survives a fresh clone
├── README.md             # this file
├── HANDOFF.md            # resume contract, updated before any agent stops
├── criteria.md           # grading standard
├── feature-tree.md       # confirmed features
├── manifest.md           # per-feature state + run history
├── decisions-needed.md   # escalations awaiting Jeff
├── summary.md            # final report (written at Phase 4)
├── templates/            # authoritative templates
├── specs/                # one per feature
├── spec-scorecards/      # one per blind review
└── gap-reports/          # 3x failures only
```

`gauntlet-active/` and `gauntlet-universal/` at the repository root are **gitignored
tooling copies of an older, vendor-specific version**. They are not authoritative and
are byte-identical to each other. Use `templates/`.

## Running it

Tell the agent to read this file, then follow `HANDOFF.md`, `manifest.md`,
`criteria.md`, and `templates/UNIVERSAL-GAUNTLET.md`. The invocation mechanism is
agent-specific; the repository files are what bind.

The gauntlet stops for Jeff at exactly three points: criteria sign-off, feature-tree
sign-off, and any spec that fails three times.

## Repository gate

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```
