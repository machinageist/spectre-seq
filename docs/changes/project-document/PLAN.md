<!--
Author: Jeff
Date: 2026-08-01
Description: Phased implementation plan for canonical ProjectDocument authority.
Notes: Execute and commit one independently validated slice at a time; blocking checkpoints are marked.
-->

# ProjectDocument Implementation Plan

## Preconditions

- Product and architecture contract: `SPEC.md`.
- Migration is a strangler; the app stays runnable after every slice.
- `ProjectDocument` lives in `crates/geist-document` and depends only on `spectre-core`.
- Publication is hybrid: immutable versioned render generations plus a bounded acknowledged control stream.
- App-thread ownership and realtime constraints remain mandatory.
- Legacy authorities are deleted only when all four deletion criteria in `SPEC.md` hold for their domain.

## Slice D1 — Crate skeleton and vocabulary relocation

**Landed 2026-08-03.** Covers acceptance criterion 11. No behavior change; this slice only moves code and adds an empty crate. Workspace tests: 646 passing before and after.

Files:

- Add `crates/geist-document/Cargo.toml`.
- Add `crates/geist-document/src/lib.rs`.
- Add `crates/spectre-core/src/time.rs`.
- Update `crates/spectre-core/src/lib.rs`.
- Move `crates/geist-timeline/src/time.rs` into `spectre-core`.
- Move `crates/geist-timeline/src/identity.rs` into `geist-document`.
- Move `crates/geist-timeline/src/arrangement.rs` into `geist-document`.
- Update `crates/geist-timeline/src/lib.rs`.
- Update `crates/geist-timeline/Cargo.toml`.
- Update `PROPOSED_FILE_TREE.md` after the move verifies.
- Update this plan after verification.

Tasks:

1. [x] Create `geist-document` with `#![deny(unsafe_code)]` and `spectre-core` as its only dependency. The workspace `members` glob `crates/*` picks it up, so the root manifest needs no edit. Write a real manifest header matching `crates/geist-timeline/Cargo.toml`, not the pseudocode-scaffold header older manifests carry.
2. [x] Move `MusicalTime`, `TICKS_PER_QUARTER`, `MAX_EXACT_MUSICAL_TIME_TICKS`, and their tests from `geist-timeline` to `spectre-core::time`; add them to the `spectre-core` prelude.
3. [x] Move the `IdSequence` and `IdentityAllocator` machinery to `geist-document`. `ClipId` and `TrackId` already live in `spectre-core::ids` and stay there; `crates/geist-timeline/src/identity.rs:14` only re-exports them.
4. [x] Move `Arrangement`, `ClipEntity`, `ArrangementTrack`, `ClipLocation`, `RemovedClip`, and `ArrangementError` to `geist-document::arrangement` with their tests.
5. [x] Re-export every moved item from `geist_timeline::prelude` for the compatibility window so no consumer breaks in this slice.
6. [x] Add the durable nonzero ID family to `spectre-core::ids` — `AssetId`, `SceneId`, `DeviceId`, `ParamKey`, `RouteId`, `NoteId`, `MappingId`, `AutomationTargetId`. Scope amendment: without it, D2 and the Wave 2 vocabulary slice both edit `crates/spectre-core/src/lib.rs` concurrently. The `define_nonzero_id!` macro already exists, so this adds no behavior.
7. [x] Confirm the workspace test count did not drop; every moved test runs at its new home.
8. [x] Resolve independent spec and code review findings.
9. [x] Commit only after all findings are resolved.

Verification:

- `cargo test -p spectre-core`
- `cargo test -p geist-document`
- `cargo test -p geist-timeline`
- `cargo check --workspace`
- `cargo test --workspace`
- `git diff --check`

Acceptance: no observable behavior changed anywhere in the workspace.

## Slice D2 — Document skeleton, revision, transactions, history

Covers acceptance criteria 2, 3, and 4.

Expected outcomes:

- `ProjectDocument` owning typed aggregates, empty where their content sub-spec has not landed.
- `DocumentRevision` advancing by exactly one per accepted transaction.
- `TransactionResult` as `Accepted { revision, effects }` or `Rejected { reason }`, with rejection reasons distinguishing stale identity, missing owner, duplicate identity, invalid coordinate, invalid duration, unresolved reference, and identity exhaustion.
- `EffectSet` naming which aggregates changed.
- One project-level history with the failure semantics in `SPEC.md`.
- Document-owned non-cloneable allocators for every identity domain listed in `SPEC.md`, including domains whose aggregates are still empty.
- Dirty state derived from the revision last saved.

Tests must prove that a rejected transaction leaves document, history, revision, and every allocator unchanged, and that batch reservation fails atomically when the batch is not fully available.

## Slice D3 — Arrangement domain onto the document

Covers acceptance criteria 5 and 6.

**Un-gated 2026-08-03.** The checkpoint that blocked this slice is resolved by `docs/changes/canonical-clip-content/SPEC.md`, accepted the same day. Its three questions were answered as:

