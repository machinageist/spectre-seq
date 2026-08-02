<!--
Author: Jeff
Date: 2026-08-01
Description: Draft ownership contract for canonical clip content under ProjectDocument.
Notes: DRAFT FOR REVIEW; the Decisions required section holds product questions the repo owner must settle.
-->

# Canonical Clip Content Ownership

## Status

**Draft for review. Not accepted. Not implementation instructions.**

This sub-specification un-gates slice D3 of [`docs/changes/project-document/PLAN.md`](../project-document/PLAN.md), which is blocked on the clip-content ownership question.

[`docs/changes/project-document/SPEC.md`](../project-document/SPEC.md) is accepted and fixes the **owner**: `ProjectDocument` and its arrangement aggregate. It deliberately leaves the **payload** undecided. This document proposes the payload.

[`docs/changes/canonical-clip-commands/SPEC.md`](../canonical-clip-commands/SPEC.md) Slice B2 is paused prior thinking, not a baseline. This draft confirms parts of it, revises parts of it, and rejects one part of it outright.

Nothing here is settled until `## Decisions required` is answered. Seven decisions change the shape of the first slice, so no `PLAN.md` accompanies this draft. See `## Planning status`.

References use post-D1 homes: `geist_core::time::MusicalTime`, `geist_core::ids`, `geist_document::arrangement`.

## Problem

`geist_document::arrangement::ClipEntity` is a content-free shell: `ClipId`, owning `TrackId`, `MusicalTime` start, `MusicalTime` duration. It cannot represent a project clip, so D3 cannot prove that undo restores clip state, and D6 cannot round-trip a clip.

Five accepted product requirements broke the earlier arrangement-only content proposal:

1. **Launcher.** A slot at each track/scene intersection owns a clip that has no arrangement start and no lane order. Content attached to arrangement placement cannot be reached by the launcher without a second, parallel clip type.
2. **Arrangement capture.** Capture must create independent clip entities with no silent propagation back to the source. That requires a defined copy boundary, not an implicit one.
3. **Warp.** Non-destructive warp markers and tempo interpretation are core audio architecture. Audio region state that omits them bakes in wrong source-coordinate semantics.
4. **MPE and tuning.** Per-note pitch, pressure, timbre, and non-12-TET tuning must be reachable from the first note model, not bolted onto `key: u8`.
5. **Three parameter-control layers.** Arrangement automation in project time, clip-local automation in clip-relative time, and realtime modulation do not share one data model. A single `ClipKind::Automation` collapses the first two.

The legacy holders confirm the gap. `crates/geist-timeline/src/clip.rs` models audio as a raw `source: u64` with no warp state, MIDI as an arena `Index` into a sample-positioned `Pattern` with no note identity, and automation as a third clip kind with untyped breakpoints. `crates/geist-project/src/schema.rs` persists `ClipKind::Audio { asset_index: usize }` and `ClipKind::Automation { lane_index: usize }` — vector positions used as durable references — and `NoteEntry` with no note identity at all.

## Ownership decision

**Proposed.** Placement and content are separate durable facts with separate owners.

- A **clip record** owns identity, kind, name, colour, and the kind-specific content payload. It knows nothing about where it sits.
- A **placement** owns where one clip sits on one production surface, and the window through which that surface views the clip's content.

One `ClipId` names one clip record and exactly one placement. A clip is never in two places and never has two contents.

### Aggregate table

This amends the aggregate table in the accepted `project-document/SPEC.md`:

| Aggregate | Owns | Change |
| --- | --- | --- |
| `clips` | Clip records: identity, kind, name, colour, content payload | **new sibling aggregate** |
| `arrangement` | Arrangement placements: owning `TrackId`, start, window; lane membership | re-scoped to placement only |
| `launcher` | Launcher placements: track/scene slot occupancy, window, launch settings | unchanged; consumes the same clip records |

`geist_document::clips` is the only owner of clip content. `geist_document::arrangement` holds no payload. Neither `geist_timeline::Clip`, `geist_timeline::Pattern`, nor `geist_project::schema::ClipKind` may hold clip content once its slice completes.

### Why the split

- The launcher gets clips without a parallel clip type. Editors, persistence, and duplication address content by `ClipId` regardless of surface.
- Cross-track move touches placement only. A move can no longer lose or reorder notes, because it never reads content.
- Arrangement capture is a content deep copy plus a new placement. Independence is structural rather than a rule someone must remember.
- Linked clip instances stay reachable. Content is already a separately keyed aggregate, so introducing a `ContentId` layer later re-keys one map instead of redesigning entities, editors, and commands. See Decision 1.

