<!--
Author: Jeff
Date: 2026-07-31
Description: Product contract for timeline clip selection synchronization.
Notes: Defines outcomes and invariants; implementation details live in PLAN.md.
-->

# Typed Timeline Selection and Inspector

## Status

Approved by default after the specification interview timed out. Scope is limited to timeline clips.

## Problem

The arrangement stores clip selection as a view-local vector index. The contextual Inspector reads `UIState::SelectedObject`, so selecting, creating, deleting, or deselecting a clip does not reliably update the Inspector. Vector indices are not durable identities, and newly drawn clips temporarily use ID `0` until the app assigns an engine ID.

## Goal

Make `UIState` the single UI selection authority for timeline clips while preserving the timeline model and engine as project/audio truth.

## User-visible behavior

- Selecting an existing timeline clip makes the Inspector show that clip by stable ID.
- Creating a clip selects it only after a nonzero stable ID is assigned.
- Re-selecting an already highlighted clip reclaims global selection from another pane.
- Dragging or resizing a clip retains its existing selection identity.
- Deleting or explicitly deselecting the selected clip clears a clip selection from the Inspector.
- A frame with no timeline selection change does not erase a track, node, parameter, or other typed selection.
- Context-shelf actions continue to come from the active workflow's `clip` configuration.

## Identity and ownership invariants

- `TimelineModel::selected` remains disposable view-local state.
- `UIState::SelectedObject::Clip` stores a stable clip ID, never a vector index or provisional ID `0`.
- Stable ID assignment occurs before selection synchronization.
- UI selection does not mutate project content or send audio-thread commands.
- No lock, allocation, filesystem access, or widget state enters the audio callback.

## Failure and edge cases

- An invalid selected index is treated as no selected clip.
- A provisional clip ID is not published to the Inspector.
- Deleting a clip clears global selection only when the global selection is a clip; unrelated typed selections are preserved.
- Loading a project with no selected clip clears a stale clip selection before the load frame completes.
- Reordering or deleting clips cannot retarget the Inspector by index accident.

## Scope

### Included

- Timeline clip selection synchronization at the app boundary.
- Stable-ID and deselection regression tests.
- Inspector behavior through the existing typed selection and workflow action APIs.

### Excluded

- Track, graph node, cable, parameter, and modulation-route synchronization.
- Multi-selection.
- Inspector property editing.
- Canonical timeline ownership, undo/redo, persistence changes, or audio-engine changes.

## Acceptance criteria

1. Existing, newly created, and recorded selected clips publish only a nonzero stable ID.
2. Selection survives move and resize without identity churn.
3. Explicit clip deselection or deletion removes stale clip Inspector state.
4. No-change frames preserve unrelated typed selection.
5. Unit tests cover select, repeated selection interaction, no-change preservation, deselect, invalid index, and provisional ID behavior.
6. View-level egui tests cover passive frames, repeated selected-clip clicks, and selected-clip deletion at the arrangement event boundary.
7. `cargo test -p geist-daw`, `cargo test -p geist-ui`, `cargo check --workspace`, and `git diff --check` pass.
