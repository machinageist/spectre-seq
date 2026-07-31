<!--
Author: Jeff
Date: 2026-07-31
Description: Phased implementation plan for canonical timeline ownership.
Notes: Execute and commit one independently validated slice at a time.
-->

# Canonical Timeline Implementation Plan

## Preconditions

- Product contract: `SPEC.md`.
- Canonical arrangement time: 960-PPQ integer ticks.
- Audio source ranges remain sample-based.
- No implicit audio time-stretch.
- App-thread ownership and realtime constraints remain mandatory.

## Slice 1 — Musical-time foundation

Files:

- Add `crates/geist-timeline/src/time.rs`.
- Update `crates/geist-timeline/src/lib.rs`.
- Update `crates/geist-timeline/src/tempo.rs`.
- Update this plan after verification.

Tasks:

1. [x] Write failing tests for raw ticks, beat quantization, invalid beats, and checked arithmetic.
2. [x] Implement opaque `MusicalTime(u64)` and public `TICKS_PER_QUARTER = 960`.
3. [x] Write failing TempoMap tests for constant tempo, tempo changes, and sample round trips.
4. [x] Add explicit tick/sample adapters while retaining existing beat/sample APIs.
5. [x] Verify no app, UI, project, engine, or transport behavior changed for valid inputs.
6. [x] Resolve independent spec and code review findings.
7. [x] Commit and push only after all findings are resolved.

Verification:

- `cargo test -p geist-timeline`
- `cargo check --workspace`
- `cargo test --workspace`
- `git diff --check`

## Slice 2 — Canonical placement and command vocabulary

Design checkpoint required before implementation.

Questions to resolve:

- Whether clip length belongs to clip content, placement, or a separate region object.
- Whether one clip content object may have multiple placements.
- Exact semantics for cross-track move and undo identity.
- Whether drag gestures coalesce into one command or publish incremental preview commands.
- Failure behavior for stale IDs and invalid destination tracks.

Expected outcomes:

- Tick-based placement and length.
- Reversible create, delete, move, resize, and cross-track commands.
- Command results that report rejection instead of silent no-ops.
- Tests for exact undo/redo restoration and redo invalidation.

## Slice 3 — Stable canonical identity

- Add `TrackId` and `ClipId` newtypes.
- Separate persistent IDs from generational arena handles.
- Define allocation, load restoration, deletion, and collision behavior.
- Keep Inspector selection stable across projection rebuilds.

## Slice 4 — UI projection and typed edit intents

- Build renderer-facing timeline projections from canonical state.
- Replace direct durable UI mutation with typed arrangement intents.
- Preserve provisional drag feedback without creating a second source of truth.
- Add interaction-to-command integration tests.

## Slice 5 — Engine publication

- Derive bounded engine mutations or immutable arrangement snapshots from committed commands.
- Preserve allocation-free, lock-free callback behavior.
- Verify MIDI and audio move/resize/cross-track parity.

## Slice 6 — Persistence migration

- Add a versioned project schema using stable IDs and ticks.
- Convert legacy floating beats deterministically.
- Preserve offline audio references and exact relinking identity.
- Add old-to-new migration fixtures and round-trip tests.

## Slice 7 — Remove transitional mirrors

- Delete app-owned arrangement truth after parity tests pass.
- Remove diff-based synchronization paths made obsolete by commands.
- Update architecture and product documentation to implementation truth.

## Stop conditions

Stop before the next slice when:

- A product or architecture question above is unresolved.
- A migration would alter persistence without fixtures.
- Engine publication would allocate or lock in the callback.
- Undo cannot restore exact identity and state.
- Validation or independent review has unresolved findings.