### Clip kinds

```
ClipKind = Audio | Midi
```

**Automation is not a clip kind.** Arrangement automation is lane-based in project musical time and belongs to the `automation` aggregate. Clip-local automation is an envelope owned by a clip record in clip-relative time. See Decision 7.

## Content record contract

A clip record owns:

- `ClipId`, allocated by the document, nonzero, never reused;
- `ClipKind`, fixed at creation; a clip never changes kind;
- user-visible name and colour;
- exactly one typed content payload matching its kind;
- zero or more clip-local automation envelopes (deferred to slice CC4).

Rules:

- Content is addressed only through a document accessor keyed by `ClipId`. No caller indexes the content map directly. This keeps a later shared-content model contained.
- A clip record with no notes, or with an unresolved asset, is complete and valid. Empty is not incomplete.
- Every clip record has exactly one placement; every placement references an existing clip record. Both directions are validated at transaction commit and at load.
- Deleting a placement and its record is one transaction. Orphan records are invalid, not tolerated.

## Windowing and extent contract

Both surfaces view content through one shared value type. Defining it once is what keeps the launcher from inventing a second windowing model.

```
ClipWindow = {
    content_offset: MusicalTime,    // clip-relative origin of the visible window
    extent: ClipExtent,             // visible length
    loop_region: Option<LoopRegion> // deferred; shape reserved
}

ClipExtent = Musical(MusicalTime) | Source(SourceFrames)
```

Rules:

- `content_offset` exists from the first slice and defaults to zero. Left-edge trim later moves the window, never the content. Notes and points never need negative coordinates.
- `extent` is at least one tick, or at least one frame, and is never zero.
- Right-edge resize changes `extent` only. It never deletes, scales, shortens, or renumbers content. Content outside the window is retained and masked.
- Left-edge trim changes `content_offset` and `extent` together. It never moves content.
- `ClipExtent::Source` exists so unwarped audio keeps a sample-domain durable length under invariant 8. Tempo-map edits then never mutate clip records. See Decision 4a.
- The arrangement placement adds `start: MusicalTime` and lane membership. The launcher placement adds slot occupancy and launch settings, specified in the launcher sub-spec.

## MIDI content contract

`MidiContent` directly owns structured note entities. It is not a reference to a shared pattern.

### Note identity

- `NoteId` is a document-scoped opaque nonzero newtype from `geist_core::ids`, allocated by the document allocator.
- Uniqueness is document-wide, not clip-wide or arrangement-wide. Launcher clips share the domain.
- Identity survives exact editing, undo, redo, expression editing, duplication of the *same* note, and persistence — invariant 9.
- Identity is never derived from MIDI channel, list position, or start time.

### Note record

- `NoteId`;
- start in clip-relative `MusicalTime`;
- duration in `MusicalTime`, at least one tick;
- `NoteKey`, a validated newtype over `0..=127`, naming the scale or keyboard position;
- `PitchOffset`, finite semitones, default zero, for static per-note detune;
- `MidiChannel`, a validated newtype over `0..=15`, carried for MIDI 1.0 interoperability only;
- normalized velocity, and normalized release velocity;
- `muted`, a non-destructive deactivation flag;
- an expression set, possibly empty.

### Tuning

- `NoteKey` is a scale position, not a frequency. Tuning maps key to pitch at render time.
- Tuning systems live at project and device scope under a declared `TuningId` domain, never on the note. Notes stay tuning-agnostic, so switching a project tuning rewrites nothing.
- Twelve-tone equal temperament is the default resolution of `TuningId`, not a schema assumption.
- An unresolved `TuningId` renders as 12-TET with a visible indicator and never rewrites durable state.
- Scala and MTS import become tuning-table content under a later tuning sub-spec. This spec only guarantees the note model does not block them.

### MPE and per-note expression

- Per-note expression is routed by `NoteId`. Channel is interop metadata and is never used to reconstruct identity. MPE import that re-derives identity from channel violates invariant 9.
- An expression dimension owns a breakpoint curve. Each breakpoint carries a note-relative position, a value, and a `CurveShape`.
- Dimensions are typed, not free-form strings, so plugin note-expression interoperability stays lossless.
- The expression set travels with its note through copy, undo, and persistence.
- The complete expression editor follows the data model; the model does not wait for the editor.

