<!--
Author: Jeff
Date: 2026-07-31
Description: Product and architecture contract for canonical clip entities and reversible edits.
Notes: Stable identity precedes command migration; UI and persistence migration remain separate slices.
-->

# Canonical Clip Entities and Reversible Commands

## Status

Slice A (stable track/clip identity) and Slice B (structural arrangement entities) are complete.

Slice B2 and subsequent commands are paused. The accepted launcher, warp, MPE, tuning, hybrid-track, and durable-target requirements expose missing prerequisites in this earlier arrangement-only content model. `docs/product/PRODUCT_VISION.md` and `docs/product/ROADMAP.md` govern its replacement with dependency-safe sub-specifications.

`docs/changes/project-document/SPEC.md` resolves the ownership question that paused this work: `ProjectDocument` owns durable truth and the canonical arrangement aggregate is one of its aggregates. Slice C reappears there as slice D3, behind a blocking checkpoint on clip-content ownership. Do not resume this document's slices directly.

The content sections below preserve the pre-audit proposal for review history. They are not approved implementation instructions.

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

## Canonical content ownership

`ClipEntity` owns one typed content payload. Content is independent per visible clip except that audio clips reference shared immutable project assets.

### Audio regions

An audio payload owns:

- stable nonzero `AssetId`;
- source start in samples;
- nonzero source length in samples;
- finite per-region gain in dB.

The project asset registry maps `AssetId` to project-relative path, exact content hash, byte size, and verified/offline state. Path, vector position, and filename are not asset identity. Duplicating an audio clip preserves `AssetId` and deep-copies its independent region state without copying media bytes.

Fades, warp markers, and time-stretch modes are deferred. If visible musical duration exceeds available source playback, no implicit stretching occurs.

### MIDI regions

A MIDI payload directly owns its note entities. Each note owns:

- stable nonzero `NoteId`;
- clip-relative start in `MusicalTime`;
- nonzero duration in `MusicalTime`;
- MIDI channel and pitch;
- normalized velocity.

Ordinary clip duplication deep-copies every note while allocating fresh `NoteId` values. Notes are not linked between duplicate clips. Right-edge clip resize is non-destructive: notes outside the visible clip duration remain stored and are masked from rendering and playback.

`NoteId` is unique across the arrangement, not merely within one MIDI clip. MIDI notes render and iterate deterministically by clip-relative start and then `NoteId`. Duplicate note IDs are rejected atomically.

### Automation regions

An automation payload owns one typed target and its clip-relative curve. Initial target variants are:

- track parameter: `TrackId` plus `ParamId`;
- graph or plugin parameter: `NodeId` plus `ParamId`.

Breakpoint positions use clip-relative `MusicalTime`. Values are normalized to the inclusive `0.0..=1.0` range; parameter metadata maps normalized values to native and display units. Segment shape reuses `spectre_automation::CurveShape`; the canonical timeline does not define another equivalent curve enum.

Points remain sorted by position with at most one point at each tick. Setting an existing position replaces its normalized value and outgoing segment shape. The last point's outgoing shape is retained for round-trip fidelity but has no evaluation effect until a later point exists.

When a target disappears, the automation clip remains visible and saveable with its target identity and complete curve. It is unresolved and cannot drive the engine until explicitly relinked. Right-edge resize preserves and masks out-of-bound points rather than deleting or scaling them.

Resolved versus unresolved is derived from the current target registry and is not persisted as a second source of truth. Moving an automation clip across tracks preserves its exact target; retargeting is a separate explicit command.

### Offline and unresolved duplication

Offline audio and unresolved automation clips may be duplicated. Duplication preserves the complete payload and offline/unresolved state; it never substitutes empty content or requires temporary resolution.

### Identity allocation

`AssetId` and `NoteId` join the shared opaque ID vocabulary in `spectre-core`. Both reject zero. Note allocation is monotonic, checked, and timeline-owned. Asset allocation is monotonic, checked, and project-registry-owned. Loading observes stored IDs so future allocation cannot collide. Exhausting one identity domain does not exhaust another.

MIDI duplication reserves all required note IDs as one checked batch. If the complete batch is unavailable, duplication fails before advancing the note allocator or mutating arrangement content. Successful reservation may produce monotonic gaps only if a later fully validated operation is abandoned; IDs are never reused.

## Stable identity

- `TrackId` and `ClipId` are opaque nonzero `u64` newtypes.
- Their single canonical definitions live in `spectre-core`; timeline code re-exports rather than duplicates them.
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
- Right-edge resize never destructively edits MIDI notes or automation points outside the new visible boundary.

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

### Slice B2 — Canonical clip content

- Add stable `AssetId` and `NoteId` identities with domain-owned allocation.
- Add typed audio, MIDI, and automation payloads to `ClipEntity`.
- Enforce nonzero source/note lengths, finite gain, normalized automation/velocity values, and checked coordinates.
- Preserve content losslessly when clips are removed, restored, duplicated offline, or temporarily masked by resize.
- Do not migrate persistence, UI, engine snapshots, or legacy playback authority.

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
