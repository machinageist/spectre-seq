<!--
Author: Jeff
Date: 2026-08-03
Description: Phased implementation plan for canonical clip content under ProjectDocument.
Notes: Written once all seven SPEC decisions were settled; execute and commit one validated slice at a time.
-->

# Canonical Clip Content Implementation Plan

## Preconditions

- Product and architecture contract: `SPEC.md`, accepted 2026-08-03.
- `docs/changes/project-document/SPEC.md` owns the transaction, history, identity, and load contracts these slices execute inside. Nothing here reopens them.
- Slice D2 of `docs/changes/project-document/PLAN.md` must land first. CC1 mutates through transactions, and transactions do not exist until D2.
- CC1 *is* slice D3 of that plan. It is written here because the content decisions live here; the two documents describe one slice from two sides.
- Every slice keeps the app runnable. The legacy `geist_timeline::Clip` path stays authoritative until CC3 completes.
- Migration is a strangler. Do not delete a legacy holder until its replacement round-trips.

## Slice CC1 — Clip records, placement split, and the window

Covers acceptance criteria for ownership, placement, extent, and clip kind.

Implements decisions 1, 1a, 4a, 6a, 6b, and 7.

Files:

- Add `crates/geist-document/src/clips.rs`.
- Update `crates/geist-document/src/arrangement.rs`.
- Update `crates/geist-document/src/lib.rs`.
- Update `docs/changes/project-document/SPEC.md` aggregate table.

Tasks:

1. [ ] Add the `clips` aggregate: clip records owning `ClipId`, `ClipKind`, name, colour, and a typed content payload. `ClipKind = Audio | Midi` — no automation variant.
2. [ ] Add the document accessor keyed by `ClipId`. No caller indexes the content map directly; this is what keeps a later `ContentId` layer contained to one map.
3. [ ] Re-scope `arrangement` to placement: owning `TrackId`, `start`, and `ClipWindow`. Remove every content field.
4. [ ] Add `ClipWindow { content_offset, extent, loop_region }` and `ClipExtent = Musical(MusicalTime) | Source(SourceFrames)`. `ClipEntity.duration: MusicalTime` becomes `ClipExtent` — a type change, not a rename.
5. [ ] Enforce the no-overlap invariant on an arrangement lane. Clip order is derived from `start`, so `rehome_clip` loses its explicit index parameter and `ClipLocation` and `RemovedClip` change shape.
6. [ ] Enforce referential consistency both ways at transaction commit: every record has exactly one placement, every placement references an existing record.
7. [ ] Prove cross-track move touches placement only, by asserting the content map is byte-identical across the move.
8. [ ] Prove cross-surface transfer is a copy: a `ClipId` never changes surface.
9. [ ] Resolve independent spec and code review findings.

Verification:

- `cargo test -p geist-document`
- `cargo test --workspace`
- `git diff --check`

Acceptance: a rejected transaction leaves both aggregates, history, revision, and every allocator unchanged.

## Slice CC2 — MIDI content

Implements decisions 5a, 5b, 5c, 6c, and the MIDI half of 3.

MIDI depends on nothing outside `geist-core`, and an empty MIDI clip is already a complete user object, which is why it goes first (decision 2).

Expected outcomes:

- `MidiContent` directly owning note entities, keyed by document-scoped `NoteId`.
- The note record: `NoteId`, clip-relative start, nonzero duration, `NoteKey` plus finite `PitchOffset`, channel, normalized velocity.
- `TuningId` declared with content deferred, resolving to 12-TET by default. Notes stay tuning-agnostic, so switching a tuning rewrites no note.
- The full note-expression dimension set — pitch/tuning, volume, pan, vibrato, expression, brightness, pressure — as a typed enum, never a free-form string.
- Expression curves on a note-relative time base, so expression travels intact when a note moves or is copied.
- Positional breakpoint keying: at most one point per dimension per tick; setting an existing position replaces its value and outgoing shape. No `BreakpointId` domain.
- Notes overrunning the window forced to note-off at the boundary, with durable duration unchanged.
- Deterministic iteration by clip-relative start, then `NoteId`.

Must prove: duplicating a MIDI clip deep-copies every note with fresh `NoteId` values and carries each note's expression curves with it. Fresh identities are a correctness requirement, not a policy choice — MPE routes per-note expression by identity, so two sounding notes sharing one identity make routing ambiguous.

Batch reservation of note IDs must fail atomically when the batch is not fully available, advancing no allocator.

## Slice CC2a — Assets aggregate

**Sequencing change from `project-document/PLAN.md`.** That plan orders D7 as tracks and devices, then assets, then conductor, then mappings — all after D3–D6. Decision 2 moves assets ahead of CC3.

The reason is invariant 11: audio clip content must not ship against a registry that cannot resolve it, because an always-unresolved audio clip is exactly the simulated behavior the vision forbids.

Scope is the assets aggregate only: `AssetId` allocation, project-relative path, content hash, byte size, and verified/offline state. Not the media pipeline, not Collect All and Save.

## Slice CC3 — Audio content

Implements decisions 4b, 4c, and the audio half of 3. Gated on CC2a.

Expected outcomes:

- `AudioContent` referencing an immutable `AssetId` with sample-domain source coordinates: source start in frames, nonzero source length in frames.
- Region-local edit state: gain, fades, reverse.
- One warp mapping mechanism — `WarpMode::Off | On { markers, algorithm }` — where a plain stretch is exactly two markers and source tempo is derived for display, never stored twice. The algorithm is a tag over an unchanged map, so spectral modes arrive at Milestone 9 without changing region identity.
- Fades in clip-relative `MusicalTime`, with a per-fade domain tag reserved but unused, so edit crossfades can switch to a source-frame domain later without a migration.
- Duplication sharing the immutable `AssetId` while deep-copying region state, warp markers, fades, and reverse. No media bytes are copied.
- Offline audio duplicating losslessly, preserving offline state rather than substituting empty content.

Must prove: a tempo-map edit mutates no audio clip record, because an unwarped clip's extent is `Source`.

## Slice CC4 — Clip-local automation envelopes

Deferred until durable target identity exists. Clip-local automation is an envelope owned by a clip record in clip-relative time — the vision's second parameter-control layer, distinct from arrangement automation lanes and from realtime modulation.

Positional breakpoint keying applies here identically to CC2, so the two share one point model.

## Slice CC5 — Delete legacy content holders

Delete only when the replacement round-trips through persistence:

- `geist_timeline::Clip` and `geist_timeline::Pattern` as content holders;
- `geist_project::schema::ClipKind::Audio { asset_index: usize }` and `ClipKind::Automation { lane_index: usize }` — vector positions used as durable references, which have to go regardless;
- `geist_project::schema::NoteEntry`, which carries no note identity at all.

## Standing verification for every slice

- `cargo test -p geist-document`
- `cargo test -p geist-core`
- `cargo test --workspace`
- `git diff --check`

## Stop conditions

Stop before the next slice when:

- D2 has not landed, so there is no transaction to mutate through;
- a slice would place clip content in more than one owner;
- an edit would delete, scale, or renumber content that resize or trim should mask;
- a note or region coordinate would depend on vector position, arena handle, or raw untyped integer;
- undo cannot restore exact clip and note identity without allocating;
- an audio region would carry project-sample or musical source coordinates instead of native frames;
- audio content would ship before an asset registry that can resolve it;
- independent review has unresolved findings.