### Ordering and masking

- Notes iterate deterministically by clip-relative start, then `NoteId`.
- Duplicate `NoteId` insertion is rejected atomically.
- A note whose start lies outside the visible window is masked from rendering and playback and retained durably.
- A note that starts inside the window and ends outside it is masked at the window boundary for playback. Its durable duration is unchanged. See Decision 6c.

## Audio content contract

`AudioContent` references an immutable managed asset and owns all region-local edit state.

### Asset reference and source coordinates

- `AssetId` is a document-scoped opaque nonzero identity. The assets aggregate maps it to project-relative path, exact content hash, byte size, native sample rate, channel count, frame count, and verified/offline state.
- Path, filename, and vector position are never asset identity. This replaces the persisted `asset_index: usize`.
- Source coordinates are typed `SourceFrame` and `SourceFrames` in the asset's **native** sample rate. They are never project samples, never `MusicalTime`, and never implicitly rate-converted — invariant 8.
- A source span is validated against the asset frame count only while the asset is resolved. An unresolved asset's span is preserved verbatim, never clamped, zeroed, or defaulted.
- If a relinked asset is shorter than the stored span, the out-of-range remainder is masked with a diagnostic. It is never clamped or rewritten.

### Region-local edit state

Owned per clip record, never shared:

- source span: start frame and frame length;
- gain in dB, finite;
- reverse flag;
- fade in and fade out;
- warp state.

Consolidate, bounce, resample, freeze, and render create **new** managed assets. They never rewrite source media.

### Warp and tempo interpretation

Warp is a durable coordinate map plus an algorithm tag. Representation lands with the audio region; rendering may lag.

```
WarpMode = Off | On { markers, algorithm }
```

- `Off`: the region plays at native rate. Project tempo does not transform its coordinates. Its durable extent is `ClipExtent::Source`.
- `On`: markers define a piecewise map between source frames and clip-relative musical time. Markers are strictly monotonic in both axes, so the map is invertible and has no zero-length or inverted spans. Durable extent is `ClipExtent::Musical`.
- A plain "stretch this loop to N bars" is exactly two markers. There is one mapping mechanism, not a warp flag plus a clip tempo plus a marker list. See Decision 4b.
- The algorithm tag selects repitch, beat-preserving, or a later spectral mode. Adding an algorithm adds an enum variant; it changes neither the map nor region identity, which is what the vision requires of later spectral modes.
- Warp markers are region-local state. They are deep-copied on duplication and never shared between clips.
- Toggling `WarpMode` converts the durable extent through the conductor and is one explicit transaction, not an implicit reinterpretation.

## Clip-local automation contract

Deferred to slice CC4. Shape reserved so it does not force a later redesign:

- A clip record owns zero or more envelopes.
- Each envelope names one durable `AutomationTargetId` and owns a clip-relative curve using the shared `CurveShape` vocabulary.
- Positions are clip-relative `MusicalTime`; values are normalized to `0.0..=1.0`; parameter metadata maps normalized to native and display units.
- An unresolved target keeps its complete descriptor and curve, stays visible and saveable, and blocks only evaluation. Resolved versus unresolved is derived at read time.
- Right-edge resize masks out-of-bound points. It never deletes or scales them.
- Arrangement automation lanes are **not** modelled here. They belong to the `automation` aggregate in project musical time.

This slice cannot start before durable parameter and device target identity exists.

## Duplication and capture contract

One rule: **immutable managed media is shared; every mutable durable fact is copied.**

| Fact | Duplicate behaviour |
| --- | --- |
| `ClipId` | New identity |
| `AssetId` and media bytes | Shared, never copied |
| Source span, gain, reverse, fades | Copied |
| Warp markers, warp mode, algorithm | Copied |
| Notes | Copied with **fresh** `NoteId` values |
| Per-note expression curves | Copied with their note |
| Clip-local envelopes | Copied; target identity preserved exactly |
| Name and colour | Copied |
| Unresolved asset or target state | Copied verbatim, never resolved or substituted |
| Window: offset, extent, loop | Copied |

Rules:

