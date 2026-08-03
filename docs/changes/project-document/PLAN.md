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
- `ProjectDocument` lives in `crates/geist-document` and depends only on `geist-core`.
- Publication is hybrid: immutable versioned render generations plus a bounded acknowledged control stream.
- App-thread ownership and realtime constraints remain mandatory.
- Legacy authorities are deleted only when all four deletion criteria in `SPEC.md` hold for their domain.

## Slice D1 — Crate skeleton and vocabulary relocation

Covers acceptance criterion 11. No behavior change; this slice only moves code and adds an empty crate.

Files:

- Add `crates/geist-document/Cargo.toml`.
- Add `crates/geist-document/src/lib.rs`.
- Add `crates/geist-core/src/time.rs`.
- Update `crates/geist-core/src/lib.rs`.
- Move `crates/geist-timeline/src/time.rs` into `geist-core`.
- Move `crates/geist-timeline/src/identity.rs` into `geist-document`.
- Move `crates/geist-timeline/src/arrangement.rs` into `geist-document`.
- Update `crates/geist-timeline/src/lib.rs`.
- Update `crates/geist-timeline/Cargo.toml`.
- Update `PROPOSED_FILE_TREE.md` after the move verifies.
- Update this plan after verification.

Tasks:

1. [ ] Create `geist-document` with `#![deny(unsafe_code)]` and `geist-core` as its only dependency. The workspace `members` glob `crates/*` picks it up, so the root manifest needs no edit. Write a real manifest header matching `crates/geist-timeline/Cargo.toml`, not the pseudocode-scaffold header older manifests carry.
2. [ ] Move `MusicalTime`, `TICKS_PER_QUARTER`, `MAX_EXACT_MUSICAL_TIME_TICKS`, and their tests from `geist-timeline` to `geist-core::time`; add them to the `geist-core` prelude.
3. [ ] Move the `IdSequence` and `IdentityAllocator` machinery to `geist-document`. `ClipId` and `TrackId` already live in `geist-core::ids` and stay there; `crates/geist-timeline/src/identity.rs:14` only re-exports them.
4. [ ] Move `Arrangement`, `ClipEntity`, `ArrangementTrack`, `ClipLocation`, `RemovedClip`, and `ArrangementError` to `geist-document::arrangement` with their tests.
5. [ ] Re-export every moved item from `geist_timeline::prelude` for the compatibility window so no consumer breaks in this slice.
6. [ ] Confirm the workspace test count did not drop; every moved test runs at its new home.
7. [ ] Resolve independent spec and code review findings.
8. [ ] Commit and push only after all findings are resolved.

Verification:

- `cargo test -p geist-core`
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

Blocking design checkpoint. Do not start until the clip-content ownership question that paused `docs/changes/canonical-clip-commands/` Slice B2 is resolved. `SPEC.md` fixes the owner; it does not decide the payload.

Questions to resolve:

- Whether clip content lives on the clip entity or in a separately owned content aggregate.
- Whether audio region state, MIDI notes, and automation payloads arrive together or in separate sub-slices.
- Cross-track move and undo identity semantics, confirmed against the D2 history contract.

Expected outcomes:

- The canonical arrangement under document ownership, mutated only through transactions.
- Create, delete, move, cross-track move, and right resize as transactions.
- Exact undo and redo restoration of identity, ordering, start, and duration.

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

Blocking design checkpoint. The render-generation *content* boundary must be settled against Milestone 3 first. `SPEC.md` fixes the protocol and explicitly leaves generation content as whatever the engine needs today.

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

- `geist-project` depending on `geist-document` and serializing a projection of the document.
- Versioned schema with migration fixtures.
- Atomic load: validate the complete candidate document, then replace the live document wholesale or leave it untouched and report exactly what failed.
- Project package layout with canonical manifest and managed subdirectories.
- Unresolved references round-tripping losslessly.

Pre-v1 prototype format breaks are permitted with explicit diagnostics and fixture-tested conversion. A load never reports success after discarding durable state.

## Slice D7 — Remaining domains

Each domain is its own sub-slice and each needs its content sub-spec before it starts. Order: tracks and devices, assets, conductor, mappings.

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
