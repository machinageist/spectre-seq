<!--
Author: Jeff
Date: 2026-07-31
Description: Product and architecture contract for canonical clip entities and reversible edits.
Notes: Stable identity precedes command migration; UI and persistence migration remain separate slices.
-->

# Canonical Clip Entities and Reversible Commands

## Status

Approved to proceed using the recommended independent-region model after the ownership checkpoint timed out without a response.

This decision is intentionally revisitable before persistence migration.

## Problem

The current timeline stores clip content in an arena and permits the same arena handle to appear in multiple track placements. A placement is identified by `(arena handle, sample start)`. This is insufficient because:

- two placements can share the same handle and start;
- arena handles are runtime-generational, not persistent identity;
- removal and movement silently select the first matching tuple;
- commands silently succeed when no mutation occurred;
- cross-track moves are not atomic;
- visible duration is inferred from content rather than represented by the arrangement entity;
- undo cannot prove that exact identity, order, and state were restored.

## Ownership decision

One canonical clip entity represents one visible arrangement region.

A clip entity owns:

- stable `ClipId`;
- owning `TrackId`;
- musical start in `MusicalTime`;
- visible musical duration in `MusicalTime`;
- kind-specific region data or a reference to independently owned content.

The first structural entity slice may establish only identity, owner, placement, duration, and ordered membership. Such a content-free shell is not yet a complete project clip and cannot enter create/delete command history until the kind-specific ownership checkpoint is resolved.

Consequences:

- One `ClipId` appears on exactly one track at exactly one position.
- Moving across tracks preserves `ClipId`.
- Ordinary duplication creates a new `ClipId` and independent editable region state.
- Audio duplicates may share the immutable source asset reference while retaining independent trim/gain/region state.
- MIDI duplicates deep-copy pattern content initially.
- Linked clip instances are deferred until their propagation, unlinking, persistence, and undo semantics are explicitly designed.

## Stable identity

- `TrackId` and `ClipId` are opaque nonzero `u64` newtypes.
- Their single canonical definitions live in `geist-core`; timeline code re-exports rather than duplicates them.
- Raw value `0` is invalid and remains available for provisional UI state only.
- Runtime arena `Index` values never cross the canonical public edit boundary.
- IDs allocate monotonically and are not reused after deletion.
- Allocation failure at exhaustion is explicit.
- Undo restores the original ID; it never allocates a replacement.
- Loading explicit IDs advances the allocator past every observed ID.
- Duplicate or zero IDs are rejected during canonical insertion and future persistence loading.

## Arrangement structure

The target canonical shape is:

- `Timeline` owns tracks and clip entities.
- Each track owns an ordered `Vec<ClipId>`.
- Timeline-owned lookup resolves `TrackId` and `ClipId` to canonical entities.
- Track order and clip order are part of edit state and must round-trip through undo/redo.
- Clip content storage may remain arena-backed internally while arena handles remain private implementation details.

## Duration and trimming

- Visible duration belongs to the clip entity, not immutable source content.
- Duration must be at least one tick.
- Initial resize support changes the right edge only: start and source offset remain unchanged.
- Left-edge trim is excluded from the first command slice because it must define source-offset conversion separately for native-rate audio and MIDI content.
- Audio source start and source length remain sample-domain region data.
- No resize or trim implies time-stretch.

## Command contract

Commands execute only on the app thread and return a typed result.

Successful command execution reports a mutation and enters undo history. Rejected commands do not enter history and do not clear redo history.

Initial commands:

- create clip;
- delete clip;
- move clip on one track;
- move clip across tracks atomically;
- resize the right edge.

Every command must:

- target stable IDs;
- validate all preconditions before mutation;
- either complete fully or leave the timeline unchanged;
- capture exact prior track, ordered position, start, duration, and region state needed by undo;
- restore the same IDs and ordering on undo;
- reapply the same accepted mutation on redo;
- report stale IDs, missing tracks, duplicate IDs, invalid duration, and exhausted identity distinctly enough for tests and UI messaging.

## Undo/redo semantics

- A failed initial apply is not recorded.
- A successful new command clears redo history.
- Undo failure does not move the command to redo history.
- Redo failure does not move the command to done history.
- Cross-track move undo restores both original track and original ordered position.
- Delete undo restores the entire entity and original ordered position.
- Create undo removes the exact created entity; redo restores its original ID rather than allocating again.
- History is session state and is not persisted in this phase.

## Gesture boundary

The intended UI policy is one history command per completed gesture:

- drag preview is disposable UI state;
- pointer release submits one validated command;
- Escape cancels preview without canonical mutation;
- incremental pointer motion does not flood command or engine queues.

This policy will receive a separate UI checkpoint before command routing is implemented.

## Realtime boundary

- Timeline mutation and undo/redo remain off the audio callback.
- Commands contain no callback-owned references.
- Successful canonical mutations later publish bounded engine commands or immutable snapshots.
- No command execution path may allocate, lock, log, perform I/O, or traverse mutable timeline state on the callback.

## Slice boundaries

### Slice A — Stable identity foundation

- Add `TrackId` and `ClipId`.
- Add checked monotonic allocators and observed-ID advancement.
- Add tests for zero rejection, monotonicity, no reuse, exhaustion, and load advancement.
- Do not migrate Timeline, UI, persistence, or engine behavior.

### Slice B — Canonical clip entity

- Introduce one-entity-per-region representation.
- Add exact stable-ID lookup and ordered track membership.
- Retain compatibility projection or adapters for existing sample-based emission until playback migration.

### Slice C — Typed reversible commands

- Replace silent command mutation with typed results.
- Add create/delete/move/cross-track/right-resize commands.
- Prove atomic rejection and exact undo/redo restoration.

## Acceptance criteria

1. Raw ID `0` cannot construct a canonical ID.
2. Valid raw IDs round-trip exactly.
3. Allocation starts at `1`, advances monotonically, and never wraps.
4. Observing loaded IDs advances future allocation without collisions.
5. Clip entities are uniquely addressable independent of vector order or arena handles.
6. Duplicate clip placement identity is structurally impossible.
7. Command rejection leaves timeline and history unchanged.
8. Undo/redo restores exact IDs, track membership, order, start, duration, and region state.
9. Cross-track moves are atomic.
10. No persistence schema or realtime callback behavior changes before its dedicated slice.