- Fresh `NoteId` values are mandatory, not incidental. Two sounding notes sharing an identity make per-note expression routing and voice identity ambiguous, which breaks MPE.
- Duplication reserves every required `NoteId` as one checked batch. A partially available batch fails before any allocator advances and before any content is mutated.
- Arrangement capture from a launcher slot is duplication under this table. Later edits cannot propagate back, because nothing mutable is shared.
- Offline audio and unresolved-target clips duplicate normally. Duplication never requires temporary resolution and never substitutes empty content.
- Linked clip instances remain deferred until propagation, unlinking, persistence, and undo are specified.

## Transaction, history, and identity contract

Bound to the accepted transaction, history, and identity contracts in `project-document/SPEC.md`.

- Content mutation is impossible outside a transaction.
- Creating a clip touches `clips` and one placement aggregate. Validation covers both before either mutates. `EffectSet` names both.
- Cross-track move touches `arrangement` only. Content is not read and cannot be disturbed.
- Cross-surface transfer is a copy under the duplication table, not a move. A `ClipId` never changes surface for its whole life. See Decision 6b.
- Delete captures the complete clip record — every note, every `NoteId`, every expression point, complete region state — in its undo payload. Undo reinserts it verbatim.
- Undo allocates nothing. Restored `NoteId` and `ClipId` values are already behind the monotonic allocator, so reinsertion cannot collide.
- Redo of a create reuses the identities recorded on first apply. It never allocates replacements.
- A rejected transaction leaves the document, history, revision, and every allocator unchanged.

New identity domains this spec requires: `NoteId` and `AssetId` allocated and enforced by the document; `TuningId` declared, with content deferred. `AutomationTargetId` is already declared.

## Load, validation, and projection contract

- Load validates the complete candidate document: every `ClipId` unique, every placement resolving to a record, every record having exactly one placement, every `NoteId` unique document-wide, every note and marker coordinate checked, every window at least one tick or frame.
- Asset and target references resolve against the current registry at read time. Unresolved is a state, not a failure. It never blocks loading, editing, or saving the rest of the project.
- Failed validation leaves the live document untouched and reports exactly what failed. There is no partial load.
- Projections are revision-stamped and read-only. Waveform caches, note grids, and warp-grid overlays are derived views with parity tests; none is a second authority.
- Content mutation is app-thread only. Nothing in this spec is traversed mutably on the audio callback. Playback consumes an immutable render generation.

## Non-goals

Deferred with their own specs: launcher slot model and launch quantization; loop and follow-action behaviour; warp playback algorithms; the expression editor; tuning tables and Scala/MTS import; arrangement automation lanes; automation evaluation precedence against realtime modulation; linked clip instances; take lanes and comping; split, left trim, and consolidate commands; UI gesture routing; recording capture paths.

## Acceptance criteria

1. Clip content lives in exactly one owner. No placement, UI model, engine mirror, or persistence DTO holds clip content once its slice completes.
2. One `ClipId` names one clip record and exactly one placement; orphan records and dangling placements are structurally rejected.
3. Cross-track move preserves `ClipId`, every `NoteId`, and every byte of content, and reports only the arrangement aggregate as changed.
4. Right-edge resize never deletes, scales, or renumbers a note, marker, or automation point; masked content round-trips exactly.
5. Left-edge trim, when it lands, moves the window and mutates no content; `content_offset` exists from the first slice.
6. Duplication produces a new `ClipId` and fresh `NoteId` values, shares `AssetId` without copying media, and copies every mutable durable fact including warp markers and expression curves.
7. Duplicating a clip whose asset or automation target is unresolved preserves the complete descriptor and the unresolved state.
8. `NoteId` is unique document-wide and is never derived from channel, position, or start time.
9. Per-note expression is routed by `NoteId`; a round trip through MPE import and export preserves note identity.
10. A note's durable pitch is a scale position plus a finite offset; changing the project tuning rewrites no note.
11. Audio source coordinates are typed frames in the asset's native sample rate and are never implicitly rate-converted or expressed in `MusicalTime`.
12. An unresolved or short asset never causes a source span to be clamped, zeroed, or defaulted.
13. Warp markers are strictly monotonic in both axes, so the source-to-musical map is invertible; adding a warp algorithm changes no region identity and no marker.
14. Toggling `WarpMode` is one explicit transaction with a defined extent conversion; a tempo-map edit alone never mutates a clip record.
15. Delete undo restores the complete clip record with identical identities, ordering, and content, allocating nothing.
16. Redo of a create restores the original `ClipId` and every original `NoteId` rather than allocating again.
17. A rejected content transaction leaves the document, history, revision, and every allocator unchanged.
18. Batch `NoteId` reservation fails atomically when the full batch is unavailable.
19. Load rejects duplicate clip identity, duplicate note identity, invalid coordinates, and zero-length windows against the candidate document, leaving live state untouched.
20. No durable clip reference depends on vector position, arena handle, or raw untyped integer.

