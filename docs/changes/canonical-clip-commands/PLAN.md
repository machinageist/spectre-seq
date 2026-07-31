<!--
Author: Jeff
Date: 2026-07-31
Description: Dependency-safe implementation plan for canonical clip entities and reversible commands.
Notes: Each slice ends with review, validation, commit, push, and remote SHA verification.
-->

# Canonical Clip Commands Implementation Plan

## Preconditions

- Canonical time foundation is committed at `a85f67a`.
- Product contract: `SPEC.md`.
- One independent clip entity represents one visible arrangement region.
- Stable identity must precede exact commands.
- Existing app/UI/project/engine behavior remains unchanged until dedicated migrations.

## Slice A — Stable identity foundation

Files:

- Specialize shared `TrackId` and `ClipId` in `crates/geist-core/src/ids.rs`.
- Add `crates/geist-timeline/src/identity.rs`.
- Update `crates/geist-timeline/src/lib.rs`.

RED tests:

1. Raw zero fails for `TrackId` and `ClipId`.
2. Valid raw values round-trip.
3. First allocated ID is `1`.
4. Successive IDs are monotonic and distinct.
5. Observing loaded ID `N` makes the next allocation greater than `N`.
6. Observing an older ID never moves allocation backward.
7. Exhaustion returns an explicit failure and never wraps to `0`.
8. Deleted IDs are not reused by allocator behavior.

Implementation:

- Keep one opaque nonzero definition for each ID in `geist-core`.
- Re-export those shared types from the timeline identity boundary.
- Keep allocation app-thread-owned and non-atomic.
- Keep generic allocation machinery private unless public need is proven.
- Expose only explicit constructors/accessors and checked allocation behavior.

Verification:

- `cargo fmt -p geist-timeline -- --check`
- `cargo test -p geist-timeline`
- `cargo clippy -p geist-timeline --all-targets -- -D warnings`
- `cargo check --workspace`
- `cargo test --workspace`
- `git diff --check`
- independent review with all findings resolved

## Slice B — Canonical clip entity and ordered membership

Design:

- Add stable-ID canonical track records.
- Add clip entities with ID, owner, start, and nonzero duration.
- Make each clip belong to exactly one track.
- Resolve entities by stable ID through Timeline-owned lookup.
- Preserve ordered track membership for deterministic rendering and undo.
- Keep existing sample-based playback structures behind a compatibility boundary.

RED tests:

- Duplicate IDs are rejected atomically.
- Missing owner track is rejected atomically.
- Zero duration is rejected.
- Cross-track rehome changes one owner and preserves one identity.
- Removing an entity returns enough exact state for restoration.
- Restoring at an original order index reproduces the prior arrangement.

Stop before command implementation if canonical entity ownership requires persistence or engine changes.

## Slice C — Typed edit result and history semantics

- Change command application and reversal to typed results.
- Record only successful mutations.
- Preserve redo after rejected initial execution.
- Keep commands on their current stack when undo or redo rejects.
- Add stale-ID and invalid-track rejection tests.

This slice may temporarily adapt existing commands; do not mix in new clip operations until history semantics are proven.

## Slice D — Create and delete

- Create reserves one ID once and reuses it on redo.
- Delete captures full entity and original ordered position.
- Undo restores exact identity and order.
- Rejection leaves allocator, timeline, and history coherent.

## Slice E — Move and cross-track move

- Same-track move changes start only.
- Cross-track move validates source and destination before mutation.
- Cross-track undo restores source track, order, and start.
- No remove-then-fail intermediate state is observable.

## Slice F — Right-edge resize

- Duration is a positive `MusicalTime` value.
- Right-edge resize preserves start and kind-specific source offset.
- Undo restores exact duration.
- Left trim and time-stretch remain excluded.

## Slice G — App/UI projection checkpoint

Interview before implementation:

- preview rendering ownership;
- snap policy and modifier behavior;
- cancel behavior;
- multi-selection command grouping;
- engine publication timing;
- error presentation and recovery.

## Completion rule

A slice is complete only when:

- tests were observed failing for the missing behavior;
- implementation is minimal and scope-contained;
- focused and workspace gates pass;
- independent review findings are resolved;
- the slice is committed separately;
- push succeeds and local/remote SHAs match;
- the working tree is clean.
