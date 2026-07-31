<!--
Author: Jeff
Date: 2026-07-31
Description: Implementation plan for timeline clip selection synchronization.
Notes: Each phase must leave the repository buildable and independently reviewable.
-->

# Typed Timeline Selection and Inspector Plan

## Preconditions

- Product contract: `SPEC.md`.
- `UIState` remains disposable UI state.
- `StudioApp` remains the boundary that assigns stable timeline IDs and mirrors edits to the engine.

## Phase 1 — Selection transition helper

- Add a small app-layer helper that accepts the current stable selected clip ID, the previously mirrored timeline selection, and whether the timeline explicitly interacted with selection this frame.
- Publish `SelectedObject::Clip(id.to_string())` only for nonzero IDs.
- On a transition to no clip, clear selection only if the global selection is currently a clip.
- Do nothing when the timeline selection has not changed and no explicit selection interaction occurred.
- Re-publish an unchanged local clip ID after an explicit interaction so it can reclaim global selection from another pane.
- Add focused tests for selection, repeated interaction, unrelated-selection preservation, deselection, and provisional ID suppression.

Verification:

- Run the focused `studio` selection tests.
- Run `git diff --check`.

## Phase 2 — App lifecycle wiring

- Add a `timeline_selection_mirror` field initialized from the initial session.
- Propagate explicit selection interactions from the arrangement view through `StudioResponse`.
- Derive the selected stable clip ID after `sync_timeline` assigns IDs.
- Invoke the helper before clip-note synchronization so downstream UI state and selected clip data agree.
- Force selection reconciliation after a successful project load so the old project's clip identity cannot survive the load frame.
- Ensure deletion transitions cannot retain stale clip Inspector state.

Verification:

- Run `cargo test -p geist-daw`.
- Run `cargo test -p geist-ui`.
- Run `cargo check --workspace`.
- Run `git diff --check`.

## Commit plan

One atomic implementation commit after the already-pushed foundation commits:

`feat(ui): synchronize timeline clip selection with inspector`

The commit includes `SPEC.md`, `PLAN.md`, implementation, and tests. It excludes canonical timeline and other object-selection work.

## Stop conditions

Stop and return to design review if implementation requires publishing provisional IDs, changing project schema, changing engine commands, adding cross-thread synchronization, or broadening selection semantics beyond one timeline clip.