## Slice boundaries

Each slice is one commit with its own validation. `CC1` and `CC2` together un-gate `D3`.

- **CC0 — This document.** Decisions answered; `PLAN.md` written.
- **CC1 — Content aggregate and placement split.** `geist_document::clips`; clip record with kind, name, colour; `ClipWindow` and `ClipExtent`; `arrangement` re-scoped to placement; referential invariants both directions; empty `MidiContent`. No audio, no notes, no envelopes.
- **CC2 — MIDI note model.** `NoteId` allocation and batch reservation; note record with key, offset, channel, velocities, mute; typed expression dimensions and curves; deterministic ordering; masking rules; duplication with fresh identities.
- **D3 runs here.** Arrangement placement onto the document with create, delete, move, cross-track move, and right resize as transactions, provable end to end on MIDI clips.
- **CC3 — Audio region content and warp state.** `AssetId` reference; typed source frames; gain, reverse, fades; `WarpMode`, markers, algorithm tag; unresolved and short-asset handling. **Gated on the assets aggregate landing first.**
- **CC4 — Clip-local automation envelopes.** **Gated on durable parameter and device target identity from Milestone 4.**

Ordering rationale: MIDI content depends on nothing outside `geist-core` and an empty MIDI clip is a complete user object, so `D3` becomes provable at the earliest possible point. Audio content depends on the asset registry. Envelopes depend on target identity that does not yet exist.

### Prerequisites outside this spec

- D1 relocation complete: `MusicalTime` in `geist-core`, arrangement in `geist-document`.
- D2 complete: document, revision, transactions, history, allocators.
- `CurveShape` relocated from `geist-automation` to `geist-core`, per Milestone 1's removal of the timeline-to-automation dependency. This spec's expression curves and envelopes both consume it.
- The assets aggregate must land before CC3. See Decision 2.

## Planning status

**No `PLAN.md` accompanies this draft.** Seven open decisions change the shape of `CC1`, the first slice, so any plan written now would be fiction:

| Blocking decision | Slice it reshapes |
| --- | --- |
| 1 — content ownership model | CC1 entirely |
| 2 — slice order and asset-registry position | CC3 gating |
| 4a — `ClipExtent` typing | CC1 window model |
| 5a — pitch and tuning model | CC2 note record |
| 6a — lane overlap and clip ordering | CC1 placement, and the existing `rehome_clip` signature |
| 6b — cross-surface copy versus move | CC1 invariants |
| 7 — automation as a clip kind | CC1 `ClipKind` |

Decisions 3, 4b, 4c, 5b, 5c, and 6c refine slices without changing their boundaries and can be settled during implementation review if the owner prefers.

`PLAN.md` is written once the blocking rows are answered.

## Stop conditions

Stop before the next slice when:

- a blocking decision above is unanswered;
- a slice would place clip content in more than one owner;
- an edit would delete, scale, or renumber content that resize or trim should mask;
- a note or region coordinate would depend on vector position, arena handle, or raw untyped integer;
- undo cannot restore exact clip and note identity without allocating;
- an audio region would carry project-sample or musical source coordinates instead of native frames;
- audio content would ship before an asset registry that can resolve it;
- independent review has unresolved findings.

---

## Decisions required

Seven decisions. Each lists concrete options, tradeoffs, and one recommendation. The owner decides; this draft does not.

### Decision 1 — Where clip content lives

**Question.** Does content live directly on the clip entity, or in a separately owned content aggregate?

**Option A — Content inline on `ClipEntity`.** The paused Slice B2 proposal. `ClipEntity { id, owner: TrackId, start, duration, content }`.

- For: one type, one map, no referential invariant to enforce, smallest diff from today.
- Against: content is welded to arrangement placement. The launcher needs a parallel clip type with duplicated content code, duplicated editors, and duplicated persistence. Capture becomes a cross-type conversion. Cross-track move reads and rewrites content, so a move bug can lose notes. Linked instances later require redesigning the entity.