- Clip content lives in a **separate `clips` aggregate** keyed by `ClipId`; placements reference it (decision 1). The aggregate table above is amended accordingly.
- Payloads land as **MIDI, then audio** (decision 2), with the assets aggregate pulled ahead of the audio slice. Automation is no longer a clip payload at all (decision 7).
- Cross-track move preserves `ClipId` and cannot touch content, because content is in a different aggregate. Cross-surface transfer is copy-only (decision 6b).

**This slice is written up as CC1 in `docs/changes/canonical-clip-content/PLAN.md`.** The two documents describe one slice from two sides; execute it from there, since that is where the content decisions live.

Expected outcomes:

- The canonical arrangement under document ownership, mutated only through transactions.
- Create, delete, move, cross-track move, and right resize as transactions.
- Exact undo and redo restoration of identity, ordering, start, and extent.
- No two clips overlapping on one arrangement lane (decision 6a); clip order derived from start rather than stored, so `rehome_clip`, `ClipLocation`, and `RemovedClip` change shape.

This un-pauses canonical-clip-commands Slice C with the correct owner.

## Slice D4 — Projection contract and arrangement UI projection

Covers acceptance criteria 7 and 15.

Expected outcomes:

- Revision-stamped read-only projections rebuilt from the document.
- `geist_ui::model::TimelineModel` demoted to a projection; the UI stops mutating durable state.
- Typed arrangement intents replacing direct UI mutation.
- Parity tests proving the projection reproduces the document facts it shows.
- Stale projections rebuilt or visibly marked, never silently trusted.
- Controls with no command behind them disabled or explained rather than simulated.

Provisional drag feedback stays disposable local state; pointer release submits one validated transaction.

## Slice D5 — Publication protocol

**Un-gated 2026-08-03.** The render-generation content boundary is settled by `docs/changes/typed-realtime-graph/SPEC.md`, accepted the same day. A generation carries a `RenderPlan`, its typed per-domain arenas, a sorted route index, and plan latency — **but not device instances**, which live in an audio-thread-owned `DeviceTable` (decision 8, ADR 005). Adoption gains one precondition: a generation declares `requires_control_sequence` and is adopted only once the audio thread has applied it.

Reuse `crates/spectre-graph/src/swap.rs` rather than inventing a second mechanism; it already refuses to drop on the callback. This slice is the same boundary as that plan's slice T2 and should be reviewed jointly with it.

Covers acceptance criteria 8, 9, and 10.

Expected outcomes:

- `RenderGeneration` built entirely on the app thread, immutable, identified by a monotonic `GenerationId`, published by a swap the audio thread cannot block on.
- The audio thread publishing the generation it is executing and the highest control sequence it has applied.
- A bounded timestamped control stream carrying sequence numbers and within-block sample offsets.
- Every send result consumed; the ignored saturation bool at `app/geist-daw/src/control.rs:225` removed from the path.
- Explicit reconciliation on saturation or rejected publication; the app never advances mirrors past acknowledgement.
- A non-realtime reclaim queue for retired generations, plugin instances, and assets.

Validation: allocator guards in debug builds, overload tests, graph-swap stress tests, and callback benchmarks against the 48 kHz / 128-frame baseline and the 64-frame stress mode.

## Slice D6 — Persistence projection

Covers acceptance criteria 6, 13, and 14.

Expected outcomes:

- `spectre-project` depending on `geist-document` and serializing a projection of the document.
- Versioned schema with migration fixtures.
- Atomic load: validate the complete candidate document, then replace the live document wholesale or leave it untouched and report exactly what failed.
- Project package layout with canonical manifest and managed subdirectories.
- Unresolved references round-tripping losslessly.

Pre-v1 prototype format breaks are permitted with explicit diagnostics and fixture-tested conversion. A load never reports success after discarding durable state.

## Slice D7 — Remaining domains

Each domain is its own sub-slice and each needs its content sub-spec before it starts.

**Order amended 2026-08-03.** The assets aggregate moves ahead of the audio clip-content slice, per `canonical-clip-content/SPEC.md` decision 2, where it appears as slice CC2a. Audio clip content must not ship against a registry that cannot resolve it, because an always-unresolved audio clip is exactly the simulated behavior invariant 11 forbids. Remaining order: assets (early, before CC3), then tracks and devices, conductor, mappings.

Every sub-slice enforces the unresolved-reference contract for its own domain, and none begins before the corresponding roadmap milestone work it depends on.

## Slice D8 — Delete legacy authorities

Covers acceptance criteria 1 and 12.

Delete only when all four deletion criteria in `SPEC.md` hold for the domain:

- `app::session::StudioSession`;
- `app::engine::Arrangement` as an authority;
- `geist_timeline::Timeline` and the legacy arena placement path;
- UI-owned durable state;
- the `geist_timeline::prelude` compatibility re-exports added in D1.

Update `docs/architecture.md`, `docs/realtime_rules.md`, and `PROPOSED_FILE_TREE.md` to implementation truth in this slice.

## Stop conditions

Stop before the next slice when:

- A domain's content sub-spec is missing.
- A blocking design checkpoint above is unresolved.
- A migration would change persistence without fixtures.
- Publication would allocate, deallocate, or lock on the callback.
- Undo cannot restore exact identity and state.
- Validation or independent review has unresolved findings.
