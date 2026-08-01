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

This slice establishes the structural entity shell only: identity, owner, musical placement, and ordered membership. Kind-specific content is deliberately excluded until Slice B2 and no create/delete command may ship against a content-free project clip.

RED tests:

- Duplicate IDs are rejected atomically.
- Missing owner track is rejected atomically.
- Zero duration is rejected.
- Cross-track rehome changes one owner and preserves one identity.
- Removing an entity returns enough exact state for restoration.
- Restoring at an original order index reproduces the prior arrangement.

Stop before command implementation if canonical entity ownership requires persistence or engine changes.

## Slice B2 — Clip content ownership checkpoint

Status: paused and superseded as an implementation unit by `docs/product/ROADMAP.md` Milestones 1, 2, 3, and 5. Do not implement the steps below as one slice. The text is retained as pre-audit design history until replacement sub-specifications land.

Interview and specify before implementation:

- audio region ownership: shared immutable asset identity versus per-clip source range, gain, and future warp state;
- MIDI ownership: deep-copied pattern content versus explicit linked-pattern identity;
- automation ownership: target identity, curve coordinates, and behavior when the target disappears;
- duplication behavior for each clip kind;
- exact payload captured by create/delete undo;
- compatibility projection from legacy arena-backed `Clip` content.

Extend `ClipEntity` with the approved typed content payload or content reference before implementing create/delete commands.

Decisions:

- Audio clips share immutable media through stable `AssetId`; each clip independently owns sample-based source start/length and finite gain dB.
- The project registry owns asset path, exact hash, byte size, and verified/offline state.
- Fades, warp state, and automatic stretching remain deferred.
- MIDI clips directly own notes with stable `NoteId`, clip-relative `MusicalTime`, channel, pitch, normalized velocity, and nonzero duration.
- MIDI duplication deep-copies notes with newly allocated note IDs.
- Automation uses a typed track-parameter or graph-node-parameter target and clip-relative musical breakpoints.
- Automation values are normalized to `0.0..=1.0`; segment shape reuses `geist_automation::CurveShape`.
- Missing automation targets preserve an unresolved, saveable clip and curve.
- Target resolution is derived rather than persisted, and cross-track movement never retargets implicitly.
- Right resize masks rather than deletes out-of-bound MIDI notes and automation points.
- Offline audio and unresolved automation duplicate losslessly.
- Note IDs are arrangement-global; notes iterate by start then ID.
- Automation points are unique by tick and sorted; replacement updates value and outgoing shape.

Implementation slices:

1. Add shared nonzero `AssetId` and `NoteId`; extend checked allocation in the owning timeline/project domains.
2. Add validated typed content records and attach one payload to every `ClipEntity`.
3. Add deep-copy helpers that preserve audio/automation references and allocate fresh MIDI note IDs atomically.
4. Add legacy projection tests without changing current playback authority.

RED tests:

- Zero asset/note identities are rejected.
- Source and note lengths are nonzero; coordinate addition cannot overflow.
- Gain and normalized values reject NaN and infinities; normalized values reject out-of-range input.
- Duplicate MIDI content has equal note values and disjoint note IDs.
- Failed duplication leaves every allocator and collection coherent.
- Batch note-ID exhaustion rejects duplication without advancing allocation state.
- Offline/unresolved payloads round-trip exactly through structural remove/restore and duplication.
- Right-resize masking does not mutate stored notes or automation points.

Stop before implementation if stable asset allocation requires changing persistence or the current project registry. Persistence schema changes are not part of B2.

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