**Option B — Separate `clips` aggregate keyed by `ClipId`; placements reference it.** *(This draft's proposal.)*

- For: one clip record serves both surfaces. Editors, persistence, and duplication address content by `ClipId` regardless of surface. Cross-track move provably cannot touch content. Capture is a copy, so independence is structural. A later `ContentId` layer re-keys one map instead of redesigning entities.
- Against: two aggregates must stay referentially consistent, and clip creation spans both. Amends the accepted aggregate table.

**Option C — Source/instance split now: `ContentId` content records, many clips referencing one.**

- For: linked clips need no migration later.
- Against: the vision explicitly defers linked-content semantics until propagation, unlinking, persistence, and undo are specified, and the roadmap warns against introducing implicit shared mutable editing. Ships an indirection with no user-visible feature, and every edit path must immediately answer "does this mutate shared content?" — the exact question the deferral exists to avoid.

**Recommendation: Option B.** It is the only option that gives the launcher clips without a parallel type, and it keeps Option C reachable as an internal re-key rather than a redesign. Mandate that all content access goes through a document accessor keyed by `ClipId`, never direct map indexing, so the later change stays contained.

**Sub-decision 1a — amending the accepted spec.** Option B adds a `clips` row to the `project-document/SPEC.md` aggregate table and re-scopes `arrangement` to placement. Approve the amendment, or reject it and nest content inside `arrangement` — which would make the launcher aggregate reach into the arrangement aggregate for its own clips. **Recommendation: approve the amendment.**

### Decision 2 — Payload slice order

**Question.** Do audio region state, MIDI notes, and automation payloads land together or as separate sub-slices, and in what order?

**Option A — All three together in one slice.** The paused B2 scope.

- For: one migration, one persistence change.
- Against: audio needs the asset registry and automation needs durable target identity, neither of which exists. The slice blocks on both, so `D3` stays blocked on both. This is what made B2 unlandable.

**Option B — MIDI, then audio, then automation.** *(This draft's proposal.)*

- For: MIDI depends on nothing outside `geist-core`, and an empty MIDI clip is a complete user object, so `D3` becomes provable at the earliest point. Audio waits only for the asset registry. Automation waits for target identity.
- Against: the app's real content today is recorded audio takes, so the legacy audio path stays authoritative longer than the legacy MIDI path.

**Option C — Audio first, matching what the app actually plays.**

- For: shortens the legacy-audio compatibility window.
- Against: pulls the asset registry, warp, and source-coordinate typing into the slice that un-gates `D3`. Largest, riskiest first slice.

**Recommendation: Option B**, with a plan change: **move the assets aggregate sub-slice ahead of CC3.** `project-document/PLAN.md` currently orders D7 as tracks and devices, assets, conductor, mappings, after D3–D6. Audio clip content must not ship against a registry that cannot resolve it, because an always-unresolved audio clip is exactly the "simulated behaviour" the vision forbids.

### Decision 3 — What duplication copies versus shares

**Question.** Confirm or revise the prior proposal: audio shares an immutable `AssetId` while MIDI deep-copies with fresh `NoteId` values.

**Verified against the vision. Both hold, with four revisions.**

- *Audio shares `AssetId`.* **Confirmed.** "Audio regions reference immutable managed assets and own their region-local edit state." Sharing immutable media is not shared mutable state.
- *MIDI deep-copies with fresh `NoteId`.* **Confirmed, with a stronger reason than the prior spec gave.** The prior spec justified it as "notes are not linked between duplicate clips." The real reason is MPE: per-note expression and voice identity are routed by `NoteId`, so two sounding notes sharing an identity make routing ambiguous. Fresh identities are a correctness requirement, not a policy choice.

Revisions to the prior proposal:

1. **Warp markers, fades, and reverse are region-local and copied.** The prior proposal deferred them entirely, leaving their copy semantics undefined.
2. **Expression curves travel with their note.** Not mentioned previously; without this, duplication silently drops MPE data.
3. **`NoteId` uniqueness widens from arrangement-scope to document-scope.** Launcher clips share the domain, and the document owns the allocator.
4. **Unresolved automation targets are preserved by identity, not relinked.** The prior spec covered offline audio but not target identity across duplication.

**Alternative the owner may prefer: copy-on-write content sharing for duplicates.** Cheaper duplication of large clips, but it reintroduces exactly the implicit shared mutable editing the roadmap warns against and makes "did this edit affect the other clip?" unanswerable without reading internals. **Recommendation: reject; keep eager deep copy.** Clip content is small; media is the large thing, and media is already shared.

**Recommendation: confirm the prior rule, adopt all four revisions.**

### Decision 4 — Warp, tempo interpretation, and extent

**Question.** How do non-destructive warp markers, tempo interpretation, and stretch mode fit without forcing a later redesign?

**Framing.** Warp has two separable halves: the durable **representation** (a coordinate map plus an algorithm tag) and the **rendering** (repitch, beat-preserving, later spectral). The vision says advanced modes may arrive later "without changing region identity." That is only true if the representation lands with the audio region and the algorithm is a tag over an unchanged map. Representation must land in CC3; rendering may lag to Milestone 9.

#### Sub-decision 4a — durable extent of an unwarped audio clip

Today `ClipEntity.duration` is `MusicalTime`. An unwarped audio region has a fixed length in *seconds*, which is not a fixed length in bars.

- **Option A — extent is always `MusicalTime`; tempo-map edits rescale unwarped clips.** Keeps one placement model, but makes a conductor edit mutate every unwarped clip record. Undo of a tempo change becomes an undo of N clip edits. Rejected as a history and transaction hazard.
- **Option B — typed `ClipExtent = Musical | Source`.** *(Proposed.)* Unwarped audio keeps a sample-domain durable length; warped audio and MIDI keep musical length. Tempo edits mutate no clip. Costs a conductor resolution step in layout and playback, which those paths already need. Directly satisfies invariant 8's "sample-domain source coordinates remain explicit."
- **Option C — extent is always `MusicalTime`; tempo change is absorbed by masking.** Nothing is destroyed, but an unwarped clip's audio drifts away from its own right edge as tempo changes, and the clip fills with silence. Surprising, and unlike any DAW in the category.

**Recommendation: Option B.** Note the consequence: `ClipEntity.duration: MusicalTime` as shipped today changes type in CC1.

#### Sub-decision 4b — how many warp mechanisms

- **Option A — `WarpMode::Off | On { markers, algorithm }`, where a plain stretch is exactly two markers.** *(Proposed.)* One mapping mechanism. Source tempo is derived for display, never stored twice.
- **Option B — warp flag plus a stored clip tempo plus an optional marker list**, as Live models it. Familiar, but three overlapping sources of the same mapping truth, and the classic source of "which one wins" bugs.

**Recommendation: Option A.**

#### Sub-decision 4c — fade coordinate domain

- **Option A — fades in clip-relative `MusicalTime`.** Consistent with the window, masking, and resize; fades stay musically placed under warping. A tempo change alters their audible length.
- **Option B — fades in source frames or absolute time.** Audibly stable, correct for de-click and edit crossfades, but a second coordinate domain inside the window and awkward under warp.

**Recommendation: Option A** for the first slice, with a reserved per-fade domain tag if edit crossfades later need Option B semantics. Low confidence; this one is genuinely close.

### Decision 5 — MPE and tuning in the note model

**Question.** How do MPE per-note expression and non-12-TET tuning fit from the start?

#### Sub-decision 5a — durable pitch

- **Option A — `key: u8` only.** Today's model. Twelve-tone equal temperament becomes a schema limitation, which the vision explicitly forbids.
- **Option B — `NoteKey` plus finite `PitchOffset` semitones; tuning systems at project and device scope under `TuningId`.** *(Proposed.)* Notes stay tuning-agnostic, so switching a tuning rewrites nothing and transposition still means something. MPE per-note bend is a separate continuous expression dimension, distinct from static detune.
- **Option C — store frequency per note.** Loses scale-degree identity; transposition, tuning switching, and scale-aware editing all break. Rejected.
- **Option D — store `(scale_degree, TuningId)` per note.** Maximum fidelity, but puts tuning resolution on every note and forces a tuning sub-spec into CC2.

**Recommendation: Option B.** Declare `TuningId` now with content deferred, so the reference exists and resolves to 12-TET by default.

#### Sub-decision 5b — expression dimension set

- **Option A — MPE three: per-note pitch, pressure, timbre.** Smallest, covers MPE controllers.
- **Option B — the full plugin note-expression set: pitch/tuning, volume, pan, vibrato, expression, brightness, pressure.** *(Proposed.)* Lossless round trip with hosted plugins and with the first-party instruments, which is the interoperability the vision requires. Costs a wider enum and more editor surface later.

**Recommendation: Option B.** The enum is cheap now and lossy interop is expensive to unwind. Either way the dimension is a typed enum, never a free-form string.

#### Sub-decision 5c — expression time base and breakpoint identity

- **Time base — note-relative** *(proposed)*: expression travels intact when a note moves or is copied. **Clip-relative** alternative: simpler to draw against the clip grid, but moving a note silently detaches its expression. **Recommendation: note-relative.**
- **Breakpoint identity — positional key** *(proposed)*: at most one point per dimension per tick; setting an existing position replaces its value and outgoing shape. Position is a durable coordinate, not a vector index, so this satisfies the identity constraint. **Alternative:** allocate a `BreakpointId` domain for stable point identity under multi-point drags. **Recommendation: positional keying**, with the same rule applied to clip-local automation points, unless the owner wants multi-point gestures to preserve point identity across a drag — in which case decide now, because retrofitting point identity means a persistence migration.

### Decision 6 — Cross-track move, ordering, and undo identity

**Question.** Cross-track move and undo identity semantics, checked against the transaction and history contracts.

Checked and consistent: cross-track move preserves `ClipId`; content is untouched because it lives in a different aggregate; undo restores exact identity, ordering, start, and content while allocating nothing; redo reuses recorded identities. Three sub-decisions remain.

#### Sub-decision 6a — overlap and clip ordering on an arrangement lane

Today `ArrangementTrack` holds a durable `Vec<ClipId>` and `rehome_clip` takes an explicit index, which permits two clips at the same start on one track with no defined playback arbitration.

- **Option A — forbid overlap on a lane; order derived from start.** *(Recommended.)* The aggregate gains a strong invariant, playback arbitration is unambiguous, and the index parameter disappears along with a class of ordering bugs. "Drop a clip over an existing one and trim it" becomes an explicit composite transaction in the editing milestone. Take lanes arrive later as additive sub-lanes without breaking the invariant.
- **Option B — permit overlap; keep durable order.** Matches the code as written and defers the collision-resolution edit, but forces this spec to define render arbitration for overlapping audio on one lane — a scheduler decision made by accident.

**Recommendation: Option A.** Note the consequence: `rehome_clip`, `ClipLocation`, and `RemovedClip` change shape in CC1/D3.

#### Sub-decision 6b — cross-surface transfer

- **Option A — copy only.** *(Recommended.)* A `ClipId` never changes surface, so "one clip, one placement, one surface, for life" is a provable invariant. Matches the vision's "arrangement capture creates independent clip entities" without exception.
- **Option B — true move between arrangement and launcher.** Saves an identity on drag-out, but weakens the invariant to "one placement in one of two aggregates" and makes undo of a cross-surface move span two placement models plus an extent conversion.

**Recommendation: Option A.**

#### Sub-decision 6c — playback of a note that overruns the window

- **Option A — force note-off at the window boundary; durable duration unchanged.** *(Recommended.)* Matches the category's behaviour and keeps the non-destructive rule.
- **Option B — let the note ring past the clip edge.** Musically useful for tails, but means a clip can sound after its own end, which complicates launcher stop, arrangement arbitration, and voice accounting.

**Recommendation: Option A**, with Option B reachable later as a per-clip flag rather than a model change.

### Decision 7 — Automation as a clip kind *(raised by this draft)*

**Question.** The paused proposal made automation a third `ClipKind` with a typed target and a clip-relative curve. The accepted vision instead names three distinct parameter-control layers that "do not share one undifferentiated data model": arrangement automation in project musical time, clip-local automation in clip-relative time, and realtime modulation.

- **Option A — keep `ClipKind::Automation`.** Preserves the prior proposal and the current `geist_timeline::Clip` shape. But it collapses layers one and two into one representation, and it makes arrangement automation inherit clip placement, ordering, duplication, and launcher-slot semantics that arrangement automation does not want.
- **Option B — remove it.** *(Proposed.)* Arrangement automation becomes lanes in the `automation` aggregate keyed by durable target in project time. Clip-local automation becomes envelopes owned by a clip record in clip-relative time. `ClipKind` is `Audio | Midi`. This matches both the vision's layering and the Live/Bitwig category the product targets.

**Recommendation: Option B.** It also retires the persisted `ClipKind::Automation { lane_index: usize }`, a vector index used as a durable reference, which has to go regardless.
